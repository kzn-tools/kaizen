# prefer-nullish-coalescing (Q023)

Suggest nullish coalescing (`??`) instead of `||` for default values.

## Description

This rule detects patterns where `||` is used with literal default values and suggests using nullish coalescing (`??`) instead, which only falls through on `null` or `undefined`.

## Rationale

The `||` operator falls through on any falsy value (`false`, `0`, `''`, `null`, `undefined`, `NaN`), which can cause unexpected behavior:

```javascript
const count = userCount || 10;  // Bug: 0 is valid but treated as missing
const name = userName || 'Anonymous';  // Bug: '' might be intentional
```

Nullish coalescing (`??`) only falls through on `null` or `undefined`, making intent clearer.

## Examples

### Bad

```javascript
value || 'default'

config.timeout || 5000

count || 0

items || []
```

### Good

```javascript
value ?? 'default'

config.timeout ?? 5000

count ?? 0

items ?? []
```

## Detected Patterns

The rule flags `||` expressions where the right side is:
- String literals (`'default'`)
- Number literals (`0`, `5000`)
- Boolean literals (`false`)
- Array literals (`[]`)
- Object literals (`{}`)

### Not Flagged

```javascript
// Boolean context (intentional)
isEnabled || hasPermission

// Dynamic default values
value || getValue()

// Identifier defaults
value || other
```

## Configuration

### Disable the rule

```toml
[rules]
disabled = ["Q023"]
# or
disabled = ["prefer-nullish-coalescing"]
```

### Change severity

```toml
[rules.severity]
"prefer-nullish-coalescing" = "error"
```

## When Not To Use It

- When you intentionally want to replace all falsy values
- When targeting environments that don't support nullish coalescing (ES2020)

## The Difference

| Value | `\|\|` Result | `??` Result |
|-------|-------------|-------------|
| `null` | default | default |
| `undefined` | default | default |
| `0` | default | `0` |
| `''` | default | `''` |
| `false` | default | `false` |
| `NaN` | default | `NaN` |

## Browser Support

Nullish coalescing (`??`) requires:
- Node.js 14+
- All modern browsers (Chrome 80+, Firefox 72+, Safari 13.1+)
- TypeScript 3.7+

## Related Rules

- [prefer-optional-chaining](prefer-optional-chaining.md) - Suggests `?.` over `&&`
