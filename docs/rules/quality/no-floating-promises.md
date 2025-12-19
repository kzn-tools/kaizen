# no-floating-promises (Q021)

Require Promises to be awaited, caught, or explicitly ignored.

## Description

This rule detects Promise values that are not handled, which can lead to silent failures and unhandled rejections.

## Rationale

Unhandled Promises:
- Can fail silently without any indication
- May cause unhandled rejection warnings/errors
- Make debugging difficult when errors occur
- Can lead to race conditions and unpredictable behavior

## Examples

### Bad

```javascript
async function foo() {
    doAsyncWork();  // Promise ignored
}

fetchData();  // Promise result lost

promise.then(handleSuccess);  // Rejection not handled
```

### Good

```javascript
async function foo() {
    await doAsyncWork();  // Awaited
}

await fetchData();  // Awaited

promise.then(handleSuccess).catch(handleError);  // Caught

void promise;  // Explicitly ignored with void
```

## Detected Patterns

The rule detects:
- Async function calls without `await`
- Functions returning Promises (pattern-based detection)
- `.then()` calls without `.catch()`

## Handling Options

1. **await**: Best for sequential async operations
   ```javascript
   await doAsyncWork();
   ```

2. **catch**: Handle errors explicitly
   ```javascript
   doAsyncWork().catch(err => console.error(err));
   ```

3. **void**: Explicitly ignore (fire-and-forget)
   ```javascript
   void doAsyncWork();
   ```

## Auto-fix

This rule provides an auto-fix to add `await` before the Promise expression.

## Configuration

### Disable the rule

```toml
[rules]
disabled = ["Q021"]
# or
disabled = ["no-floating-promises"]
```

### Change severity

```toml
[rules.severity]
"no-floating-promises" = "error"
```

## When Not To Use It

- In fire-and-forget scenarios (use `void` instead)
- When Promises are intentionally unhandled (rare)

## Related Rules

- [prefer-using](prefer-using.md) - For async disposable resources
