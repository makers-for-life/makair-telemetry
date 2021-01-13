// MakAir Telemetry
//
// Copyright: 2020, Makers For Life
// License: Public Domain License

use std::cmp::{Ord, Ordering, PartialOrd};
use std::convert::TryFrom;
use std::io;

pub use crate::control::ControlSetting;

/// Variants of the MakAir firmware
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize-messages", derive(serde::Serialize))]
pub enum Mode {
    /// Production mode
    Production,
    /// (obsolete) Qualification mode
    Qualification,
    /// (obsolete) Integration test mode
    IntegrationTest,
}

/// Phases of the respiratory cycle
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize-messages", derive(serde::Serialize))]
pub enum Phase {
    /// Inhalation
    Inhalation,
    /// Exhalation
    Exhalation,
}

/// [obsolete in protocol v2] Sub-phases of the respiratory cycle
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize-messages", derive(serde::Serialize))]
pub enum SubPhase {
    /// Inspiration
    Inspiration,
    /// HoldInspiration
    HoldInspiration,
    /// Exhale
    Exhale,
}

/// Supported alarm priorities
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize-messages", derive(serde::Serialize))]
pub enum AlarmPriority {
    /// High
    High,
    /// Medium
    Medium,
    /// Low
    Low,
}

impl PartialOrd for AlarmPriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AlarmPriority {
    fn cmp(&self, other: &Self) -> Ordering {
        let priority_to_int = |priority: &Self| match priority {
            Self::High => 3,
            Self::Medium => 2,
            Self::Low => 1,
        };

        priority_to_int(self).cmp(&priority_to_int(other))
    }
}

impl TryFrom<u8> for AlarmPriority {
    type Error = io::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            10..=19 | 40..=49 => Ok(Self::High),
            20..=29 => Ok(Self::Medium),
            30..=39 => Ok(Self::Low),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid priority {}", value),
            )),
        }
    }
}

/// Supported ventilation modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serialize-messages", derive(serde::Serialize))]
#[allow(non_camel_case_types)]
pub enum VentilationMode {
    /// PC-CMV
    PC_CMV = 1,
    /// PC-AC (default)
    PC_AC = 2,
    /// VC-CMV
    VC_CMV = 3,
    /// PC-VSAI
    PC_VSAI = 4,
    /// VC-AC
    VC_AC = 5,
}

/// Ventilation mode class
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VentilationModeClass {
    /// PC
    Pressure,
    /// VC
    Volume,
}

/// Ventilation mode kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VentilationModeKind {
    /// CMV
    Cmv,
    /// AC
    Ac,
    /// VSAI
    Vsai,
}

impl TryFrom<u8> for VentilationMode {
    type Error = io::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::PC_CMV),
            2 => Ok(Self::PC_AC),
            3 => Ok(Self::VC_CMV),
            4 => Ok(Self::PC_VSAI),
            5 => Ok(Self::VC_AC),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid ventilation mode {}", value),
            )),
        }
    }
}

impl Default for VentilationMode {
    fn default() -> Self {
        Self::PC_AC
    }
}

impl From<&VentilationMode> for u8 {
    fn from(mode: &VentilationMode) -> u8 {
        *mode as u8
    }
}

impl VentilationMode {
    /// Get the class of the ventilation mode
    pub fn class(&self) -> VentilationModeClass {
        match self {
            Self::PC_CMV | Self::PC_AC | Self::PC_VSAI => VentilationModeClass::Pressure,
            Self::VC_CMV | Self::VC_AC => VentilationModeClass::Volume,
        }
    }

    /// Get the kind of the ventilation mode
    pub fn kind(&self) -> VentilationModeKind {
        match self {
            Self::PC_CMV | Self::VC_CMV => VentilationModeKind::Cmv,
            Self::PC_AC | Self::VC_AC => VentilationModeKind::Ac,
            Self::PC_VSAI => VentilationModeKind::Vsai,
        }
    }
}

/// Details of fatal errors
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize-messages", derive(serde::Serialize))]
pub enum FatalErrorDetails {
    /// MCU was restarted by watchdog
    WatchdogRestart,
    /// Calibration failed
    CalibrationError {
        /// Measured pressure offset in mmH2O
        pressure_offset: i16,
        /// Minimum presure measured during calibration in mmH2O
        min_pressure: i16,
        /// Maximum presure measured during calibration in mmH2O
        max_pressure: i16,
        /// Air flow measured at starting in cL/min (SLM * 100)
        flow_at_starting: Option<i16>,
        /// Air flow measured with blower ON in cL/min (SLM * 100)
        flow_with_blower_on: Option<i16>,
    },
    /// Battery is too discharged
    BatteryDeeplyDischarged {
        /// Battery level in centivolts
        battery_level: u16,
    },
    /// Could not read mass flow meter
    MassFlowMeterError,
    /// Read an inconsistent pressure
    InconsistentPressure {
        /// Measured pressure in mmH2O
        pressure: u16,
    },
}

/// Step of the end of line test
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serialize-messages", derive(serde::Serialize))]
#[allow(non_camel_case_types, missing_docs)]
pub enum EolTestStep {
    START,
    SUPPLY_TO_EXPANDER_NOT_CONNECTED,
    CHECK_FAN,
    TEST_BAT_DEAD,
    BATTERY_DEEP_DISCHARGE,
    DISCONNECT_MAINS,
    CONNECT_MAINS,
    CHECK_BUZZER,
    CHECK_ALL_BUTTONS,
    CHECK_UI_SCREEN,
    PLUG_AIR_TEST_SYTEM,
    REACH_MAX_PRESSURE,
    MAX_PRESSURE_REACHED_OK,
    MAX_PRESSURE_NOT_REACHED,
    START_LEAK_MESURE,
    LEAK_IS_TOO_HIGH,
    REACH_NULL_PRESSURE,
    MIN_PRESSURE_NOT_REACHED,
    USER_CONFIRMATION_BEFORE_O2_TEST,
    START_O2_TEST,
    O2_PRESSURE_NOT_REACH,
    WAIT_USER_BEFORE_LONG_RUN,
    START_LONG_RUN_BLOWER,
    PRESSURE_NOT_STABLE,
    FLOW_NOT_STABLE,
    END_SUCCESS,
    DISPLAY_PRESSURE,
    DISPLAY_FLOW,
}

impl TryFrom<u8> for EolTestStep {
    type Error = io::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::START),
            1 => Ok(Self::SUPPLY_TO_EXPANDER_NOT_CONNECTED),
            2 => Ok(Self::CHECK_FAN),
            3 => Ok(Self::TEST_BAT_DEAD),
            4 => Ok(Self::BATTERY_DEEP_DISCHARGE),
            5 => Ok(Self::DISCONNECT_MAINS),
            6 => Ok(Self::CONNECT_MAINS),
            7 => Ok(Self::CHECK_BUZZER),
            8 => Ok(Self::CHECK_ALL_BUTTONS),
            9 => Ok(Self::CHECK_UI_SCREEN),
            10 => Ok(Self::PLUG_AIR_TEST_SYTEM),
            11 => Ok(Self::REACH_MAX_PRESSURE),
            12 => Ok(Self::MAX_PRESSURE_REACHED_OK),
            13 => Ok(Self::MAX_PRESSURE_NOT_REACHED),
            14 => Ok(Self::START_LEAK_MESURE),
            15 => Ok(Self::LEAK_IS_TOO_HIGH),
            16 => Ok(Self::REACH_NULL_PRESSURE),
            17 => Ok(Self::MIN_PRESSURE_NOT_REACHED),
            18 => Ok(Self::USER_CONFIRMATION_BEFORE_O2_TEST),
            19 => Ok(Self::START_O2_TEST),
            20 => Ok(Self::O2_PRESSURE_NOT_REACH),
            21 => Ok(Self::WAIT_USER_BEFORE_LONG_RUN),
            22 => Ok(Self::START_LONG_RUN_BLOWER),
            23 => Ok(Self::PRESSURE_NOT_STABLE),
            24 => Ok(Self::FLOW_NOT_STABLE),
            25 => Ok(Self::END_SUCCESS),
            26 => Ok(Self::DISPLAY_PRESSURE),
            27 => Ok(Self::DISPLAY_FLOW),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid EOL test step {}", value),
            )),
        }
    }
}

/// Content of end of line test snapshots
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize-messages", derive(serde::Serialize))]
pub enum EolTestSnapshotContent {
    /// Test is in progress
    InProgress,
    /// There was an error during test
    Error(String),
    /// End of line test succeeded
    Success,
}

/// A telemetry message that is sent once every time the MCU boots
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize-messages", derive(serde::Serialize))]
pub struct BootMessage {
    /// Version of the telemetry protocol
    pub telemetry_version: u8,
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
#[cfg_attr(feature = "serialize-messages", derive(serde::Serialize))]
pub struct StoppedMessage {
    /// Version of the telemetry protocol
    pub telemetry_version: u8,
    /// Version of the MCU firmware
    pub version: String,
    /// Internal ID of the MCU
    pub device_id: String,
    /// Number of microseconds since the MCU booted
    pub systick: u64,
    /// [protocol v2] Requested peak command in cmH2O
    pub peak_command: Option<u8>,
    /// [protocol v2] Requested plateau command in cmH2O
    pub plateau_command: Option<u8>,
    /// [protocol v2] Requested PEEP command in cmH2O
    pub peep_command: Option<u8>,
    /// [protocol v2] Requested number of cycles per minute
    pub cpm_command: Option<u8>,
    /// [protocol v2] Expiration term in the "Inspiration/Expiration" ratio given that Inspiration = 10
    pub expiratory_term: Option<u8>,
    /// [protocol v2] State of the trigger
    pub trigger_enabled: Option<bool>,
    /// [protocol v2] Trigger offset in mmH2O
    pub trigger_offset: Option<u8>,
    /// [protocol v2] State of the alarm snooze
    pub alarm_snoozed: Option<bool>,
    /// [protocol v2] CPU load in percent
    pub cpu_load: Option<u8>,
    /// Ventilation mode
    pub ventilation_mode: VentilationMode,
    /// [protocol v2] Inspiratory trigger flow in percent
    pub inspiratory_trigger_flow: Option<u8>,
    /// [protocol v2] Expiratory trigger flow in percent
    pub expiratory_trigger_flow: Option<u8>,
    /// [protocol v2] Minimum duration of inhalation in ms
    pub ti_min: Option<u16>,
    /// [protocol v2] Maximum duration of inhalation in ms
    pub ti_max: Option<u16>,
    /// [protocol v2] Threshold for low inspiratory minute volume alarm in L/min
    pub low_inspiratory_minute_volume_alarm_threshold: Option<u8>,
    /// [protocol v2] Threshold for high inspiratory minute volume alarm in L/min
    pub high_inspiratory_minute_volume_alarm_threshold: Option<u8>,
    /// [protocol v2] Threshold for low expiratory minute volume alarm in L/min
    pub low_expiratory_minute_volume_alarm_threshold: Option<u8>,
    /// [protocol v2] Threshold for high expiratory minute volume alarm in L/min
    pub high_expiratory_minute_volume_alarm_threshold: Option<u8>,
    /// [protocol v2] Threshold for low respiratory rate alarm in cycle per minute
    pub low_respiratory_rate_alarm_threshold: Option<u8>,
    /// [protocol v2] Threshold for high respiratory rate alarm in cycle per minute
    pub high_respiratory_rate_alarm_threshold: Option<u8>,
    /// [protocol v2] Target tidal volume in mL
    pub target_tidal_volume: Option<u16>,
    /// [protocol v2] Threshold for low tidal volume in mL
    pub low_tidal_volume_alarm_threshold: Option<u16>,
    /// [protocol v2] Threshold for high tidal volume in mL
    pub high_tidal_volume_alarm_threshold: Option<u16>,
    /// [protocol v2] Duration in ms of closing both valves to effectively measure plateau pressure in volume control modes
    pub plateau_duration: Option<u16>,
    /// [protocol v2] Threshold for leak alarm in cL/min
    pub leak_alarm_threshold: Option<u16>,
    /// [protocol v2] Target flow during inspiration in L/min
    pub target_inspiratory_flow: Option<u8>,
    /// [protocol v2] Requested duration of inspiration in ms
    pub inspiratory_duration_command: Option<u16>,
    /// [protocol v2] Measured battery level value in centivolts (precise value)
    pub battery_level: Option<u16>,
    /// [protocol v2] Codes of the alarms that are currently triggered
    pub current_alarm_codes: Option<Vec<u8>>,
    /// [protocol v2] Patient's height in centimeters
    pub patient_height: Option<u8>,
}

/// A telemetry message that is sent every time the firmware does a control iteration (every 10 ms)
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize-messages", derive(serde::Serialize))]
pub struct DataSnapshot {
    /// Version of the telemetry protocol
    pub telemetry_version: u8,
    /// Version of the MCU firmware
    pub version: String,
    /// Internal ID of the MCU
    pub device_id: String,
    /// Number of microseconds since the MCU booted
    pub systick: u64,
    /// Number of hundredth of seconds since the begining of the current breathing cycle
    pub centile: u16,
    /// Current pressure in mmH2O (can be negative)
    ///
    /// _[protocol v2] Changed from u16 to i16 (values above i16::MAX will be assigned the value i16::MAX, but this should not happen)_
    pub pressure: i16,
    /// Current phase
    pub phase: Phase,
    /// [obsolete in protocol v2] Current sub-phase
    pub subphase: Option<SubPhase>,
    /// Current angle of the blower valve
    pub blower_valve_position: u8,
    /// Current angle of the patient valve
    pub patient_valve_position: u8,
    /// Current blower speed (no unit)
    pub blower_rpm: u8,
    /// Current battery level in volts (imprecise value)
    pub battery_level: u8,
    /// [protocol v2] Inspiratory flow in cL/min (SLM * 100)
    pub inspiratory_flow: Option<i16>,
    /// [protocol v2] Expiratory flow in cL/min (SLM * 100)
    pub expiratory_flow: Option<i16>,
}

/// A telemetry message that is sent at the end of every respiratory cycle
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize-messages", derive(serde::Serialize))]
pub struct MachineStateSnapshot {
    /// Version of the telemetry protocol
    pub telemetry_version: u8,
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
    /// Expiration term in the "Inspiration/Expiration" ratio given that Inspiration = 10
    pub expiratory_term: u8,
    /// State of the trigger
    pub trigger_enabled: bool,
    /// Trigger offset in mmH2O
    pub trigger_offset: u8,
    /// [protocol v2] Measured number of cycles per minute
    pub previous_cpm: Option<u8>,
    /// [protocol v2] State of the alarm snooze
    pub alarm_snoozed: Option<bool>,
    /// [protocol v2] CPU load in percent
    pub cpu_load: Option<u8>,
    /// Ventilation mode
    pub ventilation_mode: VentilationMode,
    /// [protocol v2] Inspiratory trigger flow in percent
    pub inspiratory_trigger_flow: Option<u8>,
    /// [protocol v2] Expiratory trigger flow in percent
    pub expiratory_trigger_flow: Option<u8>,
    /// [protocol v2] Minimum duration of inhalation in ms
    pub ti_min: Option<u16>,
    /// [protocol v2] Maximum duration of inhalation in ms
    pub ti_max: Option<u16>,
    /// [protocol v2] Threshold for low inspiratory minute volume alarm in L/min
    pub low_inspiratory_minute_volume_alarm_threshold: Option<u8>,
    /// [protocol v2] Threshold for high inspiratory minute volume alarm in L/min
    pub high_inspiratory_minute_volume_alarm_threshold: Option<u8>,
    /// [protocol v2] Threshold for low expiratory minute volume alarm in L/min
    pub low_expiratory_minute_volume_alarm_threshold: Option<u8>,
    /// [protocol v2] Threshold for high expiratory minute volume alarm in L/min
    pub high_expiratory_minute_volume_alarm_threshold: Option<u8>,
    /// [protocol v2] Threshold for low respiratory rate alarm in cycle per minute
    pub low_respiratory_rate_alarm_threshold: Option<u8>,
    /// [protocol v2] Threshold for high respiratory rate alarm in cycle per minute
    pub high_respiratory_rate_alarm_threshold: Option<u8>,
    /// [protocol v2] Target tidal volume in mL
    pub target_tidal_volume: Option<u16>,
    /// [protocol v2] Threshold for low tidal volume in mL
    pub low_tidal_volume_alarm_threshold: Option<u16>,
    /// [protocol v2] Threshold for high tidal volume in mL
    pub high_tidal_volume_alarm_threshold: Option<u16>,
    /// [protocol v2] Duration in ms of closing both valves to effectively measure plateau pressure in volume control modes
    pub plateau_duration: Option<u16>,
    /// [protocol v2] Threshold for leak alarm in cL/min
    pub leak_alarm_threshold: Option<u16>,
    /// [protocol v2] Target flow during inspiration in L/min
    pub target_inspiratory_flow: Option<u8>,
    /// [protocol v2] Requested duration of inspiration in ms
    pub inspiratory_duration_command: Option<u16>,
    /// [protocol v2] Measured duration of inspiration in ms
    pub previous_inspiratory_duration: Option<u16>,
    /// [protocol v2] Measured battery level value in centivolts (precise value)
    pub battery_level: Option<u16>,
    /// [protocol v2] Patient's height in centimeters
    pub patient_height: Option<u8>,
}

/// A telemetry message that is sent every time an alarm is triggered or stopped
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize-messages", derive(serde::Serialize))]
pub struct AlarmTrap {
    /// Version of the telemetry protocol
    pub telemetry_version: u8,
    /// Version of the MCU firmware
    pub version: String,
    /// Internal ID of the MCU
    pub device_id: String,
    /// Number of microseconds since the MCU booted
    pub systick: u64,
    /// Number of hundredth of seconds since the begining of the current breathing cycle
    pub centile: u16,
    /// Current pressure in mmH2O (can be negative)
    ///
    /// _[protocol v2] Changed from u16 to i16 (values above i16::MAX will be assigned the value i16::MAX, but this should not happen)_
    pub pressure: i16,
    /// Current phase
    pub phase: Phase,
    /// [obsolete in protocol v2] Current sub-phase
    pub subphase: Option<SubPhase>,
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

/// An ACK message that is sent every time a setting is changed on the MCU side
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize-messages", derive(serde::Serialize))]
pub struct ControlAck {
    /// Version of the telemetry protocol
    pub telemetry_version: u8,
    /// Version of the MCU firmware
    pub version: String,
    /// Internal ID of the MCU
    pub device_id: String,
    /// Number of microseconds since the MCU booted
    pub systick: u64,
    /// Setting that was changed
    pub setting: ControlSetting,
    /// New value
    pub value: u16,
}

/// [protocol v2] A message sent when a fatal error occurs
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize-messages", derive(serde::Serialize))]
pub struct FatalError {
    /// Version of the telemetry protocol
    pub telemetry_version: u8,
    /// Version of the MCU firmware
    pub version: String,
    /// Internal ID of the MCU
    pub device_id: String,
    /// Number of microseconds since the MCU booted
    pub systick: u64,
    /// Details of the error
    pub error: FatalErrorDetails,
}

/// [protocol v2] A message sent during end of line tests
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize-messages", derive(serde::Serialize))]
pub struct EolTestSnapshot {
    /// Version of the telemetry protocol
    pub telemetry_version: u8,
    /// Version of the MCU firmware
    pub version: String,
    /// Internal ID of the MCU
    pub device_id: String,
    /// Number of microseconds since the MCU booted
    pub systick: u64,
    /// Current step
    pub current_step: EolTestStep,
    /// Content of the snapshot
    pub content: EolTestSnapshotContent,
}

/// Supported telemetry messages
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize-messages", derive(serde::Serialize))]
#[cfg_attr(feature = "serialize-messages", serde(tag = "message_type"))]
pub enum TelemetryMessage {
    /// A telemetry message that is sent once every time the MCU boots
    BootMessage(BootMessage),
    /// A telemetry message that is sent every 100 ms when the MCU is in "stop" mode
    StoppedMessage(StoppedMessage),
    /// A telemetry message that is sent every time the firmware does a control iteration (every 10 ms)
    DataSnapshot(DataSnapshot),
    /// A telemetry message that is sent at the end of every respiratory cycle
    MachineStateSnapshot(MachineStateSnapshot),
    /// A telemetry message that is sent every time an alarm is triggered or stopped
    AlarmTrap(AlarmTrap),
    /// An ACK message that is sent every time a setting is changed using the control protocol
    ControlAck(ControlAck),
    /// [protocol v2] A message sent when a fatal error occurs
    FatalError(FatalError),
    /// [protocol v2] A message sent during end of line tests
    EolTestSnapshot(EolTestSnapshot),
}

impl TelemetryMessage {
    /// Version of the telemetry protocol
    pub fn telemetry_version(&self) -> u8 {
        let val = match self {
            Self::BootMessage(BootMessage {
                telemetry_version, ..
            }) => telemetry_version,
            Self::StoppedMessage(StoppedMessage {
                telemetry_version, ..
            }) => telemetry_version,
            Self::DataSnapshot(DataSnapshot {
                telemetry_version, ..
            }) => telemetry_version,
            Self::MachineStateSnapshot(MachineStateSnapshot {
                telemetry_version, ..
            }) => telemetry_version,
            Self::AlarmTrap(AlarmTrap {
                telemetry_version, ..
            }) => telemetry_version,
            Self::ControlAck(ControlAck {
                telemetry_version, ..
            }) => telemetry_version,
            Self::FatalError(FatalError {
                telemetry_version, ..
            }) => telemetry_version,
            Self::EolTestSnapshot(EolTestSnapshot {
                telemetry_version, ..
            }) => telemetry_version,
        };
        *val
    }

    /// Version of the MCU firmware
    pub fn version(&self) -> String {
        let val = match self {
            Self::BootMessage(BootMessage { version, .. }) => version,
            Self::StoppedMessage(StoppedMessage { version, .. }) => version,
            Self::DataSnapshot(DataSnapshot { version, .. }) => version,
            Self::MachineStateSnapshot(MachineStateSnapshot { version, .. }) => version,
            Self::AlarmTrap(AlarmTrap { version, .. }) => version,
            Self::ControlAck(ControlAck { version, .. }) => version,
            Self::FatalError(FatalError { version, .. }) => version,
            Self::EolTestSnapshot(EolTestSnapshot { version, .. }) => version,
        };
        val.clone()
    }

    /// Internal ID of the MCU
    pub fn device_id(&self) -> String {
        let val = match self {
            Self::BootMessage(BootMessage { device_id, .. }) => device_id,
            Self::StoppedMessage(StoppedMessage { device_id, .. }) => device_id,
            Self::DataSnapshot(DataSnapshot { device_id, .. }) => device_id,
            Self::MachineStateSnapshot(MachineStateSnapshot { device_id, .. }) => device_id,
            Self::AlarmTrap(AlarmTrap { device_id, .. }) => device_id,
            Self::ControlAck(ControlAck { device_id, .. }) => device_id,
            Self::FatalError(FatalError { device_id, .. }) => device_id,
            Self::EolTestSnapshot(EolTestSnapshot { device_id, .. }) => device_id,
        };
        val.clone()
    }

    /// Number of microseconds since the MCU booted
    pub fn systick(&self) -> u64 {
        let val = match self {
            Self::BootMessage(BootMessage { systick, .. }) => systick,
            Self::StoppedMessage(StoppedMessage { systick, .. }) => systick,
            Self::DataSnapshot(DataSnapshot { systick, .. }) => systick,
            Self::MachineStateSnapshot(MachineStateSnapshot { systick, .. }) => systick,
            Self::AlarmTrap(AlarmTrap { systick, .. }) => systick,
            Self::ControlAck(ControlAck { systick, .. }) => systick,
            Self::FatalError(FatalError { systick, .. }) => systick,
            Self::EolTestSnapshot(EolTestSnapshot { systick, .. }) => systick,
        };
        *val
    }
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
    /// Unsupported protocol (message header contains an unsupported protocol version)
    UnsupportedProtocolVersion {
        /// Maximum supported version of the telemetry protocol
        maximum_supported: u8,
        /// Found version of the telemetry protocol
        found: u8,
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

impl<I> From<nom::error::Error<I>> for TelemetryError<I> {
    fn from(error: nom::error::Error<I>) -> Self {
        TelemetryError(error.input, TelemetryErrorKind::ParserError(error.code))
    }
}

/// Errors that need to be reported to the UI
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize-messages", derive(serde::Serialize))]
pub enum HighLevelError {
    /// CRC error
    CrcError {
        /// Expected CRC (included in the message)
        expected: u32,
        /// Computed CRC (from the actual message)
        computed: u32,
    },
    /// Unsupported protocol (message header contains an unsupported protocol version)
    UnsupportedProtocolVersion {
        /// Maximum supported version of the telemetry protocol
        maximum_supported: u8,
        /// Found version of the telemetry protocol
        found: u8,
    },
}

/// A telemetry message or a high-level error
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize-messages", derive(serde::Serialize))]
pub enum TelemetryMessageOrError {
    /// A telemetry message
    Message(TelemetryMessage),
    /// A high-level error
    Error(HighLevelError),
}

impl From<TelemetryMessage> for TelemetryMessageOrError {
    fn from(message: TelemetryMessage) -> Self {
        TelemetryMessageOrError::Message(message)
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
