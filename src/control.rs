/// Available setting in the control protocol
#[derive(Debug, Copy, Clone)]
pub enum ControlSetting {
    PeakPressure = 1,
    PlateauPressure = 2,
    PEEP = 3,
}

/// A control message
#[derive(Debug, Clone)]
pub struct ControlMessage {
    /// The setting to change
    pub setting: ControlSetting,
    /// The new value of the setting
    pub value: u16,
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
    pub fn to_control_frame(&self) -> Vec<u8> {
        flat(&[
            b"\x05\x0A",
            &self.to_bytes(),
            &self.crc().to_be_bytes(),
            b"\x50\xA0",
        ])
    }
}
