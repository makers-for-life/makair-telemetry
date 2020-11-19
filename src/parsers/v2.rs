use nom::number::streaming::{be_i16, be_u16, be_u32, be_u64, be_u8};
use nom::IResult;
use std::convert::TryFrom;

use super::super::structures::*;

const VERSION: u8 = 2;

named!(sep, tag!("\t"));
named!(end, tag!("\n"));

named!(
    mode<Mode>,
    alt!(
        map!(tag!(b"\x01"), |_| Mode::Production)
            | map!(tag!(b"\x02"), |_| Mode::Qualification)
            | map!(tag!(b"\x03"), |_| Mode::IntegrationTest)
    )
);

named!(
    phase<Phase>,
    alt!(map!(tag!([17u8]), |_| Phase::Inhalation) | map!(tag!([68u8]), |_| Phase::Exhalation))
);

named!(
    control_setting<ControlSetting>,
    map_res!(be_u8, |num| ControlSetting::try_from(num))
);

named!(
    alarm_priority<AlarmPriority>,
    alt!(
        map!(tag!([4u8]), |_| AlarmPriority::High)
            | map!(tag!([2u8]), |_| AlarmPriority::Medium)
            | map!(tag!([1u8]), |_| AlarmPriority::Low)
    )
);

named!(
    u8_array<Vec<u8>>,
    map!(length_data!(be_u8), |slice| Vec::from(slice))
);

named!(
    triggered<bool>,
    alt!(map!(tag!([240u8]), |_| true) | map!(tag!([15u8]), |_| false))
);

named!(
    ventilation_mode<VentilationMode>,
    map_res!(be_u8, |num| VentilationMode::try_from(num))
);

named!(
    boot<TelemetryMessage>,
    do_parse!(
        tag!("B:")
            >> tag!([VERSION])
            >> software_version_len: be_u8
            >> software_version:
                map_res!(take!(software_version_len), |bytes| std::str::from_utf8(
                    bytes
                ))
            >> device_id1: be_u32
            >> device_id2: be_u32
            >> device_id3: be_u32
            >> sep
            >> systick: be_u64
            >> sep
            >> mode: mode
            >> sep
            >> value128: be_u8
            >> end
            >> ({
                TelemetryMessage::BootMessage(BootMessage {
                    telemetry_version: VERSION,
                    version: software_version.to_string(),
                    device_id: format!("{}-{}-{}", device_id1, device_id2, device_id3),
                    systick,
                    mode,
                    value128,
                })
            })
    )
);

named!(
    stopped<TelemetryMessage>,
    do_parse!(
        tag!("O:")
            >> tag!([VERSION])
            >> software_version_len: be_u8
            >> software_version:
                map_res!(take!(software_version_len), |bytes| std::str::from_utf8(
                    bytes
                ))
            >> device_id1: be_u32
            >> device_id2: be_u32
            >> device_id3: be_u32
            >> sep
            >> systick: be_u64
            >> sep
            >> peak_command: be_u8
            >> sep
            >> plateau_command: be_u8
            >> sep
            >> peep_command: be_u8
            >> sep
            >> cpm_command: be_u8
            >> sep
            >> expiratory_term: be_u8
            >> sep
            >> trigger_enabled: be_u8
            >> sep
            >> trigger_offset: be_u8
            >> sep
            >> alarm_snoozed: be_u8
            >> sep
            >> cpu_load: be_u8
            >> sep
            >> ventilation_mode: ventilation_mode
            >> sep
            >> inspiratory_trigger_flow: be_u8
            >> sep
            >> expiratory_trigger_flow: be_u8
            >> sep
            >> ti_min: be_u16
            >> sep
            >> ti_max: be_u16
            >> sep
            >> low_inspiratory_minute_volume_alarm_threshold: be_u8
            >> sep
            >> high_inspiratory_minute_volume_alarm_threshold: be_u8
            >> sep
            >> low_expiratory_minute_volume_alarm_threshold: be_u8
            >> sep
            >> high_expiratory_minute_volume_alarm_threshold: be_u8
            >> sep
            >> low_expiratory_rate_alarm_threshold: be_u8
            >> sep
            >> high_expiratory_rate_alarm_threshold: be_u8
            >> end
            >> ({
                TelemetryMessage::StoppedMessage(StoppedMessage {
                    telemetry_version: VERSION,
                    version: software_version.to_string(),
                    device_id: format!("{}-{}-{}", device_id1, device_id2, device_id3),
                    systick,
                    peak_command: Some(peak_command),
                    plateau_command: Some(plateau_command),
                    peep_command: Some(peep_command),
                    cpm_command: Some(cpm_command),
                    expiratory_term: Some(expiratory_term),
                    trigger_enabled: Some(trigger_enabled != 0),
                    trigger_offset: Some(trigger_offset),
                    alarm_snoozed: Some(alarm_snoozed != 0),
                    cpu_load: Some(cpu_load),
                    ventilation_mode,
                    inspiratory_trigger_flow: Some(inspiratory_trigger_flow),
                    expiratory_trigger_flow: Some(expiratory_trigger_flow),
                    ti_min: Some(ti_min),
                    ti_max: Some(ti_max),
                    low_inspiratory_minute_volume_alarm_threshold: Some(
                        low_inspiratory_minute_volume_alarm_threshold,
                    ),
                    high_inspiratory_minute_volume_alarm_threshold: Some(
                        high_inspiratory_minute_volume_alarm_threshold,
                    ),
                    low_expiratory_minute_volume_alarm_threshold: Some(
                        low_expiratory_minute_volume_alarm_threshold,
                    ),
                    high_expiratory_minute_volume_alarm_threshold: Some(
                        high_expiratory_minute_volume_alarm_threshold,
                    ),
                    low_expiratory_rate_alarm_threshold: Some(low_expiratory_rate_alarm_threshold),
                    high_expiratory_rate_alarm_threshold: Some(
                        high_expiratory_rate_alarm_threshold,
                    ),
                })
            })
    )
);

named!(
    data_snapshot<TelemetryMessage>,
    do_parse!(
        tag!("D:")
            >> tag!([VERSION])
            >> software_version_len: be_u8
            >> software_version:
                map_res!(take!(software_version_len), |bytes| std::str::from_utf8(
                    bytes
                ))
            >> device_id1: be_u32
            >> device_id2: be_u32
            >> device_id3: be_u32
            >> sep
            >> systick: be_u64
            >> sep
            >> centile: be_u16
            >> sep
            >> pressure: be_i16
            >> sep
            >> phase: phase
            >> sep
            >> blower_valve_position: be_u8
            >> sep
            >> patient_valve_position: be_u8
            >> sep
            >> blower_rpm: be_u8
            >> sep
            >> battery_level: be_u8
            >> sep
            >> inspiratory_flow: be_i16
            >> sep
            >> expiratory_flow: be_i16
            >> end
            >> (TelemetryMessage::DataSnapshot(DataSnapshot {
                telemetry_version: VERSION,
                version: software_version.to_string(),
                device_id: format!("{}-{}-{}", device_id1, device_id2, device_id3),
                systick,
                centile,
                pressure,
                phase,
                subphase: None,
                blower_valve_position,
                patient_valve_position,
                blower_rpm,
                battery_level,
                inspiratory_flow: Some(inspiratory_flow),
                expiratory_flow: Some(expiratory_flow),
            }))
    )
);

named!(
    machine_state_snapshot<TelemetryMessage>,
    do_parse!(
        tag!("S:")
            >> tag!([VERSION])
            >> software_version_len: be_u8
            >> software_version:
                map_res!(take!(software_version_len), |bytes| std::str::from_utf8(
                    bytes
                ))
            >> device_id1: be_u32
            >> device_id2: be_u32
            >> device_id3: be_u32
            >> sep
            >> systick: be_u64
            >> sep
            >> cycle: be_u32
            >> sep
            >> peak_command: be_u8
            >> sep
            >> plateau_command: be_u8
            >> sep
            >> peep_command: be_u8
            >> sep
            >> cpm_command: be_u8
            >> sep
            >> previous_peak_pressure: be_u16
            >> sep
            >> previous_plateau_pressure: be_u16
            >> sep
            >> previous_peep_pressure: be_u16
            >> sep
            >> current_alarm_codes: u8_array
            >> sep
            >> previous_volume: be_u16
            >> sep
            >> expiratory_term: be_u8
            >> sep
            >> trigger_enabled: be_u8
            >> sep
            >> trigger_offset: be_u8
            >> sep
            >> previous_cpm: be_u8
            >> sep
            >> alarm_snoozed: be_u8
            >> sep
            >> cpu_load: be_u8
            >> sep
            >> ventilation_mode: ventilation_mode
            >> sep
            >> inspiratory_trigger_flow: be_u8
            >> sep
            >> expiratory_trigger_flow: be_u8
            >> sep
            >> ti_min: be_u16
            >> sep
            >> ti_max: be_u16
            >> sep
            >> low_inspiratory_minute_volume_alarm_threshold: be_u8
            >> sep
            >> high_inspiratory_minute_volume_alarm_threshold: be_u8
            >> sep
            >> low_expiratory_minute_volume_alarm_threshold: be_u8
            >> sep
            >> high_expiratory_minute_volume_alarm_threshold: be_u8
            >> sep
            >> low_expiratory_rate_alarm_threshold: be_u8
            >> sep
            >> high_expiratory_rate_alarm_threshold: be_u8
            >> end
            >> (TelemetryMessage::MachineStateSnapshot(MachineStateSnapshot {
                telemetry_version: VERSION,
                version: software_version.to_string(),
                device_id: format!("{}-{}-{}", device_id1, device_id2, device_id3),
                systick,
                cycle,
                peak_command,
                plateau_command,
                peep_command,
                cpm_command,
                previous_peak_pressure,
                previous_plateau_pressure,
                previous_peep_pressure,
                current_alarm_codes,
                previous_volume: if previous_volume == 0xFFFF {
                    None
                } else {
                    Some(previous_volume)
                },
                expiratory_term,
                trigger_enabled: trigger_enabled != 0,
                trigger_offset,
                previous_cpm: Some(previous_cpm),
                alarm_snoozed: Some(alarm_snoozed != 0),
                cpu_load: Some(cpu_load),
                ventilation_mode,
                inspiratory_trigger_flow: Some(inspiratory_trigger_flow),
                expiratory_trigger_flow: Some(expiratory_trigger_flow),
                ti_min: Some(ti_min),
                ti_max: Some(ti_max),
                low_inspiratory_minute_volume_alarm_threshold: Some(
                    low_inspiratory_minute_volume_alarm_threshold
                ),
                high_inspiratory_minute_volume_alarm_threshold: Some(
                    high_inspiratory_minute_volume_alarm_threshold
                ),
                low_expiratory_minute_volume_alarm_threshold: Some(
                    low_expiratory_minute_volume_alarm_threshold
                ),
                high_expiratory_minute_volume_alarm_threshold: Some(
                    high_expiratory_minute_volume_alarm_threshold
                ),
                low_expiratory_rate_alarm_threshold: Some(low_expiratory_rate_alarm_threshold),
                high_expiratory_rate_alarm_threshold: Some(high_expiratory_rate_alarm_threshold),
            }))
    )
);

named!(
    alarm_trap<TelemetryMessage>,
    do_parse!(
        tag!("T:")
            >> tag!([VERSION])
            >> software_version_len: be_u8
            >> software_version:
                map_res!(take!(software_version_len), |bytes| std::str::from_utf8(
                    bytes
                ))
            >> device_id1: be_u32
            >> device_id2: be_u32
            >> device_id3: be_u32
            >> sep
            >> systick: be_u64
            >> sep
            >> centile: be_u16
            >> sep
            >> pressure: be_i16
            >> sep
            >> phase: phase
            >> sep
            >> cycle: be_u32
            >> sep
            >> alarm_code: be_u8
            >> sep
            >> alarm_priority: alarm_priority
            >> sep
            >> triggered: triggered
            >> sep
            >> expected: be_u32
            >> sep
            >> measured: be_u32
            >> sep
            >> cycles_since_trigger: be_u32
            >> end
            >> (TelemetryMessage::AlarmTrap(AlarmTrap {
                telemetry_version: VERSION,
                version: software_version.to_string(),
                device_id: format!("{}-{}-{}", device_id1, device_id2, device_id3),
                systick,
                centile,
                pressure,
                phase,
                subphase: None,
                cycle,
                alarm_code,
                alarm_priority,
                triggered,
                expected,
                measured,
                cycles_since_trigger,
            }))
    )
);

named!(
    control_ack<TelemetryMessage>,
    do_parse!(
        tag!("A:")
            >> tag!([VERSION])
            >> software_version_len: be_u8
            >> software_version:
                map_res!(take!(software_version_len), |bytes| std::str::from_utf8(
                    bytes
                ))
            >> device_id1: be_u32
            >> device_id2: be_u32
            >> device_id3: be_u32
            >> sep
            >> systick: be_u64
            >> sep
            >> setting: control_setting
            >> sep
            >> value: be_u16
            >> end
            >> (TelemetryMessage::ControlAck(ControlAck {
                telemetry_version: VERSION,
                version: software_version.to_string(),
                device_id: format!("{}-{}-{}", device_id1, device_id2, device_id3),
                systick,
                setting,
                value,
            }))
    )
);

/// Transform bytes into a structured telemetry message
///
/// * `input` - Bytes to parse.
///
/// This only decodes the message body: header, CRC and footer must be stripped beforehand.
pub fn message(input: &[u8]) -> IResult<&[u8], TelemetryMessage, TelemetryError<&[u8]>> {
    nom::branch::alt((
        boot,
        stopped,
        data_snapshot,
        machine_state_snapshot,
        alarm_trap,
        control_ack,
    ))(input)
    .map_err(nom::Err::convert)
}

#[cfg(test)]
mod tests {
    use super::super::tests::*;
    use super::*;
    use proptest::bool;
    use proptest::collection;
    use proptest::num;
    use proptest::option;
    use proptest::prelude::*;

    fn phase_strategy() -> impl Strategy<Value = Phase> {
        prop_oneof![Just(Phase::Inhalation), Just(Phase::Exhalation)]
    }

    fn phase_value(phase: Phase) -> u8 {
        match phase {
            Phase::Inhalation => 17,
            Phase::Exhalation => 68,
        }
    }

    fn alarm_priority_strategy() -> impl Strategy<Value = AlarmPriority> {
        prop_oneof![
            Just(AlarmPriority::Low),
            Just(AlarmPriority::Medium),
            Just(AlarmPriority::High),
        ]
    }

    fn control_setting_strategy() -> impl Strategy<Value = ControlSetting> {
        proptest::num::u8::ANY.prop_filter_map("Invalid control setting", |n| {
            ControlSetting::try_from(n).ok()
        })
    }

    fn alarm_priority_value(m: &AlarmPriority) -> u8 {
        match m {
            AlarmPriority::High => 4,
            AlarmPriority::Medium => 2,
            AlarmPriority::Low => 1,
        }
    }

    fn ventilation_mode_strategy() -> impl Strategy<Value = VentilationMode> {
        prop_oneof![
            Just(VentilationMode::PC_CMV),
            Just(VentilationMode::PC_AC),
            Just(VentilationMode::VC_CMV),
            Just(VentilationMode::PC_BIPAP),
        ]
    }

    fn ventilation_mode_value(m: &VentilationMode) -> u8 {
        m.into()
    }

    proptest! {
        #[test]
        fn test_boot_message_parser(
            version in ".*",
            device_id1 in (0u32..),
            device_id2 in (0u32..),
            device_id3 in (0u32..),
            systick in (0u64..),
            mode in mode_strategy(),
            value128 in (0u8..),
        ) {
            let msg = BootMessage {
                telemetry_version: VERSION,
                version,
                device_id: format!("{}-{}-{}", device_id1, device_id2, device_id3),
                systick,
                mode,
                value128,
            };

            // This needs to be consistent with sendBootMessage() defined in src/software/firmware/srcs/telemetry.cpp
            let input = &flat(&[
                b"B:",
                &[VERSION],
                &[msg.version.len() as u8],
                &msg.version.as_bytes(),
                &device_id1.to_be_bytes(),
                &device_id2.to_be_bytes(),
                &device_id3.to_be_bytes(),
                b"\t",
                &msg.systick.to_be_bytes(),
                b"\t",
                &[mode_ordinal(&msg.mode)],
                b"\t",
                &[msg.value128],
                b"\n",
            ]);

            let expected = TelemetryMessage::BootMessage(msg);
            assert_eq!(nom::dbg_dmp(boot, "boot")(input), Ok((&[][..], expected)));
        }
    }

    proptest! {
        #[test]
        fn test_stopped_message_parser(
            version in ".*",
            device_id1 in (0u32..),
            device_id2 in (0u32..),
            device_id3 in (0u32..),
            systick in (0u64..),
            peak_command in (0u8..),
            plateau_command in (0u8..),
            peep_command in (0u8..),
            cpm_command in (0u8..),
            expiratory_term in (0u8..),
            trigger_enabled in bool::ANY,
            trigger_offset in (0u8..),
            alarm_snoozed in bool::ANY,
            cpu_load in num::u8::ANY,
            ventilation_mode in ventilation_mode_strategy(),
            inspiratory_trigger_flow in num::u8::ANY,
            expiratory_trigger_flow in num::u8::ANY,
            ti_min in num::u16::ANY,
            ti_max in num::u16::ANY,
            low_inspiratory_minute_volume_alarm_threshold in num::u8::ANY,
            high_inspiratory_minute_volume_alarm_threshold in num::u8::ANY,
            low_expiratory_minute_volume_alarm_threshold in num::u8::ANY,
            high_expiratory_minute_volume_alarm_threshold in num::u8::ANY,
            low_expiratory_rate_alarm_threshold in num::u8::ANY,
            high_expiratory_rate_alarm_threshold in num::u8::ANY,
        ) {
            let msg = StoppedMessage {
                telemetry_version: VERSION,
                version,
                device_id: format!("{}-{}-{}", device_id1, device_id2, device_id3),
                systick,
                peak_command: Some(peak_command),
                plateau_command: Some(plateau_command),
                peep_command: Some(peep_command),
                cpm_command: Some(cpm_command),
                expiratory_term: Some(expiratory_term),
                trigger_enabled: Some(trigger_enabled),
                trigger_offset: Some(trigger_offset),
                alarm_snoozed: Some(alarm_snoozed),
                cpu_load: Some(cpu_load),
                ventilation_mode,
                inspiratory_trigger_flow: Some(inspiratory_trigger_flow),
                expiratory_trigger_flow: Some(expiratory_trigger_flow),
                ti_min: Some(ti_min),
                ti_max: Some(ti_max),
                low_inspiratory_minute_volume_alarm_threshold: Some(low_inspiratory_minute_volume_alarm_threshold),
                high_inspiratory_minute_volume_alarm_threshold: Some(high_inspiratory_minute_volume_alarm_threshold),
                low_expiratory_minute_volume_alarm_threshold: Some(low_expiratory_minute_volume_alarm_threshold),
                high_expiratory_minute_volume_alarm_threshold: Some(high_expiratory_minute_volume_alarm_threshold),
                low_expiratory_rate_alarm_threshold: Some(low_expiratory_rate_alarm_threshold),
                high_expiratory_rate_alarm_threshold: Some(high_expiratory_rate_alarm_threshold),
            };

            // This needs to be consistent with sendStoppedMessage() defined in src/software/firmware/srcs/telemetry.cpp
            let input = &flat(&[
                b"O:",
                &[VERSION],
                &[msg.version.len() as u8],
                &msg.version.as_bytes(),
                &device_id1.to_be_bytes(),
                &device_id2.to_be_bytes(),
                &device_id3.to_be_bytes(),
                b"\t",
                &msg.systick.to_be_bytes(),
                b"\t",
                &[msg.peak_command.unwrap_or_default()],
                b"\t",
                &[msg.plateau_command.unwrap_or_default()],
                b"\t",
                &[msg.peep_command.unwrap_or_default()],
                b"\t",
                &[msg.cpm_command.unwrap_or_default()],
                b"\t",
                &msg.expiratory_term.unwrap_or_default().to_be_bytes(),
                b"\t",
                if msg.trigger_enabled.unwrap_or_default() { b"\x01" } else { b"\x00" },
                b"\t",
                &msg.trigger_offset.unwrap_or_default().to_be_bytes(),
                b"\t",
                if msg.alarm_snoozed.unwrap_or_default() { b"\x01" } else { b"\x00" },
                b"\t",
                &msg.cpu_load.unwrap_or_default().to_be_bytes(),
                b"\t",
                &[ventilation_mode_value(&msg.ventilation_mode)],
                b"\t",
                &msg.inspiratory_trigger_flow.unwrap_or_default().to_be_bytes(),
                b"\t",
                &msg.expiratory_trigger_flow.unwrap_or_default().to_be_bytes(),
                b"\t",
                &msg.ti_min.unwrap_or_default().to_be_bytes(),
                b"\t",
                &msg.ti_max.unwrap_or_default().to_be_bytes(),
                b"\t",
                &msg.low_inspiratory_minute_volume_alarm_threshold.unwrap_or_default().to_be_bytes(),
                b"\t",
                &msg.high_inspiratory_minute_volume_alarm_threshold.unwrap_or_default().to_be_bytes(),
                b"\t",
                &msg.low_expiratory_minute_volume_alarm_threshold.unwrap_or_default().to_be_bytes(),
                b"\t",
                &msg.high_expiratory_minute_volume_alarm_threshold.unwrap_or_default().to_be_bytes(),
                b"\t",
                &msg.low_expiratory_rate_alarm_threshold.unwrap_or_default().to_be_bytes(),
                b"\t",
                &msg.high_expiratory_rate_alarm_threshold.unwrap_or_default().to_be_bytes(),
                b"\n",
            ]);

            let expected = TelemetryMessage::StoppedMessage(msg);
            assert_eq!(nom::dbg_dmp(stopped, "stopped")(input), Ok((&[][..], expected)));
        }
    }

    proptest! {
        #[test]
        fn test_data_snapshot_message_parser(
            version in ".*",
            device_id1 in (0u32..),
            device_id2 in (0u32..),
            device_id3 in (0u32..),
            systick in (0u64..),
            centile in (0u16..),
            pressure in (0i16..),
            phase in phase_strategy(),
            blower_valve_position in (0u8..),
            patient_valve_position in (0u8..),
            blower_rpm in (0u8..),
            battery_level in (0u8..),
            inspiratory_flow in num::i16::ANY,
            expiratory_flow in num::i16::ANY,
        ) {
            let msg = DataSnapshot {
                telemetry_version: VERSION,
                version,
                device_id: format!("{}-{}-{}", device_id1, device_id2, device_id3),
                systick,
                centile,
                pressure,
                phase: phase.clone(),
                subphase: None,
                blower_valve_position,
                patient_valve_position,
                blower_rpm,
                battery_level,
                inspiratory_flow: Some(inspiratory_flow),
                expiratory_flow: Some(expiratory_flow),
            };

            // This needs to be consistent with sendDataSnapshot() defined in src/software/firmware/srcs/telemetry.cpp
            let input = &flat(&[
                b"D:",
                &[VERSION],
                &[msg.version.len() as u8],
                &msg.version.as_bytes(),
                &device_id1.to_be_bytes(),
                &device_id2.to_be_bytes(),
                &device_id3.to_be_bytes(),
                b"\t",
                &msg.systick.to_be_bytes(),
                b"\t",
                &msg.centile.to_be_bytes(),
                b"\t",
                &msg.pressure.to_be_bytes(),
                b"\t",
                &[phase_value(phase)],
                b"\t",
                &[msg.blower_valve_position],
                b"\t",
                &[msg.patient_valve_position],
                b"\t",
                &[msg.blower_rpm],
                b"\t",
                &[msg.battery_level],
                b"\t",
                &msg.inspiratory_flow.unwrap_or_default().to_be_bytes(),
                b"\t",
                &msg.expiratory_flow.unwrap_or_default().to_be_bytes(),
                b"\n",
            ]);

            let expected = TelemetryMessage::DataSnapshot(msg);
            assert_eq!(nom::dbg_dmp(data_snapshot, "data_snapshot")(input), Ok((&[][..], expected)));
        }
    }

    proptest! {
        #[test]
        fn test_machine_state_snapshot_message_parser(
            version in ".*",
            device_id1 in (0u32..),
            device_id2 in (0u32..),
            device_id3 in (0u32..),
            systick in (0u64..),
            cycle in (0u32..),
            peak_command in (0u8..),
            plateau_command in (0u8..),
            peep_command in (0u8..),
            cpm_command in (0u8..),
            previous_peak_pressure in (0u16..),
            previous_plateau_pressure in (0u16..),
            previous_peep_pressure in (0u16..),
            current_alarm_codes in collection::vec(0u8.., 0..100),
            previous_volume in option::of(0u16..0xFFFE),
            expiratory_term in (0u8..),
            trigger_enabled in bool::ANY,
            trigger_offset in (0u8..),
            previous_cpm in (0u8..),
            alarm_snoozed in bool::ANY,
            cpu_load in num::u8::ANY,
            ventilation_mode in ventilation_mode_strategy(),
            inspiratory_trigger_flow in num::u8::ANY,
            expiratory_trigger_flow in num::u8::ANY,
            ti_min in num::u16::ANY,
            ti_max in num::u16::ANY,
            low_inspiratory_minute_volume_alarm_threshold in num::u8::ANY,
            high_inspiratory_minute_volume_alarm_threshold in num::u8::ANY,
            low_expiratory_minute_volume_alarm_threshold in num::u8::ANY,
            high_expiratory_minute_volume_alarm_threshold in num::u8::ANY,
            low_expiratory_rate_alarm_threshold in num::u8::ANY,
            high_expiratory_rate_alarm_threshold in num::u8::ANY,
        ) {
            let msg = MachineStateSnapshot {
                telemetry_version: VERSION,
                version,
                device_id: format!("{}-{}-{}", device_id1, device_id2, device_id3),
                systick,
                cycle,
                peak_command,
                plateau_command,
                peep_command,
                cpm_command,
                previous_peak_pressure,
                previous_plateau_pressure,
                previous_peep_pressure,
                current_alarm_codes,
                previous_volume,
                expiratory_term,
                trigger_enabled,
                trigger_offset,
                previous_cpm: Some(previous_cpm),
                alarm_snoozed: Some(alarm_snoozed),
                cpu_load: Some(cpu_load),
                ventilation_mode,
                inspiratory_trigger_flow: Some(inspiratory_trigger_flow),
                expiratory_trigger_flow: Some(expiratory_trigger_flow),
                ti_min: Some(ti_min),
                ti_max: Some(ti_max),
                low_inspiratory_minute_volume_alarm_threshold: Some(low_inspiratory_minute_volume_alarm_threshold),
                high_inspiratory_minute_volume_alarm_threshold: Some(high_inspiratory_minute_volume_alarm_threshold),
                low_expiratory_minute_volume_alarm_threshold: Some(low_expiratory_minute_volume_alarm_threshold),
                high_expiratory_minute_volume_alarm_threshold: Some(high_expiratory_minute_volume_alarm_threshold),
                low_expiratory_rate_alarm_threshold: Some(low_expiratory_rate_alarm_threshold),
                high_expiratory_rate_alarm_threshold: Some(high_expiratory_rate_alarm_threshold),
            };

            // This needs to be consistent with sendMachineStateSnapshot() defined in src/software/firmware/srcs/telemetry.cpp
            let input = &flat(&[
                b"S:",
                &[VERSION],
                &[msg.version.len() as u8],
                &msg.version.as_bytes(),
                &device_id1.to_be_bytes(),
                &device_id2.to_be_bytes(),
                &device_id3.to_be_bytes(),
                b"\t",
                &msg.systick.to_be_bytes(),
                b"\t",
                &msg.cycle.to_be_bytes(),
                b"\t",
                &[msg.peak_command],
                b"\t",
                &[msg.plateau_command],
                b"\t",
                &[msg.peep_command],
                b"\t",
                &[msg.cpm_command],
                b"\t",
                &msg.previous_peak_pressure.to_be_bytes(),
                b"\t",
                &msg.previous_plateau_pressure.to_be_bytes(),
                b"\t",
                &msg.previous_peep_pressure.to_be_bytes(),
                b"\t",
                &[msg.current_alarm_codes.len() as u8],
                &msg.current_alarm_codes,
                b"\t",
                &msg.previous_volume.unwrap_or(0xFFFF).to_be_bytes(),
                b"\t",
                &msg.expiratory_term.to_be_bytes(),
                b"\t",
                if msg.trigger_enabled { b"\x01" } else { b"\x00" },
                b"\t",
                &msg.trigger_offset.to_be_bytes(),
                b"\t",
                &msg.previous_cpm.unwrap_or_default().to_be_bytes(),
                b"\t",
                if msg.alarm_snoozed.unwrap_or_default() { b"\x01" } else { b"\x00" },
                b"\t",
                &msg.cpu_load.unwrap_or_default().to_be_bytes(),
                b"\t",
                &[ventilation_mode_value(&msg.ventilation_mode)],
                b"\t",
                &msg.inspiratory_trigger_flow.unwrap_or_default().to_be_bytes(),
                b"\t",
                &msg.expiratory_trigger_flow.unwrap_or_default().to_be_bytes(),
                b"\t",
                &msg.ti_min.unwrap_or_default().to_be_bytes(),
                b"\t",
                &msg.ti_max.unwrap_or_default().to_be_bytes(),
                b"\t",
                &msg.low_inspiratory_minute_volume_alarm_threshold.unwrap_or_default().to_be_bytes(),
                b"\t",
                &msg.high_inspiratory_minute_volume_alarm_threshold.unwrap_or_default().to_be_bytes(),
                b"\t",
                &msg.low_expiratory_minute_volume_alarm_threshold.unwrap_or_default().to_be_bytes(),
                b"\t",
                &msg.high_expiratory_minute_volume_alarm_threshold.unwrap_or_default().to_be_bytes(),
                b"\t",
                &msg.low_expiratory_rate_alarm_threshold.unwrap_or_default().to_be_bytes(),
                b"\t",
                &msg.high_expiratory_rate_alarm_threshold.unwrap_or_default().to_be_bytes(),
                b"\n",
            ]);

            let expected = TelemetryMessage::MachineStateSnapshot(msg);
            assert_eq!(nom::dbg_dmp(machine_state_snapshot, "machine_state_snapshot")(input), Ok((&[][..], expected)));
        }
    }

    proptest! {
        #[test]
        fn test_alarm_trap_message_parser(
            version in ".*",
            device_id1 in (0u32..),
            device_id2 in (0u32..),
            device_id3 in (0u32..),
            systick in (0u64..),
            centile in (0u16..),
            pressure in (0i16..),
            phase in phase_strategy(),
            cycle in (0u32..),
            alarm_code in (0u8..),
            alarm_priority in alarm_priority_strategy(),
            triggered in proptest::bool::ANY,
            expected in (0u32..),
            measured in (0u32..),
            cycles_since_trigger in (0u32..),
        ) {
            let msg = AlarmTrap {
                telemetry_version: VERSION,
                version,
                device_id: format!("{}-{}-{}", device_id1, device_id2, device_id3),
                systick,
                centile,
                pressure,
                phase: phase.clone(),
                subphase: None,
                cycle,
                alarm_code,
                alarm_priority,
                triggered,
                expected,
                measured,
                cycles_since_trigger,
            };

            // This needs to be consistent with sendAlarmTrap() defined in src/software/firmware/srcs/telemetry.cpp
            let input = &flat(&[
                b"T:",
                &[VERSION],
                &[msg.version.len() as u8],
                &msg.version.as_bytes(),
                &device_id1.to_be_bytes(),
                &device_id2.to_be_bytes(),
                &device_id3.to_be_bytes(),
                b"\t",
                &msg.systick.to_be_bytes(),
                b"\t",
                &msg.centile.to_be_bytes(),
                b"\t",
                &msg.pressure.to_be_bytes(),
                b"\t",
                &[phase_value(phase)],
                b"\t",
                &msg.cycle.to_be_bytes(),
                b"\t",
                &[msg.alarm_code],
                b"\t",
                &[alarm_priority_value(&msg.alarm_priority)],
                b"\t",
                &[if msg.triggered { 240u8 } else { 15u8 }],
                b"\t",
                &msg.expected.to_be_bytes(),
                b"\t",
                &msg.measured.to_be_bytes(),
                b"\t",
                &msg.cycles_since_trigger.to_be_bytes(),
                b"\n",
            ]);

            let expected = TelemetryMessage::AlarmTrap(msg);
            assert_eq!(nom::dbg_dmp(alarm_trap, "alarm_trap")(input), Ok((&[][..], expected)));
        }
    }

    proptest! {
        #[test]
        fn test_control_ack_message_parser(
            version in ".*",
            device_id1 in (0u32..),
            device_id2 in (0u32..),
            device_id3 in (0u32..),
            systick in (0u64..),
            setting in control_setting_strategy(),
            value in (0u16..),
        ) {
            let msg = ControlAck {
                telemetry_version: VERSION,
                version,
                device_id: format!("{}-{}-{}", device_id1, device_id2, device_id3),
                systick,
                setting,
                value,
            };

            // This needs to be consistent with sendAlarmTrap() defined in src/software/firmware/srcs/telemetry.cpp
            let input = &flat(&[
                b"A:",
                &[VERSION],
                &[msg.version.len() as u8],
                &msg.version.as_bytes(),
                &device_id1.to_be_bytes(),
                &device_id2.to_be_bytes(),
                &device_id3.to_be_bytes(),
                b"\t",
                &msg.systick.to_be_bytes(),
                b"\t",
                &(msg.setting as u8).to_be_bytes(),
                b"\t",
                &msg.value.to_be_bytes(),
                b"\n",
            ]);

            let expected = TelemetryMessage::ControlAck(msg);
            assert_eq!(nom::dbg_dmp(control_ack, "control_ack")(input), Ok((&[][..], expected)));
        }
    }
}
