#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

cd "$REPO_ROOT"
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.120 --locked
cargo build --release -p smart_servo_wasm --target wasm32-unknown-unknown
wasm-bindgen \
  target/wasm32-unknown-unknown/release/smart_servo_wasm.wasm \
  --out-dir examples/wasm/browser-monitor-demo/pkg \
  --target web

echo "Built. Run:  cd examples/wasm/browser-monitor-demo && python3 -m http.server 8080"
