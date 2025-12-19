# prefer-optional-chaining (Q022)

Suggest optional chaining (`?.`) instead of `&&` for property access.

## Description

This rule detects patterns where `&&` is used to guard property access and suggests using optional chaining (`?.`) instead, which is more concise and readable.

## Rationale

Optional chaining:
- Is more concise and readable
- Clearly expresses intent (safe navigation)
- Handles `null` and `undefined` consistently
- Reduces cognitive load when reading code

## Examples

### Bad

```javascript
obj && obj.prop

obj && obj.a && obj.a.b

foo && foo.bar && foo.bar.baz
```

### Good

```javascript
obj?.prop

obj?.a?.b

foo?.bar?.baz
```

## Detected Patterns

The rule detects `&&` expressions where:
- Left side is an identifier or member expression
- Right side is a member expression
- Left side is a prefix of the right side's property path

```javascript
// Detected:
obj && obj.prop           // obj is prefix of obj.prop
obj.a && obj.a.b          // obj.a is prefix of obj.a.b
x.y.z && x.y.z.w          // x.y.z is prefix of x.y.z.w

// Not detected (different bases):
obj && other.prop
foo && bar.baz
```

## Configuration

### Disable the rule

```toml
[rules]
disabled = ["Q022"]
# or
disabled = ["prefer-optional-chaining"]
```

### Change severity

```toml
[rules.severity]
"prefer-optional-chaining" = "error"
```

## When Not To Use It

- When targeting environments that don't support optional chaining (ES2020)
- When the `&&` pattern has side effects in the left operand

## Browser Support

Optional chaining (`?.`) requires:
- Node.js 14+
- All modern browsers (Chrome 80+, Firefox 74+, Safari 13.1+)
- TypeScript 3.7+

## Related Rules

- [prefer-nullish-coalescing](prefer-nullish-coalescing.md) - Suggests `??` over `||`
