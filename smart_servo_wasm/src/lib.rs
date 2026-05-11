use js_sys::Date;
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

    pub fn with_config(zero_eps_deg: f32, zero_confirm_duration_s: f32) -> Self {
        Self {
            inner: AngleReliability {
                config: AngleReliabilityConfig {
                    zero_eps_deg,
                    zero_confirm_duration_s: zero_confirm_duration_s.max(0.0),
                    valid_range_deg: 3_686_400.0,
                },
                state: Default::default(),
            },
        }
    }

    pub fn set_zero_confirm_duration_s(&mut self, seconds: f32) {
        self.inner.config.zero_confirm_duration_s = seconds.max(0.0);
    }

    pub fn filter(&mut self, raw_deg: f32) -> WasmAngleSample {
        let (filtered_deg, reliable) = self.inner.filter_at(raw_deg, Date::now() / 1000.0);
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
pub fn fashionstar_sync_monitor_packet(ids: Vec<u8>) -> Result<Vec<u8>, JsValue> {
    protocol::encode_sync_monitor(&ids).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Count how many valid monitor (code 22) packets are in `data`.
/// JS uses this to know when all servo responses have arrived.
#[wasm_bindgen]
pub fn fashionstar_count_monitor_packets(data: &[u8]) -> u32 {
    let report = protocol::parse_response_stream(data);
    report
        .packets
        .iter()
        .filter(|p| protocol::decode_monitor(p).is_ok())
        .count() as u32
}

/// Decode the monitor response for a specific servo ID from the raw byte stream.
/// Returns the raw angle (before JS-side reliability filtering) and voltage.
#[wasm_bindgen]
pub fn fashionstar_decode_monitor_angle(data: &[u8], id: u8) -> WasmMonitorDecodeResult {
    let report = protocol::parse_response_stream(data);
    for packet in &report.packets {
        if let Ok(m) = protocol::decode_monitor(packet) {
            if m.id == id {
                return WasmMonitorDecodeResult {
                    found: true,
                    raw_deg: m.raw_deg,
                    voltage_mv: m.voltage_mv,
                };
            }
        }
    }
    WasmMonitorDecodeResult {
        found: false,
        raw_deg: 0.0,
        voltage_mv: 0,
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct WasmMonitorDecodeResult {
    found: bool,
    raw_deg: f32,
    voltage_mv: u16,
}

#[wasm_bindgen]
impl WasmMonitorDecodeResult {
    #[wasm_bindgen(getter)]
    pub fn found(&self) -> bool {
        self.found
    }

    #[wasm_bindgen(getter)]
    pub fn raw_deg(&self) -> f32 {
        self.raw_deg
    }

    #[wasm_bindgen(getter)]
    pub fn voltage_mv(&self) -> u16 {
        self.voltage_mv
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
