//! System-level error types.

// Layer 1: Standard library
use std::time::Duration;

// Layer 2: Third-party
use thiserror::Error;

// Layer 3: Internal
use crate::broker::BrokerError;
use crate::util::ActorId;

/// System-level errors for actor runtime operations.
///
/// Follows §6.3 M-ERRORS-CANONICAL-STRUCTS pattern with
/// structured error types and helper methods.
#[derive(Error, Debug)]
pub enum SystemError {
    /// Actor with given ID not found in registry
    #[error("Actor not found: {0}")]
    ActorNotFound(ActorId),

    /// Failed to spawn actor
    #[error("Failed to spawn actor: {0}")]
    SpawnFailed(String),

    /// System is shutting down, cannot accept new operations
    #[error("System shutdown in progress")]
    ShuttingDown,

    /// Actor mailbox is full (bounded mailbox with backpressure)
    #[error("Actor mailbox full: {0}")]
    MailboxFull(ActorId),

    /// Message broker error
    #[error("Broker error: {0}")]
    BrokerError(#[from] BrokerError),

    /// Configuration validation error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Actor limit exceeded
    #[error("Actor limit exceeded: current {current}, max {max}")]
    ActorLimitExceeded { current: usize, max: usize },

    /// Shutdown timeout exceeded
    #[error("Shutdown timeout exceeded after {0:?}")]
    ShutdownTimeout(Duration),
}

impl SystemError {
    /// Check if error is transient (can retry).
    ///
    /// Transient errors are temporary conditions that may resolve
    /// with retry logic (e.g., mailbox full).
    pub fn is_transient(&self) -> bool {
        matches!(self, SystemError::MailboxFull(_))
    }

    /// Check if error is fatal (system must stop).
    ///
    /// Fatal errors indicate the system cannot continue operating
    /// and must shut down.
    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            SystemError::ShuttingDown | SystemError::ShutdownTimeout(_)
        )
    }

    /// Check if error is recoverable.
    ///
    /// Recoverable errors can be handled without stopping the system.
    pub fn is_recoverable(&self) -> bool {
        !self.is_fatal()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::ActorAddress;

    #[test]
    fn test_actor_not_found_display() {
        let id = ActorId::new();
        let err = SystemError::ActorNotFound(id);
        let msg = err.to_string();
        assert!(msg.contains("Actor not found"));
        assert!(msg.contains(&id.to_string()));
    }

    #[test]
    fn test_spawn_failed_display() {
        let err = SystemError::SpawnFailed("initialization error".to_string());
        assert!(err.to_string().contains("Failed to spawn"));
        assert!(err.to_string().contains("initialization error"));
    }

    #[test]
    fn test_shutting_down_display() {
        let err = SystemError::ShuttingDown;
        assert_eq!(err.to_string(), "System shutdown in progress");
    }

    #[test]
    fn test_mailbox_full_display() {
        let id = ActorId::new();
        let err = SystemError::MailboxFull(id);
        assert!(err.to_string().contains("mailbox full"));
    }

    #[test]
    fn test_config_error_display() {
        let err = SystemError::ConfigError("invalid timeout".to_string());
        assert!(err.to_string().contains("Configuration error"));
        assert!(err.to_string().contains("invalid timeout"));
    }

    #[test]
    fn test_actor_limit_exceeded_display() {
        let err = SystemError::ActorLimitExceeded {
            current: 100,
            max: 50,
        };
        let msg = err.to_string();
        assert!(msg.contains("100"));
        assert!(msg.contains("50"));
        assert!(msg.contains("exceeded"));
    }

    #[test]
    fn test_shutdown_timeout_display() {
        let timeout = Duration::from_secs(30);
        let err = SystemError::ShutdownTimeout(timeout);
        assert!(err.to_string().contains("timeout"));
        assert!(err.to_string().contains("30"));
    }

    #[test]
    fn test_transient_errors() {
        let mailbox_err = SystemError::MailboxFull(ActorId::new());
        assert!(mailbox_err.is_transient());
        assert!(!mailbox_err.is_fatal());
        assert!(mailbox_err.is_recoverable());

        let spawn_err = SystemError::SpawnFailed("error".to_string());
        assert!(!spawn_err.is_transient());
    }

    #[test]
    fn test_fatal_errors() {
        let shutdown_err = SystemError::ShuttingDown;
        assert!(!shutdown_err.is_transient());
        assert!(shutdown_err.is_fatal());
        assert!(!shutdown_err.is_recoverable());

        let timeout_err = SystemError::ShutdownTimeout(Duration::from_secs(30));
        assert!(timeout_err.is_fatal());
        assert!(!timeout_err.is_recoverable());
    }

    #[test]
    fn test_recoverable_errors() {
        let not_found = SystemError::ActorNotFound(ActorId::new());
        assert!(not_found.is_recoverable());

        let config_err = SystemError::ConfigError("test".to_string());
        assert!(config_err.is_recoverable());
    }

    #[test]
    fn test_broker_error_conversion() {
        let broker_err = BrokerError::ActorNotFound(ActorAddress::anonymous());
        let system_err: SystemError = broker_err.into();
        assert!(matches!(system_err, SystemError::BrokerError(_)));
    }

    #[test]
    fn test_error_debug_impl() {
        let err = SystemError::SpawnFailed("test".to_string());
        let debug = format!("{:?}", err);
        assert!(debug.contains("SpawnFailed"));
    }
}
