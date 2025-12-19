# no-command-injection (S003)

Disallow shell commands constructed with untrusted data.

## Description

This rule uses taint analysis to detect when untrusted user input flows into shell command execution, which could allow attackers to execute arbitrary system commands.

## Rationale

Command injection allows attackers to:
- Execute arbitrary system commands
- Read or modify sensitive files
- Install malware or backdoors
- Compromise the entire server
- Pivot to other systems on the network

## Examples

### Bad

```javascript
function handler(req, res) {
    const filename = req.body.filename;
    exec("cat " + filename);  // Command injection
}

function process(req, res) {
    const pattern = req.query.pattern;
    exec(`grep ${pattern} /etc/passwd`);  // Command injection
}

function deploy(req, res) {
    const args = req.body.args;
    spawn(args);  // Command injection
}
```

### Good

```javascript
function handler(req, res) {
    const filename = req.body.filename;
    // Use execFile with an arguments array
    execFile("cat", [filename], (error, stdout) => {
        res.send(stdout);
    });
}

function process(req, res) {
    const pattern = req.query.pattern;
    // Sanitize with shell-escape
    const safePattern = shellEscape(pattern);
    exec(`grep ${safePattern} /etc/passwd`);
}

// Better: avoid shell entirely
function readFile(req, res) {
    const filename = req.body.filename;
    const content = fs.readFileSync(filename, 'utf8');
    res.send(content);
}
```

## Taint Sources

The rule tracks data from:
- `req.body.*` - Request body
- `req.query.*` - Query parameters
- `req.params.*` - URL parameters
- `process.env.*` - Environment variables
- `process.argv` - Command line arguments

## Taint Sinks

Shell execution functions:
- `exec()`
- `execSync()`
- `spawn()` (when first arg is tainted)
- `spawnSync()`
- `child_process.exec()`
- `child_process.spawn()`

## Prevention

1. **Use execFile** with an arguments array instead of exec
2. **Avoid shell execution** entirely when possible
3. **Use built-in Node.js APIs** (fs, path, etc.) instead of shell commands
4. **Sanitize input** with shell-escape libraries
5. **Validate input** against allowlists

### execFile vs exec

```javascript
// BAD - exec uses shell, vulnerable to injection
exec("ls " + userInput);

// GOOD - execFile doesn't use shell
execFile("ls", [userInput]);
```

## Configuration

### Disable the rule

```toml
[rules]
disabled = ["S003"]
# or
disabled = ["no-command-injection"]
```

## When Not To Use It

This rule should always be enabled for server-side applications.

## Related Rules

- [no-sql-injection](no-sql-injection.md) - SQL injection
- [no-eval-injection](no-eval-injection.md) - Code execution injection
