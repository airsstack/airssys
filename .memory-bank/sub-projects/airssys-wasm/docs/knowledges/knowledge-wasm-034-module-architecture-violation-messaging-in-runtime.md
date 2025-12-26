# Knowledge Document: Module Architecture Violation - Messaging in Runtime

**Document ID:** KNOWLEDGE-WASM-034  
**Created:** 2025-12-26  
**Updated:** 2025-12-26  
**Category:** Architecture & Remediation  
**Complexity:** Medium  
**Maturity:** Stable  

## Overview

Documents a critical architectural violation where messaging infrastructure was incorrectly placed in the `runtime/` module instead of being a top-level `messaging/` module. This violation creates confusion about module boundaries, potential circular dependencies, and violates the three-layer architecture defined in ADR-WASM-018.

## Context

### Problem Statement

During architectural audit of `airssys-wasm`, discovered that `src/runtime/messaging.rs` contains messaging infrastructure including:

- `MessagingService` - Manages MessageBroker singleton
- `ResponseRouter` - Routes request-response messages
- `MessageReceptionMetrics` - Tracks message delivery statistics

This code **does not belong in `runtime/`** for several reasons:

1. **Wrong Responsibility**: `runtime/` should only contain WASM execution engine (Block 1)
2. **Module Boundary Violation**: Messaging is a separate responsibility (Block 5)
3. **Potential Circular Dependency**: `runtime/messaging.rs` imports from `actor/message/`
4. **Violates KNOWLEDGE-WASM-012**: Module structure specifies `messaging/` as top-level module

### Scope

This knowledge applies to:
- `airssys-wasm` module architecture refactoring
- Block 5 (Inter-Component Communication) implementation
- All future messaging infrastructure development

### Prerequisites

- **KNOWLEDGE-WASM-012**: Module Structure Architecture
- **KNOWLEDGE-WASM-003**: Core Architecture Design  
- **ADR-WASM-018**: Three-Layer Architecture
- **ADR-WASM-023**: Module Boundary Enforcement

## Technical Content

### Core Concepts

#### Three-Layer Architecture

The airssys-wasm module architecture defines three clear layers (ADR-WASM-018):

```
core/ (foundation, no internal deps)
  ↓
runtime/ (WASM execution, depends on core only)
  ↓  
actor/ (Actor integration with airssys-rt, depends on runtime)
  ↓
messaging/ (Inter-component communication, depends on actor)
```

**Key Rule**: Dependencies flow ONE WAY (top to bottom). Higher layers CANNOT import from lower layers.

#### Module Responsibilities

From KNOWLEDGE-WASM-012, each top-level module has clear responsibilities:

| Module | Block | Responsibility | Current Status |
|--------|--------|---------------|----------------|
| `runtime/` | Block 1 | WASM execution engine, component loading, resource limiting | ✅ Correct, except messaging.rs |
| `actor/` | Block 3 | ComponentActor, supervision, actor-level message integration | ✅ Correct |
| `messaging/` | Block 5 | Inter-component communication infrastructure | ❌ **MISSING** - code in runtime/ instead |

#### Correct Architecture

Following KNOWLEDGE-WASM-012 (lines 506-596), the `messaging/` module should be:

```
src/messaging/                  # Block 5: Inter-Component Communication
  ├── mod.rs                 # Module declarations only
  ├── messaging_service.rs     # MessagingService (moved from runtime/)
  ├── router.rs              # MessageBroker routing integration
  ├── fire_and_forget.rs     # Fire-and-forget pattern
  ├── request_response.rs    # Request-response pattern
  ├── codec.rs              # Multicodec message encoding
  └── topics.rs             # Topic-based pub-sub (Phase 2)
```

### Implementation Details

#### Current Violation

**Location**: `src/runtime/messaging.rs` (1,313 lines)

**Contains**:
- MessagingService (lines 126-387)
- MessagingMetrics (lines 418-431)
- MessagingStats (lines 448-467)
- ResponseRouter (lines 511-666)
- MessageReceptionMetrics (lines 736-852)
- ResponseRouterStats (lines 668-679)

**Violates**:
- KNOWLEDGE-WASM-012 (lines 506-596): Specifies messaging/ as top-level module
- ADR-WASM-018: Three-layer architecture (runtime/ should not depend on actor/)
- ADR-WASM-023: Module boundary enforcement

**Why It's Wrong**:

1. **runtime/ is for WASM execution only** (ADR-WASM-002)
   - WasmEngine, ComponentLoader, ResourceLimits
   - NOT for inter-component communication infrastructure

2. **Creates circular dependency risk**:
   ```
   runtime/messaging.rs:76 → use crate::actor::message::{CorrelationId, CorrelationTracker}
   ```
   - runtime/ should not know about actor/ internal types
   - Creates reverse dependency (lower → higher layer)

3. **Violates separation of concerns**:
   - Messaging infrastructure is a distinct responsibility
   - Should be in own top-level module (Block 5)
   - Makes navigation confusing (messaging code scattered)

#### Correct Implementation

**Move to**: `src/messaging/messaging_service.rs`

```rust
//! Messaging infrastructure for inter-component communication.
//!
//! This module provides MessagingService which manages MessageBroker
//! singleton and coordinates runtime-level message routing.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                  MessagingService                        │
//! │  ┌────────────────────────────────────────────────┐     │
//! │  │  • Initialize MessageBroker singleton          │     │
//! │  │  • Provide broker access to ActorSystem        │     │
//! │  │  • Track messaging metrics                     │     │
//! │  └────────────────────────────────────────────────┘     │
//! └─────────────────────────────────────────────────────────┘
//!                         ↓ provides access to
//! ┌─────────────────────────────────────────────────────────┐
//! │         airssys-rt InMemoryMessageBroker                │
//! └─────────────────────────────────────────────────────────┘
//! ```

use crate::actor::message::{CorrelationTracker, CorrelationId, RequestError};
use crate::core::messaging::{ResponseMessage};
use airssys_rt::broker::InMemoryMessageBroker;

/// Service managing MessageBroker integration for inter-component communication.
pub struct MessagingService {
    broker: Arc<InMemoryMessageBroker<ComponentMessage>>,
    correlation_tracker: Arc<CorrelationTracker>,
    metrics: Arc<MessagingMetrics>,
    response_router: Arc<ResponseRouter>,
}
```

**Import Path Changes**:

```rust
// BEFORE (WRONG):
use airssys_wasm::runtime::MessagingService;

// AFTER (CORRECT):
use airssys_wasm::messaging::MessagingService;
```

### Code Examples

#### Correct Import Pattern

```rust
// src/messaging/mod.rs (new file)
pub mod messaging_service;
pub mod router;
pub mod fire_and_forget;
pub mod request_response;
pub mod codec;
pub mod topics;  // Phase 2

pub use messaging_service::{MessagingService, MessagingStats};
pub use router::MessageRouter;
pub use fire_and_forget::FireAndForget;
pub use request_response::{RequestResponse, RequestError};
```

#### lib.rs Module Declaration

```rust
// src/lib.rs - Add messaging module
pub mod messaging;  // NEW: Block 5 module

// Keep actor/message for actor-specific integration
pub mod actor;
```

### Configuration

**No runtime configuration required** - this is a module structure refactoring, not runtime behavior change.

## Usage Patterns

### Common Use Cases

1. **Messaging Infrastructure Development**
   - Add new messaging features to `src/messaging/`
   - Import via `use airssys_wasm::messaging::*`

2. **Runtime Integration**
   - `runtime/` depends on `messaging/` (not vice versa)
   - Runtime uses MessagingService for message routing

3. **Actor Integration**
   - `actor/` depends on `messaging/`
   - ComponentActor uses messaging infrastructure

### Best Practices

1. **Follow Module Responsibility**: Keep WASM execution in `runtime/`, messaging in `messaging/`
2. **Maintain One-Way Dependencies**: Higher modules import lower, never reverse
3. **Clear Boundaries**: Each module has distinct, well-defined responsibility
4. **Architecture Verification**: Run CI checks to prevent future violations

### Antipatterns

❌ **Wrong**: Place messaging code in `runtime/`
```rust
// runtime/messaging.rs - DON'T DO THIS
pub struct MessagingService { ... }
```

✅ **Correct**: Place messaging code in top-level `messaging/`
```rust
// messaging/messaging_service.rs - DO THIS
pub struct MessagingService { ... }
```

❌ **Wrong**: runtime/ imports from actor/
```rust
// runtime/messaging.rs - DON'T DO THIS
use crate::actor::message::{CorrelationTracker, CorrelationId};
```

✅ **Correct**: actor/ and runtime/ both import from `messaging/`
```rust
// actor/message/ and runtime/ both DO THIS
use crate::messaging::{...}
```

## Performance Considerations

### Performance Characteristics

**No Performance Impact** - This is a code organization refactoring only:
- No behavioral changes
- Same code, different location
- Identical runtime performance
- Zero functional difference

### Optimization Opportunities

**Code Organization Benefits**:
- Clearer navigation for developers
- Easier to understand module boundaries
- Reduces cognitive load when finding messaging code
- Prevents future architectural violations

## Integration Points

### Dependencies

- **actor/**: Uses messaging infrastructure
- **runtime/**: Uses messaging infrastructure  
- **core/messaging**: Shared types (ResponseMessage, RequestError)

### Compatibility

- **Breaking Change**: Import paths change from `runtime::` to `messaging::`
- **Migration Required**: Update all imports referencing `runtime::MessagingService`
- **Backward Compatibility**: Provide re-exports in `runtime/` temporarily (deprecation cycle)

### Migration Paths

**Phase 1**: Create `messaging/` module
**Phase 2**: Move code from `runtime/messaging.rs`
**Phase 3**: Update all imports
**Phase 4**: Remove `runtime/messaging.rs`
**Phase 5**: Verify no circular dependencies

## Security Considerations

### Security Implications

**None** - This is a module organization change with no security impact.

### Threat Model

**No new threats** - Security behavior unchanged.

### Compliance

**Maintains Compliance**:
- ✅ Enforces ADR-WASM-018 three-layer architecture
- ✅ Follows ADR-WASM-023 module boundary enforcement
- ✅ Aligns with KNOWLEDGE-WASM-012 module structure

## Maintenance

### Review Schedule

**Review After**: WASM-TASK-HOTFIX-001 completion

### Update Triggers

- When new messaging infrastructure is added
- If circular dependency violations detected
- During architecture audits

### Owner/Maintainer

**Module Architecture Owner**: All developers working on airssys-wasm

## References

### Related Documentation

**ADRs:**
- **ADR-WASM-018**: Three-Layer Architecture
- **ADR-WASM-023**: Module Boundary Enforcement

**Knowledge:**
- **KNOWLEDGE-WASM-002**: High-Level Overview
- **KNOWLEDGE-WASM-003**: Core Architecture Design
- **KNOWLEDGE-WASM-012**: Module Structure Architecture (PRIMARY REFERENCE)
- **KNOWLEDGE-WASM-024**: Component Messaging Clarifications
- **KNOWLEDGE-WASM-029**: Messaging Patterns

**Technical Debt:**
- **DEBT-WASM-004**: Message Delivery Runtime Glue Missing
- **DEBT-WASM-027**: Duplicate WASM Runtime Fatal Architecture Violation
- **DEBT-WASM-028**: Circular Dependency Actor Runtime

**Task:**
- **WASM-TASK-HOTFIX-001**: Messaging Module Architecture Refactoring

### Workspace Standards

- **§4.3**: Module Architecture (mod.rs declaration-only pattern)
- **§6.2**: Avoid `dyn` patterns (not applicable here)
- **§6.3**: Microsoft Rust Guidelines (code organization)

## History

### Version History
- **[2025-12-26]:** 1.0 - Initial creation documenting runtime/messaging.rs violation

### Review History
- **[2025-12-26]:** Documented based on architectural audit

---

**Template Version:** 1.0  
**Last Updated:** 2025-12-26
