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

pub mod engine;
pub mod loader;

// Future submodules:
// pub mod store;
// pub mod host_fn;
// pub mod limiter;
