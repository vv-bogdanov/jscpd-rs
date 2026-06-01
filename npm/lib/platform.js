"use strict";

const fs = require("node:fs");
const path = require("node:path");

const targets = require("../prebuilt-targets.json");

function packageRoot() {
  return path.resolve(__dirname, "..", "..");
}

function exeName(name, platform = process.platform) {
  return platform === "win32" ? `${name}.exe` : name;
}

function sourceBinaryPath(name) {
  const targetDir =
    process.env.CARGO_TARGET_DIR || path.join(packageRoot(), "target");
  return path.join(targetDir, "release", exeName(name));
}

function detectLinuxLibc() {
  if (process.platform !== "linux") {
    return undefined;
  }

  const report =
    process.report && typeof process.report.getReport === "function"
      ? process.report.getReport()
      : undefined;
  if (report && report.header && report.header.glibcVersionRuntime) {
    return "glibc";
  }
  return "musl";
}

function currentTargetKey() {
  const libc = detectLinuxLibc();
  for (const [key, target] of Object.entries(targets)) {
    if (target.os !== process.platform || target.cpu !== process.arch) {
      continue;
    }
    if (target.os === "linux" && target.libc !== libc) {
      continue;
    }
    return key;
  }
  return undefined;
}

function currentTarget() {
  const key = currentTargetKey();
  return key ? { key, ...targets[key] } : undefined;
}

function resolvePrebuiltBinary(name) {
  if (process.env.JSCPD_RS_FORCE_BUILD === "1") {
    return undefined;
  }

  const target = currentTarget();
  if (!target) {
    return undefined;
  }

  let packageJson;
  try {
    packageJson = require.resolve(`${target.packageName}/package.json`, {
      paths: [packageRoot()],
    });
  } catch {
    return undefined;
  }

  const binary = path.join(path.dirname(packageJson), "bin", exeName(name, target.os));
  return fs.existsSync(binary) ? binary : undefined;
}

function resolveBinary(name) {
  return resolvePrebuiltBinary(name) || sourceBinaryPath(name);
}

module.exports = {
  currentTarget,
  currentTargetKey,
  exeName,
  packageRoot,
  resolveBinary,
  resolvePrebuiltBinary,
  sourceBinaryPath,
  targets,
};
