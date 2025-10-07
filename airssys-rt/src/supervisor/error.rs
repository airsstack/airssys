//! Error types for supervisor operations.
//!
//! This module defines all error types used by the supervisor framework,
//! following Microsoft Rust Guidelines for canonical error structures
//! (M-ERRORS-CANONICAL-STRUCTS).

// Layer 1: Standard library imports
use std::error::Error;
use std::time::Duration;

// Layer 2: Third-party crate imports
use thiserror::Error;

// Layer 3: Internal module imports
use super::types::ChildId;

/// Errors that can occur during supervisor operations.
///
/// All variants include contextual information to aid in debugging and
/// error handling. Error types follow workspace standards (§6.3) and
/// Microsoft Rust Guidelines (M-ERRORS-CANONICAL-STRUCTS).
///
/// # Examples
///
/// ```rust
/// use airssys_rt::supervisor::SupervisorError;
///
/// fn check_error_type(err: &SupervisorError) {
///     if err.is_fatal() {
///         println!("Fatal error - supervisor should escalate");
///     } else if err.is_retryable() {
///         println!("Retryable error - supervisor may retry");
///     }
/// }
/// ```
#[derive(Debug, Error)]
pub enum SupervisorError {
    /// Child with the specified ID was not found.
    #[error("Child not found: {id}")]
    ChildNotFound { id: ChildId },

    /// Failed to start a child process.
    #[error("Failed to start child '{id}': {source}")]
    ChildStartFailed {
        id: String,
        #[source]
        source: Box<dyn Error + Send + Sync>,
    },

    /// Failed to stop a child process.
    #[error("Failed to stop child '{id}': {source}")]
    ChildStopFailed {
        id: String,
        #[source]
        source: Box<dyn Error + Send + Sync>,
    },

    /// Child restart limit has been exceeded.
    ///
    /// This occurs when a child fails and restarts too many times within
    /// the configured time window, indicating a persistent problem that
    /// cannot be resolved through simple restarts.
    #[error("Restart limit exceeded for child '{id}': {max_restarts} restarts in {window:?}")]
    RestartLimitExceeded {
        id: String,
        max_restarts: u32,
        window: Duration,
    },

    /// Invalid supervisor configuration.
    #[error("Invalid supervisor configuration: {reason}")]
    InvalidConfiguration { reason: String },

    /// Monitoring system error.
    #[error("Monitoring error: {source}")]
    MonitoringError {
        #[source]
        source: Box<dyn Error + Send + Sync>,
    },

    /// Child factory function failed to create a new child instance.
    #[error("Child factory failed for '{id}': {source}")]
    ChildFactoryFailed {
        id: String,
        #[source]
        source: Box<dyn Error + Send + Sync>,
    },

    /// Child shutdown timeout exceeded.
    #[error("Child '{id}' shutdown timeout after {timeout:?}")]
    ShutdownTimeout { id: String, timeout: Duration },

    /// Supervisor tree integrity violation.
    ///
    /// This indicates an internal consistency error in the supervisor tree
    /// structure, such as circular dependencies or orphaned children.
    #[error("Supervisor tree integrity violation: {reason}")]
    TreeIntegrityViolation { reason: String },

    /// Health monitoring is not enabled for this supervisor.
    ///
    /// Attempted to perform a health check operation when health monitoring
    /// has not been enabled via `enable_health_checks()`.
    #[error("Health monitoring not enabled for child '{id}'")]
    HealthMonitoringNotEnabled { id: String },
}

impl SupervisorError {
    /// Returns `true` if this error is fatal and should cause supervisor escalation.
    ///
    /// Fatal errors indicate problems that cannot be resolved at the current
    /// supervisor level and should be escalated to the parent supervisor.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::supervisor::SupervisorError;
    /// use std::time::Duration;
    ///
    /// let err = SupervisorError::RestartLimitExceeded {
    ///     id: "worker".into(),
    ///     max_restarts: 5,
    ///     window: Duration::from_secs(60),
    /// };
    ///
    /// assert!(err.is_fatal());
    /// ```
    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            SupervisorError::RestartLimitExceeded { .. }
                | SupervisorError::TreeIntegrityViolation { .. }
                | SupervisorError::InvalidConfiguration { .. }
        )
    }

    /// Returns `true` if this error is retryable.
    ///
    /// Retryable errors indicate transient failures that might succeed
    /// on subsequent attempts.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::supervisor::SupervisorError;
    /// use std::time::Duration;
    ///
    /// let err = SupervisorError::ShutdownTimeout {
    ///     id: "worker".into(),
    ///     timeout: Duration::from_secs(5),
    /// };
    ///
    /// assert!(err.is_retryable());
    /// ```
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            SupervisorError::ChildStartFailed { .. }
                | SupervisorError::ChildStopFailed { .. }
                | SupervisorError::ShutdownTimeout { .. }
                | SupervisorError::MonitoringError { .. }
        )
    }

    /// Returns `true` if this error indicates a missing child.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::supervisor::{SupervisorError, ChildId};
    ///
    /// let child_id = ChildId::new();
    /// let err = SupervisorError::ChildNotFound { id: child_id.clone() };
    ///
    /// assert!(err.is_not_found());
    /// ```
    pub fn is_not_found(&self) -> bool {
        matches!(self, SupervisorError::ChildNotFound { .. })
    }

    /// Returns the child ID associated with this error, if any.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::supervisor::{SupervisorError, ChildId};
    ///
    /// let child_id = ChildId::new();
    /// let err = SupervisorError::ChildNotFound { id: child_id.clone() };
    ///
    /// assert_eq!(err.child_id(), Some(&child_id));
    /// ```
    pub fn child_id(&self) -> Option<&ChildId> {
        match self {
            SupervisorError::ChildNotFound { id } => Some(id),
            _ => None,
        }
    }

    /// Returns the child name associated with this error, if any.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::supervisor::SupervisorError;
    /// use std::time::Duration;
    ///
    /// let err = SupervisorError::RestartLimitExceeded {
    ///     id: "worker-1".into(),
    ///     max_restarts: 5,
    ///     window: Duration::from_secs(60),
    /// };
    ///
    /// assert_eq!(err.child_name(), Some("worker-1"));
    /// ```
    pub fn child_name(&self) -> Option<&str> {
        match self {
            SupervisorError::ChildStartFailed { id, .. }
            | SupervisorError::ChildStopFailed { id, .. }
            | SupervisorError::RestartLimitExceeded { id, .. }
            | SupervisorError::ChildFactoryFailed { id, .. }
            | SupervisorError::ShutdownTimeout { id, .. } => Some(id),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_child_not_found_error() {
        let child_id = ChildId::new();
        let err = SupervisorError::ChildNotFound {
            id: child_id.clone(),
        };

        assert!(err.is_not_found());
        assert!(!err.is_fatal());
        assert!(!err.is_retryable());
        assert_eq!(err.child_id(), Some(&child_id));
        assert!(err.to_string().contains(&child_id.to_string()));
    }

    #[test]
    fn test_child_start_failed_error() {
        let err = SupervisorError::ChildStartFailed {
            id: "worker-1".into(),
            source: Box::new(io::Error::new(io::ErrorKind::Other, "test error")),
        };

        assert!(!err.is_fatal());
        assert!(err.is_retryable());
        assert!(!err.is_not_found());
        assert_eq!(err.child_name(), Some("worker-1"));
        assert!(err.to_string().contains("worker-1"));
        assert!(err.source().is_some());
    }

    #[test]
    fn test_restart_limit_exceeded_error() {
        let err = SupervisorError::RestartLimitExceeded {
            id: "failing-worker".into(),
            max_restarts: 5,
            window: Duration::from_secs(60),
        };

        assert!(err.is_fatal());
        assert!(!err.is_retryable());
        assert_eq!(err.child_name(), Some("failing-worker"));
        let msg = err.to_string();
        assert!(msg.contains("failing-worker"));
        assert!(msg.contains("5"));
        assert!(msg.contains("60"));
    }

    #[test]
    fn test_invalid_configuration_error() {
        let err = SupervisorError::InvalidConfiguration {
            reason: "max_restarts cannot be zero".into(),
        };

        assert!(err.is_fatal());
        assert!(!err.is_retryable());
        assert_eq!(err.child_name(), None);
        assert!(err.to_string().contains("max_restarts cannot be zero"));
    }

    #[test]
    fn test_shutdown_timeout_error() {
        let err = SupervisorError::ShutdownTimeout {
            id: "slow-worker".into(),
            timeout: Duration::from_secs(10),
        };

        assert!(!err.is_fatal());
        assert!(err.is_retryable());
        assert_eq!(err.child_name(), Some("slow-worker"));
        assert!(err.to_string().contains("slow-worker"));
        assert!(err.to_string().contains("10"));
    }

    #[test]
    fn test_tree_integrity_violation_error() {
        let err = SupervisorError::TreeIntegrityViolation {
            reason: "circular dependency detected".into(),
        };

        assert!(err.is_fatal());
        assert!(!err.is_retryable());
        assert!(err.to_string().contains("circular dependency detected"));
    }
}
