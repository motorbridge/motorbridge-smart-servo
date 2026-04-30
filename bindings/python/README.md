# motorbridge-smart-servo Python binding

PyO3 + maturin Python binding for MotorBridge Smart Servo.

The binding exposes FashionStar UART smart-servo control with both protocol raw
angle and filtered angle. The filtered value suppresses the power-cycle
`A -> 0 -> B` glitch while allowing normal `A -> B` motion.

The Rust core is compiled directly into `motorbridge_smart_servo._native`, so
wheels are real platform wheels such as `cp39-abi3-win_amd64.whl` or
`cp39-abi3-manylinux2014_aarch64.whl`. There is no runtime `ctypes.CDLL(...)`
load path and no bundled external ABI DLL for Python.

## Install for development

```bash
cd bindings/python
python -m pip install -U maturin
python -m pip install -e .
```

## Build wheel

```bash
cd bindings/python
python -m maturin build --release --out dist
python -m pip install --force-reinstall dist/*.whl
```

On PowerShell:

```powershell
cd bindings\python
python -m maturin build --release --out dist
python -m pip install --force-reinstall (Get-ChildItem dist\*.whl | Select-Object -Last 1).FullName
```

## Usage

```python
from motorbridge_smart_servo import FashionStarServo

with FashionStarServo("COM5", 1_000_000) as bus:
    print(bus.scan(max_id=20))

    sample = bus.read_angle(0, multi_turn=True)
    print(sample.raw_deg, sample.filtered_deg, sample.reliable)

    # Business logic should usually use filtered_deg.
    angle = sample.filtered_deg
```

## Monitor

```python
with FashionStarServo("COM5") as bus:
    for sample in bus.monitor(0, multi_turn=True, interval_s=0.02):
        print(f"raw={sample.raw_deg:8.3f} filtered={sample.filtered_deg:8.3f} reliable={sample.reliable}")
```

During power loss or serial timeout, `monitor()` does not stop after at least one
valid angle has been observed. It yields `reliable=False` samples that hold the
last filtered angle.

## Move

```python
with FashionStarServo("COM5") as bus:
    bus.set_angle(0, -45.0, multi_turn=False, interval_ms=500)
```
