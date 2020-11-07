// MakAir Telemetry
//
// Copyright: 2020, Makers For Life
// License: Public Domain License

#[macro_use]
extern crate log;

mod convert;
mod statistics;
mod storm;

use clap::Clap;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};

use control::*;
use convert::*;
use statistics::*;
use storm::*;
use structures::*;
use telemetry::*;

#[derive(Debug, Clap)]
#[clap(author, about, version)]
struct Opts {
    #[clap(subcommand)]
    mode: Mode,
}

#[derive(Debug, Clap)]
enum Mode {
    /// Reads telemetry from a serial port, parses it and streams result to stdout
    Debug(Debug),

    /// Reads telemetry from a serial port and save bytes to a file
    Record(Record),

    /// Reads telemetry from a recorded file, parses it and streams result to stdout
    Play(Play),

    /// Reads telemetry from a recorded file, parses it and compute some statistics
    Stats(Stats),

    /// Send one specific control message to a serial port, then run debug mode
    Control(Control),

    /// Send a lot of control messages and/or bytes to a serial port
    Storm(Storm),

    /// Reads telemetry from a recorded file, parses it and converts it to another format
    Convert(Convert),

    /// Send a control message to disable the RPi watchdog (until MCU is restarted)
    DisableRpiWatchdog(DisableRpiWatchdog),
}

#[derive(Debug, Clap)]
struct Debug {
    /// Address of the port to use
    #[clap(short = 'p')]
    port: String,

    /// Randomly send control messages at a normal pace
    #[clap(short = 'c', long = "random-control-messages")]
    random_control_messages: bool,
}

#[derive(Debug, Clap)]
struct Record {
    /// Address of the port to use
    #[clap(short = 'p')]
    port: String,

    /// Path of the file to write to
    #[clap(short = 'o')]
    output: String,
}

#[derive(Debug, Clap)]
struct Play {
    /// Path of the recorded file
    #[clap(short = 'i')]
    input: String,

    /// Parse and output data as fast as possible
    #[clap(long = "full-blast")]
    full_blast: bool,
}

#[derive(Debug, Clap)]
struct Stats {
    /// Path of the recorded file
    #[clap(short = 'i')]
    input: String,
}

#[derive(Debug, Clap)]
struct Control {
    /// Address of the port to use
    #[clap(short = 'p')]
    port: String,

    /// Setting internal number
    #[clap(name = "setting")]
    setting: u8,

    /// Value
    #[clap(name = "value")]
    value: u16,
}

#[derive(Debug, Clap)]
struct Storm {
    /// Address of the port to use
    #[clap(short = 'p')]
    port: String,

    /// [generator] Send valid control messages
    #[clap(short = 'v', long = "valid")]
    valid: bool,

    /// [generator] Send random bytes
    #[clap(short = 'b', long = "bytes")]
    bytes: bool,

    /// [generator] Send control messages with wrong CRC
    #[clap(short = 'c', long = "wrong-crc")]
    wrong_crc: bool,

    /// Send data as fast as possible (MCU might not be able to read it, but it should not crash)
    #[clap(short = 'f', long = "full-blast")]
    full_blast: bool,
}

#[derive(Debug, Clap)]
struct Convert {
    /// Path of the recorded file
    #[clap(short = 'i')]
    input: String,

    /// Path of the converted file
    #[clap(short = 'o')]
    output: String,

    /// If a systick value is specified, only messages with a greater or equal systick will be included
    #[clap(long)]
    from: Option<u64>,

    /// If a systick value is specified, only messages with a smaller or equal systick will be included
    #[clap(long)]
    to: Option<u64>,

    /// Output format
    #[clap(short = 'f')]
    format: Format,

    /// [GTS] Value to use in a "source" label in every GTS line; uses the input filename if not specified
    #[clap(long)]
    gts_source_label: Option<String>,

    /// [GTS] Do not put automatic or manual "source" label in every GTS line
    #[clap(long)]
    gts_disable_source_label: bool,
}

#[derive(Debug, Clap)]
struct DisableRpiWatchdog {
    /// Address of the port to use
    #[clap(short = 'p')]
    port: String,
}

const THREAD_SLEEP_THROTTLE: std::time::Duration = std::time::Duration::from_millis(10);
const HEARTBEAT_PERIOD: std::time::Duration = std::time::Duration::from_secs(30);

fn main() {
    env_logger::init();
    let opts: Opts = Opts::parse();

    match opts.mode {
        Mode::Debug(cfg) => debug(cfg),
        Mode::Record(cfg) => record(cfg),
        Mode::Play(cfg) => play(cfg),
        Mode::Stats(cfg) => stats(cfg),
        Mode::Control(cfg) => control(cfg),
        Mode::Storm(cfg) => storm(cfg),
        Mode::Convert(cfg) => convert(cfg),
        Mode::DisableRpiWatchdog(cfg) => disable_rpi_watchdog(cfg),
    }
}

fn debug(cfg: Debug) {
    let (control_tx, control_rx): (Sender<ControlMessage>, Receiver<ControlMessage>) =
        std::sync::mpsc::channel();

    let heartbeat_tx = control_tx.clone();
    std::thread::spawn(move || loop {
        heartbeat_tx
            .send(ControlMessage {
                setting: ControlSetting::Heartbeat,
                value: 0,
            })
            .expect("[heartbeat tx] failed to send heartbeat message");
        std::thread::sleep(HEARTBEAT_PERIOD);
    });

    if cfg.random_control_messages {
        std::thread::spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_secs(3));
            control_tx
                .send(gen_random_message())
                .expect("[control tx] failed to send control message");
        });
    };

    let (tx, rx): (Sender<TelemetryChannelType>, Receiver<TelemetryChannelType>) =
        std::sync::mpsc::channel();
    std::thread::spawn(move || {
        gather_telemetry(&cfg.port, tx, None, Some(control_rx));
    });
    loop {
        match rx.try_recv() {
            Ok(msg) => {
                display_message(msg);
            }
            Err(TryRecvError::Empty) => {
                std::thread::sleep(THREAD_SLEEP_THROTTLE);
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

    let (heartbeat_tx, control_rx): (Sender<ControlMessage>, Receiver<ControlMessage>) =
        std::sync::mpsc::channel();
    std::thread::spawn(move || loop {
        heartbeat_tx
            .send(ControlMessage {
                setting: ControlSetting::Heartbeat,
                value: 0,
            })
            .expect("[heartbeat tx] failed to send heartbeat message");
        std::thread::sleep(HEARTBEAT_PERIOD);
    });

    let (tx, rx): (Sender<TelemetryChannelType>, Receiver<TelemetryChannelType>) =
        std::sync::mpsc::channel();
    std::thread::spawn(move || {
        gather_telemetry(&cfg.port, tx, Some(file_buffer), Some(control_rx));
    });
    loop {
        match rx.try_recv() {
            Ok(msg) => {
                display_message(msg);
            }
            Err(TryRecvError::Empty) => {
                std::thread::sleep(THREAD_SLEEP_THROTTLE);
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
    let enable_time_simulation = !cfg.full_blast;
    std::thread::spawn(move || {
        info!("start playing telemetry messages");
        gather_telemetry_from_file(file, tx, enable_time_simulation);
    });

    loop {
        match rx.try_recv() {
            Ok(msg) => {
                display_message(msg);
            }
            Err(TryRecvError::Empty) => {
                std::thread::sleep(THREAD_SLEEP_THROTTLE);
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
    let mut nb_control_ack: u32 = 0;

    loop {
        match rx.try_recv() {
            Ok(channel_message) => {
                if let Ok(TelemetryMessageOrError::Message(message)) = channel_message {
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
                        TelemetryMessage::ControlAck(_) => {
                            nb_control_ack += 1;
                        }
                    }
                    telemetry_messages.push(message);
                }
            }
            Err(TryRecvError::Empty) => {
                std::thread::sleep(THREAD_SLEEP_THROTTLE);
            }
            Err(TryRecvError::Disconnected) => {
                println!("Statistics");
                println!("Nb BootMessages: {}", nb_boot_messages);
                println!("Nb AlarmTraps: {}", nb_alarm_traps);
                println!("Nb DataSnapshots: {}", nb_data_snapshots);
                println!("Nb MachineStateSnapshot: {}", nb_machine_state_snapshots);
                println!("Nb StoppedMessage: {}", nb_stopped_messages);
                println!("Nb ControlAck: {}", nb_control_ack);
                println!(
                    "Estimated duration: {:.3} seconds",
                    compute_duration(telemetry_messages) as f32 / 1000_f32
                );
                std::process::exit(0);
            }
        }
    }
}

fn control(cfg: Control) {
    use std::convert::TryFrom;

    let setting = ControlSetting::try_from(cfg.setting).expect("invalid control setting passed");
    let value = cfg.value;

    let (control_tx, control_rx): (Sender<ControlMessage>, Receiver<ControlMessage>) =
        std::sync::mpsc::channel();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(3));
        control_tx
            .send(ControlMessage { setting, value })
            .expect("[control tx] failed to send control message");
    });

    let (tx, rx): (Sender<TelemetryChannelType>, Receiver<TelemetryChannelType>) =
        std::sync::mpsc::channel();
    std::thread::spawn(move || {
        gather_telemetry(&cfg.port, tx, None, Some(control_rx));
    });
    loop {
        match rx.try_recv() {
            Ok(msg) => {
                display_message(msg);
            }
            Err(TryRecvError::Empty) => {
                std::thread::sleep(THREAD_SLEEP_THROTTLE);
            }
            Err(TryRecvError::Disconnected) => {
                panic!("channel to serial port thread was closed");
            }
        }
    }
}

fn storm(cfg: Storm) {
    use serial::prelude::*;
    use std::io::Write;

    let port_id = cfg.port;
    let full_blast = cfg.full_blast;
    let (tx, rx): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = std::sync::mpsc::channel();

    let mut generators: Vec<&'static str> = vec![];
    if cfg.valid {
        generators.push("valid");
    };
    if cfg.bytes {
        generators.push("bytes");
    };
    if cfg.wrong_crc {
        generators.push("wrong_crc");
    };
    if generators.is_empty() {
        panic!("You must specify at least one generator; use '-h' to see the list");
    }

    std::thread::spawn(move || {
        use rand::seq::SliceRandom;

        std::thread::sleep(std::time::Duration::from_secs(3));
        loop {
            let bytes = match generators.choose(&mut rand::thread_rng()) {
                Some(&"valid") => gen_random_message_bytes(),
                Some(&"bytes") => gen_random_bytes(),
                Some(&"wrong_crc") => gen_random_message_with_wrong_crc(),
                _ => unreachable!(),
            };
            tx.send(bytes).expect("[tx] failed to send bytes");
            if !full_blast {
                std::thread::sleep(std::time::Duration::from_millis(15));
            }
        }
    });

    info!("opening {}", &port_id);
    match serial::open(&port_id) {
        Err(e) => {
            error!("{:?}", e);
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        Ok(mut port) => {
            match port.reconfigure(&|settings| {
                settings.set_char_size(serial::Bits8);
                settings.set_parity(serial::ParityNone);
                settings.set_stop_bits(serial::Stop1);
                settings.set_flow_control(serial::FlowNone);
                settings.set_baud_rate(serial::Baud115200)
            }) {
                Err(e) => {
                    error!("{}", e);
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
                Ok(_) => loop {
                    match rx.try_recv() {
                        Ok(bytes) => {
                            let write = port.write_all(&bytes);
                            match write {
                                Ok(_) => debug!("→ {:?}", &bytes),
                                Err(e) => warn!("Could not send bytes '{:?}': {:?}", &bytes, &e),
                            }
                        }
                        Err(std::sync::mpsc::TryRecvError::Empty) => (),
                        Err(e) => panic!(e),
                    }
                },
            }
        }
    }
}

fn convert(cfg: Convert) {
    use std::io::Write;
    use std::path::Path;

    let from = cfg.from.unwrap_or(u64::MIN);
    let to = cfg.to.unwrap_or(u64::MAX);
    let mut skipped = 0u64;

    if from > to {
        error!("systick in --from cannot be greater than systick in --to");
        std::process::exit(1);
    }

    let input_file_name = cfg.input;
    let input_file = File::open(&input_file_name).expect("failed to open recorded file");
    let output_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&cfg.output)
        .expect("failed to create recording file");
    let mut output_buffer = BufWriter::new(output_file);

    let gts_source_label = if cfg.format == Format::GTS && !cfg.gts_disable_source_label {
        cfg.gts_source_label.or_else(|| {
            Path::new(&input_file_name)
                .file_name()
                .map(|ostr| ostr.to_string_lossy().into_owned())
        })
    } else {
        None
    };

    let (tx, rx): (Sender<TelemetryChannelType>, Receiver<TelemetryChannelType>) =
        std::sync::mpsc::channel();
    std::thread::spawn(move || {
        info!("start playing telemetry messages");
        gather_telemetry_from_file(input_file, tx, false);
    });

    loop {
        match rx.try_recv() {
            Ok(Ok(TelemetryMessageOrError::Message(msg))) => {
                if msg.systick() >= from && msg.systick() <= to {
                    let output_payload = match cfg.format {
                        Format::GTS => telemetry_to_gts(&msg, &gts_source_label),
                        Format::JSON => {
                            telemetry_to_json(&msg).expect("Failed to serialize a message to JSON")
                        }
                    };
                    output_buffer
                        .write_all(output_payload.as_bytes())
                        .expect("failed to write to output file");
                } else {
                    skipped += 1;
                }
            }
            Ok(msg) => {
                display_message(msg);
            }
            Err(TryRecvError::Empty) => {
                std::thread::sleep(THREAD_SLEEP_THROTTLE);
            }
            Err(TryRecvError::Disconnected) => {
                warn!("end of recording");
                if skipped != 0 {
                    info!("{} records were skipped", &skipped);
                }
                output_buffer
                    .flush()
                    .expect("failed to write to output file");
                std::process::exit(0);
            }
        }
    }
}

fn disable_rpi_watchdog(cfg: DisableRpiWatchdog) {
    control(Control {
        port: cfg.port,
        setting: ControlSetting::Heartbeat as u8,
        value: DISABLE_RPI_WATCHDOG,
    })
}
