# [WASM-TASK-004 Phase 5 Task 5.2] - Lifecycle Hooks and Custom State Management 📋

**Status:** pending  
**Added:** 2025-12-16  
**Estimated Effort:** 4-6 hours  
**Quality Target:** 9.5/10  
**Test Target:** 20-30 comprehensive tests  
**Documentation:** 100% rustdoc coverage

---

## Overview

This task implements custom lifecycle hooks and state management for ComponentActor, enabling developers to hook into key lifecycle events (pre/post-start/stop) and maintain custom state across message handling. This completes Phase 5 (Advanced Actor Patterns) and brings Block 3 to 100% completion (18/18 tasks).

**Why This Matters:**
- **Extensibility**: Enables custom component behavior without modifying framework code
- **Debugging**: Provides hooks for observability and error handling (on_error, on_restart)
- **State Management**: Allows components to maintain persistent state across requests
- **Phase Completion**: Completes Phase 5 Advanced Actor Patterns (1/2 tasks done)
- **Block Gate**: Final task before Phase 6 (Testing & Validation)

**Fits Into Phase 5:**
- Phase 5 is "Advanced Actor Patterns"
- Task 5.1: Message Correlation and Request-Response ✅ (just completed, 9.5/10)
- Task 5.2: Lifecycle Hooks and Custom State (THIS TASK)
- After completion: Ready for Phase 6 (Integration Testing & Validation)

---

## Objectives

### Primary Objective
Implement extensible lifecycle hooks (pre/post-start/stop, on_message, on_error, on_restart) and custom state management for ComponentActor to enable advanced component patterns while maintaining <50μs hook overhead and production-ready quality.

### Secondary Objectives
1. Enable components to maintain custom state across message handling
2. Provide event callbacks for monitoring and debugging
3. Implement timeout protection for hooks (configurable, 1000ms default)
4. Implement panic handling (catch_unwind) to prevent hook crashes
5. Maintain zero impact on message throughput (>10,000 msg/sec)
6. Establish testing framework for lifecycle patterns

---

## Scope

### In Scope
1. **LifecycleHooks trait** (7 hook methods)
   - Pre/post-start hooks
   - Pre/post-stop hooks
   - on_message_received hook
   - on_error hook
   - on_restart hook

2. **Custom State Management**
   - Arc<RwLock<dyn Any>> storage in ComponentActor
   - Type-safe get/set/with_custom_state methods
   - Downcast checking and error handling

3. **EventCallback trait** (5 event methods)
   - on_message_received (before routing)
   - on_message_processed (after routing, with latency)
   - on_error_occurred (with error context)
   - on_restart_triggered (with reason)
   - on_health_changed (health status changes)

4. **Hook Integration**
   - Call pre_start before Child::start()
   - Call post_start after Child::start()
   - Call pre_stop before Child::stop()
   - Call post_stop after Child::stop()
   - Call on_message_received in Actor::handle_message()
   - Call on_error on any WasmError
   - Call on_restart after supervisor restart

5. **Testing & Documentation**
   - 20-30 comprehensive tests
   - 100% rustdoc coverage with examples
   - Performance validation
   - Integration tests with real lifecycle

### Out of Scope
- Security filtering of hooks (all hooks trusted code)
- Distributed hook execution (all hooks execute locally)
- Hook middleware pipeline (single hook per type)
- Persistent hook state (state not persisted to disk)
- Hook configuration persistence
- Advanced async patterns (sync hooks only for reliability)

---

## Architecture & Design

### 1. LifecycleHooks Trait Design

```rust
// File: src/actor/lifecycle/hooks.rs

use crate::core::WasmError;
use crate::actor::ComponentMessage;

/// Context passed to lifecycle hooks
#[derive(Clone)]
pub struct LifecycleContext {
    pub component_id: ComponentId,
    pub actor_address: ActorAddress,
    pub timestamp: SystemTime,
}

/// Hook invocation results
#[derive(Debug, Clone)]
pub enum HookResult {
    Ok,
    Error(String),
    Timeout,
}

/// Lifecycle hooks for ComponentActor
/// 
/// Hooks are called at key lifecycle events. Implement this trait to customize
/// component behavior without modifying framework code.
/// 
/// Default implementations are no-op (empty), allowing opt-in customization.
/// All hooks have 1000ms timeout protection and panic safety.
pub trait LifecycleHooks: Send + Sync {
    /// Called before component starts
    /// 
    /// Use for: initial setup, configuration validation, resource allocation
    fn pre_start(&mut self, ctx: &LifecycleContext) -> HookResult {
        HookResult::Ok
    }
    
    /// Called after component successfully starts
    /// 
    /// Use for: startup completion logging, dependency injection, initialization callbacks
    fn post_start(&mut self, ctx: &LifecycleContext) -> HookResult {
        HookResult::Ok
    }
    
    /// Called before component stops
    /// 
    /// Use for: cleanup logic, state saving, connection closing
    fn pre_stop(&mut self, ctx: &LifecycleContext) -> HookResult {
        HookResult::Ok
    }
    
    /// Called after component stops
    /// 
    /// Use for: final cleanup, resource release, stop confirmation
    fn post_stop(&mut self, ctx: &LifecycleContext) -> HookResult {
        HookResult::Ok
    }
    
    /// Called when message received (before routing to WASM)
    /// 
    /// Use for: request logging, authentication, rate limiting pre-checks
    fn on_message_received(&mut self, ctx: &LifecycleContext, msg: &ComponentMessage) -> HookResult {
        HookResult::Ok
    }
    
    /// Called when error occurs anywhere in component
    /// 
    /// Use for: error logging, metrics collection, error recovery
    fn on_error(&mut self, ctx: &LifecycleContext, error: &WasmError) -> HookResult {
        HookResult::Ok
    }
    
    /// Called when supervisor triggers component restart
    /// 
    /// Use for: restart logging, state reset, notification hooks
    fn on_restart(&mut self, ctx: &LifecycleContext, reason: RestartReason) -> HookResult {
        HookResult::Ok
    }
}

/// Default no-op implementation (opt-in customization)
pub struct NoOpHooks;

impl LifecycleHooks for NoOpHooks {}
```

**Design Decisions:**
- ✅ **Default no-op implementations** - Opt-in customization, zero overhead if not used
- ✅ **HookResult enum** - Allows hooks to report errors without panicking
- ✅ **LifecycleContext** - Provides component_id, actor_address, timestamp
- ✅ **&mut self** - Allows hooks to maintain state across calls
- ✅ **Send + Sync** - Ensures thread-safe hook execution

### 2. Custom State Management Design

```rust
// File: src/actor/lifecycle/state.rs

use std::any::Any;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Custom state storage for ComponentActor
/// 
/// Type-safe state management using Arc<RwLock<dyn Any>>.
/// Downcast checks ensure type safety at runtime.
pub struct CustomState {
    state: Arc<RwLock<Box<dyn Any + Send + Sync>>>,
}

impl CustomState {
    /// Set custom state (overwrites previous state)
    pub async fn set<T: Any + Send + Sync + 'static>(&self, value: T) -> Result<()> {
        let mut guard = self.state.write().await;
        *guard = Box::new(value);
        Ok(())
    }
    
    /// Get custom state by reference
    pub async fn get<T: Any + Send + Sync + 'static>(&self) -> Result<Arc<T>> {
        let guard = self.state.read().await;
        guard
            .downcast_ref::<T>()
            .map(|t| Arc::new(t.clone()))
            .ok_or_else(|| WasmError::CustomStateTypeMismatch)
    }
    
    /// Execute closure with custom state
    pub async fn with_custom_state<T, F, R>(&self, f: F) -> Result<R>
    where
        T: Any + Send + Sync + 'static,
        F: FnOnce(&T) -> R,
    {
        let guard = self.state.read().await;
        guard
            .downcast_ref::<T>()
            .map(f)
            .ok_or_else(|| WasmError::CustomStateTypeMismatch)
    }
}

// Add to ComponentActor struct:
// custom_state: Arc<RwLock<Box<dyn Any + Send + Sync>>>,
```

**Design Decisions:**
- ✅ **Arc<RwLock<>>** - Thread-safe, zero-copy access via Arc
- ✅ **Box<dyn Any>** - Type-erased storage for any type
- ✅ **Downcast at runtime** - Catches type mismatches with clear errors
- ✅ **Async API** - Matches actor model (async message handling)
- ✅ **Clone trait requirement** - Unnecessary, removing from design

### 3. EventCallback Trait Design

```rust
// File: src/actor/lifecycle/callbacks.rs

use std::time::Duration;

/// Event callback for monitoring component lifecycle
pub trait EventCallback: Send + Sync {
    /// Called when component receives a message
    fn on_message_received(&self, component_id: ComponentId) {}
    
    /// Called when component finishes processing message (with latency)
    fn on_message_processed(&self, component_id: ComponentId, latency: Duration) {}
    
    /// Called when error occurs in component
    fn on_error_occurred(&self, component_id: ComponentId, error: &WasmError) {}
    
    /// Called when supervisor restarts component
    fn on_restart_triggered(&self, component_id: ComponentId, reason: RestartReason) {}
    
    /// Called when component health status changes
    fn on_health_changed(&self, component_id: ComponentId, new_health: HealthStatus) {}
}

/// Default no-op implementation
pub struct NoOpEventCallback;

impl EventCallback for NoOpEventCallback {}
```

**Design Decisions:**
- ✅ **Immutable &self** - Callbacks don't modify component state
- ✅ **Arc<dyn EventCallback>** - Optional registration, Arc for sharing
- ✅ **Non-blocking** - No async, fires immediately
- ✅ **Error context** - Full error and reason passed to callbacks
- ✅ **Latency tracking** - Measure message processing time

### 4. Integration Points

#### In Child::start()
```rust
pub fn start(&mut self, ctx: &ActorContext) -> Result<()> {
    let hook_ctx = LifecycleContext::new(self.component_id.clone(), ctx.address().clone());
    
    // CALL PRE-START HOOK (timeout protected)
    match timeout(Duration::from_millis(1000), 
                  self.hooks.pre_start(&hook_ctx)).await {
        Ok(HookResult::Ok) => {},
        Ok(HookResult::Error(e)) => log_hook_error(&e),
        Ok(HookResult::Timeout) => {},
        Err(_) => {} // Timeout
    }
    
    // ... rest of start logic ...
    
    // CALL POST-START HOOK (timeout protected)
    match timeout(Duration::from_millis(1000), 
                  self.hooks.post_start(&hook_ctx)).await {
        Ok(HookResult::Ok) => {},
        Ok(HookResult::Error(e)) => log_hook_error(&e),
        Ok(HookResult::Timeout) => {},
        Err(_) => {} // Timeout
    }
}
```

#### In Child::stop()
Same pattern: pre_stop before cleanup, post_stop after cleanup

#### In Actor::handle_message()
```rust
pub async fn handle_message(&mut self, msg: ComponentMessage, ctx: &ActorContext) -> Result<()> {
    let hook_ctx = LifecycleContext::new(self.component_id.clone(), ctx.address().clone());
    let start_time = Instant::now();
    
    // CALL ON-MESSAGE-RECEIVED HOOK
    let _ = catch_unwind(AssertUnwindSafe(|| {
        self.hooks.on_message_received(&hook_ctx, &msg)
    }));
    
    // EVENT CALLBACK
    if let Some(cb) = &self.event_callback {
        cb.on_message_received(self.component_id.clone());
    }
    
    // ... process message ...
    
    // EVENT CALLBACK WITH LATENCY
    let latency = start_time.elapsed();
    if let Some(cb) = &self.event_callback {
        cb.on_message_processed(self.component_id.clone(), latency);
    }
}
```

---

## Implementation Plan

### Step 1: Create Lifecycle Hooks Module (1 hour)
**File:** `src/actor/lifecycle/hooks.rs` (250 lines)
**Deliverables:**
- LifecycleHooks trait with 7 hook methods
- LifecycleContext struct
- HookResult enum
- NoOpHooks default implementation
- 8 unit tests

**Code:**
```rust
pub mod lifecycle;

// hooks.rs exports:
pub use hooks::{LifecycleHooks, LifecycleContext, HookResult, NoOpHooks};
```

**Success Criteria:**
- ✅ Trait compiles with default implementations
- ✅ HookResult covers Ok/Error/Timeout
- ✅ LifecycleContext has component_id, actor_address, timestamp
- ✅ 8 unit tests for trait basics

**Performance:**
- Zero overhead for no-op implementations
- Trait object indirection: ~1-2μs per call

---

### Step 2: Create Custom State Management Module (1 hour)
**File:** `src/actor/lifecycle/state.rs` (200 lines)
**Deliverables:**
- CustomState struct
- set/get/with_custom_state methods
- Type safety with downcast checking
- Error handling for type mismatches
- 10 unit tests (type safety, concurrency)

**Code:**
```rust
pub struct CustomState {
    state: Arc<RwLock<Box<dyn Any + Send + Sync>>>,
}

impl CustomState {
    pub fn new() -> Self { /* ... */ }
    pub async fn set<T: Any + Send + Sync + 'static>(&self, value: T) -> Result<()> { /* ... */ }
    pub async fn get<T: Any + Send + Sync + 'static>(&self) -> Result<Arc<T>> { /* ... */ }
}
```

**Success Criteria:**
- ✅ Type-safe access with downcast
- ✅ Thread-safe Arc<RwLock<>>
- ✅ Clear error messages for type mismatches
- ✅ 10 passing tests

**Performance:**
- Arc clone: <100ns
- RwLock acquisition: <100ns
- Downcast: <50ns
- Total: <1μs per access

---

### Step 3: Create EventCallback Module (45 minutes)
**File:** `src/actor/lifecycle/callbacks.rs` (150 lines)
**Deliverables:**
- EventCallback trait with 5 event methods
- NoOpEventCallback implementation
- RestartReason enum (if not exists)
- 6 unit tests

**Code:**
```rust
pub trait EventCallback: Send + Sync {
    fn on_message_received(&self, component_id: ComponentId) {}
    fn on_message_processed(&self, component_id: ComponentId, latency: Duration) {}
    fn on_error_occurred(&self, component_id: ComponentId, error: &WasmError) {}
    fn on_restart_triggered(&self, component_id: ComponentId, reason: RestartReason) {}
    fn on_health_changed(&self, component_id: ComponentId, new_health: HealthStatus) {}
}
```

**Success Criteria:**
- ✅ All 5 events defined
- ✅ No-op implementations
- ✅ 6 passing tests

---

### Step 4: Add Hooks and State to ComponentActor (1 hour)
**File:** `src/actor/component/component_actor.rs` (modify, +50 lines)
**Deliverables:**
- Add `hooks: Box<dyn LifecycleHooks>` field
- Add `custom_state: CustomState` field
- Add `event_callback: Option<Arc<dyn EventCallback>>` field
- Add setter methods: set_lifecycle_hooks, set_event_callback
- Add custom state accessors (delegate to CustomState)
- 8 unit tests

**Code Snippet:**
```rust
pub struct ComponentActor {
    // ... existing fields ...
    
    /// Custom lifecycle hooks (default: no-op)
    hooks: Box<dyn LifecycleHooks>,
    
    /// Custom state storage
    custom_state: CustomState,
    
    /// Optional event callback
    event_callback: Option<Arc<dyn EventCallback>>,
}

impl ComponentActor {
    pub fn set_lifecycle_hooks(&mut self, hooks: Box<dyn LifecycleHooks>) {
        self.hooks = hooks;
    }
    
    pub fn set_event_callback(&mut self, callback: Arc<dyn EventCallback>) {
        self.event_callback = Some(callback);
    }
    
    pub async fn set_custom_state<T: Any + Send + Sync + 'static>(&self, value: T) -> Result<()> {
        self.custom_state.set(value).await
    }
}
```

**Success Criteria:**
- ✅ Fields added to ComponentActor
- ✅ Setters implemented
- ✅ Accessors delegate to CustomState
- ✅ 8 passing tests

---

### Step 5: Integrate Hooks into Child::start() (1 hour)
**File:** `src/actor/component/child_impl.rs` (modify, +80 lines)
**Deliverables:**
- Call pre_start hook before WASM loading
- Call post_start hook after WASM loading
- Timeout protection (1000ms, configurable)
- Panic handling (catch_unwind)
- Error logging
- 5 integration tests

**Code Pattern:**
```rust
pub async fn start(&mut self, ctx: &ActorContext) -> Result<()> {
    let hook_ctx = self.create_lifecycle_context(ctx);
    
    // Pre-start hook (timeout + panic safe)
    if let Err(e) = self.call_hook_with_timeout(&hook_ctx, |h| h.pre_start(&hook_ctx)).await {
        warn!("pre_start hook failed: {}", e);
    }
    
    // ... WASM loading logic ...
    
    // Post-start hook
    if let Err(e) = self.call_hook_with_timeout(&hook_ctx, |h| h.post_start(&hook_ctx)).await {
        warn!("post_start hook failed: {}", e);
    }
    
    Ok(())
}
```

**Success Criteria:**
- ✅ Hooks called in correct order
- ✅ Timeout protection working
- ✅ Panic handling (no crash)
- ✅ 5 passing integration tests

**Performance:**
- Hook overhead: <50μs per hook
- Timeout check: <10μs

---

### Step 6: Integrate Hooks into Child::stop() (1 hour)
**File:** `src/actor/component/child_impl.rs` (modify, +80 lines)
**Deliverables:**
- Call pre_stop hook before cleanup
- Call post_stop hook after cleanup
- Timeout and panic protection
- 5 integration tests

**Same pattern as Step 5**

---

### Step 7: Integrate Hooks into Actor::handle_message() (1.5 hours)
**File:** `src/actor/component/actor_impl.rs` (modify, +120 lines)
**Deliverables:**
- Call on_message_received hook before WASM invocation
- Fire on_message_received event callback
- Fire on_message_processed event callback (with latency)
- Fire on_error event callback (in error path)
- Performance measurement (message latency)
- 8 integration tests

**Code Pattern:**
```rust
pub async fn handle_message(&mut self, msg: ComponentMessage, ctx: &ActorContext) -> Result<()> {
    let hook_ctx = self.create_lifecycle_context(ctx);
    let start_time = Instant::now();
    
    // Hook: on_message_received
    if let Err(e) = self.call_hook_with_timeout(&hook_ctx, |h| h.on_message_received(&hook_ctx, &msg)).await {
        warn!("on_message_received hook failed: {}", e);
    }
    
    // Callback: on_message_received
    if let Some(cb) = &self.event_callback {
        cb.on_message_received(self.component_id.clone());
    }
    
    // Process message
    match self.invoke_wasm(&msg).await {
        Ok(response) => {
            // Callback: on_message_processed (with latency)
            let latency = start_time.elapsed();
            if let Some(cb) = &self.event_callback {
                cb.on_message_processed(self.component_id.clone(), latency);
            }
            Ok(())
        }
        Err(e) => {
            // Hook: on_error
            let _ = self.call_hook_with_timeout(&hook_ctx, |h| h.on_error(&hook_ctx, &e)).await;
            
            // Callback: on_error_occurred
            if let Some(cb) = &self.event_callback {
                cb.on_error_occurred(self.component_id.clone(), &e);
            }
            
            Err(e)
        }
    }
}
```

**Success Criteria:**
- ✅ Hooks called at correct points
- ✅ Event callbacks fired
- ✅ Latency measurement working
- ✅ 8 passing integration tests

**Performance:**
- Hook + callback overhead: <40μs
- Latency measurement: <5μs

---

### Step 8: Helper Methods for Hook Execution (45 minutes)
**File:** `src/actor/lifecycle/executor.rs` (new, 200 lines)
**Deliverables:**
- call_hook_with_timeout - Wraps hook call with timeout
- catch_unwind_hook - Handles hook panics
- log_hook_error - Centralized error logging
- Tests: 6 unit tests

**Code:**
```rust
pub async fn call_hook_with_timeout<F>(
    f: F,
    timeout_ms: u64,
) -> Result<HookResult>
where
    F: FnOnce() -> HookResult + Send + 'static,
{
    match tokio::time::timeout(
        Duration::from_millis(timeout_ms),
        async {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(f))
                .unwrap_or(HookResult::Error("hook panicked".to_string()))
        },
    ).await {
        Ok(result) => Ok(result),
        Err(_) => Ok(HookResult::Timeout),
    }
}
```

**Success Criteria:**
- ✅ Timeout protection working
- ✅ Panic handling working
- ✅ 6 passing unit tests

---

### Step 9: Unit Tests (1 hour)
**File:** `tests/lifecycle_hooks_tests.rs` (new, 300 lines)
**Test Cases:**
1. Hook invocation order (pre_start → WASM → post_start)
2. Hook error handling (error doesn't crash actor)
3. Hook timeout (timeout doesn't block actor)
4. Hook panic (panic doesn't crash actor)
5. Custom state set/get
6. Custom state type safety (downcast error)
7. Custom state concurrency (multiple readers)
8. Event callbacks fired
9. Event callback latency measurement
10. Multiple components with different hooks

**Success Criteria:**
- ✅ 10 passing tests
- ✅ <50μs hook overhead verified
- ✅ <1μs state access verified
- ✅ <10μs callback overhead verified

---

### Step 10: Integration Tests (1.5 hours)
**File:** `tests/lifecycle_integration_tests.rs` (new, 400 lines)
**Test Cases:**
1. Full lifecycle: spawn → start (hooks) → message → stop (hooks)
2. Hook with custom state persistence
3. Multiple messages with consistent state
4. Hook error recovery
5. Hook timeout recovery
6. Event callbacks throughout lifecycle
7. Concurrent components with different hooks
8. Component restart with on_restart hook
9. Error handling with on_error hook
10. Health monitoring with on_health_changed callback

**Success Criteria:**
- ✅ 10 passing integration tests
- ✅ Full lifecycle verified
- ✅ State persistence verified
- ✅ Error recovery verified

---

### Step 11: Documentation & Examples (1 hour)
**Deliverables:**
- 100% rustdoc coverage
- Usage examples for each hook type
- Custom state example
- Event callback example
- Performance benchmarks in BENCHMARKING.md
- Integration patterns documentation

**Files:**
- Comprehensive rustdoc for all public types
- Example: `examples/lifecycle_hooks_example.rs` (200 lines)
- Example: `examples/custom_state_example.rs` (200 lines)
- BENCHMARKING.md section (100 lines)

---

### Step 12: Code Review & Final Verification (1 hour)
**Deliverables:**
- All tests passing (20-30 total)
- Zero compiler warnings
- Zero clippy warnings
- Zero rustdoc warnings
- Performance targets verified
- Code quality 9.5/10
- Standards compliance (§2.1-§6.3)

---

## File Structure

### New Files
```
src/actor/lifecycle/
├── mod.rs (60 lines)
├── hooks.rs (250 lines) - LifecycleHooks trait, LifecycleContext
├── state.rs (200 lines) - CustomState struct
├── callbacks.rs (150 lines) - EventCallback trait
└── executor.rs (200 lines) - Hook execution helpers

tests/
├── lifecycle_hooks_tests.rs (300 lines) - Unit tests
└── lifecycle_integration_tests.rs (400 lines) - Integration tests

examples/
├── lifecycle_hooks_example.rs (200 lines)
└── custom_state_example.rs (200 lines)
```

### Modified Files
```
src/actor/
├── component/component_actor.rs (+50 lines) - Add hooks, state, callbacks
├── component/child_impl.rs (+160 lines) - Integrate start/stop hooks
├── component/actor_impl.rs (+120 lines) - Integrate message hooks
└── mod.rs (+10 lines) - Re-export lifecycle module

src/lib.rs (+5 lines) - Re-export lifecycle module

BENCHMARKING.md (+100 lines) - Hook performance section
```

### Total Code Volume
- **New Code:** ~1,560 lines (hooks, state, callbacks, executor, examples)
- **Modified Code:** ~345 lines (integration points)
- **Tests:** ~700 lines (unit + integration)
- **Documentation:** ~300 lines (examples, rustdoc, BENCHMARKING.md)
- **Total:** ~2,905 lines

---

## Testing Strategy

### Unit Tests (15 tests, ~300 lines)

**LifecycleHooks Tests (4 tests):**
1. Default no-op implementations return Ok
2. Hook context creation
3. Hook result enum variants
4. Multiple hook calls

**CustomState Tests (5 tests):**
1. Set and get state
2. Type mismatch error handling
3. Concurrent reads
4. Concurrent read/write
5. Multiple state updates

**EventCallback Tests (3 tests):**
1. Default no-op implementations
2. Event callback registration
3. Event firing

**Hook Executor Tests (3 tests):**
1. Timeout protection
2. Panic handling
3. Error logging

### Integration Tests (15 tests, ~400 lines)

**Lifecycle Integration (5 tests):**
1. Full lifecycle with hooks firing in order
2. pre_start → start → post_start
3. pre_stop → stop → post_stop
4. Message routing with on_message_received
5. Error propagation with on_error

**Custom State Integration (3 tests):**
1. State persistence across messages
2. State access in hooks
3. Concurrent state access

**Event Callbacks Integration (4 tests):**
1. Event callbacks fire at correct times
2. Latency measurement in callbacks
3. Error events
4. Multiple callbacks

**End-to-End Scenarios (3 tests):**
1. Component lifecycle with state and callbacks
2. Multiple components with different hooks
3. Component restart triggering on_restart

### Performance Tests

**Hook Overhead (<50μs target):**
- pre_start: <50μs
- post_start: <50μs
- on_message_received: <50μs
- on_error: <50μs

**State Access (<1μs target):**
- set_custom_state: <1μs
- get_custom_state: <1μs
- with_custom_state: <1μs

**Callback Dispatch (<10μs target):**
- on_message_received: <10μs
- on_message_processed: <10μs
- on_error_occurred: <10μs

**Message Throughput (>10k/sec baseline):**
- With hooks enabled: >10k/sec maintained
- Overhead: <5% of message time

---

## Success Criteria

### Trait Implementations
- [ ] LifecycleHooks trait with 7 methods (default no-op)
- [ ] EventCallback trait with 5 methods (default no-op)
- [ ] LifecycleContext struct with component_id, actor_address, timestamp
- [ ] HookResult enum with Ok, Error, Timeout variants
- [ ] CustomState struct with set/get/with_custom_state methods

### Hook Integration
- [ ] pre_start hook called before Child::start()
- [ ] post_start hook called after Child::start()
- [ ] pre_stop hook called before Child::stop()
- [ ] post_stop hook called after Child::stop()
- [ ] on_message_received hook called before message routing
- [ ] on_error hook called on WasmError
- [ ] on_restart hook called on component restart

### Custom State
- [ ] Custom state storage in ComponentActor
- [ ] Type-safe set/get operations
- [ ] Downcast error handling
- [ ] Thread-safe Arc<RwLock<>>
- [ ] State persistence across messages

### Event Callbacks
- [ ] Event callbacks optional (registration)
- [ ] on_message_received fired
- [ ] on_message_processed fired with latency
- [ ] on_error_occurred fired
- [ ] on_restart_triggered fired
- [ ] on_health_changed fired

### Timeout & Panic Safety
- [ ] Timeout protection (1000ms default)
- [ ] Panic handling (catch_unwind)
- [ ] Hook errors don't crash actor
- [ ] Hook timeouts don't block actor

### Testing
- [ ] 20-30 comprehensive tests passing
- [ ] Unit tests: ≥10 tests, 100% passing
- [ ] Integration tests: ≥10 tests, 100% passing
- [ ] Performance tests: All targets met

### Performance Targets
- [ ] Hook overhead: <50μs per hook
- [ ] State access: <1μs
- [ ] Callback dispatch: <10μs
- [ ] Message throughput: >10,000 msg/sec maintained
- [ ] No regression in existing performance

### Code Quality
- [ ] Zero compiler warnings
- [ ] Zero clippy warnings
- [ ] Zero rustdoc warnings
- [ ] 100% rustdoc coverage
- [ ] Code quality: 9.5/10
- [ ] Standards compliance: §2.1-§6.3

### Documentation
- [ ] Implementation plan (this document)
- [ ] Completion summary
- [ ] 100% rustdoc with examples
- [ ] Usage examples (hooks, state, callbacks)
- [ ] BENCHMARKING.md updated
- [ ] Architecture patterns documented

---

## Performance Targets

| Component | Target | Baseline | Status |
|-----------|--------|----------|--------|
| Hook execution | <50μs | <50μs (target) | ⏳ TBD |
| Custom state access | <1μs | <1μs (target) | ⏳ TBD |
| Event callback dispatch | <10μs | <10μs (target) | ⏳ TBD |
| Message throughput | >10k/sec | 4.7M+/sec (Phase 4) | ⏳ Verify maintained |
| Total overhead per message | <100μs | <100μs (target) | ⏳ TBD |

---

## Quality Standards

### Code Quality
- **Target:** 9.5/10 (maintain Phase 1-5 standard)
- **Metrics:** Consistency, clarity, maintainability, test coverage

### Rustdoc Coverage
- **Target:** 100%
- **Requirement:** Every public type, trait, method documented with examples

### Warnings Policy
- **Target:** Zero
- **Requirement:** No compiler, clippy, or rustdoc warnings

### Standards Compliance
- **Workspace Patterns:** §2.1 (3-layer imports), §4.3 (modules), §5.1 (dependencies)
- **Error Handling:** §6.1 (structured errors), §6.2 (async patterns)
- **Testing:** Comprehensive unit + integration + performance tests
- **Microsoft Rust:** M-STATIC-VERIFICATION (zero warnings), M-ERRORS-CANONICAL-STRUCTS

---

## Dependencies & Integration

### Upstream Dependencies (All Complete ✅)
- ✅ Phase 5 Task 5.1: Message Correlation & Request-Response (9.5/10, complete)
- ✅ Phase 4 Tasks 4.1-4.3: MessageBroker Integration (100% complete)
- ✅ Phase 3 Tasks 3.1-3.3: SupervisorNode Integration (100% complete)
- ✅ Phase 2 Tasks 2.1-2.3: ActorSystem Integration (100% complete)
- ✅ Phase 1 Tasks 1.1-1.4: ComponentActor Foundation (100% complete)

### Downstream Dependencies (Phase 6)
- ⏳ Phase 6 Task 6.1: Integration Test Suite (needs lifecycle hooks)
- ⏳ Phase 6 Task 6.2: Performance Validation (validates hook performance)
- ⏳ Phase 6 Task 6.3: Actor-Based Testing Framework (uses hooks for test setup)

### Integration Points
- ✅ ComponentActor: Add hooks, state, callback fields
- ✅ Child::start(): Call pre/post_start hooks
- ✅ Child::stop(): Call pre/post_stop hooks
- ✅ Actor::handle_message(): Call on_message_received, fire callbacks
- ✅ airssys-rt ActorContext: Use for LifecycleContext
- ✅ WASM runtime: Error handling with on_error hook

---

## Risks & Mitigations

### Risk 1: Hook Complexity and User Confusion
**Impact:** High (confusing API slows adoption)  
**Probability:** Medium (7 hook types)  
**Mitigation:**
- Provide comprehensive examples for each hook type
- Document common patterns (logging, metrics, state management)
- Default no-op implementations minimize accidental issues
- Clear error messages guide users

### Risk 2: Hook Timeout Complexity
**Impact:** Medium (timeout tuning difficult)  
**Probability:** Medium (1000ms default might not fit all use cases)  
**Mitigation:**
- 1000ms default is conservative (most hooks complete <100μs)
- Make timeout configurable per hook
- Document timeout implications
- Provide logging for timeout occurrences

### Risk 3: Custom State Type Safety Issues
**Impact:** High (runtime type errors painful)  
**Probability:** Low (downcast checks prevent crashes)  
**Mitigation:**
- Implement downcasts carefully with clear error messages
- Provide WasmError::CustomStateTypeMismatch variant
- Document type safety requirements
- Test type mismatches in unit tests

### Risk 4: Performance Regression on Message Throughput
**Impact:** High (breaks Phase 4 performance baseline)  
**Probability:** Low (hooks are optional, no-op by default)  
**Mitigation:**
- Profile hook execution path thoroughly
- Maintain <50μs per hook overhead
- Validate message throughput unchanged (>10k/sec)
- Performance tests catch regressions

### Risk 5: Hook Panic Crashes Actor
**Impact:** High (hook panic should not crash component)  
**Probability:** Medium (user code in hooks)  
**Mitigation:**
- Implement catch_unwind protection
- Log panics with full context
- Return HookResult::Error on panic
- Test panic scenarios thoroughly

---

## Timeline & Estimation

| Step | Description | Duration | Cumulative |
|------|-------------|----------|-----------|
| 1 | Lifecycle Hooks Module | 1.0h | 1.0h |
| 2 | Custom State Module | 1.0h | 2.0h |
| 3 | Event Callback Module | 0.75h | 2.75h |
| 4 | ComponentActor Integration | 1.0h | 3.75h |
| 5 | Child::start() Hooks | 1.0h | 4.75h |
| 6 | Child::stop() Hooks | 1.0h | 5.75h |
| 7 | Actor::handle_message() Hooks | 1.5h | 7.25h |
| 8 | Hook Executor Helpers | 0.75h | 8.0h |
| 9 | Unit Tests | 1.0h | 9.0h |
| 10 | Integration Tests | 1.5h | 10.5h |
| 11 | Documentation | 1.0h | 11.5h |
| 12 | Code Review & Verification | 1.0h | 12.5h |

**TOTAL ESTIMATED EFFORT:** 12.5 hours

**Optimized Estimate:** 4-6 hours (with parallel implementation)

**Actual Target:** 4-6 hours (high efficiency based on Phase 1-5 experience)

---

## Next Steps After Task 5.2

### Phase 6: Testing and Integration Validation (12-16 hours, 3 tasks)

After Task 5.2 completion, Phase 6 begins:

**Task 6.1:** Integration Test Suite (end-to-end lifecycle, multi-component scenarios)  
**Task 6.2:** Performance Validation (benchmarks for all components)  
**Task 6.3:** Actor-Based Testing Framework (test utilities, mock system)

### Block 3 Completion Gate

Once Phase 6 completes:
- ✅ Block 3: 100% complete (18/18 tasks)
- ✅ Actor System Integration: FOUNDATION READY
- ✅ Performance: ALL TARGETS EXCEEDED
- ✅ Quality: 9.5/10 across all phases
- ✅ Tests: 900+ passing
- ✅ Warnings: ZERO

**Ready for Layer 2 (Blocks 4-7):**
- Block 4: Security & Isolation
- Block 5: Inter-Component Communication
- Block 6: Persistent Storage
- Block 7: Component Lifecycle

---

## Related Documentation

### Essential References
- **Previous Plans:** `task-004-phase-5-task-5.1-message-correlation-request-response-plan.md` (structure template)
- **ADR-WASM-018:** Three-Layer Architecture
- **ADR-WASM-019:** Runtime Dependency Management ✅ (from Task 5.1)
- **ADR-WASM-006:** Component Isolation and Sandboxing
- **KNOWLEDGE-WASM-019:** Runtime Dependency Architecture (from Task 5.1)

### Architecture Decisions
- ✅ Hooks are WASM-specific (Layer 2 feature)
- ✅ Use Tokio directly for timeouts (ADR-WASM-019)
- ✅ Hooks respect actor isolation (ADR-WASM-006)
- ✅ Default no-op implementations (opt-in, zero overhead)
- ✅ Thread-safe Arc<RwLock<>> for state

### Previous Task Documentation
- Phase 5 Task 5.1: Message Correlation (9.5/10, complete)
- Phase 4 Task 4.3: MessageBroker Integration (complete)
- Phase 3 Task 3.3: Component Health Monitoring (complete)

---

## Notes

**Phase 5 Completion:**
- Task 5.1 ✅ Complete (Message Correlation & Request-Response)
- Task 5.2 ⏳ THIS TASK (Lifecycle Hooks & Custom State)
- After Task 5.2: Phase 5 **100% COMPLETE**

**Block 3 Status:**
- Current: 94% (17/18 tasks)
- After Task 5.2: 100% (18/18 tasks)
- FOUNDATION LAYER COMPLETE

**Quality Maintained:**
- Phase 1-4: 9.5/10 average
- Phase 5 Task 5.1: 9.5/10
- Phase 5 Task 5.2: TARGET 9.5/10

**Production Readiness:**
- After Block 3: Ready for Layer 2 (Blocks 4-7)
- All foundation patterns proven
- All performance targets exceeded
- All quality standards met

