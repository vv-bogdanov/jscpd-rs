"use strict";

const fs = require("node:fs");
const path = require("node:path");
const { spawnSync } = require("node:child_process");
const {
  packageRoot,
  resolvePrebuiltBinary,
  sourceBinaryPath,
} = require("./platform");

function buildIfMissing(name) {
  const prebuilt = resolvePrebuiltBinary(name);
  if (prebuilt) {
    return prebuilt;
  }

  const binary = sourceBinaryPath(name);
  if (fs.existsSync(binary)) {
    return binary;
  }

  const script = path.join(packageRoot(), "npm", "scripts", "postinstall.js");
  const result = spawnSync(process.execPath, [script], {
    cwd: packageRoot(),
    stdio: "inherit",
    env: process.env,
  });

  if (result.error) {
    throw result.error;
  }
  if (result.signal) {
    process.kill(process.pid, result.signal);
  }
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
  if (!fs.existsSync(binary)) {
    console.error(`jscpd-rs: expected binary was not built: ${binary}`);
    process.exit(1);
  }

  return binary;
}

function runBinary(name, args) {
  const binary = buildIfMissing(name);
  const result = spawnSync(binary, args, {
    stdio: "inherit",
    env: process.env,
  });

  if (result.error) {
    if (result.error.code === "ENOENT") {
      console.error(`jscpd-rs: binary not found: ${binary}`);
      process.exit(1);
    }
    throw result.error;
  }
  if (result.signal) {
    process.kill(process.pid, result.signal);
  }
  process.exit(result.status ?? 0);
}

module.exports = { runBinary };
