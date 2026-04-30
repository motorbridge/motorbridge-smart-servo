from __future__ import annotations

import shutil
import subprocess
from pathlib import Path

from setuptools import build_meta as _orig


ROOT = Path(__file__).resolve().parents[2]
PKG_NATIVE = Path(__file__).resolve().parent / "src" / "motorbridge_smart_servo" / "native"


def _native_lib_name() -> str:
    import platform

    system = platform.system().lower()
    if system == "windows":
        return "smart_servo_abi.dll"
    if system == "darwin":
        return "libsmart_servo_abi.dylib"
    return "libsmart_servo_abi.so"


def _build_native() -> None:
    subprocess.run(
        ["cargo", "build", "-p", "smart_servo_abi", "--release"],
        cwd=ROOT,
        check=True,
    )

    lib_name = _native_lib_name()
    src = ROOT / "target" / "release" / lib_name
    if not src.exists():
        raise FileNotFoundError(f"native library not found after build: {src}")

    PKG_NATIVE.mkdir(parents=True, exist_ok=True)
    shutil.copy2(src, PKG_NATIVE / lib_name)


def build_wheel(wheel_directory, config_settings=None, metadata_directory=None):
    _build_native()
    return _orig.build_wheel(wheel_directory, config_settings, metadata_directory)


def build_sdist(sdist_directory, config_settings=None):
    return _orig.build_sdist(sdist_directory, config_settings)


def prepare_metadata_for_build_wheel(metadata_directory, config_settings=None):
    return _orig.prepare_metadata_for_build_wheel(metadata_directory, config_settings)

