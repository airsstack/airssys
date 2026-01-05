//! # airssys-wasm - WASM Plugin/Extension Platform for AirsStack
//!
//! This crate provides a WebAssembly-based plugin/extension platform where:
//! - Each WASM component is treated as an Actor (via airssys-rt)
//! - Components have isolated storage (like smart contracts on NEAR/Polkadot)
//! - Communication happens via Erlang-style mailbox messaging
//! - Security follows deny-by-default capability model
//!
//! ## Architecture
//!
//! The crate is organized into four root modules:
//!
//! - **core/** - Foundation: shared types and abstractions (imports nothing internal)
//! - **security/** - Security: capabilities, policies, validation (imports core/)
//! - **runtime/** - WASM Execution: Wasmtime integration (imports core/, security/)
//! - **actor/** - Integration: actor system, messaging, lifecycle (imports core/, security/, runtime/)
//!
//! ## Dependency Rules (ADR-WASM-023 - MANDATORY)
//!
//! ```
//! actor/ ──► runtime/ ──► security/ ──► core/
//!    │            │              │               │
//!    └────────────┴──────────────┴───────────────┘
//!                      All can import from core/
//! ```
//!
//! **ALLOWED:**
//! - ✅ actor/    → runtime/
//! - ✅ actor/    → security/
//! - ✅ actor/    → core/
//! - ✅ runtime/  → security/
//! - ✅ runtime/  → core/
//! - ✅ security/ → core/
//!
//! **FORBIDDEN (never):**
//! - ❌ runtime/  → actor/
//! - ❌ security/ → actor/
//! - ❌ security/ → runtime/
//! - ❌ core/     → ANY MODULE
//!
//! ## Getting Started
//!
//! ```rust,no_run
//! use airssys_wasm::prelude::*;
//! use airssys_wasm::actor::ComponentActor;
//!
//! // Components will be added in subsequent tasks
//! ```
//!
//! ## Module Documentation
//!
//! - [core](core) - Core types and abstractions
//! - [security](security) - Security capabilities and policies
//! - [runtime](runtime) - WASM execution engine
//! - [actor](actor) - Actor system integration

// Layer 1: Standard library imports (per PROJECTS_STANDARD.md §2.1)

// Layer 2: Third-party crate imports (per PROJECTS_STANDARD.md §2.1)

// Layer 3: Internal module imports (per PROJECTS_STANDARD.md §2.1)

// Foundation layer (no internal dependencies)
pub mod core;

// Security layer (imports from core/)
pub mod security;

// WASM execution layer (imports from core/, security/)
pub mod runtime;

// Actor integration layer (imports from core/, security/, runtime/)
pub mod actor;

// Prelude - common re-exports for ergonomic API (per ADR-WASM-011)
pub mod prelude;

// Re-export core error types at crate root (will be populated in subsequent tasks)
