from __future__ import annotations

import time
from pathlib import Path
from typing import Iterator, Optional

from ._native import AngleSample, FashionStarServo as _NativeFashionStarServo


class SmartServoError(RuntimeError):
    """Base exception raised by motorbridge-smart-servo."""


class LibraryLoadError(SmartServoError):
    """Kept for source compatibility; PyO3 wheels do not load an external DLL."""


class ServoBusError(SmartServoError):
    """Raised when a native bus operation fails."""


def _wrap_native_error(exc: RuntimeError) -> ServoBusError:
    return ServoBusError(str(exc))


class FashionStarServo:
    """FashionStar UART smart-servo bus.

    The Python API is intentionally vendor-stable. The implementation underneath
    is a PyO3 native extension, so wheels contain the Rust core directly instead
    of loading a separate smart_servo_abi DLL/SO with ctypes.
    """

    def __init__(self, port: str, baudrate: int = 1_000_000, library_path: Optional[str] = None):
        if library_path is not None:
            # Keep the old constructor shape compatible, but the new PyO3 backend
            # no longer needs or honors an external native-library path.
            raise LibraryLoadError("library_path is not supported by the PyO3 backend")
        self.port = port
        self.baudrate = baudrate
        try:
            self._inner = _NativeFashionStarServo(port, baudrate)
        except RuntimeError as exc:
            raise _wrap_native_error(exc) from exc

    @property
    def is_open(self) -> bool:
        return bool(self._inner.is_open)

    def close(self) -> None:
        try:
            self._inner.close()
        except RuntimeError as exc:
            raise _wrap_native_error(exc) from exc

    def __enter__(self) -> "FashionStarServo":
        return self

    def __exit__(self, *_exc) -> None:
        self.close()

    @staticmethod
    def _check_id(servo_id: int) -> int:
        if not 0 <= int(servo_id) <= 253:
            raise ValueError("servo_id must be in range 0..253")
        return int(servo_id)

    def ping(self, servo_id: int) -> bool:
        try:
            return bool(self._inner.ping(self._check_id(servo_id)))
        except RuntimeError as exc:
            raise _wrap_native_error(exc) from exc

    def scan(self, max_id: int = 253) -> list[int]:
        """Return online servo IDs in `0..max_id`."""
        try:
            return list(self._inner.scan(self._check_id(max_id)))
        except RuntimeError as exc:
            raise _wrap_native_error(exc) from exc

    def read_angle(self, servo_id: int, multi_turn: bool = True) -> AngleSample:
        """Read one angle sample.

        `raw_deg` is the protocol value read from the servo.
        `filtered_deg` suppresses power-cycle A->0->B glitches.
        `reliable=False` means `filtered_deg` is being held from the last good value.
        """
        try:
            return self._inner.read_angle(self._check_id(servo_id), bool(multi_turn))
        except RuntimeError as exc:
            raise _wrap_native_error(exc) from exc

    def read_raw_angle(self, servo_id: int, multi_turn: bool = True) -> float:
        return float(self.read_angle(servo_id, multi_turn=multi_turn).raw_deg)

    def read_filtered_angle(self, servo_id: int, multi_turn: bool = True) -> float:
        return float(self.read_angle(servo_id, multi_turn=multi_turn).filtered_deg)

    def monitor(
        self,
        servo_id: int,
        multi_turn: bool = True,
        interval_s: float = 0.02,
        count: Optional[int] = None,
    ) -> Iterator[AngleSample]:
        """Yield angle samples at a fixed interval without crashing on transient timeouts."""
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
        if interval_ms < 0:
            raise ValueError("interval_ms must be >= 0")
        try:
            self._inner.set_angle(self._check_id(servo_id), float(angle_deg), bool(multi_turn), int(interval_ms))
        except ValueError:
            raise
        except RuntimeError as exc:
            raise _wrap_native_error(exc) from exc

    move_to = set_angle

    def __del__(self) -> None:
        try:
            self.close()
        except Exception:
            pass


def list_library_candidates() -> list[Path]:
    """Return an empty list because PyO3 wheels do not use external native libraries."""
    return []
