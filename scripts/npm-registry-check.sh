#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP_ROOT="${TMP_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/jscpd-rs-registry.XXXXXX")}"
REQUIRE_PUBLISHED="${NPM_REGISTRY_REQUIRE_PUBLISHED:-0}"
RETRIES="${NPM_REGISTRY_RETRIES:-1}"
RETRY_DELAY_MS="${NPM_REGISTRY_RETRY_DELAY_MS:-15000}"

cleanup() {
  rm -rf "$TMP_ROOT"
}
trap cleanup EXIT

cd "$ROOT"

fail() {
  printf 'npm registry check failed: %s\n' "$*" >&2
  exit 1
}

version="${1:-$(node -p 'require("./package.json").version')}"

mapfile -t package_names < <(
  printf '%s\n' "$(node -p 'require("./package.json").name')"
  node --input-type=module <<'NODE'
import fs from 'node:fs';

const pkg = JSON.parse(fs.readFileSync('package.json', 'utf8'));
for (const name of Object.keys(pkg.optionalDependencies ?? {}).sort()) {
  console.log(name);
}
NODE
)

for attempt in $(seq 1 "$RETRIES"); do
  missing=0
  for package_name in "${package_names[@]}"; do
    spec="${package_name}@${version}"
    if ! npm view "$spec" dist --json >"$TMP_ROOT/${package_name}.dist.json" 2>"$TMP_ROOT/${package_name}.err"; then
      missing=1
      printf '%s is not available in npm registry yet\n' "$spec" >&2
    fi
  done
  if [[ "$missing" == "0" ]]; then
    break
  fi
  if [[ "$attempt" != "$RETRIES" ]]; then
    sleep "$(node -e "console.log(Number(process.argv[1]) / 1000)" "$RETRY_DELAY_MS")"
  fi
done
if [[ "$missing" != "0" && "$REQUIRE_PUBLISHED" != "1" ]]; then
  printf 'npm registry check skipped: jscpd-rs@%s package set is not fully published yet\n' "$version"
  exit 0
fi

for package_name in "${package_names[@]}"; do
  spec="${package_name}@${version}"
  dist_file="$TMP_ROOT/${package_name}.dist.json"
  if [[ ! -s "$dist_file" ]]; then
    if [[ "$REQUIRE_PUBLISHED" == "1" ]]; then
      cat "$TMP_ROOT/${package_name}.err" >&2 || true
      fail "$spec is not published"
    fi
    printf 'npm registry check skipped: %s is not published yet\n' "$spec"
    exit 0
  fi
  node --input-type=module - "$dist_file" "$spec" <<'NODE'
import fs from 'node:fs';

const [distFile, spec] = process.argv.slice(2);
const dist = JSON.parse(fs.readFileSync(distFile, 'utf8'));

if (!dist.integrity) {
  console.error(`${spec} is missing dist.integrity`);
  process.exit(1);
}
if (!Array.isArray(dist.signatures) || dist.signatures.length === 0) {
  console.error(`${spec} is missing npm registry signatures`);
  process.exit(1);
}
if (dist.attestations?.provenance?.predicateType !== 'https://slsa.dev/provenance/v1') {
  console.error(`${spec} is missing npm SLSA provenance attestation`);
  process.exit(1);
}
console.log(`${spec}: registry integrity, signature, and provenance are present`);
NODE
done

install_dir="$TMP_ROOT/install"
mkdir -p "$install_dir"
(
  cd "$install_dir"
  npm init -y >/dev/null
  npm install --ignore-scripts --no-audit --no-fund "jscpd-rs@${version}" >/dev/null
  npm audit signatures
)
