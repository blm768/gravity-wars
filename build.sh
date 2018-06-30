#!/bin/sh
set -e

(
    cd gravity-wars
    cargo build --target wasm32-unknown-unknown
)

mkdir -p wasm

wasm-bindgen \
    gravity-wars/target/wasm32-unknown-unknown/debug/gravity_wars.wasm \
    --no-modules \
    --out-dir wasm
