# TL;DR: KAIZEN CODEBASE EN 5 MINUTES

## Qu'est-ce que Kaizen?

Analyseur statique ultra-rapide pour JavaScript/TypeScript, écrit en Rust.
- **20 règles:** 13 qualité + 7 sécurité
- **Modes:** CLI, LSP (IDE), GitHub Actions
- **Spécialité:** Taint analysis pour détection vulnérabilités (SQL injection, XSS, etc.)

## Architecture (3 Crates)

```
kaizen-core   → Moteur d'analyse (12,000 LOC)
              ├─ Parser (swc)
              ├─ Semantic analysis
              ├─ Taint analysis
              ├─ 20 règles
              └─ Configuration

kaizen-cli    → Interface terminal (1,000 LOC)
              ├─ kaizen check ./src
              ├─ kaizen init
              ├─ kaizen explain Q001
              └─ Formats: json, sarif, pretty

kaizen-lsp    → Language Server Protocol (1,500 LOC)
              ├─ Intégration Zed/VS Code
              ├─ Diagnostics temps réel
              └─ Code actions (quick fixes)
```

## Où Ajouter une Règle? (5 minutes)

1. Créer `/crates/kaizen-core/src/rules/quality/no_magic_numbers.rs`
2. Implémenter `Rule` trait
3. Ajouter au `mod.rs`
4. Enregistrer dans `analysis.rs`
5. Tester: `cargo test`

**Guide complet:** `/QUICK_START_AJOUTER_REGLE.md`

## Où Injecter API Key? (Licensing)

Option: Créer `/crates/kaizen-core/src/licensing.rs`:
```rust
pub struct LicenseValidator {
    pub fn validate_local(&self, api_key: &str) -> Result<LicenseInfo>
}
```

Points d'intégration:
- CLI: `/crates/kaizen-cli/src/commands/check.rs`
- Rules: Filter par tier dans `RuleRegistry`
- LSP: Load key depuis workspace config

**Guide complet:** `/POINTS_EXTENSION_ET_MONETISATION.md` section 3

## Points d'Extension (Premium Candidates)

1. **Règles avancées** (S020-S024): Prototype pollution, Regex DoS (Value: ⭐⭐⭐)
2. **Framework support** (React, Vue): Hooks rules (Value: ⭐⭐)
3. **AI suggestions**: LLM-powered fixes (Value: ⭐⭐)
4. **API REST + Dashboard**: Web UI, analytics (Value: ⭐⭐)
5. **Custom Rules DSL**: User-defined rules (Value: ⭐)

**Guide complet:** `/POINTS_EXTENSION_ET_MONETISATION.md` section 4

## Fonctionnalités Actuelles

### Règles Qualité (13)
Q001 (unused-vars) | Q003 (unused-imports) | Q010 (max-complexity) | Q011 (max-depth) |
Q020 (prefer-using) ⭐ Node.js 24+ | Q021 (floating-promises) | Q022 (optional-chaining) |
Q023 (nullish-coalescing) | Q030 (no-var) | Q031 (prefer-const) | Q032 (no-console) |
Q033 (eqeqeq) | Q034 (no-eval)

### Règles Sécurité (7 - Taint Analysis)
S001 (SQL injection) | S002 (XSS) | S003 (Command injection) | S005 (Code injection) |
S010 (Hardcoded secrets) | S011 (Weak crypto) | S012 (Insecure random)

### Commandes CLI
```bash
kaizen check ./src              # Analyser
kaizen init                     # Créer config
kaizen explain Q020             # Docs d'une règle
kaizen check --staged           # Fichiers git staged
kaizen check --format json      # Format JSON
kaizen check --format sarif     # Pour GitHub Code Scanning
```

### Modes d'Utilisation
- **CLI:** Terminal, CI/CD, pre-commit hook
- **LSP:** Zed editor, VS Code (feedback temps réel)
- **GitHub Actions:** Integration CI/CD

## Taint Analysis (Cœur du SAST)

```
Source (données non-fiables)
  req.query.id, process.env, location.href
         ↓
DataFlowGraph (qui pointe vers qui)
         ↓
TaintPropagator (marque variables tainted)
         ↓
Cherche chemin vers Sink (eval, db.query, innerHTML)
         ↓
Applique Sanitizers si présents (DOMPurify, parameterized queries)
         ↓
TaintFinding = rapport vulnérabilité
```

## Fichiers Clés

| Fichier | Rôle | LOC |
|---------|------|-----|
| `analysis.rs` | Orchestre l'analyse + enregistrement règles | 150 |
| `rules/mod.rs` | Rule trait + RuleRegistry | 200 |
| `parser.rs` | Integration SWC + ParsedFile | 200 |
| `taint/mod.rs` | TaintAnalyzer orchestration | 80 |
| `taint/dfg.rs` | DataFlowGraph construction | 400+ |
| `commands/check.rs` | CLI check command | 1600 |
| `server.rs` | LSP server implementation | 300+ |

## Configuration (kaizen.toml)

```toml
include = ["src/**/*.ts"]
exclude = ["node_modules", "**/*.test.ts"]

[rules]
quality = true
security = true
disabled = ["no-console"]

[rules.severity]
"no-console" = "error"
```

## Technologies

- **Parser:** swc_ecma_parser (JS/TS ultra-rapide)
- **LSP:** tower-lsp
- **Async:** tokio
- **Parallélisation:** rayon (par_iter)
- **Config:** TOML

## Roadmap Actuellement

✅ Phase 1-3: Complétées (MVP)
- Parser, LSP, 20 règles, Taint analysis

⏳ Phase 4: Optimisation
- Performance tuning, plus de règles

🎯 Phase 5+: Monétisation
- API key licensing, Premium rules, Dashboard

## Statistiques

- **Codebase total:** ~20,000 LOC Rust
- **Règles:** 20 implémentées, 30+ planifiées
- **Dépendances:** ~5 essentielles
- **Performance:** < 30ms par fichier (target)
- **Platforms:** Linux, macOS, Windows (x64, ARM64)
- **Distribution:** npm package + GitHub Releases

## Pour Commencer

1. **Comprendre l'architecture:**
   - Lire `/EXPLORATION_CODEBASE.md` (30 min)

2. **Ajouter une règle simple:**
   - Suivre `/QUICK_START_AJOUTER_REGLE.md` (30 min)

3. **Implémenter licensing (optionnel):**
   - Suivre `/POINTS_EXTENSION_ET_MONETISATION.md` (3-4 h)

4. **Questions détaillées:**
   - Consulter `/EXPLORATION_INDEX.md`

## Commandes Utiles

```bash
# Compiler
cargo build --release

# Tester
cargo test --workspace

# Linter
cargo clippy
cargo fmt

# Benchmark
cargo bench --package kaizen-core

# Utiliser
kaizen check ./src --format json
```

## Takeaways Clés

✅ Code bien structuré, patterns clairs
✅ Facile d'ajouter des règles (~30 min pour une simple)
✅ Taint analysis sophistiquée pour SAST
✅ Prêt pour monétisation (licensing framework absent actuellement)
✅ Extensible (sources, sinks, sanitizers configurables)

---

**Plus d'infos:** Voir documents détaillés dans le repo
**Généré:** 2025-12-19
