use nom::number::streaming::{be_i16, be_u16, be_u32, be_u64, be_u8};
use nom::IResult;
use nom::{alt, do_parse, length_data, map, map_res, named, tag, take};
use std::convert::TryFrom;

use super::super::locale::Locale;
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

fn fatal_error_details(input: &[u8]) -> IResult<&[u8], FatalErrorDetails> {
    use nom::error::{Error, ErrorKind};
    use nom::Err::Failure;
    use FatalErrorDetails::*;

    let (input, error_type) = be_u8(input)?;
    match error_type {
        1 => Ok((input, WatchdogRestart)),
        2 => {
            do_parse!(
                input,
                sep >> pressure_offset: be_i16
                    >> sep
                    >> min_pressure: be_i16
                    >> sep
                    >> max_pressure: be_i16
                    >> sep
                    >> flow_at_starting: be_i16
                    >> sep
                    >> flow_with_blower_on: be_i16
                    >> (CalibrationError {
                        pressure_offset,
                        min_pressure,
                        max_pressure,
                        flow_at_starting: if flow_at_starting == i16::MAX {
                            None
                        } else {
                            Some(flow_at_starting)
                        },
                        flow_with_blower_on: if flow_with_blower_on == i16::MAX {
                            None
                        } else {
                            Some(flow_with_blower_on)
                        },
                    })
            )
        }
        3 => {
            do_parse!(
                input,
                sep >> battery_level: be_u16 >> (BatteryDeeplyDischarged { battery_level })
            )
        }
        4 => Ok((input, MassFlowMeterError)),
        5 => {
            do_parse!(
                input,
                sep >> pressure: be_u16 >> (InconsistentPressure { pressure })
            )
        }
        _ => Err(Failure(Error::new(input, ErrorKind::Switch))),
    }
}

named!(
    eol_test_step<EolTestStep>,
    map_res!(be_u8, |step| EolTestStep::try_from(step))
);

fn eol_test_snapshot_content(input: &[u8]) -> IResult<&[u8], EolTestSnapshotContent> {
    use nom::error::{Error, ErrorKind};
    use nom::Err::Failure;
    use EolTestSnapshotContent::*;

    let (input, content_type) = be_u8(input)?;
    match content_type {
        0 => Ok((input, InProgress)),
        1 => {
            do_parse!(
                input,
                sep >> error: u8_array >> (Error(String::from_utf8_lossy(&error).into_owned()))
            )
        }
        2 => Ok((input, Success)),
        _ => Err(Failure(Error::new(input, ErrorKind::Switch))),
    }
}

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
            >> low_respiratory_rate_alarm_threshold: be_u8
            >> sep
            >> high_respiratory_rate_alarm_threshold: be_u8
            >> sep
            >> target_tidal_volume: be_u16
            >> sep
            >> low_tidal_volume_alarm_threshold: be_u16
            >> sep
            >> high_tidal_volume_alarm_threshold: be_u16
            >> sep
            >> plateau_duration: be_u16
            >> sep
            >> leak_alarm_threshold: be_u16
            >> sep
            >> target_inspiratory_flow: be_u8
            >> sep
            >> inspiratory_duration_command: be_u16
            >> sep
            >> battery_level: be_u16
            >> sep
            >> current_alarm_codes: u8_array
            >> sep
            >> patient_height: be_u8
            >> sep
            >> locale: be_u16
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
                    low_respiratory_rate_alarm_threshold: Some(
                        low_respiratory_rate_alarm_threshold,
                    ),
                    high_respiratory_rate_alarm_threshold: Some(
                        high_respiratory_rate_alarm_threshold,
                    ),
                    target_tidal_volume: Some(target_tidal_volume),
                    low_tidal_volume_alarm_threshold: Some(low_tidal_volume_alarm_threshold),
                    high_tidal_volume_alarm_threshold: Some(high_tidal_volume_alarm_threshold),
                    plateau_duration: Some(plateau_duration),
                    leak_alarm_threshold: Some(leak_alarm_threshold),
                    target_inspiratory_flow: Some(target_inspiratory_flow),
                    inspiratory_duration_command: Some(inspiratory_duration_command),
                    battery_level: Some(battery_level),
                    current_alarm_codes: Some(current_alarm_codes),
                    patient_height: Some(patient_height),
                    locale: Locale::try_from_u16(locale),
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
            >> low_respiratory_rate_alarm_threshold: be_u8
            >> sep
            >> high_respiratory_rate_alarm_threshold: be_u8
            >> sep
            >> target_tidal_volume: be_u16
            >> sep
            >> low_tidal_volume_alarm_threshold: be_u16
            >> sep
            >> high_tidal_volume_alarm_threshold: be_u16
            >> sep
            >> plateau_duration: be_u16
            >> sep
            >> leak_alarm_threshold: be_u16
            >> sep
            >> target_inspiratory_flow: be_u8
            >> sep
            >> inspiratory_duration_command: be_u16
            >> sep
            >> previous_inspiratory_duration: be_u16
            >> sep
            >> battery_level: be_u16
            >> sep
            >> patient_height: be_u8
            >> sep
            >> locale: be_u16
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
                low_respiratory_rate_alarm_threshold: Some(low_respiratory_rate_alarm_threshold),
                high_respiratory_rate_alarm_threshold: Some(high_respiratory_rate_alarm_threshold),
                target_tidal_volume: Some(target_tidal_volume),
                low_tidal_volume_alarm_threshold: Some(low_tidal_volume_alarm_threshold),
                high_tidal_volume_alarm_threshold: Some(high_tidal_volume_alarm_threshold),
                plateau_duration: Some(plateau_duration),
                leak_alarm_threshold: Some(leak_alarm_threshold),
                target_inspiratory_flow: Some(target_inspiratory_flow),
                inspiratory_duration_command: Some(inspiratory_duration_command),
                previous_inspiratory_duration: Some(previous_inspiratory_duration),
                battery_level: Some(battery_level),
                patient_height: Some(patient_height),
                locale: Locale::try_from_u16(locale),
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

named!(
    fatal_error<TelemetryMessage>,
    do_parse!(
        tag!("E:")
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
            >> error: fatal_error_details
            >> end
            >> (TelemetryMessage::FatalError(FatalError {
                telemetry_version: VERSION,
                version: software_version.to_string(),
                device_id: format!("{}-{}-{}", device_id1, device_id2, device_id3),
                systick,
                error,
            }))
    )
);

named!(
    eol_test_snapshot<TelemetryMessage>,
    do_parse!(
        tag!("L:")
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
            >> current_step: eol_test_step
            >> sep
            >> content: eol_test_snapshot_content
            >> end
            >> (TelemetryMessage::EolTestSnapshot(EolTestSnapshot {
                telemetry_version: VERSION,
                version: software_version.to_string(),
                device_id: format!("{}-{}-{}", device_id1, device_id2, device_id3),
                systick,
                current_step,
                content,
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
        fatal_error,
        eol_test_snapshot,
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
    use proptest::num;
    use proptest::option;
    use proptest::prelude::*;

    fn phase_strategy() -> impl Strategy<Value = Phase> {
        prop_oneof![Just(Phase::Inhalation), Just(Phase::Exhalation)]
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

    fn ventilation_mode_strategy() -> impl Strategy<Value = VentilationMode> {
        prop_oneof![
            Just(VentilationMode::PC_CMV),
            Just(VentilationMode::PC_AC),
            Just(VentilationMode::VC_CMV),
            Just(VentilationMode::PC_VSAI),
            Just(VentilationMode::VC_AC),
        ]
    }

    fn fatal_error_details_strategy() -> BoxedStrategy<FatalErrorDetails> {
        prop_oneof![
            Just(FatalErrorDetails::WatchdogRestart),
            fatal_error_details_calibration_error_strategy(),
            fatal_error_details_battery_deeply_discharged_strategy(),
            Just(FatalErrorDetails::MassFlowMeterError),
            fatal_error_details_inconsistent_pressure_strategy(),
        ]
        .boxed()
    }

    prop_compose! {
        fn fatal_error_details_calibration_error_strategy()(
            pressure_offset in num::i16::ANY,
            min_pressure in num::i16::ANY,
            max_pressure in num::i16::ANY,
            flow_at_starting in option::of(num::i16::ANY),
            flow_with_blower_on in option::of(num::i16::ANY),
        ) -> FatalErrorDetails {
            FatalErrorDetails::CalibrationError { pressure_offset, min_pressure, max_pressure, flow_at_starting, flow_with_blower_on }
        }
    }

    prop_compose! {
        fn fatal_error_details_battery_deeply_discharged_strategy()(battery_level in num::u16::ANY) -> FatalErrorDetails {
            FatalErrorDetails::BatteryDeeplyDischarged { battery_level }
        }
    }

    prop_compose! {
        fn fatal_error_details_inconsistent_pressure_strategy()(pressure in num::u16::ANY) -> FatalErrorDetails {
            FatalErrorDetails::InconsistentPressure { pressure }
        }
    }

    fn eol_test_step_strategy() -> impl Strategy<Value = EolTestStep> {
        proptest::num::u8::ANY
            .prop_filter_map("Invalid test step", |n| EolTestStep::try_from(n).ok())
    }

    fn eol_test_snapshot_content_strategy() -> BoxedStrategy<EolTestSnapshotContent> {
        prop_oneof![
            Just(EolTestSnapshotContent::InProgress),
            eol_test_snapshot_content_error_strategy(),
            Just(EolTestSnapshotContent::Success),
        ]
        .boxed()
    }

    prop_compose! {
        fn eol_test_snapshot_content_error_strategy()(reason in ".+") -> EolTestSnapshotContent {
            EolTestSnapshotContent::Error(reason)
        }
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
            let input = &msg.to_bytes_v2();
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
            low_respiratory_rate_alarm_threshold in num::u8::ANY,
            high_respiratory_rate_alarm_threshold in num::u8::ANY,
            target_tidal_volume in num::u16::ANY,
            low_tidal_volume_alarm_threshold in num::u16::ANY,
            high_tidal_volume_alarm_threshold in num::u16::ANY,
            plateau_duration in num::u16::ANY,
            leak_alarm_threshold in num::u16::ANY,
            target_inspiratory_flow in num::u8::ANY,
            inspiratory_duration_command in num::u16::ANY,
            battery_level in num::u16::ANY,
            current_alarm_codes in collection::vec(0u8.., 0..100),
            patient_height in num::u8::ANY,
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
                low_respiratory_rate_alarm_threshold: Some(low_respiratory_rate_alarm_threshold),
                high_respiratory_rate_alarm_threshold: Some(high_respiratory_rate_alarm_threshold),
                target_tidal_volume: Some(target_tidal_volume),
                low_tidal_volume_alarm_threshold: Some(low_tidal_volume_alarm_threshold),
                high_tidal_volume_alarm_threshold: Some(high_tidal_volume_alarm_threshold),
                plateau_duration: Some(plateau_duration),
                leak_alarm_threshold: Some(leak_alarm_threshold),
                target_inspiratory_flow: Some(target_inspiratory_flow),
                inspiratory_duration_command: Some(inspiratory_duration_command),
                battery_level: Some(battery_level),
                current_alarm_codes: Some(current_alarm_codes),
                patient_height: Some(patient_height),
                locale: Some(Locale::default()),
            };
            let input = &msg.to_bytes_v2();
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
                phase,
                subphase: None,
                blower_valve_position,
                patient_valve_position,
                blower_rpm,
                battery_level,
                inspiratory_flow: Some(inspiratory_flow),
                expiratory_flow: Some(expiratory_flow),
            };
            let input = &msg.to_bytes_v2();
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
            low_respiratory_rate_alarm_threshold in num::u8::ANY,
            high_respiratory_rate_alarm_threshold in num::u8::ANY,
            target_tidal_volume in num::u16::ANY,
            low_tidal_volume_alarm_threshold in num::u16::ANY,
            high_tidal_volume_alarm_threshold in num::u16::ANY,
            plateau_duration in num::u16::ANY,
            leak_alarm_threshold in num::u16::ANY,
            target_inspiratory_flow in num::u8::ANY,
            inspiratory_duration_command in num::u16::ANY,
            previous_inspiratory_duration in num::u16::ANY,
            battery_level in num::u16::ANY,
            patient_height in num::u8::ANY,
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
                low_respiratory_rate_alarm_threshold: Some(low_respiratory_rate_alarm_threshold),
                high_respiratory_rate_alarm_threshold: Some(high_respiratory_rate_alarm_threshold),
                target_tidal_volume: Some(target_tidal_volume),
                low_tidal_volume_alarm_threshold: Some(low_tidal_volume_alarm_threshold),
                high_tidal_volume_alarm_threshold: Some(high_tidal_volume_alarm_threshold),
                plateau_duration: Some(plateau_duration),
                leak_alarm_threshold: Some(leak_alarm_threshold),
                target_inspiratory_flow: Some(target_inspiratory_flow),
                inspiratory_duration_command: Some(inspiratory_duration_command),
                previous_inspiratory_duration: Some(previous_inspiratory_duration),
                battery_level: Some(battery_level),
                patient_height: Some(patient_height),
                locale: Some(Locale::default()),
            };
            let input = &msg.to_bytes_v2();
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
                phase,
                subphase: None,
                cycle,
                alarm_code,
                alarm_priority,
                triggered,
                expected,
                measured,
                cycles_since_trigger,
            };
            let input = &msg.to_bytes_v2();
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
            let input = &msg.to_bytes_v2();
            let expected = TelemetryMessage::ControlAck(msg);

            assert_eq!(nom::dbg_dmp(control_ack, "control_ack")(input), Ok((&[][..], expected)));
        }
    }

    proptest! {
        #[test]
        fn test_fatal_error_message_parser(
            version in ".*",
            device_id1 in (0u32..),
            device_id2 in (0u32..),
            device_id3 in (0u32..),
            systick in (0u64..),
            error in fatal_error_details_strategy(),
        ) {
            let msg = FatalError {
                telemetry_version: VERSION,
                version,
                device_id: format!("{}-{}-{}", device_id1, device_id2, device_id3),
                systick,
                error,
            };
            let input = &msg.to_bytes_v2();
            let expected = TelemetryMessage::FatalError(msg);

            assert_eq!(nom::dbg_dmp(fatal_error, "fatal_error")(input), Ok((&[][..], expected)));
        }
    }

    proptest! {
        #[test]
        fn test_eol_test_snapshot_message_parser(
            version in ".*",
            device_id1 in (0u32..),
            device_id2 in (0u32..),
            device_id3 in (0u32..),
            systick in (0u64..),
            current_step in eol_test_step_strategy(),
            content in eol_test_snapshot_content_strategy(),
        ) {
            let msg = EolTestSnapshot {
                telemetry_version: VERSION,
                version,
                device_id: format!("{}-{}-{}", device_id1, device_id2, device_id3),
                systick,
                current_step,
                content,
            };
            let input = &msg.to_bytes_v2();
            let expected = TelemetryMessage::EolTestSnapshot(msg);

            assert_eq!(nom::dbg_dmp(eol_test_snapshot, "eol_test_snapshot")(input), Ok((&[][..], expected)));
        }
    }
}
