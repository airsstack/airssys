//! Security configuration and policy types.
//!
//! This module provides security-related configuration structures for AirsSys OSL,
//! including security policies, enforcement levels, and audit configuration.

// Layer 1: Standard library imports
use std::path::PathBuf;

/// Security configuration for OS operations.
///
/// `SecurityConfig` defines security policies, access control settings, and
/// audit configuration for executing OS operations securely.
///
/// # Examples
///
/// ```rust
/// use airssys_osl::core::security::{SecurityConfig, EnforcementLevel};
///
/// // Default security configuration (enforce mode, audit enabled)
/// let config = SecurityConfig::default();
/// assert_eq!(config.enforcement_level, EnforcementLevel::Enforce);
/// assert!(config.audit_config.enabled);
///
/// // Custom security configuration
/// let config = SecurityConfig {
///     logging_enabled: true,
///     policy_file: Some("/etc/osl/policies.toml".into()),
///     enforcement_level: EnforcementLevel::LogOnly,
///     audit_config: Default::default(),
/// };
/// ```
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// Enable comprehensive security logging
    pub logging_enabled: bool,

    /// Path to security policy file (optional)
    pub policy_file: Option<PathBuf>,

    /// Enforcement level for security policies
    pub enforcement_level: EnforcementLevel,

    /// Audit trail configuration
    pub audit_config: AuditConfig,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            logging_enabled: true,
            policy_file: None,
            enforcement_level: EnforcementLevel::Enforce,
            audit_config: AuditConfig::default(),
        }
    }
}

impl SecurityConfig {
    /// Create a new security configuration with default settings.
    ///
    /// Defaults to enforce mode with audit logging enabled.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a security configuration with logging disabled.
    ///
    /// Useful for testing or development environments.
    pub fn without_logging() -> Self {
        Self {
            logging_enabled: false,
            ..Default::default()
        }
    }

    /// Create a security configuration with a specific policy file.
    pub fn with_policy_file(policy_file: PathBuf) -> Self {
        Self {
            policy_file: Some(policy_file),
            ..Default::default()
        }
    }
}

/// Security policy enforcement levels.
///
/// Defines how security policy violations should be handled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EnforcementLevel {
    /// Disable security policy enforcement.
    ///
    /// Security policies are not checked. Only use in development environments.
    Disabled,

    /// Log policy violations but allow operations to continue.
    ///
    /// Useful for monitoring and gradual policy rollout.
    LogOnly,

    /// Enforce security policies and block violations.
    ///
    /// This is the recommended setting for production environments.
    #[default]
    Enforce,
}

/// Audit trail configuration.
///
/// Controls what information is included in audit logs for security monitoring.
#[derive(Debug, Clone)]
pub struct AuditConfig {
    /// Enable audit logging for all operations
    pub enabled: bool,

    /// Include operation input/output in audit logs.
    ///
    /// **Security Note:** Disabled by default to prevent logging sensitive data.
    pub include_data: bool,

    /// Include detailed timing information
    pub include_timing: bool,

    /// Include security context in audit logs
    pub include_context: bool,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            include_data: false, // Security: don't log sensitive data by default
            include_timing: true,
            include_context: true,
        }
    }
}

impl AuditConfig {
    /// Create a new audit configuration with all features enabled.
    ///
    /// **Warning:** This includes sensitive data in logs. Only use in secure environments.
    pub fn full() -> Self {
        Self {
            enabled: true,
            include_data: true,
            include_timing: true,
            include_context: true,
        }
    }

    /// Create an audit configuration with minimal logging.
    pub fn minimal() -> Self {
        Self {
            enabled: true,
            include_data: false,
            include_timing: false,
            include_context: false,
        }
    }

    /// Disable all audit logging.
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            include_data: false,
            include_timing: false,
            include_context: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_config_default() {
        let config = SecurityConfig::default();
        assert!(config.logging_enabled);
        assert_eq!(config.enforcement_level, EnforcementLevel::Enforce);
        assert!(config.audit_config.enabled);
        assert!(!config.audit_config.include_data);
    }

    #[test]
    fn test_security_config_without_logging() {
        let config = SecurityConfig::without_logging();
        assert!(!config.logging_enabled);
        assert_eq!(config.enforcement_level, EnforcementLevel::Enforce);
    }

    #[test]
    fn test_enforcement_level_default() {
        assert_eq!(EnforcementLevel::default(), EnforcementLevel::Enforce);
    }

    #[test]
    fn test_audit_config_default() {
        let config = AuditConfig::default();
        assert!(config.enabled);
        assert!(!config.include_data);
        assert!(config.include_timing);
        assert!(config.include_context);
    }

    #[test]
    fn test_audit_config_full() {
        let config = AuditConfig::full();
        assert!(config.enabled);
        assert!(config.include_data);
        assert!(config.include_timing);
        assert!(config.include_context);
    }

    #[test]
    fn test_audit_config_disabled() {
        let config = AuditConfig::disabled();
        assert!(!config.enabled);
        assert!(!config.include_data);
        assert!(!config.include_timing);
        assert!(!config.include_context);
    }
}
