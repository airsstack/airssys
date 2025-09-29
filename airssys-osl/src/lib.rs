//! AirssysOSL - Operating System Layer for Airssys
//!
//! This crate provides the core operating system abstraction layer for the Airssys platform.
//! It defines essential types, traits, and functionality for secure, cross-platform
//! operating system interactions.
//!
//! # Architecture
//!
//! The OSL is built around several core concepts:
//!
//! - **Operations**: Abstract representations of OS-level operations
//! - **Executors**: Components that can execute specific types of operations  
//! - **Middleware**: Cross-cutting concerns like logging, security, and validation
//! - **Context**: Execution context including security and metadata
//! - **Results**: Standardized result types with comprehensive error handling
//!
//! # Quick Start
//!
//! ```rust
//! use airssys_osl::core::context::{SecurityContext, ExecutionContext};
//! use airssys_osl::core::operation::OperationType;
//!
//! let security_context = SecurityContext::new("user123".to_string());
//! let execution_context = ExecutionContext::new(security_context);
//!
//! assert_eq!(execution_context.principal(), "user123");
//! ```
//!
//! # Complete Examples
//!
//! For comprehensive usage examples, see the executable examples in the
//! `examples/` directory. Run them with:
//!
//! ```bash
//! cargo run --example basic_usage
//! cargo run --example filesystem_pipeline
//! ```
//!
//! # Core Modules
//!
//! ## [`core`] - Foundational Framework Abstractions
//! **Primary Module**: Contains all essential traits, types, and abstractions for the OSL framework
//!
//! - **[`core::context`]** - Execution and security context management
//!   - Manages security boundaries and execution metadata
//!   - Provides audit trail and permission enforcement
//!
//! - **[`core::executor`]** - Operation executor framework
//!   - Defines contracts for OS operation execution
//!   - Handles standardized result processing
//!
//! - **[`core::middleware`]** - Cross-cutting concerns pipeline
//!   - Interceptor patterns for logging, validation, monitoring
//!   - Composable request/response processing
//!
//! - **[`core::operation`]** - Operation modeling and permissions
//!   - Abstract representations of system operations
//!   - Type-safe permission and capability system
//!
//! - **[`core::result`]** - Comprehensive error handling
//!   - Structured error types with context
//!   - Consistent result propagation patterns
//!
//! ## Module Integration Philosophy
//!
//! This library uses **explicit module imports** instead of crate-level re-exports
//! to maintain clear architectural boundaries. Import specific types from their modules:
//!
//! ```rust
//! use airssys_osl::core::context::ExecutionContext;
//! use airssys_osl::core::operation::OperationType;
//! ```
//!
//! This approach provides:
//! - **Clear dependency tracking**: Easy to understand what each component uses
//! - **Better IDE support**: Precise navigation and completion
//! - **Maintainable architecture**: Explicit module boundaries prevent coupling

pub mod core;
