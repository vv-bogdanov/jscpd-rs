"use strict";

const fs = require("node:fs");
const { spawnSync } = require("node:child_process");
const {
  currentTarget,
  resolvePrebuiltBinary,
  sourceBinaryPath,
} = require("./platform");

function missingBinaryExit(name) {
  const target = currentTarget();
  if (target) {
    console.error(
      `jscpd-rs: native package ${target.packageName} is not installed or does not contain ${name}.`,
    );
  } else {
    console.error(
      `jscpd-rs: no prebuilt native package is available for ${process.platform}/${process.arch}.`,
    );
  }
  console.error(
    "Install jscpd-rs with optional dependencies enabled, or use: cargo install jscpd-rs --locked",
  );
  process.exit(1);
}

function resolveRunnableBinary(name) {
  const prebuilt = resolvePrebuiltBinary(name);
  if (prebuilt) {
    return prebuilt;
  }

  const binary = sourceBinaryPath(name);
  if (fs.existsSync(binary)) {
    return binary;
  }

  missingBinaryExit(name);
}

function runBinary(name, args) {
  const binary = resolveRunnableBinary(name);
  const result = spawnSync(binary, args, {
    stdio: "inherit",
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
