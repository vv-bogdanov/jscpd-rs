#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cd "$ROOT"

if ! cargo llvm-cov --version >/dev/null 2>&1; then
  cat >&2 <<'EOF'
coverage check requires cargo-llvm-cov.

Install it with:
  cargo install cargo-llvm-cov --locked
EOF
  exit 127
fi

mkdir -p target/coverage

coverage_args=()
case "${SCOPE:-full}" in
  full)
    coverage_args+=(--all-targets)
    ;;
  core)
    coverage_args+=(--all-targets)
    coverage_args+=(
      --ignore-filename-regex
      '(^|/)src/(app\.rs|main\.rs|server\.rs|bin/|server/)'
    )
    ;;
  *)
    printf 'unknown coverage SCOPE: %s\n' "$SCOPE" >&2
    printf 'supported scopes: full, core\n' >&2
    exit 2
    ;;
esac

if [[ -n "${FAIL_UNDER_LINES:-}" ]]; then
  coverage_args+=(--fail-under-lines "$FAIL_UNDER_LINES")
fi
if [[ -n "${FAIL_UNDER_FILE_LINES:-}" ]]; then
  coverage_args+=(--fail-under-file-lines "$FAIL_UNDER_FILE_LINES")
fi

if [[ "${SUMMARY:-0}" == "1" ]]; then
  cargo llvm-cov "${coverage_args[@]}" --summary-only "$@"
elif [[ "${HTML:-0}" == "1" ]]; then
  cargo llvm-cov "${coverage_args[@]}" --html --output-dir target/coverage/html "$@"
else
  cargo llvm-cov "${coverage_args[@]}" --lcov --output-path target/coverage/lcov.info "$@"
fi
