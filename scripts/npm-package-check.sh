#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP_ROOT="${TMP_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/jscpd-rs-npm.XXXXXX")}"

cleanup() {
  rm -rf "$TMP_ROOT"
}
trap cleanup EXIT

cd "$ROOT"

fail() {
  printf 'npm package check failed: %s\n' "$*" >&2
  exit 1
}

if ! command -v npm >/dev/null 2>&1; then
  fail "npm is required"
fi
if ! command -v node >/dev/null 2>&1; then
  fail "node is required"
fi

cargo_version="$(
  cargo metadata --no-deps --format-version 1 \
    | node --input-type=module -e 'let data = ""; process.stdin.on("data", chunk => data += chunk); process.stdin.on("end", () => console.log(JSON.parse(data).packages[0].version));'
)"
npm_version="$(node -p 'require("./package.json").version')"
if [[ "$npm_version" != "$cargo_version" ]]; then
  fail "package.json version $npm_version does not match Cargo.toml version $cargo_version"
fi

PACK_DIR="$TMP_ROOT/pack"
INSTALL_DIR="$TMP_ROOT/install"
FAIL_DIR="$TMP_ROOT/install-no-cargo"
mkdir -p "$PACK_DIR" "$INSTALL_DIR" "$FAIL_DIR"

npm pack --pack-destination "$PACK_DIR" --json >"$TMP_ROOT/npm-pack.json"
tarball="$(node --input-type=module - "$TMP_ROOT/npm-pack.json" <<'NODE'
import fs from 'node:fs';

const pack = JSON.parse(fs.readFileSync(process.argv[2], 'utf8'));
console.log(pack[0].filename);
NODE
)"
tarball="$PACK_DIR/$tarball"

node --input-type=module - "$TMP_ROOT/npm-pack.json" <<'NODE'
import fs from 'node:fs';

const pack = JSON.parse(fs.readFileSync(process.argv[2], 'utf8'))[0];
const files = pack.files.map((file) => file.path).sort();
const required = [
  'Cargo.toml',
  'Cargo.lock',
  'docs/migrating-from-jscpd.md',
  'docs/user-guide.md',
  'examples/library_api.rs',
  'LICENSE',
  'README.md',
  'npm/bin/jscpd-rs.js',
  'npm/bin/jscpd-server.js',
  'npm/lib/run-binary.js',
  'npm/scripts/postinstall.js',
  'package.json',
  'skills/dry-refactoring/SKILL.md',
  'skills/jscpd/SKILL.md',
  'src/main.rs',
  'src/bin/jscpd-server.rs',
];
const forbidden = [
  /^jscpd\//,
  /^target\//,
  /^report\//,
  /^scripts\//,
  /(^|\/)node_modules\//,
];

for (const path of required) {
  if (!files.includes(path)) {
    console.error(`npm package is missing required file: ${path}`);
    process.exit(1);
  }
}
for (const path of files) {
  const match = forbidden.find((pattern) => pattern.test(path));
  if (match) {
    console.error(`npm package includes forbidden file: ${path}`);
    process.exit(1);
  }
}
console.log(`npm package file count: ${files.length}`);
NODE

set +e
npm publish --dry-run --json \
  >"$TMP_ROOT/npm-publish-dry-run.json" \
  2>"$TMP_ROOT/npm-publish-dry-run.err"
npm_publish_dry_run_status=$?
set -e
if [[ "$npm_publish_dry_run_status" -eq 0 ]]; then
  node --input-type=module - "$TMP_ROOT/npm-publish-dry-run.json" "$npm_version" <<'NODE'
import fs from 'node:fs';

const [file, expectedVersion] = process.argv.slice(2);
const publish = JSON.parse(fs.readFileSync(file, 'utf8'));
if (publish.name !== 'jscpd-rs' || publish.version !== expectedVersion) {
  console.error(
    `unexpected npm publish dry-run package: ${publish.name}@${publish.version}`,
  );
  process.exit(1);
}
console.log(`npm publish dry-run: ${publish.name}@${publish.version}`);
NODE
elif grep -Fq "cannot publish over the previously published versions" \
  "$TMP_ROOT/npm-publish-dry-run.err"; then
  printf 'npm publish dry-run skipped: jscpd-rs@%s is already published\n' "$npm_version"
else
  cat "$TMP_ROOT/npm-publish-dry-run.err" >&2
  fail "npm publish dry-run failed"
fi

(
  cd "$FAIL_DIR"
  npm init -y >/dev/null
  set +e
  CARGO="$TMP_ROOT/missing-cargo" npm install --no-audit --no-fund "$tarball" \
    >"$TMP_ROOT/npm-install-no-cargo.log" 2>&1
  status=$?
  set -e
  if [[ "$status" -eq 0 ]]; then
    fail "npm install unexpectedly succeeded without Cargo"
  fi
  grep -Fq \
    "jscpd-rs: Cargo was not found. Install Rust from https://rustup.rs/ and retry." \
    "$TMP_ROOT/npm-install-no-cargo.log" \
    || fail "npm install without Cargo did not print the expected Rust toolchain hint"
)

if [[ "${NPM_PACKAGE_CHECK_REUSE_TARGET:-0}" == "1" ]]; then
  export CARGO_TARGET_DIR="$ROOT/target"
fi

(
  cd "$INSTALL_DIR"
  npm init -y >/dev/null
  npm install --no-audit --no-fund "$tarball"
  test "$("./node_modules/.bin/jscpd-rs" --version)" = "$cargo_version"
  test "$("./node_modules/.bin/jscpd" --version)" = "$cargo_version"
  test "$("./node_modules/.bin/jscpd-server" --version)" = "$cargo_version"
  "./node_modules/.bin/jscpd" --help | grep -Fq 'Usage: jscpd [options] <path ...>'
)

npx --yes --package "$tarball" jscpd-rs --version | grep -Fxq "$cargo_version"

printf 'npm package check complete: %s\n' "$tarball"
