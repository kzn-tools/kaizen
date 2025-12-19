const fs = require("fs");
const path = require("path");
const zlib = require("zlib");
const https = require("https");

const PLATFORMS = {
  "linux-x64": "x86_64-unknown-linux-gnu",
  "linux-arm64": "aarch64-unknown-linux-gnu",
  "darwin-x64": "x86_64-apple-darwin",
  "darwin-arm64": "aarch64-apple-darwin",
  "win32-x64": "x86_64-pc-windows-msvc",
};

const VERSION = require("./package.json").version;
const BINARY_NAME = "kaizen-cli";
const REPO = "mpiton/kaizen";

function getPlatformPackageName() {
  const platformKey = `${process.platform}-${process.arch}`;
  return `@kaizen/cli-${platformKey.replace("-", "-")}`;
}

function isPlatformPackageInstalled() {
  const platformKey = `${process.platform}-${process.arch}`;
  const binaryName = process.platform === "win32" ? `${BINARY_NAME}.exe` : BINARY_NAME;
  try {
    require.resolve(`@kaizen/cli-${platformKey}/bin/${binaryName}`);
    return true;
  } catch {
    return false;
  }
}

function makeRequest(url) {
  return new Promise((resolve, reject) => {
    https
      .get(url, (response) => {
        if (response.statusCode >= 200 && response.statusCode < 300) {
          const chunks = [];
          response.on("data", (chunk) => chunks.push(chunk));
          response.on("end", () => resolve(Buffer.concat(chunks)));
        } else if (response.statusCode >= 300 && response.statusCode < 400 && response.headers.location) {
          makeRequest(response.headers.location).then(resolve, reject);
        } else {
          reject(new Error(`Request failed with status ${response.statusCode}`));
        }
      })
      .on("error", reject);
  });
}

function extractFromTarGz(buffer, filename) {
  const unzipped = zlib.gunzipSync(buffer);
  let offset = 0;

  while (offset < unzipped.length) {
    const header = unzipped.subarray(offset, offset + 512);
    offset += 512;

    const name = header.toString("utf-8", 0, 100).replace(/\0.*/g, "");
    if (!name) break;

    const sizeStr = header.toString("utf-8", 124, 136).replace(/\0.*/g, "").trim();
    const size = parseInt(sizeStr, 8) || 0;

    if (name === filename || name.endsWith(`/${filename}`)) {
      return unzipped.subarray(offset, offset + size);
    }

    offset += Math.ceil(size / 512) * 512;
  }

  throw new Error(`File ${filename} not found in archive`);
}

function extractFromZip(buffer, filename) {
  // Simple ZIP extraction for single file
  let offset = 0;
  while (offset < buffer.length) {
    const sig = buffer.readUInt32LE(offset);
    if (sig !== 0x04034b50) break; // Local file header signature

    const compMethod = buffer.readUInt16LE(offset + 8);
    const compSize = buffer.readUInt32LE(offset + 18);
    const uncompSize = buffer.readUInt32LE(offset + 22);
    const nameLen = buffer.readUInt16LE(offset + 26);
    const extraLen = buffer.readUInt16LE(offset + 28);
    const name = buffer.toString("utf-8", offset + 30, offset + 30 + nameLen);

    const dataOffset = offset + 30 + nameLen + extraLen;

    if (name === filename || name.endsWith(`/${filename}`)) {
      if (compMethod === 0) {
        return buffer.subarray(dataOffset, dataOffset + uncompSize);
      } else if (compMethod === 8) {
        return zlib.inflateRawSync(buffer.subarray(dataOffset, dataOffset + compSize));
      }
      throw new Error(`Unsupported compression method: ${compMethod}`);
    }

    offset = dataOffset + compSize;
  }

  throw new Error(`File ${filename} not found in archive`);
}

async function downloadBinary() {
  const platformKey = `${process.platform}-${process.arch}`;
  const target = PLATFORMS[platformKey];

  if (!target) {
    console.error(`Unsupported platform: ${platformKey}`);
    console.error(`Supported platforms: ${Object.keys(PLATFORMS).join(", ")}`);
    process.exit(1);
  }

  const isWindows = process.platform === "win32";
  const ext = isWindows ? "zip" : "tar.gz";
  const binaryName = isWindows ? `${BINARY_NAME}.exe` : BINARY_NAME;
  const archiveName = `${BINARY_NAME}-${target}.${ext}`;
  const url = `https://github.com/${REPO}/releases/download/v${VERSION}/${archiveName}`;

  console.log(`Downloading ${archiveName}...`);

  const buffer = await makeRequest(url);
  const binaryData = isWindows ? extractFromZip(buffer, binaryName) : extractFromTarGz(buffer, binaryName);

  const destPath = path.join(__dirname, binaryName);
  fs.writeFileSync(destPath, binaryData, { mode: 0o755 });

  console.log(`Installed ${binaryName}`);
}

(async () => {
  if (isPlatformPackageInstalled()) {
    return;
  }

  try {
    await downloadBinary();
  } catch (error) {
    console.error(`Failed to install binary: ${error.message}`);
    console.error("Please report this issue at https://github.com/mpiton/kaizen/issues");
    process.exit(1);
  }
})();
