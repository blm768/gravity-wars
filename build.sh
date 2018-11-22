#!/bin/bash
set -e

cargo_opts=(--target wasm32-unknown-unknown)

case "$1" in
    -r|--release)
        cargo_opts+=(--release)
    ;;
esac

cargo +nightly build "${cargo_opts[@]}"

wasm-bindgen \
    target/wasm32-unknown-unknown/release/gravity_wars.wasm \
    --out-dir .
