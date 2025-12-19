# no-eval-injection (S005)

Disallow code execution with untrusted data.

## Description

This rule uses taint analysis to detect when untrusted user input flows into code execution functions like `eval()`, which could allow attackers to execute arbitrary JavaScript.

## Rationale

Code injection via eval allows attackers to:
- Execute arbitrary JavaScript in your context
- Access sensitive data and secrets
- Modify application behavior
- Escalate privileges
- Compromise the entire application

## Examples

### Bad

```javascript
function handler(req, res) {
    const code = req.body.code;
    eval(code);  // Code injection
}

function process(req, res) {
    const userCode = req.query.code;
    new Function(userCode)();  // Code injection
}

function execute(req, res) {
    const callback = req.body.callback;
    setTimeout(callback, 100);  // Code injection if callback is a string
}
```

### Good

```javascript
// Avoid eval entirely
function handler(req, res) {
    const action = req.body.action;
    // Use a lookup table instead
    const handlers = {
        'greet': () => 'Hello',
        'bye': () => 'Goodbye'
    };
    const result = handlers[action]?.();
    res.send(result);
}

// Use function references, not strings
setTimeout(() => console.log('test'), 100);

// Static code is safe
eval('const x = 1');  // Not flagged - no tainted data
```

## Taint Sources

The rule tracks data from:
- `req.body.*` - Request body
- `req.query.*` - Query parameters
- `req.params.*` - URL parameters
- `process.env.*` - Environment variables
- `process.argv` - Command line arguments

## Taint Sinks

Code execution functions:
- `eval()`
- `new Function()`
- `setTimeout()` with string argument
- `setInterval()` with string argument
- `vm.runInContext()`
- `vm.runInNewContext()`
- `vm.runInThisContext()`

## Prevention

1. **Never use eval with user input** - there's almost always a better way
2. **Use function references** instead of strings in setTimeout/setInterval
3. **Use lookup tables** for dynamic dispatch
4. **Parse data, don't execute it** - use JSON.parse for data
5. **Sandbox untrusted code** if you must execute it

### Alternative Patterns

```javascript
// Instead of eval for dynamic property access
// BAD
eval(`obj.${prop}`);
// GOOD
obj[prop];

// Instead of eval for JSON
// BAD
eval('(' + jsonString + ')');
// GOOD
JSON.parse(jsonString);

// Instead of Function for dynamic logic
// BAD
new Function('x', 'y', userCode);
// GOOD
const operations = { add: (x, y) => x + y };
operations[userOp]?.(x, y);
```

## Configuration

### Disable the rule

```toml
[rules]
disabled = ["S005"]
# or
disabled = ["no-eval-injection"]
```

## When Not To Use It

This rule should always be enabled. If you have a legitimate need to execute user code, it must be properly sandboxed.

## Related Rules

- [no-eval](../quality/no-eval.md) - Quality rule for all eval usage
- [no-command-injection](no-command-injection.md) - Shell command injection
- [no-sql-injection](no-sql-injection.md) - SQL injection
