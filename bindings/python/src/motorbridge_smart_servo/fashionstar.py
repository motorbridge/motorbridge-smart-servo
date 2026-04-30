from __future__ import annotations

import ctypes
import os
import time
from dataclasses import dataclass
from typing import Iterator, Optional
from pathlib import Path


class SmartServoError(RuntimeError):
    """Base exception raised by motorbridge-smart-servo."""


class LibraryLoadError(SmartServoError):
    """Raised when the native ABI library cannot be found or loaded."""


class ServoBusError(SmartServoError):
    """Raised when the native bus operation fails."""


class _AngleSample(ctypes.Structure):
    _fields_ = [
        ("raw_deg", ctypes.c_float),
        ("filtered_deg", ctypes.c_float),
        ("reliable", ctypes.c_bool),
    ]


@dataclass(frozen=True)
class AngleSample:
    """A raw/filtered angle sample returned by the native core.

    `raw_deg` is the protocol value read from the servo.
    `filtered_deg` is the value after suppressing power-cycle A->0->B glitches.
    `reliable=False` means `filtered_deg` is being held from the last good value.
    """

    raw_deg: float
    filtered_deg: float
    reliable: bool


def _configure_library(lib: ctypes.CDLL) -> ctypes.CDLL:
    lib.mbss_open.argtypes = [ctypes.c_char_p, ctypes.c_char_p, ctypes.c_uint32]
    lib.mbss_open.restype = ctypes.c_void_p
    lib.mbss_fashionstar_open.argtypes = [ctypes.c_char_p, ctypes.c_uint32]
    lib.mbss_fashionstar_open.restype = ctypes.c_void_p
    lib.mbss_close.argtypes = [ctypes.c_void_p]
    lib.mbss_close_handle.argtypes = [ctypes.POINTER(ctypes.c_void_p)]
    lib.mbss_ping.argtypes = [ctypes.c_void_p, ctypes.c_uint8]
    lib.mbss_ping.restype = ctypes.c_int
    lib.mbss_read_angle.argtypes = [
        ctypes.c_void_p,
        ctypes.c_uint8,
        ctypes.c_bool,
        ctypes.POINTER(_AngleSample),
    ]
    lib.mbss_read_angle.restype = ctypes.c_int
    lib.mbss_set_angle.argtypes = [
        ctypes.c_void_p,
        ctypes.c_uint8,
        ctypes.c_float,
        ctypes.c_bool,
        ctypes.c_uint32,
    ]
    lib.mbss_set_angle.restype = ctypes.c_int
    return lib


def _candidate_libraries(explicit_path: Optional[str] = None) -> list[Path]:
    explicit = explicit_path or os.environ.get("MOTORBRIDGE_SMART_SERVO_LIB")
    candidates = []
    if explicit:
        candidates.append(Path(explicit))

    here = Path(__file__).resolve()
    package_native = here.parent / "native"
    root = here.parents[4]
    candidates.extend(
        [
            package_native / "smart_servo_abi.dll",
            package_native / "libsmart_servo_abi.so",
            package_native / "libsmart_servo_abi.dylib",
            root / "target" / "release" / "smart_servo_abi.dll",
            root / "target" / "debug" / "smart_servo_abi.dll",
            root / "target" / "release" / "libsmart_servo_abi.so",
            root / "target" / "debug" / "libsmart_servo_abi.so",
            root / "target" / "release" / "libsmart_servo_abi.dylib",
            root / "target" / "debug" / "libsmart_servo_abi.dylib",
        ]
    )
    return candidates


def _load_library(explicit_path: Optional[str] = None) -> ctypes.CDLL:
    candidates = _candidate_libraries(explicit_path)
    for path in candidates:
        if path.exists():
            return _configure_library(ctypes.CDLL(str(path)))

    searched = "\n".join(str(p) for p in candidates)
    raise LibraryLoadError(
        "smart_servo_abi library not found; build with `cargo build -p smart_servo_abi` "
        "or set MOTORBRIDGE_SMART_SERVO_LIB.\n"
        f"Searched:\n{searched}"
    )


class FashionStarServo:
    """FashionStar UART smart-servo bus.

    The bus owns the serial port. Use it as a context manager when possible.
    """

    def __init__(self, port: str, baudrate: int = 1_000_000, library_path: Optional[str] = None):
        self.port = port
        self.baudrate = baudrate
        self._lib = _load_library(library_path)
        self._handle = self._lib.mbss_fashionstar_open(port.encode("utf-8"), baudrate)
        if not self._handle:
            raise ServoBusError(f"failed to open FashionStar servo bus: {port}")

    @property
    def is_open(self) -> bool:
        return bool(self._handle)

    def close(self) -> None:
        handle = getattr(self, "_handle", None)
        lib = getattr(self, "_lib", None)
        if handle and lib:
            ptr = ctypes.c_void_p(handle)
            lib.mbss_close_handle(ctypes.byref(ptr))
            self._handle = None

    def __enter__(self) -> "FashionStarServo":
        return self

    def __exit__(self, *_exc) -> None:
        self.close()

    def _ensure_open(self) -> None:
        if not self._handle:
            raise ServoBusError("servo bus is closed")

    @staticmethod
    def _check_id(servo_id: int) -> int:
        if not 0 <= int(servo_id) <= 253:
            raise ValueError("servo_id must be in range 0..253")
        return int(servo_id)

    def ping(self, servo_id: int) -> bool:
        self._ensure_open()
        servo_id = self._check_id(servo_id)
        rc = self._lib.mbss_ping(self._handle, servo_id)
        if rc < 0:
            raise ServoBusError("ping failed")
        return rc == 1

    def scan(self, max_id: int = 253) -> list[int]:
        """Return online servo IDs in `0..max_id`."""
        max_id = self._check_id(max_id)
        return [servo_id for servo_id in range(max_id + 1) if self.ping(servo_id)]

    def read_angle(self, servo_id: int, multi_turn: bool = True) -> AngleSample:
        """Read one angle sample.

        The sample contains both raw protocol data and filtered data. For control
        logic, prefer `sample.filtered_deg`.

        Native return codes:
        0 means fresh sample.
        1 means communication failed but the native core returned the last
        reliable filtered value with `reliable=False`.
        """
        self._ensure_open()
        servo_id = self._check_id(servo_id)
        out = _AngleSample()
        rc = self._lib.mbss_read_angle(self._handle, servo_id, multi_turn, ctypes.byref(out))
        if rc < 0:
            raise ServoBusError("read_angle failed")
        return AngleSample(float(out.raw_deg), float(out.filtered_deg), bool(out.reliable))

    def read_raw_angle(self, servo_id: int, multi_turn: bool = True) -> float:
        return self.read_angle(servo_id, multi_turn=multi_turn).raw_deg

    def read_filtered_angle(self, servo_id: int, multi_turn: bool = True) -> float:
        return self.read_angle(servo_id, multi_turn=multi_turn).filtered_deg

    def monitor(
        self,
        servo_id: int,
        multi_turn: bool = True,
        interval_s: float = 0.02,
        count: Optional[int] = None,
    ) -> Iterator[AngleSample]:
        """Yield angle samples at a fixed interval."""
        emitted = 0
        while count is None or emitted < count:
            try:
                yield self.read_angle(servo_id, multi_turn=multi_turn)
                emitted += 1
            except ServoBusError:
                if count is not None:
                    emitted += 1
            time.sleep(interval_s)

    def set_angle(self, servo_id: int, angle_deg: float, multi_turn: bool = False, interval_ms: int = 0) -> None:
        self._ensure_open()
        servo_id = self._check_id(servo_id)
        if interval_ms < 0:
            raise ValueError("interval_ms must be >= 0")
        rc = self._lib.mbss_set_angle(self._handle, servo_id, float(angle_deg), multi_turn, interval_ms)
        if rc != 0:
            raise ServoBusError("set_angle failed")

    move_to = set_angle

    def __del__(self) -> None:
        try:
            self.close()
        except Exception:
            pass


def list_library_candidates() -> list[Path]:
    """Return native library search paths in priority order."""
    return _candidate_libraries()
