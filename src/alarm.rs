// MakAir Telemetry
//
// Copyright: 2020, Makers For Life
// License: Public Domain License

pub const RMC_SW_1: u8 = 12;
pub const RMC_SW_2: u8 = 11;
pub const RMC_SW_3: u8 = 14;
pub const RMC_SW_11: u8 = 21;
pub const RMC_SW_12: u8 = 13;
pub const RMC_SW_14: u8 = 22;
pub const RMC_SW_15: u8 = 23;
pub const RMC_SW_16: u8 = 31;
pub const RMC_SW_18: u8 = 17;
pub const RMC_SW_19: u8 = 24;

/// Wrapper arround an alarm code
#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy, PartialOrd, Ord)]
pub struct AlarmCode {
    code: u8,
}

pub enum AlarmCodeDescription {
    PlateauPressureNotReached,
    PatientUnplugged,
    PEEPPressureNotReached,
    BatteryLow,
    BatteryVeryLow,
    PowerCableUnplugged,
    PressureTooHigh,
    Unknown(u8)
}

impl AlarmCode {
    /// Get a textual description of the inner alarm code
    pub fn description(self) -> AlarmCodeDescription {
        match self.code {
            RMC_SW_1 | RMC_SW_14 => AlarmCodeDescription::PlateauPressureNotReached,
            RMC_SW_2 | RMC_SW_19 => AlarmCodeDescription::PatientUnplugged,
            RMC_SW_3 | RMC_SW_15 => AlarmCodeDescription::PEEPPressureNotReached,
            RMC_SW_11 => AlarmCodeDescription::BatteryLow,
            RMC_SW_12 => AlarmCodeDescription::BatteryVeryLow,
            RMC_SW_16 => AlarmCodeDescription::PowerCableUnplugged,
            RMC_SW_18 => AlarmCodeDescription::PressureTooHigh,
            _ => AlarmCodeDescription::Unknown(self.code),
        }
    }

    /// Unwrap the inner alarm code
    pub fn code(self) -> u8 {
        self.code
    }
}

impl From<u8> for AlarmCode {
    /// Wrap a raw alarm code
    fn from(code: u8) -> AlarmCode {
        AlarmCode { code }
    }
}
