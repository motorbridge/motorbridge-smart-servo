# motorbridge-smart-servo

Version: `v0.0.1`

Rust-first control stack for serial bus smart servos.

The project mirrors the MotorBridge shape:

- `smart_servo_core`: shared bus, device, error, and controller abstractions
- `smart_servo_vendors/fashionstar`: FashionStar UART protocol implementation
- `smart_servo_abi`: stable C ABI for native/C integration
- `smart_servo_cli`: native CLI for scan/read/move/debug
- `smart_servo_py`: PyO3 native Python extension crate
- `bindings/python`: maturin Python package using the PyO3 extension

Current vendor target: FashionStar UART smart servo.

## Quick CLI

```bash
cargo run -p smart_servo_cli -- scan --port COM5 --baudrate 1000000 --max-id 20
cargo run -p smart_servo_cli -- read-angle --port COM5 --baudrate 1000000 --id 0 --multi-turn
cargo run -p smart_servo_cli -- monitor --port COM5 --baudrate 1000000 --id 0 --multi-turn --interval-ms 20
```

`read-angle` returns the filtered angle by default. Use `--raw` for raw protocol data.

## Python Binding

Python uses PyO3 + maturin. The Rust core is compiled directly into the Python
extension module, so the wheel is platform tagged and does not load an external
`smart_servo_abi.dll/.so` through `ctypes`.

Install for development:

```bash
cd bindings/python
python -m pip install -U maturin
python -m pip install -e .
```

Build a wheel:

```bash
cd bindings/python
python -m maturin build --release --out dist
```

Use it from Python:

```python
from motorbridge_smart_servo import SmartServoBus

with SmartServoBus.open(vendor="fashionstar", port="COM5", baudrate=1_000_000) as bus:
    print(bus.scan(max_id=20))
    sample = bus.read_angle(0, multi_turn=True)
    print(sample.raw_deg, sample.filtered_deg, sample.reliable)
```

`sample.filtered_deg` is the business-safe value. `sample.raw_deg` is the protocol raw value.

Python CLI after install:

```bash
motorbridge-smart-servo scan --port COM5 --baudrate 1000000 --max-id 20
motorbridge-smart-servo monitor --port COM5 --baudrate 1000000 --id 0 --multi-turn
```

See [USAGE.md](USAGE.md) for native CLI, Python CLI, Python API, wheel, CI, and platform notes.
See [VENDOR_EXTENSION.md](VENDOR_EXTENSION.md) for adding new brands without changing old interfaces.
