use std::io::{Read, Write};
use std::time::{Duration, Instant};

use serialport::SerialPort;

use crate::{Result, SmartServoError};

#[derive(Debug, Clone)]
pub struct SerialBusConfig {
    pub port: String,
    pub baudrate: u32,
    pub timeout: Duration,
    pub read_timeout: Duration,
}

impl SerialBusConfig {
    pub fn new(port: impl Into<String>, baudrate: u32) -> Self {
        Self {
            port: port.into(),
            baudrate,
            timeout: Duration::from_millis(100),
            read_timeout: Duration::from_millis(10),
        }
    }
}

pub struct SerialBus {
    port: Box<dyn SerialPort>,
    timeout: Duration,
}

impl SerialBus {
    pub fn open(config: SerialBusConfig) -> Result<Self> {
        let port = serialport::new(&config.port, config.baudrate)
            .timeout(config.read_timeout)
            .open()?;
        Ok(Self {
            port,
            timeout: config.timeout,
        })
    }

    pub fn write_all(&mut self, data: &[u8]) -> Result<()> {
        self.port.write_all(data)?;
        self.port.flush()?;
        Ok(())
    }

    pub fn clear(&mut self) -> Result<()> {
        self.port.clear(serialport::ClearBuffer::All)?;
        Ok(())
    }

    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    pub fn read_until_idle(&mut self) -> Result<Vec<u8>> {
        let started = Instant::now();
        let mut out = Vec::new();
        let mut scratch = [0_u8; 256];

        while started.elapsed() < self.timeout {
            match self.port.read(&mut scratch) {
                Ok(0) => std::thread::sleep(Duration::from_millis(1)),
                Ok(n) => out.extend_from_slice(&scratch[..n]),
                Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {}
                Err(e) => return Err(e.into()),
            }
        }

        if out.is_empty() {
            return Err(SmartServoError::Timeout);
        }
        Ok(out)
    }
}
