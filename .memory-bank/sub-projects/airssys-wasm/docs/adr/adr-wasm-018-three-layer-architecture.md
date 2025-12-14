# ADR-WASM-018: Three-Layer Architecture and Boundary Definitions

**Status:** ACCEPTED  
**Date:** 2025-12-14  
**Deciders:** Architecture Team  
**Related ADRs:** ADR-WASM-006 (Component Isolation), ADR-RT-004 (Actor/Child Separation), ADR-WASM-001 (Multicodec)  
**Related KNOWLEDGEs:** KNOWLEDGE-WASM-018 (Component Definitions)  

## Context

As airssys-wasm integrates with airssys-rt, there is significant risk of architectural confusion regarding:
1. Which layer "owns" which concerns (actor system, supervision, mailbox, etc.)
2. Where WASM-specific logic should live vs. reusable actor system logic
3. How "components" are defined and what they consist of
4. Clear boundaries between three integration layers

This ADR explicitly codifies the three-layer architecture to provide unambiguous boundaries for all future development.

## Problem Statement

Without explicit layer definitions, developers may:
- Duplicate supervision logic between airssys-wasm and airssys-rt
- Create WASM-specific actor systems instead of using airssys-rt
- Misunderstand what a "component" is at runtime
- Implement features in the wrong layer
- Create circular dependencies

## Decision

We adopt a **Three-Layer Architecture** with explicit ownership and responsibility boundaries:

### Layer 1: WASM Component Configuration & Tracking
**Location:** `airssys-wasm/src/actor/supervisor_config.rs`, `component_supervisor.rs`  
**Ownership:** airssys-wasm team  
**Responsibility:** WASM-specific configuration, component state tracking  

**What This Layer Owns:**
- ✅ `SupervisorConfig` - WASM component configuration (Permanent/Transient/Temporary)
- ✅ `BackoffStrategy` - WASM-specific backoff variants (Immediate/Linear/Exponential)
- ✅ `ComponentSupervisor` - Tracks WASM component lifecycle state
- ✅ `SupervisionHandle` - Component-level restart history
- ✅ `SupervisionState` - WASM component state machine

**What This Layer Does NOT Own:**
- ❌ Actor system implementation (use airssys-rt)
- ❌ Mailbox patterns (use airssys-rt)
- ❌ Message broker (use airssys-rt)
- ❌ Supervision tree strategy (use airssys-rt)
- ❌ Restart backoff implementation (use airssys-rt)

**Integration Point:**
- Provides configuration objects that will be **used by** ComponentSpawner and ComponentRegistry

---

### Layer 2: WASM Component Lifecycle & Spawning
**Location:** `airssys-wasm/src/actor/component_spawner.rs`, `component_registry.rs`, `component_actor.rs`  
**Ownership:** airssys-wasm team  
**Responsibility:** WASM binary loading, actor creation, component registration  

**What This Layer Owns:**
- ✅ `ComponentActor` - Actor trait implementation wrapping WASM binary
  - Implements `Actor` trait (message handling)
  - Implements `Child` trait (WASM lifecycle)
- ✅ `ComponentSpawner` - Orchestrates component spawning
  - Calls `ActorSystem::spawn()` (NOT replaces it)
  - Creates ComponentActor instances
  - Returns ActorAddress for routing
- ✅ `ComponentRegistry` - Maps ComponentId → ActorAddress
  - O(1) lookup by component ID
  - Tracks active component instances
  - Supports lifecycle visibility (register/lookup/unregister)
- ✅ `WasmRuntime` - Wasmtime Engine, Store, Instance wrapper
  - Loads WASM binaries
  - Caches export functions
  - Manages resource limits

**What This Layer Does NOT Own:**
- ❌ Mailbox implementation (airssys-rt provides)
- ❌ Message routing logic (airssys-rt MessageBroker)
- ❌ Restart decision logic (airssys-rt SupervisorNode)
- ❌ Actor spawning mechanics (airssys-rt ActorSystem)
- ❌ Message broker configuration (airssys-rt)

**Integration Point:**
- Uses `ActorSystem::spawn()` to create actors
- Uses `MessageBroker` for message delivery
- Returns `ActorAddress` from airssys-rt
- Will integrate with `SupervisorNode` in Phase 3

---

### Layer 3: Actor System Runtime
**Location:** `airssys-rt/src/` (system/, actor/, supervisor/, mailbox/, broker/)  
**Ownership:** airssys-rt team  
**Responsibility:** Low-level actor system, supervision, message delivery  

**What This Layer Owns:**
- ✅ `ActorSystem` - Core actor system implementation
  - Actor spawning infrastructure
  - Mailbox creation and assignment
  - Actor lifecycle management
- ✅ `SupervisorNode` - Supervision tree implementation
  - Restart decision making
  - Supervision strategy (OneForOne, OneForAll, RestForOne)
  - RestartBackoff with sliding windows
- ✅ `MessageBroker` - Message delivery
  - InMemoryMessageBroker
  - Message envelope handling
  - Delivery guarantees
- ✅ `Mailbox` - Message queuing
  - Bounded and unbounded variants
  - Backpressure handling
  - Async message delivery
- ✅ `ActorAddress` - Actor identification and routing
  - Named and ID-based addressing
  - Cross-thread safe routing

**Critical Note:**
airssys-wasm is a **consumer, not a replacement** of these services.

---

## Architectural Diagram

```
┌──────────────────────────────────────────────────────────────────┐
│                    APPLICATION LAYER                             │
│  (Uses components to build applications)                         │
└────────────────────────┬─────────────────────────────────────────┘
                         │
┌────────────────────────▼──────────────────────────────────────────┐
│  LAYER 2: WASM COMPONENT LIFECYCLE & SPAWNING (airssys-wasm)    │
│                                                                   │
│  ComponentSpawner                                                │
│    ├─ spawn_component() → ComponentActor                         │
│    └─ ActorSystem::spawn()                                       │
│                                                                   │
│  ComponentRegistry (HashMap<ComponentId, ActorAddress>)         │
│    ├─ register() ✓                                               │
│    ├─ lookup() - O(1)                                            │
│    └─ unregister() ✓                                             │
│                                                                   │
│  ComponentActor                                                  │
│    ├─ Actor trait (handle_message)                               │
│    ├─ Child trait (start/stop WASM)                              │
│    └─ WasmRuntime (loaded WASM binary)                           │
│                                                                   │
└────────────────────────┬──────────────────────────────────────────┘
                         │ (Uses)
┌────────────────────────▼──────────────────────────────────────────┐
│  LAYER 3: ACTOR SYSTEM RUNTIME (airssys-rt) ◄─ THE FOUNDATION   │
│                                                                   │
│  ActorSystem::spawn()                                            │
│    └─ Creates actor with mailbox assigned                        │
│                                                                   │
│  SupervisorNode<Strategy>                                        │
│    ├─ OneForOne / OneForAll / RestForOne                         │
│    ├─ RestartBackoff (exponential delays)                        │
│    └─ Makes restart decisions                                    │
│                                                                   │
│  MessageBroker<ComponentMessage>                                 │
│    ├─ Routes messages to actors                                  │
│    └─ InMemoryMessageBroker implementation                       │
│                                                                   │
│  Mailbox<ComponentMessage>                                       │
│    ├─ Bounded / Unbounded                                        │
│    ├─ Async message delivery                                     │
│    └─ Backpressure handling                                      │
│                                                                   │
│  ActorAddress (for routing)                                      │
│    └─ Identifies and routes to actors                            │
│                                                                   │
└──────────────────────────────────────────────────────────────────┘
                         ▲
        ┌────────────────┴──────────────┐
        │                               │
  OWNS              ◄───────── USES───► LAYER 1: WASM Config & Tracking
  SUPPLIES                            (airssys-wasm)
                                
  RestartBackoff ◄────────────────── SupervisorConfig
  RestartPolicy ◄────────────────── BackoffStrategy
  (Exponential only)   (Immediate/Linear/Exponential)
  
  Used by:
  SupervisorNode<OneForOne/OneForAll/RestForOne>
  ◄───────── ComponentSupervisor (optional future integration)

```

## Layer Responsibilities Matrix

| Concern | Layer 1 (WASM Config) | Layer 2 (WASM Spawning) | Layer 3 (Actor System) |
|---------|----------------------|-------------------------|------------------------|
| **Configuration** | ✅ WASM policies | ✅ Component creation | ✅ ActorSystem config |
| **Component Loading** | ❌ | ✅ WasmRuntime | ❌ |
| **Actor Creation** | ❌ | ✅ ComponentActor | ✅ (via spawn()) |
| **Message Delivery** | ❌ | ❌ | ✅ MessageBroker |
| **Mailbox** | ❌ | ❌ | ✅ Bounded/Unbounded |
| **Supervision Tree** | ❌ | ❌ (future) | ✅ SupervisorNode |
| **Restart Decisions** | ❌ | ❌ | ✅ RestartBackoff |
| **Restart Strategies** | ❌ | ❌ | ✅ OneForOne/All/Rest |
| **Component Registry** | ❌ | ✅ ComponentRegistry | ❌ |
| **Component Addressing** | ❌ | ✅ (uses RT's ActorAddress) | ✅ ActorAddress |

## Integration Patterns

### Pattern 1: Component Spawning (Phase 2)
```
Application
    ↓
ComponentSpawner::spawn_component(component_id, metadata, capabilities)
    ↓
Creates ComponentActor { wasm_runtime: Some(loaded_wasm) }
    ↓
ActorSystem::spawn(actor)  ← Crosses to Layer 3
    ↓
SupervisorNode<OneForOne> receives actor
    ↓
Mailbox<ComponentMessage> assigned
    ↓
ActorAddress returned to ComponentRegistry
    ↓
Application can send messages via ActorAddress
```

### Pattern 2: Component Message Handling (Phase 2)
```
Application sends ComponentMessage via ActorAddress
    ↓
MessageBroker routes to mailbox
    ↓
Actor::handle_message() invoked
    ↓
Calls WASM export function via WasmRuntime
    ↓
Returns result or error
```

### Pattern 3: Component Restart (Phase 3)
```
ComponentActor fails (Child::start returns error)
    ↓
SupervisorNode detects failure
    ↓
Checks SupervisorConfig (from Layer 1)
    ↓
RestartBackoff calculates delay (from Layer 3)
    ↓
ComponentActor::start() called again
    ↓
WasmRuntime loads WASM binary again
    ↓
Mailbox routing resumed
```

## Explicit Non-Ownership Statements

### What airssys-wasm DOES NOT own:
1. **Actor System Infrastructure** - airssys-rt owns ActorSystem, spawn(), mailbox creation
2. **Message Broker** - airssys-rt owns MessageBroker, message routing
3. **Supervision Tree** - airssys-rt owns SupervisorNode and supervision strategies
4. **Low-Level Backoff** - airssys-rt owns RestartBackoff implementation
5. **Restart Policies** - airssys-rt defines Permanent/Transient/Temporary semantics
6. **Actor Addressing** - airssys-rt owns ActorAddress and routing
7. **Mailbox Patterns** - airssys-rt owns bounded/unbounded mailbox implementations

### What airssys-rt DOES NOT own:
1. **WASM Binary Loading** - airssys-wasm owns WasmRuntime and Wasmtime integration
2. **Component Definition** - airssys-wasm owns ComponentActor (what a "component" is)
3. **Component Configuration** - airssys-wasm owns SupervisorConfig
4. **Component Spawning Orchestration** - airssys-wasm owns ComponentSpawner
5. **Component Registry** - airssys-wasm owns ComponentRegistry
6. **WASM-Specific Supervision** - airssys-wasm owns ComponentSupervisor

## Dependencies

### airssys-wasm depends on:
- ✅ airssys-rt (ActorSystem, MessageBroker, ActorAddress, SupervisorNode)
- ✅ airssys-osl (SecurityContext for future capability enforcement)
- ✅ Standard Rust async ecosystem

### airssys-rt depends on:
- ❌ airssys-wasm (independent actor system)

## Consequences

### Positive Consequences
1. ✅ Clear ownership prevents code duplication
2. ✅ Single source of truth for each concern
3. ✅ Easy to test each layer independently
4. ✅ Prevents "actor system reimplementation" risk
5. ✅ Scales to multiple consumer crates using airssys-rt
6. ✅ Supports future component frameworks (not just WASM)

### Constraints
1. ⚠️ airssys-wasm must stay dependent on airssys-rt (one-way dependency)
2. ⚠️ Layer 1 and 2 tightly coupled (acceptable, both airssys-wasm)
3. ⚠️ Future phases must maintain these boundaries
4. ⚠️ Cross-cutting concerns must be explicitly assigned to exactly one layer

## Implementation Status

### Currently Implemented (2025-12-14)
- ✅ Layer 1: SupervisorConfig, BackoffStrategy, ComponentSupervisor
- ✅ Layer 2: ComponentActor, ComponentSpawner, ComponentRegistry, WasmRuntime
- ✅ Layer 3: Used as-is from airssys-rt (no changes required)

### Upcoming Phases
- **Phase 3**: Integrate Layer 1 + Layer 2 with Layer 3's SupervisorNode
- **Block 4**: Security integration (airssys-osl bridge)
- **Block 5**: Inter-component communication
- **Block 6+**: Additional features maintaining layer boundaries

## Related Documents

- **ADR-WASM-006**: Component Isolation and Sandboxing (Actor-based approach)
- **ADR-RT-004**: Actor and Child Trait Separation (foundation of Layer 2)
- **KNOWLEDGE-WASM-018**: Component Definitions and Architecture Layers (detailed reference)
- **KNOWLEDGE-WASM-016**: Actor System Integration Implementation Guide

## Approval Checklist

- [x] All layer responsibilities clearly defined
- [x] No ambiguous ownership of concerns
- [x] Integration patterns documented
- [x] Non-ownership explicitly stated
- [x] Consequences analyzed
- [x] Related to existing ADRs
- [x] Implementation status captured
