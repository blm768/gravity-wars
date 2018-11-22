Param(
    [switch]
    [alias("r")]
    $Release
)

$CargoArgs = "--target", "wasm32-unknown-unknown"
if ($Release) {
    $CargoArgs += "--release"
}

cargo build @CargoArgs

wasm-bindgen `
    target\wasm32-unknown-unknown\release\gravity_wars.wasm `
    --out-dir .
