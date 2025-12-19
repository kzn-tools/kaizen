#!/bin/bash
# scripts/review-pr.sh
# Review de PR/branches avec Claude Code (Pro Max)
#
# Usage:
#   ./scripts/review-pr.sh              # Review branche courante vs main
#   ./scripts/review-pr.sh 42           # Review PR #42 (nécessite gh CLI)
#   ./scripts/review-pr.sh feature      # Review branche feature vs main
#   ./scripts/review-pr.sh --staged     # Review uniquement les fichiers stagés
#   ./scripts/review-pr.sh --post 42    # Review et poste sur PR #42

set -euo pipefail

RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Parse arguments
POST_TO_PR=""
STAGED_ONLY=false
TARGET=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --post)
            POST_TO_PR="$2"
            shift 2
            ;;
        --staged)
            STAGED_ONLY=true
            shift
            ;;
        *)
            TARGET="$1"
            shift
            ;;
    esac
done

# Check claude CLI
if ! command -v claude &> /dev/null; then
    echo -e "${RED}❌ Claude Code non trouvé.${NC}"
    echo "Installe-le avec: npm install -g @anthropic-ai/claude-code"
    echo "Puis authentifie-toi: claude"
    exit 1
fi

BASE_BRANCH="${BASE_BRANCH:-main}"

# Get the diff based on mode
if [[ "$STAGED_ONLY" == true ]]; then
    echo -e "${BLUE}📋 Review des fichiers stagés...${NC}"
    DIFF=$(git diff --cached)
    CHANGED_FILES=$(git diff --cached --name-only | grep '\.rs$' || true)
elif [[ -n "$TARGET" && "$TARGET" =~ ^[0-9]+$ ]]; then
    if ! command -v gh &> /dev/null; then
        echo -e "${RED}❌ gh CLI requis pour les PR. Install: https://cli.github.com${NC}"
        exit 1
    fi
    echo -e "${BLUE}📋 Review de la PR #$TARGET...${NC}"
    DIFF=$(gh pr diff "$TARGET")
    CHANGED_FILES=$(gh pr view "$TARGET" --json files -q '.files[].path' | grep '\.rs$' || true)
elif [[ -n "$TARGET" ]]; then
    echo -e "${BLUE}📋 Review de la branche '$TARGET' vs '$BASE_BRANCH'...${NC}"
    git fetch origin "$BASE_BRANCH" --quiet 2>/dev/null || true
    DIFF=$(git diff "origin/$BASE_BRANCH...$TARGET")
    CHANGED_FILES=$(git diff --name-only "origin/$BASE_BRANCH...$TARGET" | grep '\.rs$' || true)
else
    CURRENT=$(git branch --show-current)
    echo -e "${BLUE}📋 Review de '$CURRENT' vs '$BASE_BRANCH'...${NC}"
    git fetch origin "$BASE_BRANCH" --quiet 2>/dev/null || true
    DIFF=$(git diff "origin/$BASE_BRANCH...HEAD")
    CHANGED_FILES=$(git diff --name-only "origin/$BASE_BRANCH...HEAD" | grep '\.rs$' || true)
fi

if [[ -z "$DIFF" ]]; then
    echo -e "${GREEN}✓ Aucun changement à reviewer.${NC}"
    exit 0
fi

if [[ -z "$CHANGED_FILES" ]]; then
    echo -e "${YELLOW}⚠ Aucun fichier .rs modifié.${NC}"
    exit 0
fi

DIFF_LINES=$(echo "$DIFF" | wc -l)
FILE_COUNT=$(echo "$CHANGED_FILES" | wc -w)
echo -e "${CYAN}  $FILE_COUNT fichier(s), $DIFF_LINES lignes de diff${NC}"
echo -e "${CYAN}  Fichiers: $CHANGED_FILES${NC}"
echo

# Truncate diff if too large
MAX_LINES=2500
if [[ $DIFF_LINES -gt $MAX_LINES ]]; then
    echo -e "${YELLOW}⚠ Diff trop long ($DIFF_LINES lignes), troncature à $MAX_LINES${NC}"
    DIFF=$(echo "$DIFF" | head -n $MAX_LINES)
    DIFF="$DIFF

... [TRONQUÉ - diff original: $DIFF_LINES lignes]"
fi

echo -e "${BLUE}🤖 Analyse en cours avec Claude...${NC}"
echo

# Build prompt
PROMPT='Tu es un expert Rust senior qui review du code pour le projet Lynx.

## Contexte Lynx
Lynx est un analyseur statique JavaScript/TypeScript ultra-rapide écrit en Rust:
- **lynx-core**: Moteur d analyse pur (AUCUN I/O)
- **lynx-lsp**: Serveur LSP (async uniquement avec tokio)
- **lynx-cli**: Interface CLI (I/O autorisé)
- Rust Edition 2024, parsing avec SWC

## Règles CRITIQUES (bloquantes)
1. **Panics interdits** dans lynx-core et lynx-lsp:
   - ❌ `.unwrap()`, `.expect()`, `panic!()`, `unreachable!()`
   - ✅ `?`, `.ok_or()`, `.map_err()`, `anyhow!`, `bail!`

2. **I/O bloquant interdit** dans lynx-lsp:
   - ❌ `std::fs::*`, `std::io::*`
   - ✅ `tokio::fs::*`, `tokio::io::*`

3. **Sécurité**:
   - Valider les chemins de fichiers
   - Pas de path traversal (`../`)

## Règles importantes (à corriger)
- Performance: éviter `.clone()` dans les boucles, préférer `&str` à `String`
- Erreurs: messages actionnables avec contexte
- Tests: toute nouvelle règle doit avoir des tests

## Format de réponse

### 🔴 BLOQUANT
> fichier:ligne - problème
> ```rust
> // suggestion de fix
> ```

### 🟡 À AMÉLIORER  
> fichier:ligne - suggestion

### 🟢 POINTS POSITIFS
> - ce qui est bien fait

### VERDICT
**[APPROVE]** ou **[REQUEST_CHANGES]** avec résumé en une phrase

---
Si tout est OK, dis simplement "✅ LGTM - Code propre, aucun problème détecté." et **[APPROVE]**'

# Run Claude
REVIEW=$(claude -p "$PROMPT

## Fichiers modifiés
$CHANGED_FILES

## Diff
\`\`\`diff
$DIFF
\`\`\`")

# Display review
echo "$REVIEW"
echo
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

# Post to PR if requested
if [[ -n "$POST_TO_PR" ]]; then
    if ! command -v gh &> /dev/null; then
        echo -e "${RED}❌ gh CLI requis pour poster sur PR${NC}"
        exit 1
    fi
    
    echo -e "${BLUE}📤 Post sur PR #$POST_TO_PR...${NC}"
    
    COMMENT_BODY="## 🔍 Claude Code Review

$REVIEW

---
<sub>Review par Claude Code • $(date '+%Y-%m-%d %H:%M')</sub>"

    gh pr comment "$POST_TO_PR" --body "$COMMENT_BODY"
    echo -e "${GREEN}✓ Review postée sur PR #$POST_TO_PR${NC}"
fi

# Copy to clipboard
if command -v pbcopy &> /dev/null; then
    echo "$REVIEW" | pbcopy
    echo -e "${CYAN}📋 Review copiée dans le presse-papier${NC}"
elif command -v xclip &> /dev/null; then
    echo "$REVIEW" | xclip -selection clipboard
    echo -e "${CYAN}📋 Review copiée dans le presse-papier${NC}"
elif command -v wl-copy &> /dev/null; then
    echo "$REVIEW" | wl-copy
    echo -e "${CYAN}📋 Review copiée dans le presse-papier${NC}"
fi
