#!/bin/sh
set -e

(
    cd gravity-wars
    cargo +nightly build --release --target wasm32-unknown-unknown
)

wasm-bindgen \
    gravity-wars/target/wasm32-unknown-unknown/release/gravity_wars.wasm \
    --out-dir .
