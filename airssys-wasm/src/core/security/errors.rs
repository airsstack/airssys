//! Security error types.

use thiserror::Error;

/// Security-related errors for capability validation and policy enforcement.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SecurityError {
    /// Capability was denied for the requested operation.
    #[error("Capability denied: {0}")]
    CapabilityDenied(String),

    /// Security policy was violated.
    #[error("Policy violation: {0}")]
    PolicyViolation(String),

    /// Security context is invalid or missing.
    #[error("Invalid context: {0}")]
    InvalidContext(String),

    /// Permission denied for the requested operation.
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_denied_display() {
        let err = SecurityError::CapabilityDenied("messaging".to_string());
        assert_eq!(err.to_string(), "Capability denied: messaging");
    }

    #[test]
    fn test_policy_violation_display() {
        let err = SecurityError::PolicyViolation("invalid access".to_string());
        assert_eq!(err.to_string(), "Policy violation: invalid access");
    }

    #[test]
    fn test_invalid_context_display() {
        let err = SecurityError::InvalidContext("missing component".to_string());
        assert_eq!(err.to_string(), "Invalid context: missing component");
    }

    #[test]
    fn test_permission_denied_display() {
        let err = SecurityError::PermissionDenied("unauthorized".to_string());
        assert_eq!(err.to_string(), "Permission denied: unauthorized");
    }

    #[test]
    fn test_security_error_clone() {
        let err1 = SecurityError::CapabilityDenied("test".to_string());
        let err2 = err1.clone();
        assert_eq!(err1, err2);
    }

    #[test]
    fn test_security_error_equality() {
        let err1 = SecurityError::PolicyViolation("test".to_string());
        let err2 = SecurityError::PolicyViolation("test".to_string());
        let err3 = SecurityError::PolicyViolation("different".to_string());

        assert_eq!(err1, err2);
        assert_ne!(err1, err3);
    }
}
