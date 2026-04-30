from motorbridge_smart_servo import FashionStarServo


with FashionStarServo("COM5", 1_000_000) as bus:
    for sample in bus.monitor(0, multi_turn=True, interval_s=0.02):
        print(
            f"raw={sample.raw_deg:9.3f} "
            f"filtered={sample.filtered_deg:9.3f} "
            f"reliable={sample.reliable}"
        )

