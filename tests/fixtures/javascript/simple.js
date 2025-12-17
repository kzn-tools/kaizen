// Basic JavaScript ES6 code - no errors expected

const PI = 3.14159;
let counter = 0;

function calculateArea(radius) {
    return PI * radius * radius;
}

const square = (x) => x * x;

class Calculator {
    constructor(initialValue = 0) {
        this.value = initialValue;
    }

    add(n) {
        this.value += n;
        return this;
    }

    result() {
        return this.value;
    }
}

const calc = new Calculator(10);
const result = calc.add(5).add(3).result();

export { calculateArea, square, Calculator };
