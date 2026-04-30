from motorbridge_smart_servo import FashionStarServo


with FashionStarServo("COM5", 1_000_000) as bus:
    sample = bus.read_angle(0, multi_turn=True)
    print(sample)

