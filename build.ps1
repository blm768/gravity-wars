Param(
    [switch]
    [alias("r")]
    $Release
)

$CargoOpts = "--target", "wasm32-unknown-unknown"
$TargetDir = "debug"

if ($Release) {
    $CargoOpts += "--release"
    $TargetDir = "release"
}

cargo build @CargoOpts

wasm-bindgen `
    "target\wasm32-unknown-unknown\$TargetDir\gravity_wars.wasm" `
    --out-dir .
