// Class patterns for semantic analysis testing

// Basic class with constructor and methods
class Animal {
    name: string;

    constructor(name: string) {
        this.name = name;
    }

    speak(): string {
        return `${this.name} makes a sound`;
    }
}

// Class with inheritance
class Dog extends Animal {
    breed: string;

    constructor(name: string, breed: string) {
        super(name);
        this.breed = breed;
    }

    speak(): string {
        return `${this.name} barks`;
    }

    fetch(): string {
        return `${this.name} fetches the ball`;
    }
}

// Class with static members
class MathUtils {
    static PI = 3.14159;

    static square(x: number): number {
        return x * x;
    }

    static circle(radius: number): number {
        return MathUtils.PI * MathUtils.square(radius);
    }
}

// Class with getters and setters
class Temperature {
    private _celsius: number;

    constructor(celsius: number) {
        this._celsius = celsius;
    }

    get celsius(): number {
        return this._celsius;
    }

    set celsius(value: number) {
        this._celsius = value;
    }

    get fahrenheit(): number {
        return this._celsius * 9/5 + 32;
    }

    set fahrenheit(value: number) {
        this._celsius = (value - 32) * 5/9;
    }
}

// Class with private fields (ES2022)
class BankAccount {
    #balance: number;
    owner: string;

    constructor(owner: string, initialBalance: number) {
        this.owner = owner;
        this.#balance = initialBalance;
    }

    deposit(amount: number): void {
        this.#balance += amount;
    }

    withdraw(amount: number): boolean {
        if (amount <= this.#balance) {
            this.#balance -= amount;
            return true;
        }
        return false;
    }

    getBalance(): number {
        return this.#balance;
    }
}

// Abstract-like pattern using interface
interface Drawable {
    draw(): void;
}

class Circle implements Drawable {
    radius: number;

    constructor(radius: number) {
        this.radius = radius;
    }

    draw(): void {
        console.log(`Drawing circle with radius ${this.radius}`);
    }

    area(): number {
        return Math.PI * this.radius * this.radius;
    }
}

// Class expression
const Rectangle = class {
    width: number;
    height: number;

    constructor(width: number, height: number) {
        this.width = width;
        this.height = height;
    }

    area(): number {
        return this.width * this.height;
    }
};

// Nested class pattern
class Outer {
    value: number;

    constructor(value: number) {
        this.value = value;
    }

    createInner(): object {
        const outerValue = this.value;

        return class Inner {
            getValue(): number {
                return outerValue;
            }
        };
    }
}

export { Animal, Dog, MathUtils, Temperature, BankAccount, Circle, Rectangle, Outer };
export type { Drawable };
