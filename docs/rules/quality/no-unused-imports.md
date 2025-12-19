# no-unused-imports (Q003)

Disallow unused imports.

## Description

This rule detects imports that are never used in the code. It helps keep your import statements clean and reduces bundle size by identifying unnecessary dependencies.

## Rationale

Unused imports:
- Increase bundle size unnecessarily
- Make code harder to understand
- May cause confusion about actual dependencies
- Can slow down build times

## Examples

### Bad

```javascript
import { unused } from 'module';

import defaultExport from 'library';  // Never used

import * as namespace from 'utils';   // Never accessed
```

### Good

```javascript
import { used } from 'module';
console.log(used);

// Re-exports are allowed
import { foo } from 'module';
export { foo };

// Underscore prefix is allowed
import { _unused } from 'module';

// Side-effect imports are allowed
import 'polyfill';
```

## Features

- **Semantic analysis**: Uses symbol table for accurate detection
- **Re-export support**: Imports that are re-exported are not flagged
- **Underscore prefix exception**: Imports starting with `_` are ignored
- **Side-effect imports**: `import 'module'` syntax is not flagged
- **Alias support**: Correctly tracks renamed imports

## Auto-fix

This rule provides an auto-fix to remove the unused import.

## Configuration

### Disable the rule

```toml
[rules]
disabled = ["Q003"]
# or
disabled = ["no-unused-imports"]
```

### Change severity

```toml
[rules.severity]
"no-unused-imports" = "error"
```

## When Not To Use It

- When imports have side effects that are needed but not explicitly used
- In TypeScript with type-only imports that may not be detected

## Related Rules

- [no-unused-vars](no-unused-vars.md) - Detects unused variables
