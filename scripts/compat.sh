#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# shellcheck source=scripts/common.sh
source "$ROOT/scripts/common.sh"
TARGET_PATH="${1:-$ROOT/jscpd/fixtures}"
if (($# > 0)); then
  shift
fi
EXTRA_ARGS=("$@")
MIN_TOKENS="${MIN_TOKENS:-20}"
MIN_LINES="${MIN_LINES:-3}"
MAX_SIZE="${MAX_SIZE:-10mb}"
FORMAT="${FORMAT:-}"
DETECTION_MODE="${DETECTION_MODE:-}"
TMP_ROOT="${TMP_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/jscpd-rs-compat.XXXXXX")}"
RUST_OUT="$TMP_ROOT/rust"
UPSTREAM_OUT="$TMP_ROOT/upstream"

jscpd_rs_trap_tmp_root_cleanup
jscpd_rs_prepare_release_tools

mkdir -p "$RUST_OUT" "$UPSTREAM_OUT"

rust_cmd=(
  "$ROOT/target/release/jscpd"
  "$TARGET_PATH"
  --reporters json
  --output "$RUST_OUT"
  --silent
  --min-tokens "$MIN_TOKENS"
  --min-lines "$MIN_LINES"
  --max-size "$MAX_SIZE"
  --exitCode 0
)
node_cmd=(
  node "$ROOT/jscpd/apps/jscpd/bin/jscpd"
  "$TARGET_PATH"
  --reporters json
  --output "$UPSTREAM_OUT"
  --silent
  --noTips
  --min-tokens "$MIN_TOKENS"
  --min-lines "$MIN_LINES"
  --max-size "$MAX_SIZE"
  --exitCode 0
)

if [[ -n "$FORMAT" ]]; then
  rust_cmd+=(--format "$FORMAT")
  node_cmd+=(--format "$FORMAT")
fi
if [[ -n "$DETECTION_MODE" ]]; then
  rust_cmd+=(--mode "$DETECTION_MODE")
  node_cmd+=(--mode "$DETECTION_MODE")
fi
jscpd_rs_append_extra_args

jscpd_rs_print_detection_header

"${rust_cmd[@]}"
"${node_cmd[@]}"

node "$ROOT/scripts/compare-reports.mjs" \
  "$RUST_OUT/jscpd-report.json" \
  "$UPSTREAM_OUT/jscpd-report.json"

if [[ "${KEEP:-0}" == "1" ]]; then
  printf '\nrust report: %s\n' "$RUST_OUT/jscpd-report.json"
  printf 'upstream report: %s\n' "$UPSTREAM_OUT/jscpd-report.json"
fi
