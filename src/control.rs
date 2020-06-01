// MakAir
//
// Copyright: 2020, Makers For Life
// License: Public Domain License

use rand::distributions::{Distribution, Standard};
use rand::Rng;
use std::convert::TryFrom;

/// Available settings in the control protocol
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ControlSetting {
    PeakPressure = 1,
    PlateauPressure = 2,
    PEEP = 3,
    CyclesPerMinute = 4,
    ExpiratoryTerm = 5,
}

impl std::convert::TryFrom<u8> for ControlSetting {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(ControlSetting::PeakPressure),
            2 => Ok(ControlSetting::PlateauPressure),
            3 => Ok(ControlSetting::PEEP),
            4 => Ok(ControlSetting::CyclesPerMinute),
            5 => Ok(ControlSetting::ExpiratoryTerm),
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
            ControlSetting::PeakPressure => rng.gen_range(0, 71),
            ControlSetting::PlateauPressure => rng.gen_range(10, 41),
            ControlSetting::PEEP => rng.gen_range(0, 31),
            ControlSetting::CyclesPerMinute => rng.gen_range(5, 36),
            ControlSetting::ExpiratoryTerm => rng.gen_range(500, 5001),
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
