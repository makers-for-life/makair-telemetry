// MakAir Telemetry
//
// Copyright: 2020, Makers For Life
// License: Public Domain License

use rand::distributions::{Distribution, Standard};
use rand::Rng;
use std::convert::TryFrom;
use std::ops::RangeInclusive;

/// Special value that can be used in a heartbeat control message to disable RPi watchdog
pub const DISABLE_RPI_WATCHDOG: u16 = 43_690;

/// Available settings in the control protocol
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize-messages", derive(serde::Serialize))]
pub enum ControlSetting {
    /// Heartbeat used for the RPi watchdog feature (value is ignored except for the special value `DISABLE_RPI_WATCHDOG` which disables watchdog)
    Heartbeat = 0,
    /// Ventilation mode, must be one of the following:
    /// - `1` → PC-CMV (default)
    /// - `2` → PC-AC
    /// - `3` → VC-CMV
    /// - `4` → PC-VSAI
    /// - `5` → VC-AC
    VentilationMode = 1,
    /// Plateau pressure in mmH2O (value bounds must be between 100 and 400)
    PlateauPressure = 2,
    /// PEEP in mmH2O (value bounds must be between 0 and 300)
    PEEP = 3,
    /// Number of cycles per minute (value bounds must be between 5 and 35)
    CyclesPerMinute = 4,
    /// Expiration term in the "Inspiration/Expiration" ratio given that Inspiration = 10 (value bounds must be between 10 and 60)
    ExpiratoryTerm = 5,
    /// State of the trigger (value must be 1 if enabled and 0 if disabled)
    TriggerEnabled = 6,
    /// Trigger offset in mmH2O (value bounds must be between 0 and 100)
    TriggerOffset = 7,
    /// State of the respiration (value must be 1 if enabled and 0 if disabled)
    RespirationEnabled = 8,
    /// Alarm snooze (value must be 1 to snooze and 0 to unsnooze)
    AlarmSnooze = 9,
    /// Inspiratory trigger flow in percent
    InspiratoryTriggerFlow = 10,
    /// Expiratory trigger flow in percent
    ExpiratoryTriggerFlow = 11,
    /// Minimum duration of inhalation in ms (value bounds must be between 100 and 3000)
    TiMin = 12,
    /// Maximum duration of inhalation in ms (value bounds must be between 200 and 5000)
    TiMax = 13,
    /// Threshold for low inspiratory minute volume alarm in L/min (value bounds must be between 0 and 20)
    LowInspiratoryMinuteVolumeAlarmThreshold = 14,
    /// Threshold for high inspiratory minute volume alarm in L/min (value bounds must be between 10 and 40)
    HighInspiratoryMinuteVolumeAlarmThreshold = 15,
    /// Threshold for low expiratory minute volume alarm in L/min (value bounds must be between 0 and 20)
    LowExpiratoryMinuteVolumeAlarmThreshold = 16,
    /// Threshold for high expiratory minute volume alarm in L/min (value bounds must be between 10 and 40)
    HighExpiratoryMinuteVolumeAlarmThreshold = 17,
    /// Threshold for low expiratory rate alarm in cycle per minute (value bounds must be between 5 and 25)
    LowExpiratoryRateAlarmThreshold = 18,
    /// Threshold for high expiratory rate alarm in cycle per minute (value bounds must be between 20 and 35)
    HighExpiratoryRateAlarmThreshold = 19,
    /// Target tidal volume in mL (value bounds must be between 50 and 2000)
    TargetTidalVolume = 20,
    /// Threshold for low tidal volume in mL (value bounds must be between 0 and 1000)
    LowTidalVolumeAlarmTreshold = 21,
    /// Threshold for high tidal volume in mL (value bounds must be between 50 and 2000)
    HighTidalVolumeAlarmTreshold = 22,
    /// Duration in ms of closing both valves to effectively measure plateau pressure in volume control modes (value bounds must be between 100 and 2000)
    PlateauDuration = 23,
    /// Threshold for leak alarm in cL/min (value bounds must be between 0 and 10000)
    LeakAlarmThreshold = 24,
    /// Target flow during inspiration in L/min (value bounds must be between 5 and 80)
    TargetInspiratoryFlow = 25,
    /// Duration of inspiration in ms (value bounds must be between 200 and 3000)
    InspiratoryDuration = 26,
}

impl ControlSetting {
    /// Default settings
    pub fn default(&self) -> usize {
        // Returns default value
        match self {
            Self::Heartbeat => 0,
            Self::VentilationMode => 1,
            Self::PlateauPressure => 0,
            Self::PEEP => 0,
            Self::CyclesPerMinute => 20,
            Self::ExpiratoryTerm => 20,
            Self::TriggerEnabled => 0,
            Self::TriggerOffset => 20,
            Self::RespirationEnabled => 0,
            Self::AlarmSnooze => 0,
            Self::InspiratoryTriggerFlow => 10,
            Self::ExpiratoryTriggerFlow => 30,
            Self::TiMin => 200,
            Self::TiMax => 1_000,
            Self::LowInspiratoryMinuteVolumeAlarmThreshold => 3,
            Self::HighInspiratoryMinuteVolumeAlarmThreshold => 20,
            Self::LowExpiratoryMinuteVolumeAlarmThreshold => 3,
            Self::HighExpiratoryMinuteVolumeAlarmThreshold => 20,
            Self::LowExpiratoryRateAlarmThreshold => 10,
            Self::HighExpiratoryRateAlarmThreshold => 30,
            Self::TargetTidalVolume => 400,
            Self::LowTidalVolumeAlarmTreshold => 200,
            Self::HighTidalVolumeAlarmTreshold => 1_000,
            Self::PlateauDuration => 200,
            Self::LeakAlarmThreshold => 200,
            Self::TargetInspiratoryFlow => 40,
            Self::InspiratoryDuration => 800,
        }
    }

    /// Allowed value bounds per setting
    pub fn bounds(&self) -> RangeInclusive<usize> {
        // Returns allowed value bounds
        match self {
            Self::Heartbeat => RangeInclusive::new(0, 255),
            Self::VentilationMode => RangeInclusive::new(1, 5),
            Self::PlateauPressure => RangeInclusive::new(100, 400),
            Self::PEEP => RangeInclusive::new(0, 300),
            Self::CyclesPerMinute => RangeInclusive::new(5, 35),
            Self::ExpiratoryTerm => RangeInclusive::new(10, 60),
            Self::TriggerEnabled => RangeInclusive::new(0, 1),
            Self::TriggerOffset => RangeInclusive::new(0, 100),
            Self::RespirationEnabled => RangeInclusive::new(0, 1),
            Self::AlarmSnooze => RangeInclusive::new(0, 1),
            Self::InspiratoryTriggerFlow => RangeInclusive::new(0, 100),
            Self::ExpiratoryTriggerFlow => RangeInclusive::new(0, 100),
            Self::TiMin => RangeInclusive::new(100, 3_000),
            Self::TiMax => RangeInclusive::new(200, 5_000),
            Self::LowInspiratoryMinuteVolumeAlarmThreshold => RangeInclusive::new(0, 20),
            Self::HighInspiratoryMinuteVolumeAlarmThreshold => RangeInclusive::new(10, 40),
            Self::LowExpiratoryMinuteVolumeAlarmThreshold => RangeInclusive::new(0, 20),
            Self::HighExpiratoryMinuteVolumeAlarmThreshold => RangeInclusive::new(10, 40),
            Self::LowExpiratoryRateAlarmThreshold => RangeInclusive::new(5, 25),
            Self::HighExpiratoryRateAlarmThreshold => RangeInclusive::new(20, 35),
            Self::TargetTidalVolume => RangeInclusive::new(50, 2_000),
            Self::LowTidalVolumeAlarmTreshold => RangeInclusive::new(0, 1_000),
            Self::HighTidalVolumeAlarmTreshold => RangeInclusive::new(50, 2_000),
            Self::PlateauDuration => RangeInclusive::new(100, 2_000),
            Self::LeakAlarmThreshold => RangeInclusive::new(0, 10_000),
            Self::TargetInspiratoryFlow => RangeInclusive::new(5, 80),
            Self::InspiratoryDuration => RangeInclusive::new(200, 3_000),
        }
    }
}

impl std::convert::TryFrom<u8> for ControlSetting {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ControlSetting::Heartbeat),
            1 => Ok(ControlSetting::VentilationMode),
            2 => Ok(ControlSetting::PlateauPressure),
            3 => Ok(ControlSetting::PEEP),
            4 => Ok(ControlSetting::CyclesPerMinute),
            5 => Ok(ControlSetting::ExpiratoryTerm),
            6 => Ok(ControlSetting::TriggerEnabled),
            7 => Ok(ControlSetting::TriggerOffset),
            8 => Ok(ControlSetting::RespirationEnabled),
            9 => Ok(ControlSetting::AlarmSnooze),
            10 => Ok(ControlSetting::InspiratoryTriggerFlow),
            11 => Ok(ControlSetting::ExpiratoryTriggerFlow),
            12 => Ok(ControlSetting::TiMin),
            13 => Ok(ControlSetting::TiMax),
            14 => Ok(ControlSetting::LowInspiratoryMinuteVolumeAlarmThreshold),
            15 => Ok(ControlSetting::HighInspiratoryMinuteVolumeAlarmThreshold),
            16 => Ok(ControlSetting::LowExpiratoryMinuteVolumeAlarmThreshold),
            17 => Ok(ControlSetting::HighExpiratoryMinuteVolumeAlarmThreshold),
            18 => Ok(ControlSetting::LowExpiratoryRateAlarmThreshold),
            19 => Ok(ControlSetting::HighExpiratoryRateAlarmThreshold),
            20 => Ok(ControlSetting::TargetTidalVolume),
            21 => Ok(ControlSetting::LowTidalVolumeAlarmTreshold),
            22 => Ok(ControlSetting::HighTidalVolumeAlarmTreshold),
            23 => Ok(ControlSetting::PlateauDuration),
            24 => Ok(ControlSetting::LeakAlarmThreshold),
            25 => Ok(ControlSetting::TargetInspiratoryFlow),
            26 => Ok(ControlSetting::InspiratoryDuration),
            _ => Err("Invalid setting number"),
        }
    }
}

impl Distribution<ControlSetting> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ControlSetting {
        let number = rng.gen_range(1, 6);
        ControlSetting::try_from(number).unwrap()
    }
}

/// A control message
#[derive(Debug, Clone)]
pub struct ControlMessage {
    /// The setting to change
    pub setting: ControlSetting,
    /// The new value of the setting
    pub value: u16,
}

impl Distribution<ControlMessage> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ControlMessage {
        let setting: ControlSetting = rng.gen();
        let value = match setting {
            ControlSetting::Heartbeat => rng.gen_range(0, 255),
            ControlSetting::VentilationMode => rng.gen_range(0, 6),
            ControlSetting::PlateauPressure => rng.gen_range(10, 41),
            ControlSetting::PEEP => rng.gen_range(0, 31),
            ControlSetting::CyclesPerMinute => rng.gen_range(5, 36),
            ControlSetting::ExpiratoryTerm => rng.gen_range(10, 61),
            ControlSetting::TriggerEnabled => rng.gen_range(0, 2),
            ControlSetting::TriggerOffset => rng.gen_range(0, 101),
            ControlSetting::RespirationEnabled => rng.gen_range(0, 2),
            ControlSetting::AlarmSnooze => rng.gen_range(0, 2),
            ControlSetting::InspiratoryTriggerFlow => rng.gen_range(0, 101),
            ControlSetting::ExpiratoryTriggerFlow => rng.gen_range(0, 101),
            ControlSetting::TiMin => rng.gen_range(100, 3_001),
            ControlSetting::TiMax => rng.gen_range(200, 5_001),
            ControlSetting::LowInspiratoryMinuteVolumeAlarmThreshold => rng.gen_range(0, 21),
            ControlSetting::HighInspiratoryMinuteVolumeAlarmThreshold => rng.gen_range(10, 41),
            ControlSetting::LowExpiratoryMinuteVolumeAlarmThreshold => rng.gen_range(0, 21),
            ControlSetting::HighExpiratoryMinuteVolumeAlarmThreshold => rng.gen_range(10, 41),
            ControlSetting::LowExpiratoryRateAlarmThreshold => rng.gen_range(5, 26),
            ControlSetting::HighExpiratoryRateAlarmThreshold => rng.gen_range(20, 36),
            ControlSetting::TargetTidalVolume => rng.gen_range(50, 2_001),
            ControlSetting::LowTidalVolumeAlarmTreshold => rng.gen_range(0, 1_001),
            ControlSetting::HighTidalVolumeAlarmTreshold => rng.gen_range(50, 2_001),
            ControlSetting::PlateauDuration => rng.gen_range(100, 2_001),
            ControlSetting::LeakAlarmThreshold => rng.gen_range(0, 10_001),
            ControlSetting::TargetInspiratoryFlow => rng.gen_range(5, 81),
            ControlSetting::InspiratoryDuration => rng.gen_range(200, 3_001),
        };
        ControlMessage { setting, value }
    }
}

impl std::fmt::Display for ControlMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} = {}", self.setting, self.value)
    }
}

fn flat(v: &[&[u8]]) -> Vec<u8> {
    v.iter().flat_map(|a| a.iter()).copied().collect()
}

impl ControlMessage {
    fn to_bytes(&self) -> Vec<u8> {
        flat(&[&[self.setting as u8], &self.value.to_be_bytes()])
    }

    fn crc(&self) -> u32 {
        let mut crc = crc32fast::Hasher::new();
        crc.update(&self.to_bytes());
        crc.finalize()
    }

    /// Create a frame to be sent trough serial port
    ///
    /// This converts message to binary and adds header, footer and CRC
    ///
    /// * `force_crc` - CRC value to be used; will be computed if not specified.
    pub fn to_control_frame_with(&self, force_crc: Option<u32>) -> Vec<u8> {
        flat(&[
            b"\x05\x0A",
            &self.to_bytes(),
            &force_crc.unwrap_or_else(|| self.crc()).to_be_bytes(),
            b"\x50\xA0",
        ])
    }

    /// Create a frame to be sent trough serial port
    ///
    /// This converts message to binary and adds header, footer and CRC
    pub fn to_control_frame(&self) -> Vec<u8> {
        self.to_control_frame_with(None)
    }
}
