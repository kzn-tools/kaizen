#!/usr/bin/env node

const { execFileSync } = require("child_process");
const path = require("path");

const PLATFORMS = {
  "linux-x64": "kzn-cli-linux-x64",
  "linux-arm64": "kzn-cli-linux-arm64",
  "darwin-x64": "kzn-cli-darwin-x64",
  "darwin-arm64": "kzn-cli-darwin-arm64",
  "win32-x64": "kzn-cli-win32-x64",
};

function getBinaryPath() {
  const platformKey = `${process.platform}-${process.arch}`;
  const platformPackage = PLATFORMS[platformKey];

  if (!platformPackage) {
    console.error(`Unsupported platform: ${platformKey}`);
    console.error(`Supported platforms: ${Object.keys(PLATFORMS).join(", ")}`);
    process.exit(1);
  }

  const binaryName = process.platform === "win32" ? "kaizen-cli.exe" : "kaizen-cli";

  // Try to load from optional dependency first
  try {
    return require.resolve(`${platformPackage}/bin/${binaryName}`);
  } catch {
    // Fallback to local binary (downloaded via postinstall)
    const localBinary = path.join(__dirname, "..", binaryName);
    const fs = require("fs");
    if (fs.existsSync(localBinary)) {
      return localBinary;
    }

    console.error(`Binary not found. Please try reinstalling kzn-cli`);
    process.exit(1);
  }
}

try {
  const binaryPath = getBinaryPath();
  execFileSync(binaryPath, process.argv.slice(2), { stdio: "inherit" });
} catch (error) {
  if (error.status !== undefined) {
    process.exit(error.status);
  }
  throw error;
}
