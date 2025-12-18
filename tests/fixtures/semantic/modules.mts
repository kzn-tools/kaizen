// Module patterns for semantic analysis testing

// Named imports
import { readFile, writeFile } from 'fs';
import { join, resolve } from 'path';

// Default import
import DefaultClass from './default-export';

// Namespace import
import * as utils from './utils';

// Mixed imports (default + named)
import React, { useState, useEffect } from 'react';

// Type imports
import type { Config } from './config';

// Re-exports
export { readFile, writeFile };
export { join as joinPath, resolve as resolvePath } from 'path';

// Default re-export
export { default as DefaultReexport } from './another-module';

// Named exports
export const VERSION = '1.0.0';
export const API_URL = 'https://api.example.com';

export function helper(x: number): number {
    return x * 2;
}

export class Service {
    name: string;

    constructor(name: string) {
        this.name = name;
    }

    run(): void {
        console.log(`Running ${this.name}`);
    }
}

// Export list
const privateValue = 42;
const anotherPrivate = 'secret';

export { privateValue as publicValue, anotherPrivate };

// Default export
export default class MainModule {
    static version = VERSION;

    init(): void {
        console.log('Initialized');
    }
}

// Type exports
export type Status = 'pending' | 'active' | 'completed';
export interface User {
    id: number;
    name: string;
    status: Status;
}

// Using imported values
const config: Config = { debug: true };
const state = useState(0);
const fullPath = join('src', 'index.ts');

// Side effects from imports (usage)
readFile('test.txt', 'utf8', (err, data) => {
    if (err) throw err;
    console.log(data);
});

utils.formatDate(new Date());
