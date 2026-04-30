# motorbridge-smart-servo Python binding

Python wrapper for the MotorBridge Smart Servo native ABI.

The binding exposes FashionStar UART smart-servo control with both protocol raw
angle and filtered angle. The filtered value suppresses the power-cycle
`A -> 0 -> B` glitch while allowing normal `A -> B` motion.

## Build native ABI

From the project root:

```bash
cargo build -p smart_servo_abi
```

The Python package automatically searches `target/debug` and `target/release`.
You can also set an explicit path:

```powershell
$env:MOTORBRIDGE_SMART_SERVO_LIB="C:\path\to\smart_servo_abi.dll"
```

## Install for development

```bash
cd bindings/python
python -m pip install -e .
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
