# Test Fixtures

This directory contains test fixtures for Lynx parser integration tests.

## Directory Structure

```
tests/fixtures/
├── javascript/          # JavaScript-specific fixtures
│   ├── simple.js        # Basic ES6 code
│   ├── esm.mjs          # ES modules (import/export)
│   ├── commonjs.cjs     # CommonJS modules (require)
│   └── jsx-component.jsx # React JSX
├── typescript/          # TypeScript-specific fixtures
│   ├── simple.ts        # Basic TypeScript
│   ├── interfaces.ts    # Interfaces, types, generics
│   └── tsx-component.tsx # TypeScript + JSX
├── valid/               # Valid code (no warnings/errors)
├── quality/             # Code quality issues
└── security/            # Security vulnerabilities
```

## Adding New Fixtures

### 1. Choose the right directory

- `javascript/` - For JavaScript-specific syntax testing
- `typescript/` - For TypeScript-specific syntax testing
- `valid/` - For code that should parse without any warnings
- `quality/` - For code with quality issues (unused vars, etc.)
- `security/` - For code with security vulnerabilities

### 2. Use the correct file extension

| Extension | Language Mode |
|-----------|--------------|
| `.js`     | JavaScript   |
| `.mjs`    | JavaScript (ESM) |
| `.cjs`    | JavaScript (CommonJS) |
| `.jsx`    | JavaScript + JSX |
| `.ts`     | TypeScript   |
| `.mts`    | TypeScript (ESM) |
| `.cts`    | TypeScript (CommonJS) |
| `.tsx`    | TypeScript + JSX |

### 3. Add a comment header

Include a comment at the top explaining what the fixture tests:

```javascript
// Description of what this fixture tests - expected outcome
```

### 4. Run the tests

```bash
cargo test --package lynx-core --test fixture_tests
```

## Snapshot Testing

AST snapshots use [insta](https://insta.rs/). To update snapshots:

```bash
# Review and accept new snapshots
cargo insta review

# Or accept all new snapshots
cargo insta accept
```

If cargo-insta is not installed:
```bash
cargo install cargo-insta
```
