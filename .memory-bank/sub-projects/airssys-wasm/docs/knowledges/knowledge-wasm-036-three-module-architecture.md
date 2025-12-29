# Three-Module Architecture: host_system/, actor/, messaging/, runtime/ Integration

**Document ID:** KNOWLEDGE-WASM-036
**Created:** 2025-12-29
**Status:** Active Reference
**Category:** Architecture / Module Design / Integration
**Complexity:** High
**Maturity:** Draft

## Overview

This document establishes the **correct architectural design** for the four primary modules in airssys-wasm: `host_system/`, `actor/`, `messaging/`, and `runtime/`. It resolves circular dependency issues in the previous design by introducing a dedicated host_system layer and establishes clear, non-overlapping responsibilities with one-way dependency flow only.

**Key Decision:** Introduce `host_system/` as the central coordinator that manages system initialization, component lifecycle, and message flow coordination. This eliminates circular dependencies and provides clear ownership of host_system logic.

## Context

### Problem Statement

The previous three-module architecture (`actor/`, `messaging/`, `runtime/`) had **critical architectural flaws**:

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
   - Message flow host_system
   - Startup/shutdown procedures

3. **Overlapping Responsibilities:** Messaging-related logic was split between `messaging/` and `actor/`, creating confusion about where features belonged.

4. **Violated Module Boundary Rules (ADR-WASM-023):** Circular dependencies violated the principle of one-way dependencies between modules.

### Scope

This knowledge applies to:
- All new development in airssys-wasm module architecture
- Refactoring efforts to eliminate circular dependencies
- Understanding the correct placement of host_system logic
- System initialization and shutdown procedures
- Component lifecycle management

### Prerequisites

- **ADR-WASM-018:** Three-Layer Architecture (foundation)
- **ADR-WASM-022:** Circular Dependency Remediation (dependency rules)
- **ADR-WASM-023:** Module Boundary Enforcement (mandatory rules)
- **KNOWLEDGE-WASM-030:** Module Architecture - Hard Requirements

## Technical Content

### The Four-Module Architecture

The corrected architecture introduces `host_system/` as a top-level coordinator:

```
┌─────────────────────────────────────────────────────────┐
│   host_system/ (NEW - coordinates everything)       │
│   • Runtime initialization                             │
│   • Component lifecycle management                     │
│   • Message flow coordination                         │
│   • Startup/shutdown procedures                       │
└──────────────────┬──────────────┬────────────────────┘
                   │              │
          owns & manages    owns & manages
                   │              │
        ┌──────────▼─────┐  ┌───▼──────────┐
        │  actor/       │  │  messaging/   │
        │  (wrappers)   │  │  (broker)    │
        │               │  │              │
        │•ComponentActor│  │•MessageBroker│
        │•Registry     │  │•FireAndForget│
        │•Router       │  │•RequestResponse│
        │•Subscriber   │  │•ResponseRouter│
        │•Spawner      │  │•MulticodecCodec│
        │•Supervisor   │  │•Topics       │
        └──────────────┘  └───────────────┘
                   │              │
          uses for         uses for
        execution         communication
                   │              │
                   └───────┬──────┘
                           │
                   ┌───────▼──────────┐
                   │  runtime/       │
                   │  (execution)    │
                   │                 │
                   │•WasmEngine      │
                   │•ComponentLoader │
                   │•AsyncHostRegistry│
                   │•HostFunctions   │
                   │•ResourceLimits  │
                   │•StoreManager   │
                   └─────────────────┘
                           │
                     depends on
                           │
                   ┌───────▼──────────┐
                   │  core/          │
                   │  (types)        │
                   │                 │
                   │•ComponentId    │
                   │•ComponentMessage│
                   │•WasmError      │
                   │•CapabilitySet  │
                   │•All traits     │
                   └─────────────────┘
```

### Dependency Flow (ONE-WAY ONLY)

The correct dependency flow is strictly one-way with **no cycles**:

```
host_system/ ───► actor/
host_system/ ───► messaging/
host_system/ ───► runtime/
actor/ ───► runtime/
messaging/ ───► runtime/
runtime/ ───► core/
core/ ───► (nothing - foundation)
```

**Explicit Dependencies:**

1. **`host_system/` depends on:**
   - `actor/` - ComponentActor, ComponentRegistry, ComponentSpawner, Supervisor
   - `messaging/` - MessageBroker, MessagingService, FireAndForget, RequestResponse
   - `runtime/` - WasmEngine, ComponentLoader, AsyncHostRegistry
   - `core/` - All shared types and traits

2. **`actor/` depends on:**
   - `runtime/` - WasmEngine (for executing WASM code)
   - `core/` - Shared types (ComponentId, ComponentMessage, errors, traits)
   - **NEVER** messaging/, host_system/ (enforced by module boundaries)

3. **`messaging/` depends on:**
   - `runtime/` - Callback execution only (via host functions)
   - `core/` - Shared types (ComponentId, ComponentMessage, CorrelationId)
   - **NEVER** actor/ (moved CorrelationTracker to host_system/ or messaging/)
   - **NEVER** host_system/ (host_system owns messaging/)

4. **`runtime/` depends on:**
   - `core/` - Shared types only
   - `security/` - Resource limits and policies
   - **NEVER** actor/, messaging/, host_system/ (enforced by ADR-WASM-023)

5. **`core/` depends on:**
   - `std` only - No internal dependencies

### Module Responsibilities

#### `host_system/`: The Host System Layer (The Conductor)

**Purpose:** Coordinate all system operations - initialization, lifecycle, and message flow.

**Owns:**
- ✅ `RuntimeManager` - Central coordinator for all operations
- ✅ System initialization logic - Create infrastructure in correct order
- ✅ Component lifecycle management - Spawn, start, stop, supervise
- ✅ Message flow coordination - Wire up components with broker
- ✅ Startup/shutdown procedures - Graceful system lifecycle
- ✅ Correlation tracking - Track pending request-response pairs (moved from actor/)
- ✅ Timeout handling - Enforce request timeouts (moved from messaging/)

**Does NOT Own:**
- ❌ WASM execution (runtime/)
- ❌ Message broker implementation (messaging/)
- ❌ Actor system primitives (actor/)
- ❌ Component actor logic (actor/)

**The Key Principle:**
> **Host System COORDINATES.** It decides what to do, when to do it, and how modules interact. It does NOT implement the core operations.

#### `actor/`: The Wrapper Layer (The Bridge)

**Purpose:** Wrap WASM components in the airssys-rt actor system.

**Owns:**
- ✅ `ComponentActor` - Wraps WASM in Actor + Child traits
- ✅ `ComponentRegistry` - Maps ComponentId → ActorAddress (O(1) lookup)
- ✅ `ComponentSpawner` - Spawns component actors
- ✅ `ComponentSupervisor` - Supervision and restart logic
- ✅ `MessageRouter` - Routes messages from broker to ComponentActor mailboxes
- ✅ `ActorSystemSubscriber` - Subscribes to MessageBroker, receives messages
- ✅ `HealthMonitor` - Health check management
- ✅ `LifecycleHooks` - Component lifecycle event hooks

**Does NOT Own:**
- ❌ Correlation tracking (moved to host_system/)
- ❌ Response routing (messaging/)
- ❌ Timeout handling (moved to host_system/)
- ❌ Message broker (messaging/)
- ❌ WASM execution (runtime/)

**The Key Principle:**
> **Actor WRAPS.** It wraps WASM components in actor system primitives. It does NOT orchestrate the system.

#### `messaging/`: The Communication Layer (The Broker)

**Purpose:** Provide message broker infrastructure and messaging patterns.

**Owns:**
- ✅ `MessageBroker` - Message distribution infrastructure
- ✅ `MessagingService` - MessageBroker singleton management
- ✅ `FireAndForget` - One-way messaging pattern
- ✅ `RequestResponse` - Request-response pattern
- ✅ `ResponseRouter` - Routes responses to callbacks
- ✅ `MulticodecCodec` - Message encoding/decoding
- ✅ `MessageReceptionMetrics` - Tracks messaging statistics
- ✅ `Topics` - Topic-based pub-sub (Phase 2)

**Does NOT Own:**
- ❌ Correlation tracking (moved to host_system/)
- ❌ Component actors (actor/)
- ❌ Message routing to actors (actor/)
- ❌ WASM execution (runtime/)
- ❌ Host system logic (host_system/)

**The Key Principle:**
> **Messaging TRANSPORTS.** It provides message distribution and patterns. It does NOT decide where messages go.

#### `runtime/`: The Execution Layer (The Muscle)

**Purpose:** Execute WASM code and provide host functions.

**Owns:**
- ✅ `WasmEngine` - Wasmtime-based execution engine
- ✅ `ComponentLoader` - WASM binary loading and validation
- ✅ `AsyncHostRegistry` - Host function registry
- ✅ `SendMessageHostFunction` - `send-message()` host function
- ✅ `SendRequestHostFunction` - `send-request()` host function
- ✅ `AsyncFileReadFunction` - File read host function
- ✅ `AsyncHttpFetchFunction` - HTTP fetch host function
- ✅ `AsyncSleepFunction` - Sleep host function
- ✅ `ComponentResourceLimiter` - Memory/CPU limits enforcement
- ✅ `StoreManager` - WASM store management

**Does NOT Own:**
- ❌ Messaging host_system (host_system/)
- ❌ Component lifecycle (host_system/, actor/)
- ❌ Message broker (messaging/)
- ❌ Actor system (actor/)

**The Key Principle:**
> **Runtime EXECUTES.** It executes WASM code and provides host functions. It does NOT orchestrate or route messages.

#### `core/`: The Foundation Layer (The Types)

**Purpose:** Provide shared types, traits, and abstractions.

**Owns:**
- ✅ All shared types (ComponentId, ComponentMessage, WasmError, etc.)
- ✅ All trait contracts (RuntimeEngine, Component, etc.)
- ✅ All configuration types (ResourceLimits, SecurityConfig, etc.)
- ✅ All error types
- ✅ All capability types

**Does NOT Own:**
- ❌ Any implementation logic

**The Key Principle:**
> **Core DEFINES.** It provides the contracts that all modules implement.

### Architectural Metaphors

| Module | Metaphor | Role |
|--------|-----------|------|
| **host_system/** | Conductor | Directs the orchestra, decides what plays when |
| **actor/** | Bridge | Connects WASM to actor system, adapts interfaces |
| **messaging/** | Courier | Delivers messages between parties |
| **runtime/** | Musician | Plays the notes when told to by conductor |
| **core/** | Sheet Music | Defines the notes and patterns |

### Dependency Verification Commands

**Run these before committing any code:**

```bash
# Check 1: runtime/ must NOT import from host_system/, actor/, messaging/
echo "Checking runtime/ → forbidden modules..."
VIOLATIONS=$(grep -r "use crate::host_system\|use crate::actor\|use crate::messaging" src/runtime/ 2>/dev/null)
if [ -n "$VIOLATIONS" ]; then
    echo "❌ VIOLATION: runtime/ imports from forbidden modules"
    echo "$VIOLATIONS"
    exit 1
fi
echo "✅ runtime/ is clean"

# Check 2: messaging/ must NOT import from host_system/, actor/
echo "Checking messaging/ → forbidden modules..."
VIOLATIONS=$(grep -r "use crate::host_system\|use crate::actor" src/messaging/ 2>/dev/null)
if [ -n "$VIOLATIONS" ]; then
    echo "❌ VIOLATION: messaging/ imports from forbidden modules"
    echo "$VIOLATIONS"
    exit 1
fi
echo "✅ messaging/ is clean"

# Check 3: actor/ must NOT import from host_system/, messaging/
echo "Checking actor/ → forbidden modules..."
VIOLATIONS=$(grep -r "use crate::host_system\|use crate::messaging" src/actor/ 2>/dev/null)
if [ -n "$VIOLATIONS" ]; then
    echo "❌ VIOLATION: actor/ imports from forbidden modules"
    echo "$VIOLATIONS"
    exit 1
fi
echo "✅ actor/ is clean"

# Check 4: core/ must NOT import from any internal module
echo "Checking core/ → internal module violations..."
VIOLATIONS=$(grep -r "use crate::" src/core/ 2>/dev/null)
if [ -n "$VIOLATIONS" ]; then
    echo "❌ VIOLATION: core/ imports from internal modules"
    echo "$VIOLATIONS"
    exit 1
fi
echo "✅ core/ is clean"

echo ""
echo "✅ All module boundary checks passed!"
```

## Usage Patterns

### System Initialization

**Pattern:** `host_system/` initializes all infrastructure in correct order.

```rust
use airssys_wasm::host_system::RuntimeManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create orchestrator (initializes everything)
    let mut orchestrator = RuntimeManager::new().await?;
    
    // 2. Load and spawn components
    let component_bytes = std::fs::read("component.wasm")?;
    let component_id = ComponentId::new("my-component");
    let caps = CapabilitySet::new();
    
    orchestrator.spawn_component(component_id.clone(), component_bytes, caps).await?;
    
    // 3. System is ready - messages flow automatically
    
    // 4. Graceful shutdown
    orchestrator.shutdown().await?;
    
    Ok(())
}
```

### Component Lifecycle

**Pattern:** `host_system/` manages complete lifecycle, delegating to `actor/` and `runtime/`.

```rust
impl RuntimeManager {
    pub async fn spawn_component(
        &mut self,
        id: ComponentId,
        bytes: Vec<u8>,
        caps: CapabilitySet,
    ) -> Result<ActorAddress, WasmError> {
        // 1. Load WASM (delegates to runtime/)
        let handle = self.engine.load(&id, &bytes).await?;
        
        // 2. Create component actor (delegates to actor/)
        let actor = ComponentActor::new(id.clone(), handle, caps);
        
        // 3. Spawn actor (delegates to actor/)
        let addr = self.spawner.spawn(actor).await?;
        
        // 4. Register for messaging (orchestrator coordinates)
        self.subscriber.register_mailbox(id.clone(), addr.mailbox()).await?;
        self.correlation_tracker.register_component(id.clone()).await?;
        
        // 5. Start health monitoring (orchestrator coordinates)
        self.health_monitor.start_monitoring(id).await?;
        
        Ok(addr)
    }
    
    pub async fn stop_component(&mut self, id: &ComponentId) -> Result<(), WasmError> {
        // 1. Stop health monitoring
        self.health_monitor.stop_monitoring(id).await?;
        
        // 2. Unregister from messaging
        self.subscriber.unregister_mailbox(id).await?;
        self.correlation_tracker.unregister_component(id).await?;
        
        // 3. Stop actor (delegates to actor/)
        let addr = self.registry.lookup(id)?;
        addr.stop(Duration::from_secs(5)).await?;
        
        Ok(())
    }
}
```

### Message Flow Coordination

**Pattern:** `host_system/` wires up message flow but doesn't route individual messages.

```rust
impl RuntimeManager {
    pub async fn new() -> Result<Self, WasmError> {
        // 1. Create infrastructure layers
        let engine = Arc::new(WasmEngine::new()?);
        let broker = Arc::new(MessageBroker::new());
        let registry = ComponentRegistry::new();
        
        // 2. Create actor-level infrastructure
        let subscriber = ActorSystemSubscriber::new(Arc::clone(&broker));
        let spawner = ComponentSpawner::new(
            Arc::clone(&engine),
            registry.clone(),
        );
        
        // 3. Create host_system-level infrastructure
        let correlation_tracker = Arc::new(RwLock::new(CorrelationTracker::new()));
        let timeout_handler = Arc::new(TimeoutHandler::new());
        let response_router = Arc::new(ResponseRouter::new(
            Arc::clone(&correlation_tracker),
            Arc::clone(&timeout_handler),
        ));
        
        // 4. Start subscriber (wires up message flow)
        subscriber.start().await?;
        
        Ok(Self {
            engine,
            broker,
            registry,
            subscriber,
            spawner,
            correlation_tracker,
            timeout_handler,
            response_router,
            health_monitor: Arc::new(HealthMonitor::new()),
        })
    }
}
```

### Adding New Features

**Pattern:** Add to appropriate module based on responsibility.

```
┌─────────────────────────────────────────────────────────┐
│  Feature Type                                        │  Location          │
├─────────────────────────────────────────────────────────┤
│  System initialization                                 │  host_system/    │
│  Component lifecycle management                         │  host_system/    │
│  Request correlation tracking                         │  host_system/    │
│  Timeout enforcement                                 │  host_system/    │
│  Actor wrappers (ComponentActor)                      │  actor/           │
│  Message routing to actors                           │  actor/           │
│  Actor system subscriber                             │  actor/           │
│  Message broker                                     │  messaging/       │
│  Message patterns (fire-and-forget, request-response)  │  messaging/       │
│  Response routing                                    │  messaging/       │
│  Message encoding/decoding                          │  messaging/       │
│  WASM execution                                    │  runtime/         │
│  Host functions                                     │  runtime/         │
│  Resource limits                                    │  runtime/         │
│  Shared types and traits                            │  core/            │
└─────────────────────────────────────────────────────────┘
```

### Common Mistakes

❌ **Wrong:** Adding host_system logic to actor/
```rust
// actor/component_spawner.rs - DON'T DO THIS
pub fn orchestrate_system_startup(...) { ... }
```

✅ **Correct:** Host system logic belongs in host_system/
```rust
// host_system/runtime_orchestrator.rs - DO THIS
pub fn new(...) -> Result<Self, WasmError> { ... }
```

❌ **Wrong:** runtime/ imports from messaging/
```rust
// runtime/async_host.rs - DON'T DO THIS
use crate::messaging::MessagingService;

// ❌ Violates module boundary!
self.messaging_service.broker().publish(...).await?;
```

✅ **Correct:** Host functions use broker passed by host_system/
```rust
// runtime/async_host.rs - DO THIS
pub struct SendMessageHostFunction {
    broker: Arc<MessageBroker>,  // Passed in, not imported
}

impl SendMessageHostFunction {
    pub async fn send(&self, msg: ComponentMessage) -> Result<(), WasmError> {
        self.broker.publish(msg).await  // ✅ Uses passed broker
    }
}
```

❌ **Wrong:** messaging/ imports from actor/
```rust
// messaging/router.rs - DON'T DO THIS
use crate::actor::message::CorrelationTracker;

// ❌ Violates module boundary!
let pending = CorrelationTracker::lookup(...)?;
```

✅ **Correct:** messaging/ receives correlation tracker from host_system/
```rust
// messaging/router.rs - DO THIS
pub struct ResponseRouter {
    tracker: Arc<RwLock<CorrelationTracker>>,  // Passed in
}

impl ResponseRouter {
    pub async fn route(&self, correlation_id: &str, response: &[u8]) {
        let tracker = self.tracker.read().await;
        let pending = tracker.lookup(correlation_id)?;  // ✅ Uses passed tracker
    }
}
```

❌ **Wrong:** actor/ imports from messaging/
```rust
// actor/message_router.rs - DON'T DO THIS
use crate::messaging::MessageBroker;

// ❌ Violates module boundary!
self.broker.publish(msg).await?;
```

✅ **Correct:** actor/ receives subscriber from host_system/
```rust
// actor/message/router.rs - DO THIS
pub struct MessageRouter {
    subscriber: Arc<RwLock<ActorSystemSubscriber>>,  // Passed in
}

impl MessageRouter {
    pub async fn route_to_component(&self, msg: ComponentMessage) {
        let subscriber = self.subscriber.read().await;
        subscriber.route_to_component(msg).await;  // ✅ Uses passed subscriber
    }
}
```

## Integration Points

### Dependencies Summary

```
host_system/
  ├── actor (ComponentActor, ComponentRegistry, ComponentSpawner, Supervisor)
  ├── messaging (MessageBroker, MessagingService, FireAndForget, RequestResponse)
  ├── runtime (WasmEngine, ComponentLoader, AsyncHostRegistry)
  ├── core (ComponentId, ComponentMessage, WasmError, all traits)
  └── airssys_rt (ActorSystem, MessageBroker, SupervisorNode)

actor/
  ├── runtime (WasmEngine for execution)
  ├── core (ComponentId, ComponentMessage, errors, traits)
  └── airssys_rt (Actor, Child, SupervisorNode, MessageEnvelope)

messaging/
  ├── runtime (host functions for callback execution only)
  ├── core (ComponentId, ComponentMessage, CorrelationId, traits)
  └── airssys_rt (InMemoryMessageBroker)

runtime/
  ├── core (ComponentId, ComponentMessage, configs, traits)
  ├── security (ResourceLimiter, CapabilitySet)
  └── airssys_rt (none - only Wasmtime)

core/
  └── std only (no internal dependencies)
```

### Performance Characteristics

| Operation | Primary Module | Secondary Module | Latency Target |
|-----------|---------------|------------------|----------------|
| System initialization | host_system/ | All modules | ≤100ms |
| Component spawn | host_system/ | actor/, runtime/ | ≤10ms |
| MessageBroker routing | messaging/ (broker) | airssys-rt | ≤211ns |
| ComponentId lookup | host_system/ (registry) | N/A | ≤9ns |
| Mailbox delivery | actor/ (subscriber) | airssys-rt | ≤40ns |
| WASM function execution | runtime/ (engine) | actor/ (orchestrator) | ≤500μs |
| Response correlation | host_system/ (tracker) | messaging/ (router) | ≤200ns |

### Migration from Previous Architecture

**Breaking Changes:**

1. **Removed imports:**
   - Remove `use crate::messaging::MessagingService;` from `runtime/async_host.rs`
   - Remove `use crate::actor::message::CorrelationTracker;` from `messaging/router.rs`
   - Move `CorrelationTracker` from `actor/message/` to `host_system/`

2. **New imports:**
   - Add `use crate::host_system::RuntimeManager;` to initialization code
   - Pass dependencies via constructors instead of importing

3. **API Changes:**
   - Host functions receive dependencies via constructor
   - Messaging patterns receive dependencies via constructor
   - Actor infrastructure receives dependencies via constructor

**Migration Steps:**

```bash
# 1. Create host_system module
mkdir -p src/host_system

# 2. Move CorrelationTracker to host_system/
git mv src/actor/message/correlation_tracker.rs src/host_system/correlation_tracker.rs

# 3. Update imports in host_system/
# Update all use crate::actor::message::CorrelationTracker to use crate::host_system::CorrelationTracker

# 4. Create RuntimeManager
# Create src/host_system/runtime_orchestrator.rs with system initialization logic

# 5. Remove forbidden imports
# Remove all imports from runtime/ that reference messaging/ or actor/
# Remove all imports from messaging/ that reference actor/
# Remove all imports from actor/ that reference messaging/

# 6. Update lib.rs
# Add pub mod host_system;
# Add re-exports for public API

# 7. Update initialization code
# Replace direct module instantiation with RuntimeManager::new()
```

## Security Considerations

### Security Implications

- **No Impact:** Module restructuring does NOT change security enforcement
- **Same Capabilities:** Capability checks remain in the same locations
- **Same Validation:** Security validation remains in `security/` module

### Migration Safety

- **Backward Incompatible:** This is a breaking change
- **Testing Required:** All existing tests must be updated
- **Audit Trail:** Document all changes for security review

## Maintenance

### Review Schedule

**Review After:** Any module architecture changes or when adding new host_system features

### Update Triggers

- New host_system patterns added
- Module boundaries adjusted
- Circular dependencies suspected
- Performance issues detected in host_system

### Owner/Maintainer

**Module Architecture Owner:** All developers working on airssys-wasm
**Knowledge Owner:** Architecture Team

## References

### Related ADRs

- **ADR-WASM-018:** Three-Layer Architecture (foundation)
- **ADR-WASM-022:** Circular Dependency Remediation (dependency rules)
- **ADR-WASM-023:** Module Boundary Enforcement (mandatory rules)
- **ADR-WASM-009:** Component Communication Model (messaging flow)

### Related Knowledge

- **KNOWLEDGE-WASM-005:** Inter-Component Messaging Architecture (comprehensive)
- **KNOWLEDGE-WASM-016:** Actor System Integration Implementation Guide
- **KNOWLEDGE-WASM-024:** Component Messaging Clarifications
- **KNOWLEDGE-WASM-030:** Module Architecture - Hard Requirements (CRITICAL)

### Replaces

- **KNOWLEDGE-WASM-035:** Module Orchestration (superseded - contained circular dependency)

## History

### Version History

- **2025-12-29:** 1.0 - Initial documentation of four-module architecture with host_system layer

### Review History

- **2025-12-29:** Documented based on architectural analysis and resolution of circular dependency issues

---

**Document Status:** Active Reference
**Template Version:** 1.0
**Last Updated:** 2025-12-29
