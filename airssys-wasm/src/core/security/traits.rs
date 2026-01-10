//! Security traits for capability validation and audit logging.
//!
//! This module defines the core security abstractions:
//! - [`SecurityValidator`] - Validates component capabilities
//! - [`SecurityAuditLogger`] - Logs security events for audit
//! - [`SecurityEvent`] - Security event data structure

use super::capability::Capability;
use super::errors::SecurityError;
use crate::core::component::id::ComponentId;

/// Trait for validating component capabilities.
///
/// Implemented by the security module, consumed by runtime and messaging modules.
///
/// # Example
///
/// ```rust
/// use airssys_wasm::core::security::traits::SecurityValidator;
/// use airssys_wasm::core::security::capability::{Capability, MessagingCapability, MessagingAction};
/// use airssys_wasm::core::security::errors::SecurityError;
/// use airssys_wasm::core::component::id::ComponentId;
///
/// struct AllowAllValidator;
///
/// impl SecurityValidator for AllowAllValidator {
///     fn validate_capability(
///         &self,
///         _component: &ComponentId,
///         _capability: &Capability,
///     ) -> Result<(), SecurityError> {
///         Ok(()) // Allow all for demo
///     }
///
///     fn can_send_to(
///         &self,
///         _sender: &ComponentId,
///         _target: &ComponentId,
///     ) -> Result<(), SecurityError> {
///         Ok(()) // Allow all for demo
///     }
/// }
/// ```
pub trait SecurityValidator: Send + Sync {
    /// Validate if component has required capability.
    ///
    /// # Arguments
    /// * `component` - The component requesting the capability
    /// * `capability` - The capability being requested
    ///
    /// # Returns
    /// * `Ok(())` if capability is granted
    /// * `Err(SecurityError)` if capability is denied
    fn validate_capability(
        &self,
        component: &ComponentId,
        capability: &Capability,
    ) -> Result<(), SecurityError>;

    /// Check if component can send message to target.
    ///
    /// # Arguments
    /// * `sender` - The sending component
    /// * `target` - The target component
    ///
    /// # Returns
    /// * `Ok(())` if sending is allowed
    /// * `Err(SecurityError)` if sending is denied
    fn can_send_to(&self, sender: &ComponentId, target: &ComponentId) -> Result<(), SecurityError>;
}

/// Trait for security audit logging.
///
/// Implemented by the audit logging system for security event tracking.
pub trait SecurityAuditLogger: Send + Sync {
    /// Log a security event.
    ///
    /// # Arguments
    /// * `event` - The security event to log
    fn log_event(&self, event: SecurityEvent);
}

/// Security event for audit logging.
///
/// Captures information about security-related actions for audit purposes.
///
/// # Example
///
/// ```rust
/// use airssys_wasm::core::security::traits::SecurityEvent;
/// use airssys_wasm::core::component::id::ComponentId;
///
/// let event = SecurityEvent {
///     component: ComponentId::new("org", "service", "inst-1"),
///     action: "send_message".to_string(),
///     resource: "org.other/target/inst-2".to_string(),
///     granted: true,
///     timestamp_ms: 1700000000000,
/// };
///
/// assert!(event.granted);
/// ```
#[derive(Debug, Clone)]
pub struct SecurityEvent {
    /// The component that performed the action.
    pub component: ComponentId,
    /// The action that was attempted.
    pub action: String,
    /// The resource being accessed.
    pub resource: String,
    /// Whether the action was granted.
    pub granted: bool,
    /// Timestamp in milliseconds since epoch.
    pub timestamp_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::security::capability::{MessagingAction, MessagingCapability};

    // SecurityEvent creation and field access tests
    #[test]
    fn test_security_event_creation() {
        let component = ComponentId::new("org", "service", "inst-1");
        let event = SecurityEvent {
            component: component.clone(),
            action: "send_message".to_string(),
            resource: "org.target/resource".to_string(),
            granted: true,
            timestamp_ms: 1700000000000,
        };
        assert_eq!(event.component, component);
        assert_eq!(event.action, "send_message");
    }

    #[test]
    fn test_security_event_field_access() {
        let event = SecurityEvent {
            component: ComponentId::new("org", "service", "inst-1"),
            action: "read_file".to_string(),
            resource: "/path/to/file".to_string(),
            granted: false,
            timestamp_ms: 1700000000000,
        };
        assert_eq!(event.action, "read_file");
        assert_eq!(event.resource, "/path/to/file");
        assert!(!event.granted);
        assert_eq!(event.timestamp_ms, 1700000000000);
    }

    #[test]
    fn test_security_event_clone() {
        let event1 = SecurityEvent {
            component: ComponentId::new("org", "service", "inst-1"),
            action: "write_file".to_string(),
            resource: "/path/to/file".to_string(),
            granted: true,
            timestamp_ms: 1700000000000,
        };
        let event2 = event1.clone();
        assert_eq!(event1.component, event2.component);
        assert_eq!(event1.action, event2.action);
        assert_eq!(event1.granted, event2.granted);
    }

    // Mock SecurityValidator implementation tests
    struct MockValidator;

    impl SecurityValidator for MockValidator {
        fn validate_capability(
            &self,
            _component: &ComponentId,
            _capability: &Capability,
        ) -> Result<(), SecurityError> {
            Ok(())
        }

        fn can_send_to(
            &self,
            _sender: &ComponentId,
            _target: &ComponentId,
        ) -> Result<(), SecurityError> {
            Ok(())
        }
    }

    #[test]
    fn test_mock_security_validator_validate_capability() {
        let validator = MockValidator;
        let component = ComponentId::new("org", "service", "inst-1");
        let cap = Capability::Messaging(MessagingCapability {
            action: MessagingAction::Send,
            target_pattern: "org.target/*".to_string(),
        });
        let result = validator.validate_capability(&component, &cap);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mock_security_validator_can_send_to() {
        let validator = MockValidator;
        let sender = ComponentId::new("org", "service", "inst-1");
        let target = ComponentId::new("org", "target", "inst-2");
        let result = validator.can_send_to(&sender, &target);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mock_security_validator_errors() {
        struct DenyingValidator;

        impl SecurityValidator for DenyingValidator {
            fn validate_capability(
                &self,
                _component: &ComponentId,
                _capability: &Capability,
            ) -> Result<(), SecurityError> {
                Err(SecurityError::CapabilityDenied("test".to_string()))
            }

            fn can_send_to(
                &self,
                _sender: &ComponentId,
                _target: &ComponentId,
            ) -> Result<(), SecurityError> {
                Err(SecurityError::PermissionDenied("test".to_string()))
            }
        }

        let validator = DenyingValidator;
        let component = ComponentId::new("org", "service", "inst-1");
        let cap = Capability::Messaging(MessagingCapability {
            action: MessagingAction::Send,
            target_pattern: "org.target/*".to_string(),
        });

        let result = validator.validate_capability(&component, &cap);
        assert!(result.is_err());
        assert!(matches!(result, Err(SecurityError::CapabilityDenied(_))));
    }

    // Mock SecurityAuditLogger implementation tests
    struct MockLogger;

    impl SecurityAuditLogger for MockLogger {
        fn log_event(&self, event: SecurityEvent) {
            // In a real implementation, this would write to a log
            // For testing, we just capture the event
            let _ = event;
        }
    }

    #[test]
    fn test_mock_security_audit_logger_logs_event() {
        let logger = MockLogger;
        let event = SecurityEvent {
            component: ComponentId::new("org", "service", "inst-1"),
            action: "test_action".to_string(),
            resource: "test_resource".to_string(),
            granted: true,
            timestamp_ms: 1700000000000,
        };

        // Just verify the method can be called without panicking
        logger.log_event(event);
    }

    #[test]
    fn test_security_audit_logger_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<MockLogger>();
    }

    // Gap analysis tests

    #[test]
    fn test_security_event_debug_format() {
        let event = SecurityEvent {
            component: ComponentId::new("org", "service", "1"),
            action: "test".to_string(),
            resource: "resource".to_string(),
            granted: true,
            timestamp_ms: 0,
        };
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("SecurityEvent"));
        assert!(debug_str.contains("granted"));
    }

    #[test]
    fn test_security_validator_trait_object_creation() {
        let validator: Box<dyn SecurityValidator> = Box::new(MockValidator);
        let component = ComponentId::new("org", "service", "1");
        let cap = Capability::Messaging(MessagingCapability {
            action: MessagingAction::Send,
            target_pattern: "*".to_string(),
        });
        assert!(validator.validate_capability(&component, &cap).is_ok());
    }

    #[test]
    fn test_security_validator_is_send_sync() {
        fn assert_send_sync<T: Send + Sync + ?Sized>() {}
        assert_send_sync::<dyn SecurityValidator>();
    }
}
