# Usage

Version: `v0.0.4`

## Build

Windows PowerShell:

```powershell
cd C:\Users\tianr\Downloads\AMOTOR\fashionstar-uart-sdk-main\motorbridge-smart-servo
cargo build --release -p smart_servo_cli -p smart_servo_abi
```

Ubuntu/bash:

```bash
cd ~/motorbridge-smart-servo
cargo build --release -p smart_servo_cli -p smart_servo_abi
```

## Ubuntu Quickstart

Install build dependencies:

```bash
sudo apt-get update
sudo apt-get install -y build-essential python3-venv python3-pip
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
cargo run -p smart_servo_cli -- monitor --vendor fashionstar --port /dev/ttyUSB0 --baudrate 1000000 --id 0 --multi-turn --interval-ms 10
```

Python wheel on Ubuntu:

```bash
python3 -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip maturin twine
cd bindings/python
python -m maturin build --release --out dist
python -m twine check dist/*.whl
python -m pip install --force-reinstall dist/*.whl
```

Python CLI on Ubuntu:

```bash
motorbridge-smart-servo scan --vendor fashionstar --port /dev/ttyUSB0 --baudrate 1000000 --max-id 20
motorbridge-smart-servo read-angle --vendor fashionstar --port /dev/ttyUSB0 --baudrate 1000000 --id 0 --multi-turn
motorbridge-smart-servo monitor --vendor fashionstar --port /dev/ttyUSB0 --baudrate 1000000 --id 0 --multi-turn --interval-s 0.01
```

Install a GitHub Release wheel on Ubuntu:

```bash
python -m pip install ./motorbridge_smart_servo-0.0.4-cp39-abi3-manylinux2014_x86_64.whl
```

## Native CLI

Scan online servos:

Windows PowerShell:

```powershell
cargo run -p smart_servo_cli -- scan --port COM5 --baudrate 1000000 --max-id 20
```

Ubuntu/bash:

```bash
cargo run -p smart_servo_cli -- scan --port /dev/ttyUSB0 --baudrate 1000000 --max-id 20
```

The default vendor is `fashionstar`. You can also be explicit:

Windows PowerShell:

```powershell
cargo run -p smart_servo_cli -- scan --vendor fashionstar --port COM5 --baudrate 1000000 --max-id 20 --timeout-ms 30
```

Ubuntu/bash:

```bash
cargo run -p smart_servo_cli -- scan --vendor fashionstar --port /dev/ttyUSB0 --baudrate 1000000 --max-id 20 --timeout-ms 30
```

`--timeout-ms` controls per-ID scan timeout. Use a smaller value for faster full-bus scans.

Read one angle sample:

Windows PowerShell:

```powershell
cargo run -p smart_servo_cli -- read-angle --port COM5 --baudrate 1000000 --id 0 --multi-turn
```

Ubuntu/bash:

```bash
cargo run -p smart_servo_cli -- read-angle --port /dev/ttyUSB0 --baudrate 1000000 --id 0 --multi-turn
```

Read raw protocol angle only:

Windows PowerShell:

```powershell
cargo run -p smart_servo_cli -- read-angle --port COM5 --baudrate 1000000 --id 0 --multi-turn --raw
```

Ubuntu/bash:

```bash
cargo run -p smart_servo_cli -- read-angle --port /dev/ttyUSB0 --baudrate 1000000 --id 0 --multi-turn --raw
```

Monitor continuously:

Windows PowerShell:

```powershell
cargo run -p smart_servo_cli -- monitor --port COM5 --baudrate 1000000 --id 0 --multi-turn --interval-ms 10
```

Ubuntu/bash:

```bash
cargo run -p smart_servo_cli -- monitor --port /dev/ttyUSB0 --baudrate 1000000 --id 0 --multi-turn --interval-ms 10
```

Move a servo:

Windows PowerShell:

```powershell
cargo run -p smart_servo_cli -- set-angle --port COM5 --baudrate 1000000 --id 0 --angle -45 --interval-ms 500
```

Ubuntu/bash:

```bash
cargo run -p smart_servo_cli -- set-angle --port /dev/ttyUSB0 --baudrate 1000000 --id 0 --angle -45 --interval-ms 500
```

Sync read multiple servos in one request:

Windows PowerShell:

```powershell
cargo run -p smart_servo_cli -- sync-monitor --port COM5 --baudrate 1000000 --ids 0 1 2 3 4 5 6
cargo run -p smart_servo_cli -- sync-monitor --port COM5 --baudrate 1000000 --ids 0 1 2 3 4 5 6 --interval-ms 10
```

Ubuntu/bash:

```bash
cargo run -p smart_servo_cli -- sync-monitor --port /dev/ttyUSB0 --baudrate 1000000 --ids 0 1 2 3 4 5 6
cargo run -p smart_servo_cli -- sync-monitor --port /dev/ttyUSB0 --baudrate 1000000 --ids 0 1 2 3 4 5 6 --interval-ms 10
```

Each row shows `angle`, `voltage`, and `reliable` for every queried servo.
A servo that does not respond returns its last known angle with `reliable=false`.
By default, the consecutive loss hard-stop is disabled, so monitoring continues
through servo power loss. Set a positive loss threshold if your application wants
offline servos to become a fatal error.

Output meaning:

```text
raw=    0.000 filtered=  -93.900 reliable=false
```

`raw` is the protocol value. `filtered` is the business-safe value. `reliable=false`
means the filter is holding the last valid value because the raw value looks like
a power-cycle bridge (`A -> 0 -> B`).

If the servo is intentionally held at real zero, the filter confirms that raw
angle has stayed near zero for `0.65 s` and eventually releases
`filtered=0 reliable=true`.

The core default confirmation window is time-based:
`zero_confirm_duration_s = 0.65`. At the recommended 100 Hz backend polling rate
this is about 60 near-zero samples, but the decision is made by elapsed time
rather than sample count. See [ARCHITECTURE.md](ARCHITECTURE.md) for the design
note.

If the bus times out during monitoring after at least one valid sample, the CLI
continues and prints `reliable=false` with the last filtered angle.

Protocol checksum errors are reported as checksum mismatches instead of generic
timeouts. Invalid commands such as `NaN`, `Infinity`, or an out-of-range
single-turn `interval_ms` are rejected before sending.

## Python Environment

Create and use the local venv:

Windows PowerShell:

```powershell
cd C:\Users\tianr\Downloads\AMOTOR\fashionstar-uart-sdk-main\motorbridge-smart-servo
.\.venv\Scripts\Activate.ps1
```

Ubuntu/bash:

```bash
cd ~/motorbridge-smart-servo
python3 -m venv .venv
source .venv/bin/activate
```

Build and install the wheel:

Windows PowerShell:

```powershell
python -m pip install --upgrade maturin
Push-Location bindings\python
python -m maturin build --release --out dist
python -m pip install --force-reinstall (Get-ChildItem dist\*.whl | Select-Object -Last 1).FullName
Pop-Location
```

Ubuntu/bash:

```bash
python -m pip install --upgrade maturin
cd bindings/python
python -m maturin build --release --out dist
python -m pip install --force-reinstall dist/*.whl
cd ../..
```

## Python CLI

Scan:

Windows PowerShell:

```powershell
motorbridge-smart-servo scan --vendor fashionstar --port COM5 --baudrate 1000000 --max-id 20
```

Ubuntu/bash:

```bash
motorbridge-smart-servo scan --vendor fashionstar --port /dev/ttyUSB0 --baudrate 1000000 --max-id 20
```

Read one sample:

Windows PowerShell:

```powershell
motorbridge-smart-servo read-angle --port COM5 --baudrate 1000000 --id 0 --multi-turn
```

Ubuntu/bash:

```bash
motorbridge-smart-servo read-angle --port /dev/ttyUSB0 --baudrate 1000000 --id 0 --multi-turn
```

Monitor:

Windows PowerShell:

```powershell
motorbridge-smart-servo monitor --port COM5 --baudrate 1000000 --id 0 --multi-turn --interval-s 0.01
```

Ubuntu/bash:

```bash
motorbridge-smart-servo monitor --port /dev/ttyUSB0 --baudrate 1000000 --id 0 --multi-turn --interval-s 0.01
```

Move:

Windows PowerShell:

```powershell
motorbridge-smart-servo set-angle --port COM5 --baudrate 1000000 --id 0 --angle -45 --interval-ms 500
```

Ubuntu/bash:

```bash
motorbridge-smart-servo set-angle --port /dev/ttyUSB0 --baudrate 1000000 --id 0 --angle -45 --interval-ms 500
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
bus.set_stop_mode(0, mode=0x10, power=0)   # stop and release torque
bus.set_loss_threshold(20)                 # opt-in hard fault after 20 misses
```

Sync read — query N servos in one request:

```python
from motorbridge_smart_servo import SmartServoBus, ServoMonitor, ServoBusError

with SmartServoBus.open(vendor="fashionstar", port="COM5", baudrate=1_000_000) as bus:
    try:
        result: dict[int, ServoMonitor | None] = bus.sync_monitor([0, 1, 2, 3, 4, 5, 6])
    except ServoBusError as exc:
        # raised only after opting into a positive consecutive loss threshold
        print(f"fatal: {exc}")
    else:
        for sid, m in result.items():
            if m is None:
                print(f"id={sid}: no data (never responded)")
            elif not m.reliable:
                print(
                    f"id={sid}: held raw={m.raw_deg:.2f} "
                    f"filtered={m.filtered_deg:.2f} (unreliable)"
                )
            else:
                print(
                    f"id={sid}: raw={m.raw_deg:.2f} "
                    f"filtered={m.filtered_deg:.2f} volt={m.voltage_mv}mV"
                )
```

`ServoMonitor` fields:

| Field | Type | Description |
|---|---|---|
| `id` | `int` | Servo ID |
| `raw_deg` | `float` | Raw protocol angle before filtering |
| `filtered_deg` | `float` | Reliability-filtered angle for control logic |
| `angle_deg` | `float` | Backward-compatible alias of `filtered_deg` |
| `voltage_mv` | `int` | Voltage (mV) |
| `current_ma` | `int` | Current (mA) |
| `power_mw` | `int` | Power (mW) |
| `temp_raw` | `int` | Temperature ADC value |
| `status` | `int` | Status flags (bit0=busy, bit2=stall, bit3=high-V, bit4=low-V, …) |
| `turn` | `int` | Turn count |
| `reliable` | `bool` | `True` = fresh reading and angle passed filter; `False` = held or glitch |

Python polling frequency:

- The binding is synchronous/on-demand. `read_angle()` and `sync_monitor()` each
  perform a serial transaction when called.
- A measured 7-servo `sync_monitor([0..6])` cycle is about `4.4 ms`.
- Use a `10 ms` target period for about `100 Hz` effective updates.
- Calling faster than the transaction can finish only blocks longer and adds
  jitter; it does not create fresher data.

Consecutive loss threshold:

```python
# Raise ServoBusError after 20 consecutive missed responses
bus.set_loss_threshold(20)

# Disable the check (default)
bus.set_loss_threshold(0)
```

Monitor generator:

```python
with SmartServoBus.open(vendor="fashionstar", port="COM5") as bus:
    for sample in bus.monitor(0, multi_turn=True, interval_s=0.01):
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

## Examples

Python SDK examples live in `examples/python`:

Windows PowerShell:

```powershell
python examples\python\scan.py
python examples\python\read_angle.py
python examples\python\monitor.py
python examples\python\ping.py
python examples\python\set_angle.py
python examples\python\sync_monitor.py
```

Ubuntu/bash:

```bash
python examples/python/scan.py
python examples/python/read_angle.py
python examples/python/monitor.py
python examples/python/ping.py
python examples/python/set_angle.py
python examples/python/sync_monitor.py
```

WASM/browser examples live in `examples/wasm`.

## WASM Browser Binding

`smart_servo_wasm` is a `wasm-bindgen` binding for JavaScript and browsers. It
exposes FashionStar packet encoding/decoding and the angle reliability filter
(including out-of-range protection).

### Multi-servo monitor dashboard

Build and run:

Ubuntu/bash:

```bash
bash examples/wasm/browser-monitor-demo/build.sh
cd examples/wasm/browser-monitor-demo
python3 -m http.server 8080
```

Windows PowerShell:

```powershell
examples\wasm\browser-monitor-demo\build.ps1
cd examples\wasm\browser-monitor-demo
python -m http.server 8080
```

Open `http://localhost:8080` in Chrome or Edge, click `Connect WebSerial`.

Enter the servo IDs you want to monitor (space-separated, default `0 1 2 3 4 5 6`).
Each servo gets its own colour curve. Solid lines are `reliable`, dashed grey
lines indicate held values (servo offline or angle filter rejected). The right
sidebar shows live angle, voltage and reliability per servo.

### Single-servo filter demo

Visualises the `A -> 0 -> B` reliability filter for one servo.
Also works in simulation mode without hardware.

Ubuntu/bash:

```bash
bash examples/wasm/browser-filter-demo/build.sh
cd examples/wasm/browser-filter-demo
python3 -m http.server 8080
```

Windows PowerShell:

```powershell
examples\wasm\browser-filter-demo\build.ps1
cd examples\wasm\browser-filter-demo
python -m http.server 8080
```

Open `http://localhost:8080` in Chrome or Edge, then click `Connect WebSerial`.

JavaScript owns WebSerial I/O; WASM owns packet encode/decode and filtering.

## Platform Support

CI is configured to build:

- Windows x86_64 MSVC native CLI + ABI + Python wheel
- Linux x86_64 GNU native CLI + ABI + Python wheel
- Linux aarch64 GNU native CLI + ABI via `cross`, plus Python wheel via maturin manylinux2014
- macOS aarch64 native CLI + ABI + Python wheel
- WASM `wasm32-unknown-unknown` reliability core

WASM currently exposes FashionStar query/decode helpers and the angle reliability
filter. Browser hardware access is implemented in JavaScript through WebSerial;
WASM owns packet encode/decode and filtering.

## GitHub Release

Tag pushes automatically create/update a GitHub Release and upload artifacts:

- native CLI + ABI archives
- PyO3 abi3 Python wheels
- WASM package archive

Create a release tag:

```powershell
git tag v0.0.4
git push origin v0.0.4
```

The workflows are:

- `.github/workflows/build-native.yml`
- `.github/workflows/build-wheels.yml`

Both workflows upload assets to the same `v0.0.4` GitHub Release.
