
## Implementation Plan - Phase 5: Refactor ActorSystemSubscriber

### Context & References

**ADR References:**

- **ADR-WASM-023: Module Boundary Enforcement** (Lines 54-80)
  - **Quote:** "FORBIDDEN (NEVER, NO EXCEPTIONS): ❌ runtime/ → actor/ (BREAKS ARCHITECTURE)"
  - **Quote:** "The Dependency Rules (MANDATORY - NO EXCEPTIONS): actor/ ─────► runtime/"
  - **Application:** Phase 5 ensures ActorSystemSubscriber (actor/) does not create circular dependencies with host_system/ by using dependency injection instead of direct ownership of ComponentRegistry.

- **ADR-WASM-020: Message Delivery Ownership**
  - **Quote:** "ActorSystemSubscriber owns message delivery via `mailbox_senders` map"
  - **Quote:** "ComponentRegistry stays pure (identity lookup only)"
  - **Application:** Phase 5 preserves this separation - ActorSystemSubscriber keeps mailbox_senders but ComponentRegistry ownership moves to host_system/.

**Knowledge References:**

- **KNOWLEDGE-WASM-036: Four-Module Architecture** (Lines 159-205)
  - **Quote (Lines 161-172):** "**Purpose:** Coordinate all system operations - initialization, lifecycle, and message flow. **Owns:** RuntimeManager - Central coordinator for all operations, Component lifecycle management, Message flow coordination"
  - **Quote (Lines 183-195):** "**Purpose:** Wrap WASM components in airssys-rt actor system. **Owns:** ComponentActor, ComponentRegistry, ComponentSpawner, ComponentSupervisor, MessageRouter, ActorSystemSubscriber"
  - **Quote (Lines 518-540):** "✅ **Correct:** Host system logic belongs in host_system/... pub struct ResponseRouter { tracker: Arc<RwLock<CorrelationTracker>>,  // Passed in }"
  - **Application:** Phase 5 applies dependency injection pattern from KNOWLEDGE-WASM-036, ensuring host_system/ owns ComponentRegistry and passes it to ActorSystemSubscriber via constructor injection.

- **KNOWLEDGE-WASM-026: Message Delivery Architecture** (Lines 186-238)
  - **Quote (Lines 188-205):** "pub struct ActorSystemSubscriber<B: MessageBroker<ComponentMessage>> { broker: Arc<B>, registry: ComponentRegistry, subscriber_manager: Arc<SubscriberManager>, routing_task: Option<JoinHandle<()>>, mailbox_senders: Arc<RwLock<HashMap<ComponentId, MailboxSender<ComponentMessage>>>>, }"
  - **Quote (Lines 208-237):** "Register a component's mailbox sender for message delivery. Called by ComponentSpawner when ComponentActor is spawned."
  - **Application:** Phase 5 removes ComponentRegistry ownership from ActorSystemSubscriber but keeps mailbox_senders for actual message delivery per KNOWLEDGE-WASM-026.

**System Patterns:**
- **Dependency Injection Pattern (KNOWLEDGE-WASM-036):** Pass dependencies via constructor instead of owning them directly. This eliminates circular dependencies.
- **Central Coordinator Pattern (KNOWLEDGE-WASM-036):** host_system/ owns and coordinates all infrastructure, actor/ uses injected dependencies.

**PROJECTS_STANDARD.md Compliance:**
- **§2.1 (3-Layer Import Organization):** Code will follow std → external → internal import pattern in all modified files.
- **§3.2 (DateTime<Utc> Standard):** Not applicable (no time operations in Phase 5).
- **§4.3 (Module Architecture):** mod.rs files will only contain declarations (no changes needed).
- **§6.1 (YAGNI Principles):** Only dependency injection pattern implemented - no speculative features added.
- **§6.2 (Avoid `dyn` Patterns):** Will use generics with `Arc` references, no trait objects.
- **§6.4 (Implementation Quality Gates):** Zero warnings, comprehensive unit + integration tests.

**Rust Guidelines Applied:**
- **M-DESIGN-FOR-AI:** Idiomatic API with clear ownership semantics via dependency injection.
- **M-MODULE-DOCS:** Module documentation will be updated to reflect new ownership model.
- **M-ERRORS-CANONICAL-STRUCTS:** Error types follow canonical structure (WasmError).
- **M-STATIC-VERIFICATION:** All lints enabled, clippy will pass with `-D warnings`.
- **M-FEATURES-ADDITIVE:** Changes will not break existing ComponentRegistry API.

**Documentation Standards:**
- **Diátaxis Type:** Reference documentation for refactored ActorSystemSubscriber API.
- **Quality:** Technical language, no hyperbole per documentation-quality-standards.md.
- **Canonical Sections:** Summary, Examples, Errors, Panics sections included in all public API docs.

### Module Architecture

**Current ActorSystemSubscriber Location:**
- **File path:** `airssys-wasm/src/actor/message/actor_system_subscriber.rs`
- **Current imports (Lines 84-99):**
  ```rust
  // Layer 1: Standard library imports
  use std::collections::HashMap;
  use std::sync::Arc;

  // Layer 2: Third-party crate imports
  use tokio::sync::mpsc::UnboundedSender;
  use tokio::sync::{Mutex, RwLock};
  use tokio::task::JoinHandle;

  // Layer 3: Internal module imports
  use crate::actor::component::ComponentRegistry;  // ← REMOVE THIS
  use crate::actor::message::SubscriberManager;
  use crate::core::ComponentMessage;
  use crate::core::{ComponentId, WasmError};
  use airssys_rt::broker::MessageBroker;
  use airssys_rt::message::MessageEnvelope;
  ```

- **Current dependencies (Lines 168-188):**
  ```rust
  pub struct ActorSystemSubscriber<B: MessageBroker<ComponentMessage>> {
      broker: Arc<B>,
      registry: ComponentRegistry,  // ← REMOVE THIS FIELD
      subscriber_manager: Arc<SubscriberManager>,
      routing_task: Option<JoinHandle<()>>,
      mailbox_senders: Arc<RwLock<HashMap<ComponentId, UnboundedSender<ComponentMessage>>>>,
  }
  ```

**After Phase 5 Changes:**

- **Target structure:**
  - ActorSystemSubscriber no longer owns ComponentRegistry
  - ComponentRegistry is passed via constructor injection (Arc<ComponentRegistry>)
  - All references to registry are via borrowed Arc reference

- **Dependency flow diagram:**
  ```text
  BEFORE (CIRCULAR RISK):
  ActorSystemSubscriber (actor/) 
      ├── owns ComponentRegistry (actor/)
      └── creates circular dependency risk

  AFTER (CLEAN ONE-WAY):
  HostSystemManager (host_system/)
      ├── owns ComponentRegistry (actor/)  ← ownership
      ├── owns ActorSystemSubscriber (actor/)
      └── passes Arc<ComponentRegistry> to ActorSystemSubscriber via constructor
  ```

- **Allowed/forbidden imports:**
  - **ALLOWED:** `use crate::actor::component::ComponentRegistry;` in host_system/manager.rs (ownership)
  - **FORBIDDEN:** `use crate::host_system::...` in actor/message/actor_system_subscriber.rs (actor/ cannot import from host_system/ per ADR-WASM-023)
  - **FORBIDDEN:** Direct ownership of ComponentRegistry in ActorSystemSubscriber (creates ownership confusion)

**Verification command (for implementer to run):**
```bash
# After Phase 5, verify ActorSystemSubscriber does NOT own ComponentRegistry
grep -n "registry: ComponentRegistry" airssys-wasm/src/actor/message/actor_system_subscriber.rs
# Expected: No output (field removed)

# Verify host_system/ owns ComponentRegistry
grep -n "registry: Arc<ComponentRegistry>" airssys-wasm/src/host_system/manager.rs
# Expected: Line found (ownership moved to host_system)

# Verify no forbidden imports (ADR-WASM-023 compliance)
grep -rn "use crate::host_system" airssys-wasm/src/actor/
# Expected: No output (clean)
```

### Phase 5 Subtasks

#### Subtask 5.1: Refactor ActorSystemSubscriber Struct Definition

**Deliverables:**
- **Exact file:** `airssys-wasm/src/actor/message/actor_system_subscriber.rs`
- **Exact line range:** 168-188
- **Exact code changes:**

**BEFORE:**
```rust
pub struct ActorSystemSubscriber<B: MessageBroker<ComponentMessage>> {
    /// MessageBroker for receiving messages
    broker: Arc<B>,
    /// ComponentRegistry for looking up component addresses (IDENTITY ONLY per ADR-WASM-020)
    #[allow(dead_code)] // Registry kept for future topic-based routing lookup
    registry: ComponentRegistry,
    /// SubscriberManager for topic-based routing decisions
    subscriber_manager: Arc<SubscriberManager>,
    /// Background routing task handle
    routing_task: Option<JoinHandle<()>>,
    /// Map of ComponentId → MailboxSender for actual message delivery (ADR-WASM-020)
    ///
    /// This field owns the delivery mechanism. When a message arrives:
    /// 1. Extract target ComponentId from message
    /// 2. Look up MailboxSender in this map
    /// 3. Call sender.send(message) for actual delivery
    ///
    /// Registration happens when ComponentSpawner spawns a component.
    /// Unregistration happens when component is stopped.
    mailbox_senders: Arc<RwLock<HashMap<ComponentId, UnboundedSender<ComponentMessage>>>>,
}
```

**AFTER:**
```rust
pub struct ActorSystemSubscriber<B: MessageBroker<ComponentMessage>> {
    /// MessageBroker for receiving messages
    broker: Arc<B>,
    /// SubscriberManager for topic-based routing decisions
    subscriber_manager: Arc<SubscriberManager>,
    /// Background routing task handle
    routing_task: Option<JoinHandle<()>>,
    /// Map of ComponentId → MailboxSender for actual message delivery (ADR-WASM-020)
    ///
    /// This field owns the delivery mechanism. When a message arrives:
    /// 1. Extract target ComponentId from message
    /// 2. Look up MailboxSender in this map
    /// 3. Call sender.send(message) for actual delivery
    ///
    /// Registration happens when ComponentSpawner spawns a component.
    /// Unregistration happens when component is stopped.
    mailbox_senders: Arc<RwLock<HashMap<ComponentId, UnboundedSender<ComponentMessage>>>>,
}
```

**Acceptance Criteria:**
1. ComponentRegistry field is completely removed from ActorSystemSubscriber struct
2. `#[allow(dead_code)]` attribute is removed (no longer needed)
3. All documentation remains accurate (no references to removed registry field)
4. Struct compiles without errors
5. Zero clippy warnings

**ADR Constraints:**
- **ADR-WASM-023:** ActorSystemSubscriber in actor/ cannot own dependencies that create circular dependencies with host_system/
- **KNOWLEDGE-WASM-036:** Dependency injection pattern - receive ComponentRegistry via constructor, don't own it (Lines 518-540)

**PROJECTS_STANDARD.md Compliance:**
- **§6.1:** YAGNI - Removed unused field, no speculative features added
- **§6.2:** Avoid `dyn` - Used concrete types (Arc<T>), no trait objects

**Rust Guidelines:**
- **M-DESIGN-FOR-AI:** Clear ownership semantics - ActorSystemSubscriber no longer owns registry

**Implementation Details:**
This is a straightforward struct field removal. The key architectural decision is:
- ComponentRegistry ownership moves to host_system/manager.rs
- ActorSystemSubscriber receives Arc<ComponentRegistry> via constructor injection (Subtask 5.2)
- This follows KNOWLEDGE-WASM-036 dependency injection pattern

**Unit Tests:**
- **Exact location:** `airssys-wasm/src/actor/message/actor_system_subscriber.rs:570-600` (existing tests need updates)
- **Exact test count:** 3 tests (modified)
- **Test names:**
  1. test_actor_system_subscriber_creation (Lines 570-579) - Update to pass Arc<ComponentRegistry> to new()
  2. test_actor_system_subscriber_start (Lines 582-595) - Update constructor call
  3. test_actor_system_subscriber_stop (Lines 598-611) - Update constructor call

#### Subtask 5.2: Refactor ActorSystemSubscriber::new() Constructor

**Deliverables:**
- **Exact file:** `airssys-wasm/src/actor/message/actor_system_subscriber.rs`
- **Exact line range:** 190-220
- **Exact code changes:**

**BEFORE:**
```rust
pub fn new(
    broker: Arc<B>,
    registry: ComponentRegistry,
    subscriber_manager: Arc<SubscriberManager>,
) -> Self {
    Self {
        broker,
        registry,
        subscriber_manager,
        routing_task: None,
        mailbox_senders: Arc::new(RwLock::new(HashMap::new())),
    }
}
```

**AFTER:**
```rust
pub fn new(
    broker: Arc<B>,
    subscriber_manager: Arc<SubscriberManager>,
) -> Self {
    Self {
        broker,
        subscriber_manager,
        routing_task: None,
        mailbox_senders: Arc::new(RwLock::new(HashMap::new())),
    }
}
```

**Acceptance Criteria:**
1. ComponentRegistry parameter is removed from new() signature
2. No field initialization for registry in constructor body
3. All documentation updated (removed registry parameter description)
4. Constructor compiles without errors
5. Zero clippy warnings

**ADR Constraints:**
- **KNOWLEDGE-WASM-036:** Dependency injection - ComponentRegistry passed from host_system/ as Arc reference, not owned by ActorSystemSubscriber (Lines 518-540)
- **ADR-WASM-023:** Eliminates potential circular dependency by removing direct ownership

**PROJECTS_STANDARD.md Compliance:**
- **§6.2:** Avoid `dyn` - Generic type parameter B retained, no trait objects introduced
- **§6.1:** YAGNI - Removed unused parameter, minimal change

**Rust Guidelines:**
- **M-DESIGN-FOR-AI:** API simplification - fewer parameters, clearer ownership model

**Implementation Details:**
The constructor becomes simpler with fewer parameters. ComponentRegistry ownership is now:
- Owned by HostSystemManager in host_system/manager.rs
- Passed to ActorSystemSubscriber only when needed (if needed at all)
- Current code shows registry field was marked `#[allow(dead_code)]`, meaning it's not actually used yet
- Phase 5 focuses on architectural correctness - future subtasks may pass registry if needed

**Unit Tests:**
- **Exact location:** `airssys-wasm/src/actor/message/actor_system_subscriber.rs:570-600`
- **Exact test count:** 3 tests (modified)
- **Test names:**
  1. test_actor_system_subscriber_creation (Lines 570-579)
     ```rust
     #[tokio::test]
     async fn test_actor_system_subscriber_creation() {
         let broker = Arc::new(InMemoryMessageBroker::new());
         // registry parameter REMOVED
         let subscriber_manager = Arc::new(SubscriberManager::new());

         let subscriber = ActorSystemSubscriber::new(broker, subscriber_manager);

         assert!(!subscriber.is_running());
         assert_eq!(subscriber.mailbox_count().await, 0);
     }
     ```
  2. test_actor_system_subscriber_start (Lines 582-595) - Same pattern
  3. test_actor_system_subscriber_stop (Lines 598-611) - Same pattern

#### Subtask 5.3: Update HostSystemManager to Own ComponentRegistry

**Deliverables:**
- **Exact file:** `airssys-wasm/src/host_system/manager.rs`
- **Exact line range:** 112-133
- **Exact code changes:**

**BEFORE:**
```rust
pub struct HostSystemManager {
    /// WASM execution engine for executing component code
    engine: Arc<WasmEngine>,

    /// Component registry for O(1) ComponentId → ActorAddress lookups
    registry: Arc<ComponentRegistry>,

    /// Component spawner for creating ComponentActor instances
    spawner: Arc<ComponentSpawner<InMemoryMessageBroker<ComponentMessage>>>,

    /// Messaging service with MessageBroker for inter-component communication
    messaging_service: Arc<MessagingService>,

    /// Correlation tracker for request-response pattern
    correlation_tracker: Arc<CorrelationTracker>,

    /// Timeout handler for request timeout enforcement
    timeout_handler: Arc<TimeoutHandler>,

    /// System startup flag - true after initialization complete
    started: Arc<AtomicBool>,
}
```

**AFTER:**
```rust
pub struct HostSystemManager {
    /// WASM execution engine for executing component code
    engine: Arc<WasmEngine>,

    /// Component registry for O(1) ComponentId → ActorAddress lookups
    /// Ownership moved to host_system/ per KNOWLEDGE-WASM-036 dependency injection pattern
    registry: Arc<ComponentRegistry>,

    /// Component spawner for creating ComponentActor instances
    spawner: Arc<ComponentSpawner<InMemoryMessageBroker<ComponentMessage>>>,

    /// Messaging service with MessageBroker for inter-component communication
    messaging_service: Arc<MessagingService>,

    /// ActorSystemSubscriber for message routing (owned by host_system/)
    actor_system_subscriber: Arc<RwLock<ActorSystemSubscriber<InMemoryMessageBroker<ComponentMessage>>>>,

    /// Correlation tracker for request-response pattern
    correlation_tracker: Arc<CorrelationTracker>,

    /// Timeout handler for request timeout enforcement
    timeout_handler: Arc<TimeoutHandler>,

    /// System startup flag - true after initialization complete
    started: Arc<AtomicBool>,
}
```

**Acceptance Criteria:**
1. ActorSystemSubscriber field added to HostSystemManager struct
2. Field is wrapped in Arc<RwLock<>> for thread-safe sharing
3. Documentation added for ActorSystemSubscriber field
4. Struct compiles without errors
5. Zero clippy warnings

**ADR Constraints:**
- **KNOWLEDGE-WASM-036:** host_system/ owns and coordinates all infrastructure, including ActorSystemSubscriber (Lines 161-172)
- **ADR-WASM-023:** host_system/ can own actor/ types (allowed dependency direction: host_system/ → actor/)

**PROJECTS_STANDARD.md Compliance:**
- **§6.2:** Avoid `dyn` - Uses concrete type ActorSystemSubscriber<InMemoryMessageBroker<ComponentMessage>>
- **§2.1:** 3-layer imports preserved (std → external → internal)

**Rust Guidelines:**
- **M-DESIGN-FOR-AI:** Clear ownership - host_system/ owns ActorSystemSubscriber, actor/ uses it via Arc

**Implementation Details:**
HostSystemManager now owns ActorSystemSubscriber, establishing clear ownership:
- ActorSystemSubscriber is created in HostSystemManager::new()
- HostSystemManager passes dependencies (broker, subscriber_manager) via constructor
- This follows KNOWLEDGE-WASM-036 dependency injection pattern

**Unit Tests:**
- **Exact location:** `airssys-wasm/src/host_system/manager.rs:604-750` (new tests added)
- **Exact test count:** 3 tests
- **Test names:**
  1. test_host_system_manager_owns_actor_system_subscriber
     - Verify HostSystemManager has actor_system_subscriber field
     - Verify field is Arc<RwLock<ActorSystemSubscriber>>
     - Test: `assert!(manager.actor_system_subscriber.read().await.is_running());`
  2. test_host_system_manager_starts_subscriber_during_init
     - Verify subscriber.start() called during HostSystemManager::new()
     - Verify subscriber.is_running() after initialization
  3. test_host_system_manager_stops_subscriber_during_shutdown
     - Verify subscriber.stop() called during HostSystemManager::shutdown()
     - Verify !subscriber.is_running() after shutdown

[Plan continues in next section...]

#### Subtask 5.4: Implement HostSystemManager::new() with ActorSystemSubscriber Creation

**Deliverables:**
- **Exact file:** `airssys-wasm/src/host_system/manager.rs`
- **Exact line range:** 150-300
- **Exact code changes:**

**NEW METHOD:**
```rust
impl HostSystemManager {
    /// Create new HostSystemManager with all infrastructure initialized.
    ///
    /// This method initializes all infrastructure in correct order per KNOWLEDGE-WASM-036:
    /// 1. Create WasmEngine for WASM execution
    /// 2. Create ComponentRegistry (owned by host_system/)
    /// 3. Create MessageBroker and SubscriberManager
    /// 4. Create ActorSystemSubscriber with dependency injection
    /// 5. Create ComponentSpawner and other infrastructure
    ///
    /// # Returns
    ///
    /// - `Ok(HostSystemManager)` - Successfully initialized
    /// - `Err(WasmError)` - Initialization failed
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let manager = HostSystemManager::new().await?;
    /// ```
    pub async fn new() -> Result<Self, WasmError> {
        // Step 1: Create WASM execution engine
        let engine = Arc::new(WasmEngine::new()?);

        // Step 2: Create ComponentRegistry (owned by host_system/ per KNOWLEDGE-WASM-036)
        let registry = Arc::new(ComponentRegistry::new());

        // Step 3: Create messaging infrastructure
        let broker = Arc::new(InMemoryMessageBroker::new());
        let subscriber_manager = Arc::new(SubscriberManager::new());

        // Step 4: Create ActorSystemSubscriber with dependency injection
        // HostSystemManager owns subscriber, passes broker and subscriber_manager
        let mut actor_system_subscriber = ActorSystemSubscriber::new(
            Arc::clone(&broker),
            Arc::clone(&subscriber_manager),
        );

        // Step 5: Start the subscriber (routes messages to components)
        actor_system_subscriber.start().await?;

        // Wrap subscriber in Arc<RwLock<>> for sharing
        let actor_system_subscriber = Arc::new(RwLock::new(actor_system_subscriber));

        // Step 6: Create component spawner
        let spawner = Arc::new(ComponentSpawner::new(
            registry.clone(),
            broker.clone(),
        ));

        // Step 7: Create correlation tracking infrastructure
        let correlation_tracker = Arc::new(CorrelationTracker::new());
        let timeout_handler = Arc::new(TimeoutHandler::new());

        // Step 8: Create messaging service
        let messaging_service = Arc::new(MessagingService::new(
            Arc::clone(&broker),
            Arc::clone(&correlation_tracker),
            Arc::clone(&timeout_handler),
        ));

        Ok(Self {
            engine,
            registry,
            spawner,
            messaging_service,
            actor_system_subscriber,
            correlation_tracker,
            timeout_handler,
            started: Arc::new(AtomicBool::new(true)),
        })
    }
}
```

**Acceptance Criteria:**
1. ActorSystemSubscriber is created in HostSystemManager::new()
2. Subscriber receives broker and subscriber_manager via dependency injection
3. Subscriber.start() is called during initialization
4. Subscriber is wrapped in Arc<RwLock<>> for thread-safe sharing
5. All infrastructure created in correct order per KNOWLEDGE-WASM-036
6. System started flag set to true after successful initialization
7. Method compiles without errors
8. Zero clippy warnings

**ADR Constraints:**
- **KNOWLEDGE-WASM-036:** Initialization order: engine → registry → broker → subscriber → spawner (Lines 414-452)
- **KNOWLEDGE-WASM-036:** Dependency injection - ActorSystemSubscriber receives dependencies via constructor (Lines 518-540)
- **ADR-WASM-023:** host_system/ → actor/ dependency is allowed (ownership flows correct direction)

**PROJECTS_STANDARD.md Compliance:**
- **§2.1:** 3-layer imports preserved in all code
- **§6.1:** YAGNI - Only create infrastructure that's actually used
- **§6.4:** Quality gates - Zero warnings, comprehensive tests

**Rust Guidelines:**
- **M-DESIGN-FOR-AI:** Clear initialization order with step-by-step comments
- **M-MODULE-DOCS:** Comprehensive documentation with summary, examples, errors sections

**Implementation Details:**
Key architectural decisions in this method:
1. **Ownership:** HostSystemManager owns ActorSystemSubscriber (not vice versa)
2. **Dependency Injection:** Subscriber receives broker and subscriber_manager via constructor
3. **Initialization Order:** Follows KNOWLEDGE-WASM-036 order for correct startup
4. **Thread Safety:** Subscriber wrapped in Arc<RwLock<>> for concurrent access

**Unit Tests:**
- **Exact location:** `airssys-wasm/src/host_system/manager.rs:751-900` (new tests added)
- **Exact test count:** 5 tests
- **Test names:**
  1. test_host_system_manager_new_success
  2. test_host_system_manager_new_creates_subscriber
  3. test_host_system_manager_new_starts_subscriber
  4. test_host_system_manager_new_initialization_order
  5. test_host_system_manager_new_error_handling

#### Subtask 5.5: Implement HostSystemManager::shutdown() with Subscriber Cleanup

**Deliverables:**
- **Exact file:** `airssys-wasm/src/host_system/manager.rs`
- **Exact line range:** 300-400 (approximate, will be added after existing methods)
- **Exact code changes:**

**NEW METHOD:**
```rust
/// Shutdown host system and cleanup all resources.
///
/// This method performs graceful shutdown in reverse initialization order:
/// 1. Stop ActorSystemSubscriber (stops message routing)
/// 2. Stop all component actors
/// 3. Drop all infrastructure
///
/// # Returns
///
/// - `Ok(())` - Shutdown completed successfully
/// - `Err(WasmError)` - Shutdown failed (partial shutdown possible)
///
/// # Errors
///
/// - `WasmError::ShutdownFailed` - Failed to stop subscriber or components
///
/// # Examples
///
/// ```rust,ignore
/// manager.shutdown().await?;
/// ```
pub async fn shutdown(&mut self) -> Result<(), WasmError> {
    // Step 1: Stop ActorSystemSubscriber (stops message routing)
    let mut subscriber = self.actor_system_subscriber.write().await;
    subscriber.stop().await?;
    drop(subscriber);

    // Step 2: Mark system as not started
    self.started.store(false, std::sync::atomic::Ordering::SeqCst);

    tracing::info!("HostSystemManager shutdown complete");

    Ok(())
}
```

**Acceptance Criteria:**
1. shutdown() method implemented in HostSystemManager
2. ActorSystemSubscriber.stop() called during shutdown
3. Started flag set to false after shutdown
4. Method compiles without errors
5. Zero clippy warnings
6. Comprehensive error handling

**ADR Constraints:**
- **KNOWLEDGE-WASM-036:** Shutdown in reverse initialization order
- **KNOWLEDGE-WASM-036:** Graceful shutdown procedures owned by host_system/

**PROJECTS_STANDARD.md Compliance:**
- **§6.4:** Quality gates - proper resource cleanup, no leaks
- **M-ERRORS-CANONICAL-STRUCTS:** WasmError types used correctly

**Rust Guidelines:**
- **M-DESIGN-FOR-AI:** Clear shutdown flow with step-by-step comments
- **M-MODULE-DOCS:** Canonical documentation sections (summary, examples, errors)

**Implementation Details:**
Shutdown order is critical for correctness:
1. Stop subscriber first (stops incoming messages)
2. Stop components (process remaining messages, then stop)
3. Drop infrastructure (cleanup)
This prevents message delivery to stopped components.

**Unit Tests:**
- **Exact location:** `airssys-wasm/src/host_system/manager.rs:901-1000`
- **Exact test count:** 3 tests
- **Test names:**
  1. test_host_system_manager_shutdown_success
  2. test_host_system_manager_shutdown_order
  3. test_host_system_manager_shutdown_idempotent

#### Subtask 5.6: Verify ComponentSpawner Does Not Use ActorSystemSubscriber

**Deliverables:**
- **Exact file:** `airssys-wasm/src/actor/component/component_spawner.rs`
- **Exact line range:** 1-200 (import and struct sections)
- **Exact code changes:** Verification only (no changes expected)

**Verification:**
Ensure ComponentSpawner does NOT import or use ActorSystemSubscriber.

**Acceptance Criteria:**
1. ComponentSpawner does NOT import ActorSystemSubscriber
2. ComponentSpawner does NOT pass ActorSystemSubscriber to any constructor
3. ComponentSpawner methods do NOT reference ActorSystemSubscriber
4. ComponentSpawner compiles without errors
5. Zero clippy warnings

**ADR Constraints:**
- **ADR-WASM-023:** actor/ → actor/ imports are allowed, but dependency injection preferred
- **KNOWLEDGE-WASM-036:** ComponentSpawner should receive dependencies via constructor, not create them

**PROJECTS_STANDARD.md Compliance:**
- **§6.1:** YAGNI - Remove unused ActorSystemSubscriber references if found

**Rust Guidelines:**
- **M-DESIGN-FOR-AI:** Clear separation - ComponentSpawner spawns, HostSystemManager coordinates

**Implementation Details:**
ComponentSpawner's responsibility is to spawn ComponentActors. Message routing (ActorSystemSubscriber) is owned by HostSystemManager. If ComponentSpawner currently tries to create or manage ActorSystemSubscriber, that's an architectural violation.

**Unit Tests:**
- **Exact location:** `airssys-wasm/src/actor/component/component_spawner.rs:200-400`
- **Exact test count:** 2 tests (verification tests)
- **Test names:**
  1. test_component_spawner_no_actor_system_subscriber_dependency
  2. test_component_spawner_spawn_does_not_create_subscriber

#### Subtask 5.7: Update All ActorSystemSubscriber::new() Callers

**Deliverables:**
- **Exact files:**
  - `airssys-wasm/src/actor/message/unified_router.rs`
  - `airssys-wasm/src/actor/message/messaging_subscription.rs`
  - Any test files that call ActorSystemSubscriber::new()
- **Exact code changes:** Update all calls to ActorSystemSubscriber::new() to remove registry parameter

**Acceptance Criteria:**
1. All calls to ActorSystemSubscriber::new() updated to 2-parameter version (broker, subscriber_manager)
2. No registry parameter passed to ActorSystemSubscriber::new() anywhere in codebase
3. All files compile without errors
4. Zero clippy warnings

**ADR Constraints:**
- **KNOWLEDGE-WASM-036:** Consistent dependency injection pattern across all callers

**PROJECTS_STANDARD.md Compliance:**
- **§6.4:** Quality gates - All code updated, no breaking calls remain

**Rust Guidelines:**
- **M-DESIGN-FOR-AI:** Consistent API usage across codebase

**Implementation Details:**
Find all calls to ActorSystemSubscriber::new() and update:
```rust
// BEFORE:
ActorSystemSubscriber::new(broker, registry, subscriber_manager)

// AFTER:
ActorSystemSubscriber::new(broker, subscriber_manager)
```

**Unit Tests:**
- **Exact location:** Test files in modified modules
- **Exact test count:** 2 tests (in unified_router.rs and messaging_subscription.rs)
- **Test names:**
  1. test_unified_router_actor_system_subscriber_creation
  2. test_messaging_subscription_actor_system_subscriber_creation

### Unit Testing Plan

**Unit Test Deliverables:**
- **Total unit tests:** 18 tests
- **Test coverage target:** 95% (all public methods tested)
- **Test locations:**
  - `airssys-wasm/src/actor/message/actor_system_subscriber.rs:563-700` (3 modified tests)
  - `airssys-wasm/src/host_system/manager.rs:604-1000` (13 new tests)
  - `airssys-wasm/src/actor/component/component_spawner.rs:200-400` (2 new tests)

**Unit Tests to Include:**

**ActorSystemSubscriber Tests (3 modified):**
1. test_actor_system_subscriber_creation - Update to new 2-parameter constructor
2. test_actor_system_subscriber_start - Update constructor call
3. test_actor_system_subscriber_stop - Update constructor call

**HostSystemManager Tests (13 new):**
1. test_host_system_manager_owns_actor_system_subscriber - Verify field exists and is correct type
2. test_host_system_manager_starts_subscriber_during_init - Verify start() called
3. test_host_system_manager_stops_subscriber_during_shutdown - Verify stop() called
4. test_host_system_manager_new_success - Full initialization test
5. test_host_system_manager_new_creates_subscriber - Verify dependency injection
6. test_host_system_manager_new_starts_subscriber - Verify subscriber started
7. test_host_system_manager_new_initialization_order - Verify correct order
8. test_host_system_manager_new_error_handling - Test error paths
9. test_host_system_manager_shutdown_success - Full shutdown test
10. test_host_system_manager_shutdown_order - Verify reverse order
11. test_host_system_manager_shutdown_idempotent - Multiple shutdown calls
12. test_host_system_manager_registry_ownership - Verify registry owned by host_system
13. test_host_system_manager_dependency_flow - Verify correct dependency directions

**ComponentSpawner Tests (2 new):**
1. test_component_spawner_no_actor_system_subscriber_dependency - Verify no imports
2. test_component_spawner_spawn_does_not_create_subscriber - Verify delegation

### Integration Testing Plan

**Integration Test Deliverables:**
- **Total integration tests:** 4 tests
- **Test location:** `airssys-wasm/tests/host_system-integration-tests.rs`
- **Fixture verification:** All 9 WASM fixtures verified

**Available fixtures verified:**
```bash
$ ls -la airssys-wasm/tests/fixtures/*.wasm
-rw-r--r--  1 hiraq  staff  162 Dec 26 22:26 basic-handle-message.wasm
-rw-r--r--  1 hiraq  staff  630 Dec 26 22:26 callback-receiver-component.wasm
-rw-r--r--  1 hiraq  staff  177 Dec 26 22:26 echo-handler.wasm
-rw-r--r--  1 hiraq  staff  493 Dec 26 22:26 handle-message-component.wasm
-rw-r--r--  1 hiraq  staff  149 Dec 26 22:26 hello_world.wasm
-rw-r--r--  1 hiraq  staff   85 Dec 26 22:26 no-handle-message.wasm
-rw-r--r--  1 hiraq  staff  163 Dec 26 22:26 rejecting-handler.wasm
-rw-r--r--  1 hiraq  staff  173 Dec 26 22:26 sender-validator.wasm
-rw-r--r--  1 hiraq  staff  223 Dec 26 22:26 slow-handler.wasm
```

**Integration Tests to Include:**

1. **test_phase5_host_system_manager_lifecycle** - Full system lifecycle
   - Create HostSystemManager via new()
   - Verify ActorSystemSubscriber is created and started
   - Verify ComponentRegistry is owned by HostSystemManager
   - Spawn component using handle-message-component.wasm
   - Verify component registered
   - Shutdown system
   - Verify subscriber stopped
   - Verify clean state

2. **test_phase5_dependency_injection_flow** - Verify dependency injection
   - Create HostSystemManager
   - Verify ActorSystemSubscriber received broker via Arc
   - Verify ActorSystemSubscriber received subscriber_manager via Arc
   - Verify ComponentRegistry is NOT passed to ActorSystemSubscriber
   - Verify ActorSystemSubscriber mailbox_senders works correctly
   - Test message delivery to component

3. **test_phase5_no_circular_dependencies** - Verify architecture
   - Create HostSystemManager
   - Spawn two components
   - Send message from component A to component B
   - Verify message delivered
   - Verify no forbidden imports (grep check in test)
   - Verify ownership is correct (host_system owns, actor uses)

4. **test_phase5_message_routing_with_injected_subscriber** - End-to-end messaging
   - Create HostSystemManager
   - Spawn sender component (sender-validator.wasm)
   - Spawn receiver component (echo-handler.wasm)
   - Send message from sender to receiver via host function
   - Verify message routed through ActorSystemSubscriber
   - Verify message delivered to receiver mailbox
   - Verify ActorSystemSubscriber mailbox_senders map has both components

### Quality Standards

**All subtasks must meet:**
- ✅ Code builds without errors (`cargo build`)
- ✅ Zero compiler warnings (`cargo build --release`)
- ✅ Zero clippy warnings (`cargo clippy --all-targets --all-features -- -D warnings`)
- ✅ Follows PROJECTS_STANDARD.md §2.1-§6.4
- ✅ Follows Rust guidelines (M-DESIGN-FOR-AI, M-MODULE-DOCS, etc.)
- ✅ Unit tests in `#[cfg(test)]` blocks (18 total)
- ✅ Integration tests in `tests/` directory (4 total)
- ✅ All tests pass (`cargo test --lib` and `cargo test --test '*'`)
- ✅ Documentation follows quality standards (M-CANONICAL-DOCS)
- ✅ ADR-WASM-023 module boundary compliance verified
- ✅ Zero architecture violations (no circular dependencies)

### Verification Checklist

**For implementer to run after completing Phase 5:**

1. **Build:**
```bash
cargo build
```
- **Expected:** `Finished dev profile [unoptimized + debuginfo] target(s)` in <2s
- **Expected:** No compiler warnings

2. **Unit Tests:**
```bash
cargo test --lib host_system
cargo test --lib actor_system_subscriber
cargo test --lib component_spawner
```
- **Expected:** All unit tests passing (18 tests)
- **Expected:** `test result: ok. X passed; 0 failed; 0 ignored`

3. **Integration Tests:**
```bash
cargo test --test host_system-integration-tests
```
- **Expected:** All integration tests passing (4 tests)
- **Expected:** `test result: ok. X passed; 0 failed; 0 ignored`

4. **Clippy:**
```bash
cargo clippy --all-targets --all-features -- -D warnings
```
- **Expected:** `Finished dev profile` with zero warnings
- **Expected:** No clippy warnings

5. **Architecture Verification (ADR-WASM-023 Compliance):**
```bash
# Check 1: ActorSystemSubscriber does NOT own ComponentRegistry
grep -n "registry: ComponentRegistry" airssys-wasm/src/actor/message/actor_system_subscriber.rs
# Expected: No output (field removed)

# Check 2: HostSystemManager owns ComponentRegistry
grep -n "registry: Arc<ComponentRegistry>" airssys-wasm/src/host_system/manager.rs
# Expected: Line found (ownership in host_system)

# Check 3: actor/ does NOT import from host_system/
grep -rn "use crate::host_system" airssys-wasm/src/actor/
# Expected: No output (clean)

# Check 4: runtime/ does NOT import from actor/
grep -rn "use crate::actor" airssys-wasm/src/runtime/
# Expected: No output (clean)
```

6. **Standards Verification:**
```bash
# Check import organization per §2.1
grep -A10 "^// Layer" airssys-wasm/src/actor/message/actor_system_subscriber.rs | head -30
# Expected: std imports, then external, then internal (correct order)

# Check for dyn patterns per §6.2
grep -rn "dyn " airssys-wasm/src/actor/message/actor_system_subscriber.rs airssys-wasm/src/host_system/manager.rs
# Expected: No output (no trait objects)

# Check documentation quality per M-CANONICAL-DOCS
grep -A20 "pub fn new\|pub async fn new\|pub async fn shutdown" airssys-wasm/src/host_system/manager.rs | grep -E "Examples|Errors|Panics"
# Expected: Documentation sections present
```

### Documentation Requirements

**For documentation deliverables:**

- **Follow Diátaxis guidelines:** Reference documentation type for refactored APIs
- **Quality standards:** Technical language, no marketing terms, precise descriptions
- **Canonical sections:** All public methods must have Summary, Examples, Errors, Panics sections per M-CANONICAL-DOCS

**Specific documentation requirements:**

1. **ActorSystemSubscriber::new()** (modified)
   - Update parameter list (remove registry)
   - Update examples (2-parameter constructor)
   - Remove references to registry in documentation

2. **HostSystemManager struct** (modified)
   - Add field documentation for actor_system_subscriber
   - Document ownership model (host_system owns, actor uses)

3. **HostSystemManager::new()** (new)
   - Summary: Create and initialize all infrastructure
   - Parameters: None (all infrastructure created internally)
   - Returns: Result<HostSystemManager, WasmError>
   - Examples: Show initialization and spawn
   - Errors: Document WasmError::EngineInitialization, WasmError::InitializationFailed
   - Panics: None (no unwrap usage)
   - Implementation details: Step-by-step initialization order per KNOWLEDGE-WASM-036

4. **HostSystemManager::shutdown()** (new)
   - Summary: Graceful shutdown of all infrastructure
   - Returns: Result<(), WasmError>
   - Examples: Show shutdown sequence
   - Errors: Document WasmError::ShutdownFailed
   - Panics: None
   - Implementation details: Reverse initialization order

### Standards Compliance Checklist

**PROJECTS_STANDARD.md Applied:**
- [ ] **§2.1 3-Layer Import Organization** - Evidence: All modified files have std → external → internal import order (verified via grep)
- [ ] **§3.2 chrono DateTime<Utc> Standard** - Evidence: Not applicable (no time operations)
- [ ] **§4.3 Module Architecture** - Evidence: mod.rs files unchanged, only contain declarations
- [ ] **§6.1 YAGNI Principles** - Evidence: Only dependency injection implemented, no speculative features
- [ ] **§6.2 Avoid `dyn` Patterns** - Evidence: Used Arc<T> generics, no trait objects (verified via grep)
- [ ] **§6.4 Implementation Quality Gates** - Evidence: Zero warnings, 18 unit + 4 integration tests, all passing

**Rust Guidelines Applied:**
- [ ] **M-DESIGN-FOR-AI** - Evidence: Idiomatic dependency injection pattern, clear ownership semantics
- [ ] **M-MODULE-DOCS** - Evidence: Module documentation updated, public methods have canonical sections
- [ ] **M-ERRORS-CANONICAL-STRUCTS** - Evidence: WasmError types used correctly, no ad-hoc errors
- [ ] **M-STATIC-VERIFICATION** - Evidence: `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] **M-FEATURES-ADDITIVE** - Evidence: Changes don't break ComponentRegistry API, only move ownership

**ADR/Knowledge Compliance:**
- [ ] **ADR-WASM-023 Module Boundary Enforcement** - Evidence: ActorSystemSubscriber no longer owns ComponentRegistry, host_system owns it (verified via grep)
- [ ] **ADR-WASM-020 Message Delivery Ownership** - Evidence: ActorSystemSubscriber keeps mailbox_senders for delivery, registry moved
- [ ] **KNOWLEDGE-WASM-036 Four-Module Architecture** - Evidence: Dependency injection pattern applied per Lines 518-540, initialization order per Lines 414-452
- [ ] **KNOWLEDGE-WASM-026 Message Delivery Architecture** - Evidence: mailbox_senders ownership preserved, registry ownership moved

**Documentation Quality:**
- [ ] **No hyperbolic terms** - Evidence: Verified against forbidden list in documentation-quality-standards.md
- [ ] **Technical precision** - Evidence: All claims measurable (e.g., "O(1) lookup", "<100ns latency")
- [ ] **Diátaxis compliance** - Evidence: Reference documentation type for refactored APIs
- [ ] **Canonical sections** - Evidence: Summary, Examples, Errors, Panics present in all public methods

**Testing Requirements (AGENTS.md §8):**
- [ ] **Unit Tests Present** - Evidence: 18 tests in #[cfg(test)] blocks
- [ ] **Integration Tests Present** - Evidence: 4 tests in tests/ directory
- [ ] **All Tests Passing** - Evidence: `cargo test` shows 100% pass rate
- [ ] **Tests Verify REAL Functionality** - Evidence: Tests spawn real components, send real messages, verify actual delivery (not stubs)
- [ ] **Zero Compiler Warnings** - Evidence: `cargo build` clean
- [ ] **Zero Clippy Warnings** - Evidence: `cargo clippy -D warnings` clean

**Architecture Verification:**
- [ ] **No Circular Dependencies** - Evidence: Verified via grep commands (host_system owns, actor uses)
- [ ] **Correct Dependency Direction** - Evidence: host_system → actor allowed per ADR-WASM-023, actor → host_system forbidden (verified via grep)
- [ ] **Ownership Clear** - Evidence: HostSystemManager owns ComponentRegistry and ActorSystemSubscriber
