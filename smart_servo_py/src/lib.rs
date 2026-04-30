use std::sync::{Mutex, MutexGuard};

use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use smart_servo_core::{AngleSample, ServoId, SmartServoController, SmartServoError};
use smart_servo_vendor_fashionstar::FashionStarController;

fn py_bus_err(err: SmartServoError) -> PyErr {
    PyRuntimeError::new_err(err.to_string())
}

fn py_closed_err() -> PyErr {
    PyRuntimeError::new_err("servo bus is closed")
}

fn check_servo_id(servo_id: u8) -> ServoId {
    servo_id
}

fn checked_interval(interval_ms: u32) -> PyResult<u32> {
    if interval_ms > u16::MAX as u32 {
        return Err(PyValueError::new_err(
            "interval_ms must be in range 0..65535",
        ));
    }
    Ok(interval_ms)
}

fn finite_angle(angle_deg: f32) -> PyResult<f32> {
    if !angle_deg.is_finite() {
        return Err(PyValueError::new_err("angle_deg must be finite"));
    }
    Ok(angle_deg)
}

#[pyclass(
    name = "AngleSample",
    module = "motorbridge_smart_servo",
    frozen,
    skip_from_py_object
)]
#[derive(Debug, Clone, Copy)]
pub struct PyAngleSample {
    #[pyo3(get)]
    raw_deg: f32,
    #[pyo3(get)]
    filtered_deg: f32,
    #[pyo3(get)]
    reliable: bool,
}

impl From<AngleSample> for PyAngleSample {
    fn from(sample: AngleSample) -> Self {
        Self {
            raw_deg: sample.raw_deg,
            filtered_deg: sample.filtered_deg,
            reliable: sample.reliable,
        }
    }
}

#[pymethods]
impl PyAngleSample {
    fn __repr__(&self) -> String {
        format!(
            "AngleSample(raw_deg={:.3}, filtered_deg={:.3}, reliable={})",
            self.raw_deg, self.filtered_deg, self.reliable
        )
    }
}

#[pyclass(name = "FashionStarServo", module = "motorbridge_smart_servo._native")]
pub struct PyFashionStarServo {
    inner: Mutex<Option<FashionStarController>>,
}

impl PyFashionStarServo {
    fn controller(&self) -> PyResult<MutexGuard<'_, Option<FashionStarController>>> {
        self.inner
            .lock()
            .map_err(|_| PyRuntimeError::new_err("servo bus lock poisoned"))
    }

    fn with_controller<T>(
        &self,
        f: impl FnOnce(&mut FashionStarController) -> PyResult<T>,
    ) -> PyResult<T> {
        let mut guard = self.controller()?;
        let controller = guard.as_mut().ok_or_else(py_closed_err)?;
        f(controller)
    }
}

#[pymethods]
impl PyFashionStarServo {
    #[new]
    #[pyo3(signature = (port, baudrate = 1_000_000))]
    fn new(port: String, baudrate: u32) -> PyResult<Self> {
        let controller = FashionStarController::open(port, baudrate).map_err(py_bus_err)?;
        Ok(Self {
            inner: Mutex::new(Some(controller)),
        })
    }

    #[getter]
    fn is_open(&self) -> PyResult<bool> {
        Ok(self.controller()?.is_some())
    }

    fn close(&self) -> PyResult<()> {
        *self.controller()? = None;
        Ok(())
    }

    fn ping(&self, py: Python<'_>, servo_id: u8) -> PyResult<bool> {
        let servo_id = check_servo_id(servo_id);
        py.detach(|| {
            self.with_controller(|controller| controller.ping(servo_id).map_err(py_bus_err))
        })
    }

    #[pyo3(signature = (max_id = 253))]
    fn scan(&self, py: Python<'_>, max_id: u8) -> PyResult<Vec<u8>> {
        py.detach(|| {
            let mut online = Vec::new();
            self.with_controller(|controller| {
                for servo_id in 0..=max_id {
                    if controller.ping(servo_id).map_err(py_bus_err)? {
                        online.push(servo_id);
                    }
                }
                Ok(online)
            })
        })
    }

    #[pyo3(signature = (servo_id, multi_turn = true))]
    fn read_angle(
        &self,
        py: Python<'_>,
        servo_id: u8,
        multi_turn: bool,
    ) -> PyResult<PyAngleSample> {
        let servo_id = check_servo_id(servo_id);
        py.detach(|| {
            self.with_controller(
                |controller| match controller.read_angle(servo_id, multi_turn) {
                    Ok(sample) => Ok(sample.into()),
                    Err(SmartServoError::Timeout) => controller
                        .filter_timeout_sample(servo_id)
                        .map(PyAngleSample::from)
                        .ok_or_else(|| py_bus_err(SmartServoError::Timeout)),
                    Err(err) => Err(py_bus_err(err)),
                },
            )
        })
    }

    #[pyo3(signature = (servo_id, multi_turn = true))]
    fn read_raw_angle(&self, py: Python<'_>, servo_id: u8, multi_turn: bool) -> PyResult<f32> {
        Ok(self.read_angle(py, servo_id, multi_turn)?.raw_deg)
    }

    #[pyo3(signature = (servo_id, multi_turn = true))]
    fn read_filtered_angle(&self, py: Python<'_>, servo_id: u8, multi_turn: bool) -> PyResult<f32> {
        Ok(self.read_angle(py, servo_id, multi_turn)?.filtered_deg)
    }

    #[pyo3(signature = (servo_id, angle_deg, multi_turn = false, interval_ms = 0))]
    fn set_angle(
        &self,
        py: Python<'_>,
        servo_id: u8,
        angle_deg: f32,
        multi_turn: bool,
        interval_ms: u32,
    ) -> PyResult<()> {
        let servo_id = check_servo_id(servo_id);
        let angle_deg = finite_angle(angle_deg)?;
        let interval_ms = checked_interval(interval_ms)?;
        py.detach(|| {
            self.with_controller(|controller| {
                controller
                    .set_angle(servo_id, angle_deg, multi_turn, Some(interval_ms))
                    .map_err(py_bus_err)
            })
        })
    }

    #[pyo3(signature = (servo_id, angle_deg, multi_turn = false, interval_ms = 0))]
    fn move_to(
        &self,
        py: Python<'_>,
        servo_id: u8,
        angle_deg: f32,
        multi_turn: bool,
        interval_ms: u32,
    ) -> PyResult<()> {
        self.set_angle(py, servo_id, angle_deg, multi_turn, interval_ms)
    }
}

#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyAngleSample>()?;
    m.add_class::<PyFashionStarServo>()?;
    Ok(())
}
