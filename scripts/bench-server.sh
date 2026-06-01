#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# shellcheck source=scripts/common.sh
source "$ROOT/scripts/common.sh"

TARGET="${TARGET:-$ROOT/jscpd/fixtures/javascript}"
SNIPPET="${SNIPPET:-$ROOT/jscpd/fixtures/javascript/file1.js}"
RUNS="${RUNS:-20}"
WARMUP="${WARMUP:-3}"
MIN_TOKENS="${MIN_TOKENS:-40}"
MIN_LINES="${MIN_LINES:-5}"
MAX_SIZE="${MAX_SIZE:-1mb}"
FORMAT="${FORMAT:-javascript}"
MIN_SPEEDUP="${MIN_SPEEDUP:-0}"
RUST_PORT="${RUST_PORT:-40181}"
UPSTREAM_PORT="${UPSTREAM_PORT:-40182}"
TMP_ROOT="${TMP_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/jscpd-rs-server-bench.XXXXXX")}"
RUST_PID=""
UPSTREAM_PID=""

cleanup() {
  if [[ -n "$RUST_PID" ]]; then
    kill "$RUST_PID" >/dev/null 2>&1 || true
  fi
  if [[ -n "$UPSTREAM_PID" ]]; then
    kill "$UPSTREAM_PID" >/dev/null 2>&1 || true
  fi
  jscpd_rs_cleanup_tmp_root
}
trap cleanup EXIT

require_tool() {
  if ! command -v "$1" >/dev/null 2>&1; then
    printf 'bench-server requires %s\n' "$1" >&2
    exit 127
  fi
}

require_tool curl
require_tool node

cd "$ROOT"
jscpd_rs_prepare_release_tools
cargo build --release --bin jscpd-server >/dev/null

PAYLOAD="$TMP_ROOT/snippet.json"
node --input-type=module - "$SNIPPET" "$PAYLOAD" "$FORMAT" <<'NODE'
import fs from 'node:fs';

const [snippetPath, payloadPath, format] = process.argv.slice(2);
const code = fs.readFileSync(snippetPath, 'utf8');
fs.writeFileSync(payloadPath, JSON.stringify({ code, format }));
NODE

start_server() {
  local label="$1"
  local port="$2"
  shift 2
  local log="$TMP_ROOT/$label.log"

  "$@" "$TARGET" \
    --host 127.0.0.1 \
    --port "$port" \
    --format "$FORMAT" \
    --min-tokens "$MIN_TOKENS" \
    --min-lines "$MIN_LINES" \
    --max-size "$MAX_SIZE" \
    >"$log" 2>&1 &
  local pid=$!

  for _ in $(seq 1 150); do
    if curl -fsS "http://127.0.0.1:$port/api/health" >/dev/null 2>&1; then
      printf '%s' "$pid"
      return 0
    fi
    if ! kill -0 "$pid" >/dev/null 2>&1; then
      printf '%s server exited before becoming ready\n' "$label" >&2
      sed -n '1,160p' "$log" >&2
      return 1
    fi
    sleep 0.1
  done

  printf '%s server did not become ready\n' "$label" >&2
  sed -n '1,160p' "$log" >&2
  return 1
}

check_snippet() {
  local port="$1"
  local output="$2"
  local code
  code="$(
    curl -sS -o "$output" -w '%{http_code}' \
      -H 'Content-Type: application/json' \
      -d @"$PAYLOAD" \
      "http://127.0.0.1:$port/api/check"
  )"
  if [[ "$code" != "200" ]]; then
    printf 'server check returned HTTP %s on port %s\n' "$code" "$port" >&2
    sed -n '1,160p' "$output" >&2
    return 1
  fi
}

measure_server() {
  local label="$1"
  local port="$2"
  local total="0"
  local output="$TMP_ROOT/$label-check.json"

  for _ in $(seq 1 "$WARMUP"); do
    check_snippet "$port" "$output"
  done

  printf '%s /api/check\n' "$label"
  for run in $(seq 1 "$RUNS"); do
    local start_ns
    local end_ns
    local seconds
    start_ns="$(date +%s%N)"
    check_snippet "$port" "$output"
    end_ns="$(date +%s%N)"
    seconds="$(awk -v start="$start_ns" -v end="$end_ns" 'BEGIN { printf "%.6f", (end - start) / 1000000000 }')"
    printf '  run %s: %ss\n' "$run" "$seconds"
    total="$(awk -v a="$total" -v b="$seconds" 'BEGIN { printf "%.6f", a + b }')"
  done
  local avg
  avg="$(awk -v total="$total" -v runs="$RUNS" 'BEGIN { printf "%.6f", total / runs }')"
  printf '  avg: %ss\n' "$avg"
  printf '%s' "$avg" >"$TMP_ROOT/$label-avg.txt"
}

printf 'target: %s\n' "$TARGET"
printf 'snippet: %s\n' "$SNIPPET"
printf 'runs: %s, warmup: %s\n' "$RUNS" "$WARMUP"
printf 'min tokens: %s, min lines: %s, max size: %s, format: %s\n\n' \
  "$MIN_TOKENS" "$MIN_LINES" "$MAX_SIZE" "$FORMAT"

RUST_PID="$(start_server rust "$RUST_PORT" "$ROOT/target/release/jscpd-server")"
UPSTREAM_PID="$(start_server upstream "$UPSTREAM_PORT" node "$ROOT/jscpd/apps/jscpd-server/bin/jscpd-server")"

measure_server rust "$RUST_PORT" | tee "$TMP_ROOT/rust-times.txt"
rust_avg="$(<"$TMP_ROOT/rust-avg.txt")"
printf '\n'
measure_server upstream "$UPSTREAM_PORT" | tee "$TMP_ROOT/upstream-times.txt"
upstream_avg="$(<"$TMP_ROOT/upstream-avg.txt")"
printf '\n'

speedup="$(awk -v rust="$rust_avg" -v upstream="$upstream_avg" 'BEGIN { printf "%.2f", upstream / rust }')"
printf 'server check speedup: %sx\n' "$speedup"

if [[ "$MIN_SPEEDUP" != "0" ]]; then
  awk -v speedup="$speedup" -v min="$MIN_SPEEDUP" 'BEGIN {
    if (speedup + 0 < min + 0) {
      printf "server benchmark failed: %.2fx < %.2fx\n", speedup, min > "/dev/stderr";
      exit 1;
    }
  }'
fi
