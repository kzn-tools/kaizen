#!/bin/bash

# Script pour créer les issues GitHub pour le projet Kaizen SaaS Launch
# Usage: ./scripts/create-github-issues.sh [repo]
# Exemple: ./scripts/create-github-issues.sh kaizen

set -e

ORG="kzn-tools"
REPO="${1:-kaizen}"

echo "🚀 Création des issues pour $ORG/$REPO"
echo ""

# Vérifier que gh est installé et authentifié
if ! command -v gh &> /dev/null; then
    echo "❌ GitHub CLI (gh) n'est pas installé"
    echo "   Installer avec: brew install gh (macOS) ou sudo apt install gh (Linux)"
    exit 1
fi

if ! gh auth status &> /dev/null; then
    echo "❌ GitHub CLI n'est pas authentifié"
    echo "   Exécuter: gh auth login"
    exit 1
fi

# Fonction pour créer une issue
create_issue() {
    local title="$1"
    local body="$2"
    local labels="$3"

    echo "📝 Creating: $title"
    gh issue create \
        --repo "$ORG/$REPO" \
        --title "$title" \
        --body "$body" \
        --label "$labels" \
        2>/dev/null || echo "   ⚠️  Erreur (issue existe peut-être déjà)"
}

# ============================================================================
# ISSUES POUR REPO: kaizen
# ============================================================================

if [ "$REPO" = "kaizen" ]; then

echo "📦 Création des issues pour kaizen..."
echo ""

# --- SETUP & MIGRATION ---

create_issue \
"[KZN-001] Transférer le repo mpiton/kaizen vers kzn-tools" \
"## Description
Transférer le repo existant vers la nouvelle organisation.

## Tâches
- [ ] Transférer ownership via GitHub Settings → Danger Zone
- [ ] Vérifier que les redirections fonctionnent
- [ ] Mettre à jour les remotes locaux

## Acceptance Criteria
- Le repo est accessible sur github.com/kzn-tools/kaizen
- Les anciennes URLs redirigent automatiquement

## Effort: 30 min" \
"priority:critical,phase:1-foundation,type:infra"

create_issue \
"[KZN-002] Mettre à jour toutes les références d'URL" \
"## Description
Mettre à jour tous les fichiers contenant l'ancienne URL.

## Fichiers à modifier
- \`Cargo.toml\` (workspace)
- \`crates/kaizen-core/Cargo.toml\`
- \`crates/kaizen-cli/Cargo.toml\`
- \`crates/kaizen-lsp/Cargo.toml\`
- \`README.md\`
- \`CONTRIBUTING.md\`
- \`action.yml\`
- \`npm/kaizen-cli/package.json\`

## Depends on
- #KZN-001

## Effort: 1h" \
"priority:critical,phase:1-foundation,type:infra"

create_issue \
"[KZN-010] Créer le module licensing.rs" \
"## Description
Implémenter le système de validation des clés API.

## Tâches
- [ ] Créer \`crates/kaizen-core/src/licensing.rs\`
- [ ] Définir \`PremiumTier\` enum (Free, Pro, Enterprise)
- [ ] Définir \`LicenseInfo\` struct
- [ ] Implémenter \`LicenseValidator\`
- [ ] Validation locale HMAC
- [ ] Vérification expiration
- [ ] Export public dans lib.rs

## Code structure
\`\`\`rust
pub enum PremiumTier { Free, Pro, Enterprise }
pub struct LicenseInfo { tier, api_key, valid_until, features }
pub struct LicenseValidator { signing_key }
impl LicenseValidator {
    pub fn validate_local(&self, api_key: &str) -> Result<LicenseInfo>
}
\`\`\`

## Tests
- [ ] Test clé valide
- [ ] Test clé expirée
- [ ] Test signature invalide
- [ ] Test format invalide

## Effort: 4h" \
"priority:critical,phase:1-foundation,type:feature"

create_issue \
"[KZN-011] Intégrer licensing dans le CLI" \
"## Description
Intégrer la validation des clés API dans la commande check.

## Tâches
- [ ] Lire KAIZEN_API_KEY depuis env
- [ ] Lire depuis ~/.kaizen/credentials (fallback)
- [ ] Lire depuis kaizen.toml [license] (fallback)
- [ ] Afficher tier activé au démarrage
- [ ] Passer tier à l'engine d'analyse

## Fichiers
- \`crates/kaizen-cli/src/commands/check.rs\`
- \`crates/kaizen-cli/src/cli.rs\`

## Depends on
- #KZN-010

## Effort: 2h" \
"priority:critical,phase:1-foundation,type:feature"

create_issue \
"[KZN-012] Intégrer licensing dans le LSP" \
"## Description
Récupérer et valider l'API key dans le serveur LSP.

## Tâches
- [ ] Récupérer API key depuis workspace settings
- [ ] Stocker LicenseInfo dans server state
- [ ] Passer tier à l'engine d'analyse

## Fichiers
- \`crates/kaizen-lsp/src/server.rs\`

## Depends on
- #KZN-010

## Effort: 2h" \
"priority:high,phase:1-foundation,type:feature"

create_issue \
"[KZN-013] Ajouter commande kaizen auth" \
"## Description
Nouvelle commande pour gérer l'authentification.

## Sous-commandes
- \`kaizen auth login\` : Sauvegarder clé API
- \`kaizen auth logout\` : Supprimer clé
- \`kaizen auth status\` : Afficher tier actuel

## Tâches
- [ ] Créer \`crates/kaizen-cli/src/commands/auth.rs\`
- [ ] Stockage sécurisé (~/.kaizen/credentials, chmod 600)
- [ ] Ajouter à la CLI

## Depends on
- #KZN-010

## Effort: 3h" \
"priority:medium,phase:1-foundation,type:feature"

create_issue \
"[KZN-014] Filtrer les règles par tier" \
"## Description
Limiter l'exécution des règles selon le tier de l'utilisateur.

## Tâches
- [ ] Ajouter \`min_tier\` à RuleMetadata
- [ ] Modifier RuleRegistry pour filtrer par tier
- [ ] Marquer les règles premium (S020+)

## Fichiers
- \`crates/kaizen-core/src/rules/mod.rs\`
- \`crates/kaizen-core/src/analysis.rs\`

## Depends on
- #KZN-010

## Effort: 2h" \
"priority:critical,phase:1-foundation,type:feature"

create_issue \
"[KZN-020] Implémenter S020 - Prototype Pollution" \
"## Description
Nouvelle règle premium pour détecter les vulnérabilités de prototype pollution.

## Patterns à détecter
- Object.assign dangereux
- Patterns merge/extend non sécurisés
- Deep merge vulnerabilities

## Références
- CVE-2019-10744 (lodash)
- CVE-2020-8203 (lodash)

## Tâches
- [ ] Créer \`crates/kaizen-core/src/rules/security/prototype_pollution.rs\`
- [ ] Ajouter au registry
- [ ] Tests avec cas réels

## Effort: 8h" \
"priority:high,phase:1-foundation,type:feature,type:security"

create_issue \
"[KZN-021] Implémenter S021 - Regex DoS (ReDoS)" \
"## Description
Détecter les expressions régulières vulnérables au ReDoS.

## Patterns à détecter
- Nested quantifiers: \`(a+)+\`
- Overlapping alternations: \`(a|a)+\`
- Large repetitions avec groupes

## Tâches
- [ ] Créer \`crates/kaizen-core/src/rules/security/redos.rs\`
- [ ] Analyser patterns regex
- [ ] Tests

## Complexité: Haute

## Effort: 12h" \
"priority:high,phase:1-foundation,type:feature,type:security"

create_issue \
"[KZN-022] Implémenter S022 - Unsafe Deserialization" \
"## Description
Détecter les chaînes de désérialisation dangereuses.

## Patterns à détecter
- JSON.parse → eval
- JSON.parse → Function constructor
- Unsafe reviver functions

## Tâches
- [ ] Créer \`crates/kaizen-core/src/rules/security/unsafe_deserialization.rs\`
- [ ] Tests

## Effort: 6h" \
"priority:high,phase:1-foundation,type:feature,type:security"

create_issue \
"[KZN-030] Documenter le système de licensing" \
"## Description
Documentation utilisateur pour le système premium.

## Tâches
- [ ] README section \"Premium Features\"
- [ ] docs/licensing.md
- [ ] docs/pricing.md (feature matrix)

## Effort: 2h" \
"priority:high,phase:1-foundation,type:docs"

echo ""
echo "✅ Issues kaizen créées!"

fi

# ============================================================================
# ISSUES POUR REPO: zed
# ============================================================================

if [ "$REPO" = "zed" ]; then

echo "🔌 Création des issues pour zed..."
echo ""

create_issue \
"[ZED-001] Setup initial du repo" \
"## Description
Configurer le nouveau repo après extraction.

## Tâches
- [ ] Initialiser repo
- [ ] Copier fichiers depuis monorepo
- [ ] Commit initial
- [ ] Push vers GitHub

## Effort: 1h" \
"priority:high,phase:1-foundation,type:infra"

create_issue \
"[ZED-002] Mettre à jour les métadonnées" \
"## Description
Mettre à jour les fichiers de configuration.

## Fichiers
- [ ] extension.toml: repository URL
- [ ] Cargo.toml: repository URL
- [ ] README.md: instructions installation

## Effort: 30min" \
"priority:high,phase:1-foundation,type:infra"

create_issue \
"[ZED-003] Configurer CI/CD" \
"## Description
Mettre en place les workflows GitHub Actions.

## Tâches
- [ ] .github/workflows/build.yml
- [ ] Build WASM sur push
- [ ] Cache cargo pour performance

## Effort: 1h" \
"priority:medium,phase:1-foundation,type:infra"

create_issue \
"[ZED-010] Écrire README complet" \
"## Description
Documentation complète pour les utilisateurs.

## Sections
- [ ] Description du plugin
- [ ] Prérequis (kaizen-lsp installé)
- [ ] Instructions d'installation
- [ ] Configuration
- [ ] Troubleshooting
- [ ] Screenshots

## Effort: 1h" \
"priority:medium,phase:1-foundation,type:docs"

create_issue \
"[ZED-020] Publier sur Zed Extensions" \
"## Description
Soumettre le plugin au marketplace officiel.

## Tâches
- [ ] Fork zed-industries/extensions
- [ ] Ajouter kaizen à la liste
- [ ] Soumettre PR
- [ ] Attendre review et merge

## Reference
https://github.com/zed-industries/extensions

## Effort: 2h" \
"priority:medium,phase:2-saas,type:feature"

echo ""
echo "✅ Issues zed créées!"

fi

# ============================================================================
# ISSUES POUR REPO: vscode
# ============================================================================

if [ "$REPO" = "vscode" ]; then

echo "💻 Création des issues pour vscode..."
echo ""

create_issue \
"[VSC-001] Setup initial du repo" \
"## Description
Configurer le nouveau repo après extraction.

## Tâches
- [ ] Initialiser repo
- [ ] Copier fichiers depuis monorepo
- [ ] Commit initial
- [ ] Push vers GitHub

## Effort: 1h" \
"priority:high,phase:1-foundation,type:infra"

create_issue \
"[VSC-002] Mettre à jour package.json" \
"## Description
Mettre à jour les métadonnées.

## Tâches
- [ ] publisher: \"kzn-tools\"
- [ ] repository: URL GitHub
- [ ] Mettre à jour version

## Effort: 30min" \
"priority:high,phase:1-foundation,type:infra"

create_issue \
"[VSC-003] Configurer CI/CD" \
"## Description
Workflows GitHub Actions.

## Tâches
- [ ] .github/workflows/build.yml
- [ ] .github/workflows/publish.yml
- [ ] Build TypeScript
- [ ] Package VSIX

## Effort: 1h" \
"priority:medium,phase:1-foundation,type:infra"

create_issue \
"[VSC-020] Créer publisher VS Code Marketplace" \
"## Description
Préparer la publication.

## Tâches
- [ ] Créer compte sur marketplace.visualstudio.com
- [ ] Créer publisher \"kzn-tools\"
- [ ] Générer Personal Access Token

## Effort: 1h" \
"priority:medium,phase:2-saas,type:infra"

create_issue \
"[VSC-021] Publier sur VS Code Marketplace" \
"## Description
Publier l'extension.

## Tâches
- [ ] \`vsce package\`
- [ ] \`vsce publish\`
- [ ] Vérifier listing sur marketplace

## Depends on
- #VSC-020

## Effort: 1h" \
"priority:medium,phase:2-saas,type:feature"

echo ""
echo "✅ Issues vscode créées!"

fi

# ============================================================================
# ISSUES POUR REPO: cloud
# ============================================================================

if [ "$REPO" = "cloud" ]; then

echo "☁️ Création des issues pour cloud..."
echo ""

create_issue \
"[CLD-001] Créer la structure du repo" \
"## Description
Structure initiale du backend SaaS.

## Structure
\`\`\`
cloud/
├── api/              # Backend Rust (Axum)
├── dashboard/        # Frontend React
├── workers/          # Background jobs
├── infra/            # Terraform
├── docker/           # Dockerfiles
└── docs/             # Documentation interne
\`\`\`

## Tâches
- [ ] Créer repo privé
- [ ] Initialiser structure
- [ ] README avec setup instructions

## Effort: 2h" \
"priority:high,phase:2-saas,type:infra"

create_issue \
"[CLD-002] Setup API Rust (Axum)" \
"## Description
Initialiser le backend API.

## Tâches
- [ ] Cargo.toml avec dépendances
- [ ] Structure src/
- [ ] Main avec server basic
- [ ] Health check endpoint

## Dependencies
\`\`\`toml
axum = \"0.7\"
tokio = { version = \"1\", features = [\"full\"] }
serde = { version = \"1.0\", features = [\"derive\"] }
sqlx = { version = \"0.8\", features = [\"postgres\", \"runtime-tokio\"] }
\`\`\`

## Effort: 4h" \
"priority:high,phase:2-saas,type:feature"

create_issue \
"[CLD-010] Implémenter OAuth Device Flow" \
"## Description
Authentification CLI via browser.

## Endpoints
- POST /auth/device - Initier flow
- GET /auth/device/token - Poll pour token

## Tâches
- [ ] Intégration GitHub OAuth
- [ ] Intégration Google OAuth (optionnel)
- [ ] Tests

## Effort: 8h" \
"priority:high,phase:2-saas,type:feature"

create_issue \
"[CLD-012] Implémenter gestion des API keys" \
"## Description
CRUD pour les clés API.

## Endpoints
- POST /keys - Créer une clé
- GET /keys - Lister les clés
- DELETE /keys/:id - Révoquer
- POST /keys/validate - Valider

## Tâches
- [ ] Génération format kz_[tier]_[org]_[ts]_[sig]
- [ ] Hash des clés en DB
- [ ] Tests

## Effort: 6h" \
"priority:critical,phase:2-saas,type:feature"

create_issue \
"[CLD-020] Intégrer Stripe" \
"## Description
Setup billing avec Stripe.

## Tâches
- [ ] Créer compte Stripe
- [ ] Créer produits (Free, Pro, Enterprise)
- [ ] Créer prix (mensuel, annuel)
- [ ] Webhook endpoint

## Effort: 8h" \
"priority:high,phase:2-saas,type:feature"

create_issue \
"[CLD-030] Setup React Dashboard" \
"## Description
Frontend pour le dashboard.

## Stack
- Vite + React + TypeScript
- TailwindCSS
- React Router
- React Query

## Pages
- [ ] Login
- [ ] Dashboard
- [ ] API Keys
- [ ] Billing
- [ ] Settings

## Effort: 4h (setup) + 18h (pages)" \
"priority:medium,phase:2-saas,type:feature"

create_issue \
"[CLD-040] Docker setup" \
"## Description
Containerisation.

## Tâches
- [ ] Dockerfile pour API
- [ ] Dockerfile pour Dashboard
- [ ] docker-compose.yml (dev)
- [ ] docker-compose.prod.yml

## Effort: 2h" \
"priority:high,phase:2-saas,type:infra"

create_issue \
"[CLD-041] Terraform setup" \
"## Description
Infrastructure as Code.

## Tâches
- [ ] Provider (Fly.io ou AWS)
- [ ] Database (Postgres managed)
- [ ] Redis
- [ ] Container runtime
- [ ] CDN pour dashboard

## Effort: 8h" \
"priority:medium,phase:2-saas,type:infra"

echo ""
echo "✅ Issues cloud créées!"

fi

echo ""
echo "🎉 Terminé!"
echo ""
echo "Pour créer les issues des autres repos:"
echo "  ./scripts/create-github-issues.sh zed"
echo "  ./scripts/create-github-issues.sh vscode"
echo "  ./scripts/create-github-issues.sh cloud"
