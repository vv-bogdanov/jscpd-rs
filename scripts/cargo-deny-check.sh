#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CARGO_DENY_VERSION="${CARGO_DENY_VERSION:-0.19.8}"

cd "$ROOT"

if ! cargo deny --version >/dev/null 2>&1; then
  cargo install cargo-deny --version "$CARGO_DENY_VERSION" --locked
fi

cargo deny check
