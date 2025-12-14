# KNOWLEDGE-WASM-018: Component Definitions and Three-Layer Architecture

**Status:** CURRENT  
**Date:** 2025-12-14  
**Revision:** 1.0  
**Related ADR:** ADR-WASM-018 (Three-Layer Architecture)  
**Related KNOWLEDGE:** KNOWLEDGE-WASM-016 (Actor System Integration)  

## Purpose

This document provides detailed reference material for:
1. **What is a "Component"?** - Precise runtime definition
2. **Three-Layer Architecture** - Detailed breakdown and relationships
3. **Ownership Matrix** - Which team owns what
4. **Integration Patterns** - How layers connect
5. **Development Guidelines** - Where to implement features

This is the **definitive reference** for component architecture in airssys-wasm.

---

## Part 1: Component Definition

### 1.1 What is a "Component"?

A **Component** in airssys-wasm is a WASM binary loaded at runtime and managed by an actor within the airssys-rt actor system.

### 1.2 Precise Component Definition

**At Creation Time:**
```
Component := {
    component_id: ComponentId,                    // Unique identifier
    wasm_binary: &[u8],                          // Raw WASM bytes
    metadata: ComponentMetadata,                  // Name, version, etc.
    capabilities: CapabilitySet,                 // Permissions granted
}
```

**At Runtime (In Memory):**
```
Component := ComponentActor {
    component_id: ComponentId,
    state: ActorState,                           // Ready, Failed, etc.
    wasm_runtime: Option<WasmRuntime> {          // ← LOADED WASM BINARY
        engine: Engine,                          // Wasmtime engine
        store: Store,                            // Wasmtime store
        instance: Instance,                      // ← Contains loaded WASM
        exports: ExportCache {                   // Cached function exports
            invoke: Option<TypedFunc>,
            handle_message: Option<TypedFunc>,
            health_check: Option<TypedFunc>,
        }
    },
    mailbox: Mailbox<ComponentMessage>,          // ← Messages from airssys-rt
    capabilities: CapabilitySet,                 // Runtime permission set
    created_at: DateTime<Utc>,                   // Creation timestamp
}
```

**Physical Reality in airssys-rt:**
```
ComponentActor (actor spawned in airssys-rt)
    ↓
    ├─ ActorAddress (from airssys-rt)
    │   └─ Routes messages to this specific actor
    │
    ├─ Mailbox<ComponentMessage> (from airssys-rt)
    │   └─ Contains pending ComponentMessage items
    │
    └─ WasmRuntime (airssys-wasm provides)
        └─ Instance = LOADED WASM BINARY
            └─ Can call exported functions
```

### 1.3 Key Characteristics

| Characteristic | Value | Owner |
|---|---|---|
| **One WASM Binary?** | ✅ Yes, per component | airssys-wasm (WasmRuntime) |
| **One Actor?** | ✅ Yes, per component | airssys-rt (ActorSystem) |
| **One Mailbox?** | ✅ Yes, per component | airssys-rt (created automatically) |
| **One ActorAddress?** | ✅ Yes, per component | airssys-rt + airssys-wasm (ComponentRegistry) |
| **One ComponentId?** | ✅ Yes, per component | airssys-wasm (ComponentRegistry) |
| **Concurrent Instances?** | ❌ No, one at a time | (Future: pooling in Block 7) |

### 1.4 Component Lifecycle

```
1. CREATION
   ComponentId + WasmBinary → Component specification

2. SPAWNING
   ComponentSpawner::spawn_component()
      ├─ Creates ComponentActor instance
      ├─ ActorSystem::spawn() registers in actor system
      └─ Returns ActorAddress for routing

3. STARTUP
   Child::start() (ComponentActor implements)
      ├─ Wasmtime::new_store()
      ├─ Wasmtime::instantiate(wasm_binary)
      ├─ Cache exported functions
      └─ Set state to "Ready"

4. RUNNING
   Messages arrive via mailbox
   Actor::handle_message() invoked
      ├─ Deserialize ComponentMessage
      ├─ Call appropriate WASM export
      ├─ Serialize result
      └─ Send response

5. RESTART (on failure)
   Child::stop() called
      ├─ Graceful shutdown timeout
      ├─ Resource cleanup
      └─ Store deallocated

   Child::start() called again
      ├─ Fresh instance created
      ├─ WASM reloaded
      └─ Ready for messages

6. SHUTDOWN
   Child::stop() called
      ├─ Graceful shutdown (configurable timeout)
      ├─ Resource cleanup
      ├─ ActorAddress invalidated
      └─ ComponentRegistry::unregister()
```

### 1.5 "Component" vs "ComponentActor" vs "WasmRuntime"

These terms are often used together. Here's the precise distinction:

| Term | Definition | Owner | Scope |
|------|-----------|-------|-------|
| **Component** | Abstract concept: WASM binary + actor | both | Logical/architectural |
| **ComponentActor** | Rust struct implementing Actor + Child | airssys-wasm | Code structure |
| **WasmRuntime** | Wasmtime wrapper (Engine, Store, Instance) | airssys-wasm | Concrete runtime |
| **Instance** | Loaded WASM module (from wasmtime-rs) | airssys-wasm | Wasmtime term |
| **ActorAddress** | Handle to send messages to this component | airssys-rt | Routing/messaging |

**In Conversation:**
- "The component failed" = ComponentActor.Child::start() returned error
- "The WASM binary was loaded" = WasmRuntime.instance is populated
- "Send a message to the component" = Use ActorAddress from ComponentRegistry

---

## Part 2: Three-Layer Architecture (Detailed)

### 2.1 Layer 1: WASM Component Configuration & Tracking

**Purpose:** Define WASM-specific configuration and track component state

**Location:** 
- `airssys-wasm/src/actor/supervisor_config.rs`
- `airssys-wasm/src/actor/component_supervisor.rs`

**Core Structs:**
```rust
pub enum RestartPolicy {
    Permanent,    // Always restart
    Transient,    // Restart only on error
    Temporary,    // Never restart
}

pub enum BackoffStrategy {
    Immediate,                   // No delay
    Linear { base_delay: Duration },
    Exponential {                // Recommended for production
        base_delay: Duration,
        multiplier: f32,
        max_delay: Duration,
    }
}

pub struct SupervisorConfig {
    pub restart_policy: RestartPolicy,
    pub max_restarts: u32,
    pub time_window: Duration,
    pub backoff_strategy: BackoffStrategy,
    pub shutdown_timeout: Duration,
    pub startup_timeout: Duration,
}

pub struct ComponentSupervisor {
    supervision_handles: HashMap<ComponentId, SupervisionHandle>,
}

pub struct SupervisionHandle {
    pub component_id: ComponentId,
    pub restart_count: u32,
    pub restart_history: Vec<(DateTime<Utc>, bool)>,
    pub config: SupervisorConfig,
    pub state: SupervisionState,  // Initializing, Running, Restarting, etc.
}
```

**Responsibilities:**
- ✅ Define what restart policies mean for WASM components
- ✅ Define what backoff strategies are available
- ✅ Track component-level restart history
- ✅ Provide restart decision input (not decision-making)

**What Layer 1 Does NOT Do:**
- ❌ Make restart decisions (Layer 3 does)
- ❌ Implement mailbox (Layer 3 does)
- ❌ Route messages (Layer 3 does)
- ❌ Implement exponential backoff calculation (Layer 3 does, but can reference our config)

**Dependencies:**
```
Layer 1 (Configuration)
    ↓ (used by)
Layer 2 (ComponentSpawner, ComponentRegistry)
    ↓ (integrates with)
Layer 3 (ActorSystem, SupervisorNode)
```

### 2.2 Layer 2: WASM Component Lifecycle & Spawning

**Purpose:** Manage WASM binary loading and component actor lifecycle

**Location:**
- `airssys-wasm/src/actor/component_actor.rs`
- `airssys-wasm/src/actor/component_spawner.rs`
- `airssys-wasm/src/actor/component_registry.rs`
- `airssys-wasm/src/runtime/engine.rs` (WasmRuntime)

**Core Structs:**
```rust
pub struct ComponentActor {
    component_id: ComponentId,
    state: ActorState,                    // Ready, Failed, etc.
    wasm_runtime: Option<WasmRuntime>,    // ← LOADED WASM
    capabilities: CapabilitySet,
    created_at: DateTime<Utc>,
}

impl Actor for ComponentActor {
    // Handles ComponentMessage from mailbox
    async fn handle_message(&mut self, msg: ComponentMessage) -> Result<()>
}

impl Child for ComponentActor {
    // Manages WASM lifecycle
    async fn start(&mut self) -> Result<()>     // Load WASM binary
    async fn stop(&mut self, timeout: Duration) -> Result<()>  // Unload
}

pub struct ComponentSpawner<B: MessageBroker<ComponentMessage>> {
    actor_system: ActorSystem<ComponentMessage, B>,
}

impl ComponentSpawner {
    pub async fn spawn_component(
        &self,
        component_id: ComponentId,
        metadata: ComponentMetadata,
        capabilities: CapabilitySet,
    ) -> Result<ActorAddress> {
        // 1. Create ComponentActor instance
        let actor = ComponentActor::new(component_id, metadata, capabilities);
        
        // 2. Spawn in airssys-rt (NOT create our own actor system)
        let addr = self.actor_system
            .spawn()
            .with_name(component_id.as_str())
            .spawn(actor)
            .await?;
        
        // 3. Return ActorAddress
        Ok(addr)
    }
}

pub struct ComponentRegistry {
    instances: Arc<RwLock<HashMap<ComponentId, ActorAddress>>>,
}

impl ComponentRegistry {
    pub fn register(&self, component_id: ComponentId, addr: ActorAddress) -> Result<()>
    pub fn lookup(&self, component_id: &ComponentId) -> Result<ActorAddress>  // O(1)
    pub fn unregister(&self, component_id: &ComponentId) -> Result<()>
}
```

**Responsibilities:**
- ✅ Define what a ComponentActor is
- ✅ Load WASM binaries via Wasmtime
- ✅ Orchestrate component spawning
- ✅ Implement Actor trait for message handling
- ✅ Implement Child trait for lifecycle
- ✅ Track component instances by ID

**What Layer 2 Does NOT Do:**
- ❌ Create mailbox (Layer 3 provides via spawn())
- ❌ Route messages (Layer 3 MessageBroker does)
- ❌ Make restart decisions (Layer 3 SupervisorNode does)
- ❌ Implement supervision tree (Layer 3 does)
- ❌ Create actor system (Layer 3 ActorSystem does)

**Key Integration Pattern:**
```rust
// Layer 2 uses Layer 3's services:
self.actor_system              // ← From Layer 3
    .spawn()                   // ← Layer 3 ActorSystem
    .spawn(component_actor)    // ← Creates mailbox automatically
    .await?
```

**Dependencies:**
```
Layer 2 (Lifecycle & Spawning)
    ├─ Uses → Layer 3 (ActorSystem, MessageBroker, ActorAddress)
    ├─ Provides config to → Layer 3 (via SupervisorNode integration - Phase 3)
    └─ Implements → ComponentActor (Actor + Child traits)
```

### 2.3 Layer 3: Actor System Runtime (airssys-rt)

**Purpose:** Provide low-level actor system infrastructure

**Location:** `airssys-rt/src/system/`, `actor/`, `supervisor/`, `mailbox/`, `broker/`

**Core Structures:**
```rust
pub struct ActorSystem<M, B: MessageBroker<M>> {
    // Actor spawning and lifecycle
}

pub struct SupervisorNode<S: SupervisionStrategy, C: Child> {
    backoff: RestartBackoff,          // Exponential backoff only
    strategy: S,                       // OneForOne/OneForAll/RestForOne
}

pub enum RestartPolicy {
    Permanent,    // Always restart
    Transient,    // Restart on error only
    Temporary,    // Never restart
}

pub struct RestartBackoff {
    max_restarts: u32,
    restart_window: Duration,
    restart_history: VecDeque<DateTime<Utc>>,
    base_delay: Duration,
    max_delay: Duration,
}

pub struct Mailbox<M> {
    // Message queue (bounded or unbounded)
    // Backpressure handling
    // Async message delivery
}

pub struct MessageBroker<M> {
    // Routes messages to actor mailboxes
}

pub struct ActorAddress {
    // Identifies and routes to an actor
    // Cross-thread safe
}
```

**Responsibilities:**
- ✅ Create and manage actors
- ✅ Assign mailboxes to actors
- ✅ Route messages via MessageBroker
- ✅ Make restart decisions
- ✅ Implement supervision strategies
- ✅ Calculate exponential backoff delays
- ✅ Provide ActorAddress for routing

**What Layer 3 Provides to Layer 2:**
```
ActorSystem::spawn()          ← Used by ComponentSpawner
MessageBroker<ComponentMessage> ← Delivers ComponentMessage
ActorAddress                  ← Returned to ComponentRegistry
SupervisorNode                ← Will integrate in Phase 3
RestartBackoff               ← Will use Layer 1 config in Phase 3
```

**Critical Note:** Layer 3 is **independent of WASM**. It's a general-purpose actor system that could be used by other frameworks too.

---

## Part 3: Ownership & Responsibility Matrix

### 3.1 Feature Ownership

| Feature | Layer 1 | Layer 2 | Layer 3 | Rationale |
|---------|---------|---------|---------|-----------|
| **Component Definition** | - | ✅ | - | ComponentActor is the implementation |
| **Restart Policy Types** | ✅ | - | ✅* | Layer 1 for WASM config, Layer 3 defines semantics |
| **Backoff Strategy Types** | ✅ (extra variants) | - | ✅ (exponential only) | Layer 1 provides more options |
| **Backoff Calculation** | - | - | ✅ | RestartBackoff in Layer 3 |
| **WASM Binary Loading** | - | ✅ | - | WasmRuntime in ComponentActor |
| **Actor Creation** | - | ✅ | ✅* | Layer 2 creates ComponentActor, Layer 3 spawns it |
| **Mailbox Creation** | - | - | ✅ | ActorSystem::spawn() creates mailbox |
| **Message Routing** | - | - | ✅ | MessageBroker |
| **Supervision Tree** | - | - | ✅ | SupervisorNode |
| **Restart Decisions** | - | - | ✅ | SupervisorNode based on RestartPolicy |
| **Component Registry** | - | ✅ | - | ComponentRegistry tracks instances |
| **Component Addressing** | - | ✅ (uses) | ✅ (provides) | Layer 2 uses Layer 3's ActorAddress |

**Legend:** `✅` = Owns, `✅*` = Shared ownership, `-` = Not applicable

### 3.2 Dependency Flow

```
Application Layer
    ↓
Layer 2 (ComponentSpawner, ComponentRegistry)
    ├─ Depends on → Layer 1 (SupervisorConfig for future integration)
    └─ Depends on → Layer 3 (ActorSystem, MessageBroker)

Layer 1 (SupervisorConfig, ComponentSupervisor)
    └─ Depends on → (nothing, standalone)

Layer 3 (Actor System)
    └─ Depends on → (nothing, standalone)
```

### 3.3 Data Flow at Runtime

```
spawn_component(config: SupervisorConfig)
    ↓
ComponentSpawner::spawn_component()
    ├─ (Layer 2)
    └─ Creates ComponentActor
        ├─ (Layer 2)
        └─ Calls ActorSystem::spawn()
            ├─ (Layer 3)
            ├─ Creates mailbox
            ├─ Creates ActorAddress
            └─ Returns ActorAddress
        └─ ComponentRegistry::register(id, address)
            ├─ (Layer 2)
            └─ HashMap insert
        └─ Application receives ActorAddress

Application sends ComponentMessage via ActorAddress
    ↓ (Layer 3: MessageBroker)
    ↓
Mailbox receives message
    ↓ (Layer 3: Mailbox)
    ↓
Actor::handle_message() invoked
    ├─ (Layer 2: ComponentActor)
    ├─ Calls WasmRuntime
    ├─ (Layer 2: calls exported WASM function)
    └─ Returns result or error

If error and should restart:
    ↓ (Layer 3: SupervisorNode)
    ├─ Checks RestartPolicy
    ├─ (Layer 1 config + Layer 3 decision)
    ├─ Calculates backoff delay
    ├─ (Layer 3: RestartBackoff)
    └─ Calls Child::start() again
        ├─ (Layer 2: ComponentActor)
        └─ Reloads WASM binary
            ├─ (Layer 2: WasmRuntime)
            └─ Back to mailbox routing
```

---

## Part 4: Development Guidelines

### 4.1 "Where Should I Put This Feature?"

Use this decision tree:

```
┌─ Is it WASM-specific configuration or policy?
│  └─ YES → Layer 1 (supervisor_config.rs, component_supervisor.rs)
│
├─ Is it WASM binary loading or component lifecycle?
│  └─ YES → Layer 2 (component_actor.rs, runtime/*.rs)
│
├─ Is it component creation/spawning/registration?
│  └─ YES → Layer 2 (component_spawner.rs, component_registry.rs)
│
├─ Is it message routing, mailbox, or actor addressing?
│  └─ YES → Layer 3 (use airssys-rt, don't add to airssys-wasm)
│
├─ Is it restart decision making or supervision strategy?
│  └─ YES → Layer 3 (use airssys-rt SupervisorNode)
│
└─ Is it general actor system infrastructure?
   └─ YES → Layer 3 (contribute to airssys-rt, not airssys-wasm)
```

### 4.2 Checklist for New Features

Before implementing a new feature:

- [ ] Which layer does this belong in?
- [ ] Am I duplicating code from another layer?
- [ ] Am I creating a new dependency on another layer?
- [ ] Is this WASM-specific or general-purpose?
- [ ] Do I need to modify Layer 3 (airssys-rt)?
- [ ] Have I reviewed the ownership matrix?
- [ ] Are all my imports following the three-layer pattern?

### 4.3 Anti-Patterns to Avoid

#### ❌ Anti-Pattern 1: "Let's create our own mailbox"
```rust
// WRONG - Don't do this!
pub struct ComponentActor {
    mailbox: Vec<ComponentMessage>,  // ← Creating our own mailbox
}
```
✅ **Correct:** Use `ActorSystem::spawn()` which provides mailbox automatically

#### ❌ Anti-Pattern 2: "Let's implement our own RestartBackoff"
```rust
// WRONG - Don't do this!
pub struct WasmBackoff {
    calculate_delay() { ... }  // ← Duplicating Layer 3
}
```
✅ **Correct:** Layer 1 defines config (SupervisorConfig), Layer 3 implements (RestartBackoff)

#### ❌ Anti-Pattern 3: "Let's create a new actor system for WASM"
```rust
// WRONG - Don't do this!
pub struct WasmActorSystem {
    spawn(&self, component) { ... }  // ← Duplicating Layer 3
}
```
✅ **Correct:** Use `ActorSystem::spawn()` from airssys-rt

#### ❌ Anti-Pattern 4: "Let's implement message broker routing"
```rust
// WRONG - Don't do this!
pub struct ComponentMessageRouter {
    route(&self, message) { ... }  // ← Layer 3 responsibility
}
```
✅ **Correct:** Use `MessageBroker` from airssys-rt

### 4.4 Integration Checklist for Phase 3+

When integrating Layer 1 and Layer 2 with Layer 3:

- [ ] SupervisorConfig can be passed to SupervisorNode builder
- [ ] RestartPolicy values match airssys-rt's definitions
- [ ] BackoffStrategy is converted to RestartBackoff for Layer 3
- [ ] ComponentSupervisor can track restarts alongside SupervisorNode
- [ ] No duplicate restart decision logic
- [ ] ActorAddress continues to work for component routing
- [ ] Tests verify Layer 3 makes final restart decisions

---

## Part 5: Common Questions & Answers

### Q1: "Can I use airssys-wasm's RestartBackoff?"
**A:** No. The RestartBackoff in Layer 1 (`BackoffStrategy`) is configuration. The actual exponential backoff calculation happens in Layer 3 (`airssys_rt::supervisor::RestartBackoff`). In Phase 3 integration, we'll convert Layer 1 config to Layer 3 implementation.

### Q2: "Should ComponentActor import from airssys-rt?"
**A:** YES! ComponentActor should:
- ✅ Implement `airssys_rt::actor::Actor` trait
- ✅ Implement `airssys_rt::supervisor::Child` trait
- ✅ Import `ActorAddress` from airssys-rt
- This is correct Layer 2 → Layer 3 dependency

### Q3: "Can I access the mailbox from ComponentActor?"
**A:** No. The mailbox is created and managed by Layer 3's ActorSystem. ComponentActor only sees messages that arrive via `Actor::handle_message()`. If you need mailbox metrics, ask Layer 3 to provide them.

### Q4: "Should ComponentSupervisor make restart decisions?"
**A:** No. ComponentSupervisor should:
- ✅ Track component state and restart history
- ✅ Provide configuration that influences decisions
- ❌ NOT make decisions (SupervisorNode does that)

### Q5: "What happens when Component A wants to send a message to Component B?"
**A:**
1. Get ActorAddress for Component B from ComponentRegistry (Layer 2)
2. Create ComponentMessage (Layer 2)
3. Send via MessageBroker (Layer 3)
4. ComponentB's mailbox receives message (Layer 3)
5. Actor::handle_message() invoked (Layer 2)
6. Calls WASM export (Layer 2)

### Q6: "Can I customize SupervisorNode strategy for WASM?"
**A:** Yes, in Phase 3:
1. Use `SupervisorNode<OneForOne>` for independent components
2. Use `SupervisorNode<RestForOne>` for dependent startup sequences
3. SupervisorConfig from Layer 1 can influence strategy choice
4. But strategy implementation stays in Layer 3

### Q7: "Where does capability checking happen?"
**A:** Currently in PermissionChecker (Layer 2). Future: Layer 4 (airssys-osl bridge) will handle security policy enforcement for components.

---

## Part 6: Phase Evolution

### Current Phase (2025-12-14): Phase 2 Complete
- ✅ Layer 1 defined (SupervisorConfig, BackoffStrategy)
- ✅ Layer 2 implemented (ComponentActor, ComponentSpawner, ComponentRegistry)
- ✅ Layer 3 used as-is (airssys-rt)
- ✅ Layers communicate one-way (Layer 2 → Layer 3)

### Phase 3 (Upcoming): Supervision Integration
- Integrate SupervisorConfig with SupervisorNode
- Convert BackoffStrategy to RestartBackoff
- Wire restart decisions through supervisor tree
- ComponentActor remains unmodified (Child::start/stop called by supervisor)

### Phase 4+: Security & Monitoring
- Layer 4: airssys-osl integration
- Layer 5: Observability (metrics, tracing)
- Maintain three-layer boundaries throughout

---

## Appendix: Quick Reference

### Layer 1 Files
```
airssys-wasm/src/actor/supervisor_config.rs    (Policies & strategies)
airssys-wasm/src/actor/component_supervisor.rs (State tracking)
```

### Layer 2 Files
```
airssys-wasm/src/actor/component_actor.rs      (ComponentActor impl)
airssys-wasm/src/actor/component_spawner.rs    (Spawning orchestration)
airssys-wasm/src/actor/component_registry.rs   (Instance tracking)
airssys-wasm/src/actor/actor_impl.rs           (Actor trait impl)
airssys-wasm/src/actor/child_impl.rs           (Child trait impl)
airssys-wasm/src/runtime/engine.rs             (WasmRuntime)
airssys-wasm/src/runtime/loader.rs             (WASM loading)
```

### Layer 3 (Reference)
```
airssys-rt/src/system/actor_system.rs          (ActorSystem)
airssys-rt/src/supervisor/node.rs              (SupervisorNode)
airssys-rt/src/supervisor/backoff.rs           (RestartBackoff)
airssys-rt/src/broker/                         (MessageBroker)
airssys-rt/src/mailbox/                        (Mailbox impl)
```

### Key Types by Layer

**Layer 1:**
- RestartPolicy, BackoffStrategy, SupervisorConfig
- ComponentSupervisor, SupervisionHandle, SupervisionState

**Layer 2:**
- ComponentActor, ComponentSpawner, ComponentRegistry
- WasmRuntime, ComponentMessage

**Layer 3:**
- ActorSystem, SupervisorNode, RestartBackoff
- MessageBroker, Mailbox, ActorAddress

---

## Document History

| Date | Version | Changes |
|------|---------|---------|
| 2025-12-14 | 1.0 | Initial creation (KNOWLEDGE-WASM-018) |

---

**Document Status:** ✅ APPROVED & CURRENT  
**Last Updated:** 2025-12-14  
**Maintainer:** Architecture Team
