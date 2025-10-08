//! Procedural macros for airssys-osl core abstractions.
//!
//! This crate provides ergonomic macros to reduce boilerplate when
//! implementing airssys-osl traits.
//!
//! # Available Macros
//!
//! - `#[executor]`: Generate `OSExecutor<O>` trait implementations
//!
//! # Example
//!
//! ```rust,ignore
//! use airssys_osl::prelude::*;
//!
//! #[executor]
//! impl MyExecutor {
//!     async fn file_read(&self, op: FileReadOperation, ctx: &ExecutionContext) 
//!         -> OSResult<ExecutionResult> 
//!     {
//!         // Custom implementation
//!         todo!()
//!     }
//! }
//! ```

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
