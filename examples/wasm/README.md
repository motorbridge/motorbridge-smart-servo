# WASM examples

`smart_servo_wasm` is a `wasm-bindgen` JavaScript/browser binding for the browser side of MotorBridge Smart Servo.

Current scope:

- WASM owns the FashionStar packet encoder for single-servo angle queries and multi-servo sync monitor.
- WASM owns the FashionStar angle and monitor response decoders.
- WASM owns the `AngleReliability` filter (zero-crossing guard + out-of-range guard).
- JavaScript owns browser serial I/O through WebSerial, because Rust `serialport` is native-only.

## Demos

### `browser-filter-demo` — single servo, raw vs filtered angle

Visualises the `A -> 0 -> B` reliability filter for a single servo.
See [`browser-filter-demo`](browser-filter-demo/README.md).

### `browser-monitor-demo` — multi-servo live dashboard

Queries up to 7 servos simultaneously with one sync_monitor command (code 25).
Shows a real-time angle curve per servo, colour-coded by `reliable` status,
plus per-servo voltage readout in the sidebar.
See [`browser-monitor-demo`](browser-monitor-demo/).
