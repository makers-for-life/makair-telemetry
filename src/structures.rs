// MakAir
//
// Copyright: 2020, Makers For Life
// License: Public Domain License

use std::cmp::{Ord, Ordering, PartialOrd};
use std::convert::TryFrom;
use std::io;

/// Variants of the MakAir firmware
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Production,
    Qualification,
    IntegrationTest,
}

/// Phases of the respiratory cycle
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Phase {
    Inhalation,
    Exhalation,
}

/// Sub-phases of the respiratory cycle
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubPhase {
    Inspiration,
    HoldInspiration,
    Exhale,
}

/// Supported alarm priorities
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlarmPriority {
    High,
    Medium,
    Low,
}

impl PartialOrd for AlarmPriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AlarmPriority {
    fn cmp(&self, other: &Self) -> Ordering {
        let priority_to_int = |priority: &AlarmPriority| match priority {
            AlarmPriority::High => 3,
            AlarmPriority::Medium => 2,
            AlarmPriority::Low => 1,
        };

        priority_to_int(self).cmp(&priority_to_int(other))
    }
}

impl TryFrom<u8> for AlarmPriority {
    type Error = io::Error;

    fn try_from(value: u8) -> Result<AlarmPriority, Self::Error> {
        match value {
            10..=19 => Ok(AlarmPriority::High),
            20..=29 => Ok(AlarmPriority::Medium),
            30..=39 => Ok(AlarmPriority::Low),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid priority {}", value),
            )),
        }
    }
}

/// A telemetry message that is sent once every time the MCU boots
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootMessage {
    /// Version of the MCU firmware
    pub version: String,
    /// Internal ID of the MCU
    pub device_id: String,
    /// Number of microseconds since the MCU booted
    pub systick: u64,
    /// Firmware variant currently flashed
    pub mode: Mode,
    /// The number "128"
    ///
    /// This is only used to make sure that serial port was correctly opened and that there is no endianness problem.
    pub value128: u8,
}

/// A telemetry message that is sent every 100 ms when the MCU is in "stop" mode
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoppedMessage {
    /// Version of the MCU firmware
    pub version: String,
    /// Internal ID of the MCU
    pub device_id: String,
    /// Number of microseconds since the MCU booted
    pub systick: u64,
}

/// A telemetry message that is sent every time the firmware does a control iteration (every 10 ms)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataSnapshot {
    /// Version of the MCU firmware
    pub version: String,
    /// Internal ID of the MCU
    pub device_id: String,
    /// Number of microseconds since the MCU booted
    pub systick: u64,
    /// Number of hundredth of seconds since the begining of the current breathing cycle
    pub centile: u16,
    /// Current pressure in mmH2O
    pub pressure: u16,
    /// Current phase
    pub phase: Phase,
    /// Current sub-phase
    pub subphase: SubPhase,
    /// Current angle of the blower valve
    pub blower_valve_position: u8,
    /// Current angle of the patient valve
    pub patient_valve_position: u8,
    /// Current blower speed (no unit)
    pub blower_rpm: u8,
    /// Current battery level in volts
    pub battery_level: u8,
}

/// A telemetry message that is sent at the end of every respiratory cycle
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MachineStateSnapshot {
    /// Version of the MCU firmware
    pub version: String,
    /// Internal ID of the MCU
    pub device_id: String,
    /// Number of microseconds since the MCU booted
    pub systick: u64,
    /// Number of the current breathing cycle since MCU booted
    pub cycle: u32,
    /// Requested peak command in cmH2O
    pub peak_command: u8,
    /// Requested plateau command in cmH2O
    pub plateau_command: u8,
    /// Requested PEEP command in cmH2O
    pub peep_command: u8,
    /// Requested number of cycles per minute
    pub cpm_command: u8,
    /// Measured peak pressure in mmH2O
    pub previous_peak_pressure: u16,
    /// Measured pleateau pressure in mmH2O
    pub previous_plateau_pressure: u16,
    /// Measured PEEP in mmH2O
    pub previous_peep_pressure: u16,
    /// Codes of the alarms that are currently triggered
    pub current_alarm_codes: Vec<u8>,
    /// Measured previous_volume in mL (sensor might not be enabled)
    pub previous_volume: Option<u16>,
}

/// A telemetry message that is sent every time an alarm is triggered or stopped
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlarmTrap {
    /// Version of the MCU firmware
    pub version: String,
    /// Internal ID of the MCU
    pub device_id: String,
    /// Number of microseconds since the MCU booted
    pub systick: u64,
    /// Number of hundredth of seconds since the begining of the current breathing cycle
    pub centile: u16,
    /// Current pressure in mmH2O
    pub pressure: u16,
    /// Current phase
    pub phase: Phase,
    /// Current sub-phase
    pub subphase: SubPhase,
    /// Number of the current breathing cycle since MCU booted
    pub cycle: u32,
    /// Code of the alarm
    pub alarm_code: u8,
    /// Priority level of the alarm
    pub alarm_priority: AlarmPriority,
    /// `true` if alarm was triggered, `false` if it was stopped
    pub triggered: bool,
    /// Expected value (unit depends on the alarm)
    pub expected: u32,
    /// Measured value (unit depends on the alarm)
    pub measured: u32,
    /// Number of cycle for which this alarm has been triggered
    pub cycles_since_trigger: u32,
}

/// Supported telemetry messages
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TelemetryMessage {
    BootMessage(BootMessage),
    StoppedMessage(StoppedMessage),
    DataSnapshot(DataSnapshot),
    MachineStateSnapshot(MachineStateSnapshot),
    AlarmTrap(AlarmTrap),
}

/// Extension of Nom's `ErrorKind` to be able to represent CRC errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TelemetryErrorKind {
    /// Standard Nom error
    ParserError(nom::error::ErrorKind),
    /// CRC error
    CrcError {
        /// Expected CRC (included in the message)
        expected: u32,
        /// Computed CRC (from the actual message)
        computed: u32,
    },
}

/// Custom parser error type to leverage `TelemetryErrorKind`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelemetryError<I>(pub I, pub TelemetryErrorKind);

impl<I> nom::error::ParseError<I> for TelemetryError<I> {
    fn from_error_kind(input: I, kind: nom::error::ErrorKind) -> Self {
        TelemetryError(input, TelemetryErrorKind::ParserError(kind))
    }
    fn append(_: I, _: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

impl<I> From<(I, nom::error::ErrorKind)> for TelemetryError<I> {
    fn from(error: (I, nom::error::ErrorKind)) -> Self {
        TelemetryError(error.0, TelemetryErrorKind::ParserError(error.1))
    }
}

#[cfg(test)]
mod tests {
    use crate::structures::AlarmPriority;
    use std::cmp::Ordering;

    #[test]
    fn order_alarm_priority() {
        let high = AlarmPriority::High;
        let medium = AlarmPriority::Medium;
        let low = AlarmPriority::Low;

        // equal
        assert_eq!(high.cmp(&high), Ordering::Equal);
        assert_eq!(medium.cmp(&medium), Ordering::Equal);
        assert_eq!(low.cmp(&low), Ordering::Equal);

        // lower
        assert_eq!(medium.cmp(&high), Ordering::Less);
        assert_eq!(low.cmp(&high), Ordering::Less);
        assert_eq!(low.cmp(&medium), Ordering::Less);

        // greater
        assert_eq!(high.cmp(&medium), Ordering::Greater);
        assert_eq!(high.cmp(&low), Ordering::Greater);
        assert_eq!(medium.cmp(&low), Ordering::Greater);
    }
}
