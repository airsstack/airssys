//! Core abstractions for airssys-wasm framework.
//!
//! This module contains foundational types, traits, and error definitions
//! used throughout the entire airssys-wasm crate. It has **ZERO internal
//! dependencies** within airssys-wasm to prevent circular dependencies.
//!
//! # Architecture
//!
//! The core module follows a two-tier structure:
//!
//! ## Universal Abstractions
//! - `component` - Component types, metadata, input/output
//! - `capability` - Capability-based security primitives
//! - `error` - Error types and result aliases
//! - `config` - Configuration types and defaults
//!
//! ## Domain-Specific Abstractions (Future Phases)
//! - `runtime` - Runtime engine traits and execution context
//! - `interface` - WIT interface metadata and type descriptors
//! - `actor` - Actor integration message envelopes
//! - `security` - Security policy traits and permission types
//! - `messaging` - Inter-component messaging protocols
//! - `storage` - Storage backend traits and operations
//! - `lifecycle` - Lifecycle state machines and transitions
//! - `management` - Component registry and management abstractions
//! - `bridge` - OSL bridge traits and capability mapping
//! - `observability` - Metrics collection and monitoring traits
//!
//! # Design Principles
//!
//! 1. **Zero Internal Dependencies** - Core depends ONLY on external crates
//! 2. **Minimalism (YAGNI)** - Include only types needed by 3+ modules
//! 3. **Type Safety** - Newtype pattern for IDs, enums for variants
//! 4. **Stability First** - Core types rarely change (breaking = major version)
//! 5. **Trait-Centric** - Behavior contracts via traits for testability
//!
//! # References
//!
//! - **ADR-WASM-011**: Module Structure Organization
//! - **ADR-WASM-012**: Comprehensive Core Abstractions Strategy
//! - **Workspace Standards**: §4.3 (Module Architecture), §6.1 (YAGNI)

// Universal Abstractions (Phase 1-5)
pub mod capability;
pub mod component;
pub mod error;

// Future phases (will be uncommented as implemented)
// Phase 5: pub mod config;
// Phase 6-10: Domain-specific abstractions
