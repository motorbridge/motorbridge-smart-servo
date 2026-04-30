"""Move a servo to a target angle and read back the result.

WARNING: This drives physical motion. Make sure the servo is mounted safely
and the motion range is clear before running.
"""

import time

from motorbridge_smart_servo import SmartServoBus

PORT = "/dev/ttyUSB0"
SERVO_ID = 0
TARGET_ANGLE = -45.0

with SmartServoBus.open(vendor="fashionstar", port=PORT) as bus:
    print(f"Moving servo {SERVO_ID} to {TARGET_ANGLE} deg ...")
    bus.set_angle(SERVO_ID, TARGET_ANGLE, multi_turn=False, interval_ms=500)

    time.sleep(1)

    sample = bus.read_angle(SERVO_ID, multi_turn=False)
    print(f"Current angle: {sample.filtered_deg:.3f} deg (reliable={sample.reliable})")
