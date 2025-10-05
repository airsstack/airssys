//! Comprehensive broker error types with context.
//!
//! This module defines all error types that can occur during message broker operations,
//! including routing failures, timeout errors, and registry management errors.

// Layer 1: Standard library imports
use std::time::Duration;

// Layer 2: Third-party crate imports
use thiserror::Error;

// Layer 3: Internal module imports
use crate::util::ActorAddress;

/// Comprehensive broker error types with contextual information.
///
/// BrokerError represents all possible failure modes in the message broker system,
/// from actor registration failures to message delivery timeouts.
///
/// # Design Principles
///
/// - **Contextual**: Each error variant includes relevant context for debugging
/// - **Structured**: Uses thiserror for automatic Error trait implementation
/// - **Type-Safe**: Strongly typed error variants with no string-only errors
///
/// # Example
///
/// ```rust
/// use airssys_rt::broker::BrokerError;
/// use airssys_rt::util::ActorAddress;
/// use std::time::Duration;
///
/// let error = BrokerError::ActorNotFound(ActorAddress::anonymous());
/// assert!(error.to_string().contains("Actor not found"));
///
/// let timeout_error = BrokerError::SendTimeout {
///     target: ActorAddress::anonymous(),
///     timeout: Duration::from_secs(5),
/// };
/// assert!(timeout_error.to_string().contains("Send timeout"));
/// ```
#[derive(Debug, Error)]
pub enum BrokerError {
    /// Actor not found in registry
    ///
    /// This error occurs when attempting to send a message to an actor address
    /// that is not currently registered in the broker's routing table.
    #[error("Actor not found: {0:?}")]
    ActorNotFound(ActorAddress),

    /// Actor mailbox is closed (actor stopped)
    ///
    /// This error occurs when the target actor has stopped and its mailbox
    /// is no longer accepting messages. The mailbox sender has been dropped.
    #[error("Mailbox closed for actor: {0:?}")]
    MailboxClosed(ActorAddress),

    /// Send operation timed out
    ///
    /// This error occurs when a send operation exceeds the configured timeout
    /// duration, typically due to backpressure or mailbox full conditions.
    #[error("Send timeout: target={target:?}, timeout={timeout:?}")]
    SendTimeout {
        /// The target actor address that was unreachable
        target: ActorAddress,
        /// The timeout duration that was exceeded
        timeout: Duration,
    },

    /// Request-reply operation timed out
    ///
    /// This error occurs when a request-reply operation does not receive a
    /// response within the configured timeout duration.
    #[error("Request timeout: target={target:?}, timeout={timeout:?}")]
    RequestTimeout {
        /// The target actor address that did not respond
        target: ActorAddress,
        /// The timeout duration that was exceeded
        timeout: Duration,
    },

    /// Registry operation failed
    ///
    /// This error occurs during actor registration, unregistration, or
    /// address resolution operations in the actor registry.
    #[error("Registry error: {0}")]
    RegistryError(String),

    /// Message routing failed
    ///
    /// This error occurs when the broker cannot route a message due to
    /// configuration issues, invalid addresses, or routing logic failures.
    #[error("Route error: message_type={message_type}, reason={reason}")]
    RouteError {
        /// The type of message that failed to route
        message_type: &'static str,
        /// The reason for the routing failure
        reason: String,
    },

    /// Broker not initialized in actor context
    ///
    /// This error occurs when attempting to use broker functionality from
    /// an ActorContext that has not been properly initialized with a broker.
    #[error("Broker not initialized in actor context")]
    BrokerNotInitialized,

    /// No response received for request
    ///
    /// This error occurs when a request-reply operation completes without
    /// a response, distinct from timeout (the operation completed but no
    /// response was provided).
    #[error("No response received for request")]
    NoResponse,

    /// Actor pool not found
    ///
    /// This error occurs when attempting to route a message to a named actor
    /// pool that does not exist in the registry.
    #[error("Actor pool not found: {0}")]
    PoolNotFound(String),

    /// Actor pool is empty
    ///
    /// This error occurs when attempting to route a message to an actor pool
    /// that exists but has no registered actors.
    #[error("Actor pool is empty: {0}")]
    PoolEmpty(String),

    /// Duplicate actor registration
    ///
    /// This error occurs when attempting to register an actor with an address
    /// that is already registered in the routing table.
    #[error("Duplicate actor registration: {0:?}")]
    DuplicateRegistration(ActorAddress),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::ActorId;
    use std::error::Error as StdError;

    #[test]
    fn test_actor_not_found_error() {
        let address = ActorAddress::anonymous();
        let error = BrokerError::ActorNotFound(address.clone());

        let error_string = error.to_string();
        assert!(error_string.contains("Actor not found"));
        assert!(error_string.contains(&format!("{address:?}")));
    }

    #[test]
    fn test_mailbox_closed_error() {
        let address = ActorAddress::Named {
            id: ActorId::new(),
            name: "test-actor".to_string(),
        };
        let error = BrokerError::MailboxClosed(address.clone());

        let error_string = error.to_string();
        assert!(error_string.contains("Mailbox closed"));
        assert!(error_string.contains("test-actor"));
    }

    #[test]
    fn test_send_timeout_error() {
        let address = ActorAddress::anonymous();
        let timeout = Duration::from_secs(5);
        let error = BrokerError::SendTimeout {
            target: address.clone(),
            timeout,
        };

        let error_string = error.to_string();
        assert!(error_string.contains("Send timeout"));
        assert!(error_string.contains("5s"));
    }

    #[test]
    fn test_request_timeout_error() {
        let address = ActorAddress::anonymous();
        let timeout = Duration::from_millis(100);
        let error = BrokerError::RequestTimeout {
            target: address.clone(),
            timeout,
        };

        let error_string = error.to_string();
        assert!(error_string.contains("Request timeout"));
        assert!(error_string.contains("100ms"));
    }

    #[test]
    fn test_registry_error() {
        let error = BrokerError::RegistryError("Invalid operation".to_string());

        let error_string = error.to_string();
        assert!(error_string.contains("Registry error"));
        assert!(error_string.contains("Invalid operation"));
    }

    #[test]
    fn test_route_error() {
        let error = BrokerError::RouteError {
            message_type: "test_message",
            reason: "No route available".to_string(),
        };

        let error_string = error.to_string();
        assert!(error_string.contains("Route error"));
        assert!(error_string.contains("test_message"));
        assert!(error_string.contains("No route available"));
    }

    #[test]
    fn test_broker_not_initialized_error() {
        let error = BrokerError::BrokerNotInitialized;

        let error_string = error.to_string();
        assert!(error_string.contains("Broker not initialized"));
    }

    #[test]
    fn test_no_response_error() {
        let error = BrokerError::NoResponse;

        let error_string = error.to_string();
        assert!(error_string.contains("No response"));
    }

    #[test]
    fn test_pool_not_found_error() {
        let error = BrokerError::PoolNotFound("worker-pool".to_string());

        let error_string = error.to_string();
        assert!(error_string.contains("Actor pool not found"));
        assert!(error_string.contains("worker-pool"));
    }

    #[test]
    fn test_pool_empty_error() {
        let error = BrokerError::PoolEmpty("worker-pool".to_string());

        let error_string = error.to_string();
        assert!(error_string.contains("Actor pool is empty"));
        assert!(error_string.contains("worker-pool"));
    }

    #[test]
    fn test_duplicate_registration_error() {
        let address = ActorAddress::Named {
            id: ActorId::new(),
            name: "duplicate".to_string(),
        };
        let error = BrokerError::DuplicateRegistration(address.clone());

        let error_string = error.to_string();
        assert!(error_string.contains("Duplicate actor registration"));
        assert!(error_string.contains("duplicate"));
    }

    #[test]
    fn test_error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<BrokerError>();
    }

    #[test]
    fn test_error_is_std_error() {
        let error = BrokerError::BrokerNotInitialized;
        let _: &dyn StdError = &error;
    }

    #[test]
    fn test_error_debug_impl() {
        let error = BrokerError::ActorNotFound(ActorAddress::anonymous());
        let debug_string = format!("{error:?}");
        assert!(debug_string.contains("ActorNotFound"));
    }
}
