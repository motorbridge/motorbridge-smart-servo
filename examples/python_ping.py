"""Ping individual servos to check which IDs are online."""

from motorbridge_smart_servo import SmartServoBus

PORT = "/dev/ttyUSB0"

with SmartServoBus.open(vendor="fashionstar", port=PORT) as bus:
    for servo_id in range(10):
        online = bus.ping(servo_id)
        print(f"ID {servo_id}: {'online' if online else 'offline'}")
