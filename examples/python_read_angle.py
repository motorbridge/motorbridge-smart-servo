"""Read a single angle sample from a servo."""

from motorbridge_smart_servo import SmartServoBus

PORT = "/dev/ttyUSB0"

with SmartServoBus.open(vendor="fashionstar", port=PORT) as bus:
    sample = bus.read_angle(0, multi_turn=True)
    print(sample)
    print(f"  raw:       {sample.raw_deg:.3f} deg")
    print(f"  filtered:  {sample.filtered_deg:.3f} deg")
    print(f"  reliable:  {sample.reliable}")

    # Convenience methods
    raw = bus.read_raw_angle(0, multi_turn=True)
    filtered = bus.read_filtered_angle(0, multi_turn=True)
    print(f"\nConvenience: raw={raw:.3f}, filtered={filtered:.3f}")
