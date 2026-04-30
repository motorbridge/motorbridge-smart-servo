use smart_servo_core::AngleReliability;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmAngleReliability {
    inner: AngleReliability,
}

#[wasm_bindgen]
impl WasmAngleReliability {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: AngleReliability::default(),
        }
    }

    pub fn filter(&mut self, raw_deg: f32) -> WasmAngleSample {
        let (filtered_deg, reliable) = self.inner.filter(raw_deg);
        WasmAngleSample {
            raw_deg,
            filtered_deg,
            reliable,
        }
    }
}

impl Default for WasmAngleReliability {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct WasmAngleSample {
    raw_deg: f32,
    filtered_deg: f32,
    reliable: bool,
}

#[wasm_bindgen]
impl WasmAngleSample {
    #[wasm_bindgen(getter)]
    pub fn raw_deg(&self) -> f32 {
        self.raw_deg
    }

    #[wasm_bindgen(getter)]
    pub fn filtered_deg(&self) -> f32 {
        self.filtered_deg
    }

    #[wasm_bindgen(getter)]
    pub fn reliable(&self) -> bool {
        self.reliable
    }
}
