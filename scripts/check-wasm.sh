#!/bin/sh
# Turbo Vision without its terminal: what an embedder (a plank WASM frame)
# builds. Needs rustup's cargo and `rustup target add wasm32-wasip1`.
set -e
cd "$(dirname "$0")/.."
cargo check --lib --no-default-features --target wasm32-wasip1
