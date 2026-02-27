#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="$ROOT/web/src/wasm"
TARGET_WASM="$ROOT/target/wasm32-unknown-unknown/release/kaido_core.wasm"

# Sanitize debug/source metadata so published WASM does not leak local absolute paths.
RUSTFLAGS_VALUE="-C debuginfo=0 --remap-path-prefix=$ROOT=/workspace --remap-path-prefix=$HOME=/home/user"

RUSTFLAGS="$RUSTFLAGS_VALUE" cargo build \
  --manifest-path "$ROOT/crates/kaido-core/Cargo.toml" \
  --target wasm32-unknown-unknown \
  --release \
  --features wasm

wasm-bindgen \
  --target web \
  --out-dir "$OUT_DIR" \
  --out-name kaido_core \
  "$TARGET_WASM"

# Optional strip for smaller artifacts / less metadata.
if command -v wasm-strip >/dev/null 2>&1; then
  wasm-strip "$OUT_DIR/kaido_core_bg.wasm"
fi

echo "WASM generated in $OUT_DIR"
