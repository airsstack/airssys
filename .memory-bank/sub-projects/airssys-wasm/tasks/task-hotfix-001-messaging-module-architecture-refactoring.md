# [WASM-TASK-HOTFIX-001] - Messaging Module Architecture Refactoring

**Task ID:** WASM-TASK-HOTFIX-001  
**Created:** 2025-12-26  
**Updated:** 2025-12-26  
**Priority:** 🔴 CRITICAL / BLOCKING  
**Status:** NOT STARTED  
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

### Phase 1: Create Top-Level messaging/ Module (Days 1-2)

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

#### Task 1.3: Create Remaining Messaging Submodules

**Objective:** Create messaging submodules per KNOWLEDGE-WASM-012 specification.

**Deliverables:**

**Files to Create:**
- `src/messaging/router.rs` - MessageBroker routing
- `src/messaging/fire_and_forget.rs` - Fire-and-forget pattern
- `src/messaging/request_response.rs` - Request-response pattern
- `src/messaging/codec.rs` - Multicodec encoding
- `src/messaging/topics.rs` - Topic-based pub/sub (stub for Phase 2)

**Success Criteria:**
- ✅ All messaging submodules created
- ✅ `src/messaging/mod.rs` declarations match created files
- ✅ `cargo build` succeeds
- ✅ Module structure follows KNOWLEDGE-WASM-012

**Estimated Effort:** 4-6 hours  
**Risk Level:** Low (new modules with clear scope)

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

### Phase 1: Create Top-Level messaging/ Module
- ✅ Task 1.1: Create messaging module structure - COMPLETE
- ✅ Task 1.2: Move messaging code from runtime/messaging.rs - COMPLETE
- ⏸️ Task 1.3: Create remaining messaging submodules - NOT STARTED
**Status:** 67% COMPLETE (2/3 tasks)

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

**Overall Status:** 33% COMPLETE (2/6 phases complete, 7/22 tasks complete)

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
