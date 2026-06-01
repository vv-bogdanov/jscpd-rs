#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CRATE_NAME="${CRATE_NAME:-jscpd-rs}"
NPM_PACKAGE_NAME="${NPM_PACKAGE_NAME:-jscpd-rs}"
EXPECTED_JSCPD_SHA="${EXPECTED_JSCPD_SHA:-50290cfd1b60b8d0d4c2929a1367328a1dddd074}"
RUN_RELEASE_CANDIDATE="${RUN_RELEASE_CANDIDATE:-1}"

cd "$ROOT"

PACKAGE_VERSION="$(
  cargo metadata --no-deps --format-version 1 \
    | node --input-type=module -e 'let data = ""; process.stdin.on("data", chunk => data += chunk); process.stdin.on("end", () => console.log(JSON.parse(data).packages[0].version));'
)"
RELEASE_TAG="${RELEASE_TAG:-v$PACKAGE_VERSION}"

fail() {
  printf 'prepublish check failed: %s\n' "$*" >&2
  exit 1
}

section() {
  printf '\n== %s ==\n' "$*"
}

section "clean git state"
status="$(git status --short)"
if [[ -n "$status" ]]; then
  printf '%s\n' "$status" >&2
  fail "working tree is not clean"
fi
git status --short --branch

section "jscpd submodule reference"
submodule_status="$(git submodule status jscpd)"
printf '%s\n' "$submodule_status"
submodule_sha="$(awk '{print $1}' <<<"$submodule_status")"
if [[ "$submodule_sha" == +* || "$submodule_sha" == -* || "$submodule_sha" == U* ]]; then
  fail "jscpd submodule is not at the recorded commit"
fi
submodule_sha="${submodule_sha# }"
if [[ -n "$EXPECTED_JSCPD_SHA" && "$submodule_sha" != "$EXPECTED_JSCPD_SHA" ]]; then
  fail "jscpd submodule is $submodule_sha, expected $EXPECTED_JSCPD_SHA"
fi

section "release tag availability"
if git tag -l "$RELEASE_TAG" | grep -Fxq "$RELEASE_TAG"; then
  fail "local tag $RELEASE_TAG already exists"
fi
if git ls-remote --tags origin "refs/tags/$RELEASE_TAG" | grep -q .; then
  fail "remote tag $RELEASE_TAG already exists"
fi
printf 'tag %s is available locally and on origin\n' "$RELEASE_TAG"

section "crate version availability"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
if (cd "$tmp" && cargo info "${CRATE_NAME}@${PACKAGE_VERSION}" >/dev/null 2>&1); then
  fail "crate ${CRATE_NAME}@${PACKAGE_VERSION} is already published"
fi
printf 'crate %s@%s is not published yet\n' "$CRATE_NAME" "$PACKAGE_VERSION"

section "npm package version availability"
set +e
npm_view_output="$(npm view "${NPM_PACKAGE_NAME}@${PACKAGE_VERSION}" version 2>&1)"
npm_view_code=$?
set -e
if [[ "$npm_view_code" == "0" ]]; then
  printf '%s\n' "$npm_view_output" >&2
  fail "npm package ${NPM_PACKAGE_NAME}@${PACKAGE_VERSION} is already published"
fi
if ! grep -Fq "E404" <<<"$npm_view_output"; then
  printf '%s\n' "$npm_view_output" >&2
  fail "could not confirm npm package ${NPM_PACKAGE_NAME}@${PACKAGE_VERSION} availability"
fi
printf 'npm package %s@%s is not published yet\n' "$NPM_PACKAGE_NAME" "$PACKAGE_VERSION"

section "benchmark docs consistency"
benchmark_source="docs/compat-baseline.md"
benchmark_cases=(react next prometheus)
benchmark_docs=(
  "README.md"
  "docs/compat-baseline.md"
  "docs/public-benchmark-suite.md"
  "docs/release-checklist.md"
  "CHANGELOG.md"
)
for case in "${benchmark_cases[@]}"; do
  row="$(grep -E "^\| \`${case}\` \|" "$benchmark_source" | head -n1)"
  if [[ -z "$row" ]]; then
    fail "$benchmark_source is missing benchmark row for $case"
  fi
  fragment="$(awk -F'|' '{for (i = 5; i <= 7; i++) {gsub(/^[ \t]+|[ \t]+$/, "", $i); printf "%s%s", $i, (i < 7 ? " | " : "")}}' <<<"$row")"
  if [[ -z "$fragment" ]]; then
    fail "could not extract benchmark numbers for $case from $benchmark_source"
  fi
  for doc in "${benchmark_docs[@]}"; do
    if ! grep -Fq "$fragment" "$doc"; then
      fail "$doc is missing benchmark row fragment for $case: $fragment"
    fi
  done
done
printf 'benchmark rows are consistent across release docs using %s as source\n' "$benchmark_source"

if [[ "$RUN_RELEASE_CANDIDATE" == "1" ]]; then
  section "release candidate gate"
  scripts/release-candidate.sh
else
  section "release candidate gate"
  printf 'skipped because RUN_RELEASE_CANDIDATE=%s\n' "$RUN_RELEASE_CANDIDATE"
fi

section "package/install check"
scripts/package-check.sh

section "cargo publish dry run"
cargo publish --dry-run --locked

section "prepublish check complete"
printf 'ready for manual tag/publish step, pending explicit release approval\n'
