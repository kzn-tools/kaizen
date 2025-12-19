# no-weak-hashing (S011)

Disallow weak hash algorithms (MD5, SHA1).

## Description

This rule detects usage of cryptographically broken hash algorithms like MD5 and SHA1, which should not be used for security purposes.

## Rationale

MD5 and SHA1 are cryptographically broken:
- **MD5**: Collision attacks demonstrated in 2004, practical attacks since 2008
- **SHA1**: Collision attack demonstrated in 2017 (SHAttered)
- Both can be used for password cracking with rainbow tables
- Neither provides adequate security for cryptographic purposes

## Examples

### Bad

```javascript
const crypto = require('crypto');

// MD5 - broken
const hash = crypto.createHash('md5').update(data).digest('hex');

// SHA1 - broken
const hash = crypto.createHash('sha1').update(data).digest('hex');

// Also detects via string patterns
const hasher = crypto.createHash('MD5');
```

### Good

```javascript
const crypto = require('crypto');

// SHA-256 - secure
const hash = crypto.createHash('sha256').update(data).digest('hex');

// SHA-384 or SHA-512 - more secure
const hash = crypto.createHash('sha512').update(data).digest('hex');

// For passwords, use bcrypt, scrypt, or Argon2
const bcrypt = require('bcrypt');
const passwordHash = await bcrypt.hash(password, 10);
```

## When MD5/SHA1 Might Be Acceptable

These algorithms may still be used for:
- **Non-security checksums** (file integrity for non-adversarial contexts)
- **Cache keys** (where collision doesn't matter)
- **Legacy system compatibility** (document and plan migration)

Even in these cases, consider using SHA-256 for future-proofing.

## Recommended Algorithms

| Purpose | Recommended Algorithm |
|---------|----------------------|
| General hashing | SHA-256, SHA-384, SHA-512 |
| Password storage | bcrypt, scrypt, Argon2 |
| HMAC | HMAC-SHA256, HMAC-SHA512 |
| File integrity | SHA-256 |

## Configuration

### Disable the rule

```toml
[rules]
disabled = ["S011"]
# or
disabled = ["no-weak-hashing"]
```

### Change severity

```toml
[rules.severity]
"no-weak-hashing" = "error"
```

## Password Hashing

Never use MD5, SHA1, or even SHA-256 directly for passwords. Use purpose-built password hashing:

```javascript
// WRONG - fast hash allows brute force
const hash = crypto.createHash('sha256').update(password).digest('hex');

// CORRECT - bcrypt with cost factor
const bcrypt = require('bcrypt');
const hash = await bcrypt.hash(password, 12);
const valid = await bcrypt.compare(password, hash);
```

## Related Rules

- [no-hardcoded-secrets](no-hardcoded-secrets.md) - Detects hardcoded credentials
- [no-insecure-random](no-insecure-random.md) - Detects insecure randomness
