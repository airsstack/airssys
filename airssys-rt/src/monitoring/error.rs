//! Monitoring error types following M-ERRORS-CANONICAL-STRUCTS pattern.

use thiserror::Error;

/// Errors that can occur during monitoring operations.
///
/// All variants include context following Microsoft Rust Guidelines
/// (M-ERRORS-CANONICAL-STRUCTS).
#[derive(Debug, Error)]
pub enum MonitoringError {
    /// Configuration error during monitor initialization
    #[error("Configuration error: {message}")]
    Configuration {
        /// Error message describing the configuration issue
        message: String,
    },

    /// Error recording an event
    #[error("Failed to record event: {message}")]
    RecordError {
        /// Error message describing the recording failure
        message: String,
    },

    /// Error generating a snapshot
    #[error("Failed to generate snapshot: {message}")]
    SnapshotError {
        /// Error message describing the snapshot failure
        message: String,
    },

    /// Error resetting monitor state
    #[error("Failed to reset monitor: {message}")]
    ResetError {
        /// Error message describing the reset failure
        message: String,
    },
}

impl MonitoringError {
    /// Creates a new configuration error.
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration {
            message: message.into(),
        }
    }

    /// Creates a new record error.
    pub fn record(message: impl Into<String>) -> Self {
        Self::RecordError {
            message: message.into(),
        }
    }

    /// Creates a new snapshot error.
    pub fn snapshot(message: impl Into<String>) -> Self {
        Self::SnapshotError {
            message: message.into(),
        }
    }

    /// Creates a new reset error.
    pub fn reset(message: impl Into<String>) -> Self {
        Self::ResetError {
            message: message.into(),
        }
    }

    /// Returns true if this is a configuration error.
    pub fn is_configuration(&self) -> bool {
        matches!(self, Self::Configuration { .. })
    }

    /// Returns true if this is a record error.
    pub fn is_record(&self) -> bool {
        matches!(self, Self::RecordError { .. })
    }

    /// Returns true if this is a snapshot error.
    pub fn is_snapshot(&self) -> bool {
        matches!(self, Self::SnapshotError { .. })
    }

    /// Returns true if this is a reset error.
    pub fn is_reset(&self) -> bool {
        matches!(self, Self::ResetError { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_configuration_error_creation() {
        let error = MonitoringError::configuration("Invalid max_history_size");
        assert!(error.is_configuration());
        assert!(!error.is_record());
        assert!(error.to_string().contains("Invalid max_history_size"));
    }

    #[test]
    fn test_record_error_creation() {
        let error = MonitoringError::record("Failed to acquire lock");
        assert!(error.is_record());
        assert!(!error.is_configuration());
        assert!(error.to_string().contains("Failed to acquire lock"));
    }

    #[test]
    fn test_snapshot_error_creation() {
        let error = MonitoringError::snapshot("Buffer overflow");
        assert!(error.is_snapshot());
        assert!(!error.is_reset());
        assert!(error.to_string().contains("Buffer overflow"));
    }

    #[test]
    fn test_reset_error_creation() {
        let error = MonitoringError::reset("Cannot reset while recording");
        assert!(error.is_reset());
        assert!(!error.is_snapshot());
        assert!(error.to_string().contains("Cannot reset while recording"));
    }

    #[test]
    fn test_error_helper_methods() {
        let config_err = MonitoringError::configuration("test");
        let record_err = MonitoringError::record("test");
        let snapshot_err = MonitoringError::snapshot("test");
        let reset_err = MonitoringError::reset("test");

        assert!(config_err.is_configuration());
        assert!(record_err.is_record());
        assert!(snapshot_err.is_snapshot());
        assert!(reset_err.is_reset());
    }
}
