use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

use clap::{Parser, Subcommand};
use smart_servo_core::{SmartServoController, SmartServoError};
use smart_servo_vendor_fashionstar::FashionStarController;

fn open_fashionstar_or_error(
    vendor: &str,
    port: String,
    baudrate: u32,
) -> Result<FashionStarController, Box<dyn std::error::Error>> {
    match vendor.to_ascii_lowercase().as_str() {
        "fashionstar" | "fashion-star" | "fs" => Ok(FashionStarController::open(port, baudrate)?),
        other => Err(format!("unsupported smart-servo vendor: {other}").into()),
    }
}

#[derive(Parser)]
#[command(name = "smart-servo")]
#[command(about = "MotorBridge Smart Servo CLI")]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Scan {
        #[arg(long, default_value = "fashionstar")]
        vendor: String,
        #[arg(long)]
        port: String,
        #[arg(long, default_value_t = 1_000_000)]
        baudrate: u32,
        #[arg(long, default_value_t = 20)]
        max_id: u8,
        #[arg(long, default_value_t = 30)]
        timeout_ms: u64,
    },
    ReadAngle {
        #[arg(long, default_value = "fashionstar")]
        vendor: String,
        #[arg(long)]
        port: String,
        #[arg(long, default_value_t = 1_000_000)]
        baudrate: u32,
        #[arg(long)]
        id: u8,
        #[arg(long)]
        multi_turn: bool,
        #[arg(long)]
        raw: bool,
    },
    Monitor {
        #[arg(long, default_value = "fashionstar")]
        vendor: String,
        #[arg(long)]
        port: String,
        #[arg(long, default_value_t = 1_000_000)]
        baudrate: u32,
        #[arg(long)]
        id: u8,
        #[arg(long)]
        multi_turn: bool,
        #[arg(long, default_value_t = 20)]
        interval_ms: u64,
    },
    SetAngle {
        #[arg(long, default_value = "fashionstar")]
        vendor: String,
        #[arg(long)]
        port: String,
        #[arg(long, default_value_t = 1_000_000)]
        baudrate: u32,
        #[arg(long)]
        id: u8,
        #[arg(long)]
        angle: f32,
        #[arg(long)]
        multi_turn: bool,
        #[arg(long)]
        interval_ms: Option<u32>,
    },
    QueryMonitor {
        #[arg(long, default_value = "fashionstar")]
        vendor: String,
        #[arg(long)]
        port: String,
        #[arg(long, default_value_t = 1_000_000)]
        baudrate: u32,
        #[arg(long)]
        id: u8,
    },
    QueryMode {
        #[arg(long, default_value = "fashionstar")]
        vendor: String,
        #[arg(long)]
        port: String,
        #[arg(long, default_value_t = 1_000_000)]
        baudrate: u32,
        #[arg(long)]
        id: u8,
    },
    /// Query multiple servos in one sync command (code 25).
    /// Example: --ids 0 1 2 3 4 5 6
    SyncMonitor {
        #[arg(long, default_value = "fashionstar")]
        vendor: String,
        #[arg(long)]
        port: String,
        #[arg(long, default_value_t = 1_000_000)]
        baudrate: u32,
        /// Servo IDs to query, e.g. --ids 0 1 2 3 4 5 6
        #[arg(long, num_args = 1..)]
        ids: Vec<u8>,
        /// Repeat continuously at this interval
        #[arg(long)]
        interval_ms: Option<u64>,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match args.command {
        Command::Scan {
            vendor,
            port,
            baudrate,
            max_id,
            timeout_ms,
        } => {
            let mut ctl = open_fashionstar_or_error(&vendor, port, baudrate)?;
            ctl.set_timeout(Duration::from_millis(timeout_ms));
            for id in 0..=max_id {
                if ctl.ping(id)? {
                    println!("{id}");
                }
            }
        }
        Command::ReadAngle {
            vendor,
            port,
            baudrate,
            id,
            multi_turn,
            raw,
        } => {
            let mut ctl = open_fashionstar_or_error(&vendor, port, baudrate)?;
            if raw {
                println!("{:.3}", ctl.read_raw_angle(id, multi_turn)?);
            } else {
                let sample = ctl.read_angle(id, multi_turn)?;
                println!(
                    "raw={:.3} filtered={:.3} reliable={}",
                    sample.raw_deg, sample.filtered_deg, sample.reliable
                );
            }
        }
        Command::Monitor {
            vendor,
            port,
            baudrate,
            id,
            multi_turn,
            interval_ms,
        } => {
            let mut ctl = open_fashionstar_or_error(&vendor, port, baudrate)?;
            let running = Arc::new(AtomicBool::new(true));
            let signal = running.clone();
            ctrlc::set_handler(move || {
                signal.store(false, Ordering::SeqCst);
            })?;
            while running.load(Ordering::SeqCst) {
                match ctl.read_angle(id, multi_turn) {
                    Ok(sample) => println!(
                        "raw={:9.3} filtered={:9.3} reliable={}",
                        sample.raw_deg, sample.filtered_deg, sample.reliable
                    ),
                    Err(SmartServoError::Timeout) => {
                        if let Some(sample) = ctl.filter_timeout_sample(id) {
                            println!(
                                "raw={:9.3} filtered={:9.3} reliable={}",
                                sample.raw_deg, sample.filtered_deg, sample.reliable
                            );
                        } else {
                            eprintln!("error: timeout");
                        }
                    }
                    Err(err) => eprintln!("error: {err}"),
                }
                thread::sleep(Duration::from_millis(interval_ms));
            }
        }
        Command::SetAngle {
            vendor,
            port,
            baudrate,
            id,
            angle,
            multi_turn,
            interval_ms,
        } => {
            let mut ctl = open_fashionstar_or_error(&vendor, port, baudrate)?;
            ctl.set_angle(id, angle, multi_turn, interval_ms)?;
        }
        Command::QueryMonitor {
            vendor,
            port,
            baudrate,
            id,
        } => {
            let mut ctl = open_fashionstar_or_error(&vendor, port, baudrate)?;
            let m = ctl.query_monitor(id)?;
            println!(
                "id={} voltage={:.3}V current={:.3}A power={:.3}W temp_raw={} status=0x{:02x} angle={:.3} turn={}",
                m.id,
                m.voltage_mv as f32 / 1000.0,
                m.current_ma as f32 / 1000.0,
                m.power_mw as f32 / 1000.0,
                m.temp_raw,
                m.status,
                m.angle_deg,
                m.turn
            );
        }
        Command::QueryMode {
            vendor,
            port,
            baudrate,
            id,
        } => {
            let mut ctl = open_fashionstar_or_error(&vendor, port, baudrate)?;
            let m = ctl.query_monitor(id)?;
            let busy = (m.status & 0x01) != 0;
            let has_exec_error = (m.status & 0x02) != 0;
            let has_stall = (m.status & 0x04) != 0;
            let has_voltage_high = (m.status & 0x08) != 0;
            let has_voltage_low = (m.status & 0x10) != 0;
            let has_current_err = (m.status & 0x20) != 0;
            let has_power_err = (m.status & 0x40) != 0;
            let has_temp_err = (m.status & 0x80) != 0;

            println!("id={}", m.id);
            println!("status=0x{:02x}", m.status);
            println!(
                "flags: busy={} exec_error={} stall={} v_high={} v_low={} i_err={} p_err={} t_err={}",
                busy, has_exec_error, has_stall, has_voltage_high, has_voltage_low, has_current_err, has_power_err, has_temp_err
            );
            println!(
                "telemetry: voltage={:.3}V current={:.3}A power={:.3}W angle={:.3} turn={}",
                m.voltage_mv as f32 / 1000.0,
                m.current_ma as f32 / 1000.0,
                m.power_mw as f32 / 1000.0,
                m.angle_deg,
                m.turn
            );
            println!(
                "mode_hint: protocol has no direct 'current mode' query; this is status-based inference only."
            );
        }
        Command::SyncMonitor {
            vendor,
            port,
            baudrate,
            ids,
            interval_ms,
        } => {
            let mut ctl = open_fashionstar_or_error(&vendor, port, baudrate)?;
            let running = Arc::new(AtomicBool::new(true));
            let signal = running.clone();
            ctrlc::set_handler(move || signal.store(false, Ordering::SeqCst))?;

            loop {
                let t0 = std::time::Instant::now();
                match ctl.sync_monitor(&ids) {
                    Ok(result) => {
                        let elapsed = t0.elapsed().as_micros();
                        let mut sorted: Vec<u8> = ids.clone();
                        sorted.sort();
                        for id in &sorted {
                            match result.get(id) {
                                Some(Some(m)) => println!(
                                    "id={id} angle={:8.3} volt={:.2}V reliable={}  +{elapsed}µs",
                                    m.angle_deg,
                                    m.voltage_mv as f32 / 1000.0,
                                    m.reliable,
                                ),
                                Some(None) => println!("id={id} no response  +{elapsed}µs"),
                                None => {}
                            }
                        }
                        println!("--- {elapsed}µs for {} servo(s)", ids.len());
                    }
                    Err(SmartServoError::ConsecutiveLoss { id, count }) => {
                        eprintln!("fatal: servo {id} lost {count} consecutive responses, stopping");
                        break;
                    }
                    Err(e) => eprintln!("error: {e}"),
                }

                if !running.load(Ordering::SeqCst) {
                    break;
                }
                if let Some(ms) = interval_ms {
                    thread::sleep(Duration::from_millis(ms));
                } else {
                    break;
                }
            }
        }
    }

    Ok(())
}
