cd gravity-wars 
cargo build --release --target wasm32-unknown-unknown
cd ..

wasm-bindgen ^
    gravity-wars\target\wasm32-unknown-unknown\release\gravity_wars.wasm ^
    --out-dir .

