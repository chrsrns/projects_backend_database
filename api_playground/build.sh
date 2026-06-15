#!/bin/bash
set -e

echo "Building API Playground WASM app..."

# Build the WASM binary
cargo build --target wasm32-unknown-unknown --release

# Generate JS bindings with wasm-bindgen
wasm-bindgen --target web --no-typescript \
    --out-dir dist \
    ../target/wasm32-unknown-unknown/release/api_playground.wasm

# Copy index.html to dist
cp index.html dist/

echo "Build complete! Files are in the dist/ directory."
echo "To serve locally, run:"
echo "  python3 -m http.server 9000 --directory dist"
