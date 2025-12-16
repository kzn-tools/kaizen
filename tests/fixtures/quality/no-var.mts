// Quality issue: using 'var' instead of 'let' or 'const'

var globalCounter: number = 0;

function incrementCounter(): number {
    var localValue: number = 1;
    globalCounter += localValue;
    return globalCounter;
}

for (var i: number = 0; i < 10; i++) {
    var loopResult: number = i * 2;
    console.log(loopResult);
}

var config: { name: string; value: number } = {
    name: "test",
    value: 123
};

console.log(config.name);
