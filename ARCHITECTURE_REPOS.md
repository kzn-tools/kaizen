# Architecture des Repositories Kaizen

## Organisation Choisie : `kzn-tools`

**URL :** https://github.com/kzn-tools

**Pourquoi ce nom :**
- Cohérent avec le package npm `kzn-cli`
- Court et mémorable
- Disponible sur GitHub

---

## Vue d'Ensemble : 4 Repositories

```
┌────────────────────────────────────────────────────────────────────────────┐
│                                                                            │
│                    ORGANISATION: kzn-tools (ou kaizen-sast)                │
│                                                                            │
├────────────────────────────────────────────────────────────────────────────┤
│                                                                            │
│     OPEN SOURCE (MIT)                      PROPRIÉTAIRE (Privé)            │
│     ─────────────────                      ────────────────────            │
│                                                                            │
│     ┌─────────────┐                        ┌─────────────┐                 │
│     │             │                        │             │                 │
│     │   kaizen    │◄───────────────────────│    cloud    │                 │
│     │   (core)    │     utilise l'API      │   (SaaS)    │                 │
│     │             │                        │             │                 │
│     └──────┬──────┘                        └─────────────┘                 │
│            │                                    privé                      │
│            │ fournit le LSP                                                │
│            │                                                               │
│     ┌──────┴──────┐                                                        │
│     │             │                                                        │
│     ▼             ▼                                                        │
│  ┌──────┐    ┌────────┐                                                    │
│  │ zed  │    │ vscode │                                                    │
│  └──────┘    └────────┘                                                    │
│                                                                            │
└────────────────────────────────────────────────────────────────────────────┘
```

---

## Les 4 Repositories

### 1. `kaizen` - Core (Public)

**URL :** `github.com/kzn-tools/kaizen`
**Licence :** MIT
**Description :** Moteur d'analyse, CLI et LSP

```
kaizen/
├── crates/
│   ├── kaizen-core/     ← Moteur d'analyse (12K LOC)
│   ├── kaizen-cli/      ← CLI : kaizen check ./src
│   └── kaizen-lsp/      ← Serveur LSP pour les IDE
├── npm/                 ← Distribution binaires npm
├── docs/
├── LICENSE (MIT)
└── README.md
```

**Contenu :**
- Toutes les règles (gratuites + premium)
- Système de licensing (validation locale)
- Distribution npm et binaires

---

### 2. `zed` - Extension Zed (Public)

**URL :** `github.com/kzn-tools/zed`
**Licence :** MIT
**Description :** Extension Zed (thin wrapper)

```
zed/
├── src/
│   └── lib.rs           ← 27 lignes, appelle kaizen-lsp
├── extension.toml
├── Cargo.toml
├── LICENSE (MIT)
└── README.md
```

**Pourquoi séparé ?**
- Les contributeurs Zed n'ont pas besoin du core
- Publication indépendante sur le marketplace Zed
- Cycle de release découplé

---

### 3. `vscode` - Extension VS Code (Public)

**URL :** `github.com/kzn-tools/vscode`
**Licence :** MIT
**Description :** Extension VS Code

```
vscode/
├── src/
│   └── extension.ts     ← Client LSP TypeScript
├── package.json
├── LICENSE (MIT)
└── README.md
```

**Pourquoi séparé ?**
- Même raisons que Zed
- Langage différent (TypeScript vs Rust)

---

### 4. `cloud` - Backend SaaS (Privé)

**URL :** `github.com/kzn-tools/cloud` (PRIVÉ)
**Licence :** Propriétaire
**Description :** API, Dashboard, Infrastructure

```
cloud/
├── api/                 ← Backend Rust (Axum)
│   ├── src/
│   │   ├── routes/      ← /auth, /keys, /scans
│   │   ├── services/    ← Stripe, analytics
│   │   └── db/          ← PostgreSQL
│   └── Cargo.toml
│
├── dashboard/           ← Frontend React
│   ├── src/
│   └── package.json
│
├── infra/               ← Terraform, Docker
│
└── README.md
```

**Contenu propriétaire :**
- Génération de clés API
- Gestion des abonnements (Stripe)
- Dashboard analytics
- Infrastructure cloud

---

## Schéma des Interactions

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              UTILISATEUR                                    │
└─────────────────────────────────────────────────────────────────────────────┘
         │                           │                          │
         │ IDE                       │ Terminal                 │ CI/CD
         ▼                           ▼                          ▼
┌─────────────────┐         ┌─────────────────┐        ┌─────────────────┐
│   Extension     │         │   CLI           │        │  GitHub Action  │
│   Zed/VSCode    │         │   kaizen check  │        │                 │
└────────┬────────┘         └────────┬────────┘        └────────┬────────┘
         │                           │                          │
         │ LSP Protocol              │                          │
         ▼                           ▼                          ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                           kaizen-lsp / kaizen-cli                           │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                         kaizen-core                                  │   │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────────────┐ │   │
│  │  │  Parser  │→ │ Semantic │→ │  Taint   │→ │  Rules (20 free +    │ │   │
│  │  │  (swc)   │  │ Analysis │  │ Analysis │  │  premium si API key) │ │   │
│  │  └──────────┘  └──────────┘  └──────────┘  └──────────────────────┘ │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│                                    │ Si API key présente                    │
│                                    ▼                                        │
│                         ┌──────────────────┐                                │
│                         │ licensing.rs     │                                │
│                         │ Validation locale│ ←─── KAIZEN_API_KEY            │
│                         └──────────────────┘                                │
└─────────────────────────────────────────────────────────────────────────────┘
                                     │
                                     │ (Optionnel) Validation distante
                                     ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                              CLOUD (Privé)                                  │
│                                                                             │
│  ┌───────────┐    ┌───────────┐    ┌───────────┐    ┌───────────┐          │
│  │    API    │    │ Dashboard │    │  Stripe   │    │ Analytics │          │
│  │  /keys    │    │   Web     │    │  Billing  │    │           │          │
│  └───────────┘    └───────────┘    └───────────┘    └───────────┘          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Tableau Récapitulatif

| Repo | Visibilité | Licence | Langage | Contenu |
|------|------------|---------|---------|---------|
| `kaizen` | Public | MIT | Rust | Core, CLI, LSP, règles |
| `zed` | Public | MIT | Rust (WASM) | Extension Zed |
| `vscode` | Public | MIT | TypeScript | Extension VS Code |
| `cloud` | **Privé** | Propriétaire | Rust + React | API, Dashboard, Infra |

---

## Dépendances entre Repos

```
cloud ──────► kaizen (utilise l'API de licensing)
                │
                │ fournit kaizen-lsp binaire
                ▼
           ┌────┴────┐
           │         │
          zed     vscode
```

**Important :** Les extensions (zed, vscode) ne dépendent PAS du code source de kaizen. Elles utilisent le binaire `kaizen-lsp` qui doit être installé séparément.

---

## Installation Utilisateur

```bash
# 1. Installer le CLI (inclut le LSP)
npm install -g kzn-cli

# 2. Installer l'extension IDE
#    - Zed : Extensions → Search "Kaizen" → Install
#    - VS Code : Extensions → Search "Kaizen" → Install

# 3. (Optionnel) Configurer la clé API pour les features premium
export KAIZEN_API_KEY="kz_pro_xxxxx"
```

---

## Timeline de Migration

| Phase | Quand | Repos | Actions |
|-------|-------|-------|---------|
| **Actuel** | Maintenant | 1 (monorepo) | Continuer développement |
| **Phase 2** | Q2 2025 | 4 repos | Extraire zed, vscode, créer cloud |
| **Maintenance** | Ongoing | 4 repos | Releases coordonnées |

---

## Actions Immédiates

1. **Choisir le nom d'organisation**
2. **Créer l'organisation sur GitHub**
3. **Réserver les noms de repos** (même vides)

Ensuite, quand prêt pour le SaaS :
4. Extraire `editors/zed` → `kzn-tools/zed`
5. Extraire `editors/vscode` → `kzn-tools/vscode`
6. Créer `kzn-tools/cloud` (privé)
