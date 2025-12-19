# no-unused-vars (Q001)

Disallow unused variables.

## Description

This rule uses semantic analysis to detect variables that are declared but never used in the code. It helps keep your codebase clean by identifying dead code that can be safely removed.

## Rationale

Unused variables often indicate:
- Incomplete refactoring
- Copy-paste errors
- Dead code that should be removed
- Variables intended to be used but forgotten

Removing unused variables improves code readability and reduces confusion for other developers.

## Examples

### Bad

```javascript
const unused = 1;

function foo(unusedParam) {
    return 42;
}

let x = 1;
x = 2;
x = 3;  // Write-only variable - assigned but never read
```

### Good

```javascript
const used = 1;
console.log(used);

function foo(param) {
    return param * 2;
}

// Intentionally unused with underscore prefix
const _intentionallyUnused = 1;
```

## Features

- **Scope-aware analysis**: Correctly handles closures and nested scopes
- **Underscore prefix exception**: Variables starting with `_` are ignored
- **Write-only detection**: Detects variables that are assigned but never read
- **Export awareness**: Exported variables are not flagged
- **Closure support**: Variables used in inner functions are correctly tracked

## Configuration

### Disable the rule

```toml
[rules]
disabled = ["Q001"]
# or
disabled = ["no-unused-vars"]
```

### Change severity

```toml
[rules.severity]
"no-unused-vars" = "error"
```

## When Not To Use It

- When you have variables that are intentionally unused (use underscore prefix instead)
- In development environments where you frequently comment out code

## Related Rules

- [no-unused-imports](no-unused-imports.md) - Detects unused imports
