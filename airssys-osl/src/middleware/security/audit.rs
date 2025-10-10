//! Security audit logging framework.
//!
//! This module provides comprehensive security audit logging for all
//! security decisions and events.

// Layer 1: Standard library imports
use std::fmt::Debug;

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// Layer 3: Internal module imports
use crate::core::context::SecurityContext;
use crate::middleware::security::policy::PolicyDecision;

/// Security event type for audit logging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityEventType {
    /// Access was granted to an operation
    AccessGranted,

    /// Access was denied to an operation
    AccessDenied,

    /// Security violation detected
    SecurityViolation,

    /// Additional authentication required
    AuthenticationRequired,

    /// Security policy evaluated
    PolicyEvaluated,
}

/// Security audit log entry.
///
/// Records a security event with full context for audit trails and
/// compliance reporting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAuditLog {
    /// When the event occurred (§3.2 - chrono `DateTime<Utc>`)
    pub timestamp: DateTime<Utc>,

    /// Type of security event
    pub event_type: SecurityEventType,

    /// Operation identifier
    pub operation_id: String,

    /// Principal who attempted the operation
    pub principal: String,

    /// Session ID for correlation
    pub session_id: String,

    /// Policy decision made
    pub decision: String,

    /// Which policy made the decision
    pub policy_applied: String,

    /// Additional metadata as JSON
    pub metadata: serde_json::Value,
}

impl SecurityAuditLog {
    /// Create a new security audit log entry.
    pub fn new(
        event_type: SecurityEventType,
        operation_id: String,
        context: &SecurityContext,
        decision: &PolicyDecision,
        policy_name: &str,
    ) -> Self {
        let decision_str = match decision {
            PolicyDecision::Allow => "Allow".to_string(),
            PolicyDecision::Deny(reason) => format!("Deny: {reason}"),
            PolicyDecision::RequireAdditionalAuth(_) => "RequireAuth".to_string(),
        };

        Self {
            timestamp: Utc::now(),
            event_type,
            operation_id,
            principal: context.principal.clone(),
            session_id: context.session_id.to_string(),
            decision: decision_str,
            policy_applied: policy_name.to_string(),
            metadata: serde_json::Value::Null,
        }
    }

    /// Add metadata to this audit log.
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Error type for audit logging operations.
#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    /// I/O error during audit logging
    #[error("Audit I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization error
    #[error("Audit serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Custom audit error
    #[error("Audit error: {0}")]
    Custom(String),
}

/// Trait for security audit logger implementations.
///
/// Security audit loggers receive security events and persist them
/// for compliance, monitoring, and incident response.
///
/// # Thread Safety
///
/// Implementations must be thread-safe (Send + Sync) as audit logging
/// may occur from multiple threads concurrently.
#[async_trait]
pub trait SecurityAuditLogger: Debug + Send + Sync + 'static {
    /// Log a security event.
    ///
    /// # Arguments
    ///
    /// * `event` - The security audit log entry to record
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the event was successfully logged, or an
    /// `AuditError` if logging failed.
    async fn log_security_event(&self, event: SecurityAuditLog) -> Result<(), AuditError>;

    /// Flush any buffered audit logs.
    ///
    /// This method ensures all pending audit logs are persisted.
    async fn flush(&self) -> Result<(), AuditError> {
        // Default implementation does nothing
        Ok(())
    }
}

/// Console-based security audit logger for development and testing.
#[derive(Debug, Default)]
pub struct ConsoleSecurityAuditLogger;

impl ConsoleSecurityAuditLogger {
    /// Create a new console security audit logger.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl SecurityAuditLogger for ConsoleSecurityAuditLogger {
    async fn log_security_event(&self, event: SecurityAuditLog) -> Result<(), AuditError> {
        // Format as JSON for structured logging
        let json = serde_json::to_string_pretty(&event)?;
        println!("[SECURITY AUDIT] {json}");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic)]
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::middleware::security::policy::AuthRequirement;

    #[test]
    fn test_security_event_type_equality() {
        assert_eq!(
            SecurityEventType::AccessGranted,
            SecurityEventType::AccessGranted
        );
        assert_ne!(
            SecurityEventType::AccessGranted,
            SecurityEventType::AccessDenied
        );
    }

    #[test]
    fn test_audit_log_creation() {
        let context = SecurityContext::new("test-user".to_string());
        let decision = PolicyDecision::Allow;

        let log = SecurityAuditLog::new(
            SecurityEventType::AccessGranted,
            "op-123".to_string(),
            &context,
            &decision,
            "test-policy",
        );

        assert_eq!(log.event_type, SecurityEventType::AccessGranted);
        assert_eq!(log.operation_id, "op-123");
        assert_eq!(log.principal, "test-user");
        assert_eq!(log.decision, "Allow");
        assert_eq!(log.policy_applied, "test-policy");
    }

    #[test]
    fn test_audit_log_with_deny() {
        let context = SecurityContext::new("test-user".to_string());
        let decision = PolicyDecision::Deny("Insufficient permissions".to_string());

        let log = SecurityAuditLog::new(
            SecurityEventType::AccessDenied,
            "op-456".to_string(),
            &context,
            &decision,
            "acl-policy",
        );

        assert_eq!(log.event_type, SecurityEventType::AccessDenied);
        assert!(log.decision.contains("Deny"));
        assert!(log.decision.contains("Insufficient permissions"));
    }

    #[tokio::test]
    async fn test_console_audit_logger() {
        let logger = ConsoleSecurityAuditLogger::new();
        let context = SecurityContext::new("test-user".to_string());
        let decision = PolicyDecision::Allow;

        let log = SecurityAuditLog::new(
            SecurityEventType::AccessGranted,
            "op-789".to_string(),
            &context,
            &decision,
            "test-policy",
        );

        let result = logger.log_security_event(log).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_audit_log_serialization() {
        let context = SecurityContext::new("test-user".to_string());
        let decision = PolicyDecision::Allow;

        let log = SecurityAuditLog::new(
            SecurityEventType::AccessGranted,
            "op-123".to_string(),
            &context,
            &decision,
            "test-policy",
        );

        // Test JSON serialization
        let json = serde_json::to_string(&log);
        assert!(json.is_ok());

        // Test deserialization
        let json_str = json.unwrap();
        let deserialized: Result<SecurityAuditLog, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());

        let log2 = deserialized.unwrap();
        assert_eq!(log.event_type, log2.event_type);
        assert_eq!(log.operation_id, log2.operation_id);
        assert_eq!(log.principal, log2.principal);
    }

    #[test]
    fn test_audit_log_with_metadata() {
        let context = SecurityContext::new("test-user".to_string());
        let decision = PolicyDecision::Allow;

        let metadata = serde_json::json!({
            "resource": "/etc/passwd",
            "action": "read"
        });

        let log = SecurityAuditLog::new(
            SecurityEventType::AccessGranted,
            "op-123".to_string(),
            &context,
            &decision,
            "test-policy",
        )
        .with_metadata(metadata.clone());

        assert_eq!(log.metadata, metadata);
    }

    #[test]
    fn test_security_violation_logging() {
        let context = SecurityContext::new("malicious-user".to_string());
        let decision = PolicyDecision::Deny("Security violation detected".to_string());

        let log = SecurityAuditLog::new(
            SecurityEventType::SecurityViolation,
            "op-666".to_string(),
            &context,
            &decision,
            "security-policy",
        );

        assert_eq!(log.event_type, SecurityEventType::SecurityViolation);
        assert!(log.decision.contains("Security violation"));
    }

    #[test]
    fn test_authentication_required_logging() {
        let context = SecurityContext::new("test-user".to_string());
        let decision = PolicyDecision::RequireAdditionalAuth(AuthRequirement::Custom(
            "MFA required".to_string(),
        ));

        let log = SecurityAuditLog::new(
            SecurityEventType::AuthenticationRequired,
            "op-999".to_string(),
            &context,
            &decision,
            "mfa-policy",
        );

        assert_eq!(log.event_type, SecurityEventType::AuthenticationRequired);
        assert_eq!(log.decision, "RequireAuth");
    }

    #[test]
    fn test_audit_log_timestamp_uses_utc() {
        let context = SecurityContext::new("test-user".to_string());
        let decision = PolicyDecision::Allow;

        let before = Utc::now();
        let log = SecurityAuditLog::new(
            SecurityEventType::AccessGranted,
            "op-time".to_string(),
            &context,
            &decision,
            "test-policy",
        );
        let after = Utc::now();

        // Verify timestamp is between before and after (§3.2 compliance)
        assert!(log.timestamp >= before);
        assert!(log.timestamp <= after);
    }

    #[tokio::test]
    async fn test_console_logger_with_deny() {
        let logger = ConsoleSecurityAuditLogger::new();
        let context = SecurityContext::new("unauthorized-user".to_string());
        let decision = PolicyDecision::Deny("Access denied".to_string());

        let log = SecurityAuditLog::new(
            SecurityEventType::AccessDenied,
            "op-deny".to_string(),
            &context,
            &decision,
            "acl-policy",
        );

        let result = logger.log_security_event(log).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_console_logger_flush() {
        let logger = ConsoleSecurityAuditLogger::new();
        let result = logger.flush().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_policy_evaluated_event() {
        let context = SecurityContext::new("test-user".to_string());
        let decision = PolicyDecision::Allow;

        let log = SecurityAuditLog::new(
            SecurityEventType::PolicyEvaluated,
            "op-eval".to_string(),
            &context,
            &decision,
            "combined-policy",
        );

        assert_eq!(log.event_type, SecurityEventType::PolicyEvaluated);
        assert_eq!(log.policy_applied, "combined-policy");
    }

    #[test]
    fn test_audit_error_types() {
        // Test IoError variant
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied");
        let audit_err: AuditError = io_err.into();
        assert!(audit_err.to_string().contains("Audit I/O error"));

        // Test Custom variant
        let custom_err = AuditError::Custom("Test error".to_string());
        assert!(custom_err.to_string().contains("Test error"));
    }
}
