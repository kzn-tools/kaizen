// Quality issue: unused variables

const unusedConstant: number = 42;

let unusedVariable: string = "never used";

function processData(input: number): number {
    const temporaryResult: number = input * 2;
    const unusedInFunction: string = "also unused";
    return temporaryResult;
}

function helperFunction(param: string): string {
    const unusedParam: string = param;
    return "result";
}

const result: number = processData(10);
console.log(result);
