# PRD: Kaizen Subscription & Licensing System

**Version:** 1.0
**Date:** 2025-12-19
**Status:** Draft
**Author:** Product Team

---

## Executive Summary

Ce document définit la stratégie de monétisation de Kaizen, un analyseur statique JavaScript/TypeScript. L'objectif est de transformer un outil open-source gratuit en un produit SaaS viable tout en préservant la confiance de la communauté et l'accessibilité pour les développeurs individuels.

**Modèle choisi : Open Core + Freemium SaaS**

---

## 1. Context & Problem Statement

### 1.1 Situation Actuelle

| Aspect | État |
|--------|------|
| **Licence** | MIT (100% permissive) |
| **Revenus** | $0 |
| **Architecture** | CLI + LSP, 100% local |
| **Règles** | 20 (13 qualité + 7 sécurité) |
| **Utilisateurs** | Open source, communauté |

### 1.2 Objectifs Business

1. **Générer des revenus récurrents** pour financer le développement
2. **Maintenir l'open source** pour l'adoption et la crédibilité
3. **Créer de la valeur premium** sans aliéner la communauté
4. **Établir un modèle scalable** pour l'entreprise

### 1.3 Contraintes

- Le plugin Zed DOIT rester open source (demande explicite)
- Les extensions IDE (VS Code, Zed) restent gratuites
- Pas de vendor lock-in agressif
- Respecter l'esprit open source de la communauté sécurité

---

## 2. Component Analysis: What Stays Open vs Proprietary

### 2.1 Architecture Actuelle

```
┌─────────────────────────────────────────────────────────────────┐
│                    KAIZEN ECOSYSTEM                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────┐  ┌─────────────────┐  ┌────────────────┐  │
│  │   kaizen-cli    │  │   kaizen-lsp    │  │  kaizen-core   │  │
│  │   (1,000 LOC)   │  │   (1,500 LOC)   │  │  (12,000 LOC)  │  │
│  └────────┬────────┘  └────────┬────────┘  └────────┬───────┘  │
│           │                    │                    │          │
│           └──────────────┬─────┴────────────────────┘          │
│                          │                                      │
│                          ▼                                      │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                    EDITORS / INTEGRATIONS                   ││
│  ├────────────────┬────────────────┬───────────────────────────┤│
│  │  Plugin Zed    │  Extension VS  │  GitHub Actions           ││
│  │  (27 LOC WASM) │  Code (TypeScript) │  (YAML workflow)      ││
│  └────────────────┴────────────────┴───────────────────────────┘│
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 Décision: Open Source vs Propriétaire

| Component | Décision | Raison |
|-----------|----------|--------|
| **kaizen-core** | Open Source (MIT) | Moteur d'analyse, crédibilité sécurité |
| **kaizen-cli** | Open Source (MIT) | Point d'entrée principal, adoption |
| **kaizen-lsp** | Open Source (MIT) | Intégration IDE, developer experience |
| **Plugin Zed** | Open Source (MIT) | Demande explicite, thin wrapper |
| **Extension VS Code** | Open Source (MIT) | Standard industrie |
| **Règles de base (20)** | Open Source | Valeur gratuite, adoption |
| **Règles avancées** | Propriétaire | Différenciateur premium |
| **Backend API** | Propriétaire | Monétisation SaaS |
| **Dashboard Web** | Propriétaire | Valeur entreprise |
| **Base de vulnérabilités** | Hybrid | NVD (public) + enrichissement (privé) |

### 2.3 Justification du Plugin Zed Open Source

Le plugin Zed est un **thin wrapper** de 27 lignes qui :
- N'a AUCUNE logique métier
- Délègue entièrement au LSP
- Peut être détaché dans un repo séparé
- Utilise le protocole LSP standard

**Recommandation :** Créer un repo séparé `github.com/mpiton/kaizen-zed` pour clarifier la gouvernance et faciliter les contributions.

---

## 3. Subscription Tiers

### 3.1 Pricing Strategy

Basé sur l'analyse de Snyk, GitLab, et Trivy :

| Plan | Prix | Cible | Modèle |
|------|------|-------|--------|
| **Free** | $0 | Individus, OSS | Illimité local |
| **Pro** | €29/mois | Équipes (≤10) | Per-team |
| **Enterprise** | Sur devis | Entreprises | Per-seat + features |

### 3.2 Feature Matrix

```
┌─────────────────────────────────────┬──────────┬──────────┬─────────────┐
│ Feature                             │   Free   │   Pro    │ Enterprise  │
├─────────────────────────────────────┼──────────┼──────────┼─────────────┤
│ RÈGLES                              │          │          │             │
├─────────────────────────────────────┼──────────┼──────────┼─────────────┤
│ Règles qualité (Q001-Q034)          │    ✓     │    ✓     │     ✓       │
│ Règles sécurité de base (S001-S012) │    ✓     │    ✓     │     ✓       │
│ Règles avancées (S020-S024)         │    ✗     │    ✓     │     ✓       │
│ Framework rules (React, Vue, Next)  │    ✗     │    ✓     │     ✓       │
│ Custom rules DSL                    │    ✗     │    ✗     │     ✓       │
├─────────────────────────────────────┼──────────┼──────────┼─────────────┤
│ INTÉGRATIONS                        │          │          │             │
├─────────────────────────────────────┼──────────┼──────────┼─────────────┤
│ CLI local                           │    ✓     │    ✓     │     ✓       │
│ LSP (IDE integration)               │    ✓     │    ✓     │     ✓       │
│ GitHub Actions                      │    ✓     │    ✓     │     ✓       │
│ GitLab CI                           │    ✗     │    ✓     │     ✓       │
│ Jenkins, CircleCI                   │    ✗     │    ✗     │     ✓       │
├─────────────────────────────────────┼──────────┼──────────┼─────────────┤
│ FONCTIONNALITÉS                     │          │          │             │
├─────────────────────────────────────┼──────────┼──────────┼─────────────┤
│ Scans illimités locaux              │    ✓     │    ✓     │     ✓       │
│ Fichiers par scan                   │   100    │ Illimité │  Illimité   │
│ AI-powered suggestions              │    ✗     │    ✓     │     ✓       │
│ API REST                            │    ✗     │    ✗     │     ✓       │
│ Dashboard web                       │    ✗     │    ✓     │     ✓       │
│ Historique & analytics              │    ✗     │  30 days │   1 year    │
├─────────────────────────────────────┼──────────┼──────────┼─────────────┤
│ SUPPORT                             │          │          │             │
├─────────────────────────────────────┼──────────┼──────────┼─────────────┤
│ Community (GitHub Issues)           │    ✓     │    ✓     │     ✓       │
│ Email support                       │    ✗     │    ✓     │     ✓       │
│ Priority support (SLA)              │    ✗     │    ✗     │     ✓       │
│ Dedicated CSM                       │    ✗     │    ✗     │     ✓       │
├─────────────────────────────────────┼──────────┼──────────┼─────────────┤
│ SÉCURITÉ ENTERPRISE                 │          │          │             │
├─────────────────────────────────────┼──────────┼──────────┼─────────────┤
│ SSO/SAML                            │    ✗     │    ✗     │     ✓       │
│ Audit logs                          │    ✗     │    ✗     │     ✓       │
│ On-premise deployment               │    ✗     │    ✗     │     ✓       │
│ Compliance reports (SOC2, etc)      │    ✗     │    ✗     │     ✓       │
└─────────────────────────────────────┴──────────┴──────────┴─────────────┘
```

### 3.3 Règles Premium (Détail)

| Rule ID | Nom | Description | Tier |
|---------|-----|-------------|------|
| S020 | Prototype Pollution | Détection de patterns `Object.assign()` dangereux | Pro |
| S021 | Regex DoS | Détection d'expressions régulières exponentielles | Pro |
| S022 | Unsafe Deserialization | Détection de chaînes `JSON.parse()` → `eval()` | Pro |
| S023 | Path Traversal | Détection d'accès fichiers non sanitisés | Pro |
| S024 | SSRF | Server-Side Request Forgery detection | Pro |
| Q050 | React Hooks Rules | useEffect cleanup, dependencies exhaustives | Pro |
| Q051 | Vue Composition | Patterns composition API | Pro |
| Q052 | Next.js Boundaries | Server/client component boundary checks | Pro |

---

## 4. Technical Architecture

### 4.1 API Key System

#### Format de clé

```
kz_[tier]_[org_id]_[timestamp]_[signature]

Exemple: kz_pro_org123_1702000000_a1b2c3d4e5f6
```

#### Validation Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                      API KEY VALIDATION                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  User sets KAIZEN_API_KEY environment variable                  │
│                          │                                      │
│                          ▼                                      │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │ CLI/LSP starts                                              ││
│  │ 1. Check for KAIZEN_API_KEY env var                         ││
│  │ 2. Or check ~/.kaizen/credentials                           ││
│  │ 3. Or check kaizen.toml [license] section                   ││
│  └────────────────────────────┬────────────────────────────────┘│
│                               │                                 │
│              ┌────────────────┴────────────────┐                │
│              │                                 │                │
│              ▼                                 ▼                │
│  ┌───────────────────────┐        ┌────────────────────────────┐│
│  │ LOCAL VALIDATION      │        │ REMOTE VALIDATION (future) ││
│  │ (Default, no network) │        │ (Opt-in, more features)    ││
│  ├───────────────────────┤        ├────────────────────────────┤│
│  │ • HMAC signature check│        │ • Real-time validation     ││
│  │ • Timestamp expiry    │        │ • Usage tracking           ││
│  │ • Tier extraction     │        │ • Revocation support       ││
│  │ • Offline capable     │        │ • Analytics                ││
│  └───────────┬───────────┘        └─────────────┬──────────────┘│
│              │                                  │               │
│              └────────────────┬─────────────────┘               │
│                               │                                 │
│                               ▼                                 │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │ LicenseInfo { tier, features, valid_until }                 ││
│  │                                                             ││
│  │ → Configure RuleRegistry with tier-appropriate rules        ││
│  │ → Enable/disable features based on plan                     ││
│  │ → Display tier status to user                               ││
│  └─────────────────────────────────────────────────────────────┘│
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 4.2 Backend Architecture (Phase 2+)

```
┌─────────────────────────────────────────────────────────────────────┐
│                        KAIZEN SAAS BACKEND                          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  CLIENTS                                                            │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌───────────┐              │
│  │   CLI    │ │ GitHub   │ │  GitLab  │ │ Dashboard │              │
│  │          │ │ Action   │ │    CI    │ │    Web    │              │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └─────┬─────┘              │
│       │            │            │             │                     │
│       └────────────┴────────────┴─────────────┘                     │
│                          │                                          │
│                          ▼                                          │
│  ┌──────────────────────────────────────────────────────────────────┤
│  │                    API GATEWAY                                   │
│  │  • Rate limiting (by tier)                                       │
│  │  • API key validation                                            │
│  │  • Request routing                                               │
│  │  • HTTPS termination                                             │
│  └────────────────────────────┬─────────────────────────────────────┤
│                               │                                     │
│       ┌───────────────────────┼───────────────────────┐             │
│       │                       │                       │             │
│       ▼                       ▼                       ▼             │
│  ┌──────────┐          ┌──────────┐           ┌──────────┐          │
│  │   Auth   │          │   Scan   │           │ Reports  │          │
│  │ Service  │          │ Service  │           │ Service  │          │
│  ├──────────┤          ├──────────┤           ├──────────┤          │
│  │• Login   │          │• Async   │           │• Generate│          │
│  │• API keys│          │  scans   │           │  SARIF   │          │
│  │• SSO     │          │• Queue   │           │• PDF     │          │
│  │• Billing │          │  mgmt    │           │• JSON    │          │
│  └────┬─────┘          └────┬─────┘           └────┬─────┘          │
│       │                     │                      │                │
│       └─────────────────────┼──────────────────────┘                │
│                             │                                       │
│                             ▼                                       │
│  ┌──────────────────────────────────────────────────────────────────┤
│  │                      DATA LAYER                                  │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐                 │
│  │  │ PostgreSQL │  │   Redis    │  │    S3      │                 │
│  │  │ (users,    │  │ (cache,    │  │ (reports,  │                 │
│  │  │  orgs,     │  │  rate      │  │  SBOMs)    │                 │
│  │  │  scans)    │  │  limits)   │  │            │                 │
│  │  └────────────┘  └────────────┘  └────────────┘                 │
│  └──────────────────────────────────────────────────────────────────┤
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### 4.3 Rate Limiting

| Plan | Requests/min | Requests/day | Concurrent scans |
|------|--------------|--------------|------------------|
| Free | 10 | 100 | 1 |
| Pro | 100 | 10,000 | 5 |
| Enterprise | 1,000 | Illimité | 50+ |

### 4.4 OAuth Device Flow (CLI Authentication)

```
┌─────────────────────────────────────────────────────────────────┐
│                    CLI LOGIN FLOW                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  $ kaizen auth login                                            │
│                                                                 │
│  1. CLI → POST /auth/device                                     │
│     Response: { device_code, user_code, verification_uri }     │
│                                                                 │
│  2. CLI displays:                                               │
│     "Open https://kaizen.dev/device and enter code: ABCD-1234" │
│                                                                 │
│  3. User opens browser, enters code                             │
│     - Authenticates via OAuth (GitHub, Google, etc)             │
│     - Authorizes Kaizen CLI                                     │
│                                                                 │
│  4. CLI polls GET /auth/device/token (every 5s)                │
│     Until: token returned or timeout (15 min)                   │
│                                                                 │
│  5. CLI stores token securely:                                  │
│     - macOS: Keychain                                           │
│     - Linux: secret-tool / encrypted file                       │
│     - Windows: Credential Manager                               │
│                                                                 │
│  6. Future commands use stored token                            │
│     $ kaizen check ./src  # Uses stored credentials             │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 5. Implementation Roadmap

### Phase 1: Foundation (Q1 2025) - 6-8 weeks

**Objectif:** Licensing local fonctionnel

| Task | Effort | Priority |
|------|--------|----------|
| Créer `kaizen-core/src/licensing.rs` | 2 days | P0 |
| Intégrer validation dans CLI | 1 day | P0 |
| Intégrer validation dans LSP | 1 day | P0 |
| Créer 3 règles premium (S020-S022) | 2 weeks | P0 |
| Documenter le système de licensing | 2 days | P1 |
| Créer landing page pricing | 1 week | P1 |
| Setup Stripe pour paiements | 3 days | P1 |
| Générer clés manuellement (admin tool) | 2 days | P1 |

**Deliverables:**
- [x] Validation API key locale
- [x] 3+ règles premium
- [x] Page pricing
- [x] Paiement Stripe

### Phase 2: SaaS MVP (Q2 2025) - 8-12 weeks

**Objectif:** Backend fonctionnel avec dashboard

| Task | Effort | Priority |
|------|--------|----------|
| Backend API (Rust/Axum ou Node) | 4 weeks | P0 |
| User authentication (OAuth) | 1 week | P0 |
| Key generation/management | 1 week | P0 |
| Dashboard web (React/Vue) | 3 weeks | P0 |
| Billing integration (Stripe) | 1 week | P0 |
| Rate limiting (Redis) | 3 days | P1 |
| Usage analytics | 1 week | P1 |

**Deliverables:**
- [x] API REST fonctionnelle
- [x] Dashboard web
- [x] Self-service key management
- [x] Billing automatique

### Phase 3: Enterprise Features (Q3-Q4 2025)

| Feature | Effort | Priority |
|---------|--------|----------|
| SSO/SAML integration | 2 weeks | P1 |
| Audit logging | 1 week | P1 |
| Custom rules DSL | 4 weeks | P2 |
| AI-powered suggestions | 4 weeks | P2 |
| On-premise deployment | 4 weeks | P2 |
| Advanced CI/CD integrations | 2 weeks | P2 |

---

## 6. OSS Strategy

### 6.1 Open Source Program

**Objectif:** Maintenir la crédibilité et l'adoption communautaire

| Initiative | Description |
|------------|-------------|
| **OSS Free Pro** | Projets open source vérifiés : Pro tier gratuit |
| **Education** | Étudiants/enseignants : Pro tier gratuit |
| **Startup Credits** | Startups < 10 personnes : 6 mois Pro gratuit |
| **Contributors** | Contributeurs actifs : Pro tier permanent |

### 6.2 Verification Process

```
1. User requests OSS license at kaizen.dev/oss
2. Provide GitHub repo URL
3. Automated checks:
   - Public repository ✓
   - OSI-approved license ✓
   - Active development (commits < 6 months) ✓
4. If pass → Auto-generate Pro key
5. If fail → Manual review
```

### 6.3 Community Engagement

- **GitHub Discussions** : Forum principal
- **Discord** : Support communautaire
- **Blog** : Tutorials, release notes
- **Contribute** : Guidelines pour contribuer des règles

---

## 7. Competitive Analysis

### 7.1 Positioning

```
                    ┌─────────────────────────────────────┐
                    │         ENTERPRISE FOCUS            │
                    │                                     │
         Snyk ●     │                        ● Checkmarx │
                    │                                     │
                    │                                     │
    ────────────────┼─────────────────────────────────────
    OPEN            │                              CLOSED
    SOURCE          │                              SOURCE
                    │                                     │
                    │                                     │
        Trivy ●     │     ● KAIZEN (target)              │
                    │                                     │
                    │         DEVELOPER FOCUS             │
                    └─────────────────────────────────────┘
```

### 7.2 Key Differentiators

| Aspect | Kaizen | Snyk | Trivy |
|--------|--------|------|-------|
| **Pricing** | Freemium agressif | Cher | Gratuit |
| **Focus** | JS/TS uniquement | Multi-language | Containers |
| **Taint Analysis** | Avancée | Basique | Non |
| **Local-first** | Oui | Non (SaaS) | Oui |
| **IDE Integration** | Natif | Plugin | Non |
| **Open Source** | Core open | Fermé | 100% open |

### 7.3 Messaging

**Tagline:** "Enterprise-grade JavaScript security, developer-friendly pricing"

**Value Props:**
1. Ultra-fast (Rust-based) - 10x faster than ESLint
2. Deep taint analysis for real vulnerabilities
3. IDE-native experience (not an afterthought)
4. Open core you can trust and audit

---

## 8. Success Metrics

### 8.1 KPIs

| Metric | Q1 Target | Q2 Target | Q4 Target |
|--------|-----------|-----------|-----------|
| **Active Users (Free)** | 1,000 | 5,000 | 20,000 |
| **Paid Subscribers** | 10 | 100 | 500 |
| **MRR** | €500 | €3,000 | €15,000 |
| **Churn Rate** | N/A | <5% | <3% |
| **NPS** | N/A | >40 | >50 |

### 8.2 Conversion Funnel

```
Download CLI → Activate Free → Hit premium feature → Convert to Pro
   100%           60%                15%                  5%
```

---

## 9. Risks & Mitigations

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Community backlash | Medium | High | Generous free tier, transparent communication |
| Key piracy | Low | Medium | HMAC signatures, periodic rotation |
| Competitor response | Medium | Medium | Focus on DX and performance |
| Slow adoption | Medium | High | Strong content marketing, OSS program |
| Backend complexity | Medium | Medium | Start simple, iterate |

---

## 10. Open Questions

1. **Limite fichiers Free tier ?**
   - Option A: 100 fichiers/scan (forcer upgrade pour gros projets)
   - Option B: Illimité (monétiser uniquement sur features)
   - **Recommandation:** Option B pour éviter friction

2. **Règles communautaires ?**
   - Accepter des contributions de règles ?
   - Si oui, restent-elles gratuites ou peuvent devenir premium ?
   - **Recommandation:** Contributions = toujours gratuites (crédibilité OSS)

3. **Offline mode Enterprise ?**
   - Full offline avec license file ?
   - Periodic phone-home (24h, 7j) ?
   - **Recommandation:** License file + phone-home 30j pour Enterprise

---

## 11. Appendix

### A. API Key Format Specification

```
Format: kz_[tier]_[org_id]_[timestamp]_[signature]

tier      := "free" | "pro" | "ent"
org_id    := base62(uuid)[0:8]    # 8 chars, unique org identifier
timestamp := unix_epoch           # Key creation time
signature := hmac_sha256(         # Truncated to 16 chars
               key = server_secret,
               msg = "{tier}_{org_id}_{timestamp}"
             )[0:16]

Example:
kz_pro_a1B2c3D4_1702000000_e5f6g7h8i9j0k1l2
```

### B. Supported CI/CD Platforms

| Platform | Free | Pro | Enterprise |
|----------|------|-----|------------|
| GitHub Actions | ✓ | ✓ | ✓ |
| GitLab CI | - | ✓ | ✓ |
| CircleCI | - | - | ✓ |
| Jenkins | - | - | ✓ |
| Azure DevOps | - | - | ✓ |
| Bitbucket Pipelines | - | ✓ | ✓ |

### C. Database Schema (Draft)

```sql
-- Users
CREATE TABLE users (
  id UUID PRIMARY KEY,
  email VARCHAR(255) UNIQUE NOT NULL,
  name VARCHAR(255),
  created_at TIMESTAMP DEFAULT NOW(),
  updated_at TIMESTAMP DEFAULT NOW()
);

-- Organizations
CREATE TABLE organizations (
  id UUID PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  tier VARCHAR(20) DEFAULT 'free',
  stripe_customer_id VARCHAR(255),
  created_at TIMESTAMP DEFAULT NOW()
);

-- API Keys
CREATE TABLE api_keys (
  id UUID PRIMARY KEY,
  org_id UUID REFERENCES organizations(id),
  key_hash VARCHAR(64) NOT NULL,  -- SHA256 of key
  key_prefix VARCHAR(20) NOT NULL, -- For display: kz_pro_a1B2...
  tier VARCHAR(20) NOT NULL,
  created_at TIMESTAMP DEFAULT NOW(),
  expires_at TIMESTAMP,
  revoked_at TIMESTAMP,
  last_used_at TIMESTAMP
);

-- Scans (for analytics)
CREATE TABLE scans (
  id UUID PRIMARY KEY,
  org_id UUID REFERENCES organizations(id),
  api_key_id UUID REFERENCES api_keys(id),
  files_count INTEGER,
  findings_count INTEGER,
  duration_ms INTEGER,
  created_at TIMESTAMP DEFAULT NOW()
);
```

---

**Document Status:** Draft
**Next Review:** 2025-01-15
**Approvers:** Product, Engineering, Legal
