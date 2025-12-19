# no-eval (Q034)

Disallow `eval()` and similar dangerous patterns.

## Description

This rule detects uses of `eval()` and other patterns that execute arbitrary strings as code, which are security risks and performance anti-patterns.

## Rationale

`eval()` and similar patterns:
- Create security vulnerabilities (code injection)
- Prevent engine optimizations
- Make code harder to understand and debug
- Can break strict mode
- Prevent proper static analysis

## Examples

### Bad

```javascript
eval("console.log('hello')");

new Function("return x + 1");

setTimeout("alert('hi')", 100);

setInterval("doWork()", 1000);
```

### Good

```javascript
console.log('hello');

const fn = (x) => x + 1;

setTimeout(() => alert('hi'), 100);

setInterval(() => doWork(), 1000);
```

## Detected Patterns

| Pattern | Alternative |
|---------|-------------|
| `eval(code)` | Avoid entirely; use proper parsing |
| `new Function(code)` | Use arrow functions or regular functions |
| `setTimeout(string, ms)` | Use `setTimeout(fn, ms)` |
| `setInterval(string, ms)` | Use `setInterval(fn, ms)` |

## Security Note

**Never pass user input to `eval()` or similar functions.** This creates a code injection vulnerability.

For security-sensitive scenarios, see:
- [no-eval-injection](../security/no-eval-injection.md) - Detects tainted data in eval

## Configuration

### Disable the rule

```toml
[rules]
disabled = ["Q034"]
# or
disabled = ["no-eval"]
```

### Change severity

```toml
[rules.severity]
"no-eval" = "error"
```

## When Not To Use It

Rare legitimate uses include:
- REPL/console tools that need to execute user code (in sandbox)
- Code editors/playgrounds (with proper sandboxing)
- Serialization/deserialization of functions (consider alternatives)

Even in these cases, consider alternatives:
- `JSON.parse()` for data
- Abstract Syntax Tree (AST) manipulation for code transformation
- Sandboxed iframes or workers for untrusted code

## Related Rules

- [no-eval-injection](../security/no-eval-injection.md) - Security-focused eval detection with taint analysis
