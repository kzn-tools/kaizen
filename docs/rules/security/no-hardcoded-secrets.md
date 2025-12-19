# no-hardcoded-secrets (S010)

Disallow hardcoded secrets (API keys, tokens, passwords).

## Description

This rule detects hardcoded secrets, API keys, passwords, and other sensitive credentials in source code using pattern matching and entropy analysis.

## Rationale

Hardcoded secrets in source code:
- Get committed to version control and exposed in git history
- Are visible to anyone with code access
- Cannot be rotated without code changes
- May be exposed in error messages or logs
- Violate security compliance requirements (SOC2, PCI-DSS, etc.)

## Examples

### Bad

```javascript
const apiKey = "sk_live_1234567890abcdef";

const config = {
    password: "super_secret_password",
    token: "ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
};

const AWS_SECRET = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";
```

### Good

```javascript
const apiKey = process.env.API_KEY;

const config = {
    password: process.env.DB_PASSWORD,
    token: process.env.GITHUB_TOKEN
};

const AWS_SECRET = process.env.AWS_SECRET_ACCESS_KEY;

// Or use a secrets manager
const apiKey = await secretsManager.getSecret('api-key');
```

## Detected Patterns

The rule detects various secret patterns:

| Type | Pattern |
|------|---------|
| API Keys | Generic API key patterns |
| AWS | `AKIA*`, secret key patterns |
| GitHub | `ghp_*`, `gho_*`, `ghu_*` |
| Stripe | `sk_live_*`, `sk_test_*` |
| Private Keys | `-----BEGIN * PRIVATE KEY-----` |
| JWTs | `eyJ*` (base64 encoded) |
| Generic | High-entropy strings in sensitive contexts |

## Context Detection

The rule analyzes variable names and contexts:
- Variables named `password`, `secret`, `key`, `token`, `credential`
- Configuration objects with sensitive field names
- String assignments in suspicious contexts

## Configuration

### Disable the rule

```toml
[rules]
disabled = ["S010"]
# or
disabled = ["no-hardcoded-secrets"]
```

## Secrets Management Best Practices

1. **Environment variables** - Simple and widely supported
   ```javascript
   const key = process.env.API_KEY;
   ```

2. **Secrets managers** - AWS Secrets Manager, HashiCorp Vault, etc.
   ```javascript
   const key = await vault.get('api-key');
   ```

3. **Configuration files** - Keep out of version control
   ```javascript
   // config.local.js (in .gitignore)
   module.exports = { apiKey: 'your-key' };
   ```

4. **.env files** - For local development only
   ```
   # .env (in .gitignore)
   API_KEY=your-key
   ```

## False Positives

If you have false positives for test/example values:

```javascript
// Use clearly fake values
const exampleKey = "EXAMPLE_KEY_NOT_REAL";
const testToken = "test_token_12345";
```

## When Not To Use It

This rule should always be enabled. There are no valid reasons to hardcode secrets.

## Compliance

This rule helps with:
- **SOC 2** - Secure credential management
- **PCI-DSS** - Protection of authentication credentials
- **HIPAA** - Access control requirements
- **GDPR** - Security of personal data processing
