# eqeqeq (Q033)

Require `===` and `!==` instead of `==` and `!=`.

## Description

This rule reports uses of loose equality operators (`==` and `!=`) and suggests using strict equality operators (`===` and `!==`) instead.

## Rationale

Loose equality (`==`) performs type coercion, which can lead to unexpected results:

```javascript
0 == ''       // true
0 == '0'      // true
false == '0'  // true
null == undefined  // true
[] == false   // true
```

Strict equality (`===`) compares both value and type, making behavior predictable.

## Examples

### Bad

```javascript
if (x == 1) { }

if (x != null) { }

value == 'string'

count != 0
```

### Good

```javascript
if (x === 1) { }

if (x !== null) { }

value === 'string'

count !== 0
```

## Auto-fix

This rule provides an auto-fix to replace:
- `==` with `===`
- `!=` with `!==`

## Type Coercion Examples

| Expression | Result | Why |
|------------|--------|-----|
| `1 == '1'` | `true` | String converted to number |
| `0 == false` | `true` | Boolean converted to number |
| `null == undefined` | `true` | Special case in spec |
| `'' == false` | `true` | Both convert to 0 |
| `[] == 0` | `true` | Array becomes '' then 0 |

## Configuration

### Disable the rule

```toml
[rules]
disabled = ["Q033"]
# or
disabled = ["eqeqeq"]
```

### Change severity

```toml
[rules.severity]
"eqeqeq" = "error"
```

## When Not To Use It

The only common case for `==`:

```javascript
// Check for null or undefined at once
if (value == null) {
    // value is null or undefined
}

// Equivalent to:
if (value === null || value === undefined) {
    // value is null or undefined
}
```

However, `value == null` is often considered acceptable since it's a well-known idiom.

## TypeScript Note

In TypeScript, the type system often prevents the bugs that `===` would catch. However, using `===` is still recommended for consistency and when working with `any` types.
