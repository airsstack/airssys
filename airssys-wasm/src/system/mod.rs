//! # System Module (Layer 4)
//!
//! Top-level system coordination and lifecycle management.
//!
//! ## Responsibilities
//!
//! - [`SystemCoordinator`]: Composition root that wires all dependencies together
//! - [`SystemBuilder`]: Constructs SystemCoordinator with dependency injection (Phase 7)
//!
//! ## Module Position
//!
//! This is **Layer 4** of the 6-layer architecture:
//!
//! ```text
//! Layer 4: system/          <- THIS MODULE (TOP OF DEPENDENCY CHAIN)
//!   imports
//! Layer 3B: messaging/
//! Layer 3A: component/
//! Layer 2B: runtime/
//! Layer 2A: security/
//! Layer 0: core/
//! ```
//!
//! ## Architecture References
//!
//! - ADR-WASM-032: System Module Design
//! - ADR-WASM-023: Module Boundary Enforcement
//! - KNOWLEDGE-WASM-037: Rebuild Architecture - Clean Slate Design

pub mod builder; // SystemBuilder (WASM-TASK-049)
pub mod coordinator; // SystemCoordinator
pub mod lifecycle; // LifecycleManager (WASM-TASK-048)
