# no-var (Q030)

Disallow `var` declarations.

## Description

This rule reports `var` declarations and suggests using `let` or `const` instead. The `var` keyword has function scope and hoisting behavior that can lead to bugs.

## Rationale

`var` has problematic behaviors:
- Function-scoped instead of block-scoped
- Hoisted to the top of the function
- Can be redeclared in the same scope
- Can be accessed before declaration (returns `undefined`)

`let` and `const` are block-scoped and don't have these issues.

## Examples

### Bad

```javascript
var x = 1;

for (var i = 0; i < 10; i++) {
    // i is function-scoped, not block-scoped
}
console.log(i);  // 10 - i leaks out of the for loop

var name = 'Alice';
var name = 'Bob';  // Accidental redeclaration allowed
```

### Good

```javascript
const x = 1;  // Use const for values that don't change

let count = 0;  // Use let for values that change

for (let i = 0; i < 10; i++) {
    // i is block-scoped
}
// console.log(i);  // ReferenceError - i is not defined
```

## Auto-fix

This rule provides an auto-fix to replace `var` with `let`. You should review whether `const` would be more appropriate.

## Configuration

### Disable the rule

```toml
[rules]
disabled = ["Q030"]
# or
disabled = ["no-var"]
```

### Change severity

```toml
[rules.severity]
"no-var" = "error"
```

## When Not To Use It

- In legacy codebases where migration to `let`/`const` is not feasible
- When you specifically need hoisting behavior (rare)

## Related Rules

- [prefer-const](prefer-const.md) - Suggests `const` for non-reassigned variables
