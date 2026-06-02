#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ACTIONLINT_VERSION="${ACTIONLINT_VERSION:-1.7.12}"
TOOL_ROOT="${TOOL_ROOT:-${XDG_CACHE_HOME:-$HOME/.cache}/jscpd-rs/tools}"
BIN_DIR="$TOOL_ROOT/actionlint-$ACTIONLINT_VERSION"
BIN="$BIN_DIR/actionlint"

cd "$ROOT"

if command -v actionlint >/dev/null 2>&1; then
  exec actionlint "$@"
fi

if [[ ! -x "$BIN" ]]; then
  os="$(uname -s | tr '[:upper:]' '[:lower:]')"
  arch="$(uname -m)"
  case "$arch" in
    x86_64 | amd64) arch="amd64" ;;
    aarch64 | arm64) arch="arm64" ;;
    *)
      printf 'unsupported actionlint architecture: %s\n' "$arch" >&2
      exit 1
      ;;
  esac

  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT
  url="https://github.com/rhysd/actionlint/releases/download/v${ACTIONLINT_VERSION}/actionlint_${ACTIONLINT_VERSION}_${os}_${arch}.tar.gz"
  curl -fsSL "$url" -o "$tmp/actionlint.tar.gz"
  mkdir -p "$BIN_DIR"
  tar -xzf "$tmp/actionlint.tar.gz" -C "$BIN_DIR" actionlint
fi

exec "$BIN" "$@"
