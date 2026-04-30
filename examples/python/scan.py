"""Scan the bus for online servos and list their IDs."""

from motorbridge_smart_servo import SmartServoBus

PORT = "/dev/ttyUSB0"
BAUDRATE = 1_000_000

with SmartServoBus.open(vendor="fashionstar", port=PORT, baudrate=BAUDRATE) as bus:
    ids = bus.scan(max_id=20)
    if ids:
        print(f"Found {len(ids)} servo(s): {ids}")
    else:
        print("No servos found. Check wiring and power supply.")
