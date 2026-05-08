# motorbridge-smart-servo

[![Release](https://img.shields.io/github/v/release/motorbridge/motorbridge-smart-servo?label=release&color=brightgreen)](https://github.com/motorbridge/motorbridge-smart-servo/releases)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![CI](https://github.com/motorbridge/motorbridge-smart-servo/actions/workflows/ci.yml/badge.svg)](https://github.com/motorbridge/motorbridge-smart-servo/actions/workflows/ci.yml)

[![Rust](https://img.shields.io/badge/rust-2021-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/python-3.9+-3776AB?logo=python&logoColor=white)](https://www.python.org/)
[![PyO3](https://img.shields.io/badge/PyO3-0.28-purple?logo=rust)](https://pyo3.rs/)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey)](https://github.com/motorbridge/motorbridge-smart-servo/releases)

Rust-first control stack for serial bus smart servos. Ships native CLI, C ABI,
PyO3 Python wheels (abi3), and WASM reliability core.

Current vendor target: **FashionStar UART smart servo**.

## Features

- Angle reliability filter — suppresses power-cycle `A -> 0 -> B` glitches
- Sync read (code 25) — query N servos in one request; ~6x faster than sequential for 7-servo setups
- Consecutive loss detection — per-servo miss counter with configurable threshold
- Vendor-neutral `SmartServoController` trait for multi-brand extension
- PyO3 + maturin abi3 wheels — Rust core compiled into Python, no ctypes
- Stable C ABI (`libsmart_servo_abi`) for native/C integration
- Rust native CLI with scan / read-angle / monitor / set-angle
- Python CLI + full API with context-manager support
- Cross-platform CI: Windows, Linux, macOS + WASM

## Project Layout

| Crate / Package | Description |
|---|---|
| `smart_servo_core` | Bus, device, error, controller abstractions + angle filter |
| `smart_servo_vendors/fashionstar` | FashionStar UART protocol implementation |
| `smart_servo_abi` | Stable C ABI (`libsmart_servo_abi.so/.dll`) |
| `smart_servo_cli` | Rust native CLI (`smart-servo`) |
| `smart_servo_py` | PyO3 native extension crate |
| `bindings/python` | maturin Python package (`motorbridge-smart-servo`) |
| `smart_servo_wasm` | WASM reliability filter core |
| `examples/python` | Python SDK examples |
| `examples/wasm` | Browser/JavaScript WASM examples |

## Quick Start — Rust CLI

```bash
# Sync read all 7 servos in one request (~24 ms, vs ~154 ms sequential)
cargo run -p smart_servo_cli -- sync-monitor --port /dev/ttyUSB0 --baudrate 1000000 --ids 0 1 2 3 4 5 6 --interval-ms 20

# Scan bus
cargo run -p smart_servo_cli -- scan --port /dev/ttyUSB0 --baudrate 1000000 --max-id 20

# Read filtered angle
cargo run -p smart_servo_cli -- read-angle --port /dev/ttyUSB0 --baudrate 1000000 --id 0 --multi-turn

# Continuous monitor at 50 Hz
cargo run -p smart_servo_cli -- monitor --port /dev/ttyUSB0 --baudrate 1000000 --id 0 --multi-turn --interval-ms 20

# Move servo
cargo run -p smart_servo_cli -- set-angle --port /dev/ttyUSB0 --baudrate 1000000 --id 0 --angle -45 --interval-ms 500
```

`read-angle` returns the filtered angle by default. Use `--raw` for raw protocol data.

## Quick Start — Python

### Install

```bash
# From GitHub Release (recommended)
pip install motorbridge_smart_servo-0.0.2-cp39-abi3-manylinux_2_17_x86_64.manylinux2014_x86_64.whl

# Or from source (requires Rust toolchain)
cd bindings/python
pip install -U maturin
pip install -e .
```

### Python API

```python
from motorbridge_smart_servo import SmartServoBus, ServoMonitor

with SmartServoBus.open(vendor="fashionstar", port="/dev/ttyUSB0", baudrate=1_000_000) as bus:
    # Scan
    print(bus.scan(max_id=20))

    # Read angle (filtered is the business-safe value)
    sample = bus.read_angle(0, multi_turn=True)
    print(sample.raw_deg, sample.filtered_deg, sample.reliable)

    # Monitor continuously
    for s in bus.monitor(0, multi_turn=True, interval_s=0.02):
        print(f"raw={s.raw_deg:9.3f} filtered={s.filtered_deg:9.3f} reliable={s.reliable}")

    # Sync read — query 7 servos in one request
    result = bus.sync_monitor([0, 1, 2, 3, 4, 5, 6])
    for sid, m in result.items():
        if m and m.reliable:
            print(f"id={sid} angle={m.angle_deg:.2f} volt={m.voltage_mv}mV")

    # Move servo
    bus.set_angle(0, -45.0, multi_turn=False, interval_ms=500)
```

### Python CLI

```bash
motorbridge-smart-servo scan --port /dev/ttyUSB0 --baudrate 1000000 --max-id 20
motorbridge-smart-servo monitor --port /dev/ttyUSB0 --baudrate 1000000 --id 0 --multi-turn
motorbridge-smart-servo set-angle --port /dev/ttyUSB0 --baudrate 1000000 --id 0 --angle -45
```

## Quick Start - WASM

The WASM crate is a `wasm-bindgen` JavaScript/browser binding. It exposes
FashionStar packet encoding/decoding and the angle reliability filter.

### Multi-servo monitor dashboard

Query up to 7 servos simultaneously via sync_monitor (code 25) and view
real-time angle curves with per-servo reliable/unreliable colour coding:

```bash
bash examples/wasm/browser-monitor-demo/build.sh
cd examples/wasm/browser-monitor-demo
python3 -m http.server 8080
```

Open `http://localhost:8080` in Chrome or Edge, click `Connect WebSerial`.

### Single-servo filter demo

Visualise the `A -> 0 -> B` reliability filter for one servo (also works in
simulation mode without hardware):

```bash
bash examples/wasm/browser-filter-demo/build.sh
cd examples/wasm/browser-filter-demo
python3 -m http.server 8080
```

On Windows PowerShell, run `examples\wasm\browser-*\build.ps1` instead.

## Platform Support

| Platform | Architecture | Native CLI | C ABI | Python Wheel | WASM |
|---|---|---|---|---|---|
| Windows | x86_64 MSVC | yes | yes | yes (abi3) | — |
| Windows | aarch64 MSVC | yes | yes | — | — |
| Linux | x86_64 GNU | yes | yes | yes (abi3) | — |
| Linux | aarch64 GNU | yes (cross) | yes (cross) | yes (abi3) | — |
| Linux | armv7hf GNU | yes (cross) | yes (cross) | — | — |
| macOS | aarch64 | yes | yes | yes (abi3) | — |
| WASM | wasm32-unknown-unknown | — | — | — | yes |

## Documentation

- [USAGE.md](USAGE.md) — CLI, Python API, wheel build, CI, and platform notes
- [USAGE_UBUNTU.md](USAGE_UBUNTU.md) — Ubuntu full guide (install, serial setup, examples, troubleshooting)
- [ARCHITECTURE.md](ARCHITECTURE.md) — Layer design, vendor boundary, angle reliability
- [VENDOR_EXTENSION.md](VENDOR_EXTENSION.md) — Adding new servo brands
- [examples/python](examples/python) — Python SDK examples
- [examples/wasm](examples/wasm) — WASM browser examples

## License

MIT
