//! Procedural macros for airssys-osl core abstractions.
//!
//! This crate provides ergonomic macros to reduce boilerplate when
//! implementing airssys-osl traits.
//!
//! # Available Macros
//!
//! - `#[executor]`: Generate `OSExecutor<O>` trait implementations
//!
//! # The `#[executor]` Macro
//!
//! The `#[executor]` macro automatically generates `OSExecutor<O>` trait implementations
//! for each operation method in your impl block. This eliminates ~85% of boilerplate code.
//!
//! ## Basic Usage
//!
//! ```rust,ignore
//! use airssys_osl::prelude::*;
//! use airssys_osl_macros::executor;
//!
//! struct MyExecutor;
//!
//! #[executor]
//! impl MyExecutor {
//!     async fn file_read(
//!         &self,
//!         operation: FileReadOperation,
//!         context: &ExecutionContext
//!     ) -> OSResult<ExecutionResult> {
//!         // Your custom implementation
//!         todo!()
//!     }
//! }
//!
//! // The macro generates:
//! // #[async_trait::async_trait]
//! // impl OSExecutor<FileReadOperation> for MyExecutor {
//! //     async fn execute(&self, operation: FileReadOperation, context: &ExecutionContext)
//! //         -> OSResult<ExecutionResult> {
//! //         self.file_read(operation, context).await
//! //     }
//! // }
//! ```
//!
//! ## Multiple Operations
//!
//! You can implement multiple operations in a single executor:
//!
//! ```rust,ignore
//! #[executor]
//! impl MyExecutor {
//!     async fn file_read(&self, operation: FileReadOperation, context: &ExecutionContext)
//!         -> OSResult<ExecutionResult> { todo!() }
//!     
//!     async fn file_write(&self, operation: FileWriteOperation, context: &ExecutionContext)
//!         -> OSResult<ExecutionResult> { todo!() }
//!     
//!     async fn process_spawn(&self, operation: ProcessSpawnOperation, context: &ExecutionContext)
//!         -> OSResult<ExecutionResult> { todo!() }
//! }
//! // Generates 3 separate OSExecutor trait implementations
//! ```
//!
//! ## Helper Methods
//!
//! Non-operation methods are preserved and ignored by the macro:
//!
//! ```rust,ignore
//! #[executor]
//! impl MyExecutor {
//!     async fn file_read(&self, operation: FileReadOperation, context: &ExecutionContext)
//!         -> OSResult<ExecutionResult> {
//!         self.validate_path(&operation.path)?;
//!         todo!()
//!     }
//!     
//!     // Helper method - ignored by macro
//!     fn validate_path(&self, path: &str) -> OSResult<()> {
//!         Ok(())
//!     }
//! }
//! ```
//!
//! ## Supported Operations
//!
//! The macro recognizes these 11 operation methods:
//!
//! ### Filesystem (5 operations)
//! - `file_read` → `FileReadOperation`
//! - `file_write` → `FileWriteOperation`
//! - `file_delete` → `FileDeleteOperation`
//! - `directory_create` → `DirectoryCreateOperation`
//! - `directory_list` → `DirectoryListOperation`
//!
//! ### Process (3 operations)
//! - `process_spawn` → `ProcessSpawnOperation`
//! - `process_kill` → `ProcessKillOperation`
//! - `process_signal` → `ProcessSignalOperation`
//!
//! ### Network (3 operations)
//! - `network_connect` → `NetworkConnectOperation`
//! - `network_listen` → `NetworkListenOperation`
//! - `network_socket` → `NetworkSocketOperation`
//!
//! ## Method Signature Requirements
//!
//! Each operation method must follow this exact signature:
//!
//! ```rust,ignore
//! async fn operation_name(
//!     &self,                      // Must be &self (not &mut self or self)
//!     operation: OperationType,   // Parameter must be named "operation"
//!     context: &ExecutionContext  // Parameter must be named "context"
//! ) -> OSResult<ExecutionResult>  // Return type must be OSResult<ExecutionResult>
//! ```
//!
//! ## Error Messages
//!
//! The macro provides helpful error messages for common mistakes:
//!
//! - **Not async**: "Method 'file_read' must be async"
//! - **Wrong receiver**: "Method 'file_read' must take &self (not &mut self or self)"
//! - **Wrong parameter names**: "First parameter must be named 'operation', found 'op'"
//! - **Duplicate methods**: "Duplicate operation method 'file_read'. Each operation can only be implemented once"
//!
//! ## Integration with airssys-osl
//!
//! This macro is designed to work with the `airssys-osl` framework. For complete
//! examples and integration patterns, see the `airssys-osl` documentation.


// Layer 1: Standard library imports
use proc_macro::TokenStream;

// Layer 2: Third-party imports (none yet)

// Layer 3: Internal imports
mod executor;
mod utils;

/// Generates `OSExecutor<O>` trait implementations from method names.
///
/// This attribute macro reduces boilerplate by automatically generating trait
/// implementations from method names. See crate documentation for details.
#[proc_macro_attribute]
pub fn executor(_attr: TokenStream, item: TokenStream) -> TokenStream {
    executor::expand(item.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
