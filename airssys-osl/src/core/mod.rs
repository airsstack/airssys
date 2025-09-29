//! Core module for airssys-osl foundational abstractions.
//!
//! This module contains the essential trait definitions, types, and abstractions
//! that form the foundation of the OS Layer Framework. All other components
//! build upon these core abstractions.

pub mod context;
pub mod executor;
pub mod middleware;
pub mod operation;
pub mod result;

// Re-exports for public API
pub use context::{ExecutionContext, SecurityContext};
pub use executor::{ExecutionResult, OSExecutor};
pub use middleware::{ErrorAction, Middleware, MiddlewareError, MiddlewareResult};
pub use operation::{Operation, OperationType, Permission};
pub use result::{OSError, OSResult};
