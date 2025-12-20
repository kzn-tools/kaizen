# Projet GitHub : Kaizen SaaS Launch

## Configuration du Projet GitHub

### Créer le Projet

1. Aller sur https://github.com/orgs/kzn-tools/projects
2. "New project" → "Board"
3. Nom : **"Kaizen SaaS Launch"**
4. Description : "Roadmap pour le lancement du système d'abonnement Kaizen"

### Colonnes du Board

| Colonne | Description |
|---------|-------------|
| 📋 Backlog | Tâches planifiées mais pas encore priorisées |
| 🎯 To Do | Prêt à être développé (sprint actuel) |
| 🚧 In Progress | En cours de développement |
| 👀 In Review | En attente de review/PR |
| ✅ Done | Terminé |

### Labels à Créer

| Label | Couleur | Description |
|-------|---------|-------------|
| `repo:kaizen` | #1D76DB | Repo principal (core) |
| `repo:zed` | #0E8A16 | Extension Zed |
| `repo:vscode` | #5319E7 | Extension VS Code |
| `repo:cloud` | #D93F0B | Backend SaaS |
| `priority:critical` | #B60205 | Bloquant |
| `priority:high` | #D93F0B | Important |
| `priority:medium` | #FBCA04 | Normal |
| `priority:low` | #0E8A16 | Nice to have |
| `type:feature` | #1D76DB | Nouvelle fonctionnalité |
| `type:bug` | #D73A4A | Bug fix |
| `type:infra` | #F9D0C4 | Infrastructure/CI/CD |
| `type:docs` | #0075CA | Documentation |
| `type:security` | #B60205 | Sécurité |
| `phase:1-foundation` | #C2E0C6 | Phase 1 |
| `phase:2-saas` | #BFD4F2 | Phase 2 |
| `phase:3-enterprise` | #D4C5F9 | Phase 3 |

---

# TÂCHES PAR REPOSITORY

---

## 📦 REPO: kzn-tools/kaizen (Core)

### 🏗️ SETUP & MIGRATION

#### KZN-001: Transférer le repo mpiton/kaizen vers kzn-tools
- **Priority:** Critical
- **Phase:** 1
- **Effort:** 30 min
- **Description:**
  - [ ] Transférer ownership via GitHub Settings
  - [ ] Vérifier que les redirections fonctionnent
  - [ ] Mettre à jour les remotes locaux
- **Acceptance Criteria:**
  - Le repo est accessible sur github.com/kzn-tools/kaizen
  - Les anciennes URLs redirigent automatiquement

#### KZN-002: Mettre à jour toutes les références d'URL
- **Priority:** Critical
- **Phase:** 1
- **Effort:** 1h
- **Depends on:** KZN-001
- **Description:**
  - [ ] Cargo.toml (workspace) : repository URL
  - [ ] README.md : badges, liens clone
  - [ ] CONTRIBUTING.md : liens
  - [ ] action.yml : uses reference
  - [ ] package.json (npm) : repository
- **Files to modify:**
  ```
  Cargo.toml
  crates/kaizen-core/Cargo.toml
  crates/kaizen-cli/Cargo.toml
  crates/kaizen-lsp/Cargo.toml
  README.md
  CONTRIBUTING.md
  action.yml
  npm/kaizen-cli/package.json
  ```

#### KZN-003: Configurer les secrets GitHub Actions
- **Priority:** High
- **Phase:** 1
- **Effort:** 30 min
- **Description:**
  - [ ] CARGO_REGISTRY_TOKEN (crates.io)
  - [ ] NPM_TOKEN (npmjs.com)
  - [ ] Vérifier que CI fonctionne après transfert
- **Acceptance Criteria:**
  - CI passe sur le nouveau repo
  - Release workflow fonctionne

#### KZN-004: Supprimer le dossier editors/ du monorepo
- **Priority:** Medium
- **Phase:** 1
- **Effort:** 15 min
- **Depends on:** ZED-001, VSC-001
- **Description:**
  - [ ] Supprimer editors/zed
  - [ ] Supprimer editors/vscode
  - [ ] Ajouter editors/README.md avec liens vers nouveaux repos
  - [ ] Commit et push

---

### 🔐 SYSTÈME DE LICENSING

#### KZN-010: Créer le module licensing.rs
- **Priority:** Critical
- **Phase:** 1
- **Effort:** 4h
- **Description:**
  Implémenter le système de validation des clés API.
  - [ ] Créer `crates/kaizen-core/src/licensing.rs`
  - [ ] Définir `PremiumTier` enum (Free, Pro, Enterprise)
  - [ ] Définir `LicenseInfo` struct
  - [ ] Implémenter `LicenseValidator`
  - [ ] Validation locale HMAC
  - [ ] Vérification expiration
  - [ ] Export public dans lib.rs
- **Code structure:**
  ```rust
  pub enum PremiumTier { Free, Pro, Enterprise }
  pub struct LicenseInfo { tier, api_key, valid_until, features }
  pub struct LicenseValidator { signing_key }
  impl LicenseValidator {
      pub fn validate_local(&self, api_key: &str) -> Result<LicenseInfo>
  }
  ```
- **Tests:**
  - [ ] Test clé valide
  - [ ] Test clé expirée
  - [ ] Test signature invalide
  - [ ] Test format invalide

#### KZN-011: Intégrer licensing dans le CLI
- **Priority:** Critical
- **Phase:** 1
- **Effort:** 2h
- **Depends on:** KZN-010
- **Description:**
  - [ ] Lire KAIZEN_API_KEY depuis env
  - [ ] Lire depuis ~/.kaizen/credentials (fallback)
  - [ ] Lire depuis kaizen.toml [license] (fallback)
  - [ ] Afficher tier activé au démarrage
  - [ ] Passer tier à l'engine d'analyse
- **Files:**
  ```
  crates/kaizen-cli/src/commands/check.rs
  crates/kaizen-cli/src/cli.rs
  ```

#### KZN-012: Intégrer licensing dans le LSP
- **Priority:** High
- **Phase:** 1
- **Effort:** 2h
- **Depends on:** KZN-010
- **Description:**
  - [ ] Récupérer API key depuis workspace settings
  - [ ] Stocker LicenseInfo dans server state
  - [ ] Passer tier à l'engine d'analyse
- **Files:**
  ```
  crates/kaizen-lsp/src/server.rs
  ```

#### KZN-013: Ajouter commande `kaizen auth`
- **Priority:** Medium
- **Phase:** 1
- **Effort:** 3h
- **Depends on:** KZN-010
- **Description:**
  - [ ] `kaizen auth login` : Sauvegarder clé API
  - [ ] `kaizen auth logout` : Supprimer clé
  - [ ] `kaizen auth status` : Afficher tier actuel
  - [ ] Stockage sécurisé (~/.kaizen/credentials, chmod 600)
- **Files:**
  ```
  crates/kaizen-cli/src/commands/auth.rs (nouveau)
  crates/kaizen-cli/src/commands/mod.rs
  crates/kaizen-cli/src/cli.rs
  ```

#### KZN-014: Filtrer les règles par tier
- **Priority:** Critical
- **Phase:** 1
- **Effort:** 2h
- **Depends on:** KZN-010
- **Description:**
  - [ ] Ajouter `min_tier` à RuleMetadata
  - [ ] Modifier RuleRegistry pour filtrer par tier
  - [ ] Marquer les règles premium (S020+)
- **Files:**
  ```
  crates/kaizen-core/src/rules/mod.rs
  crates/kaizen-core/src/analysis.rs
  ```

#### KZN-015: Ajouter messages upgrade pour règles premium
- **Priority:** Medium
- **Phase:** 1
- **Effort:** 1h
- **Depends on:** KZN-014
- **Description:**
  - [ ] Afficher message quand règle premium est skipped
  - [ ] Lien vers pricing page
  - [ ] Option --quiet pour désactiver messages

---

### 🛡️ RÈGLES PREMIUM

#### KZN-020: Implémenter S020 - Prototype Pollution
- **Priority:** High
- **Phase:** 1
- **Effort:** 8h
- **Description:**
  Détecter les vulnérabilités de prototype pollution.
  - [ ] Patterns Object.assign dangereux
  - [ ] Patterns merge/extend non sécurisés
  - [ ] Deep merge vulnerabilities
  - [ ] Tests avec cas réels (CVEs connus)
- **Files:**
  ```
  crates/kaizen-core/src/rules/security/prototype_pollution.rs (nouveau)
  crates/kaizen-core/src/rules/security/mod.rs
  ```
- **References:**
  - CVE-2019-10744 (lodash)
  - CVE-2020-8203 (lodash)

#### KZN-021: Implémenter S021 - Regex DoS (ReDoS)
- **Priority:** High
- **Phase:** 1
- **Effort:** 12h
- **Description:**
  Détecter les expressions régulières vulnérables au ReDoS.
  - [ ] Analyser patterns regex pour backtracking exponentiel
  - [ ] Nested quantifiers: (a+)+
  - [ ] Overlapping alternations: (a|a)+
  - [ ] Large repetitions avec groupes
- **Files:**
  ```
  crates/kaizen-core/src/rules/security/redos.rs (nouveau)
  ```
- **Complexity:** Haute - nécessite analyse de regex

#### KZN-022: Implémenter S022 - Unsafe Deserialization
- **Priority:** High
- **Phase:** 1
- **Effort:** 6h
- **Description:**
  Détecter les chaînes de désérialisation dangereuses.
  - [ ] JSON.parse → eval patterns
  - [ ] JSON.parse → Function constructor
  - [ ] Unsafe reviver functions
- **Files:**
  ```
  crates/kaizen-core/src/rules/security/unsafe_deserialization.rs (nouveau)
  ```

#### KZN-023: Implémenter S023 - Path Traversal
- **Priority:** Medium
- **Phase:** 1
- **Effort:** 6h
- **Description:**
  Détecter les vulnérabilités de path traversal.
  - [ ] fs.readFile avec input non sanitisé
  - [ ] path.join avec input utilisateur
  - [ ] require() dynamique
- **Files:**
  ```
  crates/kaizen-core/src/rules/security/path_traversal.rs (nouveau)
  ```

#### KZN-024: Implémenter S024 - SSRF
- **Priority:** Medium
- **Phase:** 2
- **Effort:** 8h
- **Description:**
  Détecter les vulnérabilités SSRF.
  - [ ] fetch/axios avec URL non validée
  - [ ] http.request avec host dynamique
  - [ ] Redirections non contrôlées
- **Files:**
  ```
  crates/kaizen-core/src/rules/security/ssrf.rs (nouveau)
  ```

#### KZN-025: Implémenter Q050 - React Hooks Rules
- **Priority:** Medium
- **Phase:** 2
- **Effort:** 8h
- **Description:**
  Règles spécifiques React.
  - [ ] useEffect missing cleanup
  - [ ] useEffect missing dependencies
  - [ ] useState in conditions
  - [ ] Custom hooks naming
- **Files:**
  ```
  crates/kaizen-core/src/rules/quality/react_hooks.rs (nouveau)
  ```

---

### 📚 DOCUMENTATION

#### KZN-030: Documenter le système de licensing
- **Priority:** High
- **Phase:** 1
- **Effort:** 2h
- **Description:**
  - [ ] README section "Premium Features"
  - [ ] docs/licensing.md
  - [ ] docs/pricing.md (feature matrix)

#### KZN-031: Documenter les règles premium
- **Priority:** Medium
- **Phase:** 1
- **Effort:** 3h
- **Depends on:** KZN-020, KZN-021, KZN-022
- **Description:**
  - [ ] docs/rules/S020-prototype-pollution.md
  - [ ] docs/rules/S021-redos.md
  - [ ] docs/rules/S022-unsafe-deserialization.md
  - [ ] Exemples de code vulnérable/sécurisé

#### KZN-032: Créer page CHANGELOG
- **Priority:** Low
- **Phase:** 1
- **Effort:** 1h
- **Description:**
  - [ ] CHANGELOG.md avec format Keep a Changelog
  - [ ] Historique des versions existantes

---

### 🔄 CI/CD

#### KZN-040: Ajouter tests pour licensing
- **Priority:** High
- **Phase:** 1
- **Effort:** 2h
- **Depends on:** KZN-010
- **Description:**
  - [ ] Tests unitaires licensing.rs
  - [ ] Tests intégration CLI avec API key
  - [ ] Tests tier filtering

#### KZN-041: Ajouter benchmark règles premium
- **Priority:** Low
- **Phase:** 2
- **Effort:** 2h
- **Description:**
  - [ ] Benchmark S020 (prototype pollution)
  - [ ] Benchmark S021 (redos) - important car analyse complexe
  - [ ] Comparer avec/sans règles premium

---

## 🔌 REPO: kzn-tools/zed (Extension Zed)

### 🏗️ SETUP

#### ZED-001: Extraire l'extension du monorepo
- **Priority:** High
- **Phase:** 1
- **Effort:** 1h
- **Description:**
  - [ ] Créer le repo kzn-tools/zed
  - [ ] Copier editors/zed vers nouveau repo
  - [ ] Initialiser git, commit initial
  - [ ] Push vers GitHub
- **Commands:**
  ```bash
  mkdir ~/kzn-zed && cd ~/kzn-zed
  cp -r ~/projets/lynx/editors/zed/* .
  git init && git add . && git commit -m "feat: initial extraction"
  git remote add origin git@github.com:kzn-tools/zed.git
  git push -u origin main
  ```

#### ZED-002: Mettre à jour les métadonnées
- **Priority:** High
- **Phase:** 1
- **Effort:** 30 min
- **Depends on:** ZED-001
- **Description:**
  - [ ] extension.toml: repository URL
  - [ ] Cargo.toml: repository URL
  - [ ] README.md: instructions installation
- **Files:**
  ```
  extension.toml
  Cargo.toml
  README.md
  ```

#### ZED-003: Configurer CI/CD
- **Priority:** Medium
- **Phase:** 1
- **Effort:** 1h
- **Description:**
  - [ ] .github/workflows/build.yml
  - [ ] Build WASM sur push
  - [ ] Cache cargo pour performance
- **Workflow:**
  ```yaml
  name: Build
  on: [push, pull_request]
  jobs:
    build:
      runs-on: ubuntu-latest
      steps:
        - uses: actions/checkout@v4
        - name: Install Rust
          uses: dtolnay/rust-action@stable
          with:
            targets: wasm32-wasip1
        - name: Build
          run: cargo build --release --target wasm32-wasip1
  ```

### 📚 DOCUMENTATION

#### ZED-010: Écrire README complet
- **Priority:** Medium
- **Phase:** 1
- **Effort:** 1h
- **Description:**
  - [ ] Description du plugin
  - [ ] Prérequis (kaizen-lsp installé)
  - [ ] Instructions d'installation
  - [ ] Configuration
  - [ ] Troubleshooting
  - [ ] Screenshots

#### ZED-011: Ajouter CONTRIBUTING.md
- **Priority:** Low
- **Phase:** 1
- **Effort:** 30 min
- **Description:**
  - [ ] Comment contribuer
  - [ ] Setup développement
  - [ ] Guidelines

### 🚀 PUBLICATION

#### ZED-020: Publier sur Zed Extensions
- **Priority:** Medium
- **Phase:** 2
- **Effort:** 2h
- **Description:**
  - [ ] Fork zed-industries/extensions
  - [ ] Ajouter kaizen à la liste
  - [ ] Soumettre PR
  - [ ] Attendre review et merge
- **Reference:** https://github.com/zed-industries/extensions

---

## 💻 REPO: kzn-tools/vscode (Extension VS Code)

### 🏗️ SETUP

#### VSC-001: Extraire l'extension du monorepo
- **Priority:** High
- **Phase:** 1
- **Effort:** 1h
- **Description:**
  - [ ] Créer le repo kzn-tools/vscode
  - [ ] Copier editors/vscode vers nouveau repo
  - [ ] Initialiser git, commit initial
  - [ ] Push vers GitHub

#### VSC-002: Mettre à jour package.json
- **Priority:** High
- **Phase:** 1
- **Effort:** 30 min
- **Depends on:** VSC-001
- **Description:**
  - [ ] publisher: "kzn-tools"
  - [ ] repository: URL GitHub
  - [ ] Mettre à jour version
- **File:** `package.json`

#### VSC-003: Configurer CI/CD
- **Priority:** Medium
- **Phase:** 1
- **Effort:** 1h
- **Description:**
  - [ ] .github/workflows/build.yml
  - [ ] .github/workflows/publish.yml
  - [ ] Build TypeScript
  - [ ] Package VSIX

### 📚 DOCUMENTATION

#### VSC-010: Écrire README complet
- **Priority:** Medium
- **Phase:** 1
- **Effort:** 1h
- **Description:**
  - [ ] Features
  - [ ] Installation depuis marketplace
  - [ ] Installation manuelle
  - [ ] Configuration settings
  - [ ] Screenshots/GIFs

### 🚀 PUBLICATION

#### VSC-020: Créer publisher VS Code Marketplace
- **Priority:** Medium
- **Phase:** 2
- **Effort:** 1h
- **Description:**
  - [ ] Créer compte sur marketplace.visualstudio.com
  - [ ] Créer publisher "kzn-tools"
  - [ ] Générer Personal Access Token

#### VSC-021: Publier sur VS Code Marketplace
- **Priority:** Medium
- **Phase:** 2
- **Effort:** 1h
- **Depends on:** VSC-020
- **Description:**
  - [ ] `vsce package`
  - [ ] `vsce publish`
  - [ ] Vérifier listing sur marketplace

---

## ☁️ REPO: kzn-tools/cloud (Backend SaaS)

### 🏗️ SETUP INITIAL

#### CLD-001: Créer la structure du repo
- **Priority:** High
- **Phase:** 2
- **Effort:** 2h
- **Description:**
  ```
  cloud/
  ├── api/              # Backend Rust (Axum)
  ├── dashboard/        # Frontend React
  ├── workers/          # Background jobs
  ├── infra/            # Terraform
  ├── docker/           # Dockerfiles
  └── docs/             # Documentation interne
  ```
  - [ ] Créer repo privé
  - [ ] Initialiser structure
  - [ ] README avec setup instructions

#### CLD-002: Setup API Rust (Axum)
- **Priority:** High
- **Phase:** 2
- **Effort:** 4h
- **Description:**
  - [ ] Cargo.toml avec dépendances
  - [ ] Structure src/
  - [ ] Main avec server basic
  - [ ] Health check endpoint
- **Dependencies:**
  ```toml
  axum = "0.7"
  tokio = { version = "1", features = ["full"] }
  tower = "0.4"
  tower-http = { version = "0.5", features = ["cors", "trace"] }
  serde = { version = "1.0", features = ["derive"] }
  serde_json = "1.0"
  sqlx = { version = "0.8", features = ["postgres", "runtime-tokio", "uuid"] }
  uuid = { version = "1", features = ["v4", "serde"] }
  chrono = { version = "0.4", features = ["serde"] }
  tracing = "0.1"
  tracing-subscriber = "0.3"
  ```

#### CLD-003: Setup PostgreSQL
- **Priority:** High
- **Phase:** 2
- **Effort:** 2h
- **Description:**
  - [ ] docker-compose.yml avec postgres
  - [ ] Schema initial (users, orgs, api_keys, scans)
  - [ ] Migrations SQLx
  - [ ] Seed data pour dev

### 🔐 AUTHENTIFICATION

#### CLD-010: Implémenter OAuth Device Flow
- **Priority:** High
- **Phase:** 2
- **Effort:** 8h
- **Description:**
  - [ ] POST /auth/device - Initier flow
  - [ ] GET /auth/device/token - Poll pour token
  - [ ] Intégration GitHub OAuth
  - [ ] Intégration Google OAuth (optionnel)
- **Endpoints:**
  ```
  POST /auth/device
    Response: { device_code, user_code, verification_uri, expires_in }

  GET /auth/device/token?device_code=xxx
    Response: { access_token, token_type, expires_in } | { error: "pending" }
  ```

#### CLD-011: Implémenter gestion des sessions
- **Priority:** High
- **Phase:** 2
- **Effort:** 4h
- **Description:**
  - [ ] JWT tokens
  - [ ] Refresh tokens
  - [ ] Session storage (Redis)
  - [ ] Middleware authentication

#### CLD-012: Implémenter gestion des API keys
- **Priority:** Critical
- **Phase:** 2
- **Effort:** 6h
- **Description:**
  - [ ] POST /keys - Créer une clé
  - [ ] GET /keys - Lister les clés
  - [ ] DELETE /keys/:id - Révoquer une clé
  - [ ] POST /keys/validate - Valider une clé
  - [ ] Génération format kz_[tier]_[org]_[ts]_[sig]
  - [ ] Hash des clés en DB (jamais en clair)
- **Schema:**
  ```sql
  CREATE TABLE api_keys (
    id UUID PRIMARY KEY,
    org_id UUID REFERENCES organizations(id),
    key_hash VARCHAR(64) NOT NULL,
    key_prefix VARCHAR(20) NOT NULL,
    tier VARCHAR(20) NOT NULL,
    created_at TIMESTAMP DEFAULT NOW(),
    expires_at TIMESTAMP,
    revoked_at TIMESTAMP,
    last_used_at TIMESTAMP
  );
  ```

### 💳 BILLING (Stripe)

#### CLD-020: Intégrer Stripe
- **Priority:** High
- **Phase:** 2
- **Effort:** 8h
- **Description:**
  - [ ] Créer compte Stripe
  - [ ] Créer produits (Free, Pro, Enterprise)
  - [ ] Créer prix (mensuel, annuel)
  - [ ] Webhook endpoint pour events

#### CLD-021: Implémenter checkout flow
- **Priority:** High
- **Phase:** 2
- **Effort:** 6h
- **Depends on:** CLD-020
- **Description:**
  - [ ] POST /billing/checkout - Créer session Stripe
  - [ ] GET /billing/portal - Lien vers customer portal
  - [ ] Webhook: checkout.session.completed
  - [ ] Webhook: customer.subscription.updated
  - [ ] Webhook: customer.subscription.deleted

#### CLD-022: Implémenter gestion des quotas
- **Priority:** Medium
- **Phase:** 2
- **Effort:** 4h
- **Description:**
  - [ ] Tracker usage par org (Redis)
  - [ ] Reset mensuel
  - [ ] Alertes à 80%, 100%
  - [ ] Enforcement des limites

### 📊 DASHBOARD WEB

#### CLD-030: Setup React app
- **Priority:** Medium
- **Phase:** 2
- **Effort:** 4h
- **Description:**
  - [ ] Vite + React + TypeScript
  - [ ] TailwindCSS
  - [ ] React Router
  - [ ] React Query pour API calls
- **Structure:**
  ```
  dashboard/
  ├── src/
  │   ├── pages/
  │   │   ├── Login.tsx
  │   │   ├── Dashboard.tsx
  │   │   ├── ApiKeys.tsx
  │   │   ├── Billing.tsx
  │   │   └── Settings.tsx
  │   ├── components/
  │   ├── api/
  │   └── hooks/
  ├── package.json
  └── vite.config.ts
  ```

#### CLD-031: Page Login
- **Priority:** Medium
- **Phase:** 2
- **Effort:** 4h
- **Description:**
  - [ ] Login avec GitHub
  - [ ] Login avec Google
  - [ ] Redirect après auth

#### CLD-032: Page Dashboard
- **Priority:** Medium
- **Phase:** 2
- **Effort:** 6h
- **Description:**
  - [ ] Vue d'ensemble usage
  - [ ] Graphiques (scans/jour, vulnérabilités)
  - [ ] Derniers scans
  - [ ] Quick actions

#### CLD-033: Page API Keys
- **Priority:** Medium
- **Phase:** 2
- **Effort:** 4h
- **Description:**
  - [ ] Liste des clés
  - [ ] Créer nouvelle clé
  - [ ] Copier clé
  - [ ] Révoquer clé
  - [ ] Voir usage par clé

#### CLD-034: Page Billing
- **Priority:** Medium
- **Phase:** 2
- **Effort:** 4h
- **Description:**
  - [ ] Plan actuel
  - [ ] Upgrade/downgrade
  - [ ] Historique factures
  - [ ] Lien vers Stripe portal

### 🔧 INFRASTRUCTURE

#### CLD-040: Docker setup
- **Priority:** High
- **Phase:** 2
- **Effort:** 2h
- **Description:**
  - [ ] Dockerfile pour API
  - [ ] Dockerfile pour Dashboard
  - [ ] docker-compose.yml (dev)
  - [ ] docker-compose.prod.yml

#### CLD-041: Terraform setup
- **Priority:** Medium
- **Phase:** 2
- **Effort:** 8h
- **Description:**
  - [ ] Provider (AWS, GCP, ou Fly.io)
  - [ ] VPC/Network
  - [ ] Database (RDS ou managed)
  - [ ] Redis (ElastiCache ou managed)
  - [ ] Container runtime (ECS, Cloud Run, Fly)
  - [ ] CDN pour dashboard

#### CLD-042: CI/CD pipelines
- **Priority:** High
- **Phase:** 2
- **Effort:** 4h
- **Description:**
  - [ ] .github/workflows/api-test.yml
  - [ ] .github/workflows/api-deploy.yml
  - [ ] .github/workflows/dashboard-deploy.yml
  - [ ] Secrets management
  - [ ] Staging vs Production

### 📧 NOTIFICATIONS

#### CLD-050: Email transactionnel
- **Priority:** Medium
- **Phase:** 2
- **Effort:** 4h
- **Description:**
  - [ ] Intégrer Resend ou SendGrid
  - [ ] Template: Welcome
  - [ ] Template: API key created
  - [ ] Template: Usage warning (80%)
  - [ ] Template: Invoice

#### CLD-051: Webhooks sortants
- **Priority:** Low
- **Phase:** 3
- **Effort:** 6h
- **Description:**
  - [ ] POST /webhooks - Configurer webhook
  - [ ] Events: scan.completed, usage.warning
  - [ ] Retry logic
  - [ ] Signature HMAC

---

## 📅 PLANNING PAR PHASE

### Phase 1: Foundation (6-8 semaines)

| Semaine | Tâches |
|---------|--------|
| S1 | KZN-001, KZN-002, KZN-003 (migration) |
| S2 | KZN-010, KZN-011 (licensing core) |
| S3 | KZN-012, KZN-013, KZN-014 (licensing integration) |
| S4 | KZN-020, KZN-021 (règles premium) |
| S5 | KZN-022, KZN-023 (règles premium) |
| S6 | ZED-001, ZED-002, VSC-001, VSC-002 (extraction) |
| S7 | KZN-030, KZN-031, KZN-040 (docs, tests) |
| S8 | Buffer, fixes, polish |

### Phase 2: SaaS MVP (8-10 semaines)

| Semaine | Tâches |
|---------|--------|
| S9 | CLD-001, CLD-002, CLD-003 (setup) |
| S10 | CLD-010, CLD-011 (auth) |
| S11 | CLD-012 (API keys) |
| S12 | CLD-020, CLD-021 (Stripe) |
| S13 | CLD-030, CLD-031 (dashboard setup, login) |
| S14 | CLD-032, CLD-033 (dashboard pages) |
| S15 | CLD-034, CLD-040 (billing page, docker) |
| S16 | CLD-041, CLD-042 (infra, CI/CD) |
| S17-18 | Testing, fixes, soft launch |

### Phase 3: Enterprise (12+ semaines)

| Focus | Tâches |
|-------|--------|
| Advanced Rules | KZN-024, KZN-025 |
| Enterprise Auth | SSO/SAML integration |
| Webhooks | CLD-051 |
| On-premise | Documentation, scripts |
| Compliance | SOC2 prep, audit logs |

---

## 📊 RÉSUMÉ

| Repo | Tâches | Effort Total |
|------|--------|--------------|
| kaizen | 25 tâches | ~80h |
| zed | 6 tâches | ~7h |
| vscode | 7 tâches | ~8h |
| cloud | 25 tâches | ~100h |
| **TOTAL** | **63 tâches** | **~195h** |

---

## 🔗 DÉPENDANCES CRITIQUES

```
KZN-001 (transfer repo)
    │
    ├──► KZN-002 (update URLs)
    │
    └──► KZN-010 (licensing.rs)
              │
              ├──► KZN-011 (CLI integration)
              ├──► KZN-012 (LSP integration)
              ├──► KZN-013 (auth command)
              └──► KZN-014 (tier filtering)
                        │
                        └──► KZN-020+ (premium rules)

ZED-001 + VSC-001 (extract extensions)
    │
    └──► KZN-004 (cleanup monorepo)

CLD-010 (OAuth)
    │
    └──► CLD-011 (sessions)
              │
              └──► CLD-012 (API keys)
                        │
                        └──► CLD-020 (Stripe)
```
