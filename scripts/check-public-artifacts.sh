#!/usr/bin/env bash
set -euo pipefail

WASM_PATH="${1:-web/src/wasm/kaido_core_bg.wasm}"

if [[ ! -f "$WASM_PATH" ]]; then
  echo "ERROR: WASM artifact not found: $WASM_PATH" >&2
  exit 1
fi

RAW_PATHS="$(strings "$WASM_PATH" | rg -n '(/Users/|/home/|[A-Za-z]:\\Users\\|/var/folders/|Dominikas-MacBook-Air|\.local)' || true)"
LEAKS="$(printf '%s\n' "$RAW_PATHS" | rg -v '/home/user/' || true)"

if [[ -n "$LEAKS" ]]; then
  echo "ERROR: public artifact contains environment-specific paths:" >&2
  echo "$LEAKS" | sed -n '1,40p' >&2
  exit 1
fi

echo "OK: no local environment path leaks in $WASM_PATH"
