//! Execution context and security context types.
//!
//! This module defines the context types that carry state and security
//! information throughout the execution of operations.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Execution context for operation processing.
///
/// Contains all the contextual information needed for executing operations,
/// including security context, timing information, and execution metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    /// Unique identifier for this execution context
    pub execution_id: Uuid,

    /// When this execution context was created
    pub created_at: DateTime<Utc>,

    /// Security context for this execution
    pub security_context: SecurityContext,

    /// Additional metadata for this execution
    pub metadata: HashMap<String, String>,
}

impl ExecutionContext {
    /// Creates a new execution context with the given security context.
    pub fn new(security_context: SecurityContext) -> Self {
        Self {
            execution_id: Uuid::new_v4(),
            created_at: Utc::now(),
            security_context,
            metadata: HashMap::new(),
        }
    }

    /// Adds metadata to this execution context.
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Adds multiple metadata entries to this execution context.
    pub fn with_metadata_map(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata.extend(metadata);
        self
    }

    /// Gets metadata value by key.
    pub fn get_metadata(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).map(|s| s.as_str())
    }

    /// Returns the age of this execution context.
    pub fn age(&self) -> chrono::Duration {
        Utc::now() - self.created_at
    }

    /// Returns true if this execution context has expired (older than given duration).
    pub fn is_expired(&self, max_age: chrono::Duration) -> bool {
        self.age() > max_age
    }

    /// Returns the principal from the security context.
    pub fn principal(&self) -> &str {
        &self.security_context.principal
    }
}

/// Security context for operation authorization.
///
/// Contains security-related information used for operation authorization
/// and audit logging.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    /// Principal (user/service) executing the operation
    pub principal: String,

    /// Session identifier for audit tracking
    pub session_id: Uuid,

    /// When this security context was established
    pub established_at: DateTime<Utc>,

    /// Security attributes for this context
    pub attributes: HashMap<String, String>,
}

impl SecurityContext {
    /// Creates a new security context for the given principal.
    pub fn new(principal: String) -> Self {
        Self {
            principal,
            session_id: Uuid::new_v4(),
            established_at: Utc::now(),
            attributes: HashMap::new(),
        }
    }

    /// Creates a new security context with a custom session ID.
    pub fn with_session_id(principal: String, session_id: Uuid) -> Self {
        Self {
            principal,
            session_id,
            established_at: Utc::now(),
            attributes: HashMap::new(),
        }
    }

    /// Adds a security attribute to this context.
    pub fn with_attribute(mut self, key: String, value: String) -> Self {
        self.attributes.insert(key, value);
        self
    }

    /// Adds multiple security attributes to this context.
    pub fn with_attributes(mut self, attributes: HashMap<String, String>) -> Self {
        self.attributes.extend(attributes);
        self
    }

    /// Gets a security attribute value by key.
    pub fn get_attribute(&self, key: &str) -> Option<&str> {
        self.attributes.get(key).map(|s| s.as_str())
    }

    /// Returns true if this security context has the given attribute.
    pub fn has_attribute(&self, key: &str) -> bool {
        self.attributes.contains_key(key)
    }

    /// Returns the age of this security context.
    pub fn age(&self) -> chrono::Duration {
        Utc::now() - self.established_at
    }

    /// Returns true if this security context has expired.
    pub fn is_expired(&self, max_age: chrono::Duration) -> bool {
        self.age() > max_age
    }

    /// Returns true if this context represents an administrative principal.
    pub fn is_admin(&self) -> bool {
        self.get_attribute("role") == Some("admin")
    }

    /// Returns true if this context represents a service account.
    pub fn is_service_account(&self) -> bool {
        self.get_attribute("type") == Some("service")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_security_context_creation() {
        let ctx = SecurityContext::new("test@example.com".to_string());
        assert_eq!(ctx.principal, "test@example.com");
        assert!(!ctx.is_admin());
        assert!(!ctx.is_service_account());
        assert!(ctx.age().num_milliseconds() >= 0);
    }

    #[test]
    fn test_security_context_attributes() {
        let ctx = SecurityContext::new("admin@example.com".to_string())
            .with_attribute("role".to_string(), "admin".to_string())
            .with_attribute("type".to_string(), "user".to_string());

        assert!(ctx.is_admin());
        assert!(!ctx.is_service_account());
        assert_eq!(ctx.get_attribute("role"), Some("admin"));
        assert!(ctx.has_attribute("type"));
    }

    #[test]
    fn test_service_account_detection() {
        let ctx = SecurityContext::new("service-account".to_string())
            .with_attribute("type".to_string(), "service".to_string());

        assert!(ctx.is_service_account());
        assert!(!ctx.is_admin());
    }

    #[test]
    fn test_execution_context_creation() {
        let sec_ctx = SecurityContext::new("test@example.com".to_string());
        let exec_ctx = ExecutionContext::new(sec_ctx.clone());

        assert_eq!(exec_ctx.principal(), "test@example.com");
        assert!(exec_ctx.age().num_milliseconds() >= 0);
        assert!(!exec_ctx.is_expired(chrono::Duration::minutes(1)));
    }

    #[test]
    fn test_execution_context_metadata() {
        let sec_ctx = SecurityContext::new("test@example.com".to_string());
        let exec_ctx = ExecutionContext::new(sec_ctx)
            .with_metadata("request_id".to_string(), "req-123".to_string())
            .with_metadata("operation".to_string(), "test".to_string());

        assert_eq!(exec_ctx.get_metadata("request_id"), Some("req-123"));
        assert_eq!(exec_ctx.get_metadata("operation"), Some("test"));
        assert_eq!(exec_ctx.get_metadata("nonexistent"), None);
    }

    #[test]
    fn test_execution_context_with_metadata_map() {
        let sec_ctx = SecurityContext::new("test@example.com".to_string());
        let mut metadata = HashMap::new();
        metadata.insert("key1".to_string(), "value1".to_string());
        metadata.insert("key2".to_string(), "value2".to_string());

        let exec_ctx = ExecutionContext::new(sec_ctx).with_metadata_map(metadata);

        assert_eq!(exec_ctx.get_metadata("key1"), Some("value1"));
        assert_eq!(exec_ctx.get_metadata("key2"), Some("value2"));
    }
}
