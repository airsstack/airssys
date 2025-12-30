# [WASM-TASK-013] - Block 1: Host System Architecture Implementation

**Task ID:** WASM-TASK-013
**Created:** 2025-12-29
**Status:** 📋 PLANNING
**Priority:** 🔴 CRITICAL FOUNDATION
**Layer:** 0 - Foundation Layer
**Block:** ALL Block 5-11 development (006, 007, 008, 009, 010, 011+)
**Estimated Effort:** 4-6 weeks

---

## Executive Summary

Implement `host_system/` module as a central coordinator for airssys-wasm framework. This module:
- Eliminates circular dependencies between `actor/`, `messaging/`, and `runtime/`
- Provides clear ownership of system initialization and orchestration
- Establishes one-way dependency chain
- Enables future development of Block 5, 6, 7, 8, 9, 10, 11+

This is a **FOUNDATIONAL REFACTOR** - creates infrastructure that all future modules depend on.

## Context

### Problem Statement

After architectural analysis (KNOWLEDGE-WASM-036), discovered that current three-module architecture has critical architectural flaws:

1. **Circular Dependency:**
   ```
   runtime/ ───► messaging/ ───► actor/ ───► runtime/
      ↑                                              │
      └────────────────── CIRCULAR! ───────────────────┘
   ```
   - `runtime/async_host.rs`: `use crate::messaging::MessagingService;`
   - `messaging/router.rs`: `use crate::actor::message::CorrelationTracker;`
   - `actor/component_actor.rs`: `use crate::runtime::WasmEngine;`

2. **Unclear Orchestration Ownership:** No single module was responsible for:
   - System initialization order
   - Component lifecycle coordination
   - Message flow orchestration
   - Startup/shutdown procedures

3. **Overlapping Responsibilities:** Messaging-related logic was split between `messaging/` and `actor/`, creating confusion about where features belonged.

4. **Violated Module Boundary Rules (ADR-WASM-023):** Circular dependencies violated the principle of one-way dependencies between modules.

### Solution Overview

Introduce `host_system/` as a top-level module that:
- Coordinates all infrastructure initialization
- Manages component lifecycle
- Orchestrates message flow
- Provides single entry point for host applications
- Establishes one-way dependency chain

### Critical Decision

**Reference:** KNOWLEDGE-WASM-036 (Four-Module Architecture)

This knowledge defines the correct four-module architecture:
```
host_system/ ───► actor/
host_system/ ───► messaging/
host_system/ ───► runtime/
actor/ ───► runtime/
messaging/ ───► runtime/
runtime/ ───► core/
core/ ───► (nothing - foundation)
```

## Scope

### In Scope

1. **Create host_system/ module structure**
   - Create `src/host_system/mod.rs` - Module declarations
   - Create `src/host_system/manager.rs` - Main manager implementation
   - Create `src/host_system/initialization.rs` - System initialization logic
   - Create `src/host_system/lifecycle.rs` - Component lifecycle management
   - Create `src/host_system/messaging.rs` - Message flow coordination
   - Update `src/lib.rs` - Add `pub mod host_system;`

2. **Move CorrelationTracker**
   - From: `src/actor/message/correlation_tracker.rs`
   - To: `src/host_system/correlation_tracker.rs`
   - Update imports in `host_system/mod.rs`
   - Remove from `actor/message/mod.rs`
   - Update tests

3. **Move TimeoutHandler**
   - From: `src/messaging/timeout_handler.rs` (if exists)
   - To: `src/host_system/timeout_handler.rs`
   - Update imports
   - Remove from messaging/
   - Update tests

4. **Implement HostSystemManager**
   - Implement initialization logic
   - Implement component lifecycle methods
   - Implement dependency wiring
   - Add comprehensive error handling
   - Add logging and observability

5. **Refactor ActorSystemSubscriber**
   - Update to accept dependencies from host_system/
   - Remove internal references to actor/messaging
   - Update messaging/ imports
   - Add integration tests

6. **Refactor Runtime Host Functions**
   - Update `runtime/async_host.rs`
   - Remove imports from messaging/
   - Pass dependencies via constructor
   - Update tests

7. **Testing**
   - Unit tests for HostSystemManager
   - Integration tests for system initialization
   - Tests for dependency flow (verify one-way only)

### Out of Scope

- ❌ MessageBroker refactoring (Task 006 scope)
- ❌ ComponentRegistry changes (separate task)
- ❌ Storage system implementation (Task 007 scope)
- ❌ Lifecycle system implementation (Task 008 scope)
- ❌ AirsSys-OSL Bridge (Task 009 scope)
- ❌ Monitoring system implementation (Task 010 scope)
- ❌ Component SDK implementation (Task 011 scope)
- ❌ CLI implementation (Task 012 scope)

## Implementation Plan

### Phase 1: Module Structure & Basic Types (Week 1)

**Deliverables:**
- Create `src/host_system/` directory
- Create `src/host_system/mod.rs` with module declarations
- Create empty `HostSystemManager` struct
- Update `src/lib.rs` to include `host_system` module
- Add basic tests

**Success Criteria:**
- ✅ `cargo build` succeeds
- ✅ `host_system` module visible
- ✅ Basic tests pass

### Phase 2: Move CorrelationTracker (Week 1-2)

**Deliverables:**
- Move `correlation_tracker.rs` to `src/host_system/correlation_tracker.rs`
- Update imports in:
  - `host_system/mod.rs`
  - All files that use CorrelationTracker
- Remove from `actor/message/mod.rs`
- Update tests

**Success Criteria:**
- ✅ All imports updated
- ✅ `cargo test --lib` passes
- ✅ No import violations

### Phase 3: Move TimeoutHandler (Week 2)

**Deliverables:**
- Move `timeout_handler.rs` to `src/host_system/timeout_handler.rs`
- Update imports
- Remove from messaging/ (if exists)
- Update tests

**Success Criteria:**
- ✅ All imports updated
- ✅ `cargo test --lib` passes
- ✅ No import violations

### Phase 4: Implement HostSystemManager (Week 2-3)

**Deliverables:**
- Implement initialization logic
- Implement component lifecycle methods
- Implement dependency wiring
- Add comprehensive error handling
- Add logging and observability

**Success Criteria:**
- ✅ Manager compiles
- ✅ System can be initialized
- ✅ Components can be spawned via HostSystemManager
- ✅ Components can be stopped via HostSystemManager
- ✅ No circular dependencies (all grep checks pass)
- ✅ All unit tests pass (`cargo test --lib`)

### Phase 5: Refactor ActorSystemSubscriber (Week 3-4)

**Deliverables:**
- Update to accept host_system dependencies
- Remove actor/messaging internal references
- Update messaging/ to use host_system types
- Add integration tests

**Success Criteria:**
- ✅ Subscriber accepts dependencies
- ✅ No circular imports
- ✅ Integration tests pass

### Phase 6: Refactor Runtime Host Functions (Week 4-5)

**Deliverables:**
- Update `runtime/async_host.rs`
- Remove imports from messaging/
- Pass dependencies via constructor
- Update tests

**Success Criteria:**
- ✅ Host functions use injected dependencies
- ✅ No forbidden imports
- ✅ All tests pass

### Phase 7: Update Knowledge & Close HOTFIX-001 (Week 5-6)

**Deliverables:**
- Update KNOWLEDGE-WASM-036 with implementation lessons
- Mark HOTFIX-001 as SUPERSEDED
- Update task index
- Create completion retrospective

**Success Criteria:**
- ✅ Documentation updated
- ✅ HOTFIX-001 closed
- ✅ Task index updated
- ✅ All verification commands pass

## Verification

### Architecture Verification

```bash
# MUST return nothing for valid architecture
grep -r "use crate::actor" src/messaging/
grep -r "use crate::messaging" src/runtime/
grep -r "use crate::actor" src/runtime/
grep -r "use crate::host_system" src/core/
```

### Functional Verification

```bash
# System initialization works
cargo test --test host_system_initialization

# Component lifecycle works
cargo test --test host_system_lifecycle

# Dependency flow verified
cargo clippy --all-targets -- -D warnings
```

## Success Criteria

### Must Have (Mandatory)

- ✅ host_system/ module exists and is visible
- ✅ HostSystemManager can be instantiated
- ✅ System initializes without errors
- ✅ Components can be spawned via HostSystemManager
- ✅ Components can be stopped via HostSystemManager
- ✅ No circular dependencies (all grep checks pass)
- ✅ All unit tests pass (`cargo test --lib`)
- ✅ All integration tests pass
- ✅ Zero clippy warnings
- ✅ Documentation updated

### Nice to Have

- ✅ Performance benchmarks meet targets (<100ms init, <10ms spawn)
- ✅ Memory usage acceptable
- ✅ Error handling comprehensive
- ✅ Observability (metrics, tracing)

## Dependencies

### Required Architecture References

- ✅ **KNOWLEDGE-WASM-036**: Four-Module Architecture
- ✅ **ADR-WASM-018**: Three-Layer Architecture
- ✅ **ADR-WASM-022**: Circular Dependency Remediation
- ✅ **ADR-WASM-023**: Module Boundary Enforcement
- ✅ **KNOWLEDGE-WASM-030**: Module Architecture Hard Requirements

### Related Tasks

**Supersedes:**
- 🔴 **WASM-TASK-HOTFIX-001**: Messaging Module Architecture Refactoring (partial fix - Phase 3 tasks included here)
- 🔴 **WASM-TASK-009**: AirsSys-OSL Bridge (different topic, outdated architecture)

**Prerequisite for:**
- WASM-TASK-006 (Block 5 - Messaging)
- WASM-TASK-007 (Block 6 - Storage)
- WASM-TASK-008 (Block 7 - Lifecycle)
- All future Block 8-11 tasks

### Blocks

This task **BLOCKS**:
- ❌ All subsequent WASM-TASK-006 development (cannot complete messaging cleanly)
- ❌ All subsequent WASM-TASK-007 development (cannot integrate storage with host_system/)
- ❌ All subsequent WASM-TASK-008 development (cannot integrate lifecycle with host_system/)
- ❌ All future Block 8-11 tasks: Need host_system/ infrastructure

### Technical Debt Impact

**Resolves:**
- ✅ Circular dependency between runtime/, messaging/, actor/
- ✅ Unclear orchestration ownership
- ✅ Overlapping responsibilities

**Related Debt:**
- DEBT-WASM-004 (deferred integration work - now integrated)
- DEBT-WASM-005 (capability system needs refactoring - now addressed)

## Notes

### HOTFIX-001 Relationship

**Action Required:**
```markdown
# In HOTFIX-001, add at end:
## Status Update

**Status:** 🔴 SUPERSEDED
**Reason:** New architecture (KNOWLEDGE-WASM-036) introduces `host_system/` module, which provides complete solution for module architecture. HOTFIX-001's partial refactoring is superseded by this comprehensive implementation.

**Replacement:** WASM-TASK-013 - Block 1: Host System Architecture Implementation

**Phase 3 Tasks Included:**
All HOTFIX-001 Phase 3 tasks (3.3, 3.4, 3.5, 3.6) are incorporated as phases 4, 5, 6 of this task.

### Migration Notes

This is a **breaking change** for:
- Any code that directly imports from actor/, messaging/, runtime/
- Any code that uses RuntimeOrchestrator (doesn't exist yet)
- Any test fixtures that assume current architecture

**Migration strategy:**
1. Update imports to use HostSystemManager
2. Update initialization patterns
3. Update test fixtures
4. Document breaking changes
5. Provide migration guide

## History

### Version History

- **2025-12-29**: 1.0 - Initial planning for host system architecture implementation

### Review History

- **2025-12-29**: Planned based on architectural analysis (KNOWLEDGE-WASM-036) and task exploration

---

## Status Update - 2025-12-30

**Cleanup Needed Before Implementation:**

Discovery during task planning: Two stub files in `messaging/` contain unused placeholder structs that should be deleted:

1. `src/messaging/fire_and_forget.rs` - Contains `FireAndForget { _inner: Arc<()> }` which is NOT used anywhere
2. `src/messaging/request_response.rs` - Contains `RequestResponse { _inner: Arc<()> }` which is NOT used anywhere

**What's Actually Used:**
- Fire-and-forget pattern: Implemented in `runtime/async_host.rs` as `SendMessageHostFunction` ✅
- Request-response pattern: Implemented in `runtime/async_host.rs` as `SendRequestHostFunction` ✅
- Message types: `FireAndForget` and `Request` are defined in `src/core/messaging.rs` (MessageType enum) ✅

**Action Required During Phase 1:**

```bash
# Delete unused stub files
rm src/messaging/fire_and_forget.rs
rm src/messaging/request_response.rs

# Update messaging/mod.rs to remove re-exports of these files
# Add direct use of core::messaging types instead:
# pub use crate::core::messaging::{FireAndForget, Request};
```

**Verification Commands:**
```bash
# Verify stub files deleted
test ! -f src/messaging/fire_and_forget.rs && echo "✅ Deleted" || echo "❌ Still exists"
test ! -f src/messaging/request_response.rs && echo "✅ Deleted" || echo "❌ Still exists"

# Verify messaging/mod.rs updated correctly
grep -n "pub use crate::core::messaging" src/messaging/mod.rs
```

**Rationale:**
- These stub files were created as placeholders in HOTFIX-001 Phase 1
- The actual messaging patterns (fire-and-forget, request-response) are implemented as host functions in `runtime/`
- No code in the project uses the structs from these stub files
- Keeping them creates confusion and violates single-responsibility principle

**Impact on TASK-013:**
- ✅ No changes to Phase 1-7 implementation plans
- ✅ Just pre-cleanup before starting implementation
- ✅ Reduces codebase confusion

---


## Implementation Plan - Phase 1: Module Structure & Basic Types

### Context & References

**ADR References:**
- **ADR-WASM-023**: Module Boundary Enforcement - Defines forbidden imports and module responsibilities. Host system must NOT import from runtime/ or any module it coordinates.
- **ADR-WASM-018**: Three-Layer Architecture - Foundation layering that host_system/ builds upon.

**Knowledge References:**
- **KNOWLEDGE-WASM-036**: Four-Module Architecture - Defines host_system/ as top-level coordinator that orchestrates actor/, messaging/, runtime/.
- **KNOWLEDGE-WASM-030**: Module Architecture Hard Requirements - Specifies dependency rules and module responsibilities.

**System Patterns:**
- Component Host Pattern from `system-patterns.md` - Host system coordinates initialization and lifecycle
- Runtime Deployment Engine pattern from `tech-context.md` - System initialization patterns

**PROJECTS_STANDARD.md Compliance:**
- **§2.1** (3-Layer Imports): All code will follow std → external → internal import organization
- **§4.3** (Module Architecture): mod.rs files will contain ONLY declarations and re-exports
- **§6.1** (YAGNI Principles): Implement only what's needed for Phase 1 - empty structs, no over-engineering
- **§6.2** (Avoid `dyn` Patterns): Use generics and concrete types, prefer static dispatch
- **§6.4** (Implementation Quality Gates): Zero warnings, comprehensive tests, clean builds

**Rust Guidelines Applied:**
- **M-DESIGN-FOR-AI**: Idiomatic APIs, thorough docs, testable code
- **M-MODULE-DOCS**: Module documentation with canonical sections (summary, examples, errors)
- **M-ERRORS-CANONICAL-STRUCTS**: Error types follow canonical structure from thiserror
- **M-STATIC-VERIFICATION**: All lints enabled, clippy passes with `-D warnings`
- **M-CANONICAL-DOCS**: Documentation includes summary, examples, errors, panics sections

**Documentation Standards:**
- **Diátaxis Type**: Reference documentation for module structure
- **Quality**: Technical language, no hyperbole per documentation-quality-standards.md
- **Compliance**: Standards Compliance Checklist will be added to task file

### Module Architecture

**Code will be placed in:** `src/host_system/`

**Module responsibilities (per KNOWLEDGE-WASM-036):**
- System initialization logic - Creating infrastructure in correct order
- Component lifecycle management - Spawn, start, stop, supervise
- Message flow coordination - Wiring up components with broker
- Correlation tracking - Track pending request-response pairs (Phase 2+)
- Timeout handling - Enforce request timeouts (Phase 2+)
- Startup/shutdown procedures - Graceful system lifecycle

**Allowed imports (per ADR-WASM-023 and KNOWLEDGE-WASM-036):**
- `host_system/` → `actor/` (ComponentActor, ComponentRegistry, ComponentSpawner, Supervisor)
- `host_system/` → `messaging/` (MessageBroker, MessagingService, FireAndForget, RequestResponse)
- `host_system/` → `runtime/` (WasmEngine, ComponentLoader, AsyncHostRegistry)
- `host_system/` → `core/` (All shared types and traits)

**Forbidden imports (per ADR-WASM-023):**
- `host_system/` → NOTHING imports from `host_system/` (it coordinates everything)

**Verification command (for implementer to run):**
```bash
# Phase 1: Verify no modules import from host_system/ (since it's new)
grep -r "use crate::host_system" airssys-wasm/src/
# Expected: no output (host_system/ is new, nothing should import it yet)

# Phase 1: Verify host_system/ doesn't create circular dependencies
# (This will be checked in later phases when dependencies are added)
```

### Phase 1 Subtasks

#### Subtask 1.1: Create host_system/ directory and mod.rs

**Deliverables:**
- Create directory: `airssys-wasm/src/host_system/`
- Create file: `airssys-wasm/src/host_system/mod.rs`

**Acceptance Criteria:**
- mod.rs follows §4.3 pattern (declarations only, no implementation)
- Module documentation follows M-CANONICAL-DOCS format

**ADR Constraints:**
- ADR-WASM-023: No forbidden imports in mod.rs
- KNOWLEDGE-WASM-036: mod.rs only contains module declarations and re-exports

**PROJECTS_STANDARD.md Compliance:**
- §2.1: Imports organized in 3 layers (std → external → internal)
- §4.3: mod.rs contains ONLY `pub mod` declarations and `pub use` re-exports

**Rust Guidelines:**
- M-MODULE-DOCS: Module documentation with summary, sections

**Documentation:**
- Diátaxis type: Reference documentation
- Quality: Technical language, no marketing terms
- Structure: Module-level docs explaining host_system/ purpose

**Implementation Details:**

```rust
// airssys-wasm/src/host_system/mod.rs

//! Host System Coordination Layer
//!
//! The host_system module provides system-wide coordination for the airssys-wasm
//! framework. It serves as the top-level orchestrator that manages component
//! lifecycle, system initialization, and message flow coordination.
//!
//! # Purpose
//!
//! The host system layer coordinates interactions between actor/, messaging/,
//! and runtime/ modules while maintaining clear separation of concerns.
//! It does NOT implement core operations but orchestrates when and how
//! operations are executed.
//!
//! # Architecture
//!
//! ```text
//! host_system/ (coordinates everything)
//!     ├── actor/ (wrappers and hosting)
//!     ├── messaging/ (message broker and patterns)
//!     ├── runtime/ (WASM execution)
//!     └── core/ (shared types and traits)
//! ```
//!
//! # Responsibilities
//!
//! - System initialization - Create infrastructure in correct order
//! - Component lifecycle management - Spawn, start, stop, supervise
//! - Message flow coordination - Wire up components with broker
//! - Correlation tracking - Track pending request-response pairs (Phase 2+)
//! - Timeout handling - Enforce request timeouts (Phase 2+)
//! - Startup/shutdown procedures - Graceful system lifecycle
//!
//! # What It Does NOT Own
//!
//! - WASM execution (runtime/)
//! - Message broker implementation (messaging/)
//! - Actor system primitives (actor/)
//! - Component actor logic (actor/)
//! ```

// Module declarations (§4.3 - declaration-only pattern)
pub mod manager;
pub mod initialization;
pub mod lifecycle;
pub mod messaging;

// Public re-exports (Phase 1 - manager only)
pub use manager::HostSystemManager;

// Additional re-exports will be added in later phases
// Phase 2+: correlation_tracker, timeout_handler
```

#### Subtask 1.2: Create manager.rs with empty HostSystemManager

**Deliverables:**
- Create file: `airssys-wasm/src/host_system/manager.rs`
- Create struct: `HostSystemManager` (empty implementation)

**Acceptance Criteria:**
- Struct is defined and compiles
- Empty new() constructor returns instance
- No implementation logic (Phase 1 only creates structure)

**ADR Constraints:**
- ADR-WASM-023: No forbidden imports
- KNOWLEDGE-WASM-036: Manager coordinates, doesn't implement (yet)

**PROJECTS_STANDARD.md Compliance:**
- §6.1 (YAGNI): No methods beyond what's needed for Phase 1
- §6.2 (Avoid `dyn`): Use concrete types, no trait objects
- §6.4 (Quality Gates): Zero warnings

**Rust Guidelines:**
- M-CANONICAL-DOCS: Struct documentation with summary, examples, errors
- M-DESIGN-FOR-AI: Testable, idiomatic API

**Documentation:**
- Diátaxis type: Reference documentation for struct
- Quality: Technical language
- Structure: Summary, examples, errors sections

**Implementation Details:**

```rust
// airssys-wasm/src/host_system/manager.rs

//! Host System Manager
//!
//! The HostSystemManager provides system-wide coordination for the airssys-wasm
//! framework. It manages component lifecycle, system initialization, and message
//! flow coordination.
//!
//! # Phase 1: Empty Placeholder
//!
//! In Phase 1, the HostSystemManager is an empty placeholder that establishes
//! the module structure. Full implementation will be added in Phase 4.
//!
//! # Planned Functionality (Phase 4+)
//!
//! - System initialization - Create and wire infrastructure
//! - Component lifecycle - Spawn, start, stop, supervise components
//! - Dependency injection - Coordinate actor/, messaging/, runtime/
//! - Graceful shutdown - Clean system teardown

use crate::core::{ComponentId, WasmError};

/// Host system coordinator for airssys-wasm framework.
///
/// The HostSystemManager manages system initialization, component lifecycle,
/// and message flow coordination between actor/, messaging/, and runtime/ modules.
///
/// # Phase 1 Implementation
///
/// In Phase 1, this struct is an empty placeholder. Full implementation
/// including initialization logic and lifecycle management will be added in Phase 4.
///
/// # Examples
///
/// ```rust,ignore
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// use airssys_wasm::host_system::HostSystemManager;
///
/// // Create manager (Phase 4+ will initialize infrastructure)
/// let manager = HostSystemManager::new().await?;
///
/// // Spawn components (Phase 4+)
/// // let component_id = manager.spawn_component(...).await?;
///
/// // Graceful shutdown (Phase 4+)
/// // manager.shutdown().await?;
///
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Phase 1: No errors (empty implementation)
///
/// Phase 4+:
/// - `WasmError::InitializationFailed`: System initialization failed
/// - `WasmError::ComponentNotFound`: Component ID not found
/// - `WasmError::ComponentSpawnFailed`: Component spawn failed
#[derive(Debug)]
pub struct HostSystemManager;

impl HostSystemManager {
    /// Creates a new HostSystemManager instance.
    ///
    /// Phase 1: Returns empty placeholder.
    ///
    /// Phase 4+: Initializes infrastructure (actor system, message broker, WASM engine).
    ///
    /// # Returns
    ///
    /// Returns a `HostSystemManager` instance.
    ///
    /// # Errors
    ///
    /// Phase 1: No errors.
    ///
    /// Phase 4+:
    /// - `WasmError::InitializationFailed`: Infrastructure initialization failed
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use airssys_wasm::host_system::HostSystemManager;
    ///
    /// let manager = HostSystemManager::new().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new() -> Result<Self, WasmError> {
        // Phase 1: Empty placeholder
        // Phase 4+: Initialize infrastructure
        Ok(Self)
    }
}
```

#### Subtask 1.3: Create initialization.rs (empty placeholder)

**Deliverables:**
- Create file: `airssys-wasm/src/host_system/initialization.rs`
- Module documentation only (no implementation yet)

**Acceptance Criteria:**
- File exists and compiles
- Documentation follows M-CANONICAL-DOCS format

**ADR Constraints:**
- ADR-WASM-023: No forbidden imports
- KNOWLEDGE-WASM-036: Initialization logic in host_system/

**PROJECTS_STANDARD.md Compliance:**
- §6.1 (YAGNI): Empty placeholder, no over-engineering
- §6.4 (Quality Gates): Zero warnings

**Rust Guidelines:**
- M-CANONICAL-DOCS: Module documentation

**Documentation:**
- Diátaxis type: Reference documentation
- Quality: Technical language

**Implementation Details:**

```rust
// airssys-wasm/src/host_system/initialization.rs

//! System Initialization Logic
//!
//! This module provides system initialization logic for the host system.
//! It coordinates the creation and wiring of infrastructure components
//! (actor system, message broker, WASM engine) in the correct order.
//!
//! # Phase 1: Empty Placeholder
//!
//! In Phase 1, this module contains only documentation. Implementation
//! will be added in Phase 4.
//!
//! # Planned Functionality (Phase 4)
//!
//! - Initialize actor system infrastructure
//! - Initialize message broker
//! - Initialize WASM engine and component loader
//! - Wire up dependencies between modules
//! - Start background tasks (subscriber, health monitor)
//!
//! # Architecture
//!
//! ```text
//! Initialization Order:
//!
//! 1. Create core infrastructure (engine, broker, registry)
//! 2. Create actor-level infrastructure (subscriber, spawner)
//! 3. Create host_system-level infrastructure (tracker, router)
//! 4. Start subscriber (wires up message flow)
//! 5. Start health monitoring
//! ```
```

#### Subtask 1.4: Create lifecycle.rs (empty placeholder)

**Deliverables:**
- Create file: `airssys-wasm/src/host_system/lifecycle.rs`
- Module documentation only (no implementation yet)

**Acceptance Criteria:**
- File exists and compiles
- Documentation follows M-CANONICAL-DOCS format

**ADR Constraints:**
- ADR-WASM-023: No forbidden imports
- KNOWLEDGE-WASM-036: Lifecycle management in host_system/

**PROJECTS_STANDARD.md Compliance:**
- §6.1 (YAGNI): Empty placeholder, no over-engineering
- §6.4 (Quality Gates): Zero warnings

**Rust Guidelines:**
- M-CANONICAL-DOCS: Module documentation

**Documentation:**
- Diátaxis type: Reference documentation
- Quality: Technical language

**Implementation Details:**

```rust
// airssys-wasm/src/host_system/lifecycle.rs

//! Component Lifecycle Management
//!
//! This module provides component lifecycle management for the host system.
//! It handles spawning, starting, stopping, and supervising components.
//!
//! # Phase 1: Empty Placeholder
//!
//! In Phase 1, this module contains only documentation. Implementation
//! will be added in Phase 4.
//!
//! # Planned Functionality (Phase 4)
//!
//! - spawn_component() - Create and start a new component
//! - stop_component() - Stop a running component
//! - restart_component() - Restart a component (for supervision)
//! - get_component_status() - Query component health and state
//!
//! # Architecture
//!
//! ```text
//! Lifecycle Flow:
//!
//! Spawn:
//!   1. Load WASM (delegates to runtime/)
//!   2. Create component actor (delegates to actor/)
//!   3. Spawn actor (delegates to actor/)
//!   4. Register for messaging (orchestrator coordinates)
//!   5. Start health monitoring (orchestrator coordinates)
//!
//! Stop:
//!   1. Stop health monitoring
//!   2. Unregister from messaging
//!   3. Stop actor (delegates to actor/)
//! ```
```

#### Subtask 1.5: Create messaging.rs (empty placeholder)

**Deliverables:**
- Create file: `airssys-wasm/src/host_system/messaging.rs`
- Module documentation only (no implementation yet)

**Acceptance Criteria:**
- File exists and compiles
- Documentation follows M-CANONICAL-DOCS format

**ADR Constraints:**
- ADR-WASM-023: No forbidden imports
- KNOWLEDGE-WASM-036: Message flow coordination in host_system/

**PROJECTS_STANDARD.md Compliance:**
- §6.1 (YAGNI): Empty placeholder, no over-engineering
- §6.4 (Quality Gates): Zero warnings

**Rust Guidelines:**
- M-CANONICAL-DOCS: Module documentation

**Documentation:**
- Diátaxis type: Reference documentation
- Quality: Technical language

**Implementation Details:**

```rust
// airssys-wasm/src/host_system/messaging.rs

//! Message Flow Coordination
//!
//! This module provides message flow coordination for the host system.
//! It wires up the message broker with component mailboxes and
//! coordinates message routing.
//!
//! # Phase 1: Empty Placeholder
//!
//! In Phase 1, this module contains only documentation. Implementation
//! will be added in Phase 4.
//!
//! # Planned Functionality (Phase 4)
//!
//! - Wire up message broker with component mailboxes
//! - Coordinate message flow through actor system
//! - Register components for message delivery
//! - Unregister components on shutdown
//!
//! # Architecture
//!
//! ```text
//! Message Flow:
//!
//! Component A → ActorSystemSubscriber → MessageBroker → Component B
//!    (host_system/ coordinates)
//!
//! The host_system/ module coordinates the wiring but does not
//! implement the message routing itself (that's in messaging/).
//! ```
```

#### Subtask 1.6: Update src/lib.rs to include host_system module

**Deliverables:**
- Update `airssys-wasm/src/lib.rs`
- Add `pub mod host_system;` declaration

**Acceptance Criteria:**
- lib.rs compiles
- host_system module is publicly visible
- Module appears in crate documentation

**ADR Constraints:**
- ADR-WASM-023: No forbidden imports in lib.rs

**PROJECTS_STANDARD.md Compliance:**
- §4.3: Module declarations organized by logical grouping
- §2.1: Imports organized in 3 layers

**Rust Guidelines:**
- M-CANONICAL-DOCS: Documentation updated to reference host_system/

**Documentation:**
- Update lib.rs module documentation to include host_system/
- Update architecture overview diagram

**Implementation Details:**

```rust
// Add to airssys-wasm/src/lib.rs after existing module declarations

// Host System Coordination Layer (Block 1 - WASM-TASK-013)
pub mod host_system;
```

```rust
// Update lib.rs overview documentation

//! # Architecture
//!
//! The framework is organized into primary modules:
//!
//! - **[`core`]** - Foundational abstractions, types, and trait contracts
//! - **[`host_system`]** - System coordination, initialization, and lifecycle
//! - **[`runtime`]** - WASM execution engine and loading
//! - **[`actor`]** - Actor system integration and component hosting
//! - **[`security`]** - Capability-based security and access control
//! - **[`messaging`]** - Inter-component communication infrastructure
```

#### Subtask 1.7: Delete unused stub files (if they exist)

**Deliverables:**
- Verify stub files exist before deletion
- Delete: `src/messaging/fire_and_forget.rs` (if exists)
- Delete: `src/messaging/request_response.rs` (if exists)
- Update messaging/mod.rs to remove any references (if they exist)

**Acceptance Criteria:**
- Stub files deleted (only if they existed)
- No compilation errors after deletion
- messaging/mod.rs compiles cleanly

**ADR Constraints:**
- No ADR violations (cleanup only)

**PROJECTS_STANDARD.md Compliance:**
- No impact (cleanup operation)

**Rust Guidelines:**
- No impact (cleanup operation)

**Implementation Details:**

```bash
# Verify files exist before deletion
if [ -f "src/messaging/fire_and_forget.rs" ]; then
    echo "Deleting stub file: src/messaging/fire_and_forget.rs"
    rm src/messaging/fire_and_forget.rs
fi

if [ -f "src/messaging/request_response.rs" ]; then
    echo "Deleting stub file: src/messaging/request_response.rs"
    rm src/messaging/request_response.rs
fi

# Verify deletion
test ! -f src/messaging/fire_and_forget.rs && echo "✅ fire_and_forget.rs deleted" || echo "✅ No file to delete"
test ! -f src/messaging/request_response.rs && echo "✅ request_response.rs deleted" || echo "✅ No file to delete"
```

#### Subtask 1.8: Add basic tests for host_system module

**Deliverables:**
- Create: `airssys-wasm/src/host_system/manager.rs` tests in `#[cfg(test)]` module
- Test: HostSystemManager::new() creates instance
- Test: Module compiles and is publicly accessible

**Acceptance Criteria:**
- Unit tests compile
- All tests pass
- Test coverage > 80% for new code

**ADR Constraints:**
- No ADR violations

**PROJECTS_STANDARD.md Compliance:**
- §6.4: Comprehensive tests
- Mandatory testing requirement: BOTH unit and integration tests required

**Rust Guidelines:**
- M-DESIGN-FOR-AI: Testable code

**Documentation:**
- Test documentation explains what is being tested

**Implementation Details:**

```rust
// Add to airssys-wasm/src/host_system/manager.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_host_system_manager_new() {
        // Test that HostSystemManager::new() creates instance
        let manager = HostSystemManager::new().await;
        assert!(manager.is_ok(), "HostSystemManager::new() should succeed");
    }

    #[tokio::test]
    async fn test_host_system_manager_is_debug() {
        // Test that HostSystemManager implements Debug
        let manager = HostSystemManager::new().await.unwrap();
        let debug_str = format!("{:?}", manager);
        assert!(!debug_str.is_empty(), "Debug output should not be empty");
    }
}
```

### Integration Testing Plan

**Integration Test Deliverables:**
- Create file: `airssys-wasm/tests/host_system-integration-tests.rs`

**Integration Tests to Include:**
1. **HostSystemManager Instantiation Test**
   - Test: Create HostSystemManager from external crate
   - Verify: Manager instantiates without errors
   - Verify: Public API is accessible

2. **Module Accessibility Test**
   - Test: Import host_system module from integration test
   - Verify: All public types are accessible
   - Verify: Module structure matches documentation

3. **Basic System Verification Test**
   - Test: Verify host_system module is properly wired in lib.rs
   - Verify: Module appears in crate documentation
   - Verify: No circular dependency violations

**Verification Command:**
```bash
# Run integration tests
cargo test --test host_system-integration-tests
# Expected: All tests pass

# Verify integration test file exists
test -f tests/host_system-integration-tests.rs && echo "✅ Integration test file exists" || echo "❌ Integration test file missing"
```

**Integration Test Implementation Details:**

```rust
// airssys-wasm/tests/host_system-integration-tests.rs

use airssys_wasm::host_system::HostSystemManager;

#[tokio::test]
async fn test_host_system_manager_integration() {
    // Test that HostSystemManager can be instantiated from external context
    let manager = HostSystemManager::new().await;
    assert!(manager.is_ok(), "HostSystemManager should instantiate");
    
    let manager = manager.unwrap();
    
    // Test Debug trait implementation (integration-level verification)
    let debug_str = format!("{:?}", manager);
    assert!(!debug_str.is_empty(), "Debug output should not be empty");
}

#[tokio::test]
async fn test_module_accessibility() {
    // Test that all public types are accessible from integration context
    // This verifies module structure and public API surface
    use airssys_wasm::host_system::HostSystemManager;
    
    // Verify we can construct types
    let manager = HostSystemManager::new().await;
    assert!(manager.is_ok(), "Module API should be accessible");
}

#[tokio::test]
async fn test_module_wiring() {
    // Test that host_system module is properly wired in lib.rs
    // This verifies the module is publicly exposed
    use airssys_wasm::host_system::HostSystemManager;
    
    // If this compiles, the module is properly wired
    let _manager = HostSystemManager::new().await;
}
```

**Mandatory Testing Requirement Reminder:**
Per AGENTS.md Section 8, this plan MUST include BOTH unit tests AND integration tests:
- ✅ Unit tests: Included in Subtask 1.8 (in `#[cfg(test)]` block)
- ✅ Integration tests: Included in this Integration Testing Plan section

### Fixture Verification

**Verification Command:**
```bash
ls -la airssys-wasm/tests/fixtures/
```

**Verification Results:**
```
total 152
drwxr-xr-x  21 hiraq  staff   672 Dec 26 22:26 .
drwxr-xr-x  53 hiraq  staff  1696 Dec 28 17:26 ..
-rw-r--r--   1 hiraq  staff   162 Dec 26 22:26 basic-handle-message.wasm
-rw-r--r--   1 hiraq  staff   965 Dec 26 18:49 basic-handle-message.wat
-rwxr-xr-x   1 hiraq  staff   448 Dec 26 18:49 build.sh
-rw-r--r--   1 hiraq  staff   630 Dec 26 22:26 callback-receiver-component.wasm
-rw-r--r--   1 hiraq  staff  3772 Dec 26 18:49 callback-receiver-component.wat
-rw-r--r--   1 hiraq  staff   177 Dec 26 22:26 echo-handler.wasm
-rw-r--r--   1 hiraq  staff  1289 Dec 26 18:49 echo-handler.wat
-rw-r--r--   1 hiraq  staff   493 Dec 26 22:26 handle-message-component.wasm
-rw-r--r--   1 hiraq  staff  2875 Dec 26 18:49 handle-message-component.wat
-rw-r--r--   1 hiraq  staff   149 Dec 26 22:26 hello_world.wasm
-rw-r--r--   1 hiraq  staff   549 Dec 26 18:49 hello_world.wat
-rw-r--r--   1 hiraq  staff    85 Dec 26 22:26 no-handle-message.wasm
-rw-r--r--   1 hiraq  staff   498 Dec 26 18:49 no-handle-message.wat
-rw-r--r--   1 hiraq  staff   163 Dec 26 22:26 rejecting-handler.wasm
-rw-r--r--   1 hiraq  staff   935 Dec 26 18:49 rejecting-handler.wat
-rw-r--r--   1 hiraq  staff   173 Dec 26 22:26 sender-validator.wasm
-rw-r--r--   1 hiraq  staff  1062 Dec 26 18:49 sender-validator.wat
-rw-r--r--   1 hiraq  staff   223 Dec 26 22:26 slow-handler.wasm
-rw-r--r--   1 hiraq  staff  1165 Dec 26 18:49 slow-handler.wat
```

**Analysis:**
- ✅ **Fixtures directory exists**: `airssys-wasm/tests/fixtures/` found
- ✅ **15 WASM files available** (9 .wasm files + 6 .wat files + build.sh)
- ✅ **Variety of test components**: Basic handlers, validators, slow handlers, etc.

**Impact on Phase 1 Implementation:**
- ✅ Integration tests can use existing fixtures
- ✅ No new fixture creation required for Phase 1
- ✅ Fixtures can be used in later phases for component lifecycle testing

**Phase 1 Integration Test Usage:**
While Phase 1 integration tests focus on module accessibility and basic instantiation, the existing fixtures provide a foundation for future phases (Phase 4+) when component lifecycle integration tests will need real WASM components.

### Quality Standards

**All subtasks must meet:**
- ✅ Code builds without errors: `cargo build`
- ✅ Zero compiler warnings: `cargo build` produces no warnings
- ✅ Zero clippy warnings: `cargo clippy --all-targets --all-features -- -D warnings`
- ✅ Follows PROJECTS_STANDARD.md §2.1-§6.4
- ✅ Follows Rust guidelines (M-DESIGN-FOR-AI, M-MODULE-DOCS, M-CANONICAL-DOCS, etc.)
- ✅ Unit tests in `#[cfg(test)]` blocks
- ✅ All tests pass: `cargo test --lib host_system`
- ✅ Documentation follows quality standards (no hyperbole)
- ✅ Module documentation includes canonical sections
- ✅ Standards Compliance Checklist in task file

### Verification Checklist

**For implementer to run after completing Phase 1:**

```bash
# 1. Build
cd airssys-wasm
cargo build
# Expected: No warnings, builds cleanly

# 2. Unit Tests
cargo test --lib host_system
# Expected: All unit tests pass

# 2b. Integration Tests
cargo test --test host_system-integration-tests
# Expected: All integration tests pass

# 3. Clippy
cargo clippy --all-targets --all-features -- -D warnings
# Expected: Zero warnings

# 4. Verify module structure
ls -la src/host_system/
# Expected: mod.rs, manager.rs, initialization.rs, lifecycle.rs, messaging.rs

# 5. Verify lib.rs includes host_system
grep "pub mod host_system" src/lib.rs
# Expected: Line found

# 6. Verify no stub files exist
test ! -f src/messaging/fire_and_forget.rs && echo "✅ Deleted" || echo "✅ No file"
test ! -f src/messaging/request_response.rs && echo "✅ Deleted" || echo "✅ No file"

# 7. Verify module is accessible
cargo doc --no-deps --open
# Expected: host_system module visible in docs

# 8. Verify integration test file exists
test -f tests/host_system-integration-tests.rs && echo "✅ Integration test file exists" || echo "❌ Integration test file missing"
# Expected: Integration test file exists

# 9. Verify import organization (§2.1)
# Check that files follow 3-layer import pattern
# (Visual inspection or automated check)
```

### Documentation Requirements

**For documentation deliverables:**
- **Follow Diátaxis guidelines**: Reference type for module and struct documentation
- **Quality standards**: No hyperbole, professional tone, technical precision per documentation-quality-standards.md
- **Canonical sections**: Summary, examples, errors, panics per M-CANONICAL-DOCS
- **Module documentation**: Clear explanation of purpose and responsibilities

**Standards Compliance Checklist (to be added to task file):**

```markdown
## Standards Compliance Checklist - Phase 1

**PROJECTS_STANDARD.md Applied:**
- [ ] **§2.1 3-Layer Import Organization** - Evidence: All files follow std → external → internal pattern
- [ ] **§4.3 Module Architecture Patterns** - Evidence: mod.rs contains only declarations and re-exports
- [ ] **§6.1 YAGNI Principles** - Evidence: Empty placeholders, no over-engineering
- [ ] **§6.2 Avoid `dyn` Patterns** - Evidence: Concrete types used, no trait objects
- [ ] **§6.4 Implementation Quality Gates** - Evidence: Build, test, clippy all pass

**Rust Guidelines Applied:**
- [ ] **M-DESIGN-FOR-AI** - Idiomatic APIs, docs, tests
- [ ] **M-MODULE-DOCS** - Module documentation complete with canonical sections
- [ ] **M-CANONICAL-DOCS** - Struct/Function docs include summary, examples, errors
- [ ] **M-STATIC-VERIFICATION** - Lints enabled, clippy passes

**Documentation Quality:**
- [ ] **No hyperbolic terms** - Verified against forbidden list
- [ ] **Technical precision** - All claims measurable and factual
- [ ] **Diátaxis compliance** - Reference documentation type used correctly
- [ ] **Canonical sections** - All public items have summary, examples, errors
```

