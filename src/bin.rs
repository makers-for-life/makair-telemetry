// MakAir
//
// Copyright: 2020, Makers For Life
// License: Public Domain License

#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;

mod statistics;

use clap::Clap;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};

use control::*;
use statistics::*;
use structures::*;
use telemetry::*;

#[derive(Clap)]
#[clap(version = crate_version!(), author = crate_authors!())]
struct Opts {
    #[clap(subcommand)]
    mode: Mode,
}

#[derive(Clap)]
enum Mode {
    /// Reads telemetry from a serial port, parses it and streams result to stdout
    #[clap(version = crate_version!(), author = crate_authors!())]
    Debug(Debug),

    /// Reads telemetry from a serial port and save bytes to a file
    #[clap(version = crate_version!(), author = crate_authors!())]
    Record(Record),

    /// Reads telemetry from a recorded file, parses it and streams result to stdout
    #[clap(version = crate_version!(), author = crate_authors!())]
    Play(Play),

    /// Reads telemetry from a recorded file, parses it and compute some statistics
    #[clap(version = crate_version!(), author = crate_authors!())]
    Stats(Stats),
}

#[derive(Clap)]
struct Debug {
    /// Address of the port to use
    #[clap(short = "p")]
    port: String,

    /// Randomly send control messages
    #[clap(short = "c", long = "random-control-messages")]
    random_control_messages: bool,
}

#[derive(Clap)]
struct Record {
    /// Address of the port to use
    #[clap(short = "p")]
    port: String,

    /// Path of the file to write to
    #[clap(short = "o")]
    output: String,
}

#[derive(Clap)]
struct Play {
    /// Path of the recorded file
    #[clap(short = "i")]
    input: String,
}

#[derive(Clap)]
struct Stats {
    /// Path of the recorded file
    #[clap(short = "i")]
    input: String,
}

fn main() {
    env_logger::init();
    let opts: Opts = Opts::parse();

    match opts.mode {
        Mode::Debug(cfg) => debug(cfg),
        Mode::Record(cfg) => record(cfg),
        Mode::Play(cfg) => play(cfg),
        Mode::Stats(cfg) => stats(cfg),
    }
}

fn debug(cfg: Debug) {
    let control_rx = if cfg.random_control_messages {
        let (control_tx, control_rx): (Sender<ControlMessage>, Receiver<ControlMessage>) =
            std::sync::mpsc::channel();
        std::thread::spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_secs(3));
            control_tx
                .send(ControlMessage {
                    setting: ControlSetting::PeakPressure,
                    value: 5,
                })
                .expect("[control tx] failed to send control message");
        });
        Some(control_rx)
    } else {
        None
    };

    let (tx, rx): (Sender<TelemetryChannelType>, Receiver<TelemetryChannelType>) =
        std::sync::mpsc::channel();
    std::thread::spawn(move || {
        gather_telemetry(&cfg.port, tx, None, control_rx);
    });
    loop {
        match rx.try_recv() {
            Ok(msg) => {
                display_message(msg);
            }
            Err(TryRecvError::Empty) => {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            Err(TryRecvError::Disconnected) => {
                panic!("channel to serial port thread was closed");
            }
        }
    }
}

fn record(cfg: Record) {
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&cfg.output)
        .expect("failed to create recording file");
    let file_buffer = BufWriter::new(file);

    let (tx, rx): (Sender<TelemetryChannelType>, Receiver<TelemetryChannelType>) =
        std::sync::mpsc::channel();
    std::thread::spawn(move || {
        gather_telemetry(&cfg.port, tx, Some(file_buffer), None);
    });
    loop {
        match rx.try_recv() {
            Ok(msg) => {
                display_message(msg);
            }
            Err(TryRecvError::Empty) => {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            Err(TryRecvError::Disconnected) => {
                panic!("channel to serial port thread was closed");
            }
        }
    }
}

fn play(cfg: Play) {
    let file = File::open(cfg.input).expect("failed to play recorded file");
    let (tx, rx): (Sender<TelemetryChannelType>, Receiver<TelemetryChannelType>) =
        std::sync::mpsc::channel();
    std::thread::spawn(move || {
        info!("start playing telemetry messages");
        gather_telemetry_from_file(file, tx, true);
    });

    loop {
        match rx.try_recv() {
            Ok(msg) => {
                display_message(msg);
            }
            Err(TryRecvError::Empty) => {
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            Err(TryRecvError::Disconnected) => {
                warn!("end of recording");
                std::process::exit(0);
            }
        }
    }
}

fn stats(cfg: Stats) {
    let file = File::open(cfg.input).expect("failed to open given recorded file");

    let (tx, rx): (Sender<TelemetryChannelType>, Receiver<TelemetryChannelType>) =
        std::sync::mpsc::channel();
    std::thread::spawn(move || {
        gather_telemetry_from_file(file, tx, false);
    });

    let mut telemetry_messages: Vec<TelemetryMessage> = Vec::new();

    let mut nb_boot_messages: u32 = 0;
    let mut nb_alarm_traps: u32 = 0;
    let mut nb_data_snapshots: u32 = 0;
    let mut nb_machine_state_snapshots: u32 = 0;
    let mut nb_stopped_messages: u32 = 0;

    loop {
        match rx.try_recv() {
            Ok(channel_message) => {
                if let Ok(message) = channel_message {
                    match message {
                        TelemetryMessage::BootMessage(_) => {
                            nb_boot_messages += 1;
                        }
                        TelemetryMessage::AlarmTrap(_) => {
                            nb_alarm_traps += 1;
                        }
                        TelemetryMessage::DataSnapshot(_) => {
                            nb_data_snapshots += 1;
                        }
                        TelemetryMessage::MachineStateSnapshot(_) => {
                            nb_machine_state_snapshots += 1;
                        }
                        TelemetryMessage::StoppedMessage(_) => {
                            nb_stopped_messages += 1;
                        }
                    }
                    telemetry_messages.push(message);
                }
            }
            Err(TryRecvError::Empty) => {
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            Err(TryRecvError::Disconnected) => {
                println!("Statistics");
                println!("Nb BootMessages: {}", nb_boot_messages);
                println!("Nb AlarmTraps: {}", nb_alarm_traps);
                println!("Nb DataSnapshots: {}", nb_data_snapshots);
                println!("Nb MachineStateSnapshot: {}", nb_machine_state_snapshots);
                println!("Nb StoppedMessage: {}", nb_stopped_messages);
                println!(
                    "Estimated duration: {:.3} seconds",
                    compute_duration(telemetry_messages) as f32 / 1000_f32
                );
                std::process::exit(0);
            }
        }
    }
}
