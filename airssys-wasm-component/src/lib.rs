//! # AirsSys WASM Component Macros
//!
//! Procedural macros for building AirsSys WASM components with ease.
//! This crate provides macro helpers that eliminate the need to write `extern "C"`
//! functions manually, inspired by CosmWasm's approach.
//!
//! ## Features
//!
//! - `#[component]` - Main component macro that generates WASM exports
//! - `#[derive(ComponentOperation)]` - Derive macro for operation types
//! - `#[derive(ComponentResult)]` - Derive macro for result types  
//! - `#[derive(ComponentConfig)]` - Derive macro for configuration types
//!
//! ## Example
//!
//! ```rust,ignore
//! use airssys_wasm::Component;
//! use airssys_wasm_component::{component, ComponentOperation, ComponentResult, ComponentConfig};
//!
//! #[derive(ComponentConfig)]
//! pub struct MyConfig {
//!     pub setting: String,
//! }
//!
//! #[derive(ComponentOperation)]
//! pub enum MyOperation {
//!     Process { data: String },
//!     GetStatus,
//! }
//!
//! #[derive(ComponentResult)]
//! pub enum MyResult {
//!     Processed { result: String },
//!     Status { active: bool },
//! }
//!
//! #[component(name = "my-processor", version = "1.0.0")]
//! pub struct MyProcessor {
//!     state: ProcessorState,
//! }
//!
//! impl Component for MyProcessor {
//!     type Config = MyConfig;
//!     type Operation = MyOperation;
//!     type Result = MyResult;
//!     
//!     fn init(&mut self, config: Self::Config) -> Result<(), ComponentError> {
//!         // Initialization logic
//!         Ok(())
//!     }
//!     
//!     fn execute(&mut self, operation: Self::Operation) -> Result<Self::Result, ComponentError> {
//!         // Business logic
//!         match operation {
//!             MyOperation::Process { data } => {
//!                 let result = format!("Processed: {}", data);
//!                 Ok(MyResult::Processed { result })
//!             }
//!             MyOperation::GetStatus => {
//!                 Ok(MyResult::Status { active: true })
//!             }
//!         }
//!     }
//! }
//! ```

use proc_macro::TokenStream;

mod codegen;
mod component;
mod derive;
mod utils;

/// Main component macro that generates WASM exports and eliminates extern "C" complexity
///
/// This macro transforms a clean component implementation into a WASM-compatible module
/// with all necessary exports and boilerplate code.
///
/// # Arguments
///
/// - `name` - Component name (required)
/// - `version` - Component version (optional, defaults to "0.1.0")
/// - `capabilities` - List of required capabilities (optional)
///
/// # Example
///
/// ```rust,ignore
/// #[component(name = "data-processor", version = "1.0.0")]
/// pub struct DataProcessor {
///     processed_count: usize,
/// }
/// ```
#[proc_macro_attribute]
pub fn component(args: TokenStream, input: TokenStream) -> TokenStream {
    component::expand_component(args, input)
}

/// Derive macro for component operation types
///
/// Automatically implements the `ComponentOperation` trait and multicodec serialization.
///
/// # Example
///
/// ```rust,ignore
/// #[derive(ComponentOperation)]
/// pub enum MyOperation {
///     Process { data: String },
///     GetStatus,
/// }
/// ```
#[proc_macro_derive(ComponentOperation)]
pub fn derive_component_operation(input: TokenStream) -> TokenStream {
    derive::expand_component_operation(input)
}

/// Derive macro for component result types
///
/// Automatically implements the `ComponentResult` trait and multicodec serialization.
///
/// # Example
///
/// ```rust,ignore
/// #[derive(ComponentResult)]
/// pub enum MyResult {
///     Processed { result: String },
///     Status { active: bool },
/// }
/// ```
#[proc_macro_derive(ComponentResult)]
pub fn derive_component_result(input: TokenStream) -> TokenStream {
    derive::expand_component_result(input)
}

/// Derive macro for component configuration types
///
/// Automatically implements the `ComponentConfig` trait and multicodec serialization.
///
/// # Example
///
/// ```rust,ignore
/// #[derive(ComponentConfig)]
/// pub struct MyConfig {
///     pub timeout_ms: u64,
///     pub max_items: usize,
/// }
/// ```
#[proc_macro_derive(ComponentConfig)]
pub fn derive_component_config(input: TokenStream) -> TokenStream {
    derive::expand_component_config(input)
}
