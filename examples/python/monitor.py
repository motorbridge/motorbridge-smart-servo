"""Continuously monitor a servo angle at ~100 Hz."""

from motorbridge_smart_servo import SmartServoBus

PORT = "/dev/ttyUSB0"

with SmartServoBus.open(vendor="fashionstar", port=PORT) as bus:
    for sample in bus.monitor(0, multi_turn=True, interval_s=0.01):
        print(
            f"raw={sample.raw_deg:9.3f} "
            f"filtered={sample.filtered_deg:9.3f} "
            f"reliable={sample.reliable}"
        )
