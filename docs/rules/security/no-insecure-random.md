# no-insecure-random (S012)

Disallow `Math.random()` for security-sensitive operations.

## Description

This rule detects usage of `Math.random()`, which is not cryptographically secure and should not be used for security purposes.

## Rationale

`Math.random()` is not cryptographically secure:
- Uses a pseudo-random number generator (PRNG)
- The output can be predicted if the seed is known
- Not suitable for tokens, keys, or security-sensitive IDs
- Provides insufficient entropy for security purposes

## Examples

### Bad

```javascript
// Token generation
const token = Math.random().toString(36).substring(2);

// Session ID
const sessionId = Math.random();

// Password generation
const password = Math.random().toString(36);

// OTP generation
const otp = Math.floor(Math.random() * 1000000);
```

### Good

```javascript
// Use crypto module for secure random values
const crypto = require('crypto');

// Secure token
const token = crypto.randomUUID();

// Secure random bytes
const bytes = crypto.randomBytes(16);
const hex = bytes.toString('hex');

// Secure random values (browser)
const array = new Uint8Array(16);
crypto.getRandomValues(array);

// Secure OTP
const otp = crypto.randomInt(100000, 1000000);
```

## When Math.random() Is Acceptable

`Math.random()` can be used for:
- Non-security purposes (games, animations, UI effects)
- Statistical sampling (non-adversarial contexts)
- Shuffling arrays for display (not security-critical)

```javascript
// Acceptable - animation timing
const delay = Math.random() * 1000;

// Acceptable - random color for UI
const color = `hsl(${Math.random() * 360}, 50%, 50%)`;
```

## Secure Alternatives

| Purpose | Recommended |
|---------|-------------|
| UUID generation | `crypto.randomUUID()` |
| Random bytes | `crypto.randomBytes(n)` |
| Random integer | `crypto.randomInt(min, max)` |
| Browser random | `crypto.getRandomValues()` |

## Node.js Examples

```javascript
const crypto = require('crypto');

// Random UUID
const uuid = crypto.randomUUID();
// => "1b9d6bcd-bbfd-4b2d-9b5d-ab8dfbbd4bed"

// Random bytes as hex
const token = crypto.randomBytes(32).toString('hex');
// => "a3f2c8e9d1b4..." (64 hex chars)

// Random integer in range
const otp = crypto.randomInt(100000, 999999);
// => 6-digit number
```

## Browser Examples

```javascript
// Random UUID (modern browsers)
const uuid = crypto.randomUUID();

// Random bytes
const bytes = new Uint8Array(16);
crypto.getRandomValues(bytes);

// Convert to hex
const hex = Array.from(bytes)
    .map(b => b.toString(16).padStart(2, '0'))
    .join('');
```

## Configuration

### Disable the rule

```toml
[rules]
disabled = ["S012"]
# or
disabled = ["no-insecure-random"]
```

### Change severity

```toml
[rules.severity]
"no-insecure-random" = "error"
```

## Related Rules

- [no-weak-hashing](no-weak-hashing.md) - Weak cryptographic hashes
- [no-hardcoded-secrets](no-hardcoded-secrets.md) - Hardcoded credentials
