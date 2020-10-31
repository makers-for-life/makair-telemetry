// MakAir Telemetry
//
// Copyright: 2020, Makers For Life
// License: Public Domain License

/// Error code of RMC SW 1
pub const RMC_SW_1: u8 = 12;
/// Error code of RMC SW 2
pub const RMC_SW_2: u8 = 11;
/// Error code of RMC SW 3
pub const RMC_SW_3: u8 = 14;
/// Error code of RMC SW 11
pub const RMC_SW_11: u8 = 21;
/// Error code of RMC SW 12
pub const RMC_SW_12: u8 = 13;
/// Error code of RMC SW 14
pub const RMC_SW_14: u8 = 22;
/// Error code of RMC SW 15
pub const RMC_SW_15: u8 = 23;
/// Error code of RMC SW 16
pub const RMC_SW_16: u8 = 31;
/// Error code of RMC SW 18
pub const RMC_SW_18: u8 = 17;
/// Error code of RMC SW 19
pub const RMC_SW_19: u8 = 24;

/// Wrapper arround an alarm code
#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy, PartialOrd, Ord)]
pub struct AlarmCode {
    code: u8,
}

/// Possible alarms causes
pub enum AlarmCodeDescription {
    /// Plateau pressure was not reached
    PlateauPressureNotReached,
    /// Patient is unplugged
    PatientUnplugged,
    /// PEEP was not reached
    PEEPPressureNotReached,
    /// Battery level is low
    BatteryLow,
    /// Battery level is very low
    BatteryVeryLow,
    /// Power outlet is unplugged
    PowerCableUnplugged,
    /// Pressure is too high
    PressureTooHigh,
    /// Unknown cause
    Unknown(u8),
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

    /// Lists adjacent (similar) alarm (if any)
    pub fn adjacent(self) -> Option<AlarmCode> {
        // Adjacent alarm codes always match from higher priority, to lower priority. This lets \
        //   a telemetry library consumer to de-duplicate alarms when a similar alarm is shown \
        //   at both medium and high level.
        match self.code {
            // 'Battery very low' high-priority alarm takes precedence over 'battery low' \
            //   medium-priority alarm
            RMC_SW_12 => Some(Self::from(RMC_SW_11)),
            // 'Patient unplugged' high-priority alarm takes precedence over its medium-priority \
            //   counterpart
            RMC_SW_2 => Some(Self::from(RMC_SW_19)),
            // 'Plateau pressure not reached' high-priority alarm takes precedence over its \
            //   medium-priority counterpart
            RMC_SW_1 => Some(Self::from(RMC_SW_14)),
            // 'PEEP pressure not reached' high-priority alarm takes precedence over its \
            //   medium-priority counterpart
            RMC_SW_3 => Some(Self::from(RMC_SW_15)),
            _ => None,
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
