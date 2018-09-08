#!/bin/sh
set -e

(
    cd gravity-wars
    cargo +nightly build --target wasm32-unknown-unknown
)

wasm-bindgen \
    gravity-wars/target/wasm32-unknown-unknown/debug/gravity_wars.wasm \
    --out-dir .
