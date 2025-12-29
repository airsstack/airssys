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

