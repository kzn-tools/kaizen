# max-complexity (Q010)

Enforce a maximum cyclomatic complexity threshold for functions.

## Description

This rule calculates the cyclomatic complexity of each function and reports when it exceeds the threshold (default: 10). High complexity indicates code that is difficult to understand, test, and maintain.

## Rationale

High cyclomatic complexity:
- Makes functions harder to understand
- Increases the likelihood of bugs
- Makes comprehensive testing difficult
- Indicates the function should be refactored into smaller pieces

## Complexity Calculation

The following constructs add to complexity:

| Construct | Complexity Added |
|-----------|-----------------|
| Function (base) | +1 |
| `if` statement | +1 |
| `else if` | +1 |
| `while` loop | +1 |
| `do...while` loop | +1 |
| `for` loop | +1 |
| `for...in` loop | +1 |
| `for...of` loop | +1 |
| `switch case` (each) | +1 |
| `catch` clause | +1 |
| Ternary `? :` | +1 |
| `&&` operator | +1 |
| `\|\|` operator | +1 |
| `??` operator | +1 |

## Examples

### Bad

```javascript
// Complexity > 10
function complex(x) {
    if (a) {
        if (b) {
            if (c) {
                if (d) {
                    if (e) {
                        if (f) {
                            if (g) {
                                if (h) {
                                    if (i) {
                                        if (j) {
                                            if (k) {}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
```

### Good

```javascript
// Complexity <= 10
function simple(x) {
    if (x > 0) {
        return x * 2;
    }
    return x;
}

// Refactored into smaller functions
function processItem(item) {
    if (!isValid(item)) return null;
    return transform(item);
}

function isValid(item) {
    return item && item.id && item.value;
}

function transform(item) {
    return { ...item, processed: true };
}
```

## Configuration

### Default Threshold

The default threshold is **10**, which is a widely accepted standard.

### Disable the rule

```toml
[rules]
disabled = ["Q010"]
# or
disabled = ["max-complexity"]
```

### Change severity

```toml
[rules.severity]
"max-complexity" = "error"
```

## When Not To Use It

- For parser or lexer functions that inherently require many branches
- For state machines where high complexity is expected

## Tips for Reducing Complexity

1. **Extract methods**: Break large functions into smaller, focused ones
2. **Use early returns**: Replace nested ifs with guard clauses
3. **Use lookup tables**: Replace switch statements with object lookups
4. **Use polymorphism**: Replace type-checking conditionals with inheritance

## Related Rules

- [max-depth](max-depth.md) - Limits nesting depth
