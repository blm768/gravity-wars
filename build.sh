#!/bin/bash
set -e

cargo_opts=(--target wasm32-unknown-unknown)
target_dir=debug

case "$1" in
    -r|--release)
        cargo_opts+=(--release)
        target_dir=release
    ;;
esac

cargo +nightly build "${cargo_opts[@]}"

wasm-bindgen \
    "target/wasm32-unknown-unknown/${target_dir}/gravity_wars.wasm" \
    --target web \
    --out-dir .
