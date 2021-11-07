use nom::branch::alt;
use nom::bytes::streaming::{tag, take};
use nom::combinator::{map, map_res};
use nom::error::{FromExternalError, ParseError};
use nom::multi::length_data;
use nom::number::streaming::{be_u16, be_u32, be_u64, be_u8};
use nom::sequence::tuple;
use nom::IResult;
use std::convert::TryFrom;

use crate::control::*;
use crate::structures::*;

const VERSION: u8 = 1;

fn sep<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], &[u8], E> {
    tag("\t")(input)
}

fn end<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], &[u8], E> {
    tag("\n")(input)
}

fn mode<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], Mode, E> {
    let mut parser = alt((
        map(tag(b"\x01"), |_| Mode::Production),
        map(tag(b"\x02"), |_| Mode::Qualification),
        map(tag(b"\x03"), |_| Mode::IntegrationTest),
    ));
    parser(input)
}

fn phase_and_subphase<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], (Phase, SubPhase), E> {
    let mut parser = alt((
        map(tag([17u8]), |_| (Phase::Inhalation, SubPhase::Inspiration)),
        map(tag([18u8]), |_| {
            (Phase::Inhalation, SubPhase::HoldInspiration)
        }),
        map(tag([68u8]), |_| (Phase::Exhalation, SubPhase::Exhale)),
    ));
    parser(input)
}

fn control_setting<'a, E: ParseError<&'a [u8]> + FromExternalError<&'a [u8], E>>(
    input: &'a [u8],
) -> IResult<&'a [u8], ControlSetting, E> {
    let mut parser = map_res(be_u8, |b| {
        ControlSetting::try_from(b)
            .map_err(|_e| E::from_error_kind(input, nom::error::ErrorKind::Fail))
    });
    parser(input)
}

fn alarm_priority<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], AlarmPriority, E> {
    let mut parser = alt((
        map(tag([4u8]), |_| AlarmPriority::High),
        map(tag([2u8]), |_| AlarmPriority::Medium),
        map(tag([1u8]), |_| AlarmPriority::Low),
    ));
    parser(input)
}

fn u8_array<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], Vec<u8>, E> {
    let mut parser = map(length_data(be_u8), Vec::from);
    parser(input)
}

fn triggered<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], bool, E> {
    let mut parser = alt((map(tag([240u8]), |_| true), map(tag([15u8]), |_| false)));
    parser(input)
}

fn software_version<'a, E: ParseError<&'a [u8]> + FromExternalError<&'a [u8], E>>(
    input: &'a [u8],
) -> IResult<&'a [u8], &str, E> {
    let (rest, len) = be_u8(input)?;
    let mut parser = map_res(take(len), |bytes| {
        std::str::from_utf8(bytes)
            .map_err(|_e| E::from_error_kind(input, nom::error::ErrorKind::Fail))
    });
    parser(rest)
}

fn device_id<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], String, E> {
    let mut parser = map(tuple((be_u32, be_u32, be_u32)), |(p1, p2, p3)| {
        format!("{}-{}-{}", p1, p2, p3)
    });
    parser(input)
}

fn boot<'a, E: ParseError<&'a [u8]> + FromExternalError<&'a [u8], E>>(
    input: &'a [u8],
) -> IResult<&'a [u8], TelemetryMessage, E> {
    let mut parser = map(
        tuple((
            tag("B:"),
            tag([VERSION]),
            software_version,
            device_id,
            sep,
            be_u64,
            sep,
            mode,
            sep,
            be_u8,
            end,
        )),
        |(_, _, software_version, device_id, _, systick, _, mode, _, value128, _)| {
            TelemetryMessage::BootMessage(BootMessage {
                telemetry_version: VERSION,
                version: software_version.to_owned(),
                device_id,
                systick,
                mode,
                value128,
            })
        },
    );
    parser(input)
}

fn stopped<'a, E: ParseError<&'a [u8]> + FromExternalError<&'a [u8], E>>(
    input: &'a [u8],
) -> IResult<&'a [u8], TelemetryMessage, E> {
    let mut parser = map(
        tuple((
            tag("O:"),
            tag([VERSION]),
            software_version,
            device_id,
            sep,
            be_u64,
            end,
        )),
        |(_, _, software_version, device_id, _, systick, _)| {
            TelemetryMessage::StoppedMessage(StoppedMessage {
                telemetry_version: VERSION,
                version: software_version.to_owned(),
                device_id,
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
                locale: None,
                patient_height: None,
                patient_gender: None,
                peak_pressure_alarm_threshold: None,
            })
        },
    );
    parser(input)
}

fn data_snapshot<'a, E: ParseError<&'a [u8]> + FromExternalError<&'a [u8], E>>(
    input: &'a [u8],
) -> IResult<&'a [u8], TelemetryMessage, E> {
    let mut parser = map(
        tuple((
            tag("D:"),
            tag([VERSION]),
            software_version,
            device_id,
            sep,
            be_u64,
            sep,
            be_u16,
            sep,
            be_u16,
            sep,
            phase_and_subphase,
            sep,
            be_u8,
            sep,
            be_u8,
            sep,
            be_u8,
            sep,
            be_u8,
            end,
        )),
        |(
            _,
            _,
            software_version,
            device_id,
            _,
            systick,
            _,
            centile,
            _,
            pressure,
            _,
            phase_and_subphase,
            _,
            blower_valve_position,
            _,
            patient_valve_position,
            _,
            blower_rpm,
            _,
            battery_level,
            _,
        )| {
            TelemetryMessage::DataSnapshot(DataSnapshot {
                telemetry_version: VERSION,
                version: software_version.to_owned(),
                device_id,
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
            })
        },
    );
    parser(input)
}

fn machine_state_snapshot<'a, E: ParseError<&'a [u8]> + FromExternalError<&'a [u8], E>>(
    input: &'a [u8],
) -> IResult<&'a [u8], TelemetryMessage, E> {
    let mut parser = map(
        tuple((
            tuple((
                tag("S:"),
                tag([VERSION]),
                software_version,
                device_id,
                sep,
                be_u64,
                sep,
                be_u32,
                sep,
                be_u8,
                sep,
                be_u8,
                sep,
                be_u8,
                sep,
                be_u8,
                sep,
                be_u16,
                sep,
                be_u16,
                sep,
            )),
            tuple((
                be_u16, sep, u8_array, sep, be_u16, sep, be_u8, sep, be_u8, sep, be_u8, end,
            )),
        )),
        |(
            (
                _,
                _,
                software_version,
                device_id,
                _,
                systick,
                _,
                cycle,
                _,
                peak_command,
                _,
                plateau_command,
                _,
                peep_command,
                _,
                cpm_command,
                _,
                previous_peak_pressure,
                _,
                previous_plateau_pressure,
                _,
            ),
            (
                previous_peep_pressure,
                _,
                current_alarm_codes,
                _,
                previous_volume,
                _,
                expiratory_term,
                _,
                trigger_enabled,
                _,
                trigger_offset,
                _,
            ),
        )| {
            TelemetryMessage::MachineStateSnapshot(MachineStateSnapshot {
                telemetry_version: VERSION,
                version: software_version.to_owned(),
                device_id,
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
                locale: None,
                patient_height: None,
                patient_gender: None,
                peak_pressure_alarm_threshold: None,
            })
        },
    );
    parser(input)
}

fn alarm_trap<'a, E: ParseError<&'a [u8]> + FromExternalError<&'a [u8], E>>(
    input: &'a [u8],
) -> IResult<&'a [u8], TelemetryMessage, E> {
    let mut parser = map(
        tuple((
            tuple((
                tag("T:"),
                tag([VERSION]),
                software_version,
                device_id,
                sep,
                be_u64,
                sep,
                be_u16,
                sep,
                be_u16,
                sep,
                phase_and_subphase,
                sep,
                be_u32,
                sep,
                be_u8,
                sep,
                alarm_priority,
                sep,
                triggered,
                sep,
            )),
            tuple((be_u32, sep, be_u32, sep, be_u32, end)),
        )),
        |(
            (
                _,
                _,
                software_version,
                device_id,
                _,
                systick,
                _,
                centile,
                _,
                pressure,
                _,
                phase_and_subphase,
                _,
                cycle,
                _,
                alarm_code,
                _,
                alarm_priority,
                _,
                triggered,
                _,
            ),
            (expected, _, measured, _, cycles_since_trigger, _),
        )| {
            TelemetryMessage::AlarmTrap(AlarmTrap {
                telemetry_version: VERSION,
                version: software_version.to_owned(),
                device_id,
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
            })
        },
    );
    parser(input)
}

fn control_ack<'a, E: ParseError<&'a [u8]> + FromExternalError<&'a [u8], E>>(
    input: &'a [u8],
) -> IResult<&'a [u8], TelemetryMessage, E> {
    let mut parser = map(
        tuple((
            tag("A:"),
            tag([VERSION]),
            software_version,
            device_id,
            sep,
            be_u64,
            sep,
            control_setting,
            sep,
            be_u16,
            end,
        )),
        |(_, _, software_version, device_id, _, systick, _, setting, _, value, _)| {
            TelemetryMessage::ControlAck(ControlAck {
                telemetry_version: VERSION,
                version: software_version.to_owned(),
                device_id,
                systick,
                setting,
                value,
            })
        },
    );
    parser(input)
}

/// Transform bytes into a structured telemetry message
///
/// * `input` - Bytes to parse.
///
/// This only decodes the message body: header, CRC and footer must be stripped beforehand.
pub fn message<'a, E: ParseError<&'a [u8]> + FromExternalError<&'a [u8], E>>(
    input: &'a [u8],
) -> IResult<&'a [u8], TelemetryMessage, E> {
    nom::branch::alt((
        boot,
        stopped,
        data_snapshot,
        machine_state_snapshot,
        alarm_trap,
        control_ack,
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::super::tests::*;
    use super::*;
    use crate::serializers::ToBytes;
    use nom::error::VerboseError;
    use proptest::bool;
    use proptest::collection;
    use proptest::option;
    use proptest::prelude::*;

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

            assert_eq!(nom::error::dbg_dmp(boot::<VerboseError<&[u8]>>, "boot")(input), Ok((&[][..], expected)));
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
                locale: None,
                patient_height: None,
                patient_gender: None,
                peak_pressure_alarm_threshold: None,
            };
            let input = &msg.to_bytes_v1();
            let expected = TelemetryMessage::StoppedMessage(msg);

            assert_eq!(nom::error::dbg_dmp(stopped::<VerboseError<&[u8]>>, "stopped")(input), Ok((&[][..], expected)));
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

            assert_eq!(nom::error::dbg_dmp(data_snapshot::<VerboseError<&[u8]>>, "data_snapshot")(input), Ok((&[][..], expected)));
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
                locale: None,
                patient_height: None,
                patient_gender: None,
                peak_pressure_alarm_threshold: None,
            };
            let input = &msg.to_bytes_v1();
            let expected = TelemetryMessage::MachineStateSnapshot(msg);

            assert_eq!(nom::error::dbg_dmp(machine_state_snapshot::<VerboseError<&[u8]>>, "machine_state_snapshot")(input), Ok((&[][..], expected)));
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

            assert_eq!(nom::error::dbg_dmp(alarm_trap::<VerboseError<&[u8]>>, "alarm_trap")(input), Ok((&[][..], expected)));
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

            assert_eq!(nom::error::dbg_dmp(control_ack::<VerboseError<&[u8]>>, "control_ack")(input), Ok((&[][..], expected)));
        }
    }
}
