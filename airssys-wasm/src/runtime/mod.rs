//! # Runtime Module
//!
//! WASM component execution using wasmtime Component Model.
//!
//! ## Layer 2B - Runtime Layer
//!
//! Import Rules (ADR-WASM-023):
//! - ✅ Can import: `core/`, `security/`
//! - ❌ Cannot import: `component/`, `messaging/`, `system/`
//!
//! ## Submodules
//!
//! - [`engine`] - WasmtimeEngine (RuntimeEngine implementation)
//! - [`loader`] - ComponentLoader implementations (FileComponentLoader, InMemoryComponentLoader)
//! - [`store`] - StoreManager for WASM stores
//! - [`limiter`] - ResourceLimiter for memory and fuel constraints

pub mod engine;
pub mod limiter;
pub mod loader;
pub mod store;

pub mod host_functions;
