// CommonJS module syntax - no errors expected

const fs = require('fs');
const path = require('path');

const ENCODING = 'utf-8';

function readFileSync(filePath) {
    const fullPath = path.resolve(filePath);
    return fs.readFileSync(fullPath, ENCODING);
}

function writeFileSync(filePath, content) {
    const fullPath = path.resolve(filePath);
    fs.writeFileSync(fullPath, content, ENCODING);
}

class FileManager {
    constructor(basePath) {
        this.basePath = basePath;
    }

    read(filename) {
        return readFileSync(path.join(this.basePath, filename));
    }

    write(filename, content) {
        writeFileSync(path.join(this.basePath, filename), content);
    }
}

module.exports = {
    readFileSync,
    writeFileSync,
    FileManager,
    ENCODING
};
