# Usage

Version: `v0.0.1`

## Build

```powershell
cd C:\Users\tianr\Downloads\AMOTOR\fashionstar-uart-sdk-main\motorbridge-smart-servo
cargo build --release -p smart_servo_cli -p smart_servo_abi
```

## Ubuntu Quickstart

Install build dependencies:

```bash
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libudev-dev python3-venv python3-pip
```

Allow the current user to access USB serial ports, then log out and back in:

```bash
sudo usermod -aG dialout "$USER"
```

Build and test:

```bash
cd ~/motorbridge-smart-servo
cargo fmt --all -- --check
cargo test --workspace
cargo build --release -p smart_servo_cli -p smart_servo_abi
```

Find your serial port:

```bash
ls /dev/ttyUSB* /dev/ttyACM* 2>/dev/null
```

Native CLI on Ubuntu:

```bash
cargo run -p smart_servo_cli -- scan --vendor fashionstar --port /dev/ttyUSB0 --baudrate 1000000 --max-id 20 --timeout-ms 30
cargo run -p smart_servo_cli -- read-angle --vendor fashionstar --port /dev/ttyUSB0 --baudrate 1000000 --id 0 --multi-turn
cargo run -p smart_servo_cli -- monitor --vendor fashionstar --port /dev/ttyUSB0 --baudrate 1000000 --id 0 --multi-turn --interval-ms 20
```

Python wheel on Ubuntu:

```bash
python3 -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip build twine
python -m build --wheel bindings/python
python -m twine check bindings/python/dist/*.whl
python -m pip install --force-reinstall bindings/python/dist/*.whl
```

Python CLI on Ubuntu:

```bash
motorbridge-smart-servo scan --vendor fashionstar --port /dev/ttyUSB0 --baudrate 1000000 --max-id 20
motorbridge-smart-servo read-angle --vendor fashionstar --port /dev/ttyUSB0 --baudrate 1000000 --id 0 --multi-turn
motorbridge-smart-servo monitor --vendor fashionstar --port /dev/ttyUSB0 --baudrate 1000000 --id 0 --multi-turn --interval-s 0.02
```

Install a GitHub Release wheel on Ubuntu:

```bash
python -m pip install ./motorbridge_smart_servo-0.0.1-py3-none-linux_x86_64.whl
```

## Native CLI

Scan online servos:

```powershell
cargo run -p smart_servo_cli -- scan --port COM5 --baudrate 1000000 --max-id 20
```

The default vendor is `fashionstar`. You can also be explicit:

```powershell
cargo run -p smart_servo_cli -- scan --vendor fashionstar --port COM5 --baudrate 1000000 --max-id 20 --timeout-ms 30
```

`--timeout-ms` controls per-ID scan timeout. Use a smaller value for faster full-bus scans.

Read one angle sample:

```powershell
cargo run -p smart_servo_cli -- read-angle --port COM5 --baudrate 1000000 --id 0 --multi-turn
```

Read raw protocol angle only:

```powershell
cargo run -p smart_servo_cli -- read-angle --port COM5 --baudrate 1000000 --id 0 --multi-turn --raw
```

Monitor continuously:

```powershell
cargo run -p smart_servo_cli -- monitor --port COM5 --baudrate 1000000 --id 0 --multi-turn --interval-ms 20
```

Move a servo:

```powershell
cargo run -p smart_servo_cli -- set-angle --port COM5 --baudrate 1000000 --id 0 --angle -45 --interval-ms 500
```

Output meaning:

```text
raw=    0.000 filtered=  -93.900 reliable=false
```

`raw` is the protocol value. `filtered` is the business-safe value. `reliable=false`
means the filter is holding the last valid value because the raw value looks like
a power-cycle bridge (`A -> 0 -> B`).

If the servo is intentionally held at real zero, the filter confirms repeated
zero samples and eventually releases `filtered=0 reliable=true`.

If the bus times out during monitoring after at least one valid sample, the CLI
continues and prints `reliable=false` with the last filtered angle.

Protocol checksum errors are reported as checksum mismatches instead of generic
timeouts. Invalid commands such as `NaN`, `Infinity`, or an out-of-range
single-turn `interval_ms` are rejected before sending.

## Python Environment

Create and use the local venv:

```powershell
cd C:\Users\tianr\Downloads\AMOTOR\fashionstar-uart-sdk-main\motorbridge-smart-servo
.\.venv\Scripts\Activate.ps1
```

Build and install the wheel:

```powershell
cargo build -p smart_servo_abi
python -m build --wheel bindings\python
python -m pip install --force-reinstall (Get-ChildItem bindings\python\dist\*.whl | Select-Object -Last 1).FullName
```

## Python CLI

Scan:

```powershell
motorbridge-smart-servo scan --vendor fashionstar --port COM5 --baudrate 1000000 --max-id 20
```

Read one sample:

```powershell
motorbridge-smart-servo read-angle --port COM5 --baudrate 1000000 --id 0 --multi-turn
```

Monitor:

```powershell
motorbridge-smart-servo monitor --port COM5 --baudrate 1000000 --id 0 --multi-turn --interval-s 0.02
```

Move:

```powershell
motorbridge-smart-servo set-angle --port COM5 --baudrate 1000000 --id 0 --angle -45 --interval-ms 500
```

## Python API

```python
from motorbridge_smart_servo import SmartServoBus

with SmartServoBus.open(vendor="fashionstar", port="COM5", baudrate=1_000_000) as bus:
    ids = bus.scan(max_id=20)
    print(ids)

    sample = bus.read_angle(0, multi_turn=True)
    print(sample.raw_deg, sample.filtered_deg, sample.reliable)

    # Use this for control logic.
    angle = sample.filtered_deg
```

Convenience methods:

```python
bus.ping(0)
bus.scan(max_id=20)
bus.read_angle(0, multi_turn=True)
bus.read_raw_angle(0, multi_turn=True)
bus.read_filtered_angle(0, multi_turn=True)
bus.set_angle(0, -45.0, multi_turn=False, interval_ms=500)
```

Monitor generator:

```python
with SmartServoBus.open(vendor="fashionstar", port="COM5") as bus:
    for sample in bus.monitor(0, multi_turn=True, interval_s=0.02):
        print(sample)
```

`monitor()` keeps running through temporary communication loss after it has seen
one valid angle. In that case `sample.reliable` is `False` and
`sample.filtered_deg` remains the last safe value.

The old compatibility entry point remains available:

```python
from motorbridge_smart_servo import FashionStarServo

with FashionStarServo("COM5") as bus:
    sample = bus.read_angle(0, multi_turn=True)
```

## Platform Support

CI is configured to build:

- Windows x86_64 MSVC native CLI + ABI + Python wheel
- Linux x86_64 GNU native CLI + ABI + Python wheel
- Linux aarch64 GNU native CLI + ABI via `cross`
- macOS x86_64 native CLI + ABI + Python wheel
- macOS aarch64 native CLI + ABI + Python wheel
- WASM `wasm32-unknown-unknown` reliability core

WASM currently exposes the protocol-independent angle reliability filter. Direct
UART access from WASM requires a host transport such as WebSerial or a native
bridge, so hardware bus control remains native for `v0.0.1`.

## GitHub Release

Tag pushes automatically create/update a GitHub Release and upload artifacts:

- native CLI + ABI archives
- Python wheels
- WASM package archive

Create a release tag:

```powershell
git tag v0.0.1
git push origin v0.0.1
```

The workflows are:

- `.github/workflows/build-native.yml`
- `.github/workflows/build-wheels.yml`

Both workflows upload assets to the same `v0.0.1` GitHub Release.
