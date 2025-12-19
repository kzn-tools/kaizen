# POINTS D'EXTENSION & STRATÉGIE DE MONÉTISATION - KAIZEN

## Vue d'Ensemble

Kaizen est actuellement un analyseur statique **gratuit, open-source et sans API key**. Ce document détaille où et comment injecter une validation d'API key et quelles fonctionnalités peuvent devenir "premium".

---

## 1. ARCHITECTURE ACTUELLE (SANS API KEY)

### Flux d'utilisation actuel

```
CLI (kaizen check)
    ↓
Load config (kaizen.toml) - local file
    ↓
Create AnalysisEngine (free rules only)
    ↓
Analyze files (in-memory, no network)
    ↓
Output results (json, sarif, pretty)
```

### Aucune dépendance externe

- Pas de fichiers téléchargés
- Pas d'appels réseau
- Pas de validation license
- Tout est en dur dans le binaire
- Configuration locale uniquement

---

## 2. OÙ AJOUTER LA VALIDATION API KEY

### 2.1 Option A: Minimal (CLI only)

**Avantage:** Impacte seulement l'utilisation CLI, LSP reste gratuit

**Implémentation (kaizen-cli/src/commands/check.rs):**

```rust
pub fn run(&self) -> Result<()> {
    // 1. Vérifier si règles premium demandées
    let uses_premium = self.config.rules.iter().any(|r| r.is_premium());

    if uses_premium {
        // 2. Chercher API key
        let api_key = std::env::var("KAIZEN_API_KEY").ok();

        if api_key.is_none() {
            // Option: warning ou error
            eprintln!("Warning: Premium rules disabled without KAIZEN_API_KEY");
            // Continuer avec règles gratuites seulement
        } else if let Some(key) = api_key {
            // 3. Valider key (local check ou remote)
            match validate_api_key(&key) {
                Ok(plan) => {
                    // 4. Configurer engine avec plan premium
                    engine.set_premium_tier(plan);
                }
                Err(e) => {
                    eprintln!("Error: Invalid API key - {}", e);
                    process::exit(1);
                }
            }
        }
    }

    // 5. Continuer analyse
    engine.analyze(&file)
}

fn validate_api_key(key: &str) -> Result<PremiumPlan> {
    // Option 1: Validation locale (HMAC signature)
    if key.starts_with("kz_") {
        return validate_local_key(key);
    }

    // Option 2: Validation distante (requires network)
    // validate_remote_key(key).await?

    Err("Invalid key format")
}
```

**Configuration kaizen.toml:**

```toml
[rules]
# Règles gratuites (toujours disponibles)
quality = true
security = true

# Règles premium (requièrent API key)
# [rules.premium]
# advanced-patterns = true
# ai-suggestions = true
```

---

### 2.2 Option B: Stricter (Tout requiert clé)

**Avantage:** Monétisation plus claire, adoption plus facile pour modèle freemium

**Implémentation:**

```rust
// Dans AnalysisEngine::new()
pub fn new() -> Self {
    // 1. Toujours chercher API key (CLI & LSP)
    let api_key = self.get_api_key();

    // 2. Déterminer tier
    let tier = match api_key {
        None => FreeTier::default(),
        Some(key) => validate_key(&key).unwrap_or(FreeTier::default()),
    };

    // 3. Initialiser uniquement règles du tier
    Self {
        registry: create_registry_for_tier(tier),
        api_key,
        tier,
    }
}

fn get_api_key() -> Option<String> {
    // Priority: env var > config file > ~/.kaizen/key.txt
    std::env::var("KAIZEN_API_KEY").ok()
        .or_else(|| config.load_api_key().ok())
        .or_else(|| read_global_key().ok())
}
```

**Tiers:**

```rust
pub enum PremiumTier {
    Free {
        rules: Vec<String>,  // 7 basic security + 5 quality
        max_files_per_run: usize,  // 100
    },
    Pro {
        rules: Vec<String>,  // Tous + 5 advanced
        max_files_per_run: usize,  // Illimité
        features: ["ai-suggestions", "custom-rules"],
    },
    Enterprise {
        // Tout débloquer
    },
}
```

---

### 2.3 Option C: Hybrid (LSP free, CLI premium)

**Avantage:** Développeurs get IDE feedback free, entreprises pay pour CI/CD

**Implémentation:**

```rust
// kaizen-lsp/src/server.rs
async fn analyze_and_publish(&self, uri: &Url) {
    // LSP: Toujours gratuit, règles basiques
    let engine = AnalysisEngine::create_free_tier();
    let diagnostics = engine.analyze(&doc);

    self.client.publish_diagnostics(uri.clone(), diagnostics, None).await;
}

// kaizen-cli/src/commands/check.rs
fn run(&self) -> Result<()> {
    // CLI: Peut être premium
    let api_key = std::env::var("KAIZEN_API_KEY").ok();
    let engine = match api_key {
        Some(key) => AnalysisEngine::with_premium_key(&key)?,
        None => AnalysisEngine::create_free_tier(),
    };

    engine.analyze_files(&paths)
}
```

---

## 3. STRUCTURE INTERNE: OÙ AJOUTER LE CODE

### 3.1 New Module: `kaizen-core/src/licensing.rs`

```rust
//! Licensing and API key validation

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PremiumTier {
    Free,
    Pro,
    Enterprise,
}

#[derive(Debug, Clone)]
pub struct LicenseInfo {
    pub tier: PremiumTier,
    pub api_key: String,
    pub valid_until: Option<chrono::DateTime<chrono::Utc>>,
    pub features: Vec<String>,
}

pub struct LicenseValidator {
    // Pour local validation
    signing_key: &'static str,  // Clé HMAC Kaizen privée
}

impl LicenseValidator {
    pub fn new() -> Self {
        Self {
            signing_key: "kaizen-secret-key-not-real",
        }
    }

    /// Valider une clé localement (pas besoin réseau)
    pub fn validate_local(&self, api_key: &str) -> Result<LicenseInfo, LicenseError> {
        // Format: kz_[tier]_[timestamp]_[hmac]
        // Exemple: kz_pro_1702000000_abc123def456

        let parts: Vec<&str> = api_key.split('_').collect();
        if parts.len() != 4 || parts[0] != "kz" {
            return Err(LicenseError::InvalidFormat);
        }

        let tier_str = parts[1];
        let timestamp_str = parts[2];
        let provided_hmac = parts[3];

        // 1. Valider format tier
        let tier = match tier_str {
            "pro" => PremiumTier::Pro,
            "ent" => PremiumTier::Enterprise,
            _ => return Err(LicenseError::InvalidTier),
        };

        // 2. Valider timestamp (ne pas accepter clés trop vieilles)
        let timestamp: u64 = timestamp_str
            .parse()
            .map_err(|_| LicenseError::InvalidTimestamp)?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if now > timestamp + (365 * 24 * 3600) {
            // Clé expirée > 1 an
            return Err(LicenseError::Expired);
        }

        // 3. Valider HMAC
        let message = format!("{}_{}", tier_str, timestamp_str);
        let computed_hmac = self.compute_hmac(&message);

        if !constant_time_compare(computed_hmac.as_bytes(), provided_hmac.as_bytes()) {
            return Err(LicenseError::InvalidSignature);
        }

        Ok(LicenseInfo {
            tier,
            api_key: api_key.to_string(),
            valid_until: Some(chrono::DateTime::from_timestamp(timestamp as i64, 0).unwrap()),
            features: tier.features(),
        })
    }

    fn compute_hmac(&self, message: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.signing_key.hash(&mut hasher);
        message.hash(&mut hasher);

        format!("{:x}", hasher.finish())
    }
}

impl PremiumTier {
    pub fn features(&self) -> Vec<String> {
        match self {
            PremiumTier::Free => vec![
                "basic-quality-rules",
                "basic-security-rules",
            ],
            PremiumTier::Pro => vec![
                "all-quality-rules",
                "all-security-rules",
                "advanced-patterns",
                "ai-suggestions",
            ],
            PremiumTier::Enterprise => vec![
                "all-features",
                "custom-rules",
                "api-access",
                "support",
            ],
        }
    }

    pub fn is_free(&self) -> bool {
        matches!(self, PremiumTier::Free)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LicenseError {
    #[error("Invalid API key format")]
    InvalidFormat,

    #[error("Invalid tier")]
    InvalidTier,

    #[error("Invalid timestamp")]
    InvalidTimestamp,

    #[error("API key expired")]
    Expired,

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Network error")]
    NetworkError,
}

fn constant_time_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (a_byte, b_byte) in a.iter().zip(b.iter()) {
        result |= a_byte ^ b_byte;
    }

    result == 0
}
```

### 3.2 Update: `kaizen-core/src/lib.rs`

```rust
pub mod licensing;

pub use licensing::{LicenseValidator, PremiumTier, LicenseInfo};
```

### 3.3 Update: `kaizen-core/src/rules/mod.rs`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuleCategory {
    Quality,
    Security,
    AdvancedPatterns,  // NEW: Nécessite tier Pro+
}

impl RuleCategory {
    pub fn min_tier(&self) -> licensing::PremiumTier {
        match self {
            RuleCategory::Quality => licensing::PremiumTier::Free,
            RuleCategory::Security => licensing::PremiumTier::Free,
            RuleCategory::AdvancedPatterns => licensing::PremiumTier::Pro,
        }
    }
}

pub struct RuleRegistry {
    rules: Vec<Box<dyn Rule>>,
    disabled_rules: HashSet<String>,
    severity_overrides: HashMap<String, Severity>,
    quality_enabled: bool,
    security_enabled: bool,
    premium_tier: licensing::PremiumTier,  // NEW
}

impl RuleRegistry {
    pub fn should_run_rule(&self, rule: &dyn Rule) -> bool {
        let metadata = rule.metadata();

        // Check tier
        if metadata.category.min_tier() as u8 > self.premium_tier as u8 {
            return false;  // Tier insuffisant
        }

        // ... reste de la logique ...
    }

    pub fn set_tier(&mut self, tier: licensing::PremiumTier) {
        self.premium_tier = tier;
    }
}
```

### 3.4 Update: `kaizen-cli/src/commands/check.rs`

```rust
use kaizen_core::licensing::{LicenseValidator, LicenseInfo};

pub fn run(&self) -> Result<()> {
    // 1. Chercher API key
    let api_key = std::env::var("KAIZEN_API_KEY").ok();

    // 2. Valider si présente
    let license_info = if let Some(key) = api_key {
        let validator = LicenseValidator::new();
        match validator.validate_local(&key) {
            Ok(info) => {
                eprintln!("{} Tier {} activated", "✓".green(), format!("{:?}", info.tier).cyan());
                Some(info)
            }
            Err(e) => {
                eprintln!("{} License error: {}", "✗".red(), e);
                eprintln!("Continuing with free tier...");
                None
            }
        }
    } else {
        None
    };

    // 3. Créer engine avec tier approprié
    let mut engine = AnalysisEngine::with_config(&config);
    if let Some(info) = license_info {
        engine.set_license_tier(info.tier);
    }

    // 4. Reste du code...
}
```

### 3.5 Update: `kaizen-lsp/src/server.rs`

```rust
#[tower_lsp::async_trait]
impl LanguageServer for KaizenLanguageServer {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        // Récupérer API key depuis workspace settings
        if let Some(init_options) = params.initialization_options {
            if let Ok(kaizen_config) = serde_json::from_value::<KaizenConfig>(init_options) {
                if let Some(api_key) = kaizen_config.api_key {
                    let validator = LicenseValidator::new();
                    match validator.validate_local(&api_key) {
                        Ok(info) => {
                            self.license_info.store(Some(info));
                        }
                        Err(_) => {
                            // Fail silently, use free tier
                        }
                    }
                }
            }
        }

        Ok(InitializeResult {
            capabilities: server_capabilities(),
            ..Default::default()
        })
    }
}
```

---

## 4. QUELLES FONCTIONNALITÉS SONT "PREMIUM"?

### 4.1 Scoring Framework

Pour décider si une feature est premium, évaluer:

| Critère | Score | Notes |
|---------|-------|-------|
| **Coût de maintenance** | Haut = premium | Règles complexes coûtent cher à maintenir |
| **Cas d'usage** | Enterprise = premium | Features pour grandes entreprises |
| **Distinctif** | Unique = premium | Fonctionnalités pas chez competitors |
| **Support requis** | Oui = premium | Features nécessitant beaucoup de support |
| **Valeur perçue** | Haut = premium | Willingness to pay |

---

### 4.2 Règles Premium Suggérées

**Candidates TRÈS BONNES (High Value):**

1. **S020: Prototype Pollution Detection** (Advanced Pattern)
   - Détecte patterns `Object.assign()` mal utilisés
   - Haut faux positif sans ML
   - Intérêt entreprises: très haut
   - **Score: 8/10 pour premium**

2. **S021: Unsafe Regex DoS** (Advanced Pattern)
   - Détecte regular expressions exponentielles
   - Requiert analyse complexe
   - Très peu d'outils le font bien
   - **Score: 9/10 pour premium**

3. **S022: Deserialization Gadget Chains** (Advanced Pattern)
   - Détecte unsafe `JSON.parse()` + `eval()`
   - Complexe, high faux positif
   - Recherche active en sécurité
   - **Score: 8/10 pour premium**

4. **Q050: Framework-Specific Rules** (Meta)
   - React: hooks rules (useEffect cleanup, etc)
   - Vue: composition API patterns
   - Next.js: server/client boundary
   - **Score: 7/10 pour premium**

5. **A001: AI-Powered Explanations** (Feature)
   - Génère explications naturelles des vulnérabilités
   - Suggestions de fix via LLM
   - Require API keys externes (LLM costs)
   - **Score: 6/10 pour premium**

---

### 4.3 Règles qui restent GRATUITES

**Raisons:**
- Faciles à implémenter et maintenir
- Intérêt universel
- Aide adoption

**Liste:**
- Q001-Q034: Toutes règles qualité actuelles
- S001-S012: Toutes règles sécurité actuelles
- Config basique
- LSP feedback basique

---

### 4.4 Tiers Proposés

```rust
// Dans licensing.rs

pub enum PremiumTier {
    Free,
    Pro,
    Enterprise,
}

impl PremiumTier {
    pub fn rules(&self) -> Vec<&'static str> {
        match self {
            PremiumTier::Free => vec![
                // 13 quality rules
                "Q001", "Q003", "Q004", "Q010", "Q011", "Q020", "Q021",
                "Q022", "Q023", "Q030", "Q031", "Q032", "Q033", "Q034",
                // 7 security rules
                "S001", "S002", "S003", "S005", "S010", "S011", "S012",
            ],
            PremiumTier::Pro => {
                let mut all = Self::Free.rules();
                all.extend(vec![
                    // Advanced patterns
                    "S020", "S021", "S022",
                    // Framework-specific
                    "Q050", "Q051", "Q052",
                    // AI features
                    "A001",
                ]);
                all
            }
            PremiumTier::Enterprise => {
                // Tout + custom
                vec!["*"]
            }
        }
    }

    pub fn features(&self) -> Vec<&'static str> {
        match self {
            PremiumTier::Free => vec![
                "basic-analysis",
                "lsp-support",
                "cli-mode",
                "github-actions",
            ],
            PremiumTier::Pro => {
                let mut all = Self::Free.features();
                all.extend(vec![
                    "advanced-rules",
                    "framework-support",
                    "ai-suggestions",
                    "priority-updates",
                ]);
                all
            }
            PremiumTier::Enterprise => {
                let mut all = Self::Pro.features();
                all.extend(vec![
                    "api-access",
                    "custom-rules",
                    "sso-support",
                    "24/7-support",
                    "audit-logs",
                ]);
                all
            }
        }
    }
}
```

---

## 5. GÉNÉRATION DES CLÉS API

### 5.1 Solution Local (Recommandée pour MVP)

**Format:** `kz_[tier]_[timestamp]_[hmac]`

**Exemple:** `kz_pro_1702000000_a1b2c3d4e5f6`

**Génération (pour Kaizen):**

```rust
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn generate_api_key(tier: &str, valid_days: u64) -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() + (valid_days * 86400);

    let message = format!("{}_{}", tier, timestamp);

    // HMAC signature
    let mut hasher = DefaultHasher::new();
    "kaizen-secret-key-not-real".hash(&mut hasher);
    message.hash(&mut hasher);
    let hmac = format!("{:x}", hasher.finish());

    format!("kz_{}_{}", message, &hmac[..16])
}

// Génération
let pro_key = generate_api_key("pro", 365);  // Valid 1 year
println!("Your API key: {}", pro_key);  // kz_pro_1702000000_a1b2c3d4
```

### 5.2 Solution Remote (Futur)

Pour plus de sécurité:

```rust
// licensing.rs - Mode remote validation

pub async fn validate_remote(api_key: &str) -> Result<LicenseInfo> {
    let client = reqwest::Client::new();
    let response = client
        .post("https://api.kaizen.dev/validate")
        .json(&json!({ "api_key": api_key }))
        .send()
        .await?;

    match response.status() {
        reqwest::StatusCode::OK => {
            response.json::<LicenseInfo>().await
        }
        reqwest::StatusCode::UNAUTHORIZED => {
            Err(LicenseError::InvalidSignature)
        }
        _ => Err(LicenseError::NetworkError),
    }
}
```

**API Backend (futur):**
- Endpoint: `POST /api/validate-key`
- Returns: `{ tier, features, valid_until }`
- Cache local pour offline usage

---

## 6. MÉCANIQUE DE MONÉTISATION

### 6.1 Pricing Model

**Suggested:**

| Plan | Prix | Utilisateurs | Règles | Support |
|------|------|--------------|--------|---------|
| Free | $0/mois | Individuals | 20 rules | Community |
| Pro | $49/mois | Teams (up to 10) | 30+ rules | Email |
| Enterprise | Custom | Unlimited | All | Priority |

**Stratégie d'adoption:**

1. **Fase 1: Free tier robuste**
   - 20 rules de qualité + sécurité de base
   - LSP gratuit pour IDE
   - CLI pour CI/CD
   - GitHub Actions gratuit

2. **Fase 2: Features premium attrayantes**
   - Règles avancées (S020-S022)
   - Framework support (React, Vue)
   - AI-powered suggestions

3. **Fase 3: Enterprise tier**
   - Custom rules DSL
   - API REST
   - SSO, audit logs
   - Priority support

---

### 6.2 Activation Points

```
User journey:

1. Découvre Kaizen (gratuit, open-source)
2. Installe: npm install -g kzn-cli
3. Utilise free tier - happy with basic rules
4. Hits limitation (wants advanced rule)
5. Voit message: "Advanced patterns require Pro plan"
6. Visits website: kaizen.dev/pricing
7. Choisit plan, achète
8. Reçoit API key par email
9. kaizen check --api-key kz_pro_... ./src
10. Analyse avec règles premium
11. Recommande à équipe (network effect)
```

---

### 6.3 Free Tier Limitations

À ajouter après quelques mois (pour pousser adoption):

```rust
// licensing.rs

pub struct UsageMetrics {
    pub files_analyzed: usize,
    pub api_calls: usize,
    pub custom_rules: usize,
}

impl PremiumTier {
    pub fn max_files_per_month(&self) -> Option<usize> {
        match self {
            PremiumTier::Free => Some(1000),  // ~30 files/day
            PremiumTier::Pro => Some(100_000),
            PremiumTier::Enterprise => None,
        }
    }

    pub fn max_concurrent_analyses(&self) -> Option<usize> {
        match self {
            PremiumTier::Free => Some(1),
            PremiumTier::Pro => Some(10),
            PremiumTier::Enterprise => None,
        }
    }
}
```

**Note:** Être doux avec limitations (warning vs error), ne pas aliéner users.

---

## 7. OÙ AJOUTER LE CODE DANS LE PROJET

### Fichiers à créer:

```
crates/kaizen-core/src/
├── licensing.rs             # NEW: Validation API key
└── lib.rs                   # UPDATE: pub mod licensing

crates/kaizen-cli/src/
├── commands/
│   └── check.rs             # UPDATE: Intégrer licensing
└── license_cli.rs           # NEW: Optionnel - kaizen license <key>

crates/kaizen-lsp/src/
└── server.rs                # UPDATE: Initialize avec license
```

### Fichiers à modifier:

1. **kaizen-core/src/rules/mod.rs**
   - Ajouter `RuleCategory::AdvancedPatterns`
   - Marquer rules avancées
   - Filter par tier dans `should_run_rule`

2. **kaizen-cli/src/commands/check.rs**
   - Chercher `KAIZEN_API_KEY` env var
   - Valider et afficher tier
   - Passer tier à engine

3. **kaizen-lsp/src/server.rs**
   - Récupérer key depuis workspace settings
   - Stocker license info dans server state

4. **Cargo.toml** (optionnel)
   - Ajouter chrono pour timestamps
   - Pas besoin d'autres dépendances

---

## 8. ROADMAP MONÉTISATION

### Q1 2025: Fondation
- ✅ Implémenter licensing.rs
- ✅ Ajouter 3 rules avancées (S020, S021, S022)
- ✅ Ajouter validatio API key
- ✅ Documenter pricing model

### Q2 2025: Lancements
- ⏳ Lancer free tier publiquement
- ⏳ Recevoir feedback sur limitations
- ⏳ Ajouter framework support (React, Vue)
- ⏳ Lancer Pro tier

### Q3 2025: Growth
- ⏳ API REST pour intégrations
- ⏳ AI-powered suggestions
- ⏳ Enterprise tier avec SSO

### Q4 2025+: Mature
- ⏳ Dashboard analytics
- ⏳ Custom rules DSL
- ⏳ Integration marketplace

---

## 9. QUESTIONS IMPORTANTANTES À RÉSOUDRE

### 1. Comment éviter les clés piratées?

**Réponses:**
- Format avec HMAC signature (validation locale)
- Clés avec timestamp (expiration)
- Logs optionnels si API remote (Q4+)
- Pas de stockage clés côté serveur (Q1-Q3)

### 2. Et pour les projets open-source?

**Réponses:**
```rust
// Dans licensing.rs
pub fn is_open_source_project() -> bool {
    // Détecter LICENSE file + public GitHub
    std::path::Path::new("LICENSE").exists()
        && is_public_github_repo()
}

// Donner free tier pro pour OSS
```

ou

```
// Website: kaizen.dev/oss
// Open source projects: ask for free key
```

### 3. Aligner avec architecture actuelle?

**Oui, Kaizen est conçu pour être extensible:**
- RuleRegistry déjà support filtering
- Config TOML déjà support override
- Pas de breaking change en ajoutant licensing

### 4. Impact sur performance?

**Négligeable:**
- Validation HMAC: < 1ms
- Aucune network call en local mode
- Cache license info en mémoire

---

## 10. RÉSUMÉ EXÉCUTIF

### Stratégie Recommandée: Hybrid (Opcion C)

1. **Aujourd'hui:** Tout gratuit (option A: minimal est trop complexe)

2. **Dans 3-6 mois:**
   - Ajouter licensing.rs (pas de breaking change)
   - Créer 3-5 règles avancées (S020-S024)
   - Marquer comme premium
   - Lancer free + Pro tiers

3. **Pricing attractif:**
   - Free: Assez bon pour indie developers
   - Pro: $49/mois, pour teams et CI/CD
   - Enterprise: custom, pour grandes entreprises

4. **Growth lever:**
   - OSS projects get free Pro (good PR)
   - GitHub Actions free (adoption)
   - Community rules contributions (engagement)

5. **Long term:**
   - API REST (B2B)
   - Dashboard + analytics
   - Marketplace de règles

### Next Steps

1. **Discuter avec communauté** (GitHub discussions)
   - Seraient-ils OK avec paid advanced features?
   - Quoi de plus utile pour eux?

2. **Implémenter licensing minimal** (1-2 semaines)
   - licensing.rs module
   - Integration dans analysis engine

3. **Créer 3 rules avancées** (2-3 semaines)
   - S020: Prototype Pollution
   - S021: Regex DoS
   - S022: Unsafe Deserialization

4. **Lancer beta Pro** (4-6 weeks)
   - Inviter 50-100 users
   - Collecter feedback
   - Itérer

---

**Document de Stratégie Monétisation Kaizen**
**Date:** 2025-12-19
**Auteur:** Exploration Codebase Deep Dive
