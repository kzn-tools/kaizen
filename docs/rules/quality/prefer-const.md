# prefer-const (Q031)

Require `const` for variables that are never reassigned.

## Description

This rule detects variables declared with `let` that are never reassigned and suggests using `const` instead. This makes code intent clearer and prevents accidental reassignment.

## Rationale

Using `const` when possible:
- Makes intent clear (this value won't change)
- Prevents accidental reassignment
- Enables potential engine optimizations
- Improves code readability

## Examples

### Bad

```javascript
let x = 1;
console.log(x);  // x is never reassigned

let config = { port: 3000 };
server.listen(config.port);  // config reference never changes
```

### Good

```javascript
const x = 1;
console.log(x);

const config = { port: 3000 };
server.listen(config.port);

let count = 0;
count++;  // Correctly uses let because it's reassigned
```

## Important Note

`const` prevents reassignment of the variable binding, not mutation of the value:

```javascript
const obj = { a: 1 };
obj.a = 2;  // Valid - mutation is allowed
obj = {};   // Error - reassignment is not allowed

const arr = [1, 2, 3];
arr.push(4);  // Valid - mutation is allowed
arr = [];     // Error - reassignment is not allowed
```

## Features

- **Scope-aware analysis**: Correctly tracks reassignments across scopes
- **Loop variables**: Ignores loop iterator variables
- **Function parameters**: Does not flag function parameters

## Auto-fix

This rule provides an auto-fix to replace `let` with `const`.

## Configuration

### Disable the rule

```toml
[rules]
disabled = ["Q031"]
# or
disabled = ["prefer-const"]
```

### Change severity

```toml
[rules.severity]
"prefer-const" = "error"
```

## When Not To Use It

- When you prefer consistent use of `let` for all mutable values
- In teams where `const` usage is not enforced

## Related Rules

- [no-var](no-var.md) - Disallows `var` declarations
