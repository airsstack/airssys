# KNOWLEDGE-WASM-038: Component Module Responsibility and Architecture

**Document ID:** KNOWLEDGE-WASM-038  
**Created:** 2026-01-08  
**Category:** Architecture / Module Design / Integration  
**Maturity:** Stable  
**Severity:** 🟡 **IMPORTANT - Implementation Reference**

## Overview

This document clarifies the responsibility and architectural distinction between `core/component/` (Layer 1 foundation types) and `component/` (Layer 3A actor integration module). It provides the definitive reference for understanding what belongs in each module and how they work together in the clean-slate rebuild architecture.

## Context

### Problem Statement

Developers often confuse `core/component/` and `component/` modules, leading to:
1. **Wrong code placement:** Business logic in `core/` or types in `component/`
2. **Dependency violations:** `component/` importing from `runtime/` or `security/`
3. **Misunderstood boundaries:** Unclear what each module owns

### Scope

This knowledge applies to:
- **WASM-TASK-017** and all Phase 3 core module tasks (core/component/, core/runtime/, etc.)
- **Phase 4** component/ integration module tasks (wrapper, registry, spawner, supervisor)
- All future development touching component-related code

### Prerequisites

- **KNOWLEDGE-WASM-037:** Rebuild Architecture Clean-Slate Design (six-module architecture)
- **KNOWLEDGE-WASM-031:** Foundational Architecture (Actor model, component lifecycle)
- **ADR-WASM-025:** Clean-Slate Rebuild Architecture (layer organization)
- **ADR-WASM-023:** Module Boundary Enforcement (dependency rules)

## Technical Content

### Core Concepts: Two-Layer Distinction

The component-related functionality is split across **TWO SEPARATE MODULES** at different architectural layers:

```
┌─────────────────────────────────────────────────────────────┐
│                    airssys-wasm                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  core/component/ (LAYER 1)  component/ (LAYER 3A)           │
│                                                             │
│  Purpose: Data Structures    Purpose: Actor Integration      │
│  Types & Abstractions       & Business Logic                │
│                                                             │
│  Ownership:                  Ownership:                      │
│  - ComponentId              - ComponentWrapper              │
│  - ComponentHandle          - ComponentRegistry             │
│  - ComponentMessage         - ComponentSpawner              │
│  - ComponentLifecycle       - Supervisor                    │
│                                                             │
│  Dependencies:               Dependencies:                   │
│  - std ONLY                 - core/component/ (types)       │
│                             - airssys-rt (Actor, Child)    │
│                                                             │
│  Implementation:              Implementation:                │
│  - Data structs              - Actor wrapping logic         │
│  - Traits (no impl)         - Lifecycle management         │
│  - NO business logic         - Registry operations          │
│                             - Message routing              │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Core Module: `core/component/` (Layer 1)

**Purpose:** Foundation types, data structures, and trait abstractions that ALL other modules can depend on.

**Key Types:**

| Type | Purpose | Implementation |
|------|---------|----------------|
| `ComponentId` | Unique identifier for component instances | Data struct with namespace/name/instance fields |
| `ComponentHandle` | Opaque handle to loaded WASM components | Data struct wrapping ComponentId + handle_id |
| `ComponentMessage` | Message envelope for component communication | Data struct with sender, payload, metadata |
| `MessageMetadata` | Metadata for messages (correlation, timestamp) | Data struct with optional correlation_id, reply_to |
| `ComponentLifecycle` | Trait for component lifecycle management | Trait definition (no implementations) |

**Architectural Rules:**

1. **Imports NOTHING except `std`** - No internal crate imports
2. **NO business logic** - Only data structures and trait definitions
3. **NO external dependencies** - No airssys-rt, no airssys-osl
4. **Trait definitions only** - Implementations live in concrete modules

**Dependency Flow:**
```
All modules ───────────────────► core/component/
(runtime/, security/, component/, messaging/, system/)
    ↓ all depend on these foundation types
```

**File Structure:**
```
core/component/
├── mod.rs           # Module declarations
├── id.rs            # ComponentId struct
├── handle.rs        # ComponentHandle struct
├── message.rs       # ComponentMessage, MessageMetadata
└── traits.rs        # ComponentLifecycle trait
```

### Integration Module: `component/` (Layer 3A)

**Purpose:** Wraps WASM components as Actors managed by airssys-rt, handles lifecycle, registry, and supervision.

**Key Components:**

| Component | Purpose | Dependencies |
|-----------|---------|--------------|
| `ComponentWrapper` | Wraps Actor + Child with WASM runtime | `core/component/*`, `airssys_rt::{Actor, Child}` |
| `ComponentRegistry` | Maps ComponentId → ActorAddress (O(1) lookup) | `core/component::ComponentId`, `airssys_rt::ActorAddress` |
| `ComponentSpawner` | Spawns ComponentActor instances | `core/component/*`, `airssys_rt::{ActorSystem, MessageBroker}` |
| `SupervisorConfig` | Supervision strategy for crash/restart | `core/component/*`, `airssys_rt::SupervisorNode` |

**Architectural Rules:**

1. **Depends ONLY on `core/component/`** (for types)
2. **Depends on `airssys-rt`** (for Actor, Child, ActorSystem)
3. **NO direct imports from `runtime/`** (RuntimeEngine injected by `system/`)
4. **NO direct imports from `security/`** (SecurityContext injected by `system/`)

**Dependency Flow:**
```
component/ ───────────────────► core/component/ (types)
component/ ───────────────────► airssys-rt (Actor, Child, ActorSystem)
system/ ─────────────────────► component/ (coordinates)
system/ ─────────────────────► runtime/ (injects RuntimeEngine)
system/ ─────────────────────► security/ (injects SecurityContext)
```

**File Structure:**
```
component/
├── mod.rs           # Module declarations
├── wrapper.rs       # ComponentWrapper (Actor + Child)
├── registry.rs      # ComponentRegistry (ComponentId → ActorAddress)
├── spawner.rs       # ComponentSpawner (creates actors)
└── supervisor.rs    # SupervisorConfig (crash handling)
```

### Dependency Inversion Pattern

The `component/` module uses traits from `core/` but receives concrete implementations from `system/`:

```rust
// core/runtime/traits.rs - ABSTRACTION
pub trait RuntimeEngine: Send + Sync {
    fn load_component(&self, id: &ComponentId, bytes: &[u8]) -> Result<ComponentHandle, WasmError>;
    fn call_handle_message(&self, handle: &ComponentHandle, msg: &ComponentMessage) 
        -> Result<Option<MessagePayload>, WasmError>;
}

// component/wrapper.rs - CONSUMER (depends on abstraction)
use crate::core::runtime::traits::RuntimeEngine;
use crate::core::component::{ComponentId, ComponentMessage};
use airssys_rt::{Actor, Child};

pub struct ComponentWrapper {
    id: ComponentId,
    engine: Arc<dyn RuntimeEngine>,  // ← Injected by system/, not concrete
    actor: Box<dyn Actor>,           // ← From airssys-rt
}

// system/manager.rs - COORDINATOR (injects concrete)
use crate::runtime::WasmtimeEngine;      // ← Concrete implementation
use crate::component::ComponentWrapper;

let engine = Arc::new(WasmtimeEngine::new());
let wrapper = ComponentWrapper::new(engine);  // ← DI
```

## Usage Patterns

### Core Module Usage (Phase 3 Tasks)

**When to add code to `core/component/`:**
- ✅ New data structures (structs, enums)
- ✅ New trait definitions (no implementations)
- ✅ Foundation types used by multiple modules
- ✅ Error types specific to component types

**When NOT to add code to `core/component/`:**
- ❌ Business logic or algorithms
- ❌ Actor lifecycle management
- ❌ Registry operations
- ❌ Imports from other internal modules

**Example - Creating New Types:**
```rust
// core/component/id.rs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ComponentId {
    pub namespace: String,
    pub name: String,
    pub instance: String,
}

impl ComponentId {
    pub fn new(namespace: impl Into<String>, name: impl Into<String>, instance: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            name: name.into(),
            instance: instance.into(),
        }
    }
}
```

### Integration Module Usage (Phase 4 Tasks)

**When to add code to `component/`:**
- ✅ Actor wrapping logic (ComponentWrapper)
- ✅ Registry operations (ComponentRegistry)
- ✅ Spawning logic (ComponentSpawner)
- ✅ Supervisor configuration
- ✅ Message handling logic

**When NOT to add code to `component/`:**
- ❌ WASM execution (belongs to runtime/)
- ❌ Security validation (belongs to security/)
- ❌ Core type definitions (belongs to core/component/)

**Example - ComponentWrapper:**
```rust
// component/wrapper.rs
use crate::core::component::{ComponentId, ComponentHandle, ComponentMessage};
use crate::core::runtime::traits::RuntimeEngine;
use airssys_rt::{Actor, Child, ActorRef};

pub struct ComponentWrapper {
    id: ComponentId,
    handle: Option<ComponentHandle>,
    engine: Arc<dyn RuntimeEngine>,  // ← From system/ injection
    actor_ref: ActorRef<ComponentMessage>,  // ← From airssys-rt
}

impl ComponentWrapper {
    pub fn new(id: ComponentId, engine: Arc<dyn RuntimeEngine>) -> Self {
        Self {
            id,
            handle: None,
            engine,
            actor_ref: ActorRef::unconnected(),
        }
    }
    
    pub fn load(&mut self, bytes: &[u8]) -> Result<(), WasmError> {
        let handle = self.engine.load_component(&self.id, bytes)?;
        self.handle = Some(handle);
        Ok(())
    }
    
    pub async fn handle_message(&mut self, msg: &ComponentMessage) -> Result<Option<MessagePayload>, WasmError> {
        if let Some(handle) = &self.handle {
            self.engine.call_handle_message(handle, msg)
        } else {
            Err(WasmError::ComponentNotFound(self.id.to_string_id()))
        }
    }
}
```

### Cross-Layer Collaboration Example

**Message Flow: Component A → Component B**

```rust
// 1. Component A sends message (via system/ coordinator)
system::RuntimeManager::send_message(&ComponentId::new("ns", "a", "1"), &ComponentId::new("ns", "b", "1"), payload)

// 2. system/ looks up ActorAddress (uses ComponentRegistry)
let address = registry.lookup(&target_id)?;  // O(1) lookup

// 3. system/ sends message to actor (via MessageBroker)
broker.send(&address, message)?;

// 4. ComponentWrapper in Component B receives message (via airssys-rt Mailbox)
//    → handle_message() called
//    → RuntimeEngine.call_handle_message() invoked
//    → WASM handle-message export called

// 5. Response routed back through ComponentWrapper → Mailbox → Component A
```

## Best Practices

### Core Module Best Practices

1. **Keep types simple and serializable** - All core types should be easy to serialize
2. **Use idiomatic Rust patterns** - `impl From<T>`, `impl Default`, `Into<String>` for flexibility
3. **Document thoroughly** - Every public type and trait must have rustdoc
4. **Test edge cases** - Test creation, validation, serialization, deserialization
5. **NO logic in tests** - Tests should verify data structure behavior, not business logic

### Integration Module Best Practices

1. **Inject dependencies** - Use Arc<dyn Trait> for RuntimeEngine, SecurityContext
2. **Delegate to airssys-rt** - Don't reimplement actor patterns, use airssys-rt
3. **Handle errors gracefully** - Convert airssys-rt errors to WASM errors
4. **Test with mocks** - Mock RuntimeEngine for unit tests
5. **Test integration** - Integration tests with real airssys-rt

### Antipatterns to Avoid

| Antipattern | Why Wrong | Correct Approach |
|-------------|-----------|------------------|
| Business logic in `core/` | Violates Layer 1 rule | Move to `component/` or appropriate module |
| Importing `runtime/` in `component/` | Breaks DI, creates coupling | Inject via `system/`, use `core/runtime/traits` |
| Direct WASM execution in `component/` | Violates module responsibility | Delegate to RuntimeEngine trait |
| Manual actor management | Duplicates airssys-rt | Use airssys-rt Actor, Child, Supervisor |
| Missing trait abstractions | Makes testing impossible | Define traits in `core/`, mock for tests |

## Integration Points

### Module Dependency Map

```
system/ (LAYER 4)
    │
    ├── Injects: Arc<WasmtimeEngine> → component/::ComponentWrapper
    ├── Injects: Arc<SecurityValidator> → component/::ComponentWrapper
    └── Coordinates: Spawning, registry, lifecycle

component/ (LAYER 3A)
    │
    ├── Uses: ComponentId, ComponentHandle, ComponentMessage (from core/component/)
    ├── Uses: Actor, Child, ActorSystem (from airssys-rt)
    ├── Implements: ComponentWrapper, ComponentRegistry, ComponentSpawner
    └── Provides: Actor wrapping, registry operations, spawning logic

runtime/ (LAYER 2B)
    │
    ├── Implements: RuntimeEngine trait (from core/runtime/)
    └── Executes: WASM handle-message, handle-callback exports

core/component/ (LAYER 1)
    │
    ├── Provides: ComponentId, ComponentHandle, ComponentMessage
    ├── Provides: ComponentLifecycle trait
    └── Depends: NOTHING except std
```

### External Integrations

| Integration | Module | Purpose |
|-------------|--------|---------|
| **airssys-rt** | `component/` | Actor, Child, ActorSystem, MessageBroker, Mailbox |
| **airssys-osl** | `system/` (injected to `component/`) | SecurityContext, ExecutionContext, AuditLogger |

## Implementation Guidelines for Tasks

### Phase 3 Tasks (Core Modules)

When implementing core module tasks (e.g., WASM-TASK-017):
1. **Create types only** - No business logic
2. **Follow ADR-WASM-028** - Use exact specifications
3. **Add unit tests** - Test type creation, validation, serialization
4. **Document thoroughly** - Every public API with rustdoc
5. **Verify no violations** - Ensure no internal imports

### Phase 4 Tasks (Integration Modules)

When implementing integration module tasks (e.g., ComponentWrapper, ComponentRegistry):
1. **Use types from core/** - ComponentId, ComponentHandle, ComponentMessage
2. **Integrate with airssys-rt** - Actor, Child, ActorSystem
3. **Implement traits from core/** - RuntimeEngine (injected), ComponentLifecycle
4. **Add unit + integration tests** - Mock RuntimeEngine, real airssys-rt
5. **Verify architecture** - No forbidden imports (runtime/, security/)

## References

### Related Documentation

**ADRs:**
- **ADR-WASM-025:** Clean-Slate Rebuild Architecture (six-module design)
- **ADR-WASM-023:** Module Boundary Enforcement (dependency rules)
- **ADR-WASM-028:** Core Module Structure (core/ specifications)

**Knowledges:**
- **KNOWLEDGE-WASM-037:** Rebuild Architecture Clean-Slate Design (layer organization)
- **KNOWLEDGE-WASM-031:** Foundational Architecture (Actor model)
- **KNOWLEDGE-WASM-018:** Component Definitions and Three-Layer Architecture

**Standards:**
- **PROJECTS_STANDARD.md:** §2.1 (3-Layer Imports), §4.3 (Module Architecture)
- **Rust Guidelines:** M-DESIGN-FOR-AI, M-MODULE-DOCS, M-STATIC-VERIFICATION

## History

| Date | Version | Change |
|------|---------|--------|
| 2026-01-08 | 1.0 | Initial creation - Clarifies core/component/ vs component/ distinction |

---

**Template Version:** 1.0  
**Last Updated:** 2026-01-08
