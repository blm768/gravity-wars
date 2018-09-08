cd gravity-wars 
cargo build --target wasm32-unknown-unknown
cd ..

wasm-bindgen ^
    gravity-wars\target\wasm32-unknown-unknown\debug\gravity_wars.wasm ^
    --out-dir .

