// Edge cases for semantic analysis testing

// Variable shadowing across scopes
const x = 1;
{
    const x = 2;
    {
        const x = 3;
        console.log(x); // 3
    }
    console.log(x); // 2
}
console.log(x); // 1

// Function shadowing
function shadow() {
    return 'outer';
}
{
    function shadow() {
        return 'inner';
    }
    shadow();
}
shadow();

// Parameter shadowing
const param = 'global';
function useParam(param: string) {
    return param;
}
useParam('local');
console.log(param);

// Var hoisting edge cases
function varHoisting() {
    console.log(hoisted); // undefined due to hoisting
    var hoisted = 'value';
    console.log(hoisted);

    if (true) {
        var insideIf = 'inside';
    }
    console.log(insideIf); // accessible due to var hoisting
}

// Let temporal dead zone
function tdzExample() {
    // Accessing 'letVar' here would cause TDZ error
    const beforeLet = 'safe';
    let letVar = 'initialized';
    console.log(letVar);
}

// Complex destructuring
const obj = {
    a: 1,
    b: {
        c: 2,
        d: {
            e: 3
        }
    },
    f: [4, 5, 6]
};

const { a, b: { c, d: { e } }, f: [first, ...rest] } = obj;

// Destructuring with defaults
const { g = 10, h: renamed = 20 } = { g: undefined };

// Array destructuring with holes
const arr = [1, 2, 3, 4, 5];
const [, second, , fourth] = arr;

// Rest parameters and spread
function restParams(first: number, ...rest: number[]) {
    return first + rest.reduce((a, b) => a + b, 0);
}

// Default parameter using previous parameter
function defaults(a: number, b: number = a * 2) {
    return a + b;
}

// Computed property names (edge case for symbol tracking)
const propName = 'dynamic';
const computed = {
    [propName]: 'value',
    [`${propName}Suffix`]: 'another'
};

// For-in and for-of with destructuring
const items = [{ key: 'a', value: 1 }, { key: 'b', value: 2 }];
for (const { key, value } of items) {
    console.log(key, value);
}

// Try-catch-finally scoping
try {
    throw new Error('test');
} catch (error) {
    console.log(error);
    const catchScoped = 'only in catch';
} finally {
    const finallyScoped = 'only in finally';
}

// Generator function with yield
function* generator() {
    const a = yield 1;
    const b = yield 2;
    return a + b;
}

// Async/await patterns
async function asyncExample() {
    const result = await Promise.resolve(42);
    return result;
}

// Labeled statements (edge case)
outer: for (let i = 0; i < 3; i++) {
    inner: for (let j = 0; j < 3; j++) {
        if (i === j) continue outer;
        console.log(i, j);
    }
}

// With statement (deprecated but valid syntax)
// Note: 'with' is not allowed in strict mode, but parser should handle it
// with (Math) { const pi = PI; } // Would fail in strict mode

// Eval and arguments (special identifiers)
function evalAndArgs() {
    const result = eval('1 + 1');
    console.log(arguments);
    return result;
}

// Re-declaration with var (allowed)
var redeclared = 1;
var redeclared = 2;

// Export/import interaction
export { x as shadowedX, shadow, computed, generator, asyncExample };
