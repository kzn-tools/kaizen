# prefer-using (Q020)

Require `using` or `await using` for disposable resources.

## Description

This rule detects variables that hold disposable resources (like file handles) and suggests using the `using` or `await using` declaration for automatic resource cleanup.

## Rationale

The `using` declaration (ECMAScript Explicit Resource Management) ensures resources are automatically disposed when they go out of scope, preventing:
- Resource leaks (unclosed files, connections)
- Memory leaks
- Lock contention from unreleased locks
- Database connection exhaustion

## Examples

### Bad

```javascript
const file = await open('./data.txt');
// file might not be closed if an error occurs

const handle = await fsPromises.open('./file');
// handle could leak

const conn = await connectToDatabase();
// connection might not be released
```

### Good

```javascript
await using file = await open('./data.txt');
// file is automatically closed when scope exits

await using handle = await fsPromises.open('./file');
// handle is automatically closed

using resource = getResource();
// resource is automatically disposed (sync)
```

## Features

- **Known types**: High confidence detection for known disposable types (FileHandle, etc.)
- **Heuristic detection**: Medium confidence for function names like `open`, `connect`, `acquire`
- **Return exemption**: Variables that are returned are not flagged (disposal is caller's responsibility)
- **Async awareness**: Suggests `await using` vs `using` based on context

## Auto-fix

This rule provides an auto-fix to replace `const` or `let` with `using` or `await using`.

## Detected Patterns

### High Confidence (Warning)
- `fsPromises.open()` - Returns FileHandle
- `open()` function calls

### Medium Confidence (Info)
Function names matching patterns like:
- `acquire*`, `*Lock`
- `connect*`, `*Connection`
- `open*`, `*Handle`
- `create*Pool*`, `create*Stream*`

## Configuration

### Disable the rule

```toml
[rules]
disabled = ["Q020"]
# or
disabled = ["prefer-using"]
```

### Change severity

```toml
[rules.severity]
"prefer-using" = "error"
```

## When Not To Use It

- When targeting environments that don't support `using` declarations
- When you need to pass resources to other scopes that manage their lifecycle

## Browser Support

The `using` declaration requires:
- Node.js 22+ (or with `--experimental-vm-modules` in earlier versions)
- TypeScript 5.2+
- Babel with appropriate plugins for browser targets
