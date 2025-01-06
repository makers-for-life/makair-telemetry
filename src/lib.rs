// MakAir Telemetry
//
// Copyright: 2020, Makers For Life
// License: Public Domain License

//! This crate is a library that handles reading and parsing the MakAir's telemetry binary protocol, and optionally sending new settings values using the Makair's control binary protocol.

// Force exposed items to be documented
#![deny(missing_docs)]
// Required for the parsers to compile
#![recursion_limit = "256"]
// Enable documentation of features
#![cfg_attr(doc_cfg, feature(doc_cfg))]

/// Utilities related to alarms
pub mod alarm;
/// Structures to represent control messages
pub mod control;
/// Error-related entities
pub mod error;
/// Tools to manipulate ISO 639-1 language codes to be used in the control protocol
pub mod locale;
/// Underlying parsers for telemetry messages
pub mod parsers;
/// Binary representation of telemtry messages
pub mod serializers;
/// Structures to represent telemetry messages
pub mod structures;

#[cfg(feature = "serial")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "serial")))]
/// Re-export serial lib
pub use serial;
#[cfg(feature = "websocket")]
/// Re-export Url lib
pub use url;

use log::{debug, error, info, warn};
#[cfg(feature = "serial")]
use serial::prelude::*;
use std::fs::File;
use std::io::{BufRead, BufReader};
#[cfg(feature = "serial")]
use std::io::{BufWriter, Read, Write};
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
#[cfg(feature = "serial")]
use std::sync::{Arc, Mutex};
use std::time::Duration;
#[cfg(feature = "websocket")]
use url::Url;

use control::*;
use parsers::*;
use structures::*;

use error::Error;

/// A decoded telemetry message
pub type TelemetryChannelType = Result<TelemetryMessage, Error>;

/// Open a serial port, consume it endlessly and send parsed telemetry messages through a channel
///
/// * `port_id` - Name or path to the serial port.
/// * `tx` - Sender of a channel.
/// * `file_buf` - Optional file buffer; if specified, messages will also be serialized and written in this file.
/// * `control_rx` - Optional receiver of a channel used to send control messages through the serial port.
///
/// This is meant to be run in a dedicated thread.
#[cfg(feature = "serial")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "serial")))]
pub fn gather_telemetry(
    port_id: &str,
    tx: Sender<TelemetryChannelType>,
    mut file_buf: Option<BufWriter<File>>,
    control_rx: Option<Receiver<ControlMessage>>,
) -> ! {
    loop {
        info!("opening {}", &port_id);
        match serial::open(&port_id) {
            Err(e) => {
                error!("{:?}", e);
                tx.send(Err(e.into()))
                    .expect("[tx channel] failed to send error");
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
                        tx.send(Err(e.into()))
                            .expect("[tx channel] failed setting up port");
                        std::thread::sleep(std::time::Duration::from_secs(1));
                    }
                    Ok(_) => {
                        let port_handle = Arc::new(Mutex::new(port));
                        let mut buffer = Vec::new();
                        loop {
                            let mut tmp = [0; 1];
                            let b = port_handle
                                .lock()
                                .expect("[port] failed getting exclusive lock on serial port to read telemetry")
                                .read(&mut tmp).map(|_| tmp[0]);
                            match b {
                                // We got a new byte
                                Ok(byte) => {
                                    // We add it to the buffer
                                    buffer.push(byte);

                                    // Let's try to parse the buffer
                                    match parse_telemetry_message(&buffer) {
                                        // It worked! Let's extract the message and replace the buffer with the rest of the bytes
                                        Ok((rest, message)) => {
                                            if let Some(file_buffer) = file_buf.as_mut() {
                                                // Write a new line with the base64 value of the message
                                                let base64 = base64::encode(&buffer);
                                                file_buffer.write_all(base64.as_bytes()).expect(
                                                    "[tx channel] failed flushing buffer to file",
                                                );
                                                file_buffer.write_all(b"\n").expect("[tx channel] failed ending buffer flush to file");
                                                file_buffer.flush().expect("[tx channel] failed flushing buffer flush to file");
                                            }

                                            tx.send(Ok(message))
                                                .expect("[tx channel] failed sending message");

                                            buffer = Vec::from(rest);
                                        }
                                        // Message was read but there was a CRC error
                                        Err(nom::Err::Failure(TelemetryError(
                                            msg_bytes,
                                            TelemetryErrorKind::CrcError { expected, computed },
                                        ))) => {
                                            warn!(
                                                "[CRC error]\texpected={}\tcomputed={}",
                                                expected, computed
                                            );

                                            tx.send(Err(HighLevelError::CrcError {
                                                expected,
                                                computed,
                                            }
                                            .into()))
                                                .expect("[tx channel] failed sending message");

                                            buffer = buffer.clone().split_off(msg_bytes.len());
                                        }
                                        // Message was built using an unsupported protocol version
                                        Err(nom::Err::Failure(TelemetryError(
                                            msg_bytes,
                                            TelemetryErrorKind::UnsupportedProtocolVersion {
                                                maximum_supported,
                                                found,
                                            },
                                        ))) => {
                                            warn!(
                                                "[unsupported protocol version]\tmaximum_supported={}\tfound={}",
                                                maximum_supported, found
                                            );

                                            tx.send(Err(
                                                HighLevelError::UnsupportedProtocolVersion {
                                                    maximum_supported,
                                                    found,
                                                }
                                                .into(),
                                            ))
                                            .expect("[tx channel] failed sending message");

                                            buffer = buffer.clone().split_off(msg_bytes.len());
                                        }
                                        // There are not enough bytes, let's wait until we get more
                                        Err(nom::Err::Incomplete(_)) => {
                                            // Do nothing
                                            if let Some(file_buffer) = file_buf.as_mut() {
                                                file_buffer.flush().expect("[tx channel] failed flushing file buffer from incomplete parsing");
                                            }
                                        }
                                        // We can't do anything with the begining of the buffer, let's drop its first byte
                                        Err(e) => {
                                            debug!("{:?}", &e);
                                            if !buffer.is_empty() {
                                                if let Some(file_buffer) = file_buf.as_mut() {
                                                    file_buffer.flush().expect("[tx channel] failed flushing file buffer from parsing error");
                                                }

                                                buffer.remove(0);
                                            }
                                        }
                                    }
                                }
                                // We failed to get a new byte from serial
                                Err(e) => {
                                    if let Some(file_buffer) = file_buf.as_mut() {
                                        file_buffer.flush().expect("[tx channel] failed flushing file buffer from serial error");
                                    }
                                    if e.kind() == std::io::ErrorKind::TimedOut { // It's OK it's just a timeout; let's try again
                                         // Do nothing
                                    } else {
                                        // It's another error, let's print it and wait a bit before retrying the whole process
                                        error!("{:?}", &e);
                                        std::thread::sleep(std::time::Duration::from_secs(1));
                                        break;
                                    }
                                }
                            };
                            if let Some(rx) = control_rx.as_ref() {
                                if let Ok(message) = rx.try_recv() {
                                    let write = port_handle
                                        .lock()
                                        .expect("[port] failed getting exclusive lock on serial port to write control message")
                                        .write_all(&message.to_control_frame());
                                    match write {
                                        Ok(_) => debug!("→ {}", &message),
                                        Err(e) => warn!(
                                            "Could not send control message '{}': {:?}",
                                            &message, &e
                                        ),
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Helper to display telemetry messages
pub fn display_message(message: TelemetryChannelType) {
    match message {
        Ok(TelemetryMessage::BootMessage(BootMessage { value128, .. })) => {
            debug!("####################################################################################");
            debug!("######### CONTROLLER STARTED #########");
            debug!("####################################################################################");
            info!(
                "{:?}",
                &message.expect("failed unwrapping message for boot")
            );
            debug!("####################################################################################");
            if value128 != 128u8 {
                error!("value128 should be equal to 128 (found {:b} = {}); check serial port configuration", &value128, &value128);
            }
        }
        Ok(TelemetryMessage::StoppedMessage(_)) => {
            debug!("stopped");
        }
        Ok(TelemetryMessage::DataSnapshot(_)) => {
            info!(
                "    {:?}",
                &message.expect("failed unwrapping message for data snapshot")
            );
        }
        Ok(TelemetryMessage::MachineStateSnapshot(_)) => {
            debug!("------------------------------------------------------------------------------------");
            info!(
                "{:?}",
                &message.expect("failed unwrapping message for machine snapshot")
            );
            debug!("------------------------------------------------------------------------------------");
        }
        Ok(TelemetryMessage::AlarmTrap(AlarmTrap { triggered, .. })) => {
            let prefix = if triggered { "NEW ALARM" } else { "STOPPED" };
            debug!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
            info!(
                "{} {:?}",
                &prefix,
                &message.expect("failed unwrapping message for alarm trap")
            );
            debug!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
        }
        Ok(TelemetryMessage::ControlAck(ControlAck { setting, value, .. })) => {
            info!("← {:?} = {}", &setting, &value);
        }
        Ok(TelemetryMessage::FatalError(FatalError { error, .. })) => {
            info!("***** FATAL ERROR ***** {:?}", &error);
        }
        Ok(TelemetryMessage::EolTestSnapshot(_)) => {
            info!(
                "    {:?}",
                &message.expect("failed unwrapping message for EOL test snapshot")
            );
        }
        Err(e) => {
            warn!("an error occurred: {:?}", e);
        }
    }
}

/// Open a file containing serialized telemetry data, read it and send back parsed telemetry messages through a channel
///
/// * `file` - Handle to a file that contains telemetry data.
/// * `tx` - Sender of a channel.
/// * `enable_time_simulation` - If `true`, telemetry messages will be sent in a realistic timing; if `false`, they will be read as fast as possible.
///
/// This is meant to be run in a dedicated thread.
pub fn gather_telemetry_from_file(
    file: File,
    tx: Sender<TelemetryChannelType>,
    enable_time_simulation: bool,
) {
    let reader = BufReader::new(file);
    let mut buffer = Vec::new();

    let stopped_message_period = std::time::Duration::from_millis(100);
    let data_message_period = std::time::Duration::from_millis(10);

    for line_str in reader.lines().flatten() {
        if let Ok(mut bytes) = base64::decode(line_str) {
            buffer.append(&mut bytes);

            while !buffer.is_empty() {
                // Let's try to parse the buffer
                match parse_telemetry_message(&buffer) {
                    // It worked! Let's extract the message and replace the buffer with the rest of the bytes
                    Ok((rest, message)) => {
                        if enable_time_simulation {
                            match message {
                                TelemetryMessage::StoppedMessage { .. } => {
                                    std::thread::sleep(stopped_message_period);
                                }
                                TelemetryMessage::DataSnapshot { .. } => {
                                    std::thread::sleep(data_message_period);
                                }
                                _ => (),
                            }
                        }
                        tx.send(Ok(message))
                            .expect("failed sending message to tx channel");
                        buffer = Vec::from(rest);
                    }
                    // There are not enough bytes, let's wait until we get more
                    Err(nom::Err::Incomplete(_)) => {
                        break;
                    }
                    // We can't do anything with the begining of the buffer, let's drop its first byte
                    Err(e) => {
                        debug!("{:?}", &e);
                        if !buffer.is_empty() {
                            buffer.remove(0);
                        }
                    }
                }
            }
        }
    }
}

/// Connect to a WebSocket server, get binary messages endlessly and send parsed telemetry messages through a channel
///
/// * `url` - URL to the WebSocket server.
/// * `tx` - Sender of a channel.
/// * `file_buf` - Optional file buffer; if specified, messages will also be serialized and written in this file.
/// * `control_rx` - Optional receiver of a channel used to send control messages through the WS session.
///
/// This is meant to be run in a dedicated thread.
#[cfg(feature = "websocket")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "websocket")))]
pub fn gather_telemetry_from_ws(
    url: &Url,
    tx: Sender<TelemetryChannelType>,
    mut file_buf: Option<BufWriter<File>>,
    control_rx: Option<Receiver<ControlMessage>>,
) -> ! {
    use tungstenite::client::connect;
    use tungstenite::protocol::Message;

    use serializers::ToBytes;

    loop {
        info!("opening {}", &url);

        match connect(url) {
            Err(e) => {
                error!("{:?}", e);
                tx.send(Err(e.into()))
                    .expect("[tx channel] failed to send error");
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
            Ok((mut socket, _response)) => {
                info!("WebSocket connection was successfuly established");
                'ws_session: loop {
                    match socket.read_message() {
                        Ok(Message::Binary(bytes)) => {
                            // Let's try to parse the received message
                            match parse_telemetry_message(&bytes) {
                                // It worked!
                                Ok((_rest, message)) => {
                                    if let Some(file_buffer) = file_buf.as_mut() {
                                        // Write a new line with the base64 value of the message
                                        let base64 = base64::encode(&message.to_bytes());
                                        file_buffer
                                            .write_all(base64.as_bytes())
                                            .expect("[tx channel] failed flushing buffer to file");
                                        file_buffer.write_all(b"\n").expect(
                                            "[tx channel] failed ending buffer flush to file",
                                        );
                                        file_buffer.flush().expect(
                                            "[tx channel] failed flushing buffer flush to file",
                                        );
                                    }

                                    tx.send(Ok(message))
                                        .expect("[tx channel] failed sending message");
                                }
                                // Message was read but there was a CRC error
                                Err(nom::Err::Failure(TelemetryError(
                                    _msg_bytes,
                                    TelemetryErrorKind::CrcError { expected, computed },
                                ))) => {
                                    warn!(
                                        "[CRC error]\texpected={}\tcomputed={}",
                                        expected, computed
                                    );

                                    tx.send(Err(
                                        HighLevelError::CrcError { expected, computed }.into()
                                    ))
                                    .expect("[tx channel] failed sending message");
                                }
                                // Message was built using an unsupported protocol version
                                Err(nom::Err::Failure(TelemetryError(
                                    _msg_bytes,
                                    TelemetryErrorKind::UnsupportedProtocolVersion {
                                        maximum_supported,
                                        found,
                                    },
                                ))) => {
                                    warn!(
                                        "[unsupported protocol version]\tmaximum_supported={}\tfound={}",
                                        maximum_supported, found
                                    );

                                    tx.send(Err(HighLevelError::UnsupportedProtocolVersion {
                                        maximum_supported,
                                        found,
                                    }
                                    .into()))
                                        .expect("[tx channel] failed sending message");
                                }
                                // We can't do anything with this message
                                Err(e) => {
                                    debug!("{:?}", &e);
                                }
                            }
                        }
                        Ok(_) => {
                            // Do nothing
                        }
                        Err(e) => {
                            error!("{:}", &e);
                            std::thread::sleep(std::time::Duration::from_secs(1));
                            break 'ws_session;
                        }
                    }

                    'sending_control_messages: loop {
                        if let Some(rx) = control_rx.as_ref() {
                            if let Ok(message) = rx.try_recv() {
                                let write = socket
                                    .write_message(Message::Binary(message.to_control_frame()));
                                match write {
                                    Ok(_) => debug!("→ {}", &message),
                                    Err(e) => {
                                        warn!(
                                            "Could not send control message '{}': {:?}",
                                            &message, &e
                                        )
                                    }
                                }
                            } else {
                                break 'sending_control_messages;
                            }
                        } else {
                            break 'sending_control_messages;
                        }
                    }
                }
            }
        }
    }
}

/// Open a byte channel, consume it endlessly and send parsed telemetry messages through another channel
///
/// * `telemetry_bytes_rx` - Receiver of a channel used to transport telemetry bytes (input).
/// * `telemetry_tx` - Sender of a channel used to transport structured telemetry messages (output).
/// * `control_rx` - Optional receiver of a channel used to transport structured control messages (input).
/// * `control_bytes_tx` - Optional sender of a channel used to transport control bytes (output).
/// * `sleep_duration` - Optional duration to wait when there are no more bytes to parse; if `None` then no sleep.
///
/// This is meant to be run in a dedicated thread.
pub fn gather_telemetry_from_bytes(
    telemetry_bytes_rx: Receiver<Vec<u8>>,
    telemetry_tx: Sender<TelemetryChannelType>,
    control_rx: Option<Receiver<ControlMessage>>,
    control_bytes_tx: Option<Sender<Vec<u8>>>,
    sleep_duration: Option<Duration>,
) -> ! {
    let mut telemetry_buffer = Vec::new();

    if control_rx.is_none() || control_bytes_tx.is_none() {
        warn!("Control messages will not be handled (optional sender/receiver were not provided)");
    }

    loop {
        // Check for new bytes from the telemetry bytes channel and handle them
        if let Ok(mut new_telemetry_bytes) = telemetry_bytes_rx.try_recv() {
            telemetry_buffer.append(&mut new_telemetry_bytes);
        }

        if !telemetry_buffer.is_empty() {
            match parse_telemetry_message(&telemetry_buffer) {
                // It worked! Let's extract the message and replace the buffer with the rest of the bytes
                Ok((rest, message)) => {
                    telemetry_tx
                        .send(Ok(message))
                        .expect("[telemetry tx channel] failed sending message");

                    telemetry_buffer = Vec::from(rest);
                }
                // Message was read but there was a CRC error
                Err(nom::Err::Failure(TelemetryError(
                    msg_bytes,
                    TelemetryErrorKind::CrcError { expected, computed },
                ))) => {
                    warn!("[CRC error]\texpected={}\tcomputed={}", expected, computed);

                    telemetry_tx
                        .send(Err(HighLevelError::CrcError { expected, computed }.into()))
                        .expect("[telemetry tx channel] failed sending message");

                    telemetry_buffer = telemetry_buffer.clone().split_off(msg_bytes.len());
                }
                // Message was built using an unsupported protocol version
                Err(nom::Err::Failure(TelemetryError(
                    msg_bytes,
                    TelemetryErrorKind::UnsupportedProtocolVersion {
                        maximum_supported,
                        found,
                    },
                ))) => {
                    warn!(
                        "[unsupported protocol version]\tmaximum_supported={}\tfound={}",
                        maximum_supported, found
                    );

                    telemetry_tx
                        .send(Err(HighLevelError::UnsupportedProtocolVersion {
                            maximum_supported,
                            found,
                        }
                        .into()))
                        .expect("[telemetry tx channel] failed sending message");

                    telemetry_buffer = telemetry_buffer.clone().split_off(msg_bytes.len());
                }
                // There are not enough bytes, let's wait until we get more
                Err(nom::Err::Incomplete(_)) => {
                    // Do nothing
                    if let Some(duration) = sleep_duration {
                        std::thread::sleep(duration);
                    }
                }
                // We can't do anything with the begining of the buffer, let's drop its first byte
                Err(e) => {
                    debug!("{:?}", &e);
                    telemetry_buffer.remove(0);
                }
            }
        } else if let Some(duration) = sleep_duration {
            std::thread::sleep(duration);
        }

        // Check for a new message from the structured control message channel and handle it
        if let (Some(rx), Some(tx)) = (control_rx.as_ref(), control_bytes_tx.as_ref()) {
            if let Ok(new_control_message) = rx.try_recv() {
                let new_control_bytes = new_control_message.to_control_frame();
                tx.send(new_control_bytes)
                    .expect("[control tx channel] failed sending bytes");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serializers::*;

    use ntest::timeout;
    use std::sync::mpsc::channel;

    const TELEMETRY_VERSION: u8 = 2;
    const VERSION: &str = "test";
    const DEVICE_ID: &str = "0-0-0";

    fn gen_fake_telemetry_messages() -> Vec<TelemetryMessage> {
        vec![
            TelemetryMessage::BootMessage(BootMessage {
                telemetry_version: TELEMETRY_VERSION,
                version: VERSION.to_owned(),
                device_id: DEVICE_ID.to_owned(),
                systick: 10,
                mode: Mode::Production,
                value128: 128,
            }),
            TelemetryMessage::ControlAck(ControlAck {
                telemetry_version: TELEMETRY_VERSION,
                version: VERSION.to_owned(),
                device_id: DEVICE_ID.to_owned(),
                systick: 50,
                setting: ControlSetting::PEEP,
                value: 0,
            }),
            TelemetryMessage::ControlAck(ControlAck {
                telemetry_version: TELEMETRY_VERSION,
                version: VERSION.to_owned(),
                device_id: DEVICE_ID.to_owned(),
                systick: 200,
                setting: ControlSetting::RespirationEnabled,
                value: 1,
            }),
            TelemetryMessage::DataSnapshot(DataSnapshot {
                telemetry_version: TELEMETRY_VERSION,
                version: VERSION.to_owned(),
                device_id: DEVICE_ID.to_owned(),
                systick: 1500,
                centile: 10,
                pressure: 200,
                phase: Phase::Inhalation,
                subphase: None,
                blower_valve_position: 35,
                patient_valve_position: 0,
                blower_rpm: 10,
                battery_level: 24,
                inspiratory_flow: Some(100),
                expiratory_flow: Some(0),
            }),
        ]
    }

    fn gen_fake_control_messages() -> Vec<ControlMessage> {
        vec![
            ControlMessage {
                setting: ControlSetting::Heartbeat,
                value: 1,
            },
            ControlMessage {
                setting: ControlSetting::PEEP,
                value: 0,
            },
            ControlMessage {
                setting: ControlSetting::RespirationEnabled,
                value: 1,
            },
            ControlMessage {
                setting: ControlSetting::Heartbeat,
                value: 1,
            },
        ]
    }

    #[test]
    #[timeout(2000)]
    fn gather_telemetry_from_bytes_works() {
        // Prepare telemetry messages
        let telemetry_messages = gen_fake_telemetry_messages();

        // Serialize these telemetry messages into binary
        let telemetry_bytes = telemetry_messages
            .iter()
            .flat_map(|m| mk_frame(&m.to_bytes()))
            .collect::<Vec<_>>();

        // Prepare control messages
        let control_messages = gen_fake_control_messages();

        // Serialize these control messages into binary
        let control_bytes = control_messages
            .iter()
            .flat_map(|m| m.to_control_frame())
            .collect::<Vec<_>>();

        // Prepare channels to communicate with the gather_telemetry* function
        let (telemetry_bytes_tx, telemetry_bytes_rx) = channel::<Vec<u8>>();
        let (telemetry_messages_tx, telemetry_messages_rx) = channel::<TelemetryChannelType>();
        let (control_bytes_tx, control_bytes_rx) = channel::<Vec<u8>>();
        let (control_messages_tx, control_messages_rx) = channel::<ControlMessage>();

        // Run the gather_telemetry* function in a thread (it will never terminate)
        std::thread::spawn(|| {
            gather_telemetry_from_bytes(
                telemetry_bytes_rx,
                telemetry_messages_tx,
                Some(control_messages_rx),
                Some(control_bytes_tx),
                None,
            )
        });

        // Send telemetry messages byte by byte
        for b in telemetry_bytes {
            telemetry_bytes_tx.send(vec![b]).unwrap();
        }

        // Send control messages
        for m in control_messages {
            control_messages_tx.send(m).unwrap();
        }

        // Wait to receive messages through one of the output channel
        let mut telemetry_messages_received = 0;
        let mut control_bytes_received: Vec<u8> = vec![];
        'check_received_messages: loop {
            // Check for telemetry messages
            if let Ok(msg) = telemetry_messages_rx.try_recv() {
                // Check that the message is right
                assert_eq!(
                    &msg.unwrap(),
                    telemetry_messages.get(telemetry_messages_received).unwrap()
                );

                // Mark this message a received
                telemetry_messages_received += 1;
            }

            // Check for control bytes
            if let Ok(mut msg) = control_bytes_rx.try_recv() {
                // Push bytes into the buffer
                control_bytes_received.append(&mut msg);
            }

            // If this was the last message expected, let's break out and end the test
            if telemetry_messages.len() == telemetry_messages_received
                && control_bytes_received == control_bytes
            {
                break 'check_received_messages;
            }
        }
    }
}
