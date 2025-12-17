// ES Module syntax - no errors expected

import { readFile } from 'fs/promises';
import path from 'path';

export const VERSION = '1.0.0';

export function greet(name) {
    return `Hello, ${name}!`;
}

export async function loadConfig(configPath) {
    const fullPath = path.resolve(configPath);
    const content = await readFile(fullPath, 'utf-8');
    return JSON.parse(content);
}

export default class Logger {
    constructor(prefix = '') {
        this.prefix = prefix;
    }

    log(message) {
        console.log(`${this.prefix}${message}`);
    }
}
