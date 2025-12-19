# no-console (Q032)

Disallow `console.*` calls.

## Description

This rule reports calls to `console` methods. Console statements are typically used for debugging and should be removed before deploying to production.

## Rationale

Leaving `console` statements in production code:
- May expose sensitive information
- Creates noise in browser developer tools
- Can impact performance (especially `console.log` with large objects)
- Indicates incomplete cleanup after debugging

## Examples

### Bad

```javascript
console.log('debug info');
console.warn('warning');
console.error('error');
console.info('info');
console.debug('debug');
console.trace('trace');
```

### Good

```javascript
// Use a proper logging library
logger.info('Application started');
logger.error('An error occurred', error);

// Or remove console statements entirely
```

## Detected Methods

All `console` methods are detected:
- `console.log()`
- `console.warn()`
- `console.error()`
- `console.info()`
- `console.debug()`
- `console.trace()`
- And other console methods

## Configuration

### Disable the rule

```toml
[rules]
disabled = ["Q032"]
# or
disabled = ["no-console"]
```

### Change severity

This rule has a default severity of `Info`. To make it stricter:

```toml
[rules.severity]
"no-console" = "warning"
# or
"no-console" = "error"
```

## When Not To Use It

- In CLI applications where console output is intended
- During active development (consider disabling temporarily)
- In test files where console output is expected

## Alternatives to Console

Instead of `console.log`, consider:

1. **Proper logging libraries**: Winston, Pino, Bunyan
2. **Debug utilities**: Node.js `util.debuglog`, debug package
3. **Environment-aware logging**: Only log in development

```javascript
// Using a logger with levels
import { logger } from './logger';

logger.debug('Debug info');  // Only in development
logger.info('User logged in');
logger.error('Database connection failed', error);
```
