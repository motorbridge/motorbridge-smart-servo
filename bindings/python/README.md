# motorbridge-smart-servo Python Binding Guide

This package is the PyO3 + maturin Python binding for MotorBridge Smart Servo.
It currently targets the FashionStar UART smart-servo protocol.

> Current status (important): angle write/control commands are temporarily
> considered unsupported in this project release line. Read/monitor APIs are
> the supported and recommended path.

The Rust core is compiled into `motorbridge_smart_servo._native`, so the wheel
contains native code directly. There is no runtime `ctypes.CDLL(...)` loading
step and no external ABI DLL/SO required by Python.

## What "Raw" vs "Calibrated/Filtered" Data Means

`read_angle()` returns an `AngleSample` with:

- `raw_deg`: protocol angle from the servo packet.
- `filtered_deg`: reliability-filtered angle for application logic.
- `reliable`: `False` when the filter is temporarily holding the last safe value.

For control/planning/business logic, use `filtered_deg`.

```python
sample = bus.read_angle(servo_id=0, multi_turn=True)
raw_angle = sample.raw_deg
safe_angle = sample.filtered_deg
is_reliable = sample.reliable
```

## API Summary

Main classes:

- `SmartServoBus` (vendor-neutral entry)
- `FashionStarServo` (direct vendor class)

Common methods:

- `scan(max_id=253) -> list[int]`
- `ping(servo_id) -> bool`
- `read_angle(servo_id, multi_turn=True) -> AngleSample`
- `read_raw_angle(servo_id, multi_turn=True) -> float`
- `read_filtered_angle(servo_id, multi_turn=True) -> float`
- `monitor(servo_id, multi_turn=True, interval_s=0.01, count=None)`
- `set_angle(servo_id, angle_deg, multi_turn=False, interval_ms=0)`

Note: `set_angle` is currently kept for API compatibility, but write-control
behavior is not guaranteed at this stage.

## Quick Start (Development Install)

Linux/macOS:

```bash
cd bindings/python
python -m pip install -U pip maturin
python -m pip install -e .
```

Windows PowerShell:

```powershell
cd bindings\python
python -m pip install -U pip maturin
python -m pip install -e .
```

## Basic Usage

```python
from motorbridge_smart_servo import SmartServoBus

with SmartServoBus.open(
    vendor="fashionstar",
    port="/dev/ttyUSB0",   # e.g. "COM5" on Windows
    baudrate=1_000_000,
) as bus:
    online = bus.scan(max_id=20)
    print("online:", online)

    sample = bus.read_angle(0, multi_turn=True)
    print(
        f"raw={sample.raw_deg:.3f}, "
        f"filtered={sample.filtered_deg:.3f}, "
        f"reliable={sample.reliable}"
    )

    # Use filtered angle in your control logic.
    angle_for_control = sample.filtered_deg
```

Direct vendor class:

```python
from motorbridge_smart_servo import FashionStarServo

with FashionStarServo("/dev/ttyUSB0", 1_000_000) as bus:
    print(bus.ping(0))
```

## Continuous Monitoring

```python
with SmartServoBus.open(vendor="fashionstar", port="/dev/ttyUSB0") as bus:
    for sample in bus.monitor(0, multi_turn=True, interval_s=0.01):
        print(
            f"raw={sample.raw_deg:9.3f} "
            f"filtered={sample.filtered_deg:9.3f} "
            f"reliable={sample.reliable}"
        )
```

`monitor()` keeps streaming after transient timeout once at least one valid
sample has been observed. In those moments you can see `reliable=False` while
`filtered_deg` holds the last safe value.

## Polling Frequency

The Python binding is synchronous today. Each call goes to the serial bus:

- `read_angle(...)`: one servo angle transaction.
- `sync_monitor([...])`: one sync-monitor transaction for all requested servos.
- `monitor(..., interval_s=0.01)`: calls `read_angle`, yields the sample, then
  sleeps for `interval_s`.

On the measured 7-servo FashionStar bus, one `sync_monitor([0..6])` cycle takes
about `4.4 ms`. For stable operation, use a `10 ms` target period, which gives
roughly `100 Hz` effective updates:

```python
period_s = 0.01
while True:
    t0 = time.monotonic()
    result = bus.sync_monitor([0, 1, 2, 3, 4, 5, 6])
    # use m.raw_deg, m.filtered_deg, m.reliable
    sleep_s = period_s - (time.monotonic() - t0)
    if sleep_s > 0:
        time.sleep(sleep_s)
```

Calling faster than the bus can complete only increases blocking and jitter; it
does not create fresher data. There is no background cache thread in the Python
binding yet.

## Move Command (Temporarily Unsupported)

```python
with SmartServoBus.open(vendor="fashionstar", port="/dev/ttyUSB0") as bus:
    bus.set_angle(0, -45.0, multi_turn=False, interval_ms=500)
```

For now, treat movement control as experimental and disabled in production use.
Use read/monitor methods for stable operation.

## Build a Wheel (`.whl`)

Linux/macOS:

```bash
cd bindings/python
python -m pip install -U pip maturin build twine
python -m maturin build --release --out dist
python -m twine check dist/*
python -m pip install --force-reinstall dist/*.whl
```

Windows PowerShell:

```powershell
cd bindings\python
python -m pip install -U pip maturin build twine
python -m maturin build --release --out dist
python -m twine check dist\*
python -m pip install --force-reinstall (Get-ChildItem dist\*.whl | Select-Object -Last 1).FullName
```

## Build Wheels for Publishing (Recommended)

For distribution, build in CI for each target platform (Windows/Linux/macOS)
to avoid local toolchain differences.

Suggested targets:

- Windows: `x86_64`
- Linux: `x86_64`, `aarch64`
- macOS: `arm64` (and optionally `x86_64`)

This package uses `abi3` (`cp39-abi3`), so one wheel per platform/arch can
cover multiple Python versions (3.9+), but you still need separate wheels per
OS/architecture.

## Publish to PyPI

1. Update version in `bindings/python/pyproject.toml`.
2. Build wheels and (optionally) source dist.
3. Validate artifacts:

```bash
python -m twine check dist/*
```

4. Upload to TestPyPI first:

```bash
python -m twine upload --repository testpypi dist/*
```

5. Install from TestPyPI and run smoke test:

```bash
python -m pip install -i https://test.pypi.org/simple/ motorbridge-smart-servo
python -c "import motorbridge_smart_servo; print('ok')"
```

6. Upload to production PyPI:

```bash
python -m twine upload dist/*
```

Use API tokens for Twine auth (`__token__` username).

## Release Checklist

- `cargo test --workspace`
- `python -m compileall bindings/python/src examples/python`
- Import smoke:
  `python -c "import motorbridge_smart_servo as m; print(m.__name__)"`
- CLI smoke:
  `python -m motorbridge_smart_servo.cli --help`
- `twine check dist/*`

## Troubleshooting

- `externally-managed-environment` (PEP 668): use a virtual environment.
- `No such file or directory` when opening serial port: wrong `port` path.
- Permission denied on Linux serial device: add user to `dialout` and relogin.
- `library_path` argument is unsupported with PyO3 backend by design.
