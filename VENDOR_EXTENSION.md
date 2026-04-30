# Vendor Extension Guide

The public API must remain stable as new smart-servo brands are added.

Stable user-facing operations:

- `ping`
- `scan`
- `read_angle`
- `read_raw_angle`
- `read_filtered_angle`
- `monitor`
- `set_angle` / `move_to`

Python stable entry point:

```python
from motorbridge_smart_servo import SmartServoBus

bus = SmartServoBus.open(vendor="fashionstar", port="COM5")
```

The old vendor-specific entry point remains supported:

```python
from motorbridge_smart_servo import FashionStarServo

bus = FashionStarServo("COM5")
```

## Adding a Vendor

1. Add a Rust crate under `smart_servo_vendors/<vendor>`.
2. Implement protocol framing/encoding/decoding in that crate.
3. Implement `SmartServoController` for the vendor controller.
4. Add a dispatch branch in:
   - `smart_servo_abi::mbss_open`
   - `smart_servo_cli::open_*`
   - `motorbridge_smart_servo.bus.SmartServoBus.open`
5. Keep Python method names and `AngleSample` unchanged.

## Compatibility Rule

Adding a new vendor must not change existing FashionStar behavior or signatures.
Existing code using `FashionStarServo` or `SmartServoBus.open(vendor="fashionstar")`
must continue to run unchanged.

