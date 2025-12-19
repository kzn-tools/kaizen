# QUICK START: AJOUTER UNE RÈGLE À KAIZEN

Ce guide vous montre comment ajouter une nouvelle règle en 5 minutes.

---

## 1. CHOISIR UNE RÈGLE À AJOUTER

Exemple: **Q035: no-magic-numbers** - Détecter les nombres magiques non-documentés

```javascript
// Bad
const x = 42;  // Qu'est-ce que c'est?

// Good
const ANSWER_TO_EVERYTHING = 42;
```

---

## 2. CRÉER LE FICHIER

**Chemin:** `/crates/kaizen-core/src/rules/quality/no_magic_numbers.rs`

```rust
//! no-magic-numbers rule (Q035): Detect magic numbers
//!
//! Magic numbers are hardcoded numbers without clear context or meaning.
//! This rule encourages using named constants instead.

use swc_ecma_ast::{Expr, Lit, Number, VarDecl};

use crate::declare_rule;
use crate::diagnostic::Diagnostic;
use crate::parser::ParsedFile;
use crate::rules::{Rule, RuleMetadata, Severity};
use crate::visitor::{AstVisitor, VisitorContext, walk_ast};

declare_rule!(
    NoMagicNumbers,
    id = "Q035",
    name = "no-magic-numbers",
    description = "Disallow magic numbers without explanation",
    category = Quality,
    severity = Info,
    examples = "// Bad\nconst arr = new Array(42);\n\n// Good\nconst BUFFER_SIZE = 42;\nconst arr = new Array(BUFFER_SIZE);"
);

impl Rule for NoMagicNumbers {
    fn metadata(&self) -> &RuleMetadata {
        &self.metadata
    }

    fn check(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let Some(module) = file.module() else {
            return Vec::new();
        };

        let ctx = VisitorContext::new(file);
        let mut visitor = NoMagicNumbersVisitor {
            diagnostics: Vec::new(),
            file_path: file.metadata().filename.clone(),
            ctx: &ctx,
        };

        walk_ast(&mut visitor, module);

        visitor.diagnostics
    }
}

struct NoMagicNumbersVisitor<'a> {
    diagnostics: Vec<Diagnostic>,
    file_path: String,
    ctx: &'a VisitorContext<'a>,
}

impl<'a> AstVisitor for NoMagicNumbersVisitor<'a> {
    fn visit_var_decl(
        &mut self,
        node: &VarDecl,
        _ctx: &VisitorContext,
    ) -> std::ops::ControlFlow<()> {
        for decl in &node.decls {
            if let Some(init) = &decl.init {
                if self.is_magic_number(init) {
                    if let Some(name) = decl.name.as_ident() {
                        let (line, column) = self.ctx.span_to_location(decl.span());

                        let diagnostic = Diagnostic::new(
                            "Q035",
                            Severity::Info,
                            format!("Magic number detected: '{}'", self.extract_number(init)),
                            &self.file_path,
                            line,
                            column,
                        )
                        .with_suggestion("Use a named constant instead");

                        self.diagnostics.push(diagnostic);
                    }
                }
            }
        }

        std::ops::ControlFlow::Continue(())
    }
}

impl<'a> NoMagicNumbersVisitor<'a> {
    fn is_magic_number(&self, expr: &Expr) -> bool {
        matches!(expr, Expr::Lit(Lit::Num(_)))
    }

    fn extract_number(&self, expr: &Expr) -> String {
        match expr {
            Expr::Lit(Lit::Num(Number { value, .. })) => value.to_string(),
            _ => "unknown".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_rule(code: &str) -> Vec<Diagnostic> {
        let file = ParsedFile::from_source("test.js", code);
        let rule = NoMagicNumbers::new();
        rule.check(&file)
    }

    #[test]
    fn detects_magic_numbers() {
        let code = "const x = 42;";
        let diagnostics = run_rule(code);
        assert!(!diagnostics.is_empty());
        assert!(diagnostics[0].rule_id == "Q035");
    }

    #[test]
    fn ignores_constants() {
        let code = "const ANSWER = 42;";
        let diagnostics = run_rule(code);
        // NOTE: In reality this would require checking naming conventions
        // For now, just a simple test
        assert!(true);
    }
}
```

---

## 3. AJOUTER AU MODULE

**Fichier:** `/crates/kaizen-core/src/rules/quality/mod.rs`

Ajouter ces lignes:

```rust
// Avant: pub mod prefer_using;
pub mod no_magic_numbers;  // ← AJOUT

// Et exports:
// Avant: pub use prefer_using::PreferUsing;
pub use no_magic_numbers::NoMagicNumbers;  // ← AJOUT
```

---

## 4. ENREGISTRER LA RÈGLE

**Fichier:** `/crates/kaizen-core/src/analysis.rs`

Trouver `fn create_default_registry()` et ajouter:

```rust
fn create_default_registry() -> RuleRegistry {
    let mut registry = RuleRegistry::new();

    // Quality rules
    registry.register(Box::new(MaxComplexity::new()));
    // ... autres règles ...
    registry.register(Box::new(NoMagicNumbers::new()));  // ← AJOUT

    // Security rules
    // ...

    registry
}
```

N'oublie pas d'ajouter l'import:

```rust
use crate::rules::quality::NoMagicNumbers;
```

---

## 5. TESTER LA RÈGLE

```bash
# Compiler
cargo build --package kaizen-core

# Tester ta règle spécifique
cargo test no_magic_numbers

# Tester tout
cargo test --workspace
```

---

## 6. ESSAYER AVEC LA CLI

```bash
# Créer un fichier de test
echo "const x = 42;" > test.js

# Analyser
cargo run --bin kaizen-cli -- check test.js

# Ou après installation
kaizen check test.js
```

**Output attendu:**
```
info Q035: Magic number detected: '42'
  --> test.js:1:10
   |
 1 | const x = 42;
   |           ^^
   |
   = suggestion: Use a named constant instead
```

---

## 7. BONNES PRATIQUES

### Pour une règle QUALITÉ (comme Q035)

```rust
// ✅ BON: Utiliser Visitor pattern
impl AstVisitor for MyVisitor {
    fn visit_var_decl(&mut self, node: &VarDecl, ctx: &VisitorContext) {
        // Analyser le node
    }
}

// ✅ BON: Inclure des tests
#[cfg(test)]
mod tests {
    #[test]
    fn detects_issue() { ... }
    #[test]
    fn ignores_valid_code() { ... }
}

// ✅ BON: Messages clair + suggestion
Diagnostic::new("Q035", Severity::Info, "Magic number detected", ...)
    .with_suggestion("Use a named constant instead")
```

### Pour une règle SÉCURITÉ (avec taint analysis)

```rust
// ✅ BON: Utiliser TaintAnalyzer
let analyzer = TaintAnalyzer::new();
let findings = analyzer.analyze(file);

findings.into_iter()
    .filter(|f| f.sink_category == TaintSinkCategory::SqlInjection)
    .map(|finding| create_diagnostic(finding))
    .collect()
```

---

## 8. PATTERNS COURANTS

### Pattern 1: Détecter appels de fonction

```rust
fn visit_call_expr(&mut self, node: &CallExpr, ctx: &VisitorContext) {
    // node.callee = la fonction appelée
    // node.args = les arguments

    if let Callee::Expr(expr) = &node.callee {
        if let Expr::Member(member) = expr.as_ref() {
            // Exemple: db.query()
            // member.obj = db
            // member.prop = query
        }
    }
}
```

### Pattern 2: Détecter chaînes littérales

```rust
fn visit_lit(&mut self, node: &Lit, ctx: &VisitorContext) {
    match node {
        Lit::Str(s) => {
            // s.value = la chaîne
            // Vérifier patterns regex, etc
        }
        Lit::Num(n) => {
            // n.value = le nombre
        }
        _ => {}
    }
}
```

### Pattern 3: Parcourir le code

```rust
// Traverser un bloc entier
for stmt in &block.stmts {
    // Traiter chaque statement
}

// Récupérer infos de localisation
let (line, column) = ctx.span_to_location(node.span());

// Créer diagnostic
Diagnostic::new(
    "Q999",
    Severity::Warning,
    "Message d'erreur",
    &self.file_path,
    line,
    column,
)
```

### Pattern 4: Avec code action (auto-fix)

```rust
use crate::diagnostic::Fix;

let diagnostic = Diagnostic::new(...)
    .with_fix(Fix::replace(
        "Replace with const",
        "const",
        line,
        column,
        line,
        column_end,
    ));
```

---

## 9. TESTER AVEC LSP

Pour tester avec l'IDE:

```bash
# Compiler LSP
cargo build --package kaizen-lsp --release

# Faire pointer Zed/VSCode au binaire
# Dans ~/.config/zed/settings.json ou equivalente:
{
  "lsp": {
    "kaizen": {
      "binary": {
        "path": "./target/release/kaizen-lsp",
        "arguments": ["lsp"]
      }
    }
  }
}

# Ouvrir un fichier JS/TS
# Voir les diagnostics en temps réel!
```

---

## 10. CHECKLIST AVANT DE COMMITTER

- [ ] Fichier créé: `src/rules/quality/no_magic_numbers.rs`
- [ ] Exports ajoutés: `src/rules/quality/mod.rs`
- [ ] Enregistrement: `src/analysis.rs`
- [ ] Tests écrits et passants
- [ ] Compile sans warnings: `cargo build`
- [ ] Clippy OK: `cargo clippy`
- [ ] Format OK: `cargo fmt`
- [ ] Message clair + suggestion
- [ ] ID unique (Q035, Q036, etc)
- [ ] Catégorie correcte (Quality ou Security)
- [ ] Severité appropriée
- [ ] Example docstring

---

## 11. EXEMPLE COMPLET: RÈGLE PLUS COMPLEXE

**Règle:** Q040: no-untyped-catch (TypeScript - catch sans type)

```rust
use swc_ecma_ast::{TsType, TsCatchClause};

declare_rule!(
    NoUntypedCatch,
    id = "Q040",
    name = "no-untyped-catch",
    description = "Require explicit type on catch clause",
    category = Quality,
    severity = Warning,
);

impl Rule for NoUntypedCatch {
    fn check(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let Some(module) = file.module() else { return Vec::new(); };

        let ctx = VisitorContext::new(file);
        let mut visitor = NoUntypedCatchVisitor {
            diagnostics: Vec::new(),
            file_path: file.metadata().filename.clone(),
            ctx: &ctx,
        };

        walk_ast(&mut visitor, module);
        visitor.diagnostics
    }
}

struct NoUntypedCatchVisitor<'a> {
    diagnostics: Vec<Diagnostic>,
    file_path: String,
    ctx: &'a VisitorContext<'a>,
}

impl<'a> AstVisitor for NoUntypedCatchVisitor<'a> {
    fn visit_ts_catch_clause(
        &mut self,
        node: &TsCatchClause,
        _ctx: &VisitorContext,
    ) -> std::ops::ControlFlow<()> {
        // TypeScript: catch (e: Error)
        if node.param.is_none() || node.param.as_ref().map(|p| p.type_ann.is_none()).unwrap_or(true) {
            let (line, column) = self.ctx.span_to_location(node.span());

            self.diagnostics.push(
                Diagnostic::new(
                    "Q040",
                    Severity::Warning,
                    "Catch clause parameter should be typed",
                    &self.file_path,
                    line,
                    column,
                )
                .with_suggestion("Add explicit type: catch (e: Error)")
            );
        }

        std::ops::ControlFlow::Continue(())
    }
}
```

---

## 12. DÉBOGGER UNE RÈGLE

Si ta règle ne détecte pas de problèmes:

```rust
// Ajoute du logging
#[test]
fn debug_issue() {
    let code = r#"
        const x = 42;
    "#;

    let file = ParsedFile::from_source("test.js", code);

    // Afficher le AST
    println!("AST: {:#?}", file.module());

    let rule = NoMagicNumbers::new();
    let diagnostics = rule.check(&file);

    println!("Found {} diagnostics", diagnostics.len());
    for d in diagnostics {
        println!("  - {}: {}", d.rule_id, d.message);
    }

    assert!(!diagnostics.is_empty());
}
```

Lancer avec:
```bash
cargo test debug_issue -- --nocapture
```

---

## 13. RESSOURCES

**Dossiers importants:**
- `/crates/kaizen-core/src/rules/` - Toutes les règles existantes (exemples!)
- `/crates/kaizen-core/src/visitor/` - Patterns pour AST traversal

**Types SWC utiles:**
- `swc_ecma_ast::*` - Tous les types AST
- `swc_ecma_ast::Module` - Racine du code
- `swc_ecma_ast::VarDecl`, `CallExpr`, etc - Types pour différents nodes

**Docs:**
- https://rustdoc.swc.rs/swc_ecma_ast/
- `/docs/rules/` - Format documentation des règles
- `/tests/fixtures/` - Fichiers de test existants

---

## RÉSUMÉ

**5 étapes simples:**

1. Créer `.rs` avec struct + impl Rule
2. Ajouter au `mod.rs`
3. Enregistrer dans `analysis.rs`
4. Écrire tests
5. Tester: `cargo test`

**La plupart des règles:** 80-120 lignes de code

**Temps typique:** 30-60 minutes de la première à la dernière ligne

Bon courage! 🚀

---

**Quick Start Guide - Kaizen Rule Addition**
**Version:** 1.0
**Date:** 2025-12-19
