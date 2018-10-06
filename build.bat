cargo build --release --target wasm32-unknown-unknown

wasm-bindgen ^
    target\wasm32-unknown-unknown\release\gravity_wars.wasm ^
    --out-dir .

