#!/bin/bash -e 
cargo b --target wasm32-unknown-unknown --release
# wasm-bindgen --no-typescript --target web --out-dir ./out/ --out-name "rotv" ./target/wasm32-unknown-unknown/release/rotv.wasm
wasm-opt -o scavengerdrop.wasm target/wasm32-unknown-unknown/release/sj6.wasm -Os
rm release.zip || true
zip release macroquad-gamepads-0.1.js mq_js_bundle.js scavengerdrop.wasm index.html
