"use strict";

const fs = require("node:fs");
const { spawnSync } = require("node:child_process");
const {
  packageRoot,
  resolvePrebuiltBinary,
  sourceBinaryPath,
} = require("../lib/platform");

const root = packageRoot();
const cargo = process.env.CARGO || "cargo";
const binaryNames = ["jscpd", "jscpd-server"];
const prebuiltBinaries = binaryNames.map(resolvePrebuiltBinary);
const sourceBinaries = binaryNames.map(sourceBinaryPath);

if (process.env.JSCPD_RS_SKIP_POSTINSTALL === "1") {
  console.log("jscpd-rs: skipping native build because JSCPD_RS_SKIP_POSTINSTALL=1");
  process.exit(0);
}

if (prebuiltBinaries.every(Boolean)) {
  process.exit(0);
}

if (sourceBinaries.every((binary) => fs.existsSync(binary))) {
  process.exit(0);
}

console.log("jscpd-rs: building native binaries with Cargo");
const result = spawnSync(
  cargo,
  ["build", "--release", "--locked", "--bin", "jscpd", "--bin", "jscpd-server"],
  {
    cwd: root,
    stdio: "inherit",
    env: process.env,
  },
);

if (result.error) {
  if (result.error.code === "ENOENT") {
    console.error(
      "jscpd-rs: Cargo was not found. Install Rust from https://rustup.rs/ and retry.",
    );
    process.exit(1);
  }
  throw result.error;
}

if (result.signal) {
  process.kill(process.pid, result.signal);
}

process.exit(result.status ?? 0);
