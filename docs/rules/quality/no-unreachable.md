# no-unreachable (Q004)

Disallow unreachable code after return, throw, break, or continue statements.

## Description

This rule detects code that can never be executed because it comes after a statement that always exits the current block (like `return`, `throw`, `break`, or `continue`).

## Rationale

Unreachable code:
- Is never executed and wastes space
- Often indicates logical errors or incomplete refactoring
- Can confuse developers reading the code
- May hide bugs if the code was intended to execute

## Examples

### Bad

```javascript
function foo() {
    return 1;
    const x = 2;  // Unreachable
}

function bar() {
    throw new Error("error");
    console.log("never runs");  // Unreachable
}

for (let i = 0; i < 10; i++) {
    break;
    console.log(i);  // Unreachable
}
```

### Good

```javascript
function foo() {
    if (condition) {
        return 1;
    }
    const x = 2;  // Reachable - if condition is false
    return x;
}

function bar() {
    try {
        return riskyOperation();
    } catch (e) {
        console.log("error caught");  // Reachable
        throw e;
    }
}
```

## Features

- **Control flow analysis**: Understands if/else, switch, and try/catch
- **All terminating statements**: Handles return, throw, break, continue
- **Function hoisting**: Function declarations after return are allowed (JavaScript hoisting)
- **Nested scope support**: Detects unreachable code in nested blocks

## Smart Detection

The rule understands when code is actually reachable:

```javascript
// Not flagged - only one branch returns
if (x) {
    return 1;
}
const y = 2;  // Reachable if x is falsy

// Flagged - all branches return
if (x) {
    return 1;
} else {
    return 2;
}
const y = 3;  // Unreachable
```

## Configuration

### Disable the rule

```toml
[rules]
disabled = ["Q004"]
# or
disabled = ["no-unreachable"]
```

### Change severity

```toml
[rules.severity]
"no-unreachable" = "error"
```

## When Not To Use It

This rule should generally always be enabled as unreachable code is almost always a mistake.
