$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path "$PSScriptRoot\..\..\.."
Push-Location $repoRoot
try {
    rustup target add wasm32-unknown-unknown
    cargo install wasm-bindgen-cli
    cargo build --release -p smart_servo_wasm --target wasm32-unknown-unknown
    wasm-bindgen target\wasm32-unknown-unknown\release\smart_servo_wasm.wasm --out-dir examples\wasm\browser-filter-demo\pkg --target web
    Write-Host "Built WASM demo package at examples\wasm\browser-filter-demo\pkg"
}
finally {
    Pop-Location
}
