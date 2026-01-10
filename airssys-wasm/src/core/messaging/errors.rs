//! Messaging error types.
//!
//! This module contains error types for inter-component messaging operations.
//! These errors are co-located with the messaging module per ADR-WASM-028.

// Layer 1: Standard library imports (per PROJECTS_STANDARD.md §2.1)
// (none needed for this module)

// Layer 2: Third-party crate imports (per PROJECTS_STANDARD.md §2.1)
use thiserror::Error;

// Layer 3: Internal module imports (per PROJECTS_STANDARD.md §2.1)
// (none - errors have no internal dependencies)

/// Messaging errors for inter-component communication.
///
/// `MessagingError` represents errors that can occur during message routing
/// and correlation tracking operations.
///
/// # Variants
///
/// - `DeliveryFailed` - Message could not be delivered to target
/// - `CorrelationTimeout` - Request-response pair timed out
/// - `InvalidMessage` - Message format or content is invalid
/// - `QueueFull` - Message queue is at capacity
/// - `TargetNotFound` - Target component does not exist
///
/// # Examples
///
/// ```rust
/// use airssys_wasm::core::messaging::errors::MessagingError;
///
/// let err = MessagingError::DeliveryFailed("connection lost".to_string());
/// assert!(format!("{}", err).contains("Message delivery failed"));
/// ```
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum MessagingError {
    /// Message delivery failed.
    #[error("Message delivery failed: {0}")]
    DeliveryFailed(String),

    /// Correlation timeout - response not received in time.
    #[error("Correlation timeout: {0}")]
    CorrelationTimeout(String),

    /// Invalid message format or content.
    #[error("Invalid message: {0}")]
    InvalidMessage(String),

    /// Message queue is full.
    #[error("Message queue is full")]
    QueueFull,

    /// Target component not found.
    #[error("Target component not found: {0}")]
    TargetNotFound(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delivery_failed_display() {
        let err = MessagingError::DeliveryFailed("connection lost".to_string());
        assert_eq!(
            format!("{}", err),
            "Message delivery failed: connection lost"
        );
    }

    #[test]
    fn test_correlation_timeout_display() {
        let err = MessagingError::CorrelationTimeout("corr-123".to_string());
        assert_eq!(format!("{}", err), "Correlation timeout: corr-123");
    }

    #[test]
    fn test_invalid_message_display() {
        let err = MessagingError::InvalidMessage("malformed header".to_string());
        assert_eq!(format!("{}", err), "Invalid message: malformed header");
    }

    #[test]
    fn test_queue_full_display() {
        let err = MessagingError::QueueFull;
        assert_eq!(format!("{}", err), "Message queue is full");
    }

    #[test]
    fn test_target_not_found_display() {
        let err = MessagingError::TargetNotFound("app/service/001".to_string());
        assert_eq!(
            format!("{}", err),
            "Target component not found: app/service/001"
        );
    }

    #[test]
    fn test_error_is_clone() {
        let err = MessagingError::QueueFull;
        let cloned = err.clone();
        assert!(matches!(cloned, MessagingError::QueueFull));
    }

    #[test]
    fn test_error_is_debug() {
        let err = MessagingError::DeliveryFailed("test".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("DeliveryFailed"));
    }

    #[test]
    fn test_error_equality() {
        let err1 = MessagingError::QueueFull;
        let err2 = MessagingError::QueueFull;
        let err3 = MessagingError::DeliveryFailed("test".to_string());

        assert_eq!(err1, err2);
        assert_ne!(err1, err3);
    }

    // Gap analysis tests

    #[test]
    fn test_messaging_error_implements_std_error() {
        let err: Box<dyn std::error::Error> = Box::new(MessagingError::QueueFull);
        assert!(err.to_string().contains("queue"));
    }

    #[test]
    fn test_messaging_error_is_send_sync() {
        fn requires_send<T: Send>(_val: T) {}
        fn requires_sync<T: Sync>(_val: T) {}

        let err = MessagingError::QueueFull;
        requires_send(err.clone());
        requires_sync(err);
    }
}
