use nom::number::streaming::{be_u16, be_u32, be_u64, be_u8};
use nom::IResult;
use nom::{alt, do_parse, length_data, map, map_res, named, tag, take};
use std::convert::TryFrom;

use crate::control::*;
use crate::structures::*;

const VERSION: u8 = 1;

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
    phase_and_subphase<(Phase, SubPhase)>,
    alt!(
        map!(tag!([17u8]), |_| (Phase::Inhalation, SubPhase::Inspiration))
            | map!(tag!([18u8]), |_| (
                Phase::Inhalation,
                SubPhase::HoldInspiration
            ))
            | map!(tag!([68u8]), |_| (Phase::Exhalation, SubPhase::Exhale))
    )
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
            >> end
            >> ({
                TelemetryMessage::StoppedMessage(StoppedMessage {
                    telemetry_version: VERSION,
                    version: software_version.to_string(),
                    device_id: format!("{}-{}-{}", device_id1, device_id2, device_id3),
                    systick,
                    peak_command: None,
                    plateau_command: None,
                    peep_command: None,
                    cpm_command: None,
                    expiratory_term: None,
                    trigger_enabled: None,
                    trigger_offset: None,
                    alarm_snoozed: None,
                    cpu_load: None,
                    ventilation_mode: VentilationMode::default(),
                    inspiratory_trigger_flow: None,
                    expiratory_trigger_flow: None,
                    ti_min: None,
                    ti_max: None,
                    low_inspiratory_minute_volume_alarm_threshold: None,
                    high_inspiratory_minute_volume_alarm_threshold: None,
                    low_expiratory_minute_volume_alarm_threshold: None,
                    high_expiratory_minute_volume_alarm_threshold: None,
                    low_respiratory_rate_alarm_threshold: None,
                    high_respiratory_rate_alarm_threshold: None,
                    target_tidal_volume: None,
                    low_tidal_volume_alarm_threshold: None,
                    high_tidal_volume_alarm_threshold: None,
                    plateau_duration: None,
                    leak_alarm_threshold: None,
                    target_inspiratory_flow: None,
                    inspiratory_duration_command: None,
                    battery_level: None,
                    current_alarm_codes: None,
                    patient_height: None,
                    locale: None,
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
            >> pressure: be_u16
            >> sep
            >> phase_and_subphase: phase_and_subphase
            >> sep
            >> blower_valve_position: be_u8
            >> sep
            >> patient_valve_position: be_u8
            >> sep
            >> blower_rpm: be_u8
            >> sep
            >> battery_level: be_u8
            >> end
            >> (TelemetryMessage::DataSnapshot(DataSnapshot {
                telemetry_version: VERSION,
                version: software_version.to_string(),
                device_id: format!("{}-{}-{}", device_id1, device_id2, device_id3),
                systick,
                centile,
                pressure: i16::try_from(pressure).unwrap_or(i16::MAX),
                phase: phase_and_subphase.0,
                subphase: Some(phase_and_subphase.1),
                blower_valve_position,
                patient_valve_position,
                blower_rpm,
                battery_level,
                inspiratory_flow: None,
                expiratory_flow: None,
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
                previous_cpm: None,
                alarm_snoozed: None,
                cpu_load: None,
                ventilation_mode: VentilationMode::default(),
                inspiratory_trigger_flow: None,
                expiratory_trigger_flow: None,
                ti_min: None,
                ti_max: None,
                low_inspiratory_minute_volume_alarm_threshold: None,
                high_inspiratory_minute_volume_alarm_threshold: None,
                low_expiratory_minute_volume_alarm_threshold: None,
                high_expiratory_minute_volume_alarm_threshold: None,
                low_respiratory_rate_alarm_threshold: None,
                high_respiratory_rate_alarm_threshold: None,
                target_tidal_volume: None,
                low_tidal_volume_alarm_threshold: None,
                high_tidal_volume_alarm_threshold: None,
                plateau_duration: None,
                leak_alarm_threshold: None,
                target_inspiratory_flow: None,
                inspiratory_duration_command: None,
                previous_inspiratory_duration: None,
                battery_level: None,
                patient_height: None,
                locale: None,
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
            >> pressure: be_u16
            >> sep
            >> phase_and_subphase: phase_and_subphase
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
                pressure: i16::try_from(pressure).unwrap_or(i16::MAX),
                phase: phase_and_subphase.0,
                subphase: Some(phase_and_subphase.1),
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
    use crate::serializers::ToBytes;
    use proptest::bool;
    use proptest::collection;
    use proptest::option;
    use proptest::prelude::*;

    // TODO Generate all combinations (independent each other) ?
    fn phase_subphase_strategy() -> impl Strategy<Value = (Phase, SubPhase)> {
        prop_oneof![
            (Just(Phase::Inhalation), Just(SubPhase::Inspiration)),
            (Just(Phase::Inhalation), Just(SubPhase::HoldInspiration)),
            (Just(Phase::Exhalation), Just(SubPhase::Exhale)),
        ]
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
            let input = &msg.to_bytes_v1();
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
        ) {
            let msg = StoppedMessage {
                telemetry_version: VERSION,
                version,
                device_id: format!("{}-{}-{}", device_id1, device_id2, device_id3),
                systick,
                peak_command: None,
                plateau_command: None,
                peep_command: None,
                cpm_command: None,
                expiratory_term: None,
                trigger_enabled: None,
                trigger_offset: None,
                alarm_snoozed: None,
                cpu_load: None,
                ventilation_mode: VentilationMode::default(),
                inspiratory_trigger_flow: None,
                expiratory_trigger_flow: None,
                ti_min: None,
                ti_max: None,
                low_inspiratory_minute_volume_alarm_threshold: None,
                high_inspiratory_minute_volume_alarm_threshold: None,
                low_expiratory_minute_volume_alarm_threshold: None,
                high_expiratory_minute_volume_alarm_threshold: None,
                low_respiratory_rate_alarm_threshold: None,
                high_respiratory_rate_alarm_threshold: None,
                target_tidal_volume: None,
                low_tidal_volume_alarm_threshold: None,
                high_tidal_volume_alarm_threshold: None,
                plateau_duration: None,
                leak_alarm_threshold: None,
                target_inspiratory_flow: None,
                inspiratory_duration_command: None,
                battery_level: None,
                current_alarm_codes: None,
                patient_height: None,
                locale: None,
            };
            let input = &msg.to_bytes_v1();
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
            pressure in (0u16..),
            phase_subphase in phase_subphase_strategy(),
            blower_valve_position in (0u8..),
            patient_valve_position in (0u8..),
            blower_rpm in (0u8..),
            battery_level in (0u8..),
        ) {
            let msg = DataSnapshot {
                telemetry_version: VERSION,
                version,
                device_id: format!("{}-{}-{}", device_id1, device_id2, device_id3),
                systick,
                centile,
                pressure: i16::try_from(pressure).unwrap_or(i16::MAX),
                phase: phase_subphase.0,
                subphase: Some(phase_subphase.1),
                blower_valve_position,
                patient_valve_position,
                blower_rpm,
                battery_level,
                inspiratory_flow: None,
                expiratory_flow: None,
            };
            let input = &msg.to_bytes_v1();
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
                previous_cpm: None,
                alarm_snoozed: None,
                cpu_load: None,
                ventilation_mode: VentilationMode::default(),
                inspiratory_trigger_flow: None,
                expiratory_trigger_flow: None,
                ti_min: None,
                ti_max: None,
                low_inspiratory_minute_volume_alarm_threshold: None,
                high_inspiratory_minute_volume_alarm_threshold: None,
                low_expiratory_minute_volume_alarm_threshold: None,
                high_expiratory_minute_volume_alarm_threshold: None,
                low_respiratory_rate_alarm_threshold: None,
                high_respiratory_rate_alarm_threshold: None,
                target_tidal_volume: None,
                low_tidal_volume_alarm_threshold: None,
                high_tidal_volume_alarm_threshold: None,
                plateau_duration: None,
                leak_alarm_threshold: None,
                target_inspiratory_flow: None,
                inspiratory_duration_command: None,
                previous_inspiratory_duration: None,
                battery_level: None,
                patient_height: None,
                locale: None,
            };
            let input = &msg.to_bytes_v1();
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
            pressure in (0u16..),
            phase_subphase in phase_subphase_strategy(),
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
                pressure: i16::try_from(pressure).unwrap_or(i16::MAX),
                phase: phase_subphase.0,
                subphase: Some(phase_subphase.1),
                cycle,
                alarm_code,
                alarm_priority,
                triggered,
                expected,
                measured,
                cycles_since_trigger,
            };
            let input = &msg.to_bytes_v1();
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
            let input = &msg.to_bytes_v1();
            let expected = TelemetryMessage::ControlAck(msg);

            assert_eq!(nom::dbg_dmp(control_ack, "control_ack")(input), Ok((&[][..], expected)));
        }
    }
}
