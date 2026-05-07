use std::collections::HashMap;
use std::time::Duration;

use smart_servo_core::{
    AngleReliability, AngleSample, Result, SerialBus, SerialBusConfig, ServoId,
    SmartServoController, SmartServoError,
};

use crate::protocol;
pub use crate::protocol::ServoMonitor;

pub struct FashionStarController {
    bus: SerialBus,
    filters: HashMap<ServoId, AngleReliability>,
}

impl FashionStarController {
    pub fn open(port: impl Into<String>, baudrate: u32) -> Result<Self> {
        Ok(Self {
            bus: SerialBus::open(SerialBusConfig::new(port, baudrate))?,
            filters: HashMap::new(),
        })
    }

    pub fn read_raw_angle(&mut self, id: ServoId, multi_turn: bool) -> Result<f32> {
        self.bus.clear()?;
        self.bus
            .write_all(&protocol::encode_query_angle(id, multi_turn)?)?;
        let data = self.bus.read_until_idle()?;
        let report = protocol::parse_response_stream(&data);
        if report.packets.is_empty() {
            if let Some(err) = report.errors.into_iter().next() {
                return Err(err);
            }
        }
        for packet in report.packets {
            if let Ok((reply_id, angle)) = protocol::decode_angle(&packet, multi_turn) {
                if reply_id == id {
                    return Ok(angle);
                }
            }
        }
        Err(SmartServoError::Timeout)
    }

    pub fn query_monitor(&mut self, id: ServoId) -> Result<ServoMonitor> {
        self.bus.clear()?;
        self.bus.write_all(&protocol::encode_query_monitor(id)?)?;
        let data = self.bus.read_until_idle()?;
        let report = protocol::parse_response_stream(&data);
        if report.packets.is_empty() {
            if let Some(err) = report.errors.into_iter().next() {
                return Err(err);
            }
        }
        for packet in report.packets {
            if let Ok(sample) = protocol::decode_monitor(&packet) {
                if sample.id == id {
                    return Ok(sample);
                }
            }
        }
        Err(SmartServoError::Timeout)
    }

    pub fn read_angle_pair(&mut self, id: ServoId, multi_turn: bool) -> Result<AngleSample> {
        let raw = self.read_raw_angle(id, multi_turn)?;
        let filter = self.filters.entry(id).or_default();
        let (filtered, reliable) = filter.filter(raw);
        Ok(AngleSample {
            raw_deg: raw,
            filtered_deg: filtered,
            reliable,
        })
    }

    pub fn set_timeout(&mut self, timeout: Duration) {
        self.bus.set_timeout(timeout);
    }

    pub fn filter_timeout_sample(&mut self, id: ServoId) -> Option<AngleSample> {
        let filter = self.filters.entry(id).or_default();
        filter.state.last_good_deg.map(|last| AngleSample {
            raw_deg: 0.0,
            filtered_deg: last,
            reliable: false,
        })
    }
}

impl SmartServoController for FashionStarController {
    fn ping(&mut self, id: ServoId) -> Result<bool> {
        self.bus.clear()?;
        self.bus.write_all(&protocol::encode_ping(id)?)?;
        let data = match self.bus.read_until_idle() {
            Ok(data) => data,
            Err(SmartServoError::Timeout) => return Ok(false),
            Err(err) => return Err(err),
        };
        let report = protocol::parse_response_stream(&data);
        if report.packets.is_empty() {
            if let Some(err) = report.errors.into_iter().next() {
                return Err(err);
            }
        }
        Ok(report
            .packets
            .iter()
            .any(|packet| protocol::decode_ping(packet) == Some(id)))
    }

    fn read_angle(&mut self, id: ServoId, multi_turn: bool) -> Result<AngleSample> {
        self.read_angle_pair(id, multi_turn)
    }

    fn set_angle(
        &mut self,
        id: ServoId,
        angle_deg: f32,
        multi_turn: bool,
        interval_ms: Option<u32>,
    ) -> Result<()> {
        let packet = protocol::encode_set_angle(id, angle_deg, multi_turn, interval_ms)?;
        self.bus.write_all(&packet)?;
        Ok(())
    }
}
