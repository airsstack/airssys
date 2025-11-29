//! AirsSys WASM Component Framework
//!
//! A WebAssembly component framework for building pluggable systems with
//! capability-based security, runtime deployment, and actor-based hosting.
//!
//! # Overview
//!
//! The airssys-wasm framework provides core abstractions for building secure,
//! composable WebAssembly components that integrate with the AirsSys ecosystem.
//! It implements the WebAssembly Component Model with capability-based security
//! and runtime lifecycle management.
//!
//! # Architecture
//!
//! The framework is organized into two primary modules:
//!
//! - **[`core`]** - Foundational abstractions, types, and trait contracts
//! - **[`prelude`]** - Convenient re-exports for common use cases
//!
//! # Quick Start
//!
//! Import commonly used types via the prelude:
//!
//! ```rust
//! use airssys_wasm::prelude::*;
//!
//! // Create component with capabilities
//! let id = ComponentId::new("my-component");
//! let mut caps = CapabilitySet::new();
//! caps.grant(Capability::FileRead(PathPattern::new("/data/config.json")));
//! ```
//!
//! For selective imports, use the core module directly:
//!
//! ```rust
//! use airssys_wasm::core::{Component, ComponentId, WasmResult};
//! ```
//!
//! # Core Abstractions
//!
//! The framework provides abstractions for:
//!
//! - **Component Model**: Component identification, metadata, state management
//! - **Capability Security**: Fine-grained capability grants for filesystem, network, and environment access
//! - **Runtime Engine**: Execution context and resource management
//! - **Lifecycle Management**: Component installation, updates, and removal
//! - **Actor Integration**: Message-based communication with actor systems
//! - **Storage Backend**: Persistence layer for component state
//! - **Observability**: Metrics collection and health monitoring
//!
//! # References
//!
//! - **Technical Context**: `.copilot/memory_bank/sub_projects/airssys-wasm/tech_context.md`
//! - **ADR-WASM-012**: Comprehensive Core Abstractions Strategy
//! - **WebAssembly Component Model**: <https://component-model.bytecodealliance.org/>

// Core abstractions (foundation)
pub mod core;

// Runtime implementation layer (Block 1 - WASM-TASK-002)
pub mod runtime;

// Generated WIT bindings (build.rs output)
// Note: This module contains auto-generated code from wit-bindgen
// DO NOT EDIT - Regenerated on every build when WIT files change
#[allow(warnings, clippy::all)]
pub mod wit_bindings {
    include!("generated/airssys_component.rs");
}

// Prelude for ergonomic imports
pub mod prelude;
