// MakAir Telemetry
//
// Copyright: 2020, Makers For Life
// License: Public Domain License

use nom::IResult;
use std::ops::RangeInclusive;

use crate::locale::Locale;
use crate::structures::{TelemetryError, TelemetryErrorKind};

/// Special value that can be used in a heartbeat control message to disable RPi watchdog
pub const DISABLE_RPI_WATCHDOG: u16 = 43_690;

/// Available settings in the control protocol
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-messages",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum ControlSetting {
    /// Heartbeat used for the RPi watchdog feature (value is ignored except for the special value `DISABLE_RPI_WATCHDOG` which disables watchdog)
    Heartbeat = 0,
    /// Ventilation mode, must be one of the following:
    /// - `1` → PC-CMV
    /// - `2` → PC-AC (default)
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
    /// Threshold for low respiratory rate alarm in cycle per minute (value bounds must be between 5 and 25)
    LowRespiratoryRateAlarmThreshold = 18,
    /// Threshold for high respiratory rate alarm in cycle per minute (value bounds must be between 15 and 35)
    HighRespiratoryRateAlarmThreshold = 19,
    /// Target tidal volume in mL (value bounds must be between 50 and 2000)
    TargetTidalVolume = 20,
    /// Threshold for low tidal volume in mL (value bounds must be between 0 and 1000)
    LowTidalVolumeAlarmThreshold = 21,
    /// Threshold for high tidal volume in mL (value bounds must be between 50 and 2000)
    HighTidalVolumeAlarmThreshold = 22,
    /// Duration in ms of closing both valves to effectively measure plateau pressure in volume control modes (value bounds must be between 100 and 2000)
    PlateauDuration = 23,
    /// Threshold for leak alarm in cL/min (value bounds must be between 0 and 10000)
    LeakAlarmThreshold = 24,
    /// Target flow during inspiration in L/min (value bounds must be between 5 and 80)
    TargetInspiratoryFlow = 25,
    /// Duration of inspiration in ms (value bounds must be between 200 and 3000)
    InspiratoryDuration = 26,
    /// Language of the system; this should be two letters (see [ISO 639-1](https://en.wikipedia.org/wiki/ISO_639-1)) in ASCII representation as two u8
    Locale = 27,
    /// Patient's height in centimeters
    PatientHeight = 28,
    /// Patient's gender (0 = male, 1 = female)
    PatientGender = 29,
    /// Threshold for peak pressure alarm in mmH2O (value bounds must be between 50 and 700)
    PeakPressureAlarmThreshold = 30,
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
            Self::LowRespiratoryRateAlarmThreshold => 10,
            Self::HighRespiratoryRateAlarmThreshold => 30,
            Self::TargetTidalVolume => 400,
            Self::LowTidalVolumeAlarmThreshold => 200,
            Self::HighTidalVolumeAlarmThreshold => 1_000,
            Self::PlateauDuration => 200,
            Self::LeakAlarmThreshold => 200,
            Self::TargetInspiratoryFlow => 40,
            Self::InspiratoryDuration => 800,
            Self::Locale => Locale::default().as_usize(),
            Self::PatientHeight => 160,
            Self::PatientGender => 0,
            Self::PeakPressureAlarmThreshold => 500,
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
            Self::LowRespiratoryRateAlarmThreshold => RangeInclusive::new(5, 25),
            Self::HighRespiratoryRateAlarmThreshold => RangeInclusive::new(15, 35),
            Self::TargetTidalVolume => RangeInclusive::new(50, 2_000),
            Self::LowTidalVolumeAlarmThreshold => RangeInclusive::new(0, 1_000),
            Self::HighTidalVolumeAlarmThreshold => RangeInclusive::new(50, 2_000),
            Self::PlateauDuration => RangeInclusive::new(100, 2_000),
            Self::LeakAlarmThreshold => RangeInclusive::new(0, 10_000),
            Self::TargetInspiratoryFlow => RangeInclusive::new(5, 80),
            Self::InspiratoryDuration => RangeInclusive::new(200, 3_000),
            Self::Locale => Locale::bounds(),
            Self::PatientHeight => RangeInclusive::new(100, 250),
            Self::PatientGender => RangeInclusive::new(0, 1),
            Self::PeakPressureAlarmThreshold => RangeInclusive::new(50, 700),
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
            18 => Ok(ControlSetting::LowRespiratoryRateAlarmThreshold),
            19 => Ok(ControlSetting::HighRespiratoryRateAlarmThreshold),
            20 => Ok(ControlSetting::TargetTidalVolume),
            21 => Ok(ControlSetting::LowTidalVolumeAlarmThreshold),
            22 => Ok(ControlSetting::HighTidalVolumeAlarmThreshold),
            23 => Ok(ControlSetting::PlateauDuration),
            24 => Ok(ControlSetting::LeakAlarmThreshold),
            25 => Ok(ControlSetting::TargetInspiratoryFlow),
            26 => Ok(ControlSetting::InspiratoryDuration),
            27 => Ok(ControlSetting::Locale),
            28 => Ok(ControlSetting::PatientHeight),
            29 => Ok(ControlSetting::PatientGender),
            30 => Ok(ControlSetting::PeakPressureAlarmThreshold),
            _ => Err("Invalid setting number"),
        }
    }
}

#[cfg(feature = "rand")]
impl rand::distributions::Distribution<ControlSetting> for rand::distributions::Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> ControlSetting {
        use std::convert::TryFrom;

        let number = rng.gen_range(1..=5);
        ControlSetting::try_from(number).unwrap()
    }
}

/// A control message
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlMessage {
    /// The setting to change
    pub setting: ControlSetting,
    /// The new value of the setting
    pub value: u16,
}

#[cfg(feature = "rand")]
impl rand::distributions::Distribution<ControlMessage> for rand::distributions::Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> ControlMessage {
        use std::convert::TryFrom;

        let setting: ControlSetting = rng.gen();
        let value = u16::try_from(rng.gen_range(setting.bounds())).unwrap_or(u16::MAX);
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

fn parse_control_setting(input: &[u8]) -> IResult<&[u8], ControlSetting> {
    use nom::combinator::map_res;
    use nom::number::streaming::be_u8;
    use std::convert::TryFrom;

    map_res(be_u8, ControlSetting::try_from)(input)
}

fn parse_inner_control_message(input: &[u8]) -> IResult<&[u8], ControlMessage> {
    use nom::number::streaming::be_u16;

    nom::do_parse!(
        input,
        setting: parse_control_setting >> value: be_u16 >> (ControlMessage { setting, value })
    )
}

/// Transform bytes into a structured control message
///
/// * `input` - Bytes to parse.
pub fn parse_control_message(
    input: &[u8],
) -> IResult<&[u8], ControlMessage, TelemetryError<&[u8]>> {
    use nom::bytes::streaming::tag;
    use nom::combinator::consumed;
    use nom::number::streaming::be_u32;
    use nom::sequence::{pair, preceded, terminated};

    let header = tag(b"\x05\x0A");
    let footer = tag(b"\x50\xA0");
    let mut parser = preceded(
        header,
        terminated(pair(consumed(parse_inner_control_message), be_u32), footer),
    );

    parser(input)
        .map_err(nom::Err::convert)
        .and_then(|(rest, ((msg_bytes, msg), expected_crc))| {
            let mut crc = crc32fast::Hasher::new();
            crc.update(msg_bytes);
            let computed_crc = crc.finalize();
            if expected_crc == computed_crc {
                Ok((rest, msg))
            } else {
                Err(nom::Err::Failure(TelemetryError(
                    input,
                    TelemetryErrorKind::CrcError {
                        expected: expected_crc,
                        computed: computed_crc,
                    },
                )))
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::num;
    use proptest::prelude::*;
    use std::convert::TryFrom;

    fn control_setting_strategy() -> impl Strategy<Value = ControlSetting> {
        proptest::num::u8::ANY.prop_filter_map("Invalid control setting", |n| {
            ControlSetting::try_from(n).ok()
        })
    }

    proptest! {
        #[test]
        fn test_control_message_parser(
            setting in control_setting_strategy(),
            value in num::u16::ANY,
        ) {
            let msg = ControlMessage {
                setting,
                value,
            };
            let input = &msg.to_control_frame();

            assert_eq!(nom::dbg_dmp(parse_control_message, "parse_control_message")(input), Ok((&[][..], msg)));
        }
    }
}
