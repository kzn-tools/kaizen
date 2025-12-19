# EXPLORATION COMPLÈTE CODEBASE LYNX/KAIZEN

## Statut du Projet

**Nom du Produit:** Kaizen (anciennement Lynx)
**Statut:** MVP avancé en production
**Langage Principal:** Rust (Edition 2021+)
**Version:** 0.1.0
**Licence:** MIT
**Repository:** https://github.com/mpiton/kaizen

---

## 1. ARCHITECTURE GLOBALE

### 1.1 Structure des Dossiers Principaux

```
lynx/
├── crates/                          # Workspace Rust (3 crates)
│   ├── kaizen-core/                 # Moteur d'analyse principal (~1500 lignes)
│   ├── kaizen-cli/                  # Interface CLI (~1000+ lignes)
│   └── kaizen-lsp/                  # Serveur Language Server Protocol
│
├── editors/                         # Intégrations IDE
│   ├── vscode/                      # Extension VS Code (Node.js)
│   └── zed/                         # Extension Zed (Rust WASM)
│
├── npm/                             # Distribution npm
│   ├── kaizen-cli/                  # Package npm main
│   ├── darwin-x64/                  # Binaires macOS Intel
│   ├── darwin-arm64/                # Binaires macOS ARM64
│   ├── linux-x64/                   # Binaires Linux x64
│   ├── linux-arm64/                 # Binaires Linux ARM64
│   └── win32-x64/                   # Binaires Windows
│
├── docs/                            # Documentation
│   ├── project/
│   │   ├── PRD-Lynx.md              # Product Requirements Document (1330 lignes!)
│   │   └── sprints/                 # 18 sprints documentés en détail
│   └── rules/                       # Documentation des règles
│
├── tests/                           # Tests intégration
│   ├── fixtures/                    # Fichiers de test
│   └── integration/
│
└── scripts/                         # Utilitaires (install, setup-hooks)
```

### 1.2 Technologies Principales

| Composant | Technologie | Raison |
|-----------|-------------|--------|
| **Parser** | `swc_ecma_parser` v22.0 | Parser JS/TS ultra-rapide, support TypeScript natif |
| **AST** | `swc_ecma_ast` v14.0 | AST complet pour Rust |
| **LSP** | `tower-lsp` v0.20 | Implémentation robuste du Language Server Protocol |
| **Async Runtime** | `tokio` v1.48 | Runtime async complet |
| **Parallélisme** | `rayon` v1.10 | Parallélisation par-iterator simple |
| **File Walking** | `walkdir` v2.5 | Traversée répertoire cross-platform |
| **CLI** | `clap` v4.5 | Parsing arguments mature |
| **Config** | `toml` v0.8 | Parsing fichiers TOML |
| **Sérialisation** | `serde` + `serde_json` | Sérialisation structurée |
| **Logging** | `tracing` v0.1 | Structured logging |
| **Testing** | `insta` v1.42 | Snapshot testing pour diagnostics |

---

## 2. LES 3 CRATES PRINCIPAUX

### 2.1 KAIZEN-CORE (Moteur d'analyse)

**Responsabilité:** Toute la logique d'analyse, les règles, le taint analysis

**Modules clés:**

```
kaizen-core/src/
├── lib.rs                   # Point d'entrée
├── analysis.rs              # AnalysisEngine - orchestre l'analyse
├── parser.rs                # Intégration SWC, ParsedFile
├── diagnostic.rs            # Structure Diagnostic avec fixes
├── config.rs                # Chargement kaizen.toml, RulesConfig
├── disable_comments.rs      # Gestion lynx-disable-line/next-line
│
├── semantic/                # Analyse sémantique
│   ├── mod.rs
│   ├── scope.rs             # ScopeTree, résolution de portée
│   ├── symbols.rs           # SymbolTable, Symbol registry
│   ├── types.rs             # DisposableTypesRegistry (pour prefer-using)
│   ├── cfg.rs               # ControlFlowGraph pour unreachable
│   └── visitor.rs           # ScopeBuilder, traversal sémantique
│
├── rules/                   # Système de règles (20 règles implémentées)
│   ├── mod.rs               # Rule trait, RuleRegistry, Severity enum
│   │
│   ├── quality/             # 13 règles de qualité
│   │   ├── no_var.rs                  # Q030: var → let/const
│   │   ├── no_console.rs              # Q032: console.* detection
│   │   ├── no_eval.rs                 # Q034: eval() interdiction
│   │   ├── eqeqeq.rs                  # Q033: == → ===
│   │   ├── no_unused_vars.rs          # Q001: Variable non utilisée
│   │   ├── no_unused_imports.rs       # Q003: Import inutilisé + auto-fix
│   │   ├── no_unreachable.rs          # Q004: Code après return/throw
│   │   ├── prefer_const.rs            # Q031: let jamais réassigné
│   │   ├── prefer_using.rs            # Q020: Disposable sans using ⭐ (Node.js 24+)
│   │   ├── floating_promises.rs       # Q021: Promise non attendue
│   │   ├── prefer_optional_chaining.rs # Q022: && → ?.
│   │   ├── prefer_nullish_coalescing.rs # Q023: || → ??
│   │   ├── max_complexity.rs          # Q010: Complexité cyclomatique > 10
│   │   └── max_depth.rs               # Q011: Imbrication > 4
│   │
│   └── security/            # 7 règles de sécurité
│       ├── sql_injection.rs           # S001: SQL injection (taint)
│       ├── xss.rs                     # S002: XSS (taint)
│       ├── command_injection.rs       # S003: Command injection (taint)
│       ├── eval_injection.rs          # S005: Code injection (taint)
│       ├── hardcoded_secrets.rs       # S010: Secrets codés en dur
│       ├── weak_hashing.rs            # S011: Algos crypto faibles (MD5, SHA1)
│       └── insecure_random.rs         # S012: Math.random() pour sécurité
│
└── taint/                   # Taint Analysis Engine (moteur core de sécurité)
    ├── mod.rs               # TaintAnalyzer
    ├── sources.rs           # TaintSourcesRegistry, PropertyMatcher
    ├── sinks.rs             # TaintSinksRegistry (eval, db.query, etc)
    ├── sanitizers.rs        # SanitizersRegistry (DOMPurify, pg.escape, etc)
    ├── dfg.rs               # DataFlowGraph - construction du graphe
    └── propagation.rs       # TaintPropagator - suivi des données
```

**Fichiers importants à lire:**
- `/analysis.rs` - Logique centrale (engine.analyze() qui applique les règles)
- `/parser.rs` - ParsedFile.from_source() - point d'entrée parsing
- `/rules/mod.rs` - Rule trait, comment enregistrer une règle
- `/taint/mod.rs` - TaintAnalyzer.analyze() - coeur du SAST

**Lignes de code:**
- Quality rules: ~6,500 lignes
- Security rules: ~2,500 lignes
- Total core: ~12,000 lignes

---

### 2.2 KAIZEN-CLI (Interface en ligne de commande)

**Responsabilité:** Intégration CLI, formats de sortie, orchestration fichiers

**Structure:**

```
kaizen-cli/src/
├── main.rs                  # Point d'entrée, CLI parser (clap)
├── commands/
│   ├── mod.rs               # Commands enum (Check, Init, Explain)
│   ├── check.rs             # kaizen check <path> (1600 lignes!)
│   ├── init.rs              # kaizen init (config generation)
│   └── explain.rs           # kaizen explain <rule>
│
└── output/                  # Formatters de sortie
    ├── pretty.rs            # Human-readable output (couleurs, ASCII)
    ├── json.rs              # JSON structuré
    ├── sarif.rs             # Format SARIF pour GitHub Code Scanning
    └── ndjson.rs            # Newline-delimited JSON (streaming)
```

**Commandes implémentées:**

1. **kaizen check [PATH]**
   - Analyse un fichier ou répertoire
   - Options:
     - `--staged` : Uniquement fichiers git staged
     - `--format (pretty|json|sarif|compact)` : Format de sortie
     - `--severity (error|warning|info|hint)` : Filtre minimum
     - `--min-confidence (high|medium|low)` : Filtre confiance
     - `--fail-on-warnings` : Exit 1 si warnings
     - `--no-color` : Désactiver couleurs
   - Utilise `rayon` pour parallélisation multifichiers

2. **kaizen init [--force] [--hook pre-commit]**
   - Génère `kaizen.toml` de configuration
   - Peut installer git hook pre-commit

3. **kaizen explain [RULE_ID|RULE_NAME]**
   - Affiche documentation d'une règle
   - `--list` : List toutes les règles

**Architecture:**
- Découverte fichiers: `walkdir` + filtering extensions
- Analyse parallèle: `rayon::par_iter()`
- Configuration: Chargement depuis `kaizen.toml`
- Sortie: Formatters abstraits

---

### 2.3 KAIZEN-LSP (Language Server Protocol)

**Responsabilité:** Intégration IDE temps réel via LSP

**Structure:**

```
kaizen-lsp/src/
├── main.rs                  # Async main + CLI logging
├── server.rs                # KaizenLanguageServer impl
├── handlers.rs              # (Empty placeholder)
├── capabilities.rs          # server_capabilities()
├── code_actions.rs          # generate_code_actions()
├── analysis.rs              # AnalysisEngine wrapper
├── document.rs              # DocumentStore (cache textes ouverts)
├── diagnostics.rs           # Conversion core → LSP diagnostics
├── debouncer.rs             # Debouncer pour didChange (50ms)
├── logging.rs               # init_logging() avec tracing
└── cli.rs                   # CLI args pour LSP
```

**Handlers LSP implémentés:**
- `initialize/initialized` - Handshake
- `didOpen` - Document ouvert
- `didChange` - Texte modifié (avec debounce)
- `didClose` - Document fermé
- `textDocument/codeAction` - Code Actions (quick fixes)
- `shutdown` - Arrêt propre

**Technologies:**
- `tower-lsp` pour le framework
- `tokio` pour async
- `dashmap` pour cache concurrent

---

## 3. COMPOSANTS MAJEURS EN DÉTAIL

### 3.1 Le Système de Règles

**Trait Rule:**
```rust
pub trait Rule: Send + Sync {
    fn metadata(&self) -> &RuleMetadata;
    fn check(&self, file: &ParsedFile) -> Vec<Diagnostic>;
}
```

**Pattern de création (macro declare_rule!):**
```rust
declare_rule!(
    RuleName,
    id = "Q001",           // ou S001 pour sécurité
    name = "rule-name",
    description = "...",
    category = Quality,    // ou Security
    severity = Warning,    // ou Error, Info, Hint
    examples = "..."
);
```

**Enregistrement (analysis.rs):**
```rust
fn create_default_registry() -> RuleRegistry {
    let mut registry = RuleRegistry::new();

    // Quality
    registry.register(Box::new(MaxComplexity::new()));
    registry.register(Box::new(PreferUsing::new()));

    // Security
    registry.register(Box::new(SqlInjection::new()));
    registry.register(Box::new(HardcodedSecrets::new()));

    registry
}
```

**RuleRegistry:**
- Stocke les règles
- Filtre par catégorie (quality, security)
- Applique les overrides de sévérité
- Respects les règles disabled
- Gère les commentaires `lynx-disable`

---

### 3.2 Le Taint Analysis Engine (SAST)

**Concept:** Suit le flux de données non-fiables (tainted) à travers le code.

**Pipeline:**
```
Source (req.body)
    ↓
DataFlowGraph (DFG) - qui pointe vers qui
    ↓
TaintPropagator - marque les variables tainted
    ↓
Cherche chemin vers Sink (eval, db.query)
    ↓
Applique Sanitizers si présents
    ↓
TaintFinding = rapport de vulnérabilité
```

**Sources (données non-fiables):**
- `req.query`, `req.body`, `req.params`, `req.headers` (Express)
- `location.href`, `window.location` (Browser)
- `document.getElementById().value` (DOM)
- `process.env`, `process.argv` (CLI args - partiellement)
- Résultats de `fetch()`, `JSON.parse()`

**Sinks (opérations dangereuses):**
- `eval()`, `Function()`, `setTimeout(string)` → Code execution
- `db.query()`, `connection.execute()` → SQL injection
- `exec()`, `spawn()`, `execSync()` → Command injection
- `innerHTML`, `outerHTML` → XSS
- `fs.readFile()`, `fs.writeFile()` → Path traversal

**Sanitizers (nettoyage de données):**
- `DOMPurify.sanitize()`, `escape()`, `encodeURIComponent()` → XSS
- `sqlstring.escape()`, Requêtes paramétrées → SQL
- `shell-escape`, validation whitelist → Command
- `path.normalize()`, `path.resolve()` → Path traversal

**Registries configurables:**
- Peuvent être étendus via `Config`
- Addition de sources/sinks/sanitizers custom

---

### 3.3 Analyse Sémantique

**ScopeTree:**
- Construit le graphe de portées (Global → Function → Block → ...)
- Associe chaque identifiant à sa déclaration
- Détecte variables inutilisées

**ControlFlowGraph (CFG):**
- Suivre les chemins d'exécution
- Identifier code unreachable (après return/throw)
- Analyser promise pending

**DisposableTypesRegistry:**
- Détecte ressources Disposable (Node.js 24+)
- FileHandle, streams, etc.
- Demande `await using` pour proper cleanup

---

### 3.4 Configuration (kaizen.toml)

**Schéma:**
```toml
# Fichiers à analyser
include = ["src/**/*.ts"]
exclude = ["node_modules", "**/*.test.ts"]

# Configuration des règles
[rules]
quality = true              # Activer catégorie qualité
security = true             # Activer catégorie sécurité
disabled = ["no-console"]   # Règles spécifiques désactivées
min_confidence = "medium"   # Filtrer par confiance

# Override sévérité par règle
[rules.severity]
"no-console" = "error"      # Élever en erreur
"no-unused-vars" = "hint"   # Réduire en hint

# Sources, sinks, sanitizers additionnels
[security.taint]
additional_sources = ["customRequest.body"]
additional_sanitizers = ["myCompany.sanitize()"]
additional_sinks = ["legacyDb.rawQuery()"]
```

**Recherche:**
1. `./kaizen.toml`
2. `./kaizen/kaizen.toml`
3. `./.kaizen.toml`
4. `~/.config/kaizen/kaizen.toml`

**Validation:**
- Warnings pour clés inconnues
- Erreurs pour TOML invalide
- Defaults sensibles si absent

---

## 4. POINTS D'EXTENSION CLÉS

### 4.1 Ajouter une Règle Qualité

**Étapes:**

1. **Créer fichier** `/crates/kaizen-core/src/rules/quality/mon_rule.rs`:
```rust
use crate::declare_rule;
use crate::diagnostic::Diagnostic;
use crate::parser::ParsedFile;
use crate::rules::{Rule, RuleMetadata, Severity};

declare_rule!(
    MonRule,
    id = "Q050",  // Next free ID
    name = "my-rule",
    description = "...",
    category = Quality,
    severity = Warning,
);

impl Rule for MonRule {
    fn metadata(&self) -> &RuleMetadata {
        &self.metadata
    }

    fn check(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let Some(module) = file.module() else { return Vec::new(); };

        // Parcourir AST, collecter diagnostics
        let mut diags = Vec::new();

        // ...logique...

        diags
    }
}
```

2. **Ajouter au mod.rs:**
```rust
// crates/kaizen-core/src/rules/quality/mod.rs
pub mod mon_rule;
pub use mon_rule::MonRule;
```

3. **Enregistrer dans analysis.rs:**
```rust
registry.register(Box::new(MonRule::new()));
```

4. **Tests:**
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn detects_issue() {
        let code = "...";
        let file = ParsedFile::from_source("test.js", code);
        let rule = MonRule::new();
        let diags = rule.check(&file);
        assert!(diags.iter().any(|d| d.rule_id == "Q050"));
    }
}
```

---

### 4.2 Ajouter une Règle de Sécurité (Taint-based)

**Pattern (voir sql_injection.rs):**

```rust
declare_rule!(
    MyVulnerability,
    id = "S020",
    name = "my-vulnerability",
    category = Security,
    severity = Error,
);

impl Rule for MyVulnerability {
    fn check(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let analyzer = TaintAnalyzer::new();
        let findings = analyzer.analyze(file);
        let ctx = VisitorContext::new(file);

        findings
            .into_iter()
            .filter(|f| f.sink_category == TaintSinkCategory::MyCategory)
            .map(|finding| {
                // Construire diagnostic à partir du finding
                Diagnostic::new(
                    "S020",
                    Severity::Error,
                    format!("Vulnerability from line {} to {}", ...),
                    &file.metadata().filename,
                    sink_line,
                    sink_column,
                )
            })
            .collect()
    }
}
```

---

### 4.3 Étendre Taint Analysis

**Ajouter une source (données non-fiables):**

Éditer `/crates/kaizen-core/src/taint/sources.rs`:
```rust
// Dans TaintSourcesRegistry::with_defaults()
self.sources.push(TaintSourcePattern {
    kind: TaintSourceKind::Request,
    category: TaintCategory::HttpRequest,
    matchers: vec![
        "customApi.untrusted"  // Nouveau matcher
    ],
});
```

**Ajouter un sink (opération dangereuse):**

Éditer `/crates/kaizen-core/src/taint/sinks.rs`:
```rust
// Dans TaintSinksRegistry::with_defaults()
self.sinks.push(TaintSinkPattern {
    kind: TaintSinkKind::Sql,
    category: TaintSinkCategory::SqlInjection,
    matchers: vec![
        ("myCustomDb.unsafeQuery", 0)  // Param index 0 is sink
    ],
});
```

**Ajouter un sanitizer (nettoyage):**

Éditer `/crates/kaizen-core/src/taint/sanitizers.rs`:
```rust
// Dans SanitizersRegistry::with_defaults()
self.sanitizers.push(SanitizerPattern {
    kind: SanitizerKind::Sql,
    category: SanitizerCategory::SqlInjection,
    matchers: vec![
        "myCompany.safe.query",  // Appels qui nettoient
    ],
});
```

---

### 4.4 Points d'injection API Key

**Potentiels points :**

**A. Dans la CLI (check.rs):**
```rust
// Avant d'analyser, vérifier si API key valide
let api_key = std::env::var("KAIZEN_API_KEY").ok();
if let Some(key) = api_key {
    validate_api_key(&key)?;
    // Envoyer usage telemetry anonymisé
}
```

**B. Dans le serveur LSP (server.rs):**
```rust
// À l'initialization
async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
    // Récupérer API key depuis workspace config
    if let Some(client_config) = params.initialization_options {
        if let Some(key) = client_config.get("apiKey") {
            // Valider et cacher
        }
    }
}
```

**C. Dans la config (config.rs):**
```toml
[license]
api_key = "kz_..."  # Clé pour débloquer règles premium
```

**D. Middleware HTTP (pour future API):**
- Ajouter route `POST /api/validate-key`
- Ajouter middleware auth sur routes premium
- Rate-limiting par clé

---

## 5. FONCTIONNALITÉS ACTUELLES DÉTAILLÉES

### 5.1 Règles de Qualité (13 implémentées)

| ID | Nom | Description | Auto-fix | Statut |
|----|-----|-------------|----------|--------|
| Q001 | no-unused-vars | Variable déclarée jamais utilisée | - | ✅ |
| Q003 | no-unused-imports | Import jamais utilisé | ✅ | ✅ |
| Q004 | no-unreachable | Code après return/throw/break | - | ✅ |
| Q010 | max-complexity | Complexité cyclomatique > 10 | - | ✅ |
| Q011 | max-depth | Imbrication > 4 | - | ✅ |
| Q020 | prefer-using | Disposable sans await using | ✅ | ✅ ⭐ Node.js 24+ |
| Q021 | no-floating-promises | Promise non attendue | - | ✅ |
| Q022 | prefer-optional-chaining | && remplaçable par ?. | - | ✅ |
| Q023 | prefer-nullish-coalescing | \|\| remplaçable par ?? | - | ✅ |
| Q030 | no-var | var non-moderne | ✅ | ✅ |
| Q031 | prefer-const | let jamais réassigné | ✅ | ✅ |
| Q032 | no-console | console.* en production | - | ✅ |
| Q033 | eqeqeq | == au lieu de === | ✅ | ✅ |
| Q034 | no-eval | eval() interdiction | - | ✅ |

---

### 5.2 Règles de Sécurité (7 implémentées)

| ID | Nom | Description | Type | Statut |
|----|-----|-------------|------|--------|
| S001 | no-sql-injection | Injection SQL | Taint | ✅ |
| S002 | no-xss | Cross-Site Scripting | Taint | ✅ |
| S003 | no-command-injection | Command injection | Taint | ✅ |
| S005 | no-eval-injection | Code injection | Taint | ✅ |
| S010 | no-hardcoded-secrets | Secrets codés | Pattern | ✅ |
| S011 | no-weak-hashing | Crypto faible (MD5, SHA1) | Pattern | ✅ |
| S012 | no-insecure-random | Math.random() pour sécurité | Pattern | ✅ |

**Patterns de secrets détectés:**
- AWS Access Keys (AKIA...)
- Stripe keys (sk_live_..., sk_test_...)
- GitHub tokens (ghp_, gho_, ghu_, ghs_, ghr_)
- Slack tokens (xox...)
- Et 10+ autres

---

### 5.3 Scan de Sécurité: Architecture

```
ParsedFile (source)
    ↓
TaintAnalyzer::analyze()
    ├─ ScopeBuilder::build()        # Construire scopes, résoudre identifiants
    ├─ DataFlowGraph::build()       # Qui assigne à qui, qui lit de qui
    └─ TaintPropagator::analyze()   # Chercher chemins Source → Sink
        ├─ Marquer sources tainted
        ├─ Propager taint via assignations
        ├─ Déterminer si sink atteint en tainted
        ├─ Vérifier sanitizers appliqués
        └─ Retourner TaintFindings
    ↓
Règles de sécurité (S001, S002, etc)
    ├─ Filtre par TaintSinkCategory
    ├─ Construit diagnostics depuis findings
    └─ Ajoute suggestions et fixes
    ↓
Diagnostics finaux (severity, confidence, message)
```

**Bases de données de vulnérabilités:**
- En dur dans le code (sources.rs, sinks.rs, sanitizers.rs)
- Extensibles via config kaizen.toml
- Pas de dépendance externe ou cloud

---

## 6. MODES D'UTILISATION

### 6.1 Mode CLI (CI/CD, Terminal)

```bash
# Installation
cargo install kaizen-cli
# ou
npm install -g kzn-cli

# Analyse basique
kaizen check ./src

# Avec options
kaizen check ./src \
  --format json \
  --severity error \
  --min-confidence high \
  --fail-on-warnings

# Fichiers staged uniquement
kaizen check --staged

# Générer SARIF (GitHub Code Scanning)
kaizen check ./src --format sarif > results.sarif
```

**Résultats:**
- Exit 0 = OK
- Exit 1 = Issues found (ou erreur runtime si fail-on-warnings)
- Exit 2 = Erreur config/crash

---

### 6.2 Mode LSP (IDE temps réel)

**VS Code:**
- Voir extension dans `/editors/vscode`
- Utilise `vscode-languageclient` pour communiquer

**Zed:**
- Extension WASM native (`/editors/zed`)
- Compilée avec `cargo build --target wasm32-wasip1`

**Configuration:**
```json
{
  "lsp": {
    "kaizen": {
      "binary": {
        "path": "kaizen",
        "arguments": ["lsp"]
      }
    }
  },
  "languages": {
    "JavaScript": {
      "language_servers": ["kaizen"]
    }
  }
}
```

**Capacités:**
- `textDocument/publishDiagnostics` - Envoyer diagnostics
- `textDocument/codeAction` - Code actions (quick fixes)
- Debouncer 50ms sur `didChange`

---

### 6.3 GitHub Actions

```yaml
name: Kaizen Security Analysis

on: [push, pull_request]

jobs:
  kaizen:
    runs-on: ubuntu-latest
    permissions:
      security-events: write

    steps:
      - uses: actions/checkout@v4

      - name: Run Kaizen
        uses: mpiton/kaizen@main
        with:
          path: './src'
          severity: 'warning'
          sarif-upload: 'true'
```

**Action inputs:**
- `path` - Répertoire à analyser
- `severity` - Seuil minimum
- `min-confidence` - Filtre confiance
- `fail-on-warnings` - Bloquer si avertissements
- `sarif-upload` - Uploader vers GitHub Code Scanning
- `sarif-category` - Catégorie pour résultats

---

## 7. POINTS D'EXTENSION POUR FONCTIONNALITÉS "PREMIUM"

### 7.1 Candidate 1: Règles Avancées (Hard to Implement)

**Premium Rules:**
- `S020: advanced-prototype-pollution` - Détection sophistiquée de prototype pollution
- `S021: vulnerable-dependency-usage` - Vérifier si fonction utilisée correctement
- `S022: unsafe-regex-dos` - Détection ReDoS patterns
- `S023: insecure-deserialization` - unsafe deserialization patterns

**Justification du Premium:**
- Require heavy ML/pattern-matching
- Nombreux faux positifs sans ML
- Intérêt pour gros projets/entreprises

**Architecture:**
```rust
// Dans rules/security/advanced_pattern_matching.rs
if is_premium_rule && !api_key_valid {
    return Vec::new();  // Pas d'analyse
}

let findings = advanced_analysis(file);
```

---

### 7.2 Candidate 2: Dashboard Web + API

**Fonctionnalités:**
- Web UI pour voir résultats historiques
- API REST pour intégrations custom
- Webhook pour notifications
- Analytics (trends, leaderboard)

**Points d'intégration:**
```rust
// Dans kaizen-lsp ou nouveau crate kaizen-api
#[tokio::main]
async fn main() {
    let api_key = std::env::var("KAIZEN_API_KEY").ok();
    if api_key.is_some() {
        start_premium_api_server().await;
    }
}
```

**Routes premium:**
- `POST /api/projects/{id}/analyses`
- `GET /api/projects/{id}/results`
- `GET /api/projects/{id}/trends`
- `POST /api/webhooks`

---

### 7.3 Candidate 3: Custom Rules DSL

**Feature:**
- Créer règles sans compiler Rust
- DSL ou JavaScript-based rules

**Points d'intégration:**
```toml
# kaizen.toml
[[rules.custom]]
name = "my-custom-rule"
type = "ast-pattern"  # ou "taint-source"
pattern = "..."
severity = "warning"
```

**Architecture:**
```rust
// Dans rules/custom/mod.rs
pub struct CustomRule {
    pattern: String,
    runner: Box<dyn Fn(&Module) -> Vec<Diagnostic>>,
}
```

---

### 7.4 Candidate 4: CI/CD Insights

**Features:**
- Tendances: Bugs introduits par PR
- Recommandations: "PRs with this pattern have 3x more bugs"
- Notifications: Slack, Teams
- SLA: "Fix security issues in 24h"

**Requis:**
- Stockage historique
- Intégration Git
- Machine learning simple

---

### 7.5 Candidate 5: AI-Powered Suggestions

**Feature:**
- Générer code fix automatique via LLM
- "Explain this vulnerability in plain English"
- Rank rules par likelihood de bug réel

**Points d'intégration:**
```rust
// Dans diagnostic.rs
pub struct Diagnostic {
    pub ai_explanation: Option<String>,  // Payant
    pub ai_fix: Option<Fix>,             // Payant
    pub likelihood_score: f32,           // ML-based
}
```

---

## 8. FONCTIONNALITÉS NON IMPLÉMENTÉES (Hors Scope MVP)

- ❌ Formatting code (délégué à Prettier/Biome)
- ❌ Auto-fix complexe (seulement cas simples)
- ❌ Framework-specific rules (React hooks, Vue composition)
- ❌ Custom rules DSL
- ❌ Dashboard web
- ❌ Règles utilisateurs
- ❌ SCA (Software Composition Analysis)
- ❌ Support mono-repo avancé
- ❌ Analyse dépendances

---

## 9. ARCHITECTURE PATTERNS CLÉS

### 9.1 Visitor Pattern

```rust
// visitor/traits.rs
pub trait AstVisitor {
    fn visit_var_decl(&mut self, node: &VarDecl, ctx: &VisitorContext) -> ControlFlow<()>;
    fn visit_function(&mut self, node: &Function, ctx: &VisitorContext) -> ControlFlow<()>;
    // etc...
}

// Usage dans les règles
pub struct MyRuleVisitor {
    diagnostics: Vec<Diagnostic>,
}

impl AstVisitor for MyRuleVisitor {
    fn visit_var_decl(&mut self, node: &VarDecl, ctx: &VisitorContext) -> ControlFlow<()> {
        // Traiter var decl
        ControlFlow::Continue(())
    }
}
```

### 9.2 Registry Pattern

```rust
pub struct RuleRegistry {
    rules: Vec<Box<dyn Rule>>,
    disabled_rules: HashSet<String>,
    severity_overrides: HashMap<String, Severity>,
}

impl RuleRegistry {
    pub fn register(&mut self, rule: Box<dyn Rule>) {
        self.rules.push(rule);
    }

    pub fn run_all(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        self.rules
            .iter()
            .filter(|r| self.should_run_rule(r.as_ref()))
            .flat_map(|r| r.check(file))
            .collect()
    }
}
```

### 9.3 Builder Pattern (Diagnostic)

```rust
let diag = Diagnostic::new("Q001", Severity::Warning, "var unused", "test.js", 5, 0)
    .with_confidence(Confidence::High)
    .with_suggestion("Use let or const")
    .with_fix(Fix::replace("const", "var", 5, 0, 5, 3));
```

---

## 10. STOCKAGE ET BASES DE DONNÉES

**Actuellement:** Aucune base de données externe
- Tout est calculé à la volée
- Pas de cache persistant
- Configuration en fichier TOML local
- Secrets patterns en dur dans le code

**Points de stockage futurs (premium):**
- Base de données pour historique analyses
- Cache de règles compilées
- User accounts + API keys
- Analytics dashboard

---

## 11. FLUX D'INTÉGRATION SÉCURITÉ

### 11.1 Pipeline Complet

```
1. Source code
    ↓
2. Parser (swc) → AST + errors
    ↓
3. Semantic Analysis (scopes, CFG)
    ↓
4. Taint Analysis (DFG + propagation)
    ↓
5. Rule Engine
    ├─ Quality Rules (14)
    ├─ Security Rules (7)
    └─ Custom Rules (future)
    ↓
6. Filter (severity, confidence, disabled)
    ↓
7. Apply disable comments (lynx-disable-line)
    ↓
8. Output
    ├─ LSP (publish_diagnostics)
    ├─ JSON
    ├─ SARIF
    └─ Pretty (terminal)
```

### 11.2 Exemple: SQL Injection Detection

```javascript
// Input code
const userId = req.query.id;  // SOURCE: tainted
const query = `SELECT * FROM users WHERE id = ${userId}`;
db.query(query);              // SINK: SQL operation

// Analysis
1. TaintAnalyzer::analyze()
2. Source registry détecte req.query.id
3. DFG traces userId ← req.query.id
4. DFG traces query ← template literal avec userId
5. Propagateur marks query as tainted
6. Sink registry détecte db.query() appel
7. Taint propagator finds path: req.query → query → db.query
8. No sanitizer found
9. TaintFinding(source=req.query, sink=db.query, category=SQL)

// Output
Diagnostic {
    rule_id: "S001",
    severity: Error,
    confidence: High,
    message: "Potential SQL injection: untrusted data flows...",
    suggestion: "Use parameterized queries",
    fixes: [Fix::replace(...)]
}
```

---

## 12. ROADMAP & STATUT ACTUEL

### Phases Complétées

✅ **Phase 1: Foundation (MVP)**
- Parser + AST (swc)
- Serveur LSP de base
- 5 règles de base

✅ **Phase 2: Quality Engine**
- 13 règles de qualité
- Support Node.js 24+ (prefer-using)
- Semantic analysis

✅ **Phase 3: Security Engine (en cours)**
- Taint analysis
- 7 règles de sécurité
- Hardcoded secrets detection

✅ **Phase 4: Polish & Release**
- CLI complet
- Formats output (JSON, SARIF, pretty)
- GitHub Actions integration
- npm package

### À Venir

⏳ **Phase 5: Framework Support**
- React hooks rules
- Vue composition API
- Next.js patterns

⏳ **Phase 6: Advanced**
- Règles custom via DSL
- Dashboard web
- AI-powered suggestions

---

## 13. RÉSUMÉ DES POINTS CLÉS

### ✅ Points Forts

1. **Performance:** ~1500 LOC analyse, utilise SWC rapide
2. **Architecture:** Modulaire, patterns clairs (Visitor, Registry)
3. **Security Focus:** Taint analysis pour SAST, 7 règles sécu
4. **IDE Integration:** LSP natif, Zed + VS Code
5. **Configuration:** Simple (kaizen.toml), override par règle
6. **Distribution:** npm package + binaires cross-platform
7. **Node.js 24+:** Support unique de `await using`
8. **Extensibilité:** Ajout facile de règles

### ⚠️ Points Attention

1. **Pas de base de données:** Tout en mémoire, pas d'historique
2. **Taint analysis limité:** Heuristiques, pas 100% correct
3. **Pas de ML:** Patterns manuels, faux positifs possibles
4. **Config unique:** Un seul kaizen.toml par project
5. **CLI seulement:** Pas de API REST (future)
6. **Tests limités:** Snapshots, pas de test complet

### 🎯 Pour Monétisation

**Meilleurs candidats:**
1. **Règles avancées** (Q+: hard to detect, high value)
2. **API REST + Dashboard** (teams, analytics)
3. **Custom rules DSL** (enterprises)
4. **Support 24/7** + Priority queue

---

## 14. FICHIERS À LIRE EN PRIORITÉ

Pour approfondir compréhension:

1. **Architecture générale:**
   - `/README.md` - Vue d'ensemble
   - `/docs/project/PRD-Lynx.md` - Vision complète

2. **Core engine:**
   - `/crates/kaizen-core/src/analysis.rs` - Orchestre analyse
   - `/crates/kaizen-core/src/parser.rs` - ParsedFile
   - `/crates/kaizen-core/src/rules/mod.rs` - Rule trait

3. **Sécurité (taint):**
   - `/crates/kaizen-core/src/taint/mod.rs` - TaintAnalyzer
   - `/crates/kaizen-core/src/taint/dfg.rs` - DataFlowGraph
   - `/crates/kaizen-core/src/taint/propagation.rs` - Propagateur

4. **CLI:**
   - `/crates/kaizen-cli/src/commands/check.rs` - Commande principale
   - `/crates/kaizen-cli/src/output/` - Formatters

5. **LSP:**
   - `/crates/kaizen-lsp/src/server.rs` - KaizenLanguageServer
   - `/crates/kaizen-lsp/src/capabilities.rs` - Capacités LSP

6. **Examples:**
   - `/crates/kaizen-core/src/rules/security/sql_injection.rs` - Exemple règle taint
   - `/crates/kaizen-core/src/rules/security/hardcoded_secrets.rs` - Exemple règle pattern
   - `/crates/kaizen-core/src/rules/quality/prefer_using.rs` - Exemple règle sémantique

---

## 15. COMMANDES UTILES

```bash
# Compiler
cargo build --release

# Tests
cargo test --workspace

# Benchmark
cargo bench --package kaizen-core

# Format + Lint
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings

# Documentation
cargo doc --open

# CLI usage
kaizen check ./src --format json
kaizen explain Q020
kaizen init --force

# LSP
kaizen lsp --log-level debug
```

---

**Document généré pour exploration complète codebase Kaizen/Lynx**
**Date:** 2025-12-19
**Version PRD:** 1.0 (1330 lignes detail)
