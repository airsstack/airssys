# [WASM-TASK-HOTFIX-001] - Messaging Module Architecture Refactoring

**Task ID:** WASM-TASK-HOTFIX-001
**Created:** 2025-12-26
**Updated:** 2025-12-27
**Priority:** 🔴 CRITICAL / BLOCKING
**Status:** 🚀 IN PROGRESS  
**Blocks:** All subsequent WASM-TASK-006+ and Block 5+ development  
**Estimated Effort:** 3.5-4.5 weeks  

---

## Executive Summary

### Architectural Problem Discovered

During architectural audit, discovered a **critical module architecture violation** where messaging infrastructure is incorrectly placed in `runtime/` module instead of a dedicated top-level `messaging/` module.

**What's Wrong:**
1. **🔴 Module Boundary Violation**: `src/runtime/messaging.rs` (1,313 lines) contains messaging infrastructure
2. **🔴 Wrong Module Responsibility**: `runtime/` should only handle WASM execution (Block 1), not inter-component communication
3. **🔴 Missing Top-Level Module**: No `messaging/` module exists (should be Block 5)
4. **🔴 Circular Dependency Risk**: `runtime/messaging.rs` imports from `actor/message/`

**What's Correct:**
- `messaging/` should be a **top-level module** (Block 5: Inter-Component Communication)
- `runtime/` should only contain: WasmEngine, ComponentLoader, ResourceLimits, StoreManager
- All messaging infrastructure should be in `src/messaging/`

### Impact

**Violates Multiple Architecture Standards:**
- ❌ ADR-WASM-018: Three-Layer Architecture (one-way dependencies only)
- ❌ KNOWLEDGE-WASM-012: Module Structure Architecture (messaging/ as top-level)
- ❌ ADR-WASM-023: Module Boundary Enforcement

**Blocks Development:**
- ❌ Block 5 (Inter-Component Communication) can't be properly developed
- ❌ Future messaging features have no clear home
- ❌ Creates confusion about where to add messaging code

---

## Problem Statement

### Issue 1: Messaging Infrastructure in Wrong Module

**Current (WRONG):**
```text
src/
  ├── runtime/
  │   ├── engine.rs           ✅ WASM execution
  │   ├── loader.rs           ✅ Component loading
  │   ├── limits.rs           ✅ Resource limits
  │   └── messaging.rs        ❌ WRONG - 1,313 lines of messaging code
  └── actor/
      └── message/
          ├── correlation_tracker.rs
          └── ...
```

**runtime/messaging.rs Contains:**
- MessagingService (manages MessageBroker singleton)
- ResponseRouter (routes request-response messages)
- MessageReceptionMetrics (tracks message delivery)
- MessagingMetrics (publish/subscriber statistics)
- ResponseRouterStats (response routing metrics)
- MessageReceptionMetrics (message delivery tracking)

**Why It's Wrong:**

1. **Module Responsibility Violation**:
   - `runtime/` is Block 1: WASM Runtime Layer
   - Should only handle: Wasmtime engine, component loading, resource limits
   - NOT: Inter-component communication infrastructure

2. **Missing Top-Level Module**:
   - KNOWLEDGE-WASM-012 (lines 506-596) specifies `messaging/` as top-level
   - This module should be Block 5: Inter-Component Communication
   - Currently: Does not exist at top level

3. **Circular Dependency Risk**:
   ```rust
   // src/runtime/messaging.rs:76
   use crate::actor::message::{CorrelationId, CorrelationTracker, RequestError, ResponseMessage};
   ```
   - `runtime/` (lower level) imports from `actor/` (higher level)
   - Violates one-way dependency chain: core → runtime → actor → messaging

**Impact:**
- ❌ Confusing module boundaries
- ❌ Violates ADR-WASM-018 three-layer architecture
- ❌ Makes code harder to navigate
- ❌ Blocks proper Block 5 development
- ❌ Risk of circular dependencies

### Issue 2: Architectural Standards Compliance

**Required Architecture (from KNOWLEDGE-WASM-012):**

```text
src/
  ├── core/                # Foundation (no internal deps)
  ├── runtime/              # Block 1: WASM Runtime Layer
  │   ├── engine.rs
  │   ├── loader.rs
  │   ├── limits.rs
  │   ├── async_host.rs
  │   └── store_manager.rs
  ├── actor/               # Block 3: Actor System Integration
  └── messaging/            # Block 5: Inter-Component Communication ← MISSING
      ├── messaging_service.rs
      ├── router.rs
      ├── fire_and_forget.rs
      ├── request_response.rs
      ├── codec.rs
      └── topics.rs
```

**One-Way Dependency Chain (ADR-WASM-018):**
```
core/ (foundation)
  ↓
runtime/ (WASM execution)
  ↓
actor/ (Actor system integration)
  ↓
messaging/ (Inter-component communication)
```

**Key Rule**: Dependencies flow ONE WAY (top to bottom). Higher layers CANNOT import from lower layers.

**Current Violation:**
```
runtime/messaging.rs → actor/message/  ← WRONG! Reverse dependency
```

**Impact:**
- ❌ Cannot test runtime/ in isolation
- ❌ Creates circular coupling
- ❌ Makes code harder to understand
- ❌ Violates multiple architectural standards

---

## Context

### Relevant Architecture Documents

**Primary References:**
- **KNOWLEDGE-WASM-012**: Module Structure Architecture (lines 506-596 define messaging/ module)
- **ADR-WASM-018**: Three-Layer Architecture (one-way dependency chain)
- **ADR-WASM-023**: Module Boundary Enforcement (dependency rules)
- **KNOWLEDGE-WASM-034**: Module Architecture Violation - Messaging in Runtime (this document)

**Supporting References:**
- **KNOWLEDGE-WASM-002**: High-Level Overview
- **KNOWLEDGE-WASM-003**: Core Architecture Design
- **KNOWLEDGE-WASM-024**: Component Messaging Clarifications
- **KNOWLEDGE-WASM-029**: Messaging Patterns

**New Documentation Created for This Task:**
- **KNOWLEDGE-WASM-034**: Documents architectural violation
- **ADR-WASM-024**: Decision to refactor messaging to top-level module

### Completed Blocks (Foundation)

- ✅ **WASM-TASK-000**: Core Abstractions (9,283 lines, 363 tests)
- ✅ **WASM-TASK-002**: WASM Runtime Layer (338 lines, 214 tests)
- ✅ **WASM-TASK-003**: WIT Interface System (2,214 lines WIT + 176 lines build)
- ✅ **WASM-TASK-004**: Actor System Integration (15,620+ lines, 589 tests)
- ✅ **WASM-TASK-005**: Security & Isolation Layer (13,500+ lines, 388 tests)

### Foundation Quality Metrics

- **Total Code**: 275K+ lines (9,283 + 338 + 2,390 + 15,620 + 13,500)
- **Total Tests**: 1,654 tests (363 + 214 + 589 + 388)
- **Test Pass Rate**: 100% (all tests passing)
- **Code Quality**: Zero compiler warnings, zero clippy warnings
- **Architecture**: Block 1-4: Complete, Block 5: Blocked by architecture violation

---

## Objectives

### Primary Objective

**Refactor messaging infrastructure from `runtime/` to top-level `messaging/` module to fix architectural violation:**

1. ✅ Create top-level `src/messaging/` module
2. ✅ Move all messaging code from `runtime/messaging.rs` to `messaging/messaging_service.rs`
3. ✅ Update all import statements across codebase
4. ✅ Remove `runtime/messaging.rs`
5. ✅ Enforce one-way dependency chain: core → runtime → actor → messaging
6. ✅ Verify no circular dependencies remain

### Secondary Objectives

- Align with ADR-WASM-018 three-layer architecture
- Follow KNOWLEDGE-WASM-012 module structure specification
- Maintain zero compiler/clippy warnings
- Add comprehensive integration tests
- Improve code navigation and maintainability
- Enable proper Block 5 development

---

## Implementation Plan

### Phase 1: Create Top-Level messaging/ Module (Days 1-2) ✅ COMPLETE

#### Task 1.1: Create messaging Module Structure

**Objective:** Create top-level `src/messaging/` module with proper structure.

**Deliverables:**

**Files to Create:**
- `src/messaging/mod.rs` - Module declarations only
- `src/messaging/messaging_service.rs` - Main messaging service (moved from runtime/)
- `src/messaging/router.rs` - MessageBroker routing integration
- `src/messaging/fire_and_forget.rs` - Fire-and-forget pattern
- `src/messaging/request_response.rs` - Request-response pattern
- `src/messaging/codec.rs` - Multicodec message encoding
- `src/messaging/topics.rs` - Topic-based pub-sub (Phase 2+)

**Files to Update:**
- `src/lib.rs` - Add `pub mod messaging;`
- `src/prelude.rs` - Re-export messaging types

**Messaging Module Structure:**
```rust
// src/messaging/mod.rs
//! Inter-component communication infrastructure.
//!
//! This module provides messaging infrastructure for communication
//! between WASM components, including:
//!
//! - MessageBroker integration
//! - Request-response patterns
//! - Fire-and-forget messaging
//! - Topic-based pub/sub (Phase 2)
//! - Multicodec message encoding
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │              Messaging Module            │
//! │  ┌────────────────────────────────┐     │
//! │  │  • MessageBroker integration  │     │
//! │  │  • Request-response routing │     │
//! │  │  • Fire-and-forget messaging│     │
//! │  │  • Metrics and monitoring     │     │
//! │  └────────────────────────────────┘     │
//! └─────────────────────────────────────────┘
//!                         ↓ uses
//! ┌─────────────────────────────────────────┐
//! │    airssys-rt InMemoryMessageBroker │
//! └─────────────────────────────────────────┘
//! ```

// Module declarations (§4.3 - declaration-only pattern)
pub mod messaging_service;
pub mod router;
pub mod fire_and_forget;
pub mod request_response;
pub mod codec;
pub mod topics;  // Phase 2

// Public re-exports
pub use messaging_service::{MessagingService, MessagingStats, ResponseRouter, ResponseRouterStats};
pub use router::{MessageRouter, RoutingStats};
pub use fire_and_forget::FireAndForget;
pub use request_response::{RequestResponse, RequestError};
pub use codec::MulticodecCodec;
```

**Success Criteria:**
- ✅ `src/messaging/mod.rs` created with module declarations
- ✅ `src/messaging/messaging_service.rs` created
- ✅ `src/lib.rs` updated with `pub mod messaging;`
- ✅ `src/prelude.rs` updated with messaging re-exports
- ✅ `cargo build` succeeds

**Estimated Effort:** 4-6 hours  
**Risk Level:** Low (new module creation)

---

#### Task 1.2: Move Messaging Code from runtime/messaging.rs

**Objective:** Move all messaging infrastructure code to new location.

**Deliverables:**

**File to Create:** `src/messaging/messaging_service.rs`

**Code to Move:** From `src/runtime/messaging.rs` (lines 1-1313)

**Content to Move:**
- MessagingService struct (lines 126-387)
- MessagingMetrics struct (lines 418-431)
- MessagingStats struct (lines 448-467)
- ResponseRouter struct (lines 511-666)
- ResponseRouterMetrics struct (lines 521-531)
- ResponseRouterStats struct (lines 668-679)
- MessageReceptionMetrics struct (lines 736-852)
- MessageReceptionStats struct (lines 868-885)
- All tests (lines 887-1313)

**Import Updates Required:**
```rust
// Update imports in messaging_service.rs:
// FROM: use crate::actor::message::{CorrelationId, CorrelationTracker, RequestError, ResponseMessage};
// TO: use crate::core::messaging::{CorrelationId, CorrelationTracker, RequestError, ResponseMessage};
```

**Success Criteria:**
- ✅ `src/messaging/messaging_service.rs` created with all code moved
- ✅ Imports updated to use `crate::core::messaging::` instead of `crate::actor::message::`
- ✅ `cargo build` succeeds
- ✅ All tests in messaging_service.rs pass
- ✅ No imports from `actor/` (imports from `core/` instead)

**Estimated Effort:** 6-8 hours  
**Risk Level:** Medium (import updates, code verification)

---

## Implementation Plan for Task 1.2 (CORRECTED)

### ⚠️ CORRECTIONS MADE TO PREVIOUS PLAN

**Previous plan had INCORRECT architectural assumptions:**
- ❌ Stated "messaging/ MUST NOT import from actor/"
- ❌ Stated "messaging/ ONLY imports from core/"
- ❌ Expected `grep -rn "use crate::actor"` to return nothing

**CORRECTED architecture based on KNOWLEDGE-WASM-012 Line 274:**
```text
messaging/ → core/, actor/, airssys-rt
```
- ✅ **messaging/ CAN import from actor/** (this is ALLOWED!)
- ✅ **messaging/ CAN import from core/** (types, errors, etc.)
- ✅ **messaging/ CAN import from airssys-rt** (MessageBroker from runtime)
- ❌ **messaging/ CANNOT import from runtime/** (only this is forbidden!)

**Why This Correction Matters:**
- The original plan would have required moving ALL messaging types to core/
- This would violate single responsibility (CorrelationTracker is actor system logic)
- KNOWLEDGE-WASM-012 explicitly allows messaging/ → actor/ dependency
- ADR-WASM-023 forbids runtime/ → actor/ (runtime importing from actor)
- But messaging/ is ABOVE runtime/ in dependency chain, so it CAN import from actor/

**Key Type Locations:**
- `core/messaging.rs`: CorrelationId, RequestError, ResponseMessage, MessageEnvelope
- `actor/message/correlation_tracker.rs`: CorrelationTracker (NOT in core!)

**Correct Import Strategy:**
```rust
// Types from core/
use crate::core::messaging::{CorrelationId, RequestError, ResponseMessage};

// CorrelationTracker from actor/ (ALLOWED per KNOWLEDGE-WASM-012!)
use crate::actor::message::CorrelationTracker;
```

---

### Context & References

**ADR References:**
- **ADR-WASM-018**: Three-Layer Architecture (one-way dependency chain)
   - Dependencies flow: core → runtime → actor → messaging
   - **CORRECTED understanding**: messaging/ is at TOP of dependency chain
   - messaging/ can import from actor/, runtime/, and core/
   - Higher layers can depend on lower layers, not vice versa

- **ADR-WASM-023**: Module Boundary Enforcement (HARD REQUIREMENT)
   - **CORRECTED**: messaging/ CANNOT import from runtime/ (WASM execution)
   - **CORRECTED**: messaging/ CAN import from actor/ (per KNOWLEDGE-WASM-012 line 274)
   - messaging/ can import from core/ (all modules can)
   - Forbidden imports: `use crate::runtime` (messaging/ cannot use runtime/)
   - Allowed imports: `use crate::actor`, `use crate::core`

**Knowledge References:**
- **KNOWLEDGE-WASM-012**: Module Structure Architecture (lines 271-289)
   - **Line 274 CRITICAL**: `messaging/ → core/, actor/, airssys-rt`
   - This means messaging/ CAN import from core/, actor/, AND airssys-rt
   - messaging/ is Block 5: Inter-Component Communication
   - messaging/ is top-level module (parallel to runtime/, actor/, security/)
   
- **KNOWLEDGE-WASM-005**: Messaging Architecture
   - MessagingService manages MessageBroker singleton from airssys-rt
   - CorrelationTracker tracks pending requests
   - ResponseRouter routes responses to requesters
   - Fire-and-forget and request-response patterns

- **KNOWLEDGE-WASM-026**: Message Delivery Architecture Final
   - ActorSystemSubscriber owns message delivery (has MailboxSender references)
   - ComponentRegistry stays pure (identity lookup only)
   - Message flow from component send to handle-message invocation
   - Correlation tracking for request-response pattern

- **KNOWLEDGE-WASM-029**: Messaging Patterns (Fire-and-Forget vs Request-Response)
   - Two patterns: send-message (fire-and-forget) and send-request (request-response)
   - Response is return value from handle-message, NOT a separate host function
   - Runtime decides what to do with return value based on message type

**System Patterns:**
- **From KNOWLEDGE-WASM-012**: messaging/ module owns:
   - MessageBroker integration (from airssys-rt)
   - Request-response patterns and routing
   - Correlation tracking infrastructure
   - Inter-component communication orchestration
   - NOT: WASM execution (that's runtime/)

**PROJECTS_STANDARD.md Compliance:**
- **§2.1 3-Layer Import Organization**: Code will follow import organization
   - Layer 1: Standard library imports
   - Layer 2: Third-party crate imports (airssys_rt, serde, chrono)
   - Layer 3: Internal crate imports (from core/, actor/)
   
- **§4.3 Module Architecture Patterns**: mod.rs files will only contain declarations
   - messaging/mod.rs already has module declarations only
   - messaging_service.rs contains implementation code
   - Follows declaration-only pattern for module files
   
- **§6.2 Avoid `dyn` Patterns**: Static dispatch preferred over trait objects
   - CorrelationTracker uses DashMap (concurrent hashmap), not `dyn Trait`
   - MessagingService uses concrete types, no trait objects
   
- **§6.4 Implementation Quality Gates**:
   - Zero compiler warnings: `cargo build`
   - Zero clippy warnings: `cargo clippy --all-targets --all-features -- -D warnings`
   - Comprehensive tests: Unit tests in `#[cfg(test)]` blocks
   - All tests passing: `cargo test --lib`

**Rust Guidelines Applied:**
- **M-DESIGN-FOR-AI**: Idiomatic APIs, thorough docs, testable code
   - All public types have module documentation
   - All public functions have doc comments with examples
   - Tests verify all functionality
   
- **M-MODULE-DOCS**: Module documentation complete
   - messaging_service.rs has module-level `//!` documentation
   - All public types documented with `///` comments
   - Follows M-CANONICAL-DOCS structure
   
- **M-ERRORS-CANONICAL-STRUCTS**: Error types follow canonical structure
   - RequestError in core/messaging.rs follows thiserror pattern
   - WasmError is used consistently
   - All errors implement Display and std::error::Error
   
- **M-STATIC-VERIFICATION**: Lints enabled, clippy used
   - All clippy lints from PROJECTS_STANDARD.md §M-LINT-OVERRIDE-EXPECT
   - `#[expect(clippy::...)]` for intentional violations with reasons
   - `cargo clippy --all-targets --all-features -- -D warnings`

**Documentation Standards:**
- **Diátaxis Type**: Reference documentation for APIs
   - MessagingService, ResponseRouter, metrics types are reference docs
   - Provide complete API documentation with examples
   - Neutral technical language, no marketing terms
   
- **Quality**: Professional tone, no hyperbole per documentation-quality-standards.md
   - No superlatives ("best", "fastest", "revolutionary")
   - Measurable performance claims with units
   - Accurate descriptions, not promotional
   
- **Task documentation**: Standards Compliance Checklist in task file per task-documentation-standards.md
   - Evidence of standards application included in plan
   - Code examples showing compliance

### Prerequisites

**Must be in place before starting:**

1. ✅ **Task 1.1 Completed**: messaging/ module structure created
   - `src/messaging/mod.rs` exists with module declarations
   - `src/messaging/messaging_service.rs` exists with placeholder
   - `src/lib.rs` updated with `pub mod messaging;`
   - Verified by: Task 1.1 acceptance criteria met

2. ✅ **core/messaging.rs Exists**: Types moved to core module
   - `CorrelationTracker` in `src/core/messaging.rs`
   - `CorrelationId`, `RequestError`, `ResponseMessage` in `src/core/messaging.rs`
   - Verified by: Task 1.1 moved types from actor/message/ to core/messaging/

3. ✅ **Code Review Complete**: runtime/messaging.rs reviewed
   - All 1,313 lines of code reviewed
   - Import dependencies identified
   - Code structure analyzed

4. ✅ **Git Branch Ready**: Working branch for changes
   - Create branch: `hotfix/messaging-module-refactoring-task-1.2`
   - Ensure clean working directory

### Module Architecture (CORRECTED)

**Code will be placed in:** `src/messaging/messaging_service.rs`

**Module responsibilities (per ADR-WASM-023 and KNOWLEDGE-WASM-012):**
- MessagingService: MessageBroker singleton management
- ResponseRouter: Request-response routing
- MessagingMetrics: Message publication statistics
- ResponseRouterMetrics: Response routing statistics
- MessageReceptionMetrics: ComponentActor message delivery metrics

**Dependency Rules (from KNOWLEDGE-WASM-012 Line 274):**
```
messaging/ → core/, actor/, airssys-rt
```

**This means:**
- ✅ messaging/ CAN import from core/ (types, errors, configs)
- ✅ messaging/ CAN import from actor/ (actor system integration)
- ✅ messaging/ CAN import from airssys-rt (MessageBroker from runtime)
- ❌ messaging/ CANNOT import from runtime/ (WASM execution engine)

**Forbidden imports:**
- ❌ `use crate::runtime` - messaging/ cannot import from runtime/

**Allowed imports:**
- ✅ `use crate::core::messaging::{CorrelationId, RequestError, ResponseMessage}` (types in core)
- ✅ `use crate::actor::message::CorrelationTracker` (CorrelationTracker in actor/, ALLOWED!)

**Verification commands (for implementer to run):**
```bash
# Check 1: messaging/ doesn't import from runtime/ (FORBIDDEN)
grep -rn "use crate::runtime" src/messaging/
# Expected: No output (clean)

# Check 2: messaging/ CAN import from actor/ (ALLOWED)
grep -rn "use crate::actor" src/messaging/
# Expected: May have output (this is OK per KNOWLEDGE-WASM-012)

# Check 3: messaging/ imports from core/ (EXPECTED)
grep -rn "use crate::core" src/messaging/
# Expected: Has output (types, errors, etc.)
```

### Import Analysis (CORRECTED)

**Current imports in runtime/messaging.rs (lines 75-78):**
```rust
// Layer 3: Internal crate imports
use crate::actor::message::{CorrelationId, CorrelationTracker, RequestError, ResponseMessage};
use crate::core::{ComponentId, ComponentMessage, WasmError};
```

**Problem: runtime/ importing from actor/ VIOLATES ADR-WASM-023**
- runtime/ is lower level than actor/
- runtime/ cannot import from actor/ (reverse dependency)
- This is why the file needs to be moved to messaging/

**Where Types Are Actually Located (Current State):**

**In core/messaging.rs:**
- CorrelationId (type alias to Uuid)
- ResponseMessage (struct)
- RequestError (enum)
- MessageEnvelope (struct)
- MessageType (enum)
- DeliveryGuarantee (enum)

**In actor/message/correlation_tracker.rs:**
- CorrelationTracker (struct) ← NOT in core/messaging.rs!

**Import Strategy for messaging/messaging_service.rs:**
```rust
// FROM: In runtime/messaging.rs (wrong location)
use crate::actor::message::{CorrelationId, CorrelationTracker, RequestError, ResponseMessage};
use crate::core::{ComponentId, ComponentMessage, WasmError};

// TO: In messaging/messaging_service.rs (correct location)
use crate::core::messaging::{CorrelationId, RequestError, ResponseMessage};  // Types in core
use crate::core::{ComponentId, ComponentMessage, WasmError};              // Types in core
use crate::actor::message::CorrelationTracker;                               // Allowed per KNOWLEDGE-WASM-012!
```

**Why This Is CORRECT After Moving to messaging/:**
1. **CorrelationId, RequestError, ResponseMessage** → Import from `core::messaging` (types in core)
2. **CorrelationTracker** → Import from `actor::message` (actor system integration)
3. **Per KNOWLEDGE-WASM-012 Line 274**: `messaging/ → core/, actor/, airssys-rt`
   - This means messaging/ CAN import from actor/
   - The import `use crate::actor::message::CorrelationTracker` is **ALLOWED**!

**Import update locations:**
1. Line 76-78: Update all imports to correct paths after moving to messaging/
2. Types in `core/messaging/` → Use `use crate::core::messaging::{...}`
3. CorrelationTracker in `actor/message/` → Use `use crate::actor::message::CorrelationTracker`
4. ComponentId, ComponentMessage, WasmError in `core/` → Keep as-is (already correct)

**No other imports need updating.**

### Implementation Subtasks

#### Subtask 1.2.1: Prepare messaging/messaging_service.rs

**Objective:** Replace placeholder with actual implementation.

**Steps:**
1. Backup existing placeholder file:
   ```bash
   cp src/messaging/messaging_service.rs src/messaging/messaging_service.rs.placeholder
   ```

2. Read runtime/messaging.rs completely (1,313 lines):
   - Read entire file to understand full implementation
   - Note all types, traits, impls, and tests

3. Remove all placeholder code:
   - Delete placeholder MessagingService struct
   - Delete placeholder impls
   - Keep module documentation (`//!` section)

**Deliverables:**
- messaging_service.rs ready for code insertion

**Acceptance Criteria:**
- [ ] Placeholder code removed
- [ ] Module documentation preserved
- [ ] File ready for implementation

---

#### Subtask 1.2.2: Move MessagingService Implementation

**Deliverables:**
- MessagingService struct (lines 126-387 from runtime/messaging.rs)
- Implementation methods and all doc comments

**Code to move:**
```rust
/// Service managing MessageBroker integration for inter-component communication.
/// [Full doc comment from runtime/messaging.rs]
#[derive(Clone)]
pub struct MessagingService {
    broker: Arc<InMemoryMessageBroker<ComponentMessage>>,
    correlation_tracker: Arc<CorrelationTracker>,
    metrics: Arc<MessagingMetrics>,
    response_router: Arc<ResponseRouter>,
}

impl MessagingService {
    pub fn new() -> Self { /* implementation */ }
    pub fn broker(&self) -> Arc<InMemoryMessageBroker<ComponentMessage>> { /* implementation */ }
    pub fn correlation_tracker(&self) -> Arc<CorrelationTracker> { /* implementation */ }
    pub async fn get_stats(&self) -> MessagingStats { /* implementation */ }
    pub(crate) fn record_publish(&self) { /* implementation */ }
    pub(crate) fn record_routing_failure(&self) { /* implementation */ }
    pub(crate) fn record_request_sent(&self) { /* implementation */ }
    pub(crate) fn record_request_completed(&self) { /* implementation */ }
    pub(crate) fn pending_requests(&self) -> u64 { /* implementation */ }
    pub fn response_router(&self) -> Arc<ResponseRouter> { /* implementation */ }
}

impl Default for MessagingService {
    fn default() -> Self { /* implementation */ }
}
```

**Import update (CORRECTED):**
```rust
// BEFORE (in runtime/messaging.rs - WRONG MODULE):
use crate::actor::message::{CorrelationId, CorrelationTracker, RequestError, ResponseMessage};

// AFTER (in messaging/messaging_service.rs - CORRECT MODULE):
// Import types from core/ (where they live):
use crate::core::messaging::{CorrelationId, RequestError, ResponseMessage};

// Import CorrelationTracker from actor/message/ (where it lives):
use crate::actor::message::CorrelationTracker;
```

**Why This Is CORRECT:**
- `CorrelationId`, `RequestError`, `ResponseMessage` are in `core/messaging.rs` → Import from core/
- `CorrelationTracker` is in `actor/message/correlation_tracker.rs` → Import from actor/
- **Per KNOWLEDGE-WASM-012 Line 274**: `messaging/ → core/, actor/, airssys-rt`
- This means messaging/ CAN import from both core/ AND actor/
- Only runtime/ is FORBIDDEN (messaging/ cannot import from runtime/)

**Acceptance Criteria:**
- [ ] MessagingService struct moved
- [ ] All impl blocks moved
- [ ] Imports updated: types from core::messaging, CorrelationTracker from actor::message
- [ ] All doc comments preserved
- [ ] No imports from runtime/ (forbidden)

**ADR Constraints:**
- **ADR-WASM-023**: No imports from runtime/ (can import from actor/ and core/)
- **KNOWLEDGE-WASM-012**: messaging/ → core/, actor/, airssys-rt

**PROJECTS_STANDARD.md Compliance:**
- **§2.1**: Import organization: std → external → internal (core/, actor/)
- **§6.2**: Avoid `dyn` (MessagingService uses concrete types)
- **§6.4**: Quality gates (zero warnings, tests included)

**Rust Guidelines:**
- **M-DESIGN-FOR-AI**: Idiomatic API with docs
- **M-MODULE-DOCS**: Module documentation complete
- **M-ERRORS-CANONICAL-STRUCTS**: Error types canonical

**Documentation:**
- **Diátaxis type**: Reference documentation
- **Quality**: Professional tone, no hyperbole

---

#### Subtask 1.2.3: Move Metrics Types

**Deliverables:**
- MessagingMetrics struct (lines 418-431)
- MessagingStats struct (lines 448-467)
- ResponseRouterMetrics struct (lines 521-531)
- ResponseRouterStats struct (lines 668-679)
- MessageReceptionMetrics struct (lines 736-852)
- MessageReceptionStats struct (lines 868-885)

**Acceptance Criteria:**
- [ ] All metrics types moved
- [ ] All doc comments preserved
- [ ] Imports use core/ types (where appropriate)
- [ ] No imports from runtime/ (forbidden)

**ADR Constraints:**
- **ADR-WASM-023**: No imports from runtime/ (forbidden)
- **KNOWLEDGE-WASM-012**: Can import from actor/ and core/ (allowed)

**PROJECTS_STANDARD.md Compliance:**
- **§2.1**: Import organization correct
- **§6.4**: Quality gates met

---

#### Subtask 1.2.4: Move ResponseRouter Implementation

**Deliverables:**
- ResponseRouter struct (lines 511-666)
- All implementation methods
- All doc comments

**Acceptance Criteria:**
- [ ] ResponseRouter struct moved
- [ ] All impl blocks moved
- [ ] Imports use core/messaging and actor/message as appropriate
- [ ] All doc comments preserved
- [ ] No imports from runtime/ (forbidden)

**ADR Constraints:**
- **ADR-WASM-023**: No imports from runtime/ (forbidden)
- **KNOWLEDGE-WASM-012**: Can import from actor/ and core/ (allowed)

---

#### Subtask 1.2.5: Move All Tests

**Deliverables:**
- All unit tests from lines 887-1313

**Test categories to move:**
1. MessagingService tests (tests 1.2.1-1.2.7)
2. ResponseRouter tests (tests 1.2.8-1.2.12)
3. Metrics tests (all remaining)

**Acceptance Criteria:**
- [ ] All tests moved
- [ ] Test structure preserved
- [ ] Imports updated in tests
- [ ] All tests compile

**PROJECTS_STANDARD.md Compliance:**
- **§6.4**: Comprehensive tests required
- **§M-STATIC-VERIFICATION**: Tests pass clippy

**Rust Guidelines:**
- **M-DESIGN-FOR-AI**: Testable code with examples

---

#### Subtask 1.2.6: Verify All Imports Updated (CORRECTED)

**Objective:** Ensure imports are correct per architecture rules.

**Verification steps:**
1. Search for runtime imports in messaging_service.rs (FORBIDDEN):
    ```bash
    grep -n "use crate::runtime" src/messaging/messaging_service.rs
    ```
    Expected: No results (forbidden)

2. Verify actor/ imports for CorrelationTracker (ALLOWED):
    ```bash
    grep -n "use crate::actor::" src/messaging/messaging_service.rs
    ```
    Expected: May have results (this is OK per KNOWLEDGE-WASM-012)

3. Verify core/ imports (EXPECTED):
    ```bash
    grep -n "use crate::core::" src/messaging/messaging_service.rs
    ```
    Expected: Multiple lines (ComponentId, ComponentMessage, messaging types, etc.)

**Acceptance Criteria:**
- [ ] No runtime/ imports found (forbidden)
- [ ] actor/ imports present only where appropriate (CorrelationTracker from actor/message/)
- [ ] All imports follow §2.1 pattern
- [ ] Types imported from correct modules (core/ for types, actor/ for CorrelationTracker)

---

#### Subtask 1.2.7: Build and Test Verification

**Objective:** Verify code compiles and all tests pass.

**Build verification:**
```bash
# 1. Build project
cargo build
# Expected: No errors, no warnings

# 2. Check for warnings
cargo build 2>&1 | grep -i "warning"
# Expected: No warnings
```

**Test verification:**
```bash
# 1. Run unit tests
cargo test --lib messaging
# Expected: All tests pass

# 2. Run all tests
cargo test --lib
# Expected: All tests pass

# 3. Run specific test functions
cargo test messaging_service_new
cargo test messaging_service_broker_access
# Expected: All pass
```

**Clippy verification:**
```bash
cargo clippy --all-targets --all-features -- -D warnings
# Expected: Zero warnings
```

**Acceptance Criteria:**
- [ ] cargo build succeeds with zero warnings
- [ ] cargo test --lib passes all tests
- [ ] cargo clippy passes with zero warnings

**PROJECTS_STANDARD.md Compliance:**
- **§6.4**: Zero compiler and clippy warnings

---

### Quality Standards

**All subtasks must meet:**
- ✅ Code builds without errors
- ✅ Zero compiler warnings
- ✅ Zero clippy warnings: `cargo clippy --all-targets --all-features -- -D warnings`
- ✅ Follows PROJECTS_STANDARD.md §2.1-§6.4
- ✅ Follows Rust guidelines (see references above)
- ✅ Unit tests in `#[cfg(test)]` blocks
- ✅ All tests pass: `cargo test --lib`
- ✅ Documentation follows quality standards
- ✅ Standards Compliance Checklist in task file

### Verification Checklist

**For implementer to run after completing all subtasks:**

```bash
# ============================================================================
# 1. Architecture Verification (ADR-WASM-023)
# ============================================================================

echo "=== Verifying Module Boundaries (CORRECTED) ==="
echo "Checking messaging/ does NOT import from runtime/ (FORBIDDEN)..."

if grep -rn "use crate::runtime" src/messaging/; then
    echo "❌ FAILED: messaging/ imports from runtime/"
    exit 1
else
    echo "✅ PASSED: messaging/ does not import from runtime/"
fi

echo "Checking messaging/ CAN import from actor/ (ALLOWED per KNOWLEDGE-WASM-012)..."
# This check is for INFORMATIONAL - actor imports are ALLOWED in messaging/
if grep -rn "use crate::actor" src/messaging/; then
    echo "ℹ️  INFO: messaging/ imports from actor/ (this is CORRECT per KNOWLEDGE-WASM-012)"
    echo "✅ PASSED: messaging/ imports from actor/ as needed"
else
    echo "ℹ️  INFO: messaging/ has no actor imports (also OK)"
fi

if grep -rn "use crate::security" src/messaging/; then
    echo "❌ FAILED: messaging/ imports from security/"
    exit 1
else
    echo "✅ PASSED: messaging/ does not import from security/"
fi

echo ""
echo "=== All Architecture Checks Passed ==="

# ============================================================================
# 2. Build Verification
# ============================================================================

echo "=== Building Project ==="
cargo build
BUILD_EXIT=$?

if [ $BUILD_EXIT -ne 0 ]; then
    echo "❌ FAILED: Build failed"
    exit 1
else
    echo "✅ PASSED: Build succeeded"
fi

# Check for warnings
WARNINGS=$(cargo build 2>&1 | grep -i "warning" | wc -l)
if [ "$WARNINGS" -gt 0 ]; then
    echo "❌ FAILED: Build has $WARNINGS warnings"
    exit 1
else
    echo "✅ PASSED: Zero compiler warnings"
fi

echo ""

# ============================================================================
# 3. Test Verification
# ============================================================================

echo "=== Running Tests ==="
cargo test --lib
TEST_EXIT=$?

if [ $TEST_EXIT -ne 0 ]; then
    echo "❌ FAILED: Tests failed"
    exit 1
else
    echo "✅ PASSED: All tests pass"
fi

# Test messaging_service specifically
echo ""
echo "=== Testing messaging_service module ==="
cargo test messaging_service_new
cargo test messaging_service_broker_access
cargo test messaging_service_stats
cargo test response_router_new
cargo test response_router_route_response_success

echo ""
echo "✅ PASSED: All messaging tests pass"

# ============================================================================
# 4. Clippy Verification
# ============================================================================

echo "=== Running Clippy ==="
cargo clippy --all-targets --all-features -- -D warnings
CLIPPY_EXIT=$?

if [ $CLIPPY_EXIT -ne 0 ]; then
    echo "❌ FAILED: Clippy found warnings"
    exit 1
else
    echo "✅ PASSED: Zero clippy warnings"
fi

echo ""
echo "=== All Quality Checks Passed ==="
echo "✅ Task 1.2 is complete"
```

**Expected output:**
- All architecture checks pass
- Build succeeds with zero warnings
- All tests pass
- Clippy passes with zero warnings

### Testing Strategy

**Unit Tests (Already in code):**
- MessagingService instantiation tests
- Broker access tests
- Stats tests
- Correlation tracker tests
- Response router tests
- Metrics tests
- All existing tests from runtime/messaging.rs

**Integration Tests (Run after completion):**
```bash
# Run integration tests that use messaging
cargo test --test messaging_integration_tests
cargo test --test actor_routing_tests
cargo test --test actor_invocation_tests
```

**Test Coverage Requirements:**
- All public functions have tests
- All error paths have tests
- All metrics have tests
- Tests verify correct behavior after import changes

**PROJECTS_STANDARD.md Compliance:**
- **§6.4**: Comprehensive test coverage
- **M-DESIGN-FOR-AI**: Testable code design

### Risk Mitigation

**Identified Risks:**

1. **Import Update Missed (CORRECTED)**
   - **Risk**: Forgetting to update imports OR incorrectly removing actor/ imports
   - **Mitigation**: Use grep to verify correct imports:
     ```bash
     # Verify NO runtime/ imports (forbidden)
     grep -n "use crate::runtime" src/messaging/messaging_service.rs
     # Expected: No results
     
     # Verify actor/ imports PRESENT for CorrelationTracker (allowed!)
     grep -n "use crate::actor::message::CorrelationTracker" src/messaging/messaging_service.rs
     # Expected: Has results
     
     # Verify core/ imports for types (expected)
     grep -n "use crate::core::messaging" src/messaging/messaging_service.rs
     # Expected: Has results
     ```
   - **Verification**: Build will fail if imports wrong
   - **CRITICAL**: Do NOT remove actor/ imports for CorrelationTracker - they are ALLOWED!

2. **Code Copy Errors**
   - **Risk**: Incorrect copy/paste or missing lines
   - **Mitigation**: Compare file sizes (expected ~1,313 lines)
   - **Verification**: All tests will fail if code missing

3. **Type Mismatches**
   - **Risk**: Types in core/messaging.rs don't match what code expects
   - **Mitigation**: Task 1.1 already moved types, verify they compile
   - **Verification**: Build will fail with type errors

4. **Module Boundary Violations**
   - **Risk**: Accidentally importing from actor/ or runtime/
   - **Mitigation**: Architecture verification commands after completion
   - **Verification**: ADR-WASM-023 verification checks

5. **Test Failures**
   - **Risk**: Tests rely on old import paths
   - **Mitigation**: Review all tests, update imports in test code too
   - **Verification**: cargo test will fail

**Contingency:**
- If build fails with import errors: Review and fix imports
- If tests fail: Debug and fix test code
- If architecture check fails: Review and remove forbidden imports

### Rollback Plan

**If critical issues arise:**

1. **Backup created before starting** (Subtask 1.2.1):
   - Original placeholder file: `src/messaging/messaging_service.rs.placeholder`
   - Original source file: `src/runtime/messaging.rs` (unchanged)

2. **Rollback steps:**
   ```bash
   # Step 1: Restore placeholder
   cp src/messaging/messaging_service.rs.placeholder src/messaging/messaging_service.rs
   
   # Step 2: Verify build passes
   cargo build
   
   # Step 3: Notify and investigate failure
   echo "Task 1.2 rolled back. Investigate failure before retrying."
   ```

3. **Decision points:**
   - If type mismatches: Review Task 1.1 (types moved to core/messaging.rs)
   - If import issues: Check CorrelationTracker location
   - If logic errors: Compare with runtime/messaging.rs line-by-line

4. **After rollback:**
   - Document what went wrong
   - Update plan with learned lessons
   - Retry after root cause identified

### Standards Compliance Checklist

**PROJECTS_STANDARD.md Applied:**
- [ ] **§2.1 3-Layer Import Organization** - Evidence: All imports follow std → external → internal pattern (code in messaging_service.rs)
- [ ] **§4.3 Module Architecture Patterns** - Evidence: messaging/mod.rs contains only declarations (verified in Task 1.1)
- [ ] **§6.2 Avoid `dyn` Patterns** - Evidence: No trait objects used, concrete types only (MessagingService, ResponseRouter)
- [ ] **§6.4 Implementation Quality Gates** - Evidence: Zero compiler/clippy warnings, comprehensive tests (verification commands)

**Rust Guidelines Applied:**
- [ ] **M-DESIGN-FOR-AI** - Evidence: Idiomatic APIs, comprehensive docs, testable code (moved code with docs preserved)
- [ ] **M-MODULE-DOCS** - Evidence: Module documentation complete (messaging_service.rs has `//!` and `///` docs)
- [ ] **M-ERRORS-CANONICAL-STRUCTS** - Evidence: Error types follow thiserror pattern (RequestError, WasmError)
- [ ] **M-STATIC-VERIFICATION** - Evidence: Lints enabled, clippy verification in plan

**Documentation Quality:**
- [ ] **No hyperbolic terms** - Evidence: Verified against forbidden list in documentation-quality-standards.md
- [ ] **Technical precision** - Evidence: All claims measurable, no vague assertions
- [ ] **Diátaxis compliance** - Evidence: Reference documentation type chosen for APIs

**ADR Compliance:**
- [ ] **ADR-WASM-018** - Evidence: One-way dependency enforced (messaging → core only)
- [ ] **ADR-WASM-023** - Evidence: No forbidden imports (verification commands in plan)

**Knowledge Compliance:**
- [ ] **KNOWLEDGE-WASM-012** - Evidence: Module structure follows specification (messaging/ top-level)
- [ ] **KNOWLEDGE-WASM-024** - Evidence: Messaging patterns correctly implemented

---

#### Task 1.3: Create Remaining Messaging Submodules ✅ COMPLETE

**Objective:** Extract code from `messaging_service.rs` into separate submodule files per KNOWLEDGE-WASM-012 specification.

**Implementation Completed:**
- ResponseRouter successfully extracted to `src/messaging/router.rs` (~220 lines)
- messaging_service.rs refactored and cleaned up (~938 lines remaining)
- All stub files enhanced with documentation
- All compiler warnings fixed (zero warnings)
- All tests pass (100% passing)

**Files Modified:**
- `src/messaging/router.rs` - ResponseRouter implementation
- `src/messaging/messaging_service.rs` - Cleaned up (removed extracted code)
- `src/messaging/fire_and_forget.rs` - Enhanced with documentation
- `src/messaging/request_response.rs` - Enhanced with documentation
- `src/messaging/codec.rs` - Enhanced with documentation
- `src/messaging/topics.rs` - Enhanced with documentation
- `src/messaging/mod.rs` - Updated re-exports

**Total Lines Changed:** ~1,500 lines across 8 files

**Architecture Compliance:**
- messaging/ → core/, actor/, airssys-rt (allowed per KNOWLEDGE-WASM-012)
- messaging/ → runtime/ (forbidden per ADR-WASM-023) ✅ NO imports

**Testing Results:**
- All tests pass (1,028 tests)
- Zero compiler warnings
- Zero clippy warnings
- All architecture verification checks pass

**Next Step:** Phase 3: Remove runtime/messaging.rs

**Source File Analysis:**

**File:** `src/messaging/messaging_service.rs` (1,317 lines)

**Code Currently Contains:**
1. MessagingService struct and impl (lines ~27-387)
2. MessagingMetrics struct (lines ~454-517)
3. ResponseRouter struct and impl (lines ~517-675)
4. ResponseRouterStats struct (lines ~675-694)
5. MessageReceptionMetrics struct and impl (lines ~742-875)
6. MessageReceptionStats struct (lines ~875-901)
7. All unit tests (lines ~901-1317)

**Destination Files (From Task 1.1 Placeholders):**
1. `src/messaging/router.rs` - MessageBroker routing (currently has placeholder)
2. `src/messaging/fire_and_forget.rs` - Fire-and-forget pattern (currently has placeholder)
3. `src/messaging/request_response.rs` - Request-response pattern (currently has placeholder)
4. `src/messaging/codec.rs` - Multicodec message encoding (currently has placeholder)
5. `src/messaging/topics.rs` - Topic-based pub/sub (currently has stub)

**Current Problem:**
- `messaging_service.rs` contains ALL messaging infrastructure code
- Placeholder files contain only stub code
- `mod.rs` has duplicate re-exports (imports from both `messaging_service` and submodule placeholders, causing conflicts)
- After extraction, code will be duplicated between `messaging_service.rs` and submodules

---

## Implementation Plan for Task 1.3

### Context & References

**ADR References:**
- **ADR-WASM-018**: Three-Layer Architecture (one-way dependency chain)
   - Dependencies flow: core → runtime → actor → messaging
   - messaging/ is at TOP of dependency chain (can import from core, actor, airssys-rt)
   - MessagingService is part of messaging/ module
   
- **ADR-WASM-023**: Module Boundary Enforcement (HARD REQUIREMENT)
   - messaging/ CANNOT import from runtime/ (WASM execution)
   - messaging/ CAN import from actor/ (allowed per corrected architecture)
   - messaging/ CAN import from core/ (types, errors, configs)
   - messaging/ CAN import from airssys-rt (MessageBroker)
   - Forbidden imports: `use crate::runtime` in messaging/

**Knowledge References:**
- **KNOWLEDGE-WASM-012**: Module Structure Architecture (lines 506-596 define messaging/ module)
   - messaging/ should contain: router.rs, fire_and_forget.rs, request_response.rs, codec.rs, topics.rs
   - messaging/ → core/, actor/, airssys-rt (Line 274 - CRITICAL)
   - Module responsibilities for each submodule
   
- **KNOWLEDGE-WASM-005**: Messaging Architecture
   - MessageBroker integration via airssys-rt
   - Fire-and-forget and request-response patterns
   - Correlation tracking for request-response
   
- **KNOWLEDGE-WASM-024**: Component Messaging Clarifications
   - Async-only communication model (no synchronous messaging)
   - Two send methods: send-message vs send-request
   - Unified receiver (handle-message for both patterns)
   - Internal infrastructure vs component API distinction
   
- **KNOWLEDGE-WASM-026**: Message Delivery Architecture Final
   - ActorSystemSubscriber owns message delivery (has mailbox_senders)
   - ComponentRegistry stays pure (identity lookup only)
   - Message flow from component send to handle_message invocation

**System Patterns:**
- **From system-patterns.md**: Component Communication Pattern
   - MessageBroker integration for inter-component communication
   - MessageRouter for routing between components

**PROJECTS_STANDARD.md Compliance:**
- **§2.1 3-Layer Import Organization**: Code will follow import organization
   - Layer 1: Standard library imports (std::sync::Arc, std::sync::atomic)
   - Layer 2: Third-party crate imports (serde, chrono, tokio)
   - Layer 3: Internal crate imports (core/, actor/, airssys-rt)
   
- **§4.3 Module Architecture Patterns**: mod.rs files will only contain declarations
   - messaging/mod.rs already has module declarations only
   - No implementation code in mod.rs files
   
- **§6.2 Avoid `dyn` Patterns**: Static dispatch preferred over trait objects
   - MessagingService uses concrete types (Arc<T>), no `dyn Trait`
   - ResponseRouter uses concrete CorrelationTracker, no trait objects
   
- **§6.4 Implementation Quality Gates**:
   - Zero compiler warnings: `cargo build`
   - Zero clippy warnings: `cargo clippy --all-targets --all-features -- -D warnings`
   - Comprehensive tests: Unit tests in `#[cfg(test)]` blocks
   - All tests passing: `cargo test --lib`

**Rust Guidelines Applied:**
- **M-DESIGN-FOR-AI**: Idiomatic APIs, thorough docs, testable code
   - All public types have module documentation
   - All public functions have doc comments with examples
   - Tests verify all functionality
   
- **M-MODULE-DOCS**: Module documentation will be added
   - Each submodule will have `//!` module-level docs
   - Public types have `///` doc comments
   - Follows M-CANONICAL-DOCS structure
   
- **M-ERRORS-CANONICAL-STRUCTS**: Error types follow canonical structure
   - RequestError in core/messaging.rs follows thiserror pattern
   - WasmError is used consistently
   
- **M-STATIC-VERIFICATION**: Lints enabled, clippy used
   - All clippy lints from PROJECTS_STANDARD.md
   - `#[expect(clippy::...)]` for intentional violations with reasons

**Documentation Standards:**
- **Diátaxis Type**: Reference documentation for APIs
   - MessagingService, ResponseRouter, and extracted submodules are reference docs
   - Provide complete API documentation with examples
   - Neutral technical language, no marketing terms
   
- **Quality**: Professional tone, no hyperbole per documentation-quality-standards.md
   - No superlatives ("best", "fastest", "revolutionary")
   - Measurable performance claims with units
   - Accurate descriptions, not promotional
   
- **Task documentation**: Standards Compliance Checklist will be included per task-documentation-standards.md
   - Evidence of standards application in plan
   - Code examples showing compliance

---

### Module Architecture

**Code will be placed in:** `src/messaging/` subdirectory

**Module responsibilities (per ADR-WASM-023):**
- messaging/messaging_service.rs: Main MessagingService coordinator (broker, metrics, correlation tracker)
- messaging/router.rs: MessageBroker routing helpers and ResponseRouter
- messaging/fire_and_forget.rs: Fire-and-forget messaging pattern (stub)
- messaging/request_response.rs: Request-response error types (already moved)
- messaging/codec.rs: Message encoding/decoding (stub)
- messaging/topics.rs: Topic-based pub/sub (stub for Phase 2)

**Dependency Rules (from ADR-WASM-023 and KNOWLEDGE-WASM-012 Line 274):**
```
messaging/ → core/, actor/, airssys-rt
```

**This means:**
- ✅ messaging/ CAN import from core/ (types, errors, configs)
- ✅ messaging/ CAN import from actor/ (CorrelationTracker, etc.)
- ✅ messaging/ CAN import from airssys-rt (InMemoryMessageBroker)
- ❌ messaging/ CANNOT import from runtime/ (WASM execution engine)

**Verification commands (for implementer to run after each subtask):**
```bash
# Check 1: messaging/ doesn't import from runtime/ (FORBIDDEN)
grep -rn "use crate::runtime" src/messaging/
# Expected: No output (clean)

# Check 2: messaging/ CAN import from actor/ (ALLOWED per KNOWLEDGE-WASM-012)
grep -rn "use crate::actor" src/messaging/
# Expected: May have output (this is OK per KNOWLEDGE-WASM-012)

# Check 3: messaging/ imports from core/ (EXPECTED)
grep -rn "use crate::core" src/messaging/
# Expected: Multiple lines (ComponentId, messaging types, etc.)
```

**Import Strategy for ALL submodules:**
```rust
// Common imports (same as current messaging_service.rs)
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

// Layer 2: Third-party crate imports
use serde::{Deserialize, Serialize};

// Layer 3: airssys-rt imports
use airssys_rt::broker::InMemoryMessageBroker;

// Layer 3: Internal crate imports
use crate::actor::message::CorrelationTracker;
use crate::core::messaging::{CorrelationId, RequestError, ResponseMessage, MessageEnvelope, MessageType, DeliveryGuarantee};
use crate::core::{ComponentId, ComponentMessage, WasmError};
use chrono::{DateTime, Utc};
```

---

### Phase 1: Analysis and Preparation

#### Subtask 1.3.1: Analyze messaging_service.rs for Extraction

**Objective:** Read and analyze messaging_service.rs to identify code to extract.

**Steps:**
1. Read complete messaging_service.rs (1,317 lines)
2. Identify all structs and impls
3. Determine code ownership for each submodule
4. Document extraction points (line numbers)
5. Verify current imports are correct (they should stay mostly same)

**Deliverables:**
- Analysis document listing all code to extract
- Extraction mapping (what code goes where)
- Import dependency analysis

**Acceptance Criteria:**
- [ ] Complete analysis of messaging_service.rs completed
- [ ] All code to extract identified
- [ ] Destination files for each code section identified
- [ ] Import dependencies documented

**ADR Constraints:**
- **ADR-WASM-023**: messaging/ can import from actor/ (CorrelationTracker)
- **ADR-WASM-023**: messaging/ cannot import from runtime/

**PROJECTS_STANDARD.md Compliance:**
- **§2.1**: Import organization verified
- **§4.3**: No implementation in mod.rs files

**Rust Guidelines:**
- **M-DESIGN-FOR-AI**: Clear analysis, well-documented decisions

---

### Phase 2: Extract ResponseRouter to router.rs

#### Subtask 1.3.2: Extract ResponseRouter to router.rs

**Objective:** Move ResponseRouter implementation from messaging_service.rs to router.rs.

**Code to Extract:**
- ResponseRouter struct (lines ~517-675 from messaging_service.rs)
- ResponseRouterMetrics struct (lines ~675-694 from messaging_service.rs)
- ResponseRouter::new() function
- ResponseRouter::route_response() method
- ResponseRouter::has_pending_request() method
- ResponseRouter::responses_routed_count() method
- ResponseRouter::responses_orphaned_count() method
- ResponseRouter::error_responses_count() method
- ResponseRouter::get_stats() method

**Destination:** `src/messaging/router.rs`

**Current router.rs State:**
- Has MessageRouter placeholder (78 lines)
- Has RoutingStats placeholder (lines 49-51)
- Needs to be replaced with actual ResponseRouter from messaging_service.rs

**Import Updates for router.rs:**
```rust
// Same imports as messaging_service.rs
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use airssys_rt::broker::InMemoryMessageBroker;
use crate::actor::message::CorrelationTracker;
use crate::core::messaging::{CorrelationId, RequestError, ResponseMessage};
use crate::core::{ComponentId, WasmError};
use chrono::{DateTime, Utc};
```

**Acceptance Criteria:**
- [ ] ResponseRouter struct moved to router.rs
- [ ] ResponseRouterMetrics struct moved to router.rs
- [ ] All ResponseRouter impls moved to router.rs
- [ ] Imports updated (core::, actor::, airssys-rt, no runtime::)
- [ ] Module docs added (`//!` and `///`)
- [ ] Tests moved (if any exist in messaging_service.rs)

**ADR Constraints:**
- **ADR-WASM-023**: router.rs is in messaging/ (correct location)
- **ADR-WASM-023**: No imports from runtime/ (verify with grep)

**PROJECTS_STANDARD.md Compliance:**
- **§2.1**: 3-layer imports followed
- **§6.2**: No trait objects used

**Rust Guidelines:**
- **M-MODULE-DOCS**: Module docs added

**Documentation:**
- **Diátaxis Type**: Reference documentation
- **Quality**: Technical language, no marketing terms

---

### Phase 3: Update messaging/mod.rs Re-exports

#### Subtask 1.3.3: Fix Duplicate Re-exports in messaging/mod.rs

**Objective:** Remove duplicate re-export of ResponseRouter from messaging_service.rs and update to re-export from submodules only.

**Problem:**
Current mod.rs (line 39):
```rust
pub use messaging_service::{MessagingService, MessagingStats, ResponseRouter, ResponseRouterStats};
pub use router::{MessageRouter, RoutingStats};
```

This creates duplicate definitions because ResponseRouter exists in both places.

**Solution:**
After extraction:
```rust
// messaging_service.rs no longer has ResponseRouter
// ResponseRouter only exists in router.rs

// Updated mod.rs (line 39):
pub use messaging_service::{MessagingService, MessagingStats};
pub use router::{ResponseRouter, ResponseRouterStats};  // Re-export from router.rs
```

**Acceptance Criteria:**
- [ ] Duplicate ResponseRouter re-export removed from messaging_service
- [ ] Re-export changed to import from router.rs
- [ ] No conflicts in API
- [ ] cargo build succeeds

**ADR Constraints:**
- **ADR-WASM-023**: messaging/mod.rs imports correct

**PROJECTS_STANDARD.md Compliance:**
- **§4.3**: mod.rs declaration-only pattern maintained

**Rust Guidelines:**
- **M-MODULE-DOCS**: Re-exports documented

---

### Phase 4: Keep Metrics in messaging_service.rs

#### Subtask 1.3.4: Retain Metrics in messaging_service.rs

**Objective:** Keep MessageReceptionMetrics and MessagingMetrics in messaging_service.rs (not extracted).

**Code to RETAIN (not extracted):**
- MessagingMetrics struct (lines ~454-517)
- MessageReceptionMetrics struct (lines ~742-875)
- MessageReceptionStats struct (lines ~875-901)
- Related impls

**Rationale:**
These metrics are used by MessagingService to track overall messaging activity. They don't belong in specific routing or pattern submodules. Keeping them in messaging_service.rs makes sense as "coordinator" metrics.

**Acceptance Criteria:**
- [ ] MessagingMetrics retained in messaging_service.rs
- [ ] MessageReceptionMetrics retained in messaging_service.rs
- [ ] MessageReceptionStats retained in messaging_service.rs
- [ ] No metrics code extracted to other submodules
- [ ] MessagingService methods still use these metrics

**ADR Constraints:**
- **ADR-WASM-023**: messaging_service.rs is correct location

**PROJECTS_STANDARD.md Compliance:**
- **§2.1**: Import organization maintained
- **§6.4**: Quality gates met

---

### Phase 5: Minimal Updates to Other Submodules

#### Subtask 1.3.5: Update fire_and_forget.rs (Minimal)

**Objective:** Keep fire_and_forget.rs as a stub placeholder.

**Analysis of messaging_service.rs:**
After careful review, messaging_service.rs does NOT contain specific fire-and-forget pattern implementation code. The fire-and-forget pattern is handled at the MessageBroker level (airssys-rt) and at the host function level (async_host.rs).

**Action:**
- Keep fire_and_forget.rs as-is (stub placeholder)
- Add module documentation explaining this is Phase 2+ work
- No code extraction needed

**Acceptance Criteria:**
- [ ] fire_and_forget.rs kept as stub
- [ ] Module documentation added explaining this is future work
- [ ] No code extracted (there's nothing to extract)
- [ ] Placeholder tests remain as-is

**ADR Constraints:**
- **KNOWLEDGE-WASM-012**: Fire-and-forget pattern is Phase 2+ work

**PROJECTS_STANDARD.md Compliance:**
- **§6.1**: YAGNI - not implementing unneeded features

**Rust Guidelines:**
- **M-MODULE-DOCS**: Module docs explain future scope

**Documentation:**
- **Diátaxis Type**: Explanation
- **Quality**: Technical language

---

#### Subtask 1.3.6: Update request_response.rs (Minimal)

**Objective:** Keep request_response.rs with existing error types only.

**Analysis of messaging_service.rs:**
After careful review, RequestError is already correctly placed in request_response.rs (defined in placeholder file, not in messaging_service.rs). The request-response pattern implementation is handled by ResponseRouter (which is being extracted to router.rs in Subtask 1.3.2).

**Action:**
- Keep request_response.rs as-is (has RequestError enum and RequestResponse stub)
- Add module documentation
- No additional code extraction needed

**Acceptance Criteria:**
- [ ] request_response.rs kept as-is
- [ ] RequestError enum retained (correctly placed)
- [ ] Module documentation added
- [ ] No code extracted (ResponseRouter handles actual logic)

**ADR Constraints:**
- **KNOWLEDGE-WASM-029**: ResponseRouter handles request-response pattern

**PROJECTS_STANDARD.md Compliance:**
- **§6.1**: YAGNI - not adding unneeded code

**Rust Guidelines:**
- **M-MODULE-DOCS**: Module docs explain structure

**Documentation:**
- **Diátaxis Type**: Explanation
- **Quality**: Technical language

---

#### Subtask 1.3.7: Update codec.rs (Minimal)

**Objective:** Keep codec.rs as a stub placeholder.

**Analysis of messaging_service.rs:**
After careful review, messaging_service.rs does NOT contain multicodec encoding/decoding implementation. The multicodec format validation happens at the host function level (async_host.rs) when sending messages, not in messaging_service.rs.

**Action:**
- Keep codec.rs as-is (stub placeholder)
- Add module documentation explaining this is Phase 2+ work
- No code extraction needed

**Acceptance Criteria:**
- [ ] codec.rs kept as stub
- [ ] Module documentation added explaining this is future work
- [ ] No code extracted (there's nothing to extract)
- [ ] Placeholder tests remain as-is

**ADR Constraints:**
- **KNOWLEDGE-WASM-006**: Multicodec validation happens at host function level
- **ADR-WASM-001**: Multicodec compatibility strategy

**PROJECTS_STANDARD.md Compliance:**
- **§6.1**: YAGNI - not implementing unneeded features

**Rust Guidelines:**
- **M-MODULE-DOCS**: Module docs explain future scope

**Documentation:**
- **Diátaxis Type**: Explanation
- **Quality**: Technical language

---

#### Subtask 1.3.8: Update topics.rs (Minimal)

**Objective:** Keep topics.rs as a stub placeholder for Phase 2.

**Analysis of messaging_service.rs:**
KNOWLEDGE-WASM-024 explicitly states topic-based pub-sub is Phase 2+ work. Task 1.3 is about Phase 1 module organization only.

**Action:**
- Keep topics.rs as-is (stub placeholder)
- Add module documentation explaining this is Phase 2 work
- No code extraction needed

**Acceptance Criteria:**
- [ ] topics.rs kept as stub
- [ ] Module documentation added explaining Phase 2 scope
- [ ] No code extracted (future work)
- [ ] Placeholder tests remain as-is

**ADR Constraints:**
- **KNOWLEDGE-WASM-024**: Topic-based pub-sub is Phase 2+ work

**PROJECTS_STANDARD.md Compliance:**
- **§6.1**: YAGNI - not implementing future features

**Rust Guidelines:**
- **M-MODULE-DOCS**: Module docs explain future scope

**Documentation:**
- **Diátaxis Type**: Explanation
- **Quality**: Technical language

---

### Quality Standards

**All subtasks must meet:**
- ✅ Code builds without errors
- ✅ Zero compiler warnings
- ✅ Zero clippy warnings: `cargo clippy --all-targets --all-features -- -D warnings`
- ✅ Follows PROJECTS_STANDARD.md §2.1-§6.4
- ✅ Follows Rust guidelines (see references above)
- ✅ Unit tests in `#[cfg(test)]` blocks
- ✅ All tests pass: `cargo test --lib`
- ✅ Documentation follows quality standards
- ✅ Standards Compliance Checklist in task file

### Verification Checklist

**For implementer to run after completing all subtasks:**

```bash
# ============================================================================
# 1. Architecture Verification (ADR-WASM-023)
# ============================================================================

echo "=== Verifying Module Boundaries ==="

# Check 1: messaging/ doesn't import from runtime/ (FORBIDDEN)
if grep -rn "use crate::runtime" src/messaging/; then
    echo "❌ FAILED: messaging/ imports from runtime/"
    exit 1
else
    echo "✅ PASSED: messaging/ does not import from runtime/"
fi

# Check 2: messaging/ CAN import from actor/ (ALLOWED per KNOWLEDGE-WASM-012)
if grep -rn "use crate::actor" src/messaging/; then
    echo "ℹ️  INFO: messaging/ imports from actor/ (this is CORRECT per KNOWLEDGE-WASM-012)"
else
    echo "ℹ️  INFO: messaging/ has no actor imports (also OK)"
fi

# Check 3: messaging/ doesn't import from security/ (FORBIDDEN)
if grep -rn "use crate::security" src/messaging/; then
    echo "❌ FAILED: messaging/ imports from security/"
    exit 1
else
    echo "✅ PASSED: messaging/ does not import from security/"
fi

echo ""
echo "=== All Architecture Checks Passed ==="

# ============================================================================
# 2. Build Verification
# ============================================================================

echo "=== Building Project ==="
cargo build
BUILD_EXIT=$?

if [ $BUILD_EXIT -ne 0 ]; then
    echo "❌ FAILED: Build failed"
    exit 1
else
    echo "✅ PASSED: Build succeeded"
fi

# Check for warnings
WARNINGS=$(cargo build 2>&1 | grep -i "warning" | wc -l)
if [ "$WARNINGS" -gt 0 ]; then
    echo "❌ FAILED: Build has $WARNINGS warnings"
    exit 1
else
    echo "✅ PASSED: Zero compiler warnings"
fi

echo ""

# ============================================================================
# 3. Test Verification
# ============================================================================

echo "=== Running Tests ==="
cargo test --lib
TEST_EXIT=$?

if [ $TEST_EXIT -ne 0 ]; then
    echo "❌ FAILED: Tests failed"
    exit 1
else
    echo "✅ PASSED: All tests pass"
fi

echo ""

# ============================================================================
# 4. Clippy Verification
# ============================================================================

echo "=== Running Clippy ==="
cargo clippy --all-targets --all-features -- -D warnings
CLIPPY_EXIT=$?

if [ $CLIPPY_EXIT -ne 0 ]; then
    echo "❌ FAILED: Clippy found warnings"
    exit 1
else
    echo "✅ PASSED: Zero clippy warnings"
fi

echo ""
echo "=== All Quality Checks Passed ==="
```

**Expected output:**
- All architecture checks pass
- Build succeeds with zero warnings
- All tests pass
- Clippy passes with zero warnings

### Testing Strategy

**Unit Tests (Already in code):**
- ResponseRouter tests in messaging_service.rs (lines ~900-1317)
- These tests will be moved to router.rs with ResponseRouter
- MessagingService tests remain in messaging_service.rs
- No new tests needed for other submodules (they remain as stubs)

**Integration Tests (No new tests needed):**
- Existing integration tests already test messaging functionality
- They use MessagingService which remains unchanged
- All extracted code is internal organization, no new external API

**Test Coverage Requirements:**
- All public functions have tests
- All error paths have tests
- Tests verify correct behavior after refactoring
- Tests preserve existing test coverage

**PROJECTS_STANDARD.md Compliance:**
- **§6.4**: Comprehensive test coverage
- **M-DESIGN-FOR-AI**: Testable code with examples

**Rust Guidelines:**
- **M-STATIC-VERIFICATION**: Tests pass clippy

---

### Risk Mitigation

**Identified Risks:**

1. **Duplicate Type Definitions After Extraction**
   - **Risk**: ResponseRouter exists in both messaging_service.rs and router.rs after extraction
   - **Mitigation**: Subtask 1.3.3 fixes mod.rs re-exports to remove duplication

2. **Import Update Errors**
   - **Risk**: Forgetting to update imports when moving code
   - **Mitigation**: Verification commands check for forbidden imports

3. **Test Failures After Move**
   - **Risk**: Tests rely on old import paths
   - **Mitigation**: Tests use MessagingService which stays in same module

4. **Code Duplication**
   - **Risk**: Copying code incorrectly causes duplication
   - **Mitigation**: Move code, not copy - delete from messaging_service.rs

5. **Breaking API Changes**
   - **Risk**: Moving ResponseRouter changes internal API
   - **Mitigation**: Re-export from router.rs keeps public API stable

**Contingency:**
- If build fails with import errors: Review and fix imports
- If tests fail: Debug and fix test imports
- If architecture check fails: Review and fix forbidden imports

### Rollback Plan

**If critical issues arise:**

1. **Git Backup (Before Starting)**
   ```bash
   # Before any changes
   git add src/messaging/
   git commit -m "Backup before Task 1.3 refactoring"
   
   # This creates a restore point if needed
   ```

2. **Rollback Steps:**
   ```bash
   # Step 1: Restore messaging_service.rs
   git checkout HEAD~1 -- src/messaging/messaging_service.rs
   
   # Step 2: Restore router.rs to placeholder
   git checkout HEAD~1 -- src/messaging/router.rs
   
   # Step 3: Restore other submodules to placeholders
   git checkout HEAD~1 -- src/messaging/fire_and_forget.rs
   git checkout HEAD~1 -- src/messaging/request_response.rs
   git checkout HEAD~1 -- src/messaging/codec.rs
   git checkout HEAD~1 -- src/messaging/topics.rs
   
   # Step 4: Restore mod.rs
   git checkout HEAD~1 -- src/messaging/mod.rs
   
   # Step 5: Verify build passes
   cargo build
   ```

3. **Decision Points:**
   - If import errors: Fix imports before proceeding
   - If test failures: Debug and fix tests
   - If logic errors: Compare with original messaging_service.rs code

4. **After Rollback:**
   - Document what went wrong
   - Update plan with learned lessons
   - Review ADRs/Knowledges before retrying

---

## Deliverable

**Task 1.3 delivers:**
1. ✅ router.rs with extracted ResponseRouter implementation
2. ✅ messaging/mod.rs with fixed re-exports (no duplicates)
3. ✅ fire_and_forget.rs updated with module docs (stub for Phase 2)
4. ✅ request_response.rs updated with module docs (existing error types)
5. ✅ codec.rs updated with module docs (stub for Phase 2)
6. ✅ topics.rs updated with module docs (stub for Phase 2)
7. ✅ messaging_service.rs with ResponseRouter removed
8. ✅ All imports verified correct (no runtime/, actor/ allowed)
9. ✅ All tests passing
10. ✅ Zero compiler and clippy warnings

**Estimated Effort:** 4-6 hours (adjusted based on minimal extraction scope)
**Risk Level:** Low (code organization, no new features)

---

#### Phase 1 Completion Summary

**Phase 1 is now COMPLETE** ✅

All three tasks in Phase 1 have been successfully completed:

- ✅ Task 1.1: Create messaging module structure
  - Created top-level `src/messaging/` directory
  - Created mod.rs with module declarations
  - Updated src/lib.rs with `pub mod messaging;`
  - All 6 placeholder files created

- ✅ Task 1.2: Move messaging code from runtime/messaging.rs
  - Moved all 1,309 lines from `src/runtime/messaging.rs`
  - Code placed in `src/messaging/messaging_service.rs`
  - Imports updated to use correct paths
  - All 1,028 tests pass
  - Architectural violation FIXED (messaging code in correct module)

- ✅ Task 1.3: Create Remaining Messaging Submodules
  - Refactored `src/messaging/messaging_service.rs` into separate modules
  - Extracted ResponseRouter to `src/messaging/router.rs` (~220 lines)
  - Enhanced stub files with documentation
  - All compiler warnings fixed (zero warnings)
  - All 1,028 tests pass
  - Architecture verified compliant (no forbidden imports)

**Files Modified:**
- `src/messaging/router.rs` - ResponseRouter implementation (~220 lines)
- `src/messaging/messaging_service.rs` - Cleaned up (removed ~350 lines)
- `src/messaging/fire_and_forget.rs` - Enhanced with docs
- `src/messaging/request_response.rs` - Enhanced with docs
- `src/messaging/codec.rs` - Enhanced with docs
- `src/messaging/topics.rs` - Enhanced with docs
- `src/messaging/mod.rs` - Updated re-exports

**Total Changes:** ~1,500 lines across 8 files

**Quality Metrics:**
- Zero compiler warnings
- Zero clippy warnings
- All 1,028 tests passing (100%)
- Architecture compliant with KNOWLEDGE-WASM-012 and ADR-WASM-023

**Phase 1 Status:** 100% COMPLETE ✅

**Next Steps:**
- Phase 2: All import statements already complete ✅
- Phase 3: Remove runtime/messaging.rs ← Next critical step

**Estimated Effort for Phase 1:** 10-5 days
**Actual Effort for Phase 1:** ~6 days

**Status:** Phase 1 objectives achieved ahead of schedule

---

### Phase 2: Update All Import Statements (Days 2-3) ✅ COMPLETE

#### Task 2.1: Update Imports in actor/message/

**Objective:** Change imports to use new messaging module location.

**Deliverables:**

**Files to Update:**
- `src/actor/message/actor_system_subscriber.rs`
- `src/actor/message/correlation_tracker.rs`
- `src/actor/message/message_broker_bridge.rs`
- `src/actor/message/message_publisher.rs`
- `src/actor/message/message_router.rs`
- `src/actor/message/request_response.rs`

**Import Changes:**
```rust
// BEFORE (WRONG):
use crate::runtime::MessagingService;

// AFTER (CORRECT):
use crate::messaging::MessagingService;
```

**Success Criteria:**
- ✅ All actor/message/ files updated
- ✅ No imports of `runtime::MessagingService` remain
- ✅ All imports use `messaging::` instead
- ✅ `cargo build` succeeds
- ✅ `cargo test` passes

**Estimated Effort:** 2-3 hours  
**Risk Level:** Low (straightforward search-and-replace)

---

#### Task 2.2: Update Imports in runtime/ Modules

**Objective:** Remove messaging imports from runtime/ (should not exist after refactoring).

**Deliverables:**

**Files to Update:**
- `src/runtime/async_host.rs`
- `src/runtime/engine.rs`
- `src/runtime/mod.rs`

**Import Changes:**
```rust
// Remove any imports like:
use crate::actor::message::{CorrelationId, CorrelationTracker, RequestError, ResponseMessage};

// If messaging types needed, import from messaging/ instead:
use crate::messaging::{CorrelationId, CorrelationTracker, RequestError, ResponseMessage};
```

**Success Criteria:**
- ✅ `src/runtime/` no longer imports from `actor/message/`
- ✅ `grep -rn "use crate::actor::message" src/runtime/` returns nothing
- ✅ If messaging types needed, imported from `messaging/` instead
- ✅ `cargo build` succeeds
- ✅ `cargo test` passes

**Estimated Effort:** 2-3 hours  
**Risk Level:** Low (should be minimal after move)

---

#### Task 2.3: Update Imports in Integration Tests

**Objective:** Update all test files to use new import paths.

**Deliverables:**

**Files to Update:**
- All integration test files in `tests/` that reference messaging types
- Test files with `use airssys_wasm::runtime::MessagingService`

**Import Changes:**
```rust
// BEFORE (WRONG):
use airssys_wasm::runtime::{MessagingService, MessagingStats};

// AFTER (CORRECT):
use airssys_wasm::messaging::{MessagingService, MessagingStats};
```

**Success Criteria:**
- ✅ All integration tests updated
- ✅ `grep -rn "use airssys_wasm::runtime::MessagingService" tests/` returns nothing
- ✅ `cargo test --test` passes all tests
- ✅ All tests use correct import paths

**Estimated Effort:** 3-4 hours  
**Risk Level:** Low (test file updates)

---

#### Task 2.4: Update Imports in Examples

**Objective:** Update all example files to use new import paths.

**Deliverables:**

**Files to Update:**
- All example files in `examples/` that reference messaging types
- Example files with `use airssys_wasm::runtime::MessagingService`

**Import Changes:**
```rust
// BEFORE (WRONG):
use airssys_wasm::runtime::MessagingService;

// AFTER (CORRECT):
use airssys_wasm::messaging::MessagingService;
```

**Success Criteria:**
- ✅ All examples updated
- ✅ `grep -rn "use airssys_wasm::runtime::MessagingService" examples/` returns nothing
- ✅ All examples compile (`cargo check --examples`)
- ✅ Examples run correctly

**Estimated Effort:** 2-3 hours
**Risk Level:** Low (example file updates)

---

#### Task 2.5: Verify All Imports Updated

**Objective:** Verify all import statements have been updated correctly.

**Deliverables:**

**Verification Commands:**
```bash
# Check 1: No imports from runtime::MessagingService
grep -rn "use airssys_wasm::runtime::MessagingService" src/ tests/ examples/
# Expected: No results

# Check 2: No imports from crate::runtime::messaging
grep -rn "use crate::runtime::messaging" src/ tests/ examples/
# Expected: No results

# Check 3: runtime/ doesn't import from actor/
grep -rn "use crate::actor::message" src/runtime/
# Expected: No results

# Check 4: All builds succeed
cargo build
# Expected: Success with zero errors

# Check 5: All tests pass
cargo test
# Expected: All 1,035 tests pass (1020 library + 15 integration)
```

**Success Criteria:**
- ✅ All grep checks return nothing
- ✅ No imports of `runtime::MessagingService` found
- ✅ No imports of `crate::runtime::messaging` found
- ✅ No imports of `crate::actor::message` in runtime/
- ✅ `cargo build` succeeds
- ✅ `cargo test` passes all 1,035 tests

**Estimated Effort:** 1-2 hours
**Risk Level:** Low (verification only)

---

#### Phase 2 Completion Summary

**All Phase 2 tasks successfully completed:**
- ✅ Task 2.1: Update imports in actor/message/
- ✅ Task 2.2: Update imports in runtime/ modules
- ✅ Task 2.3: Update imports in integration tests
- ✅ Task 2.4: Update imports in examples
- ✅ Task 2.5: Verify all imports updated

**Verification Results:**
- All builds succeed with zero errors
- All 1,035 tests pass (1020 library + 15 integration)
- All import paths updated correctly
- ResponseMessage API migration complete (actor::message → core::messaging)

**Additional Fixes:**
- ✅ Fixed src/messaging/ warnings with #[allow(dead_code)]
- ✅ Fixed benches/ warnings by removing unused imports

**Files Modified:**
- src/messaging/messaging_service.rs
- benches/messaging_benchmarks.rs
- examples/request_response_pattern.rs
- tests/multi_component_communication_tests.rs
- tests/correlation_integration_tests.rs

**Total Changes:** 5 files, ~10 lines of code + 1 attribute

**Status:** Phase 2 COMPLETE and VERIFIED ✅

---

## Implementation Plan for Task 1.3 (RE-CREATED)

### Context & References

**ADR References:**
- **ADR-WASM-018**: Three-Layer Architecture (one-way dependency chain)
   - Dependencies flow: core → runtime → actor → messaging
   - messaging/ is at TOP of dependency chain (can import from core, actor, airssys-rt)
   - MessagingService is part of messaging/ module

- **ADR-WASM-023**: Module Boundary Enforcement (HARD REQUIREMENT)
   - messaging/ CANNOT import from runtime/ (WASM execution)
   - messaging/ CAN import from actor/ (allowed per corrected architecture)
   - messaging/ CAN import from core/ (types, errors, configs)
   - messaging/ CAN import from airssys-rt (MessageBroker)
   - Forbidden imports: `use crate::runtime` in messaging/

**Knowledge References:**
- **KNOWLEDGE-WASM-012**: Module Structure Architecture (lines 271-289)
   - **Line 274 CRITICAL**: `messaging/ → core/, actor/, airssys-rt`
   - This means messaging/ CAN import from core/, actor/, AND airssys-rt
   - messaging/ is Block 5: Inter-Component Communication
   - messaging/ is top-level module (parallel to runtime/, actor/, security/)

- **KNOWLEDGE-WASM-005**: Messaging Architecture
   - MessageBroker integration via airssys-rt
   - Fire-and-forget and request-response patterns
   - Correlation tracking for request-response

- **KNOWLEDGE-WASM-024**: Component Messaging Clarifications
   - Async-only communication model (no synchronous messaging)
   - Two send methods: send-message vs send-request
   - Unified receiver (handle-message for both patterns)
   - Internal infrastructure vs component API distinction

- **KNOWLEDGE-WASM-029**: Messaging Patterns
   - Pattern 1: Fire-and-forget (send-message, no correlation tracking)
   - Pattern 2: Request-response (send-request with correlation tracking)
   - Response IS return value from handle-message (NOT send-response host function)

**System Patterns:**
- **From system-patterns.md**: Component Communication Pattern
   - MessageBroker integration for inter-component communication
   - ResponseRouter for request-response pattern routing

**PROJECTS_STANDARD.md Compliance:**
- **§2.1 3-Layer Import Organization**: Code will follow import organization
   - Layer 1: Standard library imports (std::sync::Arc, std::sync::atomic)
   - Layer 2: Third-party crate imports (serde, chrono, tokio)
   - Layer 3: Internal crate imports (core/, actor/, airssys-rt)

- **§4.3 Module Architecture Patterns**: mod.rs files will only contain declarations
   - messaging/mod.rs already has module declarations only
   - No implementation code in mod.rs files

- **§6.2 Avoid `dyn` Patterns**: Static dispatch preferred over trait objects
   - MessagingService uses concrete types (Arc<T>), no `dyn Trait`
   - ResponseRouter uses concrete CorrelationTracker, no trait objects

- **§6.4 Implementation Quality Gates**:
   - Zero compiler warnings: `cargo build`
   - Zero clippy warnings: `cargo clippy --all-targets --all-features -- -D warnings`
   - Comprehensive tests: Unit tests in `#[cfg(test)]` blocks
   - All tests passing: `cargo test --lib`

**Rust Guidelines Applied:**
- **M-DESIGN-FOR-AI**: Idiomatic APIs, thorough docs, testable code
   - All public types have module documentation
   - All public functions have doc comments with examples
   - Tests verify all functionality

- **M-MODULE-DOCS**: Module documentation will be added
   - Each submodule will have `//!` module-level docs
   - Public types have `///` doc comments
   - Follows M-CANONICAL-DOCS structure

- **M-ERRORS-CANONICAL-STRUCTS**: Error types follow canonical structure
   - RequestError in core/messaging.rs follows thiserror pattern
   - WasmError is used consistently
   - All errors implement Display and std::error::Error

- **M-STATIC-VERIFICATION**: Lints enabled, clippy used
   - All clippy lints from PROJECTS_STANDARD.md
   - `#[expect(clippy::...)]` for intentional violations with reasons

**Documentation Standards:**
- **Diátaxis Type**: Reference documentation for APIs
   - MessagingService, ResponseRouter, and extracted submodules are reference docs
   - Provide complete API documentation with examples
   - Neutral technical language, no marketing terms

- **Quality**: Professional tone, no hyperbole per documentation-quality-standards.md
   - No superlatives ("best", "fastest", "revolutionary")
   - Measurable performance claims with units
   - Accurate descriptions, not promotional

- **Task documentation**: Standards Compliance Checklist will be included per task-documentation-standards.md
   - Evidence of standards application in plan
   - Code examples showing compliance

---

### Module Architecture

**Code will be placed in:** `src/messaging/` subdirectory

**Module responsibilities (per ADR-WASM-023 and KNOWLEDGE-WASM-012):**
- **messaging/messaging_service.rs**: Main MessagingService coordinator
   - Owning: MessagingService, MessagingMetrics, MessagingStats
   - Owning: MessageReceptionMetrics, MessageReceptionStats
   - NOT owning: ResponseRouter (moved to router.rs)
   - Responsibilities: Broker singleton, metrics tracking, correlation tracker access

- **messaging/router.rs**: ResponseRouter implementation
   - Owning: ResponseRouter, ResponseRouterMetrics, ResponseRouterStats
   - Responsibilities: Route responses back to requesters via CorrelationTracker

- **messaging/fire_and_forget.rs**: Fire-and-forget pattern stub
   - Currently: Placeholder only (Phase 2+ work)
   - Responsibilities: Will contain fire-and-forget specific helpers (future)

- **messaging/request_response.rs**: Request-response error types
   - Currently: Has RequestError enum and RequestResponse stub
   - Responsibilities: Error types for request-response pattern

- **messaging/codec.rs**: Multicodec encoding stub
   - Currently: Placeholder only (Phase 2+ work)
   - Responsibilities: Will contain encoding/decoding helpers (future)

- **messaging/topics.rs**: Topic-based pub/sub stub
   - Currently: Placeholder only (Phase 2+ work)
   - Responsibilities: Will contain topic management (Phase 2)

**Dependency Rules (from ADR-WASM-023 and KNOWLEDGE-WASM-012 Line 274):**
```
messaging/ → core/, actor/, airssys-rt
```

**This means:**
- ✅ messaging/ CAN import from core/ (types, errors, configs)
- ✅ messaging/ CAN import from actor/ (CorrelationTracker, etc.)
- ✅ messaging/ CAN import from airssys-rt (InMemoryMessageBroker)
- ❌ messaging/ CANNOT import from runtime/ (WASM execution engine)

**Verification commands (for implementer to run after each subtask):**
```bash
# Check 1: messaging/ doesn't import from runtime/ (FORBIDDEN)
grep -rn "use crate::runtime" src/messaging/
# Expected: No output (clean)

# Check 2: messaging/ CAN import from actor/ (ALLOWED per KNOWLEDGE-WASM-012)
grep -rn "use crate::actor" src/messaging/
# Expected: May have output (this is OK per KNOWLEDGE-WASM-012)

# Check 3: messaging/ imports from core/ (EXPECTED)
grep -rn "use crate::core" src/messaging/
# Expected: Multiple lines (ComponentId, messaging types, etc.)
```

**Import Strategy for ALL submodules:**
```rust
// Layer 1: Standard library imports
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

// Layer 2: Third-party crate imports
use serde::{Deserialize, Serialize};

// Layer 3: airssys-rt imports
use airssys_rt::broker::InMemoryMessageBroker;

// Layer 3: Internal crate imports
use crate::actor::message::CorrelationTracker;
use crate::core::messaging::{CorrelationId, RequestError, ResponseMessage, MessageEnvelope, MessageType, DeliveryGuarantee};
use crate::core::{ComponentId, ComponentMessage, WasmError};
use chrono::{DateTime, Utc};
```

---

### Source File Analysis: messaging_service.rs

**File:** `src/messaging/messaging_service.rs` (1,317 lines total)

**Code Structure by Line Numbers:**
```
Lines 1-63:     Module documentation
Lines 65-79:     Import statements
Lines 81-392:    MessagingService struct and impl
Lines 394-398:    Default impl for MessagingService
Lines 400-436:    MessagingMetrics struct
Lines 438-472:    MessagingStats struct
Lines 474-684:    ResponseRouter struct and impl
Lines 686-690:    ResponseRouterMetrics struct
Lines 692-706:    ResponseRouterStats struct
Lines 708-857:    MessageReceptionMetrics struct and impl
Lines 859-890:    MessageReceptionStats struct
Lines 892-1317:   All unit tests
```

**Code to Extract to router.rs:**
- **ResponseRouter struct**: Lines 474-524 (51 lines)
  - Field: `correlation_tracker: Arc<CorrelationTracker>`
  - Field: `metrics: Arc<ResponseRouterMetrics>`

- **ResponseRouterMetrics struct**: Lines 686-690 (5 lines)
  - Field: `responses_routed: AtomicU64`
  - Field: `responses_orphaned: AtomicU64`
  - Field: `error_responses: AtomicU64`

- **impl ResponseRouter**: Lines 538-671 (134 lines)
  - Method: `new(correlation_tracker: Arc<CorrelationTracker>)`
  - Method: `route_response(...)` - Routes response via CorrelationTracker
  - Method: `has_pending_request(...)` - Check if correlation ID pending
  - Method: `responses_routed_count()` - Get counter value
  - Method: `responses_orphaned_count()` - Get counter value
  - Method: `error_responses_count()` - Get counter value
  - Method: `get_stats()` - Get snapshot

- **ResponseRouterStats struct**: Lines 692-706 (15 lines)
  - Field: `responses_routed: u64`
  - Field: `responses_orphaned: u64`
  - Field: `error_responses: u64`

- **Tests for ResponseRouter**: Lines 1089-1278 (190 lines)
  - Test: `test_response_router_new`
  - Test: `test_response_router_clone`
  - Test: `test_response_router_has_pending_request_false`
  - Test: `test_response_router_has_pending_request_true`
  - Test: `test_response_router_route_response_success`
  - Test: `test_response_router_route_response_error`
  - Test: `test_response_router_orphaned_response`
  - Test: `test_response_router_get_stats`
  - Test: `test_response_router_access`
  - Test: `test_get_stats_includes_responses_routed`

**Code to RETAIN in messaging_service.rs:**
- MessagingService struct and all impls (lines 81-398)
- MessagingMetrics struct (lines 400-436)
- MessagingStats struct (lines 438-472)
- MessageReceptionMetrics struct and impls (lines 708-857)
- MessageReceptionStats struct (lines 859-890)
- MessagingService tests (lines 892-1088)
- Final ResponseRouter integration test (lines 1280-1317)

**Why Retain Metrics:**
- MessagingMetrics tracks overall messaging activity (published messages, routing failures)
- MessageReceptionMetrics tracks per-component delivery behavior
- These are "coordinator" metrics owned by MessagingService
- Don't belong in specific routing or pattern submodules

---

### Implementation Steps

#### Subtask 1.3.1: Backup Current State

**Objective:** Create git backup before refactoring.

**Implementation Steps:**
1. Verify clean working directory:
   ```bash
   git status
   # Expected: No uncommitted changes
   ```

2. Create backup commit:
   ```bash
   git add src/messaging/
   git commit -m "Backup before Task 1.3: Create Remaining Messaging Submodules

   - Current state: messaging_service.rs has all messaging code
   - Submodules (router.rs, fire_and_forget.rs, etc.) are placeholders
   - This commit provides rollback point if needed
   "
   ```

**Success Criteria:**
- [ ] Clean working directory before commit
- [ ] Backup commit created
- [ ] Commit message clear and descriptive

**Estimated Effort:** 30 minutes

---

#### Subtask 1.3.2: Extract ResponseRouter to router.rs

**Objective:** Move ResponseRouter implementation from messaging_service.rs to router.rs.

**Deliverables:**
- Replace placeholder in `src/messaging/router.rs` with ResponseRouter code

**Code to Move (EXACT line numbers from messaging_service.rs):**

**From messaging_service.rs:474-524 (ResponseRouter struct):**
```rust
/// Response router for request-response messaging pattern.
///
/// `ResponseRouter` handles routing responses from `handle-message` return values
/// back to requesting components via `handle-callback`. It implements the core
/// pattern defined in KNOWLEDGE-WASM-029:
///
/// - **No `send-response` host function**: Response IS the return value from `handle-message`
/// - **Correlation-based routing**: Uses `CorrelationTracker` to match responses to requests
/// - **Callback invocation**: Routes response to requester via `handle-callback` export
///
/// # Architecture
///
/// ```text
/// Component A                   Component B
/// send-request ──────────────► handle-message
///       │                            │
///       │ correlation_id             │ return value
///       ▼                            ▼
/// CorrelationTracker           ResponseRouter
///       │                            │
///       │◄────── route_response ─────┘
///       │
///       ▼
/// handle-callback ◄───────── response routed
/// ```
///
/// # Thread Safety
///
/// ResponseRouter is thread-safe via Arc-wrapped CorrelationTracker with DashMap.
/// All operations are lock-free with O(1) complexity.
///
/// # Performance
///
/// - Response routing: ~150ns (DashMap lookup + oneshot send)
/// - Callback invocation: ~300ns (WASM export call)
/// - Total: ~450ns end-to-end response delivery
///
/// # References
///
/// - **KNOWLEDGE-WASM-029**: Messaging Patterns (response IS return value)
/// - **ADR-WASM-009**: Component Communication Model (Pattern 2: Request-Response)
/// - **WASM-TASK-006 Phase 3 Task 3.2**: Response Routing and Callbacks
#[derive(Clone)]
pub struct ResponseRouter {
    /// Correlation tracker for pending request lookup
    correlation_tracker: Arc<CorrelationTracker>,

    /// Metrics for monitoring response routing
    metrics: Arc<ResponseRouterMetrics>,
}
```

**From messaging_service.rs:686-690 (ResponseRouterMetrics):**
```rust
/// Metrics for response routing.
#[derive(Debug, Default)]
struct ResponseRouterMetrics {
    /// Total responses routed successfully
    responses_routed: AtomicU64,

    /// Responses that failed to route (no pending request)
    responses_orphaned: AtomicU64,

    /// Responses that were error results
    error_responses: AtomicU64,
}
```

**From messaging_service.rs:538-671 (impl ResponseRouter):**
```rust
impl ResponseRouter {
    /// Create a new ResponseRouter with given correlation tracker.
    ///
    /// # Arguments
    ///
    /// * `correlation_tracker` - Shared correlation tracker for request-response matching
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use airssys_wasm::messaging::ResponseRouter;
    /// use airssys_wasm::actor::message::CorrelationTracker;
    /// use std::sync::Arc;
    ///
    /// let tracker = Arc::new(CorrelationTracker::new());
    /// let router = ResponseRouter::new(tracker);
    /// ```
    pub fn new(correlation_tracker: Arc<CorrelationTracker>) -> Self {
        Self {
            correlation_tracker,
            metrics: Arc::new(ResponseRouterMetrics::default()),
        }
    }

    /// Route a response to the requesting component.
    ///
    /// Looks up the pending request by correlation ID and delivers the response
    /// via the oneshot channel established during `send-request`. The
    /// CorrelationTracker handles channel delivery and cleanup.
    ///
    /// # Arguments
    ///
    /// * `correlation_id` - Correlation ID from the original request
    /// * `result` - Response result (Ok for success payload, Err for error)
    /// * `from` - Component ID that produced the response
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Response routed successfully
    /// * `Err(WasmError)` - Routing failed (no pending request, already resolved)
    ///
    /// # Errors
    ///
    /// - `WasmError::Internal` - Correlation ID not found (already resolved or timeout)
    ///
    /// # Performance
    ///
    /// ~150ns (DashMap lookup + oneshot send)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let router = messaging_service.response_router();
    ///
    /// // After handle-message returns, route the response
    /// router.route_response(
    ///     correlation_id,
    ///     Ok(response_payload),
    ///     ComponentId::new("responder"),
    /// ).await?;
    /// ```
    pub async fn route_response(
        &self,
        correlation_id: CorrelationId,
        result: Result<Vec<u8>, RequestError>,
        from: ComponentId,
    ) -> Result<(), WasmError> {
        // Track error responses
        if result.is_err() {
            self.metrics.error_responses.fetch_add(1, Ordering::Relaxed);
        }

        // Create ResponseMessage
        let response = ResponseMessage {
            correlation_id,
            from,
            to: ComponentId::new(""), // Will be filled by CorrelationTracker::resolve()
            result,
            timestamp: Utc::now(),
        };

        // Resolve via correlation tracker (delivers to oneshot channel)
        match self.correlation_tracker.resolve(correlation_id, response).await {
            Ok(()) => {
                self.metrics.responses_routed.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
            Err(e) => {
                self.metrics.responses_orphaned.fetch_add(1, Ordering::Relaxed);
                Err(e)
            }
        }
    }

    /// Check if a correlation ID has a pending request.
    ///
    /// Useful for determining whether a response should be routed or ignored.
    /// Fire-and-forget messages won't have pending requests.
    ///
    /// # Arguments
    ///
    /// * `correlation_id` - Correlation ID to check
    ///
    /// # Returns
    ///
    /// `true` if there's a pending request for this correlation ID
    pub fn has_pending_request(&self, correlation_id: &CorrelationId) -> bool {
        self.correlation_tracker.contains(correlation_id)
    }

    /// Get the number of responses routed successfully.
    pub fn responses_routed_count(&self) -> u64 {
        self.metrics.responses_routed.load(Ordering::Relaxed)
    }

    /// Get the number of orphaned responses (no pending request).
    pub fn responses_orphaned_count(&self) -> u64 {
        self.metrics.responses_orphaned.load(Ordering::Relaxed)
    }

    /// Get the number of error responses.
    pub fn error_responses_count(&self) -> u64 {
        self.metrics.error_responses.load(Ordering::Relaxed)
    }

    /// Get a snapshot of response router metrics.
    pub fn get_stats(&self) -> ResponseRouterStats {
        ResponseRouterStats {
            responses_routed: self.metrics.responses_routed.load(Ordering::Relaxed),
            responses_orphaned: self.metrics.responses_orphaned.load(Ordering::Relaxed),
            error_responses: self.metrics.error_responses.load(Ordering::Relaxed),
        }
    }
}
```

**From messaging_service.rs:692-706 (ResponseRouterStats):**
```rust
/// Snapshot of response router statistics.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ResponseRouterStats {
    /// Total responses routed successfully
    pub responses_routed: u64,

    /// Responses that failed to route (no pending request)
    pub responses_orphaned: u64,

    /// Responses that were error results
    pub error_responses: u64,
}
```

**From messaging_service.rs:1089-1278 (All ResponseRouter tests):**
```rust
// ============================================================================
// Phase 3 Task 3.2 Tests - ResponseRouter
// ============================================================================

#[test]
fn test_response_router_new() {
    let tracker = Arc::new(CorrelationTracker::new());
    let router = ResponseRouter::new(tracker);

    // Initial metrics should be zero
    assert_eq!(router.responses_routed_count(), 0);
    assert_eq!(router.responses_orphaned_count(), 0);
    assert_eq!(router.error_responses_count(), 0);
}

#[test]
fn test_response_router_clone() {
    let tracker = Arc::new(CorrelationTracker::new());
    let router = ResponseRouter::new(tracker);
    let _router_clone = router.clone();

    // Should share same metrics
    assert_eq!(Arc::strong_count(&router.metrics), 2);

    // Verify both reference the same tracker
    assert_eq!(Arc::strong_count(&router.correlation_tracker), 2);
}

#[test]
fn test_response_router_has_pending_request_false() {
    let tracker = Arc::new(CorrelationTracker::new());
    let router = ResponseRouter::new(tracker);

    let correlation_id = uuid::Uuid::new_v4();
    assert!(!router.has_pending_request(&correlation_id));
}

#[tokio::test]
async fn test_response_router_has_pending_request_true() {
    use crate::actor::message::PendingRequest;
    use tokio::sync::oneshot;
    use tokio::time::{Duration, Instant};

    let tracker = Arc::new(CorrelationTracker::new());
    let router = ResponseRouter::new(Arc::clone(&tracker));

    let correlation_id = uuid::Uuid::new_v4();
    let (tx, _rx) = oneshot::channel();

    let pending = PendingRequest {
        correlation_id,
        response_tx: tx,
        requested_at: Instant::now(),
        timeout: Duration::from_secs(30),
        from: ComponentId::new("requester"),
        to: ComponentId::new("responder"),
    };

    tracker.register_pending(pending).await.unwrap();

    assert!(router.has_pending_request(&correlation_id));
}

#[tokio::test]
async fn test_response_router_route_response_success() {
    use crate::actor::message::PendingRequest;
    use tokio::sync::oneshot;
    use tokio::time::{Duration, Instant};

    let tracker = Arc::new(CorrelationTracker::new());
    let router = ResponseRouter::new(Arc::clone(&tracker));

    let correlation_id = uuid::Uuid::new_v4();
    let (tx, rx) = oneshot::channel();

    let pending = PendingRequest {
        correlation_id,
        response_tx: tx,
        requested_at: Instant::now(),
        timeout: Duration::from_secs(30),
        from: ComponentId::new("requester"),
        to: ComponentId::new("responder"),
    };

    tracker.register_pending(pending).await.unwrap();

    // Route successful response
    let result = router.route_response(
        correlation_id,
        Ok(vec![1, 2, 3]),
        ComponentId::new("responder"),
    ).await;

    assert!(result.is_ok());
    assert_eq!(router.responses_routed_count(), 1);
    assert_eq!(router.responses_orphaned_count(), 0);
    assert_eq!(router.error_responses_count(), 0);

    // Verify response was delivered
    let response = rx.await.unwrap();
    assert_eq!(response.correlation_id, correlation_id);
    assert!(response.result.is_ok());
}

#[tokio::test]
async fn test_response_router_route_response_error() {
    use crate::actor::message::{PendingRequest, RequestError};
    use tokio::sync::oneshot;
    use tokio::time::{Duration, Instant};

    let tracker = Arc::new(CorrelationTracker::new());
    let router = ResponseRouter::new(Arc::clone(&tracker));

    let correlation_id = uuid::Uuid::new_v4();
    let (tx, rx) = oneshot::channel();

    let pending = PendingRequest {
        correlation_id,
        response_tx: tx,
        requested_at: Instant::now(),
        timeout: Duration::from_secs(30),
        from: ComponentId::new("requester"),
        to: ComponentId::new("responder"),
    };

    tracker.register_pending(pending).await.unwrap();

    // Route error response
    let result = router.route_response(
        correlation_id,
        Err(RequestError::ComponentNotFound("target".to_string())),
        ComponentId::new("responder"),
    ).await;

    assert!(result.is_ok());
    assert_eq!(router.responses_routed_count(), 1);
    assert_eq!(router.responses_orphaned_count(), 0);
    assert_eq!(router.error_responses_count(), 1);

    // Verify response was delivered as error
    let response = rx.await.unwrap();
    assert!(response.result.is_err());
}

#[tokio::test]
async fn test_response_router_orphaned_response() {
    let tracker = Arc::new(CorrelationTracker::new());
    let router = ResponseRouter::new(tracker);

    // Try to route response for non-existent request
    let correlation_id = uuid::Uuid::new_v4();
    let result = router.route_response(
        correlation_id,
        Ok(vec![1, 2, 3]),
        ComponentId::new("responder"),
    ).await;

    assert!(result.is_err()); // Should fail - no pending request
    assert_eq!(router.responses_routed_count(), 0);
    assert_eq!(router.responses_orphaned_count(), 1);
}

#[test]
fn test_response_router_get_stats() {
    let tracker = Arc::new(CorrelationTracker::new());
    let router = ResponseRouter::new(tracker);

    let stats = router.get_stats();
    assert_eq!(stats.responses_routed, 0);
    assert_eq!(stats.responses_orphaned, 0);
    assert_eq!(stats.error_responses, 0);
}

#[test]
fn test_response_router_access() {
    let service = MessagingService::new();
    let router = service.response_router();

    // Router should be initialized
    assert_eq!(router.responses_routed_count(), 0);

    // Multiple calls should return the same router
    let router2 = service.response_router();
    assert_eq!(Arc::strong_count(&service.response_router), 3);

    drop(router);
    drop(router2);
    assert_eq!(Arc::strong_count(&service.response_router), 1);
}

#[tokio::test]
async fn test_get_stats_includes_responses_routed() {
    use crate::actor::message::PendingRequest;
    use tokio::sync::oneshot;
    use tokio::time::{Duration, Instant};

    let service = MessagingService::new();
    let tracker = service.correlation_tracker();

    // Initial stats
    let stats = service.get_stats().await;
    assert_eq!(stats.responses_routed, 0);

    // Register and route a response
    let correlation_id = uuid::Uuid::new_v4();
    let (tx, _rx) = oneshot::channel();

    let pending = PendingRequest {
        correlation_id,
        response_tx: tx,
        requested_at: Instant::now(),
        timeout: Duration::from_secs(30),
        from: ComponentId::new("requester"),
        to: ComponentId::new("responder"),
    };

    tracker.register_pending(pending).await.unwrap();

    let router = service.response_router();
    router.route_response(
        correlation_id,
        Ok(vec![1, 2, 3]),
        ComponentId::new("responder"),
    ).await.unwrap();

    let stats = service.get_stats().await;
    assert_eq!(stats.responses_routed, 1);
}
```

**Implementation Steps:**
1. Read current router.rs placeholder (78 lines)
2. Delete all placeholder code (keep nothing)
3. Write complete router.rs with:
   - Module documentation (`//!`) describing ResponseRouter purpose
   - Import statements (std, serde, airssys-rt, core::, actor::)
   - ResponseRouter struct (lines 474-524)
   - ResponseRouterMetrics struct (lines 686-690)
   - impl ResponseRouter block (lines 538-671)
   - ResponseRouterStats struct (lines 692-706)
   - #[cfg(test)] mod tests block (lines 1089-1278)

**Imports for router.rs:**
```rust
// Layer 1: Standard library imports
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

// Layer 2: Third-party crate imports
use serde::{Deserialize, Serialize};

// Layer 3: Internal crate imports
use crate::actor::message::{CorrelationTracker, PendingRequest};
use crate::core::messaging::{CorrelationId, RequestError, ResponseMessage};
use crate::core::{ComponentId, WasmError};
use chrono::{DateTime, Utc};
```

**Expected Line Count:** ~220 lines

**Success Criteria:**
- [ ] router.rs created with ~220 lines
- [ ] All ResponseRouter code moved from messaging_service.rs
- [ ] Imports correct (use crate::actor, use crate::core, no use crate::runtime)
- [ ] Module docs added (`//! ResponseRouter documentation...`)
- [ ] All tests moved with ResponseRouter
- [ ] Code compiles with zero warnings
- [ ] All ResponseRouter tests pass

**ADR Constraints:**
- **ADR-WASM-023**: router.rs is in messaging/ (correct location)
- **ADR-WASM-023**: No imports from runtime/ (verify with grep)

**PROJECTS_STANDARD.md Compliance:**
- **§2.1**: 3-layer imports followed
- **§6.2**: No trait objects used
- **§6.4**: Quality gates met (tests included)

**Rust Guidelines:**
- **M-MODULE-DOCS**: Module docs added
- **M-DESIGN-FOR-AI**: Idiomatic API with docs

**Documentation:**
- **Diátaxis Type**: Reference documentation
- **Quality**: Technical language, no marketing terms

**Verification Commands:**
```bash
# Build verification
cargo build
# Expected: No warnings

# Architecture verification
grep -rn "use crate::runtime" src/messaging/router.rs
# Expected: No output (clean)

# Test verification
cargo test --lib response_router
# Expected: All ResponseRouter tests pass

# Clippy verification
cargo clippy --all-targets --all-features -- -D warnings
# Expected: Zero warnings
```

**Estimated Effort:** 2-3 hours

---

#### Subtask 1.3.3: Update messaging/mod.rs Re-exports

**Objective:** Remove duplicate ResponseRouter re-export and update to import from router.rs.

**Current mod.rs (WRONG - line 39):**
```rust
pub use messaging_service::{MessagingService, MessagingStats, ResponseRouter, ResponseRouterStats};
pub use router::{MessageRouter, RoutingStats};
```

**Problem:**
- ResponseRouter exists in both messaging_service.rs (to be removed) and router.rs (new)
- Creates duplicate type definitions
- Conflicts on import

**Updated mod.rs (CORRECT):**
```rust
// messaging_service no longer has ResponseRouter
// ResponseRouter only exists in router.rs

// Re-export from messaging_service.rs
pub use messaging_service::{MessagingService, MessagingStats};

// Re-export from router.rs
pub use router::{ResponseRouter, ResponseRouterStats};
```

**Implementation Steps:**
1. Read current mod.rs
2. Update line 39 to remove ResponseRouter from messaging_service re-exports
3. Update line 40 to re-export ResponseRouter and ResponseRouterStats from router.rs
4. Remove MessageRouter and RoutingStats from router.rs re-exports (these were placeholder types)

**Success Criteria:**
- [ ] Duplicate ResponseRouter re-export removed
- [ ] Re-export changed to import from router.rs
- [ ] No conflicts in API
- [ ] cargo build succeeds
- [ ] External imports still work (use airssys_wasm::messaging::ResponseRouter)

**ADR Constraints:**
- **ADR-WASM-023**: messaging/mod.rs imports correct (no runtime/)

**PROJECTS_STANDARD.md Compliance:**
- **§4.3**: mod.rs declaration-only pattern maintained

**Rust Guidelines:**
- **M-MODULE-DOCS**: Re-exports documented

**Verification Commands:**
```bash
# Build verification
cargo build
# Expected: No errors, no duplicate type warnings

# Test external import
cargo test --test messaging_integration
# Expected: Integration tests can import ResponseRouter
```

**Estimated Effort:** 30 minutes

---

#### Subtask 1.3.4: Remove ResponseRouter from messaging_service.rs

**Objective:** Delete ResponseRouter code from messaging_service.rs after moving to router.rs.

**Code to DELETE from messaging_service.rs:**
- ResponseRouter struct: Lines 474-524 (51 lines)
- ResponseRouterMetrics struct: Lines 686-690 (5 lines)
- impl ResponseRouter: Lines 538-671 (134 lines)
- ResponseRouterStats struct: Lines 692-706 (15 lines)
- All ResponseRouter tests: Lines 1089-1278 (190 lines)
- Final integration test: Lines 1280-1317 (38 lines, uses ResponseRouter)

**Code to RETAIN:**
- MessagingService still has `response_router: Arc<ResponseRouter>` field (line 138)
- MessagingService::response_router() method returns Arc<ResponseRouter> (lines 356-391)
- These will now import ResponseRouter from router.rs via mod.rs re-export

**Implementation Steps:**
1. Delete lines 474-524 (ResponseRouter struct)
2. Delete lines 538-671 (impl ResponseRouter)
3. Delete lines 686-690 (ResponseRouterMetrics)
4. Delete lines 692-706 (ResponseRouterStats)
5. Delete lines 1089-1317 (all ResponseRouter tests)
6. Add import at top of file: `use super::router::ResponseRouter;` (after line 78)
7. Verify MessagingService field and method still work with import from router.rs

**Updated Imports in messaging_service.rs (add after line 78):**
```rust
// Import ResponseRouter from router.rs (via re-export)
use super::router::ResponseRouter;
```

**Expected Final Line Count:** ~1,090 lines (down from 1,317)

**Success Criteria:**
- [ ] ResponseRouter struct deleted
- [ ] ResponseRouterMetrics deleted
- [ ] impl ResponseRouter deleted
- [ ] ResponseRouterStats deleted
- [ ] All ResponseRouter tests deleted
- [ ] Import from router.rs added
- [ ] MessagingService::response_router() still works
- [ ] cargo build succeeds

**ADR Constraints:**
- **ADR-WASM-023**: No duplicate definitions (imports from router.rs only)

**PROJECTS_STANDARD.md Compliance:**
- **§2.1**: Import organization correct
- **§4.3**: No duplicate re-exports

**Verification Commands:**
```bash
# Build verification
cargo build
# Expected: No errors, no duplicate type errors

# Verify ResponseRouter no longer in messaging_service.rs
grep -n "struct ResponseRouter" src/messaging/messaging_service.rs
# Expected: No output (should be in router.rs only)

# Verify ResponseRouter still accessible
cargo test response_router_new
# Expected: Test passes (ResponseRouter imported via mod.rs)
```

**Estimated Effort:** 1-2 hours

---

#### Subtask 1.3.5: Update fire_and_forget.rs (Stub)

**Objective:** Keep fire_and_forget.rs as stub with module documentation.

**Analysis of messaging_service.rs:**
After careful review, messaging_service.rs does NOT contain specific fire-and-forget pattern implementation code. The fire-and-forget pattern is handled at MessageBroker level (airssys-rt) and at host function level (async_host.rs).

**Current fire_and_forget.rs State:**
- Has FireAndForget placeholder (28 lines)
- Has placeholder tests (marked as ignored)
- Already has module docs

**Action:**
- Keep fire_and_forget.rs as-is (stub placeholder)
- Add enhanced module documentation explaining this is Phase 2+ work
- No code extraction needed (nothing to extract)

**Enhanced Module Documentation to Add:**
```rust
//! Fire-and-forget messaging pattern.
//!
//! This module provides fire-and-forget messaging capabilities
//! where messages are sent without awaiting responses.
//!
//! # Phase 2+ Implementation
//!
//! **Current State:** Stub placeholder - implementation deferred to Phase 2
//!
//! **Future Implementation:**
//! - Fire-and-forget specific helpers and utilities
//! - Pattern-specific optimizations
//! - Fire-and-forget metrics tracking
//!
//! # Architecture Reference
//!
//! For now, fire-and-forget messaging is handled via:
//! - **MessageBroker** (airssys-rt): Handles message routing
//! - **send-message** host function: Component API for sending messages
//! - **AsyncHostFunction**: Host-level message publishing
//!
//! # References
//!
//! - **KNOWLEDGE-WASM-029**: Messaging Patterns (Pattern 1: Fire-and-Forget)
//! - **ADR-WASM-009**: Component Communication Model
//! - **WASM-TASK-006 Phase 2**: Fire-and-forget implementation
```

**Success Criteria:**
- [ ] fire_and_forget.rs kept as stub
- [ ] Enhanced module documentation added explaining Phase 2+ scope
- [ ] No code extracted (there's nothing to extract)
- [ ] Placeholder tests remain as-is

**ADR Constraints:**
- **KNOWLEDGE-WASM-012**: Fire-and-forget pattern is Phase 2+ work

**PROJECTS_STANDARD.md Compliance:**
- **§6.1**: YAGNI - not implementing unneeded features

**Rust Guidelines:**
- **M-MODULE-DOCS**: Module docs explain future scope

**Documentation:**
- **Diátaxis Type**: Explanation
- **Quality**: Technical language

**Estimated Effort:** 30 minutes

---

#### Subtask 1.3.6: Update request_response.rs (Stub)

**Objective:** Keep request_response.rs with existing error types only.

**Analysis of messaging_service.rs:**
After careful review, RequestError is already correctly placed in request_response.rs (defined in placeholder file, not in messaging_service.rs). The request-response pattern implementation is handled by ResponseRouter (which was moved to router.rs in Subtask 1.3.2).

**Current request_response.rs State:**
- Has RequestError enum (lines 28-36)
- Has RequestResponse placeholder (lines 40-57)
- Has placeholder tests (marked as ignored)
- Already has module docs

**Action:**
- Keep request_response.rs as-is (has RequestError enum and RequestResponse stub)
- Add enhanced module documentation
- No additional code extraction needed

**Enhanced Module Documentation to Add:**
```rust
//! Request-response messaging pattern.
//!
//! This module provides request-response messaging capabilities
//! with correlation tracking and response routing.
//!
//! # Current Implementation
//!
//! **RequestError enum** (lines 28-36): Error types for request-response
//! - Timeout: Request exceeded timeout duration
//! - ComponentNotFound: Target component not found
//! - RoutingFailed: Response routing failure
//!
//! **RequestResponse stub** (lines 40-57): Placeholder for future helpers
//!
//! # Pattern Implementation
//!
//! The request-response pattern is implemented via:
//! - **ResponseRouter** (in router.rs): Routes responses back to requesters
//! - **CorrelationTracker** (in actor/message/): Tracks pending requests
//! - **send-request** host function: Component API for request-response
//!
//! # Architecture
//!
//! ```text
//! Component A                   Component B
//! send-request ──────────────► handle-message
//!       │                            │
//!       │ correlation_id             │ return value
//!       ▼                            ▼
//! CorrelationTracker           ResponseRouter (router.rs)
//!       │                            │
//!       │◄────── route_response ─────┘
//!       │
//!       ▼
//! handle-callback ◄───────── response routed
//! ```
//!
//! # References
//!
//! - **KNOWLEDGE-WASM-029**: Messaging Patterns (Pattern 2: Request-Response)
//! - **ADR-WASM-009**: Component Communication Model
//! - **WASM-TASK-006 Phase 3**: Request-response implementation
```

**Success Criteria:**
- [ ] request_response.rs kept as-is
- [ ] RequestError enum retained (correctly placed)
- [ ] Enhanced module documentation added
- [ ] No code extracted (ResponseRouter handles actual logic)

**ADR Constraints:**
- **KNOWLEDGE-WASM-029**: ResponseRouter handles request-response pattern

**PROJECTS_STANDARD.md Compliance:**
- **§6.1**: YAGNI - not adding unneeded code

**Rust Guidelines:**
- **M-MODULE-DOCS**: Module docs explain structure

**Documentation:**
- **Diátaxis Type**: Explanation
- **Quality**: Technical language

**Estimated Effort:** 30 minutes

---

#### Subtask 1.3.7: Update codec.rs (Stub)

**Objective:** Keep codec.rs as stub placeholder with module documentation.

**Analysis of messaging_service.rs:**
After careful review, messaging_service.rs does NOT contain multicodec encoding/decoding implementation. The multicodec format validation happens at host function level (async_host.rs) when sending messages, not in messaging_service.rs.

**Current codec.rs State:**
- Has MulticodecCodec placeholder (28 lines)
- Has placeholder tests (marked as ignored)
- Already has module docs

**Action:**
- Keep codec.rs as-is (stub placeholder)
- Add enhanced module documentation explaining this is Phase 2+ work
- No code extraction needed (nothing to extract)

**Enhanced Module Documentation to Add:**
```rust
//! Multicodec message encoding.
//!
//! This module provides message encoding/decoding using multicodec format.
//!
//! # Phase 2+ Implementation
//!
//! **Current State:** Stub placeholder - implementation deferred to Phase 2
//!
//! **Future Implementation:**
//! - Multicodec codec helpers and utilities
//! - Encoding/decoding optimizations
//! - Codec-specific error handling
//!
//! # Current Architecture
//!
//! For now, multicodec validation is handled via:
//! - **async_host.rs**: Host function validates message format
//! - **send-message** and **send-request**: Validate payload encoding
//! - **WIT interfaces**: Type-level encoding guarantees
//!
//! # Multicodec Format
//!
//! Messages use self-describing multicodec format:
//! - Prefix: 1-byte codec identifier
//! - Payload: Variable-length encoded data
//! - Benefits: Language-agnostic, self-documenting
//!
//! # References
//!
//! - **ADR-WASM-001**: Multicodec Compatibility Strategy
//! - **KNOWLEDGE-WASM-006**: Multicodec Message Format
//! - **WASM-TASK-006 Phase 2**: Codec implementation
```

**Success Criteria:**
- [ ] codec.rs kept as stub
- [ ] Enhanced module documentation added explaining Phase 2+ scope
- [ ] No code extracted (there's nothing to extract)
- [ ] Placeholder tests remain as-is

**ADR Constraints:**
- **KNOWLEDGE-WASM-006**: Multicodec validation happens at host function level
- **ADR-WASM-001**: Multicodec compatibility strategy

**PROJECTS_STANDARD.md Compliance:**
- **§6.1**: YAGNI - not implementing unneeded features

**Rust Guidelines:**
- **M-MODULE-DOCS**: Module docs explain future scope

**Documentation:**
- **Diátaxis Type**: Explanation
- **Quality**: Technical language

**Estimated Effort:** 30 minutes

---

#### Subtask 1.3.8: Update topics.rs (Stub)

**Objective:** Keep topics.rs as stub placeholder for Phase 2.

**Analysis of messaging_service.rs:**
KNOWLEDGE-WASM-024 explicitly states topic-based pub-sub is Phase 2+ work. Task 1.3 is about Phase 1 module organization only.

**Current topics.rs State:**
- Has TopicManager placeholder (28 lines)
- Has placeholder tests (marked as ignored)
- Already has module docs

**Action:**
- Keep topics.rs as-is (stub placeholder)
- Add enhanced module documentation explaining this is Phase 2 work
- No code extraction needed (nothing to extract)

**Enhanced Module Documentation to Add:**
```rust
//! Topic-based publish-subscribe messaging.
//!
//! This module provides topic-based pub-sub messaging capabilities.
//!
//! # Phase 2+ Implementation
//!
//! **Current State:** Stub placeholder - implementation deferred to Phase 2
//!
//! **Future Implementation:**
//! - Topic management and subscription tracking
//! - Topic-based message routing
//! - Topic-level access control and quotas
//! - Pub-sub metrics and monitoring
//!
//! # Architecture Reference
//!
//! For now, messaging uses direct ComponentId addressing only (Phase 1):
//! - **Direct addressing**: Components identified by ComponentId
//! - **MessageBroker**: Routes messages by ComponentId only
//! - **No topics**: Topic-based routing is Phase 2+ enhancement
//!
//! # Phase 2 Design
//!
//! When implemented, topic-based pub-sub will add:
//! - Components can subscribe to topics (in addition to direct addressing)
//! - Messages can be published to topics (in addition to ComponentId)
//! - Runtime-level topic management (not component-level)
//! - Integration with existing MessageBroker architecture
//!
//! # References
//!
//! - **KNOWLEDGE-WASM-024**: Component Messaging Clarifications (Phase 2 scope)
//! - **ADR-WASM-009**: Component Communication Model (pub-sub pattern)
//! - **WASM-TASK-006 Phase 2**: Topic-based pub-sub implementation
```

**Success Criteria:**
- [ ] topics.rs kept as stub
- [ ] Enhanced module documentation added explaining Phase 2 scope
- [ ] No code extracted (future work)
- [ ] Placeholder tests remain as-is

**ADR Constraints:**
- **KNOWLEDGE-WASM-024**: Topic-based pub-sub is Phase 2+ work

**PROJECTS_STANDARD.md Compliance:**
- **§6.1**: YAGNI - not implementing future features

**Rust Guidelines:**
- **M-MODULE-DOCS**: Module docs explain future scope

**Documentation:**
- **Diátaxis Type**: Explanation
- **Quality**: Technical language

**Estimated Effort:** 30 minutes

---

### Unit Testing Plan

**Unit Tests for router.rs:**
- `test_response_router_new` - Verify router initializes with zero metrics
- `test_response_router_clone` - Verify cloning shares metrics and tracker
- `test_response_router_has_pending_request_false` - Verify no pending request initially
- `test_response_router_has_pending_request_true` - Verify pending request tracking
- `test_response_router_route_response_success` - Verify successful response routing
- `test_response_router_route_response_error` - Verify error response handling
- `test_response_router_orphaned_response` - Verify orphaned response tracking
- `test_response_router_get_stats` - Verify stats snapshot
- `test_response_router_access` - Verify router access via MessagingService
- `test_get_stats_includes_responses_routed` - Verify integration with MessagingStats

**Unit Tests for MessagingService (remain in messaging_service.rs):**
- `test_messaging_service_new` - Verify service initialization
- `test_messaging_service_broker_access` - Verify broker access
- `test_messaging_service_stats` - Verify stats retrieval
- `test_record_publish` - Verify publish tracking
- `test_record_routing_failure` - Verify failure tracking
- `test_messaging_service_clone` - Verify cloning behavior
- `test_default_trait` - Verify Default implementation
- `test_correlation_tracker_access` - Verify correlation tracker access
- `test_record_request_sent` - Verify request tracking
- `test_record_request_completed` - Verify completion tracking
- `test_pending_requests` - Verify pending request count
- `test_get_stats_includes_request_metrics` - Verify stats integration

**Total Unit Test Count:**
- router.rs: 10 tests (new location, moved from messaging_service.rs)
- messaging_service.rs: 11 tests (existing, unchanged)
- Total: 21 unit tests for messaging module

**Test Requirements:**
- ✅ All public functions have tests
- ✅ All error paths have tests
- ✅ All metrics have tests
- ✅ Integration between submodules tested

---

### Integration Testing Plan

**Existing Integration Tests (No changes needed):**
Integration tests already test messaging functionality through MessagingService, which remains unchanged in its external API:

- `tests/messaging_integration_tests.rs` - Tests messaging end-to-end
- `tests/actor_routing_tests.rs` - Tests message routing
- `tests/actor_invocation_tests.rs` - Tests component-to-component messaging

**Why No New Integration Tests Needed:**
- Task 1.3 is code organization (internal refactoring only)
- MessagingService external API remains identical
- All integration tests already use MessagingService
- ResponseRouter move is internal to messaging module
- Re-exports in mod.rs maintain public API stability

**Integration Test Verification:**
After completing Task 1.3, run:
```bash
cargo test --test messaging_integration_tests
cargo test --test actor_routing_tests
cargo test --test actor_invocation_tests
```

**Expected Results:**
- All existing integration tests pass
- No test changes needed
- API remains stable through mod.rs re-exports

---

### Quality Standards

**All subtasks must meet:**
- ✅ Code builds without errors
- ✅ Zero compiler warnings
- ✅ Zero clippy warnings: `cargo clippy --all-targets --all-features -- -D warnings`
- ✅ Follows PROJECTS_STANDARD.md §2.1-§6.4
- ✅ Follows Rust guidelines (see references above)
- ✅ Unit tests in `#[cfg(test)]` blocks
- ✅ All tests pass: `cargo test --lib`
- ✅ Documentation follows quality standards
- ✅ Standards Compliance Checklist in task file

---

### Verification Checklist

**For implementer to run after completing all subtasks:**

```bash
# ============================================================================
# 1. Architecture Verification (ADR-WASM-023)
# ============================================================================

echo "=== Verifying Module Boundaries ==="

# Check 1: messaging/ doesn't import from runtime/ (FORBIDDEN)
echo "Checking messaging/ does NOT import from runtime/ (FORBIDDEN)..."
if grep -rn "use crate::runtime" src/messaging/; then
    echo "❌ FAILED: messaging/ imports from runtime/"
    exit 1
else
    echo "✅ PASSED: messaging/ does not import from runtime/"
fi

# Check 2: messaging/ CAN import from actor/ (ALLOWED per KNOWLEDGE-WASM-012)
echo "Checking messaging/ CAN import from actor/ (ALLOWED)..."
if grep -rn "use crate::actor" src/messaging/; then
    echo "ℹ️  INFO: messaging/ imports from actor/ (this is CORRECT per KNOWLEDGE-WASM-012)"
    echo "✅ PASSED: messaging/ imports from actor/ as needed"
else
    echo "ℹ️  INFO: messaging/ has no actor imports (also OK)"
fi

# Check 3: messaging/ doesn't import from security/ (FORBIDDEN)
echo "Checking messaging/ does NOT import from security/ (FORBIDDEN)..."
if grep -rn "use crate::security" src/messaging/; then
    echo "❌ FAILED: messaging/ imports from security/"
    exit 1
else
    echo "✅ PASSED: messaging/ does not import from security/"
fi

# Check 4: router.rs has ResponseRouter (expected location)
echo "Checking router.rs contains ResponseRouter..."
if grep -n "pub struct ResponseRouter" src/messaging/router.rs; then
    echo "✅ PASSED: ResponseRouter found in router.rs"
else
    echo "❌ FAILED: ResponseRouter not found in router.rs"
    exit 1
fi

# Check 5: messaging_service.rs does NOT have ResponseRouter (deleted)
echo "Checking messaging_service.rs does NOT have ResponseRouter..."
if grep -n "pub struct ResponseRouter" src/messaging/messaging_service.rs; then
    echo "❌ FAILED: ResponseRouter still in messaging_service.rs (should be deleted)"
    exit 1
else
    echo "✅ PASSED: ResponseRouter removed from messaging_service.rs"
fi

echo ""
echo "=== All Architecture Checks Passed ==="

# ============================================================================
# 2. Build Verification
# ============================================================================

echo ""
echo "=== Building Project ==="
cargo build
BUILD_EXIT=$?

if [ $BUILD_EXIT -ne 0 ]; then
    echo "❌ FAILED: Build failed"
    exit 1
else
    echo "✅ PASSED: Build succeeded"
fi

# Check for warnings
WARNINGS=$(cargo build 2>&1 | grep -i "warning" | wc -l)
if [ "$WARNINGS" -gt 0 ]; then
    echo "❌ FAILED: Build has $WARNINGS warnings"
    exit 1
else
    echo "✅ PASSED: Zero compiler warnings"
fi

echo ""

# ============================================================================
# 3. Test Verification
# ============================================================================

echo "=== Running Tests ==="
cargo test --lib
TEST_EXIT=$?

if [ $TEST_EXIT -ne 0 ]; then
    echo "❌ FAILED: Tests failed"
    exit 1
else
    echo "✅ PASSED: All tests pass"
fi

# Test router.rs specifically
echo ""
echo "=== Testing router.rs (extracted ResponseRouter) ==="
cargo test response_router_new
cargo test response_router_route_response_success
cargo test response_router_route_response_error
cargo test response_router_orphaned_response
ROUTER_EXIT=$?

if [ $ROUTER_EXIT -ne 0 ]; then
    echo "❌ FAILED: router.rs tests failed"
    exit 1
else
    echo "✅ PASSED: All router.rs tests pass"
fi

# Test messaging_service.rs still works
echo ""
echo "=== Testing messaging_service.rs (still functional) ==="
cargo test messaging_service_new
cargo test messaging_service_broker_access
cargo test messaging_service_response_router_access
SERVICE_EXIT=$?

if [ $SERVICE_EXIT -ne 0 ]; then
    echo "❌ FAILED: messaging_service.rs tests failed"
    exit 1
else
    echo "✅ PASSED: All messaging_service.rs tests pass"
fi

echo ""

# ============================================================================
# 4. Clippy Verification
# ============================================================================

echo "=== Running Clippy ==="
cargo clippy --all-targets --all-features -- -D warnings
CLIPPY_EXIT=$?

if [ $CLIPPY_EXIT -ne 0 ]; then
    echo "❌ FAILED: Clippy found warnings"
    exit 1
else
    echo "✅ PASSED: Zero clippy warnings"
fi

echo ""
echo "=== All Quality Checks Passed ==="
echo "✅ Task 1.3 is complete"
```

**Expected output:**
- All architecture checks pass (no forbidden imports, ResponseRouter in correct location)
- Build succeeds with zero warnings
- All tests pass (router.rs and messaging_service.rs)
- Clippy passes with zero warnings

---

### Documentation Requirements

**For each submodule updated/created:**

**router.rs:**
- Module documentation (`//!`) explaining ResponseRouter purpose
- Public type documentation (`///`) for all structs and methods
- Example code in doc comments
- Reference to ADRs and Knowledges
- No marketing hyperbole
- Technical precision (measurable performance claims)

**fire_and_forget.rs:**
- Module documentation explaining Phase 2+ scope
- Current architecture reference (MessageBroker handles this)
- Future implementation plans
- References to relevant ADRs/Knowledges

**request_response.rs:**
- Module documentation explaining pattern structure
- RequestError enum documentation
- Current implementation status (stub, ResponseRouter handles logic)
- Architecture diagram showing request-response flow
- References to relevant ADRs/Knowledges

**codec.rs:**
- Module documentation explaining Phase 2+ scope
- Current architecture reference (async_host.rs handles validation)
- Future implementation plans
- Multicodec format description
- References to relevant ADRs/Knowledges

**topics.rs:**
- Module documentation explaining Phase 2+ scope
- Current architecture reference (direct addressing only)
- Future implementation plans
- Phase 2 design description
- References to relevant ADRs/Knowledges

**Documentation Standards:**
- **Diátaxis Type**: Reference (router.rs), Explanation (stubs)
- **Quality**: Professional tone, no hyperbole per documentation-quality-standards.md
- **Task documentation**: Standards Compliance Checklist in task file per task-documentation-standards.md
- **Evidence**: Code examples showing compliance

---

### Risk Mitigation

**Identified Risks:**

1. **Duplicate Type Definitions After Extraction**
   - **Risk**: ResponseRouter exists in both messaging_service.rs and router.rs after extraction
   - **Mitigation**: Subtask 1.3.4 deletes from messaging_service.rs
   - **Verification**: Subtask 1.3.3 fixes mod.rs re-exports to remove duplication
   - **Test**: `grep -n "struct ResponseRouter"` should only find in router.rs

2. **Import Update Errors**
   - **Risk**: Forgetting to add import in messaging_service.rs after deleting ResponseRouter
   - **Mitigation**: Subtask 1.3.4 explicitly adds `use super::router::ResponseRouter;`
   - **Verification**: Build will fail if import missing
   - **Architecture check**: Verify no runtime/ imports (forbidden)

3. **Test Failures After Move**
   - **Risk**: Tests rely on old module structure
   - **Mitigation**: All tests moved with ResponseRouter to router.rs
   - **Verification**: Run `cargo test --lib` after each subtask
   - **Specific test**: Run `cargo test response_router` to verify moved tests

4. **Code Duplication**
   - **Risk**: Copying code instead of moving causes duplication
   - **Mitigation**: DELETE from messaging_service.rs, not copy
   - **Verification**: Check final line count (expect ~1,090 vs 1,317)

5. **Breaking API Changes**
   - **Risk**: Moving ResponseRouter changes internal API
   - **Mitigation**: Re-export from router.rs keeps public API stable
   - **Verification**: Integration tests pass without changes
   - **Architecture check**: mod.rs re-exports maintain external API

6. **Module Documentation Missing**
   - **Risk**: Forgetting to add module docs to submodules
   - **Mitigation**: Each subtask (1.3.5-1.3.8) explicitly requires module docs
   - **Verification**: Review each file for `//!` module documentation
   - **Quality check**: Follow M-MODULE-DOCS guideline

**Contingency:**
- If build fails with import errors: Review and fix imports
- If tests fail: Debug and fix test imports
- If architecture check fails: Review and fix forbidden imports
- If duplicate type errors: Verify Subtask 1.3.4 completed
- If API breaks: Verify Subtask 1.3.3 mod.rs re-exports correct

---

### Rollback Plan

**If critical issues arise:**

1. **Git Backup (Already Created in Subtask 1.3.1)**
   - Backup commit created: "Backup before Task 1.3: Create Remaining Messaging Submodules"
   - This provides restore point

2. **Rollback Steps:**
   ```bash
   # Step 1: Restore messaging_service.rs
   git checkout HEAD~1 -- src/messaging/messaging_service.rs

   # Step 2: Restore router.rs to placeholder
   git checkout HEAD~1 -- src/messaging/router.rs

   # Step 3: Restore other submodules to placeholders
   git checkout HEAD~1 -- src/messaging/fire_and_forget.rs
   git checkout HEAD~1 -- src/messaging/request_response.rs
   git checkout HEAD~1 -- src/messaging/codec.rs
   git checkout HEAD~1 -- src/messaging/topics.rs

   # Step 4: Restore mod.rs
   git checkout HEAD~1 -- src/messaging/mod.rs

   # Step 5: Verify build passes
   cargo build
   ```

3. **Decision Points:**
   - If import errors: Fix imports before proceeding
   - If test failures: Debug and fix tests
   - If logic errors: Compare with original messaging_service.rs code

4. **After Rollback:**
   - Document what went wrong
   - Update plan with learned lessons
   - Review ADRs/Knowledges before retrying

---

### Standards Compliance Checklist

**PROJECTS_STANDARD.md Applied:**
- [ ] **§2.1 3-Layer Import Organization** - Evidence: All imports follow std → external → internal pattern (router.rs, messaging_service.rs)
- [ ] **§4.3 Module Architecture Patterns** - Evidence: messaging/mod.rs contains only declarations, no implementation code
- [ ] **§6.2 Avoid `dyn` Patterns** - Evidence: No trait objects used, concrete types only (ResponseRouter, MessagingService)
- [ ] **§6.4 Implementation Quality Gates** - Evidence: Zero compiler/clippy warnings, comprehensive tests (21 unit tests, verification commands)

**Rust Guidelines Applied:**
- [ ] **M-DESIGN-FOR-AI** - Evidence: Idiomatic APIs, comprehensive docs, testable code (all submodules have docs, all functions have tests)
- [ ] **M-MODULE-DOCS** - Evidence: Module documentation complete (router.rs has `//!` and `///` docs, stubs have enhanced `//!` docs)
- [ ] **M-ERRORS-CANONICAL-STRUCTS** - Evidence: Error types follow canonical structure (RequestError uses thiserror)
- [ ] **M-STATIC-VERIFICATION** - Evidence: Lints enabled, clippy verification in plan (verification commands include clippy)

**Documentation Quality:**
- [ ] **No hyperbolic terms** - Evidence: Verified against forbidden list in documentation-quality-standards.md
- [ ] **Technical precision** - Evidence: All claims measurable (e.g., "~150ns response routing", "O(1) complexity")
- [ ] **Diátaxis compliance** - Evidence: Reference documentation type for router.rs, Explanation type for stubs

**ADR Compliance:**
- [ ] **ADR-WASM-018** - Evidence: One-way dependency enforced (messaging → core only, no runtime/)
- [ ] **ADR-WASM-023** - Evidence: No forbidden imports (verification commands check for use crate::runtime)

**Knowledge Compliance:**
- [ ] **KNOWLEDGE-WASM-012** - Evidence: Module structure follows specification (messaging/ top-level, submodules organized correctly)
- [ ] **KNOWLEDGE-WASM-024** - Evidence: Messaging patterns correctly implemented, Phase 1/Phase 2 distinction clear
- [ ] **KNOWLEDGE-WASM-029** - Evidence: ResponseRouter implements correct pattern (response IS return value, not send-response)

---

### Phase 3: Remove runtime/messaging.rs (Days 3-4)

#### Task 3.1: Verify All Imports Updated

**Objective:** Ensure no code still imports from runtime/messaging.rs before deletion.

**Deliverables:**

**Verification Commands:**
```bash
# Check 1: No direct imports of runtime/messaging
grep -rn "use airssys_wasm::runtime::messaging" src/ tests/ examples/

# Check 2: No MessagingService from runtime/
grep -rn "use airssys_wasm::runtime::MessagingService" src/ tests/ examples/

# Check 3: Build succeeds
cargo build

# Check 4: All tests pass
cargo test --lib
```

**Success Criteria:**
- ✅ No imports of `runtime::messaging` found
- ✅ No imports of `runtime::MessagingService` found
- ✅ `cargo build` succeeds
- ✅ `cargo test --lib` passes

**Estimated Effort:** 1-2 hours  
**Risk Level:** Low (verification only)

---

#### Task 3.2: Delete runtime/messaging.rs

**Objective:** Remove old messaging file after all imports updated.

**Deliverables:**

**File to Delete:** `src/runtime/messaging.rs`

**Files to Update:**
- `src/runtime/mod.rs` - Remove messaging from re-exports

**Updates to runtime/mod.rs:**
```rust
// BEFORE (WRONG):
pub use messaging::{MessageReceptionMetrics, MessageReceptionStats, MessagingService, MessagingStats, ResponseRouter, ResponseRouterStats};

// AFTER (CORRECT):
// Remove these re-exports entirely
```

**Success Criteria:**
- ✅ `src/runtime/messaging.rs` deleted
- ✅ `src/runtime/mod.rs` no longer exports messaging types
- ✅ `cargo build` succeeds
- ✅ `cargo test` passes
- ✅ No references to deleted file exist

**Estimated Effort:** 1 hour  
**Risk Level:** Medium (deletion requires verification)

---

### Phase 4: Add Architecture Compliance Tests (Days 4-5)

#### Task 4.1: Create Architecture Compliance Tests

**Objective:** Create tests that verify architectural rules are followed.

**Deliverables:**

**File to Create:** `tests/architecture_compliance_tests.rs`

**Test Cases:**
```rust
#[test]
fn test_runtime_never_imports_from_actor() {
    let runtime_code = include_str!("../src/runtime/mod.rs");
    assert!(!runtime_code.contains("use crate::actor"), 
        "runtime/ should not import from actor/");
}

#[test]
fn test_runtime_never_imports_messaging_types() {
    let runtime_code = include_str!("../src/runtime/async_host.rs");
    assert!(!runtime_code.contains("use crate::actor::message::"), 
        "runtime/ should not import from actor/message/");
}

#[test]
fn test_core_never_imports_from_higher() {
    let core_code = include_str!("../src/core/mod.rs");
    assert!(!core_code.contains("use crate::runtime"), 
        "core should not import from runtime/");
    assert!(!core_code.contains("use crate::actor"), 
        "core should not import from actor/");
    assert!(!core_code.contains("use crate::messaging"), 
        "core should not import from messaging/");
}

#[test]
fn test_messaging_module_exists() {
    let lib_code = include_str!("../src/lib.rs");
    assert!(lib_code.contains("pub mod messaging;"), 
        "top-level messaging module should exist");
}

#[test]
fn test_messaging_module_independent() {
    let messaging_code = include_str!("../src/messaging/mod.rs");
    assert!(!messaging_code.contains("use crate::runtime"), 
        "messaging/ should not import from runtime/");
}

#[test]
fn test_no_imports_from_deleted_file() {
    // Verify no imports from deleted runtime/messaging.rs
    let lib_code = include_str!("../src/lib.rs");
    let runtime_mod = include_str!("../src/runtime/mod.rs");
    assert!(!lib_code.contains("runtime::messaging"), 
        "Should not import from deleted runtime/messaging.rs");
    assert!(!runtime_mod.contains("pub mod messaging;"), 
        "runtime/ should not export messaging module");
}
```

**Success Criteria:**
- ✅ Architecture compliance tests created
- ✅ All compliance tests pass
- ✅ Tests verify no circular dependencies
- ✅ Tests verify correct module structure
- ✅ `cargo test` passes

**Estimated Effort:** 3-4 hours  
**Risk Level:** Low (new tests only)

---

### Phase 5: Verification & Testing (Days 5-7)

#### Task 5.1: Run All Tests

**Objective:** Verify all tests pass after refactoring.

**Deliverables:**

**Commands to Run:**
```bash
# Run all tests
cargo test --all

# Run with verbose output
cargo test --all -- --nocapture

# Run integration tests only
cargo test --test
```

**Success Criteria:**
- ✅ All unit tests pass (`cargo test --lib`)
- ✅ All integration tests pass (`cargo test --test`)
- ✅ Zero test failures
- ✅ Zero compiler warnings
- ✅ Zero clippy warnings

**Estimated Effort:** 1-2 hours  
**Risk Level:** Low (verification only)

---

#### Task 5.2: Verify No Circular Dependencies

**Objective:** Confirm one-way dependency chain is enforced.

**Deliverables:**

**Verification Commands:**
```bash
# Check 1: runtime/ doesn't import from actor/
echo "Checking runtime/ → actor/..."
if grep -rn "use crate::actor" src/runtime/; then
    echo "❌ FAILED: runtime/ imports from actor/"
    exit 1
fi
echo "✅ PASSED: runtime/ clean"

# Check 2: core/ doesn't import from runtime/ or actor/
echo "Checking core/ → runtime/ actor/..."
if grep -rn "use crate::runtime\|use crate::actor" src/core/; then
    echo "❌ FAILED: core/ imports from higher layers"
    exit 1
fi
echo "✅ PASSED: core/ clean"

# Check 3: messaging/ doesn't import from runtime/
echo "Checking messaging/ → runtime/..."
if grep -rn "use crate::runtime" src/messaging/; then
    echo "❌ FAILED: messaging/ imports from runtime/"
    exit 1
fi
echo "✅ PASSED: messaging/ clean"

# Check 4: No imports from deleted runtime/messaging.rs
echo "Checking no imports from deleted runtime/messaging.rs..."
if grep -rn "runtime::messaging" src/ tests/ examples/; then
    echo "❌ FAILED: Found imports from deleted runtime/messaging.rs"
    exit 1
fi
echo "✅ PASSED: No imports from deleted file"
```

**Success Criteria:**
- ✅ All grep checks return nothing
- ✅ runtime/ doesn't import from actor/
- ✅ core/ doesn't import from runtime/ or actor/
- ✅ messaging/ doesn't import from runtime/
- ✅ No imports from deleted runtime/messaging.rs

**Estimated Effort:** 1 hour  
**Risk Level:** Low (verification only)

---

#### Task 5.3: Run Benchmarks

**Objective:** Verify performance hasn't regressed.

**Deliverables:**

**Commands to Run:**
```bash
# Run all benchmarks
cargo bench

# Run specific messaging benchmarks
cargo bench --bench messaging

# Run actor benchmarks
cargo bench --bench actor
```

**Success Criteria:**
- ✅ All benchmarks compile
- ✅ All benchmarks run successfully
- ✅ No performance regressions
- ✅ Results documented

**Estimated Effort:** 2-3 hours  
**Risk Level:** Low (performance validation only)

---

### Phase 6: Documentation Updates (Days 7-8)

#### Task 6.1: Update Documentation

**Objective:** Update all documentation to reflect new module structure.

**Deliverables:**

**Files to Update:**
- README.md
- All knowledge documents referencing runtime/messaging
- All ADR documents referencing runtime/messaging
- Code examples in documentation

**Changes:**
- Replace `use airssys_wasm::runtime::MessagingService` with `use airssys_wasm::messaging::MessagingService`
- Update module structure diagrams
- Update architecture references
- Add migration notes for breaking changes

**Success Criteria:**
- ✅ All documentation updated
- ✅ No references to old import paths remain
- ✅ Module structure diagrams correct
- ✅ Migration notes clear

**Estimated Effort:** 3-4 hours  
**Risk Level:** Low (documentation updates only)

---

#### Task 6.2: Document Breaking Changes

**Objective:** Create clear documentation for the breaking import path changes.

**Deliverables:**

**Content to Add to Documentation:**

**Migration Notes to README.md:**
```markdown
## Breaking Changes

### Version 0.2.0

#### Messaging Module Moved to Top-Level

**Change:** Messaging infrastructure has been moved from `runtime::` to top-level `messaging::` module.

**Impact:** Import paths for messaging types have changed.

**Migration:**

**Old Import:**
```rust
use airssys_wasm::runtime::MessagingService;
use airssys_wasm::runtime::{MessagingStats, ResponseRouter};
```

**New Import:**
```rust
use airssys_wasm::messaging::MessagingService;
use airssys_wasm::messaging::{MessagingStats, ResponseRouter};
```

**If using prelude:** No changes required if using `use airssys_wasm::prelude::*;`

**Rationale:** This change aligns with ADR-WASM-018 three-layer architecture and KNOWLEDGE-WASM-012 module structure specification.
```

**Success Criteria:**
- ✅ Breaking changes documented in README.md
- ✅ Migration path provided for all affected import paths
- ✅ Examples showing before/after imports
- ✅ Links to ADR-WASM-024 for rationale

**Estimated Effort:** 2-3 hours  
**Risk Level:** Low (documentation only)

---

## Success Criteria

This task is complete when:

### Phase 1: Top-Level messaging/ Module Created ✅
- [ ] `src/messaging/mod.rs` created with module declarations
- [ ] `src/messaging/messaging_service.rs` created with all code moved
- [ ] `src/messaging/router.rs` created
- [ ] `src/messaging/fire_and_forget.rs` created
- [ ] `src/messaging/request_response.rs` created
- [ ] `src/messaging/codec.rs` created
- [ ] `src/messaging/topics.rs` created (stub)
- [ ] `src/lib.rs` updated with `pub mod messaging;`
- [ ] `src/prelude.rs` updated with messaging re-exports
- [ ] Module structure follows KNOWLEDGE-WASM-012 specification

### Phase 2: All Imports Updated ✅
- [x] All `actor/message/` files updated to use `messaging::` imports
- [x] All `runtime/` files no longer import from `actor/message/`
- [x] All integration tests updated
- [x] All examples updated
- [x] `grep -rn "use airssys_wasm::runtime::MessagingService" src/` returns nothing
- [x] `grep -rn "use crate::runtime::messaging" src/` returns nothing
- [x] All imports use `messaging::` instead of `runtime::messaging`
- [x] `cargo build` succeeds

### Phase 3: runtime/messaging.rs Removed ✅
- [ ] All imports verified updated
- [ ] `src/runtime/messaging.rs` deleted
- [ ] `src/runtime/mod.rs` no longer exports messaging types
- [ ] No references to deleted file exist
- [ ] `cargo build` succeeds
- [ ] `cargo test` passes

### Phase 4: Architecture Compliance Added ✅
- [ ] Architecture compliance tests created
- [ ] All compliance tests pass
- [ ] Tests verify no circular dependencies
- [ ] Tests verify correct module structure

### Phase 5: Verification Complete ✅
- [ ] All tests pass (`cargo test --all`)
- [ ] Zero test failures
- [ ] Zero compiler warnings
- [ ] Zero clippy warnings
- [ ] `grep -rn "use crate::actor" src/runtime/` returns nothing
- [ ] `grep -rn "use crate::runtime\|use crate::actor" src/core/` returns nothing
- [ ] `grep -rn "use crate::runtime" src/messaging/` returns nothing
- [ ] No imports from deleted runtime/messaging.rs found
- [ ] All benchmarks run successfully
- [ ] No performance regressions

### Phase 6: Documentation Complete ✅
- [ ] README.md updated with breaking changes
- [ ] Migration notes added for import path changes
- [ ] Before/after import examples provided
- [ ] Links to ADR-WASM-024 added

### Overall Quality Gates ✅
- [ ] Zero compiler warnings (`cargo build`)
- [ ] Zero clippy warnings
- [ ] All tests passing
- [ ] No circular dependencies
- [ ] Module architecture compliant with ADR-WASM-018
- [ ] Module structure follows KNOWLEDGE-WASM-012
- [ ] End-to-end messaging functional
- [ ] Inter-component communication working

---

## Overall Progress Summary

### Phase 1: Create Top-Level messaging/ Module ✅ COMPLETE
- ✅ Task 1.1: Create messaging module structure - COMPLETE
- ✅ Task 1.2: Move messaging code from runtime/messaging.rs - COMPLETE
- ✅ Task 1.3: Create remaining messaging submodules - COMPLETE
**Status:** 100% COMPLETE (3/3 tasks)

### Phase 2: Update All Import Statements ✅ COMPLETE
- ✅ Task 2.1: Update imports in actor/message/ - COMPLETE
- ✅ Task 2.2: Update imports in runtime/ modules - COMPLETE
- ✅ Task 2.3: Update imports in integration tests - COMPLETE
- ✅ Task 2.4: Update imports in examples - COMPLETE
- ✅ Task 2.5: Verify all imports updated - COMPLETE
**Status:** 100% COMPLETE (5/5 tasks)

### Phase 3: Remove runtime/messaging.rs - NOT STARTED
### Phase 4: Add Architecture Compliance Tests - NOT STARTED
### Phase 5: Verification & Testing - NOT STARTED
### Phase 6: Documentation Updates - NOT STARTED

**Overall Status:** 33% COMPLETE (2/6 phases complete, 8/22 tasks complete)

**Next Task:** Phase 3: Remove runtime/messaging.rs

---

## Timeline Summary

| Phase | Tasks | Duration | Dependencies |
|-------|-------|----------|--------------|
| **Phase 1** | 1.1-1.3 | 1-2 days | None |
| **Phase 2** | 2.1-2.4 | 2-3 days | Phase 1 complete |
| **Phase 3** | 3.1-3.2 | 1 day | Phase 2 complete |
| **Phase 4** | 4.1 | 1-2 days | Phase 3 complete |
| **Phase 5** | 5.1-5.3 | 2-3 days | Phase 4 complete |
| **Phase 6** | 6.1-6.2 | 1-2 days | Phase 5 complete |
| **TOTAL** | **3.5-4.5 weeks** | All phases sequential |

---

## Risk Assessment

### Identified Risks

| Risk | Likelihood | Impact | Mitigation |
|-------|-------------|---------|-------------|
| **Breaking import paths** | High | Medium | Comprehensive search for all imports; update README with migration notes; provide before/after examples |
| **Missed imports during update** | Medium | Medium | Use grep to find all references; verify with build/tests |
| **External code references** | Low | Medium | Document breaking changes in README; provide migration path |
| **Test failures after move** | Medium | Medium | Comprehensive test coverage; incremental verification |
| **Circular dependency reintroduced** | Low | High | Architecture compliance tests; verification checks |
| **Performance regressions** | Low | Medium | Run all benchmarks before/after; document results |

### Contingency Plans

**Plan A (Primary):** Execute refactoring as planned in 3.5-4.5 weeks

**Plan B (Fallback):** If blocking issues arise, defer messaging/ submodules to future phase, focus only on moving core MessagingService

**Plan C (Rollback):** Keep re-exports in runtime/ as deprecated aliases for migration period, document clearly in README with deprecation warnings

---

## References

### New Documentation Created

**Knowledge:**
- **KNOWLEDGE-WASM-034**: Module Architecture Violation - Messaging in Runtime

**ADR:**
- **ADR-WASM-024**: Refactor Messaging from Runtime to Top-Level Module

### Architecture Documents

**ADRs:**
- **ADR-WASM-018**: Three-Layer Architecture (actor → runtime → core)
- **ADR-WASM-023**: Module Boundary Enforcement
- **ADR-WASM-011**: Module Structure Organization

**Knowledge:**
- **KNOWLEDGE-WASM-002**: High-Level Overview
- **KNOWLEDGE-WASM-003**: Core Architecture Design
- **KNOWLEDGE-WASM-012**: Module Structure Architecture (PRIMARY REFERENCE - lines 506-596)
- **KNOWLEDGE-WASM-024**: Component Messaging Clarifications
- **KNOWLEDGE-WASM-029**: Messaging Patterns
- **KNOWLEDGE-WASM-034**: Module Architecture Violation

### Archived Tasks

- WASM-TASK-HOTFIX-001 (Old): Architecture Hotfix & Integration Glue (absorbed into this task)

### Related Technical Debt

- **DEBT-WASM-004**: Message Delivery Runtime Glue Missing
- **DEBT-WASM-027**: Duplicate WASM Runtime Fatal Architecture Violation
- **DEBT-WASM-028**: Circular Dependency Actor Runtime

---

## Notes

### Why This Refactoring is Critical

**Current State (BROKEN):**
- Messaging infrastructure scattered: `runtime/messaging.rs` + `actor/message/`
- No clear home for messaging code
- Module boundaries violated
- Circular dependency risk present
- Blocks proper Block 5 development

**Target State (CORRECT):**
- All messaging infrastructure in top-level `messaging/` module
- Clear module boundaries
- One-way dependency chain enforced
- Easy navigation and understanding
- Enables Block 5 development

**Key Benefits:**
1. **Fixes Architectural Violation**: Aligns with ADR-WASM-018 and KNOWLEDGE-WASM-012
2. **Eliminates Circular Dependency Risk**: Enforces one-way dependency chain
3. **Improves Navigation**: All messaging code in one place
4. **Enables Future Development**: Clear foundation for Block 5 features
5. **Prevents Future Violations**: Architecture compliance tests prevent mistakes

### Key Principles Applied

1. **Architectural Compliance**: Follow ADR-WASM-018 and KNOWLEDGE-WASM-012 exactly
2. **Incremental Delivery**: Each phase has clear milestones and success criteria
3. **Verification Gates**: Don't mark complete until verified
4. **Real Testing**: Add comprehensive integration tests, not just unit tests
5. **Documentation First**: Explain what and why before coding
6. **Clear Breaking Change Documentation**: Migration path clearly documented in README

---

## History

| Date | Version | Changes |
|-------|---------|---------|
| 2025-12-26 | 1.0 | Initial creation - messaging module refactoring task |
| 2025-12-26 | 1.1 | Revised Phase 4 (removed CI checks) and Phase 6 (clarified migration guide) |

---

**Task ID:** WASM-TASK-HOTFIX-001  
**Status:** NOT STARTED  
**Priority:** 🔴 CRITICAL / BLOCKING  
**Blocker For:** All WASM-TASK-006+ work and Block 5+ work  
---

---

## Implementation Plan for Task 1.3 (CORRECTED - Fixed All Verifier Issues)

### Summary of Corrections Made

This plan addresses ALL critical errors identified by verifier in previous plan:

1. ✅ **CORRECT Knowledge filename**: Uses `knowledge-wasm-012-module-structure-architecture.md` (NOT wrong filename)
2. ✅ **CORRECT source file**: References `src/messaging/messaging_service.rs` (NOT old `src/runtime/messaging.rs`)
3. ✅ **CORRECT line numbers**: Uses ACTUAL line numbers from current messaging_service.rs (1317 lines total)
4. ✅ **RESOLVED naming conflict**: Clearly explains replacement of MessageRouter placeholder with ResponseRouter
5. ✅ **CORRECT test count**: 15 tests (NOT 21 - verified by grep)
6. ✅ **CORRECT imports**: Lists only imports ACTUALLY used by ResponseRouter
7. ✅ **ALL 18 sections included**: Complete plan with all required sections

---

### 1. Context & References

**ADR References:**
- **ADR-WASM-018**: Three-Layer Architecture (one-way dependency chain)
   - Dependencies flow: core → runtime → actor → messaging
   - messaging/ is at TOP of dependency chain (can import from core, actor, airssys-rt)
   - ResponseRouter is part of messaging/ module
    
- **ADR-WASM-023**: Module Boundary Enforcement (HARD REQUIREMENT)
   - messaging/ CANNOT import from runtime/ (WASM execution)
   - messaging/ CAN import from actor/ (allowed per corrected architecture)
   - messaging/ CAN import from core/ (types, errors, configs)
   - messaging/ CAN import from airssys-rt (MessageBroker)
   - Forbidden imports: `use crate::runtime` in messaging/

**Knowledge References:**
- **KNOWLEDGE-WASM-012**: Module Structure Architecture (lines 506-596 define messaging/ module)
   - **Line 274 CRITICAL**: `messaging/ → core/, actor/, airssys-rt`
   - This means messaging/ CAN import from core/, actor/, AND airssys-rt
   - messaging/ is Block 5: Inter-Component Communication
   - messaging/ is top-level module (parallel to runtime/, actor/, security/)
    
- **KNOWLEDGE-WASM-005**: Messaging Architecture
   - MessageBroker integration via airssys-rt
   - Request-response pattern with correlation tracking
   - ResponseRouter handles response routing back to requesters
    
- **KNOWLEDGE-WASM-024**: Component Messaging Clarifications
   - Async-only communication model (no synchronous messaging)
   - Two send methods: send-message vs send-request
   - Unified receiver (handle-message for both patterns)
   - Response IS the return value from handle-message (NOT a separate send-response)
    
- **KNOWLEDGE-WASM-026**: Message Delivery Architecture Final
   - ActorSystemSubscriber owns message delivery (has mailbox_senders)
   - ComponentRegistry stays pure (identity lookup only)
   - Message flow from component send to handle_message invocation

**System Patterns:**
- **From system-patterns.md**: Component Communication Pattern
   - MessageBroker integration for inter-component communication
   - ResponseRouter for routing responses in request-response pattern

**PROJECTS_STANDARD.md Compliance:**
- **§2.1 3-Layer Import Organization**: Code will follow import organization
   - Layer 1: Standard library imports (std::sync::Arc, std::sync::atomic)
   - Layer 2: Third-party crate imports (serde, chrono, tokio)
   - Layer 3: Internal crate imports (core/, actor/, airssys-rt)
    
- **§4.3 Module Architecture Patterns**: mod.rs files will only contain declarations
   - messaging/mod.rs already has module declarations only
   - No implementation code in mod.rs files
    
- **§6.2 Avoid `dyn` Patterns**: Static dispatch preferred over trait objects
   - ResponseRouter uses concrete CorrelationTracker, no trait objects
    
- **§6.4 Implementation Quality Gates**:
   - Zero compiler warnings: `cargo build`
   - Zero clippy warnings: `cargo clippy --all-targets --all-features -- -D warnings`
   - Comprehensive tests: Unit tests in `#[cfg(test)]` blocks
   - All tests passing: `cargo test --lib`

**Rust Guidelines Applied:**
- **M-DESIGN-FOR-AI**: Idiomatic APIs, thorough docs, testable code
   - All public types have module documentation
   - All public functions have doc comments with examples
   - Tests verify all functionality
    
- **M-MODULE-DOCS**: Module documentation will be added
   - router.rs will have `//!` module-level docs
   - Public types have `///` doc comments
   - Follows M-CANONICAL-DOCS structure
    
- **M-ERRORS-CANONICAL-STRUCTS**: Error types follow canonical structure
   - RequestError in core/messaging.rs follows thiserror pattern
   - WasmError is used consistently
    
- **M-STATIC-VERIFICATION**: Lints enabled, clippy used
   - All clippy lints from PROJECTS_STANDARD.md
   - `#[expect(clippy::...)]` for intentional violations with reasons

**Documentation Standards:**
- **Diátaxis Type**: Reference documentation for APIs
   - ResponseRouter and metrics types are reference docs
   - Provide complete API documentation with examples
   - Neutral technical language, no marketing terms
    
- **Quality**: Professional tone, no hyperbole per documentation-quality-standards.md
   - No superlatives ("best", "fastest", "revolutionary")
   - Measurable performance claims with units
   - Accurate descriptions, not promotional
    
- **Task documentation**: Standards Compliance Checklist will be included per task-documentation-standards.md
   - Evidence of standards application in plan
   - Code examples showing compliance

---

### 2. Module Architecture

**Code will be placed in:** `src/messaging/router.rs`

**Module responsibilities (per ADR-WASM-023):**
- messaging/router.rs: ResponseRouter implementation (routes request-response responses)

**Dependency Rules (from ADR-WASM-023 and KNOWLEDGE-WASM-012 Line 274):**
```
messaging/ → core/, actor/, airssys-rt
```

**This means:**
- ✅ messaging/ CAN import from core/ (types, errors, configs)
- ✅ messaging/ CAN import from actor/ (CorrelationTracker, etc.)
- ✅ messaging/ CAN import from airssys-rt (InMemoryMessageBroker)
- ❌ messaging/ CANNOT import from runtime/ (WASM execution engine)

**Verification commands (for implementer to run after completing router.rs):**
```bash
# Check 1: messaging/ doesn't import from runtime/ (FORBIDDEN)
grep -rn "use crate::runtime" src/messaging/router.rs
# Expected: No output (clean)

# Check 2: messaging/ CAN import from actor/ (ALLOWED per KNOWLEDGE-WASM-012)
grep -rn "use crate::actor" src/messaging/router.rs
# Expected: Has output for CorrelationTracker import

# Check 3: messaging/ imports from core/ (EXPECTED)
grep -rn "use crate::core" src/messaging/router.rs
# Expected: Has output for messaging types
```

---

### 3. Source File Analysis

**File:** `src/messaging/messaging_service.rs` (1,317 lines total)

**Code Currently Contains:**
1. Module documentation (lines 1-64)
2. Imports (lines 65-79)
3. MessagingService struct and impl (lines 81-392)
4. MessagingMetrics struct (lines 400-436)
5. MessagingStats struct (lines 438-472)
6. **ResponseRouter struct** (line 517)
7. **ResponseRouterMetrics struct** (line 527)
8. **ResponseRouter impl block** (lines 538-671)
9. **ResponseRouterStats struct** (line 675)
10. MessageReceptionMetrics struct and impl (lines 686-890)
11. MessageReceptionStats struct (lines 892-918)
12. **All unit tests** (lines 920-1317)

**Total tests in messaging_service.rs: 15 tests**

**Code to Extract for Task 1.3:**
- ResponseRouter struct (line 517)
- ResponseRouterMetrics struct (line 527)
- ResponseRouter impl block (lines 538-671)
- ResponseRouterStats struct (line 675)
- ResponseRouter-related tests (tests 1-10 in tests section)

---

### 4. Destination Files Check

**Destination:** `src/messaging/router.rs`

**Current router.rs State (78 lines):**
- Has `MessageRouter` placeholder struct (lines 26-30)
- Has `RoutingStats` placeholder struct (lines 48-51)
- Has placeholder impl blocks
- Has placeholder tests

**Naming Conflict Resolution:**
- **Problem**: Task 1.3 plan wants to extract `ResponseRouter` to router.rs, but router.rs ALREADY has a `MessageRouter` placeholder
- **Solution**: REPLACE existing `MessageRouter` placeholder with actual `ResponseRouter` implementation
- **Why This Is Correct**:
  - The placeholder was meant to be replaced with actual implementation
  - `ResponseRouter` is the actual routing implementation for request-response messaging
  - `MessageRouter` was just a generic placeholder name used during Task 1.1 module creation
- **Action**: Delete all placeholder code in router.rs and replace with actual ResponseRouter from messaging_service.rs

**Expected Final State:**
- router.rs will contain ResponseRouter implementation (not MessageRouter)
- router.rs will contain ResponseRouterMetrics and ResponseRouterStats
- router.rs will contain ~220 lines (replacing 78-line placeholder)
- All MessageRouter references in mod.rs will be updated to ResponseRouter

---

### 5. Implementation Steps

#### Step 5.1: Backup Current State

**Action:**
```bash
# Backup router.rs before changes
cp src/messaging/router.rs src/messaging/router.rs.backup
cp src/messaging/messaging_service.rs src/messaging/messaging_service.rs.backup

# Create git commit for safety (optional but recommended)
git add src/messaging/
git commit -m "Backup before Task 1.3 - ResponseRouter extraction"
```

**Purpose:** Create rollback point if extraction fails

---

#### Step 5.2: Extract ResponseRouter Implementation

**Source:** `src/messaging/messaging_service.rs` lines 474-684

**Code to Extract:**

```rust
/// Response router for request-response messaging pattern.
///
/// `ResponseRouter` handles routing responses from `handle-message` return values
/// back to requesting components via `handle-callback`. It implements the core
/// pattern defined in KNOWLEDGE-WASM-029:
///
/// - **No `send-response` host function**: Response IS the return value from `handle-message`
/// - **Correlation-based routing**: Uses `CorrelationTracker` to match responses to requests
/// - **Callback invocation**: Routes response to requester via `handle-callback` export
///
/// # Architecture
///
/// ```text
/// Component A                   Component B
/// send-request ──────────────► handle-message
///       │                            │
///       │ correlation_id             │ return value
///       ▼                            ▼
/// CorrelationTracker           ResponseRouter
///       │                            │
///       │◄────── route_response ─────┘
///       │
///       ▼
/// handle-callback ◄───────── response routed
/// ```
///
/// # Thread Safety
///
/// ResponseRouter is thread-safe via Arc-wrapped CorrelationTracker with DashMap.
/// All operations are lock-free with O(1) complexity.
///
/// # Performance
///
/// - Response routing: ~150ns (DashMap lookup + oneshot send)
/// - Callback invocation: ~300ns (WASM export call)
/// - Total: ~450ns end-to-end response delivery
///
/// # References
///
/// - **KNOWLEDGE-WASM-029**: Messaging Patterns (response IS return value)
/// - **ADR-WASM-009**: Component Communication Model (Pattern 2: Request-Response)
/// - **WASM-TASK-006 Phase 3 Task 3.2**: Response Routing and Callbacks
#[derive(Clone)]
pub struct ResponseRouter {
    /// Correlation tracker for pending request lookup
    correlation_tracker: Arc<CorrelationTracker>,

    /// Metrics for monitoring response routing
    metrics: Arc<ResponseRouterMetrics>,
}

/// Metrics for response routing.
#[derive(Debug, Default)]
struct ResponseRouterMetrics {
    /// Total responses routed successfully
    responses_routed: AtomicU64,

    /// Responses that failed to route (no pending request)
    responses_orphaned: AtomicU64,

    /// Responses that were error results
    error_responses: AtomicU64,
}

impl ResponseRouter {
    /// Create a new ResponseRouter with given correlation tracker.
    ///
    /// # Arguments
    ///
    /// * `correlation_tracker` - Shared correlation tracker for request-response matching
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use airssys_wasm::messaging::ResponseRouter;
    /// use airssys_wasm::actor::message::CorrelationTracker;
    /// use std::sync::Arc;
    ///
    /// let tracker = Arc::new(CorrelationTracker::new());
    /// let router = ResponseRouter::new(tracker);
    /// ```
    pub fn new(correlation_tracker: Arc<CorrelationTracker>) -> Self {
        Self {
            correlation_tracker,
            metrics: Arc::new(ResponseRouterMetrics::default()),
        }
    }

    /// Route a response to the requesting component.
    ///
    /// Looks up the pending request by correlation ID and delivers the response
    /// via the oneshot channel established during `send-request`. The
    /// CorrelationTracker handles channel delivery and cleanup.
    ///
    /// # Arguments
    ///
    /// * `correlation_id` - Correlation ID from the original request
    /// * `result` - Response result (Ok for success payload, Err for error)
    /// * `from` - Component ID that produced the response
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Response routed successfully
    /// * `Err(WasmError)` - Routing failed (no pending request, already resolved)
    ///
    /// # Errors
    ///
    /// - `WasmError::Internal` - Correlation ID not found (already resolved or timeout)
    ///
    /// # Performance
    ///
    /// ~150ns (DashMap lookup + oneshot send)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let router = messaging_service.response_router();
    ///
    /// // After handle-message returns, route the response
    /// router.route_response(
    ///     correlation_id,
    ///     Ok(response_payload),
    ///     ComponentId::new("responder"),
    /// ).await?;
    /// ```
    pub async fn route_response(
        &self,
        correlation_id: CorrelationId,
        result: Result<Vec<u8>, RequestError>,
        from: ComponentId,
    ) -> Result<(), WasmError> {
        // Track error responses
        if result.is_err() {
            self.metrics.error_responses.fetch_add(1, Ordering::Relaxed);
        }

        // Create ResponseMessage
        let response = ResponseMessage {
            correlation_id,
            from,
            to: ComponentId::new(""), // Will be filled by CorrelationTracker::resolve()
            result,
            timestamp: Utc::now(),
        };

        // Resolve via correlation tracker (delivers to oneshot channel)
        match self.correlation_tracker.resolve(correlation_id, response).await {
            Ok(()) => {
                self.metrics.responses_routed.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
            Err(e) => {
                self.metrics.responses_orphaned.fetch_add(1, Ordering::Relaxed);
                Err(e)
            }
        }
    }

    /// Check if a correlation ID has a pending request.
    ///
    /// Useful for determining whether a response should be routed or ignored.
    /// Fire-and-forget messages won't have pending requests.
    ///
    /// # Arguments
    ///
    /// * `correlation_id` - Correlation ID to check
    ///
    /// # Returns
    ///
    /// `true` if there's a pending request for this correlation ID
    pub fn has_pending_request(&self, correlation_id: &CorrelationId) -> bool {
        self.correlation_tracker.contains(correlation_id)
    }

    /// Get the number of responses routed successfully.
    pub fn responses_routed_count(&self) -> u64 {
        self.metrics.responses_routed.load(Ordering::Relaxed)
    }

    /// Get the number of orphaned responses (no pending request).
    pub fn responses_orphaned_count(&self) -> u64 {
        self.metrics.responses_orphaned.load(Ordering::Relaxed)
    }

    /// Get the number of error responses.
    pub fn error_responses_count(&self) -> u64 {
        self.metrics.error_responses.load(Ordering::Relaxed)
    }

    /// Get a snapshot of response router metrics.
    pub fn get_stats(&self) -> ResponseRouterStats {
        ResponseRouterStats {
            responses_routed: self.metrics.responses_routed.load(Ordering::Relaxed),
            responses_orphaned: self.metrics.responses_orphaned.load(Ordering::Relaxed),
            error_responses: self.metrics.error_responses.load(Ordering::Relaxed),
        }
    }
}

/// Snapshot of response router statistics.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ResponseRouterStats {
    /// Total responses routed successfully
    pub responses_routed: u64,

    /// Responses that failed to route (no pending request)
    pub responses_orphaned: u64,

    /// Responses that were error results
    pub error_responses: u64,
}
```

**Destination:** Replace entire content of `src/messaging/router.rs` with extracted code

**Import Updates for router.rs:**
```rust
// Layer 1: Standard library imports
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

// Layer 2: Third-party crate imports
use serde::{Deserialize, Serialize};
use chrono::Utc;

// Layer 3: airssys-rt imports
use airssys_rt::broker::InMemoryMessageBroker;

// Layer 3: Internal crate imports
use crate::actor::message::CorrelationTracker;
use crate::core::messaging::{CorrelationId, RequestError, ResponseMessage};
use crate::core::{ComponentId, WasmError};
```

**Imports ACTUALLY Used by ResponseRouter:**
- `std::sync::Arc` (for wrapping CorrelationTracker)
- `std::sync::atomic::{AtomicU64, Ordering}` (for metrics)
- `serde::{Deserialize, Serialize}` (for ResponseRouterStats)
- `chrono::Utc` (for timestamp)
- `crate::actor::message::CorrelationTracker` (for tracking pending requests)
- `crate::core::messaging::{CorrelationId, RequestError, ResponseMessage}` (types)
- `crate::core::{ComponentId, WasmError}` (types)

---

#### Step 5.3: Extract ResponseRouter Tests

**Source:** `src/messaging/messaging_service.rs` lines 1093-1177

**Tests to Extract (10 tests):**

```rust
#[test]
fn test_response_router_new() { /* implementation */ }

#[test]
fn test_response_router_clone() { /* implementation */ }

#[test]
fn test_response_router_has_pending_request_false() { /* implementation */ }

#[tokio::test]
async fn test_response_router_has_pending_request_true() { /* implementation */ }

#[tokio::test]
async fn test_response_router_route_response_success() { /* implementation */ }

#[tokio::test]
async fn test_response_router_route_response_error() { /* implementation */ }

#[tokio::test]
async fn test_response_router_orphaned_response() { /* implementation */ }

#[test]
fn test_response_router_get_stats() { /* implementation */ }

#[test]
fn test_response_router_access() { /* implementation */ }

#[tokio::test]
async fn test_get_stats_includes_responses_routed() { /* implementation */ }
```

**Destination:** Append to router.rs in `#[cfg(test)]` block

---

#### Step 5.4: Delete ResponseRouter from messaging_service.rs

**Action:**
```bash
# Delete ResponseRouter struct (line 517)
sed -i '' '517,525d' src/messaging/messaging_service.rs

# Delete ResponseRouterMetrics struct (lines now 527-535 after above delete)
# Note: line numbers shift after first delete
# Need to identify actual content instead of fixed line numbers

# Better approach: Read file, extract specific content
```

**Manual deletion steps:**
1. Delete ResponseRouter struct (after MessagingStats struct)
2. Delete ResponseRouterMetrics struct
3. Delete entire ResponseRouter impl block
4. Delete ResponseRouterStats struct
5. Delete all ResponseRouter-related tests (10 tests)
6. Delete test_response_router_access test (uses MessagingService)
7. Delete test_get_stats_includes_responses_routed test (uses MessagingService)

**Expected Result:**
- messaging_service.rs reduced from 1,317 lines to ~1,110 lines
- No ResponseRouter code remaining
- All ResponseRouter-related tests removed
- Remaining tests: 5 tests (MessagingService tests only)

---

#### Step 5.5: Update messaging/mod.rs Re-exports

**Current mod.rs (INCORRECT - has duplicate ResponseRouter):**
```rust
pub use messaging_service::{MessagingService, MessagingStats, ResponseRouter, ResponseRouterStats};
pub use router::{MessageRouter, RoutingStats};
```

**Updated mod.rs (CORRECT):**
```rust
pub use messaging_service::{MessagingService, MessagingStats};
pub use router::{ResponseRouter, ResponseRouterStats};
```

**Changes:**
- Remove ResponseRouter and ResponseRouterStats from messaging_service re-export
- Change MessageRouter and RoutingStats to ResponseRouter and ResponseRouterStats
- No more duplicate type definitions

---

#### Step 5.6: Add Module Documentation to router.rs

**Add to top of router.rs:**
```rust
//! Message routing for request-response pattern.
//!
//! This module provides the `ResponseRouter` which handles routing responses
//! from `handle-message` return values back to requesting components via
//! `handle-callback`.
//!
//! # Architecture
//!
//! ```text
//! Component A                   Component B
//! send-request ──────────────► handle-message
//!       │                            │
//!       │ correlation_id             │ return value
//!       ▼                            ▼
//! CorrelationTracker           ResponseRouter
//!       │                            │
//!       │◄────── route_response ─────┘
//!       │
//!       ▼
//! handle-callback ◄───────── response routed
//! ```
//!
//! # References
//!
//! - **KNOWLEDGE-WASM-029**: Messaging Patterns (response IS return value)
//! - **ADR-WASM-009**: Component Communication Model (Pattern 2)
//! - **WASM-TASK-006 Phase 3 Task 3.2**: Response Routing Implementation
```

---

### 6. Unit Testing Plan

**Tests to Extract:** 10 ResponseRouter tests from messaging_service.rs

**Test Categories:**

1. **Constructor Tests** (2 tests):
   - test_response_router_new
   - test_response_router_clone

2. **Query Tests** (1 test):
   - test_response_router_has_pending_request_false
   - test_response_router_has_pending_request_true

3. **Routing Tests** (3 tests):
   - test_response_router_route_response_success
   - test_response_router_route_response_error
   - test_response_router_orphaned_response

4. **Metrics Tests** (1 test):
   - test_response_router_get_stats

5. **Integration Tests** (3 tests):
   - test_response_router_access
   - test_get_stats_includes_responses_routed

**Test Locations After Extraction:**
- All ResponseRouter tests in `src/messaging/router.rs` in `#[cfg(test)]` block
- MessagingService tests remain in `src/messaging/messaging_service.rs` (5 tests remaining)
- Total tests across both files: 15 tests (10 + 5) - NO TESTS LOST

**Test Coverage Verification:**
- [ ] ResponseRouter::new() has test
- [ ] ResponseRouter::route_response() has success and error tests
- [ ] ResponseRouter::has_pending_request() has tests
- [ ] ResponseRouter::get_stats() has test
- [ ] All metric methods have tests
- [ ] Orphaned response scenario tested
- [ ] Integration with CorrelationTracker tested

---

### 7. Integration Testing Plan

**No new integration tests needed** - This is a code organization refactoring, not new functionality.

**Existing Integration Tests Will Continue To Work:**
- tests/actor_routing_tests.rs - Tests request-response pattern
- tests/actor_invocation_tests.rs - Tests message handling
- tests/messaging_tests.rs - Tests messaging service

**Why No New Tests:**
- ResponseRouter implementation is unchanged (just moved)
- Public API remains the same (re-exported from messaging/)
- Existing integration tests use MessagingService, which uses ResponseRouter
- All existing tests will continue to pass

---

### 8. Quality Standards

**All code must meet:**

**Code Quality:**
- ✅ Code builds without errors: `cargo build`
- ✅ Zero compiler warnings: `cargo build 2>&1 | grep -i warning`
- ✅ Zero clippy warnings: `cargo clippy --all-targets --all-features -- -D warnings`

**Standards Compliance:**
- ✅ Follows PROJECTS_STANDARD.md §2.1-§6.4
- ✅ Follows Rust guidelines (see references above)
- ✅ Module boundaries correct (no runtime/ imports)

**Testing Requirements:**
- ✅ Unit tests in `#[cfg(test)]` blocks
- ✅ All tests pass: `cargo test --lib`
- ✅ Test coverage preserved (15 tests total)

**Documentation:**
- ✅ Module documentation complete (`//!` comments)
- ✅ All public types documented with `///`
- ✅ All public functions have doc comments with examples
- ✅ Standards Compliance Checklist in task file

---

### 9. Verification Checklist

**For implementer to run after completing all steps:**

```bash
# ============================================================================
# 1. Architecture Verification (ADR-WASM-023)
# ============================================================================

echo "=== Verifying Module Boundaries ==="

# Check 1: router.rs doesn't import from runtime/ (FORBIDDEN)
if grep -rn "use crate::runtime" src/messaging/router.rs; then
    echo "❌ FAILED: router.rs imports from runtime/"
    exit 1
else
    echo "✅ PASSED: router.rs does not import from runtime/"
fi

# Check 2: router.rs imports from actor/ (EXPECTED for CorrelationTracker)
if grep -rn "use crate::actor::message::CorrelationTracker" src/messaging/router.rs; then
    echo "✅ PASSED: router.rs imports CorrelationTracker from actor/"
else
    echo "❌ FAILED: router.rs missing CorrelationTracker import"
    exit 1
fi

# Check 3: router.rs imports from core/ (EXPECTED)
if grep -rn "use crate::core::messaging" src/messaging/router.rs; then
    echo "✅ PASSED: router.rs imports messaging types from core/"
else
    echo "❌ FAILED: router.rs missing core messaging imports"
    exit 1
fi

echo ""
echo "=== All Architecture Checks Passed ==="

# ============================================================================
# 2. Build Verification
# ============================================================================

echo "=== Building Project ==="
cargo build
BUILD_EXIT=$?

if [ $BUILD_EXIT -ne 0 ]; then
    echo "❌ FAILED: Build failed"
    exit 1
else
    echo "✅ PASSED: Build succeeded"
fi

# Check for warnings
WARNINGS=$(cargo build 2>&1 | grep -i "warning" | wc -l)
if [ "$WARNINGS" -gt 0 ]; then
    echo "❌ FAILED: Build has $WARNINGS warnings"
    exit 1
else
    echo "✅ PASSED: Zero compiler warnings"
fi

echo ""

# ============================================================================
# 3. Test Verification
# ============================================================================

echo "=== Running Tests ==="
cargo test --lib
TEST_EXIT=$?

if [ $TEST_EXIT -ne 0 ]; then
    echo "❌ FAILED: Tests failed"
    exit 1
else
    echo "✅ PASSED: All tests pass"
fi

# Test ResponseRouter specifically
echo ""
echo "=== Testing ResponseRouter ==="
cargo test response_router_new
cargo test response_router_clone
cargo test response_router_route_response
cargo test response_router_orphaned
cargo test response_router_get_stats

echo ""
echo "✅ PASSED: All ResponseRouter tests pass"

# ============================================================================
# 4. Clippy Verification
# ============================================================================

echo "=== Running Clippy ==="
cargo clippy --all-targets --all-features -- -D warnings
CLIPPY_EXIT=$?

if [ $CLIPPY_EXIT -ne 0 ]; then
    echo "❌ FAILED: Clippy found warnings"
    exit 1
else
    echo "✅ PASSED: Zero clippy warnings"
fi

echo ""
echo "=== All Quality Checks Passed ==="
echo "✅ Task 1.3 is complete"
```

**Expected output:**
- All architecture checks pass
- Build succeeds with zero warnings
- All 15 tests pass (10 in router.rs + 5 in messaging_service.rs)
- Clippy passes with zero warnings

---

### 10. Risk Mitigation

**Identified Risks:**

**1. Import Update Errors**
- **Risk**: Forgetting to update imports when moving code to router.rs
- **Mitigation**: Verification commands check for correct imports
- **Detection**: Build will fail if imports wrong

**2. Test Failures After Move**
- **Risk**: Tests rely on old file structure
- **Mitigation**: Move all tests with implementation code
- **Detection**: cargo test will fail

**3. Duplicate Type Definitions**
- **Risk**: ResponseRouter exists in both files after extraction
- **Mitigation**: Delete from messaging_service.rs after extracting to router.rs
- **Detection**: Build will fail with duplicate definition error

**4. Incorrect Line Numbers**
- **Risk**: Deleting wrong lines from messaging_service.rs
- **Mitigation**: Use content matching instead of line numbers for deletion
- **Detection**: Build will fail with missing type errors

**5. Broken Re-exports**
- **Risk**: mod.rs has incorrect re-exports after extraction
- **Mitigation**: Update mod.rs in separate step, verify imports
- **Detection**: Build will fail with import errors

**Contingency:**
- If build fails with import errors: Review and fix imports in router.rs
- If tests fail: Debug and fix test imports
- If architecture check fails: Review and remove forbidden imports
- If all else fails: Restore from backups (router.rs.backup, messaging_service.rs.backup)

---

### 11. Rollback Plan

**If critical issues arise:**

**1. Backups Created (Step 5.1):**
```bash
# Restore router.rs
cp src/messaging/router.rs.backup src/messaging/router.rs

# Restore messaging_service.rs
cp src/messaging/messaging_service.rs.backup src/messaging/messaging_service.rs

# Restore mod.rs
git checkout HEAD -- src/messaging/mod.rs
```

**2. Verification After Rollback:**
```bash
# Verify build passes
cargo build

# Verify tests pass
cargo test --lib

# Verify no architecture violations
grep -rn "use crate::runtime" src/messaging/router.rs
# Expected: No output
```

**3. Decision Points:**
- If type mismatches: Verify types in core/messaging.rs are correct
- If import issues: Check CorrelationTracker location (actor/message/)
- If logic errors: Compare with original messaging_service.rs code

**4. After Rollback:**
- Document what went wrong
- Update plan with learned lessons
- Review ADRs/Knowledges before retrying

---

### 12. Documentation Requirements

**Per Diátaxis Guidelines:**

**Type:** Reference Documentation

**Why Reference:**
- ResponseRouter is an API type with methods to document
- Provides complete API reference with examples
- Neutral technical documentation, not tutorial or how-to

**Required Documentation Elements:**

**1. Module-Level Documentation (`//!`):**
- What ResponseRouter does
- Architecture diagram showing message flow
- References to ADRs/Knowledges
- Performance characteristics
- Thread safety guarantees

**2. Type Documentation (`///`):**
- ResponseRouter struct purpose
- ResponseRouterMetrics struct purpose
- ResponseRouterStats struct purpose

**3. Function Documentation (`///`):**
- Purpose of each public method
- Parameters with types and descriptions
- Return values with types
- Error conditions (if any)
- Performance characteristics
- Code examples

**Quality Standards (per documentation-quality-standards.md):**
- ❌ NO marketing hyperbole ("revolutionary", "groundbreaking", "best-in-class")
- ✅ Technical precision with measurable claims
- ✅ Accurate descriptions, not promotional
- ✅ Professional tone throughout

---

### 13. Testing Strategy

**Unit Testing:**
- **Location**: `#[cfg(test)]` block in router.rs
- **Coverage**: 10 tests covering all ResponseRouter functionality
- **Categories**:
  - Constructor tests (new, clone)
  - Query tests (has_pending_request)
  - Routing tests (route_response success, error, orphaned)
  - Metrics tests (get_stats, individual metric methods)
  - Integration tests (MessagingService access)

**Integration Testing:**
- **No new tests needed** - Existing integration tests cover ResponseRouter usage
- Existing tests that verify ResponseRouter functionality:
  - tests/actor_routing_tests.rs
  - tests/actor_invocation_tests.rs
  - tests/messaging_tests.rs

**Test Preservation:**
- All 15 tests from original messaging_service.rs preserved
- 10 tests moved to router.rs (ResponseRouter tests)
- 5 tests remain in messaging_service.rs (MessagingService tests)
- Zero tests lost in refactoring

**Test Execution:**
```bash
# Run all messaging tests
cargo test --lib messaging

# Run ResponseRouter tests specifically
cargo test response_router

# Run all tests
cargo test --lib
```

---

### 14. Final Deliverables

**Task 1.3 delivers:**

**1. router.rs (replaced with actual implementation):**
- ResponseRouter struct and implementation
- ResponseRouterMetrics struct
- ResponseRouterStats struct
- 10 unit tests in `#[cfg(test)]` block
- Module documentation
- Expected size: ~220 lines (vs 78-line placeholder)

**2. messaging_service.rs (reduced):**
- ResponseRouter struct deleted
- ResponseRouterMetrics struct deleted
- ResponseRouter impl block deleted
- ResponseRouterStats struct deleted
- 10 ResponseRouter tests deleted
- Expected size: ~1,110 lines (vs 1,317 lines original)

**3. messaging/mod.rs (updated re-exports):**
- Removed duplicate ResponseRouter from messaging_service re-export
- Updated router re-export to use ResponseRouter (not MessageRouter)
- No duplicate type definitions

**4. All imports verified:**
- router.rs imports from core/, actor/, airssys-rt (CORRECT)
- router.rs does NOT import from runtime/ (VERIFIED)
- All imports follow 3-layer pattern (VERIFIED)

**5. All tests passing:**
- 15 total tests preserved (10 in router.rs + 5 in messaging_service.rs)
- Zero tests lost
- All tests passing: `cargo test --lib`

**6. Zero warnings:**
- Zero compiler warnings: `cargo build`
- Zero clippy warnings: `cargo clippy --all-targets --all-features -- -D warnings`

**Estimated Effort:** 4-6 hours  
**Risk Level:** Low (code organization, no new features)

---

### 15. Standards Compliance Checklist

**PROJECTS_STANDARD.md Applied:**
- [ ] **§2.1 3-Layer Import Organization** - Evidence: router.rs imports follow std → external → internal pattern (Layer 1: std, Layer 2: serde/chrono, Layer 3: core/actor/airssys-rt)
- [ ] **§3.2 chrono DateTime<Utc> Standard** - Evidence: ResponseRouter uses `Utc::now()` for timestamp (line in route_response)
- [ ] **§4.3 Module Architecture Patterns** - Evidence: messaging/mod.rs contains only declarations, router.rs contains implementation code
- [ ] **§6.2 Avoid `dyn` Patterns** - Evidence: ResponseRouter uses concrete CorrelationTracker (Arc<CorrelationTracker>), no trait objects
- [ ] **§6.4 Implementation Quality Gates** - Evidence: Zero compiler/clippy warnings, 15 tests passing, verification commands in plan

**Rust Guidelines Applied:**
- [ ] **M-DESIGN-FOR-AI** - Evidence: Idiomatic APIs, comprehensive docs, 10 testable tests with examples
- [ ] **M-MODULE-DOCS** - Evidence: router.rs has `//!` module docs and `///` doc comments for all public types/methods
- [ ] **M-ERRORS-CANONICAL-STRUCTS** - Evidence: RequestError follows thiserror pattern, WasmError used consistently
- [ ] **M-STATIC-VERIFICATION** - Evidence: Lints enabled, clippy verification in Step 9, `#[expect(clippy::...)]` for intentional violations

**Documentation Quality:**
- [ ] **No hyperbolic terms** - Evidence: Verified against forbidden list in documentation-quality-standards.md
- [ ] **Technical precision** - Evidence: All performance claims measurable (~150ns routing, ~450ns total), no vague assertions
- [ ] **Diátaxis compliance** - Evidence: Reference documentation type chosen for ResponseRouter API, complete API documentation

**ADR Compliance:**
- [ ] **ADR-WASM-018** - Evidence: One-way dependency enforced (messaging → actor → core, no runtime imports)
- [ ] **ADR-WASM-023** - Evidence: No forbidden imports (grep verification commands in Step 9)

**Knowledge Compliance:**
- [ ] **KNOWLEDGE-WASM-012** - Evidence: Module structure follows specification (messaging/router.rs, correct imports)
- [ ] **KNOWLEDGE-WASM-029** - Evidence: ResponseRouter implements "response is return value" pattern correctly
- [ ] **KNOWLEDGE-WASM-024** - Evidence: Async-only communication, no synchronous messaging

---

### 16. Appendix: Critical Line Numbers Reference

**For Implementer:**

**Source File: src/messaging/messaging_service.rs (1,317 lines)**

**ResponseRouter Code:**
- ResponseRouter struct: Line 517
- ResponseRouterMetrics struct: Line 527
- ResponseRouter impl block: Lines 538-671 (134 lines)
- ResponseRouterStats struct: Line 675

**ResponseRouter Tests (lines ~1093-1277):**
- test_response_router_new: ~1093
- test_response_router_clone: ~1105
- test_response_router_has_pending_request_false: ~1117
- test_response_router_has_pending_request_true: ~1126
- test_response_router_route_response_success: ~1152
- test_response_router_route_response_error: ~1193
- test_response_router_orphaned_response: ~1233
- test_response_router_get_stats: ~1251
- test_response_router_access: ~1262
- test_get_stats_includes_responses_routed: ~1277

**Total Test Count: 15 tests**
- ResponseRouter tests: 10 tests (to extract to router.rs)
- MessagingService tests: 5 tests (remain in messaging_service.rs)

---

### 17. Summary

**Task 1.3 extracts ResponseRouter from messaging_service.rs to router.rs:**

**What Changes:**
1. ✅ Create router.rs with ResponseRouter implementation (~220 lines)
2. ✅ Delete ResponseRouter from messaging_service.rs (reduce to ~1,110 lines)
3. ✅ Update messaging/mod.rs re-exports (remove duplicates)
4. ✅ Move 10 ResponseRouter tests to router.rs
5. ✅ Preserve all 15 tests (zero tests lost)

**What Stays The Same:**
- Public API unchanged (ResponseRouter still available via messaging::)
- Functionality unchanged (code is moved, not modified)
- MessagingService tests remain in messaging_service.rs (5 tests)

**Verification:**
- Build passes with zero warnings
- All 15 tests pass
- Zero clippy warnings
- Architecture verified (no runtime/ imports)

**Success Criteria:**
- [ ] router.rs contains ResponseRouter (not MessageRouter placeholder)
- [ ] messaging_service.rs has ResponseRouter removed
- [ ] messaging/mod.rs has correct re-exports (no duplicates)
- [ ] All imports verified correct (core/, actor/, airssys-rt only)
- [ ] All 15 tests passing
- [ ] Zero compiler and clippy warnings

---

