//! WASM capability audit logging.
//!
//! This module provides audit logging for all WASM capability checks,
//! integrating with airssys-osl's security audit framework for unified
//! logging, compliance, and forensic analysis.

//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │ check_capability(component_id, resource, permission)    │
//! │   ├─> Perform capability check (DashMap lookup + ACL)  │
//! │   └─> WasmAuditLogger::log_capability_check()          │
//! │         └─> Convert to SecurityAuditLog (OSL format)   │
//! │             └─> SecurityAuditLogger::log_security_event │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! # Performance
//!
//! - **Async Logging:** Non-blocking via `tokio::spawn` (~1-5μs overhead)
//! - **Best-Effort:** Logging failures don't prevent capability checks
//! - **Runtime Detection:** Only logs when Tokio runtime is available
//!
//! # Compliance
//!
//! Audit logging supports compliance with:
//! - **GDPR:** Right to access (Article 15) - audit trail of data access
//! - **SOC 2:** Logical access controls - who accessed what and when
//! - **ISO 27001:** Access control monitoring - comprehensive audit logs
//!
//! # Custom Logger Example
//!
//! ```no_run
//! use std::sync::Arc;
//! use airssys_wasm::security::audit::{WasmAuditLogger, WasmCapabilityAuditLog};
//! use airssys_osl::middleware::security::audit::{SecurityAuditLogger, SecurityAuditLog, AuditError};
//! use async_trait::async_trait;
//!
//! # #[derive(Debug)]
//! # struct FileAuditLogger;
//! #[async_trait]
//! impl SecurityAuditLogger for FileAuditLogger {
//!     async fn log_security_event(&self, event: SecurityAuditLog) -> Result<(), AuditError> {
//!         // Write to file, database, etc.
//!         Ok(())
//!     }
//! }
//!
//! # fn main() {
//! // Inject custom logger
//! let file_logger = Arc::new(FileAuditLogger);
//! let wasm_logger = WasmAuditLogger::new(file_logger);
//! airssys_wasm::security::enforcement::set_global_audit_logger(wasm_logger).unwrap();
//! # }
//! ```

// Layer 1: Standard library imports
use std::sync::Arc;

// Layer 2: Third-party crate imports
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// Layer 3: airssys-osl imports
use airssys_osl::middleware::security::audit::{
    SecurityAuditLog, SecurityAuditLogger, SecurityEventType,
};
use airssys_osl::middleware::security::policy::PolicyDecision;
use airssys_osl::core::context::SecurityContext;

// Layer 4: Internal imports
use crate::security::enforcement::CapabilityCheckError;

// ═════════════════════════════════════════════════════════════════════════════
// WASM Capability Audit Log Type
// ═════════════════════════════════════════════════════════════════════════════

/// Result type for capability checks (for audit logging).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityCheckResultType {
    /// Capability check granted
    Granted,
    /// Capability check denied
    Denied,
}

/// WASM capability check audit log entry.
///
/// Records all capability checks (granted and denied) with full context
/// for security monitoring, compliance (GDPR, SOC2), and forensic analysis.
///
/// # Examples
///
/// ```
/// use airssys_wasm::security::audit::{WasmCapabilityAuditLog, CapabilityCheckResultType};
/// use chrono::Utc;
///
/// let log = WasmCapabilityAuditLog {
///     timestamp: Utc::now(),
///     component_id: "component-123".to_string(),
///     resource: "/data/file.txt".to_string(),
///     permission: "read".to_string(),
///     result: CapabilityCheckResultType::Granted,
///     trust_level: Some("trusted".to_string()),
///     denial_reason: None,
///     metadata: serde_json::json!({"source": "filesystem"}),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmCapabilityAuditLog {
    /// Timestamp when the capability check occurred (§3.2 - chrono DateTime&lt;Utc&gt;)
    pub timestamp: DateTime<Utc>,

    /// Component ID requesting the capability
    pub component_id: String,

    /// Resource path/pattern being accessed
    pub resource: String,

    /// Permission requested (read, write, execute, etc.)
    pub permission: String,

    /// Check result (granted or denied)
    pub result: CapabilityCheckResultType,

    /// Trust level of component (if available from Phase 2)
    pub trust_level: Option<String>,

    /// Denial reason (if denied)
    pub denial_reason: Option<String>,

    /// Additional metadata (JSON)
    pub metadata: serde_json::Value,
}

impl WasmCapabilityAuditLog {
    /// Create a new WASM capability audit log (granted).
    pub fn granted(
        component_id: impl Into<String>,
        resource: impl Into<String>,
        permission: impl Into<String>,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            component_id: component_id.into(),
            resource: resource.into(),
            permission: permission.into(),
            result: CapabilityCheckResultType::Granted,
            trust_level: None,
            denial_reason: None,
            metadata: serde_json::Value::Null,
        }
    }

    /// Create a new WASM capability audit log (denied).
    pub fn denied(
        component_id: impl Into<String>,
        resource: impl Into<String>,
        permission: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            component_id: component_id.into(),
            resource: resource.into(),
            permission: permission.into(),
            result: CapabilityCheckResultType::Denied,
            trust_level: None,
            denial_reason: Some(reason.into()),
            metadata: serde_json::Value::Null,
        }
    }

    /// Add trust level to this audit log.
    #[must_use]
    pub fn with_trust_level(mut self, trust_level: impl Into<String>) -> Self {
        self.trust_level = Some(trust_level.into());
        self
    }

    /// Add metadata to this audit log.
    #[must_use]
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Convert to airssys-osl SecurityAuditLog for unified logging.
    ///
    /// Maps WASM-specific audit log to OSL format for integration
    /// with airssys-osl security audit infrastructure.
    pub fn to_osl_audit_log(&self) -> SecurityAuditLog {
        let event_type = match self.result {
            CapabilityCheckResultType::Granted => SecurityEventType::AccessGranted,
            CapabilityCheckResultType::Denied => SecurityEventType::AccessDenied,
        };

        let decision = match self.result {
            CapabilityCheckResultType::Granted => PolicyDecision::Allow,
            CapabilityCheckResultType::Denied => PolicyDecision::Deny(
                self.denial_reason
                    .clone()
                    .unwrap_or_else(|| "Capability check failed".to_string()),
            ),
        };

        // Create minimal SecurityContext for OSL compatibility
        let context = SecurityContext::new(self.component_id.clone());

        let operation_id = format!("{}:{}", self.resource, self.permission);

        SecurityAuditLog::new(
            event_type,
            operation_id,
            &context,
            &decision,
            "wasm-capability-check",
        )
        .with_metadata(self.metadata.clone())
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// WASM Audit Logger
// ═════════════════════════════════════════════════════════════════════════════

/// WASM capability audit logger.
///
/// Logs all capability checks using airssys-osl SecurityAuditLogger
/// with WASM-specific context (component ID, resource, permission).
///
/// # Thread Safety
///
/// This type is thread-safe (Send + Sync) and can be shared across threads.
///
/// # Examples
///
/// ```no_run
/// use std::sync::Arc;
/// use airssys_wasm::security::audit::{WasmAuditLogger, WasmCapabilityAuditLog};
/// use airssys_osl::middleware::security::audit::ConsoleSecurityAuditLogger;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create logger with console output
/// let console_logger = Arc::new(ConsoleSecurityAuditLogger::new());
/// let audit_logger = WasmAuditLogger::new(console_logger);
///
/// // Log a capability check
/// let log = WasmCapabilityAuditLog::granted("component-123", "/data/file.txt", "read");
/// audit_logger.log_capability_check(log).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct WasmAuditLogger {
    /// Underlying OSL audit logger
    logger: Arc<dyn SecurityAuditLogger>,
}

impl WasmAuditLogger {
    /// Create new WASM audit logger wrapping an OSL logger.
    pub fn new(logger: Arc<dyn SecurityAuditLogger>) -> Self {
        Self { logger }
    }

    /// Log a capability check (granted or denied).
    ///
    /// # Arguments
    ///
    /// * `log` - The WASM capability audit log entry
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if logged successfully, or a `CapabilityCheckError` if logging failed.
    ///
    /// # Performance
    ///
    /// This method is async and non-blocking to meet <100ns overhead target.
    pub async fn log_capability_check(
        &self,
        log: WasmCapabilityAuditLog,
    ) -> Result<(), CapabilityCheckError> {
        // Convert to OSL SecurityAuditLog
        let osl_log = log.to_osl_audit_log();

        // Async log (non-blocking)
        self.logger
            .log_security_event(osl_log)
            .await
            .map_err(|e| CapabilityCheckError::AuditLogError {
                reason: format!("Failed to log capability check: {e}"),
            })
    }

    /// Get reference to underlying OSL logger.
    pub fn inner(&self) -> &Arc<dyn SecurityAuditLogger> {
        &self.logger
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Tests
// ═════════════════════════════════════════════════════════════════════════════

#[allow(clippy::expect_used, clippy::unwrap_used, clippy::unwrap_err_used, clippy::expect_err_used, clippy::panic, clippy::unwrap_on_result, clippy::indexing_slicing, clippy::too_many_arguments, clippy::type_complexity, reason = "test code")]
#[cfg(test)]
mod tests {
    use super::*;
    use airssys_osl::middleware::security::audit::ConsoleSecurityAuditLogger;

    #[test]
    fn test_audit_log_granted() {
        let log = WasmCapabilityAuditLog::granted("component-123", "/data/file.txt", "read");

        assert_eq!(log.component_id, "component-123");
        assert_eq!(log.resource, "/data/file.txt");
        assert_eq!(log.permission, "read");
        assert_eq!(log.result, CapabilityCheckResultType::Granted);
        assert!(log.denial_reason.is_none());
    }

    #[test]
    fn test_audit_log_denied() {
        let log = WasmCapabilityAuditLog::denied(
            "component-456",
            "/data/secret.txt",
            "write",
            "Permission denied",
        );

        assert_eq!(log.component_id, "component-456");
        assert_eq!(log.resource, "/data/secret.txt");
        assert_eq!(log.permission, "write");
        assert_eq!(log.result, CapabilityCheckResultType::Denied);
        assert_eq!(log.denial_reason.as_deref(), Some("Permission denied"));
    }

    #[test]
    fn test_audit_log_with_trust_level() {
        let log = WasmCapabilityAuditLog::granted("component-123", "/data/file.txt", "read")
            .with_trust_level("trusted");

        assert_eq!(log.trust_level.as_deref(), Some("trusted"));
    }

    #[test]
    fn test_audit_log_with_metadata() {
        let metadata = serde_json::json!({"source": "filesystem", "size": 1024});
        let log = WasmCapabilityAuditLog::granted("component-123", "/data/file.txt", "read")
            .with_metadata(metadata.clone());

        assert_eq!(log.metadata, metadata);
    }

    #[test]
    fn test_to_osl_audit_log_granted() {
        let log = WasmCapabilityAuditLog::granted("component-123", "/data/file.txt", "read");
        let osl_log = log.to_osl_audit_log();

        assert_eq!(osl_log.event_type, SecurityEventType::AccessGranted);
        assert_eq!(osl_log.operation_id, "/data/file.txt:read");
        assert_eq!(osl_log.principal, "component-123");
        assert_eq!(osl_log.decision, "Allow");
        assert_eq!(osl_log.policy_applied, "wasm-capability-check");
    }

    #[test]
    fn test_to_osl_audit_log_denied() {
        let log = WasmCapabilityAuditLog::denied(
            "component-456",
            "/data/secret.txt",
            "write",
            "Insufficient permissions",
        );
        let osl_log = log.to_osl_audit_log();

        assert_eq!(osl_log.event_type, SecurityEventType::AccessDenied);
        assert_eq!(osl_log.operation_id, "/data/secret.txt:write");
        assert_eq!(osl_log.principal, "component-456");
        assert!(osl_log.decision.contains("Insufficient permissions"));
        assert_eq!(osl_log.policy_applied, "wasm-capability-check");
    }

    #[test]
    fn test_wasm_audit_logger_creation() {
        let console_logger = Arc::new(ConsoleSecurityAuditLogger::new());
        let _audit_logger = WasmAuditLogger::new(console_logger.clone());

        // Arc::ptr_eq cannot compare trait objects directly
    }

    #[tokio::test]
    async fn test_wasm_audit_logger_log_granted() {
        let console_logger = Arc::new(ConsoleSecurityAuditLogger::new());
        let audit_logger = WasmAuditLogger::new(console_logger);

        let log = WasmCapabilityAuditLog::granted("component-123", "/data/file.txt", "read");
        let result = audit_logger.log_capability_check(log).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_wasm_audit_logger_log_denied() {
        let console_logger = Arc::new(ConsoleSecurityAuditLogger::new());
        let audit_logger = WasmAuditLogger::new(console_logger);

        let log = WasmCapabilityAuditLog::denied(
            "component-456",
            "/data/secret.txt",
            "write",
            "Permission denied",
        );
        let result = audit_logger.log_capability_check(log).await;

        assert!(result.is_ok());
    }

    #[test]
    fn test_result_type_serialization() {
        let granted = CapabilityCheckResultType::Granted;
        let denied = CapabilityCheckResultType::Denied;

        let granted_json = serde_json::to_string(&granted).unwrap();
        let denied_json = serde_json::to_string(&denied).unwrap();

        assert_eq!(granted_json, r#""Granted""#);
        assert_eq!(denied_json, r#""Denied""#);
    }

    #[test]
    fn test_audit_log_serialization() {
        let log = WasmCapabilityAuditLog::granted("component-123", "/data/file.txt", "read")
            .with_trust_level("trusted")
            .with_metadata(serde_json::json!({"source": "filesystem"}));

        let json = serde_json::to_string(&log).unwrap();
        let deserialized: WasmCapabilityAuditLog = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.component_id, log.component_id);
        assert_eq!(deserialized.resource, log.resource);
        assert_eq!(deserialized.permission, log.permission);
        assert_eq!(deserialized.result, log.result);
        assert_eq!(deserialized.trust_level, log.trust_level);
    }
}
