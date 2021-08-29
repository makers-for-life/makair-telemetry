// MakAir Telemetry
//
// Copyright: 2020, Makers For Life
// License: Public Domain License

use log::warn;

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

fn phase_value_v1(phase: Phase, subphase: Option<SubPhase>) -> u8 {
    let subphase = subphase.unwrap_or_else(|| match phase {
        Phase::Inhalation => SubPhase::Inspiration,
        Phase::Exhalation => SubPhase::Exhale,
    });

    match (phase, subphase) {
        (Phase::Inhalation, SubPhase::Inspiration) => 17,
        (Phase::Inhalation, SubPhase::HoldInspiration) => 18,
        (Phase::Exhalation, SubPhase::Exhale) => 68,
        _ => 0,
    }
}

fn phase_value_v2(phase: Phase) -> u8 {
    match phase {
        Phase::Inhalation => 17,
        Phase::Exhalation => 68,
    }
}

fn alarm_priority_value(m: &AlarmPriority) -> u8 {
    match m {
        AlarmPriority::High => 4,
        AlarmPriority::Medium => 2,
        AlarmPriority::Low => 1,
    }
}

impl ToBytes for BootMessage {
    fn to_bytes_v1(&self) -> Vec<u8> {
        let (device_id1, device_id2, device_id3) = split_device_id(&self.device_id);

        flat(&[
            b"B:",
            &[1],
            &[self.version.len() as u8],
            self.version.as_bytes(),
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
            self.version.as_bytes(),
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

impl ToBytes for StoppedMessage {
    fn to_bytes_v1(&self) -> Vec<u8> {
        let (device_id1, device_id2, device_id3) = split_device_id(&self.device_id);

        flat(&[
            b"O:",
            &[1],
            &[self.version.len() as u8],
            self.version.as_bytes(),
            &device_id1.to_be_bytes(),
            &device_id2.to_be_bytes(),
            &device_id3.to_be_bytes(),
            b"\t",
            &self.systick.to_be_bytes(),
            b"\n",
        ])
    }

    fn to_bytes_v2(&self) -> Vec<u8> {
        let (device_id1, device_id2, device_id3) = split_device_id(&self.device_id);

        flat(&[
            b"O:",
            &[2],
            &[self.version.len() as u8],
            self.version.as_bytes(),
            &device_id1.to_be_bytes(),
            &device_id2.to_be_bytes(),
            &device_id3.to_be_bytes(),
            b"\t",
            &self.systick.to_be_bytes(),
            b"\t",
            &[self.peak_command.unwrap_or_default()],
            b"\t",
            &[self.plateau_command.unwrap_or_default()],
            b"\t",
            &[self.peep_command.unwrap_or_default()],
            b"\t",
            &[self.cpm_command.unwrap_or_default()],
            b"\t",
            &self.expiratory_term.unwrap_or_default().to_be_bytes(),
            b"\t",
            if self.trigger_enabled.unwrap_or_default() {
                b"\x01"
            } else {
                b"\x00"
            },
            b"\t",
            &self.trigger_offset.unwrap_or_default().to_be_bytes(),
            b"\t",
            if self.alarm_snoozed.unwrap_or_default() {
                b"\x01"
            } else {
                b"\x00"
            },
            b"\t",
            &self.cpu_load.unwrap_or_default().to_be_bytes(),
            b"\t",
            &[self.ventilation_mode as u8],
            b"\t",
            &self
                .inspiratory_trigger_flow
                .unwrap_or_default()
                .to_be_bytes(),
            b"\t",
            &self
                .expiratory_trigger_flow
                .unwrap_or_default()
                .to_be_bytes(),
            b"\t",
            &self.ti_min.unwrap_or_default().to_be_bytes(),
            b"\t",
            &self.ti_max.unwrap_or_default().to_be_bytes(),
            b"\t",
            &self
                .low_inspiratory_minute_volume_alarm_threshold
                .unwrap_or_default()
                .to_be_bytes(),
            b"\t",
            &self
                .high_inspiratory_minute_volume_alarm_threshold
                .unwrap_or_default()
                .to_be_bytes(),
            b"\t",
            &self
                .low_expiratory_minute_volume_alarm_threshold
                .unwrap_or_default()
                .to_be_bytes(),
            b"\t",
            &self
                .high_expiratory_minute_volume_alarm_threshold
                .unwrap_or_default()
                .to_be_bytes(),
            b"\t",
            &self
                .low_respiratory_rate_alarm_threshold
                .unwrap_or_default()
                .to_be_bytes(),
            b"\t",
            &self
                .high_respiratory_rate_alarm_threshold
                .unwrap_or_default()
                .to_be_bytes(),
            b"\t",
            &self.target_tidal_volume.unwrap_or_default().to_be_bytes(),
            b"\t",
            &self
                .low_tidal_volume_alarm_threshold
                .unwrap_or_default()
                .to_be_bytes(),
            b"\t",
            &self
                .high_tidal_volume_alarm_threshold
                .unwrap_or_default()
                .to_be_bytes(),
            b"\t",
            &self.plateau_duration.unwrap_or_default().to_be_bytes(),
            b"\t",
            &self.leak_alarm_threshold.unwrap_or_default().to_be_bytes(),
            b"\t",
            &self
                .target_inspiratory_flow
                .unwrap_or_default()
                .to_be_bytes(),
            b"\t",
            &self
                .inspiratory_duration_command
                .unwrap_or_default()
                .to_be_bytes(),
            b"\t",
            &self.battery_level.unwrap_or_default().to_be_bytes(),
            b"\t",
            &[self.current_alarm_codes.clone().unwrap_or_default().len() as u8],
            &self.current_alarm_codes.clone().unwrap_or_default(),
            b"\t",
            &self.locale.unwrap_or_default().as_u16().to_be_bytes(),
            b"\t",
            &self.patient_height.unwrap_or_default().to_be_bytes(),
            b"\t",
            &[self.patient_gender.unwrap_or_default() as u8],
            b"\t",
            &self
                .peak_pressure_alarm_threshold
                .unwrap_or_default()
                .to_be_bytes(),
            b"\n",
        ])
    }
}

impl ToBytes for DataSnapshot {
    fn to_bytes_v1(&self) -> Vec<u8> {
        let (device_id1, device_id2, device_id3) = split_device_id(&self.device_id);

        flat(&[
            b"D:",
            &[1],
            &[self.version.len() as u8],
            self.version.as_bytes(),
            &device_id1.to_be_bytes(),
            &device_id2.to_be_bytes(),
            &device_id3.to_be_bytes(),
            b"\t",
            &self.systick.to_be_bytes(),
            b"\t",
            &self.centile.to_be_bytes(),
            b"\t",
            &self.pressure.to_be_bytes(),
            b"\t",
            &[phase_value_v1(self.phase, self.subphase)],
            b"\t",
            &[self.blower_valve_position],
            b"\t",
            &[self.patient_valve_position],
            b"\t",
            &[self.blower_rpm],
            b"\t",
            &[self.battery_level],
            b"\n",
        ])
    }

    fn to_bytes_v2(&self) -> Vec<u8> {
        let (device_id1, device_id2, device_id3) = split_device_id(&self.device_id);

        flat(&[
            b"D:",
            &[2],
            &[self.version.len() as u8],
            self.version.as_bytes(),
            &device_id1.to_be_bytes(),
            &device_id2.to_be_bytes(),
            &device_id3.to_be_bytes(),
            b"\t",
            &self.systick.to_be_bytes(),
            b"\t",
            &self.centile.to_be_bytes(),
            b"\t",
            &self.pressure.to_be_bytes(),
            b"\t",
            &[phase_value_v2(self.phase)],
            b"\t",
            &[self.blower_valve_position],
            b"\t",
            &[self.patient_valve_position],
            b"\t",
            &[self.blower_rpm],
            b"\t",
            &[self.battery_level],
            b"\t",
            &self.inspiratory_flow.unwrap_or_default().to_be_bytes(),
            b"\t",
            &self.expiratory_flow.unwrap_or_default().to_be_bytes(),
            b"\n",
        ])
    }
}

impl ToBytes for MachineStateSnapshot {
    fn to_bytes_v1(&self) -> Vec<u8> {
        let (device_id1, device_id2, device_id3) = split_device_id(&self.device_id);

        flat(&[
            b"S:",
            &[1],
            &[self.version.len() as u8],
            self.version.as_bytes(),
            &device_id1.to_be_bytes(),
            &device_id2.to_be_bytes(),
            &device_id3.to_be_bytes(),
            b"\t",
            &self.systick.to_be_bytes(),
            b"\t",
            &self.cycle.to_be_bytes(),
            b"\t",
            &[self.peak_command],
            b"\t",
            &[self.plateau_command],
            b"\t",
            &[self.peep_command],
            b"\t",
            &[self.cpm_command],
            b"\t",
            &self.previous_peak_pressure.to_be_bytes(),
            b"\t",
            &self.previous_plateau_pressure.to_be_bytes(),
            b"\t",
            &self.previous_peep_pressure.to_be_bytes(),
            b"\t",
            &[self.current_alarm_codes.len() as u8],
            &self.current_alarm_codes,
            b"\t",
            &self.previous_volume.unwrap_or(0xFFFF).to_be_bytes(),
            b"\t",
            &self.expiratory_term.to_be_bytes(),
            b"\t",
            if self.trigger_enabled {
                b"\x01"
            } else {
                b"\x00"
            },
            b"\t",
            &self.trigger_offset.to_be_bytes(),
            b"\n",
        ])
    }

    fn to_bytes_v2(&self) -> Vec<u8> {
        let (device_id1, device_id2, device_id3) = split_device_id(&self.device_id);

        flat(&[
            b"S:",
            &[2],
            &[self.version.len() as u8],
            self.version.as_bytes(),
            &device_id1.to_be_bytes(),
            &device_id2.to_be_bytes(),
            &device_id3.to_be_bytes(),
            b"\t",
            &self.systick.to_be_bytes(),
            b"\t",
            &self.cycle.to_be_bytes(),
            b"\t",
            &[self.peak_command],
            b"\t",
            &[self.plateau_command],
            b"\t",
            &[self.peep_command],
            b"\t",
            &[self.cpm_command],
            b"\t",
            &self.previous_peak_pressure.to_be_bytes(),
            b"\t",
            &self.previous_plateau_pressure.to_be_bytes(),
            b"\t",
            &self.previous_peep_pressure.to_be_bytes(),
            b"\t",
            &[self.current_alarm_codes.len() as u8],
            &self.current_alarm_codes,
            b"\t",
            &self.previous_volume.unwrap_or(0xFFFF).to_be_bytes(),
            b"\t",
            &self.expiratory_term.to_be_bytes(),
            b"\t",
            if self.trigger_enabled {
                b"\x01"
            } else {
                b"\x00"
            },
            b"\t",
            &self.trigger_offset.to_be_bytes(),
            b"\t",
            &self.previous_cpm.unwrap_or_default().to_be_bytes(),
            b"\t",
            if self.alarm_snoozed.unwrap_or_default() {
                b"\x01"
            } else {
                b"\x00"
            },
            b"\t",
            &self.cpu_load.unwrap_or_default().to_be_bytes(),
            b"\t",
            &[self.ventilation_mode as u8],
            b"\t",
            &self
                .inspiratory_trigger_flow
                .unwrap_or_default()
                .to_be_bytes(),
            b"\t",
            &self
                .expiratory_trigger_flow
                .unwrap_or_default()
                .to_be_bytes(),
            b"\t",
            &self.ti_min.unwrap_or_default().to_be_bytes(),
            b"\t",
            &self.ti_max.unwrap_or_default().to_be_bytes(),
            b"\t",
            &self
                .low_inspiratory_minute_volume_alarm_threshold
                .unwrap_or_default()
                .to_be_bytes(),
            b"\t",
            &self
                .high_inspiratory_minute_volume_alarm_threshold
                .unwrap_or_default()
                .to_be_bytes(),
            b"\t",
            &self
                .low_expiratory_minute_volume_alarm_threshold
                .unwrap_or_default()
                .to_be_bytes(),
            b"\t",
            &self
                .high_expiratory_minute_volume_alarm_threshold
                .unwrap_or_default()
                .to_be_bytes(),
            b"\t",
            &self
                .low_respiratory_rate_alarm_threshold
                .unwrap_or_default()
                .to_be_bytes(),
            b"\t",
            &self
                .high_respiratory_rate_alarm_threshold
                .unwrap_or_default()
                .to_be_bytes(),
            b"\t",
            &self.target_tidal_volume.unwrap_or_default().to_be_bytes(),
            b"\t",
            &self
                .low_tidal_volume_alarm_threshold
                .unwrap_or_default()
                .to_be_bytes(),
            b"\t",
            &self
                .high_tidal_volume_alarm_threshold
                .unwrap_or_default()
                .to_be_bytes(),
            b"\t",
            &self.plateau_duration.unwrap_or_default().to_be_bytes(),
            b"\t",
            &self.leak_alarm_threshold.unwrap_or_default().to_be_bytes(),
            b"\t",
            &self
                .target_inspiratory_flow
                .unwrap_or_default()
                .to_be_bytes(),
            b"\t",
            &self
                .inspiratory_duration_command
                .unwrap_or_default()
                .to_be_bytes(),
            b"\t",
            &self
                .previous_inspiratory_duration
                .unwrap_or_default()
                .to_be_bytes(),
            b"\t",
            &self.battery_level.unwrap_or_default().to_be_bytes(),
            b"\t",
            &self.locale.unwrap_or_default().as_u16().to_be_bytes(),
            b"\t",
            &self.patient_height.unwrap_or_default().to_be_bytes(),
            b"\t",
            &[self.patient_gender.unwrap_or_default() as u8],
            b"\t",
            &self
                .peak_pressure_alarm_threshold
                .unwrap_or_default()
                .to_be_bytes(),
            b"\n",
        ])
    }
}

impl ToBytes for AlarmTrap {
    fn to_bytes_v1(&self) -> Vec<u8> {
        let (device_id1, device_id2, device_id3) = split_device_id(&self.device_id);

        flat(&[
            b"T:",
            &[1],
            &[self.version.len() as u8],
            self.version.as_bytes(),
            &device_id1.to_be_bytes(),
            &device_id2.to_be_bytes(),
            &device_id3.to_be_bytes(),
            b"\t",
            &self.systick.to_be_bytes(),
            b"\t",
            &self.centile.to_be_bytes(),
            b"\t",
            &self.pressure.to_be_bytes(),
            b"\t",
            &[phase_value_v1(self.phase, self.subphase)],
            b"\t",
            &self.cycle.to_be_bytes(),
            b"\t",
            &[self.alarm_code],
            b"\t",
            &[alarm_priority_value(&self.alarm_priority)],
            b"\t",
            &[if self.triggered { 240u8 } else { 15u8 }],
            b"\t",
            &self.expected.to_be_bytes(),
            b"\t",
            &self.measured.to_be_bytes(),
            b"\t",
            &self.cycles_since_trigger.to_be_bytes(),
            b"\n",
        ])
    }

    fn to_bytes_v2(&self) -> Vec<u8> {
        let (device_id1, device_id2, device_id3) = split_device_id(&self.device_id);

        flat(&[
            b"T:",
            &[2],
            &[self.version.len() as u8],
            self.version.as_bytes(),
            &device_id1.to_be_bytes(),
            &device_id2.to_be_bytes(),
            &device_id3.to_be_bytes(),
            b"\t",
            &self.systick.to_be_bytes(),
            b"\t",
            &self.centile.to_be_bytes(),
            b"\t",
            &self.pressure.to_be_bytes(),
            b"\t",
            &[phase_value_v2(self.phase)],
            b"\t",
            &self.cycle.to_be_bytes(),
            b"\t",
            &[self.alarm_code],
            b"\t",
            &[alarm_priority_value(&self.alarm_priority)],
            b"\t",
            &[if self.triggered { 240u8 } else { 15u8 }],
            b"\t",
            &self.expected.to_be_bytes(),
            b"\t",
            &self.measured.to_be_bytes(),
            b"\t",
            &self.cycles_since_trigger.to_be_bytes(),
            b"\n",
        ])
    }
}

impl ToBytes for ControlAck {
    fn to_bytes_v1(&self) -> Vec<u8> {
        let (device_id1, device_id2, device_id3) = split_device_id(&self.device_id);

        flat(&[
            b"A:",
            &[1],
            &[self.version.len() as u8],
            self.version.as_bytes(),
            &device_id1.to_be_bytes(),
            &device_id2.to_be_bytes(),
            &device_id3.to_be_bytes(),
            b"\t",
            &self.systick.to_be_bytes(),
            b"\t",
            &(self.setting as u8).to_be_bytes(),
            b"\t",
            &self.value.to_be_bytes(),
            b"\n",
        ])
    }

    fn to_bytes_v2(&self) -> Vec<u8> {
        let (device_id1, device_id2, device_id3) = split_device_id(&self.device_id);

        flat(&[
            b"A:",
            &[2],
            &[self.version.len() as u8],
            self.version.as_bytes(),
            &device_id1.to_be_bytes(),
            &device_id2.to_be_bytes(),
            &device_id3.to_be_bytes(),
            b"\t",
            &self.systick.to_be_bytes(),
            b"\t",
            &(self.setting as u8).to_be_bytes(),
            b"\t",
            &self.value.to_be_bytes(),
            b"\n",
        ])
    }
}

impl ToBytes for FatalError {
    fn to_bytes_v1(&self) -> Vec<u8> {
        warn!(
            "trying to serialize a FatalError message that did not exist in telemetry protocol v1"
        );
        vec![]
    }

    fn to_bytes_v2(&self) -> Vec<u8> {
        let (device_id1, device_id2, device_id3) = split_device_id(&self.device_id);

        let fatal_error_details: Vec<u8> = match self.error {
            FatalErrorDetails::WatchdogRestart => vec![1],
            FatalErrorDetails::CalibrationError {
                pressure_offset,
                min_pressure,
                max_pressure,
                flow_at_starting,
                flow_with_blower_on,
            } => flat(&[
                &[2],
                b"\t",
                &pressure_offset.to_be_bytes(),
                b"\t",
                &min_pressure.to_be_bytes(),
                b"\t",
                &max_pressure.to_be_bytes(),
                b"\t",
                &flow_at_starting.unwrap_or(i16::MAX).to_be_bytes(),
                b"\t",
                &flow_with_blower_on.unwrap_or(i16::MAX).to_be_bytes(),
            ]),
            FatalErrorDetails::BatteryDeeplyDischarged { battery_level } => {
                flat(&[&[3], b"\t", &battery_level.to_be_bytes()])
            }
            FatalErrorDetails::MassFlowMeterError => vec![4],
            FatalErrorDetails::InconsistentPressure { pressure } => {
                flat(&[&[5], b"\t", &pressure.to_be_bytes()])
            }
        };

        flat(&[
            b"E:",
            &[2],
            &[self.version.len() as u8],
            self.version.as_bytes(),
            &device_id1.to_be_bytes(),
            &device_id2.to_be_bytes(),
            &device_id3.to_be_bytes(),
            b"\t",
            &self.systick.to_be_bytes(),
            b"\t",
            &fatal_error_details,
            b"\n",
        ])
    }
}

impl ToBytes for EolTestSnapshot {
    fn to_bytes_v1(&self) -> Vec<u8> {
        warn!("trying to serialize a EolTestSnapshot message that did not exist in telemetry protocol v1");
        vec![]
    }

    fn to_bytes_v2(&self) -> Vec<u8> {
        let (device_id1, device_id2, device_id3) = split_device_id(&self.device_id);

        let eol_test_snapshot_content: Vec<u8> = match self.content {
            EolTestSnapshotContent::InProgress(ref message) => {
                flat(&[&[0], b"\t", &[message.len() as u8], message.as_bytes()])
            }
            EolTestSnapshotContent::Error(ref message) => {
                flat(&[&[1], b"\t", &[message.len() as u8], message.as_bytes()])
            }
            EolTestSnapshotContent::Success(ref message) => {
                flat(&[&[2], b"\t", &[message.len() as u8], message.as_bytes()])
            }
        };

        flat(&[
            b"L:",
            &[2],
            &[self.version.len() as u8],
            self.version.as_bytes(),
            &device_id1.to_be_bytes(),
            &device_id2.to_be_bytes(),
            &device_id3.to_be_bytes(),
            b"\t",
            &self.systick.to_be_bytes(),
            b"\t",
            &[self.current_step as u8],
            b"\t",
            &eol_test_snapshot_content,
            b"\n",
        ])
    }
}

/// Wrap a binary payload into a CRC-aware binary frame
pub fn mk_frame(payload: &[u8]) -> Vec<u8> {
    let mut crc = crc32fast::Hasher::new();
    crc.update(payload);

    flat(&[
        b"\x03\x0C",
        payload,
        &crc.finalize().to_be_bytes(),
        b"\x30\xC0",
    ])
}

impl ToBytes for TelemetryMessage {
    fn to_bytes_v1(&self) -> Vec<u8> {
        let payload = match self {
            Self::BootMessage(m) => m.to_bytes_v1(),
            Self::StoppedMessage(m) => m.to_bytes_v1(),
            Self::DataSnapshot(m) => m.to_bytes_v1(),
            Self::MachineStateSnapshot(m) => m.to_bytes_v1(),
            Self::AlarmTrap(m) => m.to_bytes_v1(),
            Self::ControlAck(m) => m.to_bytes_v1(),
            Self::FatalError(m) => m.to_bytes_v1(),
            Self::EolTestSnapshot(m) => m.to_bytes_v1(),
        };
        mk_frame(&payload)
    }

    fn to_bytes_v2(&self) -> Vec<u8> {
        let payload = match self {
            Self::BootMessage(m) => m.to_bytes_v2(),
            Self::StoppedMessage(m) => m.to_bytes_v2(),
            Self::DataSnapshot(m) => m.to_bytes_v2(),
            Self::MachineStateSnapshot(m) => m.to_bytes_v2(),
            Self::AlarmTrap(m) => m.to_bytes_v2(),
            Self::ControlAck(m) => m.to_bytes_v2(),
            Self::FatalError(m) => m.to_bytes_v2(),
            Self::EolTestSnapshot(m) => m.to_bytes_v2(),
        };
        mk_frame(&payload)
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
