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
    }

    Ok(())
}
