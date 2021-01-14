use thiserror::Error;

use crate::structures::HighLevelError;

/// An error that can be provided to consumers
#[derive(Debug, Error)]
pub enum Error {
    /// Telemetry error
    #[error("Telemetry error: {0}")]
    TelemetryError(#[from] HighLevelError),

    #[cfg(feature = "serial")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "serial")))]
    /// Serial error
    #[error("Serial error: {0}")]
    SerialError(#[from] serial::core::Error),
}
