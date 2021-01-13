// MakAir Telemetry
//
// Copyright: 2020, Makers For Life
// License: Public Domain License

use crate::structures::*;

/// Serialize to binary using the telemetry protocol
pub trait ToBytes {
    /// Serialize to binary using the latest telemetry protocol
    fn to_bytes(&self) -> Vec<u8> {
        self.to_bytes_v2()
    }

    /// Serialize to binary using the telemetry protocol v1
    fn to_bytes_v1(&self) -> Vec<u8>;

    /// Serialize to binary using the telemetry protocol v2
    fn to_bytes_v2(&self) -> Vec<u8>;
}

fn flat(v: &[&[u8]]) -> Vec<u8> {
    v.iter().flat_map(|a| a.iter()).copied().collect()
}

fn split_device_id(device_id: &str) -> (u32, u32, u32) {
    use std::str::FromStr;

    let mut device_id = device_id.split('-');
    let device_id1 = device_id
        .next()
        .and_then(|str| u32::from_str(str).ok())
        .unwrap_or_default();
    let device_id2 = device_id
        .next()
        .and_then(|str| u32::from_str(str).ok())
        .unwrap_or_default();
    let device_id3 = device_id
        .next()
        .and_then(|str| u32::from_str(str).ok())
        .unwrap_or_default();
    (device_id1, device_id2, device_id3)
}

impl ToBytes for BootMessage {
    fn to_bytes_v1(&self) -> Vec<u8> {
        let (device_id1, device_id2, device_id3) = split_device_id(&self.device_id);

        flat(&[
            b"B:",
            &[1],
            &[self.version.len() as u8],
            &self.version.as_bytes(),
            &device_id1.to_be_bytes(),
            &device_id2.to_be_bytes(),
            &device_id3.to_be_bytes(),
            b"\t",
            &self.systick.to_be_bytes(),
            b"\t",
            &[self.mode as u8],
            b"\t",
            &[self.value128],
            b"\n",
        ])
    }

    fn to_bytes_v2(&self) -> Vec<u8> {
        let (device_id1, device_id2, device_id3) = split_device_id(&self.device_id);

        flat(&[
            b"B:",
            &[2],
            &[self.version.len() as u8],
            &self.version.as_bytes(),
            &device_id1.to_be_bytes(),
            &device_id2.to_be_bytes(),
            &device_id3.to_be_bytes(),
            b"\t",
            &self.systick.to_be_bytes(),
            b"\t",
            &[self.mode as u8],
            b"\t",
            &[self.value128],
            b"\n",
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_valid_device_id() {
        assert_eq!(split_device_id("123-456-789"), (123, 456, 789))
    }

    #[test]
    fn split_invalid_device_id() {
        assert_eq!(split_device_id("123-456789"), (123, 456789, 0))
    }
}
