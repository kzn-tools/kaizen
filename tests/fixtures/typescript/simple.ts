// Basic TypeScript code - no errors expected

const PI: number = 3.14159;

function calculateArea(radius: number): number {
    return PI * radius * radius;
}

function formatResult(value: number, decimals: number = 2): string {
    return value.toFixed(decimals);
}

const radius: number = 5;
const area: number = calculateArea(radius);
const formatted: string = formatResult(area);

console.log(`Area of circle with radius ${radius}: ${formatted}`);

export { calculateArea, formatResult };
