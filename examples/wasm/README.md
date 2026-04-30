# WASM examples

`smart_servo_wasm` is a `wasm-bindgen` JavaScript/browser binding for the protocol-independent reliability core.

Current scope:

- Available in WASM: `WasmAngleReliability` and `WasmAngleSample`.
- Not in WASM yet: serial UART transport and FashionStar command protocol.

That means the browser can use the exact Rust filter to suppress `A -> 0 -> B` angle glitches, but direct hardware control still needs WebSerial integration or a native bridge that feeds raw angle samples into the WASM filter.

## Browser filter demo

See [`browser-filter-demo`](browser-filter-demo/README.md).
