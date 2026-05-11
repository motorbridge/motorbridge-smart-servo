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

The core filter (`AngleReliability`) suppresses the middle zero bridge and also
rejects out-of-range values that the firmware emits during startup (e.g.
`-23592960°` from the monitor command on power-on):

```text
raw:      -70 -> 0   -> 0   -> -55
filtered: -70 -> -70 -> -70 -> -55

raw:      5.0 -> -23592960 -> -23592960 -> 5.1
filtered: 5.0 ->       5.0 ->       5.0 -> 5.1
```

Two guard conditions are checked before accepting a raw value:

1. **Near-zero**: `|raw| <= zero_eps_deg` — hold last good, require the
   reading to remain near zero for `zero_confirm_duration_s` before accepting.
2. **Out of range**: `|raw| > valid_range_deg` — hold last good indefinitely
   until an in-range value arrives.

`valid_range_deg` defaults to `3,686,400°` (FashionStar ±1024-turn limit).
This covers both the angle-query path (code 16) and the monitor path (code 22),
which have different startup behavior at the firmware level.

Normal movement is not delayed:

```text
raw:      -70 -> -55 -> -20
filtered: -70 -> -55 -> -20
```

Repeated zero readings are treated as an intentional real zero after the
duration confirmation, so a servo commanded to zero is not held forever at the
previous non-zero angle.

### Zero Confirmation Timing

The reliability core expresses zero confirmation as wall-clock time, not as a
sample count, so behavior stays stable when the polling rate changes:

```text
confirmation_time = zero_confirm_duration_s
```

Current defaults:

- Core default: `zero_confirm_duration_s = 0.65`
- At a 100 Hz backend poll rate, this is about 60 near-zero samples.
- The WASM demos expose the same duration value as "Zero hold seconds".

## Sync Read (code 25)

The FashionStar sync command wraps any supported sub-command and delivers it to
multiple servos simultaneously. For reading, the sub-command is `22` (data
monitor): one request packet causes each online servo to reply independently
with voltage, current, power, temperature, status, angle, and turn count.

Latency comparison at 1 Mbaud with 7 servos:

| Mode | Per-cycle latency |
|---|---|
| Sequential angle read (code 16 × 7) | ~154 ms |
| Sync monitor (code 25 + sub 22) | ~4.4 ms observed on a 7-servo bus |

The Fashion Star protocol recommends a 5-10 ms command interval. With a
measured ~4.4 ms `sync_monitor` cycle and a 10 ms target polling period, a
polling loop can update at about 100 Hz while still leaving bus and OS jitter
room.

Current CLI and Python APIs are synchronous/on-demand: each `read_angle()` or
`sync_monitor()` call performs a serial transaction immediately. To run at a
stable 100 Hz, the caller should schedule calls on a 10 ms period and account for
the time spent in the serial transaction. A future background cache can use the
same 10 ms period and expose non-blocking `get_latest` style reads.

Partial responses (some servos offline) are handled gracefully: offline servos
are silently absent; their last known angle is held with `reliable = false`.
`idle_gap` is disabled for sync reads so that silence between individual servo
responses does not cause premature exit.

## Consecutive Loss Detection

`LossTracker` counts per-servo consecutive missed responses. The default
threshold is `0`, which disables hard-stop errors so monitoring can keep
returning held values with `reliable = false` during servo power loss. When an
application opts into a positive threshold and a servo exceeds it,
`SmartServoError::ConsecutiveLoss` is raised. A successful response from that
servo resets its counter to zero.

## Serial Bus Read Strategy

`SerialBus::read_until(idle_gap, done)` replaces the old fixed-timeout
`read_until_idle`. The `done` closure is called after every successful read;
as soon as it returns `true` (complete valid packet detected), the read exits.
The serial port's own `read_timeout` (10 ms) doubles as an idle-gap detector
for single-packet reads (`idle_gap = true`). Sync reads use `idle_gap = false`
to avoid exiting early when individual servo responses arrive with small gaps.
