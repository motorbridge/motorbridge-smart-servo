"""Query all servos in one sync command (code 25) and print their angles.

Sync monitor sends a single request and each online servo replies
independently, making it significantly faster than sequential reads for
multi-servo setups.

Usage:
    python examples/python/sync_monitor.py
"""

import time

from motorbridge_smart_servo import ServoMonitor, SmartServoBus, ServoBusError

PORT = "/dev/cu.wchusbserial11230"
BAUDRATE = 1_000_000
SERVO_IDS = [0, 1, 2, 3, 4, 5, 6]
INTERVAL_S = 0.01  # ~100 Hz

with SmartServoBus.open(vendor="fashionstar", port=PORT, baudrate=BAUDRATE) as bus:
    print(f"Polling {len(SERVO_IDS)} servos via sync_monitor at {1/INTERVAL_S:.0f} Hz")
    print(f"{'ID':>3}  {'angle':>9}  {'volt':>7}  {'reliable':>8}")
    print("-" * 38)

    while True:
        t0 = time.monotonic()

        try:
            result: dict[int, ServoMonitor | None] = bus.sync_monitor(SERVO_IDS)
        except ServoBusError as exc:
            print(f"fatal: {exc}")
            continue

        elapsed_ms = (time.monotonic() - t0) * 1000

        for sid in SERVO_IDS:
            m = result.get(sid)
            if m is None:
                print(f"{sid:>3}  {'no data':>9}")
            else:
                flag = "ok" if m.reliable else "~"
                print(
                    f"{sid:>3}  raw={m.raw_deg:>8.2f}°  "
                    f"filtered={m.filtered_deg:>8.2f}°  "
                    f"angle={m.angle_deg:>8.2f}°  "
                    f"{m.voltage_mv / 1000:>6.2f}V  "
                    f"{flag:>8}"
                )

        print(f"     --- {elapsed_ms:.1f} ms for {len(SERVO_IDS)} servos ---")
        print()

        sleep = INTERVAL_S - (time.monotonic() - t0)
        if sleep > 0:
            time.sleep(sleep)
