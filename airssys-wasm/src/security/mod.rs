//! WASM Component Security & Isolation Layer
//!
//! This module provides WASM-specific security by bridging to airssys-osl
//! security infrastructure (ACL/RBAC/audit logging).
//!
//! # Overview
//!
//! The security layer implements a **deny-by-default** capability-based security model
//! where WASM components must explicitly declare required host resource access in their
//! `Component.toml` manifest. These declarations are validated and mapped to airssys-osl
//! ACL/RBAC policies for enforcement at host function boundaries.
//!
//! # Architecture
//!
//! ```text
//! ┌────────────────────────────────────────────────────────────────┐
//! │ Layer 5: WASM Component (Untrusted Code)                       │
//! │ - Runs in isolated WASM linear memory (512KB-4MB)              │
//! │ - No direct host access (sandboxed)                            │
//! └────────────────┬───────────────────────────────────────────────┘
//!                  │ Host Function Call
//!                  ▼
//! ┌────────────────────────────────────────────────────────────────┐
//! │ Layer 4: Capability Check (check_capability() - Task 3.1)      │
//! │ - Validates component ID + resource + permission               │
//! │ - Maps to airssys-osl SecurityContext                          │
//! └────────────────┬───────────────────────────────────────────────┘
//!                  │ Security Evaluation
//!                  ▼
//! ┌────────────────────────────────────────────────────────────────┐
//! │ Layer 3: airssys-osl Security (ACL/RBAC/Audit)                 │
//! │ - Pattern matching (glob, wildcards)                           │
//! │ - Policy evaluation (deny-by-default)                          │
//! │ - Audit logging (all access attempts)                          │
//! └────────────────┬───────────────────────────────────────────────┘
//!                  │ Decision: Allow/Deny
//!                  ▼
//! ┌────────────────────────────────────────────────────────────────┐
//! │ Layer 2: Host Function Implementation (Block 8)                │
//! │ - Filesystem operations (read/write/stat)                      │
//! │ - Network operations (connect/bind/listen)                     │
//! │ - Storage operations (get/set/delete)                          │
//! └────────────────┬───────────────────────────────────────────────┘
//!                  │ System Calls
//!                  ▼
//! ┌────────────────────────────────────────────────────────────────┐
//! │ Layer 1: Operating System (Trusted)                            │
//! │ - Filesystem, network, process management                      │
//! └────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Module Structure
//!
//! This module is organized following PROJECTS_STANDARD.md §4.3:
//! - `mod.rs` contains ONLY module declarations and re-exports (this file)
//! - Implementation lives in separate files (`capability.rs`, etc.)
//!
//! ## Submodules
//!
//! - [`capability`]: WASM capability types and airssys-osl ACL/RBAC bridge
//!
//! # Security Model
//!
//! ## Deny-by-Default
//!
//! Components without declared capabilities are **denied all access** to host resources.
//! This prevents accidental privilege escalation and limits blast radius of vulnerabilities.
//!
//! ## Least Privilege
//!
//! Components should declare **minimal capabilities** required for functionality:
//! - ✅ Good: Request `/app/data/*.json` (specific files)
//! - ❌ Bad: Request `/app/**` (overly broad, entire app directory tree)
//!
//! ## Explicit Declaration
//!
//! All capabilities must be declared in `Component.toml` before component spawn:
//!
//! ```toml
//! [capabilities]
//! filesystem.read = ["/app/config/*", "/app/data/*.json"]
//! network.connect = ["api.example.com:443"]
//! storage.namespace = ["component:<id>:*"]
//! ```
//!
//! ## Capability Immutability
//!
//! Once a component is spawned, its capability set is **immutable**. Components
//! cannot request additional privileges at runtime, preventing privilege escalation.
//!
//! # Integration with airssys-osl
//!
//! Instead of reimplementing security infrastructure, this module **bridges** to
//! airssys-osl's production-ready ACL/RBAC/audit system:
//!
//! ## What We Reuse
//!
//! - ✅ **ACL (Access Control Lists)**: Glob pattern matching for resources
//! - ✅ **RBAC (Role-Based Access Control)**: Permission-based authorization
//! - ✅ **Audit Logging**: Comprehensive security event logging
//! - ✅ **SecurityPolicy**: Pluggable policy evaluation engine
//! - ✅ **Pattern Matching**: Glob patterns (`*`, `**`, `?`, `[...]`)
//!
//! ## What We Add
//!
//! - 🆕 **WASM Capability Types**: WASM-specific capability declarations
//! - 🆕 **Component.toml Parser**: Parse capabilities from manifest (Task 1.2)
//! - 🆕 **Trust-Level System**: Trusted/unknown/dev source workflows (Phase 2)
//! - 🆕 **WasmSecurityContext**: Bridge to airssys-osl SecurityContext (Task 1.3)
//!
//! # Task Roadmap
//!
//! ## Phase 1: WASM-OSL Security Bridge (Week 1)
//!
//! - **Task 1.1**: ✅ COMPLETE - Capability types & OSL mapping
//! - **Task 1.2**: ⏳ IN PROGRESS - Component.toml parser
//! - **Task 1.3**: Pending - SecurityContext bridge
//!
//! ## Phase 2: Trust-Level System (Week 2)
//!
//! - **Task 2.1**: Pending - Trust level implementation
//! - **Task 2.2**: Pending - Approval workflow engine
//! - **Task 2.3**: Pending - Trust configuration system
//!
//! ## Phase 3: Capability Enforcement (Week 2-3)
//!
//! - **Task 3.1**: Pending - Capability check API
//! - **Task 3.2**: Pending - Host function integration
//! - **Task 3.3**: Pending - Audit logging integration
//!
//! ## Phase 4: ComponentActor Integration (Week 3)
//!
//! - **Task 4.1**: Pending - Security context attachment
//! - **Task 4.2**: ✅ COMPLETE - Message passing security
//! - **Task 4.3**: Pending - Resource quota system
//!
//! ## Phase 5: Testing & Documentation (Week 4)
//!
//! - **Task 5.1**: Pending - Security integration testing
//! - **Task 5.2**: Pending - Security documentation
//! - **Task 5.3**: Pending - Production readiness checklist
//!
//! # Examples
//!
//! See [`capability`] module for detailed examples of:
//! - Declaring capabilities in `Component.toml`
//! - Building `WasmCapabilitySet` programmatically
//! - Converting capabilities to ACL entries
//! - Creating `WasmSecurityContext` for components
//!
//! # Performance Targets
//!
//! - Capability check: <5μs (includes ACL evaluation)
//! - ACL conversion: ~1μs for 10 capabilities
//! - Audit logging: <100ns per event
//! - Total overhead: <10μs per host function call
//!
//! # Standards Compliance
//!
//! - **ADR-WASM-005**: Capability-Based Security Model ✅
//! - **PROJECTS_STANDARD.md**: §4.3 (module structure), §5.1 (dependencies) ✅
//! - **Microsoft Rust Guidelines**: M-MODULE-DOCS, M-CANONICAL-DOCS ✅

// Module declarations (§4.3 - ONLY declarations in mod.rs)
pub mod capability;
pub mod parser;
pub mod trust;

// Re-export primary types for ergonomic imports
pub use capability::{WasmCapability, WasmCapabilitySet, WasmSecurityContext};
pub use parser::{ComponentManifestParser, ParseError, ParseResult};
pub use trust::{ComponentSource, TrustError, TrustLevel, TrustRegistry, TrustResult, TrustSource};
