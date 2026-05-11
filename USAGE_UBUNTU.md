# Ubuntu 完整使用说明

motorbridge-smart-servo `v0.0.2` — FashionStar UART 智能舵机控制库

v0.0.2 将 Python 绑定从 ctypes 迁移到了 PyO3 + maturin abi3 方案。Rust 核心直接编译进 Python 扩展模块，不再需要外部 `libsmart_servo_abi.so`。

---

## 目录

1. [系统要求](#1-系统要求)
2. [安装方法](#2-安装方法)
3. [串口权限配置](#3-串口权限配置)
4. [确认串口设备](#4-确认串口设备)
5. [CLI 命令行工具](#5-cli-命令行工具)
6. [Python API](#6-python-api)
7. [Python 示例脚本](#7-python-示例脚本)
8. [输出含义说明](#8-输出含义说明)
9. [故障排查](#9-故障排查)
10. [项目架构](#10-项目架构)

---

## 1. 系统要求

| 项目 | 要求 |
|------|------|
| 操作系统 | Ubuntu 20.04+ x86_64 或 aarch64 |
| Python | 3.9+ |
| 硬件 | USB 转 TTL 串口模块（如 CH340、CP2102、FT232） |
| 舵机 | FashionStar UART 总线舵机 + 独立电源（6-12V） |

## 2. 安装方法

### 方法一：下载预编译 Wheel（推荐，无需编译）

从 GitHub Release 下载对应平台的 abi3 wheel：

```bash
# 创建虚拟环境
python3 -m venv .venv
source .venv/bin/activate

# 升级 pip
python -m pip install --upgrade pip

# 下载并安装 wheel（x86_64 示例）
wget https://github.com/motorbridge/motorbridge-smart-servo/releases/download/v0.0.2/motorbridge_smart_servo-0.0.2-cp39-abi3-manylinux_2_17_x86_64.manylinux2014_x86_64.whl


python -m pip install motorbridge_smart_servo-0.0.2-cp39-abi3-manylinux_2_17_x86_64.manylinux2014_x86_64.whl
```

如果是 aarch64（如树莓派 4/5）：

```bash
wget https://github.com/motorbridge/motorbridge-smart-servo/releases/download/v0.0.2/motorbridge_smart_servo-0.0.2-cp39-abi3-manylinux_2_17_aarch64.manylinux2014_aarch64.whl
python -m pip install motorbridge_smart_servo-0.0.2-cp39-abi3-manylinux_2_17_aarch64.manylinux2014_aarch64.whl
```

abi3 wheel 兼容 Python 3.9 及以上所有版本，无需针对每个 Python 版本单独编译。

验证安装：

```bash
python -c "from motorbridge_smart_servo import SmartServoBus; print('OK')"
```

### 方法二：从源码编译（需要 Rust 工具链）

```bash
# 安装编译依赖
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libudev-dev python3-venv python3-pip

# 安装 Rust（如果没有）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# 克隆仓库
git clone https://github.com/motorbridge/motorbridge-smart-servo.git
cd motorbridge-smart-servo

# 运行测试
cargo test --workspace

# 创建虚拟环境并编译安装 Python wheel
python3 -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip maturin twine

# 用 maturin 编译并安装（Rust 核心自动编译进扩展模块）
cd bindings/python
python -m maturin build --release --out dist
python -m twine check dist/*.whl
python -m pip install --force-reinstall dist/*.whl
```

也可以用 `maturin develop` 直接开发安装（无需生成 wheel 文件）：

```bash
cd bindings/python
python -m maturin develop --release
```

### 编译 Rust 原生 CLI + ABI（可选）

如果需要独立的 Rust 命令行工具或 C ABI 动态库：

```bash
cargo build --release -p smart_servo_cli -p smart_servo_abi
# 产物在 target/release/ 下：
#   smart-servo              (CLI)
#   libsmart_servo_abi.so    (C ABI)
```

## 3. 串口权限配置

将当前用户加入 `dialout` 组以获得串口访问权限：

```bash
sudo usermod -aG dialout "$USER"
```

**必须注销并重新登录**才能生效。验证：

```bash
groups
# 输出中应包含 dialout
```

## 4. 确认串口设备

插入 USB 转串口模块后：

```bash
ls /dev/ttyUSB* /dev/ttyACM* 2>/dev/null
```

常见输出：

- `/dev/ttyUSB0` — CH340、CP2102 等芯片
- `/dev/ttyACM0` — 某些 Arduino 或 STM32 板

后续所有命令中的 `--port` 参数使用此路径。

检查串口是否可访问：

```bash
ls -l /dev/ttyUSB0
# crw-rw---- 1 root dialout ... /dev/ttyUSB0
```

## 5. CLI 命令行工具

以下所有命令需要先激活虚拟环境：

```bash
source .venv/bin/activate
```

### 5.1 扫描总线上的舵机

```bash
motorbridge-smart-servo scan --vendor fashionstar --port /dev/ttyUSB0 --baudrate 1000000 --max-id 20
```

参数说明：

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `--vendor` | `fashionstar` | 厂商，目前仅支持 `fashionstar` |
| `--port` | 必填 | 串口设备路径 |
| `--baudrate` | `1000000` | 波特率 |
| `--max-id` | `20` | 扫描最大 ID（范围 0 到 max-id） |

输出为每行一个在线舵机 ID：

```
0
1
3
```

### 5.2 读取角度（单次）

```bash
motorbridge-smart-servo read-angle --port /dev/ttyUSB0 --baudrate 1000000 --id 0 --multi-turn
```

参数说明：

| 参数 | 说明 |
|------|------|
| `--id` | 舵机 ID（0-253） |
| `--multi-turn` | 多圈模式 |

输出：

```
raw=  -70.000 filtered=  -70.000 reliable=true
```

### 5.3 持续监控

```bash
motorbridge-smart-servo monitor --port /dev/ttyUSB0 --baudrate 1000000 --id 0 --multi-turn --interval-s 0.01
```

参数说明：

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `--interval-s` | `0.01` | 采样间隔（秒），即 100Hz |

`Ctrl+C` 停止。

输出（每行一个采样）：

```
raw=  -70.000 filtered=  -70.000 reliable=true
raw=    0.000 filtered=  -70.000 reliable=false
raw=  -55.000 filtered=  -55.000 reliable=true
```

### 5.4 控制舵机转动

> **[未测试] 此命令会驱动舵机物理运动，操作不当可能导致机械损坏。请确认舵机安装安全、运动范围无遮挡后再使用。**

```bash
motorbridge-smart-servo set-angle --port /dev/ttyUSB0 --baudrate 1000000 --id 0 --angle -45 --interval-ms 500
```

参数说明：

| 参数 | 说明 |
|------|------|
| `--angle` | 目标角度（度） |
| `--multi-turn` | 多圈模式（默认单圈） |
| `--interval-ms` | 运动时间（毫秒），0 为最快 |

### 5.5 Rust 原生 CLI（可选）

如果编译了 `smart_servo_cli`，可直接运行 Rust 原生 CLI：

```bash
cargo run -p smart_servo_cli -- scan --vendor fashionstar --port /dev/ttyUSB0 --baudrate 1000000 --max-id 20 --timeout-ms 30
cargo run -p smart_servo_cli -- read-angle --vendor fashionstar --port /dev/ttyUSB0 --baudrate 1000000 --id 0 --multi-turn
cargo run -p smart_servo_cli -- monitor --vendor fashionstar --port /dev/ttyUSB0 --baudrate 1000000 --id 0 --multi-turn --interval-ms 10
cargo run -p smart_servo_cli -- set-angle --vendor fashionstar --port /dev/ttyUSB0 --baudrate 1000000 --id 0 --angle -45 --interval-ms 500
```

原生 CLI 支持 `--timeout-ms` 参数控制单次扫描超时，适合快速全总线扫描。

## 6. Python API

### 6.1 打开总线

```python
from motorbridge_smart_servo import SmartServoBus

bus = SmartServoBus.open(vendor="fashionstar", port="/dev/ttyUSB0", baudrate=1_000_000)
bus.close()
```

推荐使用上下文管理器自动关闭：

```python
with SmartServoBus.open(vendor="fashionstar", port="/dev/ttyUSB0") as bus:
    ...
```

### 6.2 扫描舵机

```python
with SmartServoBus.open(vendor="fashionstar", port="/dev/ttyUSB0") as bus:
    ids = bus.scan(max_id=20)
    print("在线舵机:", ids)
```

### 6.3 Ping 单个舵机

```python
online = bus.ping(0)  # 返回 True/False
```

### 6.4 读取角度

```python
sample = bus.read_angle(0, multi_turn=True)
print(sample.raw_deg)       # 原始协议角度
print(sample.filtered_deg)  # 滤波后的安全角度（控制逻辑用这个）
print(sample.reliable)      # True/False
```

便捷方法：

```python
raw = bus.read_raw_angle(0, multi_turn=True)            # 仅原始角度（float）
filtered = bus.read_filtered_angle(0, multi_turn=True)  # 仅滤波角度（float）
```

### 6.5 持续监控

```python
with SmartServoBus.open(vendor="fashionstar", port="/dev/ttyUSB0") as bus:
    for sample in bus.monitor(0, multi_turn=True, interval_s=0.01):
        print(f"raw={sample.raw_deg:9.3f} filtered={sample.filtered_deg:9.3f} reliable={sample.reliable}")
```

限制采样数量：

```python
for sample in bus.monitor(0, multi_turn=True, interval_s=0.01, count=100):
    ...
```

### 6.6 控制转动

> **[未测试] 此方法会驱动舵机物理运动，操作不当可能导致机械损坏。请确认舵机安装安全、运动范围无遮挡后再使用。**

```python
bus.set_angle(0, -45.0, multi_turn=False, interval_ms=500)
# 或者用别名
bus.move_to(0, -45.0, multi_turn=False, interval_ms=500)
```

### 6.7 兼容旧接口

```python
from motorbridge_smart_servo import FashionStarServo

with FashionStarServo("/dev/ttyUSB0", 1_000_000) as bus:
    sample = bus.read_angle(0, multi_turn=True)
```

注意：v0.0.2 的 `FashionStarServo` 不再接受 `library_path` 参数（PyO3 后端不需要外部动态库）。

### 6.8 AngleSample 字段

| 字段 | 类型 | 说明 |
|------|------|------|
| `raw_deg` | `float` | 原始协议角度值 |
| `filtered_deg` | `float` | 滤波后的安全角度值 |
| `reliable` | `bool` | `True` = 数据可信；`False` = 滤波器在保持上一有效值 |

### 6.9 Python 调用频率

当前 Python binding 是同步/按需调用模式：

- `read_angle(...)`：调用一次，就执行一次单舵机串口查询。
- `sync_monitor([...])`：调用一次，就执行一次多舵机同步查询。
- `monitor(..., interval_s=0.01)`：内部循环调用 `read_angle`，输出样本后再 sleep。

当前 7 个舵机 `sync_monitor([0..6])` 实测一轮约 `4.4ms`。建议上层按
`10ms` 周期调度，也就是约 `100Hz` 有效更新。调用频率高于总线事务完成
速度只会增加阻塞和抖动，不会产生更新鲜的数据。当前 Python binding 还
没有后台缓存线程；如果后续加入后台 cache，也应由底层固定 `10ms` 周期
采样，上层只读最近样本。

### 6.10 异常类型

| 异常 | 说明 |
|------|------|
| `SmartServoError` | 基础异常 |
| `LibraryLoadError` | 保留兼容，PyO3 后端不会抛出此异常 |
| `ServoBusError` | 总线操作失败（串口错误、超时、校验错误等） |

## 7. Python 示例脚本

所有示例位于 `examples/` 目录，统一使用 `/dev/ttyUSB0` 端口。按需修改。

### 7.1 扫描总线（`examples/python/scan.py`）

```bash
python examples/python/scan.py
```

### 7.2 Ping 单个舵机（`examples/python/ping.py`）

逐个 ping ID 0-9，显示在线状态：

```bash
python examples/python/ping.py
```

### 7.3 读取角度（`examples/python/read_angle.py`）

读取单次角度，展示 `raw_deg` / `filtered_deg` / `reliable` 及便捷方法：

```bash
python examples/python/read_angle.py
```

### 7.4 持续监控（`examples/python/monitor.py`）

以 100Hz 持续采样，`Ctrl+C` 停止：

```bash
python examples/python/monitor.py
```

### 7.5 控制转动（`examples/python/set_angle.py`）

> **[未测试] 此脚本会驱动舵机物理运动，操作不当可能导致机械损坏。请确认舵机安装安全、运动范围无遮挡后再使用。**

```bash
python examples/python/set_angle.py
```

## 8. 输出含义说明

### 角度字段

- **raw_deg** — 从舵机协议直接读取的原始角度值，可能包含掉电瞬间的毛刺（突然跳到 0 再跳回）
- **filtered_deg** — 经过可靠性滤波后的角度值，用于实际控制逻辑
- **reliable** — `True` 表示当前采样可信；`False` 表示滤波器检测到疑似掉电毛刺，正在保持上一个有效角度

### 滤波行为

FashionStar 舵机掉电瞬间会产生 `A → 0 → B` 的跳变（中间经过一个假 0 值）。滤波器自动抑制这种情况：

```
原始值:   -70 →  0  →  0  → -55
滤波值:   -70 → -70 → -70 → -55
可信度:    T  →  F  →  F  →  T
```

正常运动不受影响：

```
原始值:   -70 → -55 → -20
滤波值:   -70 → -55 → -20
可信度:    T  →  T  →  T
```

如果舵机被命令转到真实的 0°，滤波器在确认接近 0 的状态持续 `0.65s` 后会释放：

```
原始值:   -70 →  0  →  0  →  0  →  0
滤波值:   -70 → -70 →  0  →  0  →  0
可信度:    T  →  F  →  T  →  T  →  T
```

当前 core 默认使用时间语义确认真实 0：`zero_confirm_duration_s = 0.65`。
在推荐的底层 `100Hz` 采样下约等于 60 个接近 0 的样本，但判断依据是
经过的时间，不会因为采样频率变化而改变确认窗口。WASM WebSerial demo
也通过 `Zero hold seconds` 配置同一个持续时间。

### 通信中断

监控模式下如果总线超时，但之前已收到过至少一个有效采样，CLI/Python 会继续运行，输出 `reliable=false` 并保持最后的滤波角度。

## 9. 故障排查

### 扫描不到舵机

逐步排查：

```bash
# 1. 确认串口存在
ls -l /dev/ttyUSB0

# 2. 确认权限
groups  # 输出中应包含 dialout

# 3. 确认舵机供电（需要独立 6-12V 电源）

# 4. 尝试不同波特率
motorbridge-smart-servo scan --port /dev/ttyUSB0 --baudrate 115200 --max-id 20
motorbridge-smart-servo scan --port /dev/ttyUSB0 --baudrate 1000000 --max-id 20

# 5. 扩大扫描范围
motorbridge-smart-servo scan --port /dev/ttyUSB0 --baudrate 1000000 --max-id 253

# 6. 单独 ping 测试
python -c "
from motorbridge_smart_servo import SmartServoBus
with SmartServoBus.open(vendor='fashionstar', port='/dev/ttyUSB0') as bus:
    for i in range(10):
        print(f'ID {i}: {bus.ping(i)}')
"
```

### ServoBusError（串口打开失败）

```bash
# 检查串口设备是否存在
ls -l /dev/ttyUSB0

# 检查是否有权限
groups  # 需要 dialout 组

# 检查串口是否被其他程序占用
sudo lsof /dev/ttyUSB0
sudo fuser /dev/ttyUSB0
```

### Permission denied 访问串口

```bash
# 方法一：加入 dialout 组（推荐，永久生效）
sudo usermod -aG dialout "$USER"
# 然后注销重新登录

# 方法二：临时权限（重启后失效）
sudo chmod 666 /dev/ttyUSB0
```

### 串口被占用

```bash
# 查看谁在使用串口
sudo lsof /dev/ttyUSB0
sudo fuser /dev/ttyUSB0

# 杀掉占用进程
sudo fuser -k /dev/ttyUSB0
```

### wheel 安装后 import 失败

```bash
# 确认安装的版本
python -m pip show motorbridge-smart-servo

# 确认 Python 版本 >= 3.9
python --version

# 重新安装
python -m pip install --force-reinstall motorbridge-smart-servo
```

## 10. 项目架构

v0.0.2 架构（PyO3 替代 ctypes）：

```
motorbridge-smart-servo/
├── smart_servo_core/           # 核心抽象层
│   └── src/
│       ├── bus.rs              # 串口总线抽象
│       ├── controller.rs       # SmartServoController trait
│       ├── model.rs            # AngleSample 等数据模型
│       ├── reliability.rs      # 角度可靠性滤波器
│       └── error.rs            # 错误类型
├── smart_servo_vendors/
│   └── fashionstar/            # FashionStar 协议实现
│       └── src/
│           ├── protocol.rs     # 帧封装、校验
│           ├── controller.rs   # FashionStar 控制器
│           └── lib.rs
├── smart_servo_abi/            # 稳定 C ABI（编译为 .so，供非 Python 调用）
│   └── src/lib.rs
├── smart_servo_cli/            # Rust 原生 CLI
│   └── src/main.rs
├── smart_servo_py/             # PyO3 原生扩展 crate
│   └── src/lib.rs              # 直接编译进 Python wheel，不经过 ctypes
├── smart_servo_wasm/           # WASM 可靠性滤波核心
│   └── src/lib.rs
├── bindings/
│   └── python/                 # Python 包
│       ├── pyproject.toml      # maturin 构建配置（abi3-py39）
│       └── src/motorbridge_smart_servo/
│           ├── __init__.py     # 导出 SmartServoBus, FashionStarServo, AngleSample 等
│           ├── bus.py          # SmartServoBus 工厂入口
│           ├── fashionstar.py  # FashionStarServo 封装（调用 _native 模块）
│           ├── cli.py          # Python CLI（motorbridge-smart-servo 命令）
│           ├── py.typed        # PEP 561 类型标记
│           └── native/.gitkeep # 不再存放 .so，仅保留目录
└── examples/
    ├── python/                 # Python SDK 示例
    │   ├── scan.py             # 扫描总线上的在线舵机
    │   ├── ping.py             # Ping 单个舵机检查在线状态
    │   ├── read_angle.py       # 读取单次角度（含便捷方法）
    │   ├── monitor.py          # 持续监控（100Hz）
    │   └── set_angle.py        # 控制舵机转动（注意安全警告）
    └── wasm/                   # 浏览器 WebSerial + WASM 示例
```

关键变化（v0.0.1 → v0.0.2）：

- Python 绑定从 ctypes 迁移到 PyO3，Rust 核心编译为 `motorbridge_smart_servo._native` 扩展模块
- Wheel 格式从 `py3-none-linux` 变为 `cp39-abi3-manylinux2014`（兼容 Python 3.9+）
- 不再需要外部 `libsmart_servo_abi.so`，不再需要 `MOTORBRIDGE_SMART_SERVO_LIB` 环境变量
- `smart_servo_abi` 仍可独立编译，供 C/C 等非 Python 场景使用
