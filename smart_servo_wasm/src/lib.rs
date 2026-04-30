use smart_servo_core::{AngleReliability, AngleReliabilityConfig};
use smart_servo_vendor_fashionstar::protocol;
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

    pub fn with_config(
        zero_eps_deg: f32,
        zero_jump_min_deg: f32,
        zero_confirm_samples: u16,
    ) -> Self {
        Self {
            inner: AngleReliability {
                config: AngleReliabilityConfig {
                    zero_eps_deg,
                    zero_jump_min_deg,
                    zero_confirm_samples: zero_confirm_samples.max(1),
                },
                state: Default::default(),
            },
        }
    }

    pub fn set_zero_confirm_samples(&mut self, samples: u16) {
        self.inner.config.zero_confirm_samples = samples.max(1);
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

#[wasm_bindgen]
pub fn fashionstar_query_angle_packet(id: u8, multi_turn: bool) -> Result<Vec<u8>, JsValue> {
    protocol::encode_query_angle(id, multi_turn).map_err(|err| JsValue::from_str(&err.to_string()))
}

#[wasm_bindgen]
pub fn fashionstar_decode_angle(data: &[u8], id: u8, multi_turn: bool) -> WasmAngleDecodeResult {
    let report = protocol::parse_response_stream(data);
    let first_error = report.errors.first().map(ToString::to_string);

    for packet in report.packets {
        if let Ok((reply_id, raw_deg)) = protocol::decode_angle(&packet, multi_turn) {
            if reply_id == id {
                return WasmAngleDecodeResult {
                    found: true,
                    raw_deg,
                    error: None,
                };
            }
        }
    }

    WasmAngleDecodeResult {
        found: false,
        raw_deg: 0.0,
        error: first_error,
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmAngleDecodeResult {
    found: bool,
    raw_deg: f32,
    error: Option<String>,
}

#[wasm_bindgen]
impl WasmAngleDecodeResult {
    #[wasm_bindgen(getter)]
    pub fn found(&self) -> bool {
        self.found
    }

    #[wasm_bindgen(getter)]
    pub fn raw_deg(&self) -> f32 {
        self.raw_deg
    }

    #[wasm_bindgen(getter)]
    pub fn error(&self) -> Option<String> {
        self.error.clone()
    }
}
