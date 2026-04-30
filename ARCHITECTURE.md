# Architecture

`motorbridge-smart-servo` keeps hardware transport, vendor protocol, ABI, CLI,
and language bindings separated.

## Layers

- `smart_servo_core`
  - Serial bus abstraction
  - Shared errors and model types
  - `SmartServoController` trait
  - Angle reliability filter for power-cycle `A -> 0 -> B` glitches
- `smart_servo_vendors/fashionstar`
  - FashionStar packet framing and checksum
  - FashionStar command encoder/decoder
  - Controller implementation
- `smart_servo_abi`
  - Stable C ABI for native callers
  - Opaque handle ownership
  - Raw/filtered angle sample struct
- `smart_servo_cli`
  - Native scan/read/monitor/set commands
- `smart_servo_py`
  - PyO3 extension crate
  - Links the Rust core directly into the Python module
  - Releases the GIL while serial bus operations block
- `bindings/python`
  - maturin package metadata, Python CLI, and stable Python wrapper API

The C ABI remains available for native consumers, but Python does not use it.
Python wheels are native platform wheels built from `smart_servo_py`, avoiding a
runtime `ctypes` dependency and avoiding incorrect pure-Python wheel tags.

## Vendor Protocol Boundary

Vendor crates own protocol framing and command-specific payloads. Core code only
knows servo IDs, angles, samples, and controller behavior.

To add a new vendor:

1. Add `smart_servo_vendors/<vendor>`.
2. Implement packet encode/decode in `protocol.rs`.
3. Implement `SmartServoController`.
4. Export it through ABI/CLI only where needed.

## Angle Reliability

FashionStar power cycling can temporarily report `0 deg` between two real
non-zero positions:

```text
A -> 0 -> B
```

The core filter suppresses only the middle zero bridge:

```text
raw:      -70 -> 0   -> 0   -> -55
filtered: -70 -> -70 -> -70 -> -55
```

Normal movement is not delayed:

```text
raw:      -70 -> -55 -> -20
filtered: -70 -> -55 -> -20
```

Repeated zero samples are treated as an intentional real zero after confirmation,
so a servo commanded to zero is not held forever at the previous non-zero angle.
