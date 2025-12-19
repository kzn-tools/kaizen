#!/bin/bash
# install-review.sh
# Installe le système de review Claude Code pour Lynx

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}🔧 Installation du système de review Claude Code pour Lynx${NC}"
echo

# Check we're in a git repo
if [[ ! -d .git ]]; then
    echo "❌ Ce script doit être exécuté à la racine du repo Lynx"
    exit 1
fi

# Create directories
mkdir -p .claude
mkdir -p scripts/hooks

# Copy files
echo -e "${BLUE}📁 Copie des fichiers...${NC}"

# Review rules
cat > .claude/review-rules.md << 'EOF'
# Lynx Code Review Rules

## 🔴 Critiques (bloquantes)

### 1. Pas de panic dans le code library
- Interdit: `unwrap()`, `expect()`, `panic!()`, `unreachable!()`
- Autorisé uniquement dans: tests, lynx-cli, assertions de debug
- Alternative: `?`, `.ok_or()`, `.map_err()`, `anyhow!`

### 2. I/O async obligatoire dans lynx-lsp
- Interdit: `std::fs::*`, `std::io::*`
- Requis: `tokio::fs::*`, `tokio::io::*`

### 3. Sécurité
- Valider tous les chemins de fichiers
- Pas de path traversal

## 🟡 Important

### Performance
- Éviter `.clone()` dans les boucles
- Préférer `&str` à `String`
- Utiliser `Cow<str>` si ownership incertain

### Architecture
- lynx-core: aucun I/O, logique pure
- lynx-lsp: handlers minces, déléguer à core
- lynx-cli: interactions utilisateur

### Qualité
- Doc comments sur APIs publiques
- Tests pour nouvelles règles
- Messages d'erreur actionnables
EOF

echo "  ✓ .claude/review-rules.md"

# Review script
cat > scripts/review-pr.sh << 'SCRIPT'
#!/bin/bash
# Review de PR avec Claude Code
set -euo pipefail

BASE="${BASE_BRANCH:-main}"
TARGET="${1:-}"

if ! command -v claude &> /dev/null; then
    echo "❌ Claude Code requis: npm install -g @anthropic-ai/claude-code"
    exit 1
fi

if [[ -n "$TARGET" && "$TARGET" =~ ^[0-9]+$ ]] && command -v gh &> /dev/null; then
    DIFF=$(gh pr diff "$TARGET")
    FILES=$(gh pr view "$TARGET" --json files -q '.files[].path' | grep '\.rs$' || true)
else
    git fetch origin "$BASE" --quiet 2>/dev/null || true
    DIFF=$(git diff "origin/$BASE...HEAD")
    FILES=$(git diff --name-only "origin/$BASE...HEAD" | grep '\.rs$' || true)
fi

[[ -z "$DIFF" ]] && { echo "✓ Rien à reviewer"; exit 0; }

claude -p "Review Rust pour Lynx. Règles: pas de unwrap/expect dans lib, tokio::fs dans LSP.

Fichiers: $FILES

\`\`\`diff
$DIFF
\`\`\`

Format: 🔴 Critiques, 🟡 Suggestions, 🟢 Positif, puis VERDICT [APPROVE/REQUEST_CHANGES]"
SCRIPT

chmod +x scripts/review-pr.sh
echo "  ✓ scripts/review-pr.sh"

# Pre-push hook
cat > scripts/hooks/pre-push << 'HOOK'
#!/bin/bash
# Quick review avant push
command -v claude &> /dev/null || exit 0

while read local_ref local_sha remote_ref remote_sha; do
    [[ "$local_sha" == "0000000000000000000000000000000000000000" ]] && continue
    
    if [[ "$remote_sha" == "0000000000000000000000000000000000000000" ]]; then
        files=$(git diff --name-only main...HEAD | grep '\.rs$' || true)
        diff=$(git diff main...HEAD -- $files 2>/dev/null || true)
    else
        files=$(git diff --name-only "$remote_sha...$local_sha" | grep '\.rs$' || true)
        diff=$(git diff "$remote_sha...$local_sha" -- $files 2>/dev/null || true)
    fi
    
    [[ -z "$files" ]] && exit 0
    
    echo "🔍 Quick review..."
    result=$(claude -p "Vérifie UNIQUEMENT: unwrap/expect dans lib, std::fs dans LSP. Diff:\n$diff\n\nRéponds: OK ou PROBLÈME: <description>" 2>/dev/null || echo "OK")
    
    if echo "$result" | grep -qi "PROBLÈME"; then
        echo "❌ $result"
        echo "Utilise --no-verify pour forcer"
        exit 1
    fi
    echo "✓ OK"
done
HOOK

chmod +x scripts/hooks/pre-push
echo "  ✓ scripts/hooks/pre-push"

# CLAUDE.md
if [[ ! -f CLAUDE.md ]]; then
    cat > CLAUDE.md << 'CLAUDEMD'
# CLAUDE.md - Lynx Guidelines

## Architecture
- `lynx-core`: Analyse pure, pas d'I/O
- `lynx-lsp`: LSP async (tokio uniquement)  
- `lynx-cli`: Interface utilisateur

## Règles critiques
1. Pas de `unwrap()`/`expect()` dans core/lsp
2. Pas de `std::fs` dans lsp (utiliser `tokio::fs`)
3. Tests requis pour nouvelles règles

## Commandes
```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
./scripts/review-pr.sh
```
CLAUDEMD
    echo "  ✓ CLAUDE.md"
fi

# Install git hook
echo
read -p "Installer le hook pre-push? [y/N] " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    cp scripts/hooks/pre-push .git/hooks/
    echo -e "  ${GREEN}✓ Hook installé${NC}"
fi

echo
echo -e "${GREEN}✅ Installation terminée!${NC}"
echo
echo "Usage:"
echo "  ./scripts/review-pr.sh         # Review branche courante"
echo "  ./scripts/review-pr.sh 42      # Review PR #42"
echo "  git push                        # Auto-review avant push"
