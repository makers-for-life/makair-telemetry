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
    /// Peak pressure in mmH20 (value bounds must be between 0 and 700)
    PeakPressure = 1,
    /// Plateau pressure in mmH2O (value bounds must be between 100 and 400)
    PlateauPressure = 2,
    /// PEEP in mmH2O (value bounds must be between 0 and 300)
    PEEP = 3,
    /// Number of cycles per minute (value bounds must be between 5 and 35)
    CyclesPerMinute = 4,
    /// Expiration term in the "Inspiration/Expiration" ratio given that Inspiration = 10 (value \
    //    bounds must be between 10 and 60)
    ExpiratoryTerm = 5,
    /// State of the trigger (value must be 1 if enabled and 0 if disabled)
    TriggerEnabled = 6,
    /// Trigger offset in mmH2O (value bounds must be between 0 and 100)
    TriggerOffset = 7,
    /// State of the respiration (value must be 1 if enabled and 0 if disabled)
    RespirationEnabled = 8,
    /// Alarm snooze (value must be 0)
    AlarmSnooze = 9,
}

impl ControlSetting {
    /// Default settings
    pub fn default(&self) -> usize {
        // Returns default value
        match self {
            Self::Heartbeat => 0,
            Self::PeakPressure => 0,
            Self::PlateauPressure => 0,
            Self::PEEP => 0,
            Self::CyclesPerMinute => 20,
            Self::ExpiratoryTerm => 20,
            Self::TriggerEnabled => 0,
            Self::TriggerOffset => 20,
            Self::RespirationEnabled => 0,
            Self::AlarmSnooze => 0,
        }
    }

    /// Allowed value bounds per setting
    pub fn bounds(&self) -> RangeInclusive<usize> {
        // Returns allowed value bounds
        match self {
            Self::Heartbeat => RangeInclusive::new(0, 255),
            Self::PeakPressure => RangeInclusive::new(0, 700),
            Self::PlateauPressure => RangeInclusive::new(100, 400),
            Self::PEEP => RangeInclusive::new(0, 300),
            Self::CyclesPerMinute => RangeInclusive::new(5, 35),
            Self::ExpiratoryTerm => RangeInclusive::new(10, 60),
            Self::TriggerEnabled => RangeInclusive::new(0, 1),
            Self::TriggerOffset => RangeInclusive::new(0, 100),
            Self::RespirationEnabled => RangeInclusive::new(0, 1),
            Self::AlarmSnooze => RangeInclusive::new(0, 0),
        }
    }
}

impl std::convert::TryFrom<u8> for ControlSetting {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ControlSetting::Heartbeat),
            1 => Ok(ControlSetting::PeakPressure),
            2 => Ok(ControlSetting::PlateauPressure),
            3 => Ok(ControlSetting::PEEP),
            4 => Ok(ControlSetting::CyclesPerMinute),
            5 => Ok(ControlSetting::ExpiratoryTerm),
            6 => Ok(ControlSetting::TriggerEnabled),
            7 => Ok(ControlSetting::TriggerOffset),
            8 => Ok(ControlSetting::RespirationEnabled),
            9 => Ok(ControlSetting::AlarmSnooze),
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
            ControlSetting::PeakPressure => rng.gen_range(0, 71),
            ControlSetting::PlateauPressure => rng.gen_range(10, 41),
            ControlSetting::PEEP => rng.gen_range(0, 31),
            ControlSetting::CyclesPerMinute => rng.gen_range(5, 36),
            ControlSetting::ExpiratoryTerm => rng.gen_range(500, 5001),
            ControlSetting::TriggerEnabled => rng.gen_range(0, 2),
            ControlSetting::TriggerOffset => rng.gen_range(0, 101),
            ControlSetting::RespirationEnabled => rng.gen_range(0, 2),
            ControlSetting::AlarmSnooze => rng.gen_range(0, 0),
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
