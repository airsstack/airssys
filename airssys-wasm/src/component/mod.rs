//! # Component Module
//!
//! Component system integration for WASM components.
//!
//! This module provides actor integration for WASM components, including:
//! - `ComponentWrapper` - Wraps WASM components as airssys-rt Actors
//! - `ComponentActorMessage` - Message type for component actors
//!
//! # Architecture
//!
//! This is Layer 3A in the six-module architecture. It:
//! - Uses types from `core/component/` (ComponentId, ComponentHandle, etc.)
//! - Uses traits from `core/runtime/` (RuntimeEngine)
//! - Integrates with `airssys-rt` (Actor, Message traits)
//! - Receives concrete implementations from `system/` (Layer 4)
//!
//! # Module Boundary Rules
//!
//! - CAN import: `core/`, `airssys-rt`
//! - CANNOT import: `runtime/`, `security/`, `system/`
//!
//! # References
//!
//! - ADR-WASM-031: Component & Messaging Module Design
//! - KNOWLEDGE-WASM-038: Component Module Responsibility

// Module declarations (per PROJECTS_STANDARD.md §4.3)
pub mod wrapper;

// NOTE: No type re-exports per module grouping policy.
// Callers use: crate::component::wrapper::ComponentWrapper
