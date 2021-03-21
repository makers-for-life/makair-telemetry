initSidebarItems({"enum":[["AlarmPriority","Supported alarm priorities"],["EolTestSnapshotContent","Content of end of line test snapshots"],["EolTestStep","Step of the end of line test"],["FatalErrorDetails","Details of fatal errors"],["HighLevelError","Errors that need to be reported to the UI"],["Mode","Variants of the MakAir firmware"],["PatientGender","Patient gender"],["Phase","Phases of the respiratory cycle"],["SubPhase","[obsolete in protocol v2] Sub-phases of the respiratory cycle"],["TelemetryErrorKind","Extension of Nom’s `ErrorKind` to be able to represent CRC errors"],["TelemetryMessage","Supported telemetry messages"],["VentilationMode","Supported ventilation modes"],["VentilationModeClass","Ventilation mode class"],["VentilationModeKind","Ventilation mode kind"]],"struct":[["AlarmTrap","A telemetry message that is sent every time an alarm is triggered or stopped"],["BootMessage","A telemetry message that is sent once every time the MCU boots"],["ControlAck","An ACK message that is sent every time a setting is changed on the MCU side"],["DataSnapshot","A telemetry message that is sent every time the firmware does a control iteration (every 10 ms)"],["EolTestSnapshot","[protocol v2] A message sent during end of line tests"],["FatalError","[protocol v2] A message sent when a fatal error occurs"],["MachineStateSnapshot","A telemetry message that is sent at the end of every respiratory cycle"],["StoppedMessage","A telemetry message that is sent every 100 ms when the MCU is in “stop” mode"],["TelemetryError","Custom parser error type to leverage `TelemetryErrorKind`"]]});