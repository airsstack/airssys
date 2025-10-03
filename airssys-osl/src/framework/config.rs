//! Configuration types for the OSL framework.
//!
//! This module provides configuration structures for the framework, including
//! security configuration, operation policies, and runtime settings.

use crate::core::result::OSResult;

/// Configuration for the OSL framework.
///
/// `OSLConfig` contains runtime configuration for the framework, focusing
/// on essential security policies and settings.
#[derive(Debug, Clone)]
pub struct OSLConfig {
    /// Security configuration settings
    pub security: SecurityConfig,
}

impl Default for OSLConfig {
    fn default() -> Self {
        Self {
            security: SecurityConfig::default(),
        }
    }
}

/// Builder for OSL framework configuration.
///
/// `OSLConfigBuilder` provides a fluent interface for constructing framework
/// configuration with validation and sensible defaults.
#[derive(Debug, Default)]
pub struct OSLConfigBuilder {
    security: Option<SecurityConfig>,
}

impl OSLConfigBuilder {
    /// Create a new configuration builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set security configuration.
    pub fn with_security(mut self, security: SecurityConfig) -> Self {
        self.security = Some(security);
        self
    }

    /// Build the configuration with validation.
    pub fn build(self) -> OSResult<OSLConfig> {
        let config = OSLConfig {
            security: self.security.unwrap_or_default(),
        };

        // Validate configuration consistency
        config.validate()?;

        Ok(config)
    }
}

impl OSLConfig {
    /// Validate configuration consistency.
    fn validate(&self) -> OSResult<()> {
        // TODO: Implement comprehensive validation in OSL-TASK-006
        // This will validate security policy consistency, etc.
        Ok(())
    }
}

/// Security configuration for the framework.
///
/// `SecurityConfig` defines security policies, access control settings, and
/// audit configuration for the framework.
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// Enable comprehensive security logging
    pub logging_enabled: bool,
    /// Path to security policy file (optional)
    pub policy_file: Option<std::path::PathBuf>,
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

/// Security policy enforcement levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnforcementLevel {
    /// Disable security policy enforcement (development only)
    Disabled,
    /// Log policy violations but allow operations to continue
    LogOnly,
    /// Enforce security policies and block violations
    Enforce,
}

impl Default for EnforcementLevel {
    fn default() -> Self {
        Self::Enforce
    }
}

/// Audit trail configuration.
#[derive(Debug, Clone)]
pub struct AuditConfig {
    /// Enable audit logging for all operations
    pub enabled: bool,
    /// Include operation input/output in audit logs
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
