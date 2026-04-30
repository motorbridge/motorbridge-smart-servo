from motorbridge_smart_servo import FashionStarServo


with FashionStarServo("COM5", 1_000_000) as bus:
    print(bus.scan(max_id=20))

