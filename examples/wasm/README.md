# WASM examples

`smart_servo_wasm` is a `wasm-bindgen` JavaScript/browser binding for the browser side of MotorBridge Smart Servo.

Current scope:

- WASM owns the FashionStar query-angle packet encoder.
- WASM owns the FashionStar angle response decoder.
- WASM owns the `A -> 0 -> B` angle reliability filter.
- JavaScript owns browser serial I/O through WebSerial, because Rust `serialport` is native-only.

So the browser demo can read real servo angle data when opened in a WebSerial-capable browser such as Chrome or Edge on `localhost` or HTTPS.

## Browser WebSerial demo

See [`browser-filter-demo`](browser-filter-demo/README.md).
