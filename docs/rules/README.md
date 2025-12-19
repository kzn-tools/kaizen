# Lynx Rules Reference

Comprehensive documentation for all Lynx linter rules. Use `lynx explain <rule-id>` or `lynx explain <rule-name>` to get rule details from the command line.

## Quick Links

- [Quality Rules](#quality-rules) - Code quality and best practices
- [Security Rules](#security-rules) - Security vulnerability detection

## All Rules

| ID | Name | Description | Severity | Category | Auto-fix |
|----|------|-------------|----------|----------|----------|
| [Q001](quality/no-unused-vars.md) | no-unused-vars | Disallow unused variables | Warning | Quality | - |
| [Q003](quality/no-unused-imports.md) | no-unused-imports | Disallow unused imports | Warning | Quality | Yes |
| [Q004](quality/no-unreachable.md) | no-unreachable | Disallow unreachable code | Warning | Quality | - |
| [Q010](quality/max-complexity.md) | max-complexity | Enforce maximum cyclomatic complexity | Warning | Quality | - |
| [Q011](quality/max-depth.md) | max-depth | Enforce maximum nesting depth | Warning | Quality | - |
| [Q020](quality/prefer-using.md) | prefer-using | Require using/await using for disposables | Warning | Quality | Yes |
| [Q021](quality/no-floating-promises.md) | no-floating-promises | Require Promises to be handled | Warning | Quality | Yes |
| [Q022](quality/prefer-optional-chaining.md) | prefer-optional-chaining | Suggest ?. over && | Warning | Quality | - |
| [Q023](quality/prefer-nullish-coalescing.md) | prefer-nullish-coalescing | Suggest ?? over \|\| | Warning | Quality | - |
| [Q030](quality/no-var.md) | no-var | Disallow var declarations | Warning | Quality | Yes |
| [Q031](quality/prefer-const.md) | prefer-const | Require const for never-reassigned variables | Warning | Quality | Yes |
| [Q032](quality/no-console.md) | no-console | Disallow console.* calls | Info | Quality | - |
| [Q033](quality/eqeqeq.md) | eqeqeq | Require === and !== | Warning | Quality | Yes |
| [Q034](quality/no-eval.md) | no-eval | Disallow eval() and dangerous patterns | Warning | Quality | - |
| [S001](security/no-sql-injection.md) | no-sql-injection | Disallow SQL injection vulnerabilities | Error | Security | - |
| [S002](security/no-xss.md) | no-xss | Disallow XSS vulnerabilities | Error | Security | - |
| [S003](security/no-command-injection.md) | no-command-injection | Disallow command injection | Error | Security | - |
| [S005](security/no-eval-injection.md) | no-eval-injection | Disallow code injection via eval | Error | Security | - |
| [S010](security/no-hardcoded-secrets.md) | no-hardcoded-secrets | Disallow hardcoded secrets | Error | Security | - |
| [S011](security/no-weak-hashing.md) | no-weak-hashing | Disallow weak hash algorithms | Warning | Security | - |
| [S012](security/no-insecure-random.md) | no-insecure-random | Disallow Math.random() for security | Warning | Security | - |

## Quality Rules

Rules focused on code quality, maintainability, and best practices.

### Unused Code Detection
- **[no-unused-vars](quality/no-unused-vars.md)** (Q001) - Detects variables that are declared but never used
- **[no-unused-imports](quality/no-unused-imports.md)** (Q003) - Detects imports that are never used
- **[no-unreachable](quality/no-unreachable.md)** (Q004) - Detects code after return/throw/break/continue

### Complexity
- **[max-complexity](quality/max-complexity.md)** (Q010) - Enforces a maximum cyclomatic complexity threshold
- **[max-depth](quality/max-depth.md)** (Q011) - Enforces a maximum nesting depth threshold

### Modern JavaScript
- **[prefer-using](quality/prefer-using.md)** (Q020) - Encourages `using` for disposable resources
- **[no-floating-promises](quality/no-floating-promises.md)** (Q021) - Ensures Promises are properly handled
- **[prefer-optional-chaining](quality/prefer-optional-chaining.md)** (Q022) - Suggests `?.` over `&&` patterns
- **[prefer-nullish-coalescing](quality/prefer-nullish-coalescing.md)** (Q023) - Suggests `??` over `||` for defaults
- **[no-var](quality/no-var.md)** (Q030) - Prevents use of `var` declarations
- **[prefer-const](quality/prefer-const.md)** (Q031) - Encourages `const` for non-reassigned variables

### Code Style
- **[no-console](quality/no-console.md)** (Q032) - Warns about console.* calls in production code
- **[eqeqeq](quality/eqeqeq.md)** (Q033) - Requires strict equality operators
- **[no-eval](quality/no-eval.md)** (Q034) - Prevents dangerous eval patterns

## Security Rules

Rules focused on detecting security vulnerabilities through taint analysis and pattern matching.

### Injection Vulnerabilities (Taint-based)
- **[no-sql-injection](security/no-sql-injection.md)** (S001) - Detects SQL injection via untrusted data
- **[no-xss](security/no-xss.md)** (S002) - Detects XSS via untrusted HTML in DOM
- **[no-command-injection](security/no-command-injection.md)** (S003) - Detects shell command injection
- **[no-eval-injection](security/no-eval-injection.md)** (S005) - Detects code execution with untrusted data

### Secret Management
- **[no-hardcoded-secrets](security/no-hardcoded-secrets.md)** (S010) - Detects hardcoded API keys and secrets

### Cryptography
- **[no-weak-hashing](security/no-weak-hashing.md)** (S011) - Detects weak algorithms (MD5, SHA1)
- **[no-insecure-random](security/no-insecure-random.md)** (S012) - Detects Math.random() misuse

## Configuration

### Disabling Rules

In your `lynx.toml` configuration:

```toml
[rules]
# Disable by rule ID
disabled = ["Q032", "Q034"]

# Or disable by rule name
disabled = ["no-console", "no-eval"]
```

### Changing Severity

```toml
[rules.severity]
# Change no-console from Info to Warning
"no-console" = "warning"

# Change no-var from Warning to Error
"Q030" = "error"
```

### Category Toggles

```toml
[rules]
# Disable all quality rules
quality = false

# Enable all security rules (default)
security = true
```

## Severity Levels

| Level | Description |
|-------|-------------|
| **Error** | Critical issues that should block deployment |
| **Warning** | Issues that should be addressed |
| **Info** | Suggestions for improvement |
| **Hint** | Minor suggestions |
