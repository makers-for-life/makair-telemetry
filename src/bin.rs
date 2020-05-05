// MakAir
//
// Copyright: 2020, Makers For Life
// License: Public Domain License

#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;

extern crate base64;

use clap::Clap;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};

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
    let (tx, rx): (Sender<TelemetryChannelType>, Receiver<TelemetryChannelType>) =
        std::sync::mpsc::channel();
    std::thread::spawn(move || {
        gather_telemetry(&cfg.port, tx, None);
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
        gather_telemetry(&cfg.port, tx, Some(file_buffer));
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
        gather_telemetry_from_file(file, tx);
    });

    let stopped_message_period = std::time::Duration::from_millis(100);
    let data_message_period = std::time::Duration::from_millis(10);

    loop {
        match rx.try_recv() {
            Ok(msg) => {
                match msg {
                    Ok(TelemetryMessage::StoppedMessage { .. }) => {
                        std::thread::sleep(stopped_message_period);
                    }
                    Ok(TelemetryMessage::DataSnapshot { .. }) => {
                        std::thread::sleep(data_message_period);
                    }
                    _ => (),
                }
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
    let file = File::open(cfg.input).expect("failed to compute statistics for recorded file");

    let (tx, rx): (Sender<TelemetryChannelType>, Receiver<TelemetryChannelType>) =
        std::sync::mpsc::channel();
    std::thread::spawn(move || {
        gather_telemetry_from_file(file, tx);
    });

    let mut telemetry_messages: Vec<TelemetryMessage> = Vec::new();

    let mut nb_boot_messages: u32 = 0;
    let mut nb_alarm_traps: u32 = 0;
    let mut nb_data_snapshots: u32 = 0;
    let mut nb_machine_state_snapshots: u32 = 0;
    let mut nb_stopped_messages: u32 = 0;

    loop {
        match rx.try_recv() {
            Ok(message) => {
                match message {
                    Ok(TelemetryMessage::BootMessage(_)) => {
                        nb_boot_messages += 1;
                    }
                    Ok(TelemetryMessage::AlarmTrap(_)) => {
                        nb_alarm_traps += 1;
                    }
                    Ok(TelemetryMessage::DataSnapshot(_)) => {
                        nb_data_snapshots += 1;
                    }
                    Ok(TelemetryMessage::MachineStateSnapshot(_)) => {
                        nb_machine_state_snapshots += 1;
                    }
                    Ok(TelemetryMessage::StoppedMessage(_)) => {
                        nb_stopped_messages += 1;
                    }
                    _ => {}
                }
                telemetry_messages.push(message.unwrap());
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

fn compute_duration(messages: Vec<TelemetryMessage>) -> u32 {
    let mut duration: u32 = 0;

    for message in &messages {
        match message {
            TelemetryMessage::DataSnapshot(_) => {
                duration += 10;
            }

            TelemetryMessage::StoppedMessage(_) => {
                duration += 100;
            }
            _ => {}
        }
    }

    duration
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_compute_duration_no_data() {
        assert_eq!(compute_duration(vec![]), 0);
    }

    #[test]
    fn test_compute_duration_one_boot_message() {
        let mut vect: Vec<TelemetryMessage> = Vec::new();
        vect.push(TelemetryMessage::BootMessage(BootMessage {
            version: String::from(""),
            device_id: String::from(""),
            systick: 0,
            mode: Mode::Production,
            value128: 0,
        }));

        assert_eq!(compute_duration(vect), 0);
    }

    #[test]
    fn test_compute_duration_one_alarm_trap() {
        let mut vect: Vec<TelemetryMessage> = Vec::new();
        vect.push(TelemetryMessage::AlarmTrap(AlarmTrap {
            version: String::from(""),
            device_id: String::from(""),
            systick: 0,
            centile: 0,
            pressure: 0,
            phase: Phase::Inhalation,
            subphase: SubPhase::Inspiration,
            cycle: 0,
            alarm_code: 0,
            alarm_priority: AlarmPriority::Low,
            triggered: true,
            expected: 0,
            measured: 0,
            cycles_since_trigger: 0,
        }));

        assert_eq!(compute_duration(vect), 0);
    }

    #[test]
    fn test_compute_duration_one_data_snapshot() {
        let mut vect: Vec<TelemetryMessage> = Vec::new();
        vect.push(TelemetryMessage::DataSnapshot(DataSnapshot {
            version: String::from(""),
            device_id: String::from(""),
            systick: 0,
            centile: 0,
            pressure: 0,
            phase: Phase::Inhalation,
            subphase: SubPhase::Inspiration,
            blower_valve_position: 0,
            patient_valve_position: 0,
            blower_rpm: 0,
            battery_level: 0,
        }));

        assert_eq!(compute_duration(vect), 10);
    }

    #[test]
    fn test_compute_duration_one_machine_state_snapshot() {
        let mut vect: Vec<TelemetryMessage> = Vec::new();
        vect.push(TelemetryMessage::MachineStateSnapshot(
            MachineStateSnapshot {
                version: String::from(""),
                device_id: String::from(""),
                systick: 0,
                cycle: 0,
                peak_command: 0,
                plateau_command: 0,
                peep_command: 0,
                cpm_command: 0,
                previous_peak_pressure: 0,
                previous_plateau_pressure: 0,
                previous_peep_pressure: 0,
                current_alarm_codes: vec![],
            },
        ));

        assert_eq!(compute_duration(vect), 0);
    }

    #[test]
    fn test_compute_duration_one_stopped_message() {
        let mut vect: Vec<TelemetryMessage> = Vec::new();

        vect.push(TelemetryMessage::StoppedMessage(StoppedMessage {
            version: String::from(""),
            device_id: String::from(""),
            systick: 0,
        }));

        assert_eq!(compute_duration(vect), 100);
    }

    #[test]
    fn test_compute_duration_one_of_each_message() {
        let mut vect: Vec<TelemetryMessage> = Vec::new();

        vect.push(TelemetryMessage::BootMessage(BootMessage {
            version: String::from(""),
            device_id: String::from(""),
            systick: 0,
            mode: Mode::Production,
            value128: 0,
        }));

        vect.push(TelemetryMessage::AlarmTrap(AlarmTrap {
            version: String::from(""),
            device_id: String::from(""),
            systick: 0,
            centile: 0,
            pressure: 0,
            phase: Phase::Inhalation,
            subphase: SubPhase::Inspiration,
            cycle: 0,
            alarm_code: 0,
            alarm_priority: AlarmPriority::Low,
            triggered: true,
            expected: 0,
            measured: 0,
            cycles_since_trigger: 0,
        }));

        vect.push(TelemetryMessage::DataSnapshot(DataSnapshot {
            version: String::from(""),
            device_id: String::from(""),
            systick: 0,
            centile: 0,
            pressure: 0,
            phase: Phase::Inhalation,
            subphase: SubPhase::Inspiration,
            blower_valve_position: 0,
            patient_valve_position: 0,
            blower_rpm: 0,
            battery_level: 0,
        }));

        vect.push(TelemetryMessage::MachineStateSnapshot(
            MachineStateSnapshot {
                version: String::from(""),
                device_id: String::from(""),
                systick: 0,
                cycle: 0,
                peak_command: 0,
                plateau_command: 0,
                peep_command: 0,
                cpm_command: 0,
                previous_peak_pressure: 0,
                previous_plateau_pressure: 0,
                previous_peep_pressure: 0,
                current_alarm_codes: vec![],
            },
        ));

        vect.push(TelemetryMessage::StoppedMessage(StoppedMessage {
            version: String::from(""),
            device_id: String::from(""),
            systick: 0,
        }));

        assert_eq!(compute_duration(vect), 110);
    }
}
