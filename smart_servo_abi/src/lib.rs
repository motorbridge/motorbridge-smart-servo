use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::ptr;

use smart_servo_core::SmartServoController;
use smart_servo_vendor_fashionstar::FashionStarController;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MbssAngleSample {
    pub raw_deg: f32,
    pub filtered_deg: f32,
    pub reliable: bool,
}

pub struct MbssHandle {
    controller: FashionStarController,
}

#[no_mangle]
/// # Safety
///
/// `vendor` and `port` must be valid, null-terminated C strings for the duration of this call.
pub unsafe extern "C" fn mbss_open(
    vendor: *const c_char,
    port: *const c_char,
    baudrate: u32,
) -> *mut MbssHandle {
    if vendor.is_null() {
        return ptr::null_mut();
    }

    let Ok(vendor) = CStr::from_ptr(vendor).to_str() else {
        return ptr::null_mut();
    };

    match vendor.to_ascii_lowercase().as_str() {
        "fashionstar" | "fashion-star" | "fs" => unsafe { mbss_fashionstar_open(port, baudrate) },
        _ => ptr::null_mut(),
    }
}

#[no_mangle]
/// # Safety
///
/// `port` must be a valid, null-terminated C string for the duration of this call.
pub unsafe extern "C" fn mbss_fashionstar_open(
    port: *const c_char,
    baudrate: u32,
) -> *mut MbssHandle {
    if port.is_null() {
        return ptr::null_mut();
    }

    let Ok(port) = CStr::from_ptr(port).to_str() else {
        return ptr::null_mut();
    };

    match FashionStarController::open(port.to_string(), baudrate) {
        Ok(controller) => Box::into_raw(Box::new(MbssHandle { controller })),
        Err(_) => ptr::null_mut(),
    }
}

#[no_mangle]
/// # Safety
///
/// `handle` must be a pointer returned by this library and must not have been closed before.
pub unsafe extern "C" fn mbss_close(handle: *mut MbssHandle) {
    if !handle.is_null() {
        drop(Box::from_raw(handle));
    }
}

#[no_mangle]
/// # Safety
///
/// `handle` must point to a handle pointer returned by this library, or be null.
pub unsafe extern "C" fn mbss_close_handle(handle: *mut *mut MbssHandle) {
    if handle.is_null() {
        return;
    }
    let inner = *handle;
    if !inner.is_null() {
        drop(Box::from_raw(inner));
        *handle = ptr::null_mut();
    }
}

#[no_mangle]
/// # Safety
///
/// `handle` must be a valid open handle returned by this library.
pub unsafe extern "C" fn mbss_ping(handle: *mut MbssHandle, id: u8) -> c_int {
    let Some(handle) = handle.as_mut() else {
        return -1;
    };
    match handle.controller.ping(id) {
        Ok(true) => 1,
        Ok(false) => 0,
        Err(_) => -1,
    }
}

#[no_mangle]
/// # Safety
///
/// `handle` must be a valid open handle and `out` must be writable for one sample.
pub unsafe extern "C" fn mbss_read_angle(
    handle: *mut MbssHandle,
    id: u8,
    multi_turn: bool,
    out: *mut MbssAngleSample,
) -> c_int {
    if out.is_null() {
        return -1;
    }
    let Some(handle) = handle.as_mut() else {
        return -1;
    };

    match handle.controller.read_angle(id, multi_turn) {
        Ok(sample) => {
            *out = MbssAngleSample {
                raw_deg: sample.raw_deg,
                filtered_deg: sample.filtered_deg,
                reliable: sample.reliable,
            };
            0
        }
        Err(_) => match handle.controller.filter_timeout_sample(id) {
            Some(sample) => {
                *out = MbssAngleSample {
                    raw_deg: sample.raw_deg,
                    filtered_deg: sample.filtered_deg,
                    reliable: sample.reliable,
                };
                1
            }
            None => -1,
        },
    }
}

#[no_mangle]
/// # Safety
///
/// `handle` must be a valid open handle returned by this library.
pub unsafe extern "C" fn mbss_set_angle(
    handle: *mut MbssHandle,
    id: u8,
    angle_deg: f32,
    multi_turn: bool,
    interval_ms: u32,
) -> c_int {
    let Some(handle) = handle.as_mut() else {
        return -1;
    };
    let interval = (interval_ms > 0).then_some(interval_ms);
    match handle
        .controller
        .set_angle(id, angle_deg, multi_turn, interval)
    {
        Ok(()) => 0,
        Err(_) => -1,
    }
}
