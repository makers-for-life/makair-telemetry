// MakAir Telemetry
//
// Copyright: 2020, Makers For Life
// License: Public Domain License

use crate::structures::*;

pub fn compute_duration(messages: Vec<TelemetryMessage>) -> u32 {
    let mut duration: u32 = 0;

    for message in &messages {
        match message {
            TelemetryMessage::DataSnapshot(_) => {
                duration += 10;
            }

            TelemetryMessage::StoppedMessage(_) => {
                duration += 100;
            }
            _ => {}
        }
    }

    duration
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_duration_no_data() {
        assert_eq!(compute_duration(vec![]), 0);
    }

    #[test]
    fn test_compute_duration_one_boot_message() {
        let vect: Vec<TelemetryMessage> = vec![TelemetryMessage::BootMessage(BootMessage {
            telemetry_version: 1,
            version: String::from(""),
            device_id: String::from(""),
            systick: 0,
            mode: Mode::Production,
            value128: 0,
        })];

        assert_eq!(compute_duration(vect), 0);
    }

    #[test]
    fn test_compute_duration_one_alarm_trap() {
        let vect: Vec<TelemetryMessage> = vec![TelemetryMessage::AlarmTrap(AlarmTrap {
            telemetry_version: 1,
            version: String::from(""),
            device_id: String::from(""),
            systick: 0,
            centile: 0,
            pressure: 0,
            phase: Phase::Inhalation,
            subphase: SubPhase::Inspiration,
            cycle: 0,
            alarm_code: 0,
            alarm_priority: AlarmPriority::Low,
            triggered: true,
            expected: 0,
            measured: 0,
            cycles_since_trigger: 0,
        })];

        assert_eq!(compute_duration(vect), 0);
    }

    #[test]
    fn test_compute_duration_one_data_snapshot() {
        let vect: Vec<TelemetryMessage> = vec![TelemetryMessage::DataSnapshot(DataSnapshot {
            telemetry_version: 1,
            version: String::from(""),
            device_id: String::from(""),
            systick: 0,
            centile: 0,
            pressure: 0,
            phase: Phase::Inhalation,
            subphase: SubPhase::Inspiration,
            blower_valve_position: 0,
            patient_valve_position: 0,
            blower_rpm: 0,
            battery_level: 0,
        })];

        assert_eq!(compute_duration(vect), 10);
    }

    #[test]
    fn test_compute_duration_one_machine_state_snapshot() {
        let vect: Vec<TelemetryMessage> = vec![TelemetryMessage::MachineStateSnapshot(
            MachineStateSnapshot {
                telemetry_version: 1,
                version: String::from(""),
                device_id: String::from(""),
                systick: 0,
                cycle: 0,
                peak_command: 0,
                plateau_command: 0,
                peep_command: 0,
                cpm_command: 0,
                previous_peak_pressure: 0,
                previous_plateau_pressure: 0,
                previous_peep_pressure: 0,
                current_alarm_codes: vec![],
                previous_volume: None,
                expiratory_term: 0,
                trigger_enabled: false,
                trigger_offset: 0,
            },
        )];

        assert_eq!(compute_duration(vect), 0);
    }

    #[test]
    fn test_compute_duration_one_stopped_message() {
        let mut vect: Vec<TelemetryMessage> = Vec::new();

        vect.push(TelemetryMessage::StoppedMessage(StoppedMessage {
            telemetry_version: 1,
            version: String::from(""),
            device_id: String::from(""),
            systick: 0,
            peak_command: None,
            plateau_command: None,
            peep_command: None,
            cpm_command: None,
            expiratory_term: None,
            trigger_enabled: None,
            trigger_offset: None,
        }));

        assert_eq!(compute_duration(vect), 100);
    }

    #[test]
    fn test_compute_duration_one_of_each_message() {
        let mut vect: Vec<TelemetryMessage> = Vec::new();

        vect.push(TelemetryMessage::BootMessage(BootMessage {
            telemetry_version: 1,
            version: String::from(""),
            device_id: String::from(""),
            systick: 0,
            mode: Mode::Production,
            value128: 0,
        }));

        vect.push(TelemetryMessage::AlarmTrap(AlarmTrap {
            telemetry_version: 1,
            version: String::from(""),
            device_id: String::from(""),
            systick: 0,
            centile: 0,
            pressure: 0,
            phase: Phase::Inhalation,
            subphase: SubPhase::Inspiration,
            cycle: 0,
            alarm_code: 0,
            alarm_priority: AlarmPriority::Low,
            triggered: true,
            expected: 0,
            measured: 0,
            cycles_since_trigger: 0,
        }));

        vect.push(TelemetryMessage::DataSnapshot(DataSnapshot {
            telemetry_version: 1,
            version: String::from(""),
            device_id: String::from(""),
            systick: 0,
            centile: 0,
            pressure: 0,
            phase: Phase::Inhalation,
            subphase: SubPhase::Inspiration,
            blower_valve_position: 0,
            patient_valve_position: 0,
            blower_rpm: 0,
            battery_level: 0,
        }));

        vect.push(TelemetryMessage::MachineStateSnapshot(
            MachineStateSnapshot {
                telemetry_version: 1,
                version: String::from(""),
                device_id: String::from(""),
                systick: 0,
                cycle: 0,
                peak_command: 0,
                plateau_command: 0,
                peep_command: 0,
                cpm_command: 0,
                previous_peak_pressure: 0,
                previous_plateau_pressure: 0,
                previous_peep_pressure: 0,
                current_alarm_codes: vec![],
                previous_volume: None,
                expiratory_term: 0,
                trigger_enabled: false,
                trigger_offset: 0,
            },
        ));

        vect.push(TelemetryMessage::StoppedMessage(StoppedMessage {
            telemetry_version: 1,
            version: String::from(""),
            device_id: String::from(""),
            systick: 0,
            peak_command: None,
            plateau_command: None,
            peep_command: None,
            cpm_command: None,
            expiratory_term: None,
            trigger_enabled: None,
            trigger_offset: None,
        }));

        assert_eq!(compute_duration(vect), 110);
    }
}
