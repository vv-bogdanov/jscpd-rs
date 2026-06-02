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
NO_PREBUILT_DIR="$TMP_ROOT/install-no-prebuilt"
mkdir -p "$PACK_DIR" "$INSTALL_DIR" "$NO_PREBUILT_DIR"

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
  'CHANGELOG.md',
  'CONTRIBUTING.md',
  'LICENSE',
  'README.md',
  'SECURITY.md',
  'npm/bin/jscpd-rs.js',
  'npm/bin/jscpd-server.js',
  'npm/lib/platform.js',
  'npm/lib/run-binary.js',
  'npm/prebuilt-targets.json',
  'package.json',
];
const forbidden = [
  /^Cargo\.(toml|lock)$/,
  /^docs\//,
  /^examples\//,
  /^jscpd\//,
  /^npm\/scripts\//,
  /^report\//,
  /^scripts\//,
  /^skills\//,
  /^src\//,
  /^target\//,
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

node --input-type=module - "$npm_version" <<'NODE'
import fs from 'node:fs';

const [expectedVersion] = process.argv.slice(2);
const pkg = JSON.parse(fs.readFileSync('package.json', 'utf8'));
const targets = JSON.parse(fs.readFileSync('npm/prebuilt-targets.json', 'utf8'));
const forbiddenLifecycleScripts = [
  'preinstall',
  'install',
  'postinstall',
  'prepublish',
  'prepare',
];
const expectedPackages = Object.values(targets)
  .map((target) => target.packageName)
  .sort();
const optionalDependencies = pkg.optionalDependencies ?? {};
const actualPackages = Object.keys(optionalDependencies).sort();

for (const script of forbiddenLifecycleScripts) {
  if (pkg.scripts?.[script]) {
    console.error(`package.json must not define lifecycle script: ${script}`);
    process.exit(1);
  }
}

if (JSON.stringify(actualPackages) !== JSON.stringify(expectedPackages)) {
  console.error(
    `optionalDependencies do not match prebuilt targets: ${actualPackages.join(', ')}`,
  );
  process.exit(1);
}

for (const name of expectedPackages) {
  if (optionalDependencies[name] !== expectedVersion) {
    console.error(
      `optional dependency ${name} uses ${optionalDependencies[name]}, expected ${expectedVersion}`,
    );
    process.exit(1);
  }
}
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
  cd "$NO_PREBUILT_DIR"
  npm init -y >/dev/null
  npm install --omit=optional --no-audit --no-fund "$tarball" \
    >"$TMP_ROOT/npm-install-no-prebuilt.log" 2>&1
  set +e
  "./node_modules/.bin/jscpd-rs" --version \
    >"$TMP_ROOT/npm-run-no-prebuilt.log" 2>&1
  status=$?
  set -e
  if [[ "$status" -eq 0 ]]; then
    fail "jscpd-rs unexpectedly ran without a prebuilt package"
  fi
  grep -Fq "Install jscpd-rs with optional dependencies enabled" \
    "$TMP_ROOT/npm-run-no-prebuilt.log" \
    || fail "running without a prebuilt package did not print the expected install hint"
)

(
  cd "$INSTALL_DIR"
  npm init -y >/dev/null
  npm install --ignore-scripts --omit=optional --no-audit --no-fund "$tarball"
  set +e
  "./node_modules/.bin/jscpd-rs" --version >/dev/null 2>&1
  status=$?
  set -e
  if [[ "$status" -eq 0 ]]; then
    fail "main npm package unexpectedly ran without a platform package"
  fi
)

current_target="$(node -e 'process.stdout.write(require("./npm/lib/platform").currentTargetKey() || "")')"
if [[ -n "$current_target" ]]; then
  cargo build --release --locked --bin jscpd --bin jscpd-server >/dev/null

  PREBUILT_ROOT="$TMP_ROOT/prebuilt"
  PREBUILT_PACK_DIR="$TMP_ROOT/prebuilt-pack"
  PREBUILT_INSTALL_DIR="$TMP_ROOT/install-prebuilt"
  mkdir -p "$PREBUILT_ROOT" "$PREBUILT_PACK_DIR" "$PREBUILT_INSTALL_DIR"

  prebuilt_package_dir="$(
    node scripts/npm-prebuilt-package.mjs \
      --target "$current_target" \
      --bin-dir "$ROOT/target/release" \
      --out-dir "$PREBUILT_ROOT"
  )"
  npm pack "$prebuilt_package_dir" --pack-destination "$PREBUILT_PACK_DIR" --json \
    >"$TMP_ROOT/npm-prebuilt-pack.json"
  prebuilt_tarball="$(node --input-type=module - "$TMP_ROOT/npm-prebuilt-pack.json" <<'NODE'
import fs from 'node:fs';

const pack = JSON.parse(fs.readFileSync(process.argv[2], 'utf8'));
console.log(pack[0].filename);
NODE
)"
  prebuilt_tarball="$PREBUILT_PACK_DIR/$prebuilt_tarball"

  (
    cd "$PREBUILT_INSTALL_DIR"
    npm init -y >/dev/null
    npm install --ignore-scripts --no-audit --no-fund "$prebuilt_tarball"
    npm install --no-audit --no-fund "$tarball"
    test "$("./node_modules/.bin/jscpd-rs" --version)" = "$cargo_version"
    test "$("./node_modules/.bin/jscpd" --version)" = "$cargo_version"
    test "$("./node_modules/.bin/jscpd-server" --version)" = "$cargo_version"
    "./node_modules/.bin/jscpd" --help | grep -Fq 'Usage: jscpd [options] <path ...>'
  )

  npx --yes \
    --package "$prebuilt_tarball" \
    --package "$tarball" \
    jscpd-rs --version | grep -Fxq "$cargo_version"
else
  printf 'npm prebuilt smoke skipped: no prebuilt target for this platform\n'
fi

printf 'npm package check complete: %s\n' "$tarball"
