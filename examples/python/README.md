# Python Examples

These scripts use the published PyPI package:

- Package name: `motorbridge-smart-servo`
- Import name: `motorbridge_smart_servo`

## Install Latest Package from PyPI

Windows PowerShell:

```powershell
python -m venv .venv
.\.venv\Scripts\Activate.ps1
python -m pip install -U pip
python -m pip install motorbridge-smart-servo
```

Linux/macOS (bash):

```bash
python3 -m venv .venv
source .venv/bin/activate
python -m pip install -U pip
python -m pip install motorbridge-smart-servo
```

## Run Examples

From repository root:

Windows PowerShell:

```powershell
python examples\python\scan.py
python examples\python\ping.py
python examples\python\read_angle.py
python examples\python\monitor.py
```

Linux/macOS (bash):

```bash
python examples/python/scan.py
python examples/python/ping.py
python examples/python/read_angle.py
python examples/python/monitor.py
```

`set_angle.py` is intentionally excluded from the default run list for now.
In the current release line, write-control is not considered stable/supported.

## Optional Advanced Examples

- `test_connection.py`: quick bus health check and basic read test
- `test_angle.py`: repeated sample test with raw/filtered stats
- `monitor_all_joints.py`: monitor multiple servo IDs in one loop
- `plot_all_joints.py`: realtime plotting (requires matplotlib and numpy)
- `set_angle.py`: write-control demo (currently experimental / not recommended)

If needed:

```bash
python -m pip install matplotlib numpy
```

## Port Configuration

Set `PORT` in each script before running:

- Windows: `COM5` (example)
- Linux: `/dev/ttyUSB0` or `/dev/ttyACM0`

Baudrate is typically `1_000_000` for FashionStar UART smart servos.
