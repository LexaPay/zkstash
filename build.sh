#!/bin/bash
set -e

echo "Building zkstash contract..."
cargo build --target wasm32-unknown-unknown --release

echo "Optimization complete. WASM binary is located at:"
echo "target/wasm32-unknown-unknown/release/zkstash_contract.wasm"
