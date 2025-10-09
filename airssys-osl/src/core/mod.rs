//! Core module for airssys-osl foundational abstractions.
//!
//! This module contains the essential trait definitions, types, and abstractions
//! that form the foundation of the OS Layer Framework. All other components
//! build upon these core abstractions.
//!
//! ## Module Architecture
//!
//! The core module is organized into distinct sub-modules, each with specific
//! responsibilities that together form a cohesive framework:
//!
//! ### [`context`] - Execution and Security Context Management
//! **Responsibility**: Manages execution environments and security boundaries
//! - `ExecutionContext`: Tracks operation metadata, timing, and context information
//! - `SecurityContext`: Enforces permission models and access control policies
//! - **Integration**: Used by all operations to maintain security and audit trails
//!
//! ### [`executor`] - Operation Execution Framework  
//! **Responsibility**: Defines the core execution contract and result handling
//! - `OSExecutor`: Main trait for implementing OS operation executors
//! - `ExecutionResult`: Standardized result type for all operations
//! - **Integration**: Implemented by platform-specific backends, consumed by middleware
//!
//! ### [`middleware`] - Request/Response Processing Pipeline
//! **Responsibility**: Provides interceptor patterns for cross-cutting concerns
//! - `Middleware`: Trait for implementing request/response interceptors
//! - `MiddlewareError` & `MiddlewareResult`: Error handling for pipeline processing
//! - `ErrorAction`: Defines how middleware handles operation failures
//! - **Integration**: Enables logging, monitoring, caching, and validation layers
//!
//! ### [`operation`] - Operation Modeling and Permission System
//! **Responsibility**: Models system operations and defines permission requirements
//! - `Operation`: Core trait representing any system operation
//! - `OperationType`: Categorizes operations (filesystem, process, network, utility)
//! - `Permission`: Models access control requirements for operations
//! - **Integration**: Used by executors to understand and validate operations
//!
//! ### [`result`] - Error Handling and Result Types
//! **Responsibility**: Provides comprehensive error modeling and result handling
//! - `OSError`: Structured error types with context and categorization
//! - `OSResult<T>`: Standard Result type for all framework operations
//! - **Integration**: Used throughout the framework for consistent error propagation
//!
//! ## Design Principles
//!
//! - **Security First**: All operations require security context and validation
//! - **Comprehensive Logging**: Full audit trail for security and debugging
//! - **Cross-Platform**: Abstractions hide platform-specific implementation details
//! - **Type Safety**: Strong typing prevents common security and reliability issues
//! - **Composability**: Middleware and contexts enable flexible operation composition

pub mod context;
pub mod executor;
pub mod middleware;
pub mod operation;
pub mod result;
pub mod security;
