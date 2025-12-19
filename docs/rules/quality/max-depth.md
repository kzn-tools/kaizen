# max-depth (Q011)

Enforce a maximum nesting depth threshold.

## Description

This rule reports when function nesting depth exceeds the threshold (default: 4). Deep nesting makes code harder to read and understand.

## Rationale

Deeply nested code:
- Is difficult to follow and understand
- Often indicates code that should be refactored
- Increases cognitive load when reading
- Makes debugging more challenging

## Nesting Constructs

The following constructs add to nesting depth:

- `if` / `else if` / `else` statements
- `for` loops (all variants)
- `while` loops
- `do...while` loops
- `switch` statements
- `try` / `catch` / `finally` blocks
- `with` statements

## Examples

### Bad

```javascript
// Depth > 4
function deep(x) {
    if (a) {                    // depth 1
        if (b) {                // depth 2
            if (c) {            // depth 3
                if (d) {        // depth 4
                    if (e) {}   // depth 5 - exceeds threshold
                }
            }
        }
    }
}
```

### Good

```javascript
// Depth <= 4
function shallow(x) {
    if (a) {                    // depth 1
        if (b) {                // depth 2
            if (c) {            // depth 3
                doSomething();  // depth 3
            }
        }
    }
}

// Refactored with early returns
function process(data) {
    if (!data) return null;
    if (!data.valid) return null;
    if (!data.ready) return null;

    return transform(data);
}
```

## Configuration

### Default Threshold

The default threshold is **4**.

### Disable the rule

```toml
[rules]
disabled = ["Q011"]
# or
disabled = ["max-depth"]
```

### Change severity

```toml
[rules.severity]
"max-depth" = "error"
```

## When Not To Use It

- For algorithms that inherently require deep nesting
- When refactoring would make the code less clear

## Tips for Reducing Nesting

1. **Use early returns**: Exit early instead of wrapping in conditions
2. **Extract functions**: Move nested logic into separate functions
3. **Use guard clauses**: Check for invalid conditions first
4. **Flatten with logical operators**: Use `&&` or `||` where appropriate

### Before

```javascript
function process(user) {
    if (user) {
        if (user.active) {
            if (user.verified) {
                return doWork(user);
            }
        }
    }
    return null;
}
```

### After

```javascript
function process(user) {
    if (!user) return null;
    if (!user.active) return null;
    if (!user.verified) return null;

    return doWork(user);
}
```

## Related Rules

- [max-complexity](max-complexity.md) - Limits cyclomatic complexity
