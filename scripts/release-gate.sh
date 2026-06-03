#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cd "$ROOT"

run_step() {
  local name="$1"
  shift
  local start
  local end
  local code
  start="$(date +%s)"
  printf '\n== %s ==\n' "$name"
  set +e
  "$@"
  code=$?
  set -e
  end="$(date +%s)"
  printf '== %s completed in %ss ==\n' "$name" "$((end - start))"
  return "$code"
}

reporter_zero_compat() {
  local reporter_zero_dir
  local code
  reporter_zero_dir="$(mktemp -d "${TMPDIR:-/tmp}/jscpd-rs-reporters-zero.XXXXXX")"
  mkdir -p "$reporter_zero_dir/src"
  cat >"$reporter_zero_dir/src/unique.js" <<'EOF'
const alpha = 1;
const beta = 2;
const gamma = 3;
console.log(alpha + beta + gamma);
EOF
  MIN_TOKENS=50 MIN_LINES=5 scripts/compat-reporters.sh "$reporter_zero_dir/src"
  code=$?
  rm -rf "$reporter_zero_dir"
  return "$code"
}

full_compat_matrix() {
  if [[ "${FULL:-0}" == "1" ]]; then
    STRICT="${STRICT:-coverage}" scripts/compat-matrix.sh
  else
    printf 'Skipping full compatibility matrix. Run FULL=1 scripts/release-gate.sh before publication.\n'
  fi
}

public_benchmark_suite() {
  if [[ "${PUBLIC:-0}" == "1" ]]; then
    CASES="${PUBLIC_CASES:-react,next,prometheus}" \
      RUNS="${PUBLIC_RUNS:-1}" \
      CHECK_COMPAT="${PUBLIC_CHECK_COMPAT:-1}" \
      MIN_SPEEDUP="${PUBLIC_MIN_SPEEDUP:-45}" \
      scripts/public-bench-suite.sh
  else
    printf 'Skipping public benchmark suite. Run PUBLIC=1 scripts/release-gate.sh before publication.\n'
  fi
}

rust_supply_chain_check() {
  if [[ "${SUPPLY_CHAIN:-0}" == "1" ]]; then
    scripts/cargo-deny-check.sh
  else
    printf 'Skipping Rust supply-chain check. Run SUPPLY_CHAIN=1 scripts/release-gate.sh before publication.\n'
  fi
}

socket_package_score() {
  if [[ "${SOCKET_PACKAGE_CHECK:-0}" == "1" ]]; then
    node scripts/socket-package-check.mjs
  else
    printf 'Skipping Socket npm package score check. Run SOCKET_PACKAGE_CHECK=1 scripts/release-gate.sh after npm publication.\n'
  fi
}

run_step "cargo fmt --check" cargo fmt --check

run_step "cargo test" cargo test

run_step "cargo rustdoc missing docs" cargo rustdoc --lib -- -D missing_docs

run_step "bash -n scripts/*.sh" bash -n scripts/*.sh

run_step "shellcheck scripts/*.sh" shellcheck scripts/*.sh

run_step "actionlint" scripts/actionlint.sh

run_step "package/install check" scripts/package-check.sh

run_step "Rust supply-chain check" rust_supply_chain_check

run_step "CLI compatibility" scripts/compat-cli.sh

run_step "config compatibility" scripts/compat-config.sh

run_step "reporter compatibility" scripts/compat-reporters.sh

run_step "reporter compatibility (no duplicates)" reporter_zero_compat

run_step "blame compatibility" scripts/compat-blame.sh

run_step "server compatibility" scripts/compat-server.sh

run_step "upstream CI fixture compatibility" scripts/compat-upstream-ci.sh

run_step "full compatibility matrix" full_compat_matrix

run_step "public benchmark suite" public_benchmark_suite

run_step "Socket npm package score" socket_package_score
