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
    
    /// Adds a security attribute to this context.
    pub fn with_attribute(mut self, key: String, value: String) -> Self {
        self.attributes.insert(key, value);
        self
    }
}