#!/usr/bin/env bash

jscpd_rs_cleanup_tmp_root() {
  if [[ "${KEEP:-0}" != "1" ]]; then
    rm -rf "$TMP_ROOT"
  fi
}

jscpd_rs_trap_tmp_root_cleanup() {
  trap jscpd_rs_cleanup_tmp_root EXIT
}

jscpd_rs_prepare_release_tools() {
  if [[ -f "$HOME/.cargo/env" ]]; then
    # shellcheck source=/dev/null
    source "$HOME/.cargo/env"
  fi

  if command -v corepack >/dev/null 2>&1; then
    corepack prepare pnpm@10.28.0 --activate >/dev/null
  fi

  cd "$ROOT"
  cargo build --release >/dev/null

  if [[ ! -d "$ROOT/jscpd/node_modules" ]]; then
    pnpm --dir "$ROOT/jscpd" install --frozen-lockfile
  fi

  if [[ ! -f "$ROOT/jscpd/apps/jscpd/dist/bin/jscpd.js" ]]; then
    pnpm --dir "$ROOT/jscpd" build
  fi
}

jscpd_rs_append_extra_args() {
  if ((${#EXTRA_ARGS[@]} > 0)); then
    rust_cmd+=("${EXTRA_ARGS[@]}")
    node_cmd+=("${EXTRA_ARGS[@]}")
  fi
}

jscpd_rs_print_detection_header() {
  printf 'target: %s\n' "$TARGET_PATH"
  if [[ -n "${REPORTERS:-}" ]]; then
    printf 'reporters: %s\n' "$REPORTERS"
  fi
  printf 'min tokens: %s, min lines: %s, max size: %s\n' "$MIN_TOKENS" "$MIN_LINES" "$MAX_SIZE"
  if [[ -n "$FORMAT" ]]; then
    printf 'format: %s\n' "$FORMAT"
  fi
  if [[ -n "$DETECTION_MODE" ]]; then
    printf 'mode: %s\n' "$DETECTION_MODE"
  fi
  if ((${#EXTRA_ARGS[@]} > 0)); then
    printf 'extra args:'
    printf ' %q' "${EXTRA_ARGS[@]}"
    printf '\n'
  fi
  printf 'tmp: %s\n\n' "$TMP_ROOT"
}
