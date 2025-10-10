//! SecurityMiddleware implementation.
//!
//! This module provides the core SecurityMiddleware that integrates
//! security policies with the middleware pipeline.

// Layer 1: Standard library imports
use std::fmt::Debug;
use std::sync::Arc;

// Layer 2: Third-party crate imports
use async_trait::async_trait;

// Layer 3: Internal module imports
use crate::core::context::ExecutionContext;
use crate::core::middleware::{Middleware, MiddlewareError, MiddlewareResult};
use crate::core::operation::Operation;
use crate::core::security::SecurityConfig;
use crate::middleware::security::audit::{ConsoleSecurityAuditLogger, SecurityAuditLogger};
use crate::middleware::security::policy::SecurityPolicy;

/// SecurityMiddleware for policy-based access control.
///
/// This middleware enforces security policies before operations are executed.
/// It runs with priority 100 (highest priority) to ensure security checks
/// happen before all other middleware.
///
/// # Architecture
///
/// - **Priority 100**: Runs FIRST before all other middleware
/// - **Deny-by-default**: Operations denied unless explicitly allowed
/// - **Policy composition**: Multiple policies evaluated in order
/// - **Comprehensive audit**: All decisions logged for compliance
///
/// # Example Usage
///
/// ```rust,no_run
/// use airssys_osl::middleware::security::{
///     SecurityMiddleware, SecurityMiddlewareBuilder,
/// };
/// use airssys_osl::core::security::SecurityConfig;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Build security middleware (Phase 1 - basic structure)
/// let security = SecurityMiddlewareBuilder::new()
///     .with_config(SecurityConfig::default())
///     .build()?;
///
/// // TODO: Phase 2 will add ACL/RBAC policy support
/// // Security middleware runs with priority 100 (first in pipeline)
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct SecurityMiddleware {
    config: SecurityConfig,
    audit_logger: Arc<dyn SecurityAuditLogger>,
    policies: Vec<Box<dyn SecurityPolicy>>,
}

impl SecurityMiddleware {
    /// Create a new SecurityMiddleware with default configuration.
    pub fn new() -> Self {
        Self {
            config: SecurityConfig::default(),
            audit_logger: Arc::new(ConsoleSecurityAuditLogger::new()),
            policies: Vec::new(),
        }
    }

    /// Create a new SecurityMiddleware with custom configuration.
    pub fn with_config(config: SecurityConfig) -> Self {
        Self {
            config,
            audit_logger: Arc::new(ConsoleSecurityAuditLogger::new()),
            policies: Vec::new(),
        }
    }

    /// Create a new SecurityMiddleware with custom audit logger.
    pub fn with_audit_logger(
        config: SecurityConfig,
        audit_logger: Arc<dyn SecurityAuditLogger>,
    ) -> Self {
        Self {
            config,
            audit_logger,
            policies: Vec::new(),
        }
    }

    /// Get reference to the security configuration.
    pub fn config(&self) -> &SecurityConfig {
        &self.config
    }

    /// Get reference to the audit logger.
    pub fn audit_logger(&self) -> &Arc<dyn SecurityAuditLogger> {
        &self.audit_logger
    }

    /// Get the number of policies configured.
    pub fn policy_count(&self) -> usize {
        self.policies.len()
    }
}

impl Default for SecurityMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<O: Operation> Middleware<O> for SecurityMiddleware {
    fn name(&self) -> &str {
        "security"
    }

    fn priority(&self) -> u32 {
        100 // Highest priority - security runs FIRST
    }

    fn is_enabled(&self) -> bool {
        // Security middleware is enabled unless explicitly disabled
        self.config.logging_enabled
    }

    async fn before_execution(
        &self,
        operation: O,
        context: &ExecutionContext,
    ) -> MiddlewareResult<Option<O>> {
        use crate::middleware::security::audit::{SecurityAuditLog, SecurityEventType};
        use crate::middleware::security::policy::PolicyDecision;

        // If no policies configured, deny by default (secure default)
        if self.policies.is_empty() {
            let reason = "No security policies configured - deny by default".to_string();

            // Log security denial
            let log = SecurityAuditLog::new(
                SecurityEventType::AccessDenied,
                operation.operation_id().to_string(),
                &context.security_context,
                &PolicyDecision::Deny(reason.clone()),
                "deny-by-default",
            );

            if let Err(e) = self.audit_logger.log_security_event(log).await {
                eprintln!("Failed to log security denial: {e}");
            }

            return Err(MiddlewareError::SecurityViolation(reason));
        }

        // Evaluate all policies - deny if ANY policy denies
        let mut auth_requirements = Vec::new();

        for policy in &self.policies {
            // Evaluate policy using only SecurityContext
            let decision = policy.evaluate(&context.security_context);

            // Log the policy decision
            let log = SecurityAuditLog::new(
                match &decision {
                    PolicyDecision::Allow => SecurityEventType::AccessGranted,
                    PolicyDecision::Deny(_) => SecurityEventType::AccessDenied,
                    PolicyDecision::RequireAdditionalAuth(_) => {
                        SecurityEventType::AuthenticationRequired
                    }
                },
                operation.operation_id().to_string(),
                &context.security_context,
                &decision,
                policy.description(),
            );

            if let Err(e) = self.audit_logger.log_security_event(log).await {
                eprintln!("Failed to log policy decision: {e}");
            }

            // Process the decision
            match decision {
                PolicyDecision::Allow => {
                    // Policy allows - continue to next policy
                    continue;
                }
                PolicyDecision::Deny(reason) => {
                    // ANY deny decision blocks the operation immediately
                    return Err(MiddlewareError::SecurityViolation(format!(
                        "Policy '{}' denied: {}",
                        policy.description(),
                        reason
                    )));
                }
                PolicyDecision::RequireAdditionalAuth(auth_req) => {
                    // Collect auth requirements (will be processed after all policies)
                    auth_requirements.push((policy.description().to_string(), auth_req));
                }
            }
        }

        // If we have auth requirements, we should handle them
        // For now, we'll just log them (future: could attach to operation metadata)
        if !auth_requirements.is_empty() {
            for (policy_name, auth_req) in auth_requirements {
                eprintln!("Policy '{policy_name}' requires additional auth: {auth_req:?}");
            }
            // TODO: Future enhancement - attach auth requirements to operation
            // For now, we allow the operation to proceed (logged for awareness)
        }

        // All policies passed (or only required additional auth)
        // Log overall approval
        let policy_count_msg = format!("{} policies evaluated", self.policies.len());
        let log = SecurityAuditLog::new(
            SecurityEventType::PolicyEvaluated,
            operation.operation_id().to_string(),
            &context.security_context,
            &PolicyDecision::Allow,
            &policy_count_msg,
        );

        if let Err(e) = self.audit_logger.log_security_event(log).await {
            eprintln!("Failed to log policy approval: {e}");
        }

        Ok(Some(operation))
    }
}

/// Builder for SecurityMiddleware.
///
/// Provides a fluent API for constructing SecurityMiddleware with
/// custom policies, configuration, and audit logging.
#[derive(Debug)]
pub struct SecurityMiddlewareBuilder {
    config: SecurityConfig,
    audit_logger: Option<Arc<dyn SecurityAuditLogger>>,
    policies: Vec<Box<dyn SecurityPolicy>>,
}

impl SecurityMiddlewareBuilder {
    /// Create a new SecurityMiddlewareBuilder.
    pub fn new() -> Self {
        Self {
            config: SecurityConfig::default(),
            audit_logger: None,
            policies: Vec::new(),
        }
    }

    /// Set the security configuration.
    pub fn with_config(mut self, config: SecurityConfig) -> Self {
        self.config = config;
        self
    }

    /// Set a custom audit logger.
    pub fn with_audit_logger(mut self, logger: Arc<dyn SecurityAuditLogger>) -> Self {
        self.audit_logger = Some(logger);
        self
    }

    /// Add a security policy.
    ///
    /// Policies are evaluated in the order they are added. If any policy
    /// denies an operation, the operation is immediately denied.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use airssys_osl::middleware::security::{
    ///     SecurityMiddlewareBuilder, AccessControlList, RoleBasedAccessControl,
    /// };
    /// use airssys_osl::core::security::SecurityConfig;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let acl = AccessControlList::new();
    /// let rbac = RoleBasedAccessControl::new();
    ///
    /// let middleware = SecurityMiddlewareBuilder::new()
    ///     .with_config(SecurityConfig::default())
    ///     .add_policy(Box::new(acl))
    ///     .add_policy(Box::new(rbac))
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn add_policy(mut self, policy: Box<dyn SecurityPolicy>) -> Self {
        self.policies.push(policy);
        self
    }

    /// Build the SecurityMiddleware.
    pub fn build(self) -> Result<SecurityMiddleware, String> {
        let audit_logger = self
            .audit_logger
            .unwrap_or_else(|| Arc::new(ConsoleSecurityAuditLogger::new()));

        Ok(SecurityMiddleware {
            config: self.config,
            audit_logger,
            policies: self.policies,
        })
    }
}

impl Default for SecurityMiddlewareBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operations::filesystem::read::FileReadOperation;

    #[test]
    fn test_security_middleware_creation() {
        let middleware = SecurityMiddleware::new();
        // Use fully qualified trait calls since Middleware is generic
        assert_eq!(
            <SecurityMiddleware as Middleware<FileReadOperation>>::name(&middleware),
            "security"
        );
        assert_eq!(
            <SecurityMiddleware as Middleware<FileReadOperation>>::priority(&middleware),
            100
        );
        assert!(<SecurityMiddleware as Middleware<FileReadOperation>>::is_enabled(&middleware));
    }

    #[test]
    fn test_security_middleware_with_config() {
        let config = SecurityConfig::without_logging();
        let middleware = SecurityMiddleware::with_config(config);
        assert!(!<SecurityMiddleware as Middleware<FileReadOperation>>::is_enabled(&middleware));
    }

    #[test]
    fn test_security_middleware_builder() {
        let result = SecurityMiddlewareBuilder::new()
            .with_config(SecurityConfig::default())
            .build();

        assert!(result.is_ok());
        if let Ok(middleware) = result {
            assert_eq!(
                <SecurityMiddleware as Middleware<FileReadOperation>>::priority(&middleware),
                100
            );
            assert!(<SecurityMiddleware as Middleware<FileReadOperation>>::is_enabled(&middleware));
        }
    }

    #[test]
    fn test_security_middleware_default() {
        let middleware = SecurityMiddleware::default();
        assert_eq!(
            <SecurityMiddleware as Middleware<FileReadOperation>>::name(&middleware),
            "security"
        );
        assert!(<SecurityMiddleware as Middleware<FileReadOperation>>::is_enabled(&middleware));
    }
}
