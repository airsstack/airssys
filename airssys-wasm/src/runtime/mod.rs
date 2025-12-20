//! Runtime implementation layer for WebAssembly component execution.
//!
//! This module provides concrete implementations of the runtime abstractions
//! defined in `core::runtime`, using Wasmtime as the underlying execution engine.
//!
//! # Architecture
//!
//! The runtime layer implements Block 1 (WASM-TASK-002) of the airssys-wasm
//! framework, providing:
//!
//! - **WasmEngine**: Wasmtime-based runtime engine with Component Model support
//! - **ComponentLoader**: Component loading and validation from files and bytes
//! - **Error Translation**: Wasmtime error mapping to `WasmError` types
//!
//! # Design Rationale (ADR-WASM-002)
//!
//! - **Wasmtime v24.0**: Production-ready runtime with Component Model support
//! - **Async-first**: Tokio integration for non-blocking execution
//! - **Fuel Metering**: CPU limiting via hybrid fuel + wall-clock timeout
//! - **Memory Safety**: Mandatory memory limits in Component.toml
//! - **Crash Isolation**: Component failures don't crash host
//!
//! # Module Organization (§4.3 Workspace Standards)
//!
//! Following workspace standards, this `mod.rs` contains ONLY module declarations
//! and re-exports. Implementation code resides in:
//!
//! - `engine.rs` - WasmEngine implementation
//! - `loader.rs` - ComponentLoader implementation
//!
//! # Example
//!
//! ```rust,ignore
//! use airssys_wasm::runtime::WasmEngine;
//! use airssys_wasm::core::{ComponentId, ExecutionContext, ComponentInput, ResourceLimits, CapabilitySet};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create engine with default configuration
//!     let engine = WasmEngine::new()?;
//!     
//!     // Load component
//!     let component_bytes = std::fs::read("component.wasm")?;
//!     let component_id = ComponentId::new("my-component");
//!     let handle = engine.load_component(&component_id, &component_bytes).await?;
//!     
//!     // Execute function
//!     let context = ExecutionContext {
//!         component_id: component_id.clone(),
//!         limits: ResourceLimits::default(),
//!         capabilities: CapabilitySet::new(),
//!         timeout_ms: 5000,
//!     };
//!     let input = ComponentInput::new(b"input data");
//!     let output = engine.execute(&handle, "process", input, context).await?;
//!     
//!     println!("Output: {} bytes", output.data().len());
//!     Ok(())
//! }
//! ```
//!
//! # References
//!
//! - **ADR-WASM-002**: WASM Runtime Engine Selection (PRIMARY REFERENCE)
//! - **ADR-WASM-010**: Implementation Strategy (Block 1 foundation)
//! - **KNOWLEDGE-WASM-012**: Phase 1 Implementation Findings
//! - **Workspace Standards**: §2.1 (imports), §4.3 (mod.rs pattern)

// Module declarations (§4.3 - mod.rs declaration-only pattern)
pub mod async_host;
pub mod engine;
pub mod limits;
pub mod loader;
pub mod messaging;
pub mod store_manager;

// Public re-exports for ergonomic imports
pub use async_host::{
    create_host_context, AsyncFileReadFunction, AsyncHostRegistry, AsyncHttpFetchFunction,
    AsyncSleepFunction,
};
pub use engine::WasmEngine;
pub use limits::{
    ComponentResourceLimiter, MemoryConfig, MemoryMetrics, ResourceLimits, ResourceLimitsBuilder,
};
pub use loader::ComponentLoader;
pub use messaging::{MessagingService, MessagingStats};
pub use store_manager::StoreWrapper;
