use smart_servo_core::{Result, SmartServoError};

pub const REQ_HEADER: [u8; 2] = [0x12, 0x4c];
pub const RESP_HEADER: [u8; 2] = [0x05, 0x1c];

pub const CODE_PING: u8 = 1;
pub const CODE_SET_SERVO_ANGLE: u8 = 8;
pub const CODE_QUERY_SERVO_ANGLE: u8 = 10;
pub const CODE_SET_SERVO_ANGLE_BY_INTERVAL: u8 = 11;
pub const CODE_SET_SERVO_ANGLE_MTURN_BY_INTERVAL: u8 = 14;
pub const CODE_QUERY_SERVO_ANGLE_MTURN: u8 = 16;

#[derive(Debug, Clone)]
pub struct Packet {
    pub code: u8,
    pub params: Vec<u8>,
}

pub fn checksum(header: [u8; 2], code: u8, params: &[u8]) -> u8 {
    let mut sum = header[0] as u16 + header[1] as u16 + code as u16 + params.len() as u16;
    for byte in params {
        sum += *byte as u16;
    }
    (sum & 0xff) as u8
}

pub fn pack_request(code: u8, params: &[u8]) -> Result<Vec<u8>> {
    if params.len() > u8::MAX as usize {
        return Err(SmartServoError::Protocol(
            "packet params too large".to_string(),
        ));
    }

    let mut out = Vec::with_capacity(5 + params.len());
    out.extend_from_slice(&REQ_HEADER);
    out.push(code);
    out.push(params.len() as u8);
    out.extend_from_slice(params);
    out.push(checksum(REQ_HEADER, code, params));
    Ok(out)
}

#[derive(Debug, Default)]
pub struct ParseReport {
    pub packets: Vec<Packet>,
    pub errors: Vec<SmartServoError>,
}

pub fn parse_response_stream(data: &[u8]) -> ParseReport {
    let mut report = ParseReport::default();
    let mut i = 0;

    while i + 5 <= data.len() {
        if data[i..].starts_with(&RESP_HEADER) {
            let code = data[i + 2];
            let size = data[i + 3] as usize;
            let end = i + 2 + 1 + 1 + size + 1;
            if end > data.len() {
                break;
            }
            let params = &data[i + 4..i + 4 + size];
            let got = data[end - 1];
            let expected = checksum(RESP_HEADER, code, params);
            if got == expected {
                report.packets.push(Packet {
                    code,
                    params: params.to_vec(),
                });
            } else {
                report.errors.push(SmartServoError::ChecksumMismatch {
                    code,
                    expected,
                    got,
                });
            }
            i = end;
        } else {
            i += 1;
        }
    }

    report
}

pub fn encode_ping(id: u8) -> Result<Vec<u8>> {
    pack_request(CODE_PING, &[id])
}

pub fn encode_query_angle(id: u8, multi_turn: bool) -> Result<Vec<u8>> {
    let code = if multi_turn {
        CODE_QUERY_SERVO_ANGLE_MTURN
    } else {
        CODE_QUERY_SERVO_ANGLE
    };
    pack_request(code, &[id])
}

pub fn encode_set_angle(
    id: u8,
    angle_deg: f32,
    multi_turn: bool,
    interval_ms: Option<u32>,
) -> Result<Vec<u8>> {
    if !angle_deg.is_finite() {
        return Err(SmartServoError::Protocol(
            "angle_deg must be finite".to_string(),
        ));
    }
    let angle_raw = (angle_deg * 10.0).round() as i32;
    match (multi_turn, interval_ms) {
        (true, Some(interval)) => {
            if interval > 4_096_000 {
                return Err(SmartServoError::Protocol(
                    "multi-turn interval_ms must be <= 4096000".to_string(),
                ));
            }
            let mut p = Vec::with_capacity(13);
            p.push(id);
            p.extend_from_slice(&angle_raw.to_le_bytes());
            p.extend_from_slice(&interval.to_le_bytes());
            p.extend_from_slice(&20_u16.to_le_bytes());
            p.extend_from_slice(&20_u16.to_le_bytes());
            p.extend_from_slice(&0_u16.to_le_bytes());
            pack_request(CODE_SET_SERVO_ANGLE_MTURN_BY_INTERVAL, &p)
        }
        (false, Some(interval)) => {
            if interval > u16::MAX as u32 {
                return Err(SmartServoError::Protocol(
                    "single-turn interval_ms must be <= 65535".to_string(),
                ));
            }
            let clamped = angle_raw.clamp(-1800, 1800) as i16;
            let mut p = Vec::with_capacity(11);
            p.push(id);
            p.extend_from_slice(&clamped.to_le_bytes());
            p.extend_from_slice(&(interval as u16).to_le_bytes());
            p.extend_from_slice(&20_u16.to_le_bytes());
            p.extend_from_slice(&20_u16.to_le_bytes());
            p.extend_from_slice(&0_u16.to_le_bytes());
            pack_request(CODE_SET_SERVO_ANGLE_BY_INTERVAL, &p)
        }
        (false, None) => {
            let clamped = angle_raw.clamp(-1800, 1800) as i16;
            let mut p = Vec::with_capacity(3);
            p.push(id);
            p.extend_from_slice(&clamped.to_le_bytes());
            pack_request(CODE_SET_SERVO_ANGLE, &p)
        }
        (true, None) => Err(SmartServoError::Unsupported(
            "FashionStar multi-turn set-angle requires interval_ms in this first implementation"
                .to_string(),
        )),
    }
}

pub fn decode_ping(packet: &Packet) -> Option<u8> {
    (packet.code == CODE_PING && packet.params.len() == 1).then_some(packet.params[0])
}

pub fn decode_angle(packet: &Packet, multi_turn: bool) -> Result<(u8, f32)> {
    if multi_turn {
        if packet.code != CODE_QUERY_SERVO_ANGLE_MTURN || packet.params.len() < 7 {
            return Err(SmartServoError::Protocol(
                "unexpected multi-turn angle response".to_string(),
            ));
        }
        let id = packet.params[0];
        let raw = i32::from_le_bytes(packet.params[1..5].try_into().unwrap());
        Ok((id, raw as f32 / 10.0))
    } else {
        if packet.code != CODE_QUERY_SERVO_ANGLE || packet.params.len() < 3 {
            return Err(SmartServoError::Protocol(
                "unexpected angle response".to_string(),
            ));
        }
        let id = packet.params[0];
        let raw = i16::from_le_bytes(packet.params[1..3].try_into().unwrap());
        Ok((id, raw as f32 / 10.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_checksum_matches_python_sdk_shape() {
        let pkt = encode_ping(1).unwrap();
        assert_eq!(&pkt[..4], &[0x12, 0x4c, CODE_PING, 1]);
        assert_eq!(pkt[4], 1);
        assert_eq!(pkt[5], checksum(REQ_HEADER, CODE_PING, &[1]));
    }

    #[test]
    fn rejects_non_finite_angle() {
        assert!(encode_set_angle(0, f32::NAN, false, None).is_err());
        assert!(encode_set_angle(0, f32::INFINITY, true, Some(100)).is_err());
    }

    #[test]
    fn rejects_single_turn_interval_truncation() {
        assert!(encode_set_angle(0, 10.0, false, Some(100_000)).is_err());
    }

    #[test]
    fn reports_checksum_mismatch() {
        let mut bad = vec![0x05, 0x1c, CODE_PING, 1, 1, 0];
        bad[5] = 0xaa;
        let report = parse_response_stream(&bad);
        assert!(report.packets.is_empty());
        assert_eq!(report.errors.len(), 1);
    }
}
