// Closure patterns for semantic analysis testing

// Simple closure capturing outer variable
const multiplier = 2;
const double = (x: number) => x * multiplier;

// Nested closures with multiple captured variables
function createCounter(initial: number) {
    let count = initial;

    return {
        increment: () => {
            count++;
            return count;
        },
        decrement: () => {
            count--;
            return count;
        },
        getCount: () => count
    };
}

// Deeply nested closures
function outer(a: number) {
    function middle(b: number) {
        function inner(c: number) {
            return a + b + c;
        }
        return inner;
    }
    return middle;
}

// IIFE (Immediately Invoked Function Expression)
const result = (function(x: number) {
    const privateValue = x * 2;
    return privateValue + 1;
})(5);

// Closure in callback pattern
const numbers = [1, 2, 3];
const prefix = "Number: ";
const formatted = numbers.map((n) => prefix + n);

// Closure with shadowed variable
const shadowTest = 10;
function shadowExample() {
    const shadowTest = 20;
    return () => shadowTest;
}
const getShadowed = shadowExample();

// Higher-order function returning closure
function makeAdder(x: number): (y: number) => number {
    return (y: number) => x + y;
}
const add5 = makeAdder(5);

// Closure capturing loop variable (let vs var behavior)
const closures: (() => number)[] = [];
for (let i = 0; i < 3; i++) {
    closures.push(() => i);
}

// Arrow function capturing this context (lexical this)
const obj = {
    value: 42,
    getValue: function() {
        return () => this.value;
    }
};

export { createCounter, outer, makeAdder, double, result, formatted, getShadowed, add5, closures, obj };
