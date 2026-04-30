# Python examples

These examples use the PyO3 Python package (`motorbridge-smart-servo`) to talk to a FashionStar UART smart-servo bus.

Install the package first:

```powershell
cd C:\Users\tianr\Downloads\AMOTOR\fashionstar-uart-sdk-main\motorbridge-smart-servo
.\.venv\Scripts\Activate.ps1
python -m pip install --force-reinstall bindings\python\dist\motorbridge_smart_servo-0.0.2-cp39-abi3-win_amd64.whl
```

Run examples:

```powershell
python examples\python\scan.py
python examples\python\read_angle.py
python examples\python\monitor.py
python examples\python\ping.py
python examples\python\set_angle.py
```

Edit `PORT` inside each file for your system, for example `COM5` on Windows or `/dev/ttyUSB0` on Linux.
