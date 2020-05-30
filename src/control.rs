#[derive(Debug, Copy, Clone)]
pub enum ControlSetting {
    PeakPressure = 1,
    PlateauPressure = 2,
    PEEP = 3,
}
/*
impl std::fmt::Display for ControlSetting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PeakPressure => write!(f, "PeakPressure"),
        }
    }
}*/

#[derive(Debug, Clone)]
pub struct ControlMessage {
    pub setting: ControlSetting,
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

    pub fn to_control_frame(&self) -> Vec<u8> {
        flat(&[
            b"\x05\x0A",
            &self.to_bytes(),
            &self.crc().to_be_bytes(),
            b"\x50\xA0",
        ])
    }
}
