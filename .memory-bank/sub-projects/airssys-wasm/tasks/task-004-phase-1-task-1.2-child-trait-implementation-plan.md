# WASM-TASK-004 Phase 1 Task 1.2: Child Trait WASM Lifecycle Implementation - Detailed Plan

**Generated:** 2025-11-30  
**Status:** ready-to-start  
**Estimated Effort:** 20-25 hours (including testing and validation)  
**Priority:** HIGH - Critical path for actor system integration  

## Overview

This document provides a comprehensive implementation plan for **WASM-TASK-004 Phase 1 Task 1.2**, which implements the `Child` trait WASM lifecycle methods to enable component loading, instantiation, and cleanup with integration to airssys-wasm Block 1 runtime.

## Context

### What's Already Done (Task 1.1 COMPLETE)

**Implemented Components** (1,620 lines):
1. **ComponentActor struct** (`src/actor/component_actor.rs`):
   - Dual trait implementation (Actor + Child from airssys-rt)
   - ActorState enum (7 states: Creating → Starting → Ready → Stopping → Terminated → Failed)
   - ComponentMessage enum (6 message types)
   - HealthStatus enum (3 health states)
   - WasmRuntime stub (ready for integration)

2. **Actor trait stub** (`src/actor/actor_impl.rs`):
   - Basic implementation with TODO markers
   - Ready for Task 1.3 message handling implementation

3. **Child trait stub** (`src/actor/child_impl.rs`):
   - `start()` method stub with TODO markers
   - `stop()` method stub with TODO markers
   - Ready for Task 1.2 WASM lifecycle implementation

**Quality Metrics:**
- Tests: 283 passing (43 new actor tests)
- Warnings: 0 (compiler + clippy)
- Coverage: >90% for actor module
- Code Quality: 9.5/10 (reviewed and approved)

### What Needs Implementation (Task 1.2)

**Primary Objective:**
Implement the `Child` trait WASM lifecycle methods to enable component loading, instantiation, and cleanup with integration to airssys-wasm Block 1 runtime.

**Scope Definition:**

**IN SCOPE for Task 1.2:**
1. **Child::start() Implementation**:
   - Load WASM component bytes from filesystem/memory
   - Create Wasmtime Engine with security configuration
   - Instantiate component with permission manifest validation
   - Apply resource limits (memory, fuel, CPU)
   - Transition state: Creating → Starting → Ready
   - Health check initialization

2. **Child::stop() Implementation**:
   - Graceful shutdown sequence
   - Resource cleanup (engine, store, instance)
   - State transition: Ready → Stopping → Terminated
   - Resource leak prevention

3. **WasmRuntime Integration**:
   - Replace stub with real Wasmtime integration
   - Connect to Block 1 runtime components
   - Use existing ComponentRuntime, ComponentInstance from `src/runtime/`
   - Integrate ResourceLimiter for enforcement

4. **Error Handling**:
   - Component loading errors
   - Instantiation failures
   - Resource limit violations
   - State transition errors

**OUT OF SCOPE for Task 1.2:**
- Actor trait message handling (Task 1.3)
- ActorSystem spawning (Phase 2)
- SupervisorNode integration (Phase 3)
- MessageBroker integration (Phase 4)

### Integration Points

**Existing airssys-wasm Components to Use:**
- `src/runtime/instance.rs`: ComponentInstance struct
- `src/runtime/execution.rs`: ComponentRuntime struct
- `src/core/config.rs`: RuntimeConfig for security settings
- `src/core/manifest.rs`: ComponentManifest, PermissionManifest
- `src/core/limits.rs`: ResourceLimiter trait

**airssys-rt Integration:**
- `Child` trait from airssys-rt (already imported)
- State management using ActorState enum
- Health reporting using HealthStatus enum

### Performance Targets
- **Component spawn time**: <5ms (Creating → Ready)
- **Memory overhead**: <2MB per component instance
- **Shutdown time**: <100ms (Ready → Terminated)
- **Resource cleanup**: Zero leaks (validated with tests)

### Technical Constraints
1. **Thread safety**: All operations must be thread-safe (Arc<Mutex<>> pattern)
2. **Security**: Enforce permission manifest before instantiation
3. **Resource limits**: Apply memory/fuel limits from RuntimeConfig
4. **Error propagation**: Use Result<(), ComponentError> with context
5. **State consistency**: Atomic state transitions with validation

## Reference Documentation

**Primary Guide**: `.memory-bank/sub_projects/airssys-wasm/docs/knowledges/knowledge_wasm_016_actor_system_integration_implementation_guide.md`
- **Section 2.1** (lines 209-437): Child Trait Implementation
- **Code examples**: Lines 238-344 (start()), 346-437 (stop())
- **Testing strategies**: Lines 439-638

**Task Specification**: `.memory-bank/sub_projects/airssys-wasm/tasks/task_004_block_3_actor_system_integration.md`
- **Phase 1 Overview**: Lines 88-133
- **Task 1.2 Details**: Lines 153-206

**Architecture Decisions**:
- **ADR-WASM-006**: Actor integration dual trait pattern
- **ADR-WASM-003**: Security policy enforcement

**Code Standards**:
- `.memory-bank/workspace/shared_patterns.md`: §2.1-§5.1 (imports, modules, dependencies)
- `.memory-bank/workspace/microsoft_rust_guidelines.md`: Production Rust standards

## Current Code Structure

### Files to Modify

**Primary File**: `airssys-wasm/src/actor/child_impl.rs` (400 lines)

Current stub implementation (lines 45-195):
```rust
impl Child for ComponentActor {
    async fn start(&mut self, _ctx: &mut ActorContext) -> Result<(), ComponentError> {
        // TODO(Task 1.2): Implement WASM component loading and instantiation
        // 1. Load WASM bytes from component path
        // 2. Create Wasmtime Engine with security config
        // 3. Instantiate component with permission validation
        // 4. Apply resource limits (memory, fuel)
        // 5. Transition state: Creating → Starting → Ready
        // 6. Initialize health check
        Ok(())
    }

    async fn stop(&mut self, _ctx: &mut ActorContext) -> Result<(), ComponentError> {
        // TODO(Task 1.2): Implement graceful shutdown
        // 1. Transition state: Ready → Stopping
        // 2. Wait for in-flight operations
        // 3. Cleanup WASM instance resources
        // 4. Drop Wasmtime store and engine
        // 5. Transition state: Stopping → Terminated
        // 6. Verify no resource leaks
        Ok(())
    }
}
```

**Supporting Files**:
- `airssys-wasm/src/actor/component_actor.rs`: Update WasmRuntime struct (replace stub)
- `airssys-wasm/src/actor/mod.rs`: Add integration types if needed
- Test files: `airssys-wasm/tests/actor_lifecycle_tests.rs` (new file)

---

## Implementation Plan Breakdown

### Subtask 1: WasmRuntime Integration (4-5 hours)

#### Objectives
- Replace WasmRuntime stub with real Wasmtime types
- Connect to Block 1 runtime components (engine, store, loader)
- Add helper methods for lifecycle management

#### Implementation Steps

**Step 1.1: Define Real WasmRuntime Structure** (60 min)

Location: `airssys-wasm/src/actor/component_actor.rs`

```rust
use wasmtime::{Engine, Store, Instance};
use crate::runtime::limits::ComponentResourceLimiter;

/// WASM runtime encapsulating Wasmtime engine, store, and instance.
pub struct WasmRuntime {
    /// Wasmtime engine (shared across components for module caching)
    engine: Engine,
    
    /// Wasmtime store with resource limiter
    store: Store<ComponentResourceLimiter>,
    
    /// WASM component instance
    instance: Instance,
    
    /// Component exports (cached for fast access)
    exports: WasmExports,
}

/// Cached WASM function exports for common operations.
pub struct WasmExports {
    /// Optional _start export (component initialization)
    pub start: Option<wasmtime::Func>,
    
    /// Optional _cleanup export (graceful shutdown)
    pub cleanup: Option<wasmtime::Func>,
    
    /// Optional _health export (health status reporting)
    pub health: Option<wasmtime::Func>,
    
    /// Optional handle-message export (inter-component messaging)
    pub handle_message: Option<wasmtime::Func>,
}
```

**Step 1.2: Implement WasmRuntime Constructor** (60 min)

```rust
impl WasmRuntime {
    /// Create new WasmRuntime from instantiated component.
    pub fn new(
        engine: Engine,
        store: Store<ComponentResourceLimiter>,
        instance: Instance,
    ) -> Result<Self, WasmError> {
        // Extract and cache exports
        let exports = WasmExports::extract(&instance, &store)?;
        
        Ok(Self {
            engine,
            store,
            instance,
            exports,
        })
    }
    
    /// Get the Wasmtime store (for function calls).
    pub fn store_mut(&mut self) -> &mut Store<ComponentResourceLimiter> {
        &mut self.store
    }
    
    /// Get the WASM instance.
    pub fn instance(&self) -> &Instance {
        &self.instance
    }
    
    /// Get cached exports.
    pub fn exports(&self) -> &WasmExports {
        &self.exports
    }
}
```

**Step 1.3: Implement WasmExports Extraction** (90 min)

```rust
impl WasmExports {
    /// Extract function exports from WASM instance.
    pub fn extract(
        instance: &Instance,
        store: &Store<ComponentResourceLimiter>
    ) -> Result<Self, WasmError> {
        Ok(Self {
            start: instance.get_func(store, "_start"),
            cleanup: instance.get_func(store, "_cleanup"),
            health: instance.get_func(store, "_health"),
            handle_message: instance.get_func(store, "handle-message"),
        })
    }
    
    /// Call _start export if available.
    pub async fn call_start(
        &self,
        store: &mut Store<ComponentResourceLimiter>
    ) -> Result<(), WasmError> {
        if let Some(start_fn) = &self.start {
            start_fn
                .call_async(store, &[], &mut [])
                .await
                .map_err(|e| WasmError::execution_failed(
                    "Component _start function failed",
                    e
                ))?;
        }
        Ok(())
    }
    
    /// Call _cleanup export with timeout.
    pub async fn call_cleanup(
        &self,
        store: &mut Store<ComponentResourceLimiter>,
        timeout: Duration,
    ) -> Result<(), WasmError> {
        if let Some(cleanup_fn) = &self.cleanup {
            tokio::time::timeout(
                timeout,
                cleanup_fn.call_async(store, &[], &mut [])
            )
            .await
            .map_err(|_| WasmError::timeout("Component cleanup timeout"))?
            .map_err(|e| WasmError::execution_failed(
                "Component cleanup function failed",
                e
            ))?;
        }
        Ok(())
    }
}
```

**Step 1.4: Add RAII Drop for WasmRuntime** (30 min)

```rust
impl Drop for WasmRuntime {
    fn drop(&mut self) {
        // Store drop: automatically frees linear memory
        // Engine drop: automatically frees module cache
        debug!("WasmRuntime dropped - all WASM resources freed");
    }
}
```

#### Testing Strategy (Subtask 1)

Location: `airssys-wasm/tests/wasm_runtime_tests.rs` (new file)

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_wasm_runtime_creation() {
        // Test: WasmRuntime can be created from Wasmtime components
        // Validate: All fields properly initialized
    }
    
    #[test]
    fn test_wasm_exports_extraction() {
        // Test: WasmExports extracts available function exports
        // Validate: Optional exports handled correctly
    }
    
    #[test]
    fn test_wasm_runtime_drop() {
        // Test: WasmRuntime drop releases resources
        // Validate: No memory leaks (Valgrind/LSAN if available)
    }
}
```

#### Validation Criteria (Subtask 1)
- ✅ WasmRuntime compiles with real Wasmtime types
- ✅ WasmExports extraction handles missing exports gracefully
- ✅ Drop implementation verified (manual inspection)
- ✅ All tests passing (3 new tests)
- ✅ Zero warnings (compiler + clippy)

---

### Subtask 2: Child::start() Implementation (6-7 hours)

#### Objectives
- Load WASM component bytes
- Create Wasmtime Engine with security configuration
- Instantiate component with ResourceLimiter
- Transition state: Creating → Starting → Ready
- Target: <5ms average spawn time

#### Implementation Steps

**Step 2.1: Load WASM Bytes** (90 min)

Location: `airssys-wasm/src/actor/child_impl.rs`

```rust
#[async_trait]
impl Child for ComponentActor {
    async fn start(&mut self) -> Result<(), Self::Error> {
        self.set_state(ActorState::Starting);
        
        // 1. Load WASM bytes from component metadata
        let wasm_bytes = self.load_component_bytes().await?;
        
        // 2. Validate WASM module magic number
        if !wasm_bytes.starts_with(b"\0asm") {
            return Err(WasmError::component_validation_failed(
                "Invalid WASM module: missing magic number"
            ));
        }
        
        // Continue with engine creation...
    }
}

impl ComponentActor {
    /// Load WASM component bytes (stub - needs storage integration).
    async fn load_component_bytes(&self) -> Result<Vec<u8>, WasmError> {
        // TODO(Block 6): Integrate with ComponentStorage
        // For now, return error indicating storage not implemented
        Err(WasmError::not_implemented(
            "Component storage integration pending (Block 6)"
        ))
    }
}
```

**Step 2.2: Create Wasmtime Engine** (120 min)

```rust
async fn start(&mut self) -> Result<(), Self::Error> {
    // ... (previous: load WASM bytes)
    
    // 2. Create Wasmtime Engine with security configuration
    let mut config = wasmtime::Config::new();
    
    // Enable async execution (required for component execution)
    config.async_support(true);
    
    // Enable multi-value returns (WebAssembly spec)
    config.wasm_multi_value(true);
    
    // Enable fuel metering for CPU limiting
    config.consume_fuel(true);
    
    // Disable features that could bypass security
    config.wasm_bulk_memory(false);
    config.wasm_reference_types(false);
    config.wasm_threads(false);
    
    let engine = wasmtime::Engine::new(&config)
        .map_err(|e| WasmError::engine_creation_failed(e))?;
    
    // Continue with module compilation...
}
```

**Step 2.3: Compile WASM Module** (60 min)

```rust
async fn start(&mut self) -> Result<(), Self::Error> {
    // ... (previous: create engine)
    
    // 3. Compile WASM module
    let module = wasmtime::Module::from_binary(&engine, &wasm_bytes)
        .map_err(|e| WasmError::compilation_failed(
            format!("Component {} compilation failed", self.component_id()),
            e
        ))?;
    
    // Continue with store creation...
}
```

**Step 2.4: Create Store with ResourceLimiter** (90 min)

```rust
async fn start(&mut self) -> Result<(), Self::Error> {
    // ... (previous: compile module)
    
    // 4. Create Store with ResourceLimiter
    let limits = self.metadata.resource_limits.clone();
    let limiter = ComponentResourceLimiter::new(limits);
    
    let mut store = wasmtime::Store::new(&engine, limiter);
    
    // Set fuel limit
    store.set_fuel(limits.max_fuel())
        .map_err(|e| WasmError::invalid_configuration(
            format!("Failed to set fuel limit: {}", e)
        ))?;
    
    // Continue with instantiation...
}
```

**Step 2.5: Instantiate Component** (120 min)

```rust
async fn start(&mut self) -> Result<(), Self::Error> {
    // ... (previous: create store)
    
    // 5. Create linker and register host functions (stub for now)
    let mut linker = wasmtime::Linker::new(&engine);
    
    // TODO(Task 1.3): Register host functions for inter-component communication
    // For now, just basic WASI imports if needed
    
    // 6. Instantiate component
    let instance = linker
        .instantiate_async(&mut store, &module)
        .await
        .map_err(|e| WasmError::instantiation_failed(
            format!("Component {} instantiation failed", self.component_id()),
            e
        ))?;
    
    // Continue with initialization...
}
```

**Step 2.6: Call _start Export and Store Runtime** (90 min)

```rust
async fn start(&mut self) -> Result<(), Self::Error> {
    // ... (previous: instantiate)
    
    // 7. Create WasmRuntime
    let mut runtime = WasmRuntime::new(engine, store, instance)?;
    
    // 8. Call optional _start export
    runtime.exports().call_start(runtime.store_mut()).await?;
    
    // 9. Store runtime for later use
    self.wasm_runtime = Some(runtime);
    self.set_started_at(Some(Utc::now()));
    self.set_state(ActorState::Ready);
    
    // 10. Log successful startup
    info!(
        component_id = %self.component_id(),
        memory_limit = self.metadata.resource_limits.max_memory_bytes(),
        "Component started successfully"
    );
    
    Ok(())
}
```

#### Testing Strategy (Subtask 2)

Location: `airssys-wasm/tests/actor_lifecycle_tests.rs` (new file)

```rust
#[tokio::test]
async fn test_child_start_loads_wasm() {
    // Test: Child::start() loads and instantiates WASM
    // Validate: State transitions correctly, wasm_runtime is Some
}

#[tokio::test]
async fn test_child_start_invalid_wasm() {
    // Test: Child::start() rejects invalid WASM bytes
    // Validate: Returns compilation error, state is Failed
}

#[tokio::test]
async fn test_child_start_resource_limits() {
    // Test: ResourceLimiter enforces memory limits
    // Validate: Memory allocation failures handled gracefully
}

#[tokio::test]
async fn test_child_start_performance() {
    // Test: Component spawn time <5ms average
    // Validate: Measure spawn time over 100 iterations
}

#[tokio::test]
async fn test_child_start_calls_start_export() {
    // Test: _start export is called if available
    // Validate: Component initialization runs successfully
}
```

#### Validation Criteria (Subtask 2)
- ✅ Child::start() loads WASM successfully
- ✅ ResourceLimiter enforces memory limits
- ✅ State transitions: Creating → Starting → Ready
- ✅ Average spawn time <5ms (or documented if higher)
- ✅ 15-20 new tests passing
- ✅ Zero warnings

---

### Subtask 3: Child::stop() Implementation (4-5 hours)

#### Objectives
- Call optional _cleanup export with timeout
- Drop WasmRuntime to free resources
- Transition state: Ready → Stopping → Terminated
- Verify zero resource leaks
- Target: <100ms shutdown time

#### Implementation Steps

**Step 3.1: Graceful Shutdown Sequence** (120 min)

Location: `airssys-wasm/src/actor/child_impl.rs`

```rust
#[async_trait]
impl Child for ComponentActor {
    async fn stop(&mut self, timeout: Duration) -> Result<(), Self::Error> {
        self.set_state(ActorState::Stopping);
        
        // 1. Call optional _cleanup export if WASM loaded
        if let Some(runtime) = &mut self.wasm_runtime {
            // Call _cleanup with timeout protection
            match runtime.exports().call_cleanup(runtime.store_mut(), timeout).await {
                Ok(()) => {
                    info!(
                        component_id = %self.component_id(),
                        "Component cleanup completed successfully"
                    );
                }
                Err(e) if e.is_timeout() => {
                    warn!(
                        component_id = %self.component_id(),
                        timeout_ms = timeout.as_millis(),
                        "Component cleanup timed out"
                    );
                    // Non-fatal: continue with resource cleanup
                }
                Err(e) => {
                    warn!(
                        component_id = %self.component_id(),
                        error = %e,
                        "Component cleanup function failed"
                    );
                    // Non-fatal: continue with resource cleanup
                }
            }
        }
        
        // Continue with resource cleanup...
    }
}
```

**Step 3.2: Resource Cleanup** (90 min)

```rust
async fn stop(&mut self, timeout: Duration) -> Result<(), Self::Error> {
    // ... (previous: call _cleanup)
    
    // 2. Drop WasmRuntime (frees all resources)
    self.clear_wasm_runtime(); // Sets self.wasm_runtime = None
    
    // 3. Verify cleanup completed
    debug_assert!(
        self.wasm_runtime.is_none(),
        "WasmRuntime should be None after stop"
    );
    
    // 4. Transition state
    self.set_state(ActorState::Terminated);
    
    // 5. Log shutdown with uptime metrics
    if let Some(uptime) = self.uptime() {
        info!(
            component_id = %self.component_id(),
            uptime_seconds = uptime.num_seconds(),
            "Component stopped successfully"
        );
    }
    
    Ok(())
}
```

#### Testing Strategy (Subtask 3)

Location: `airssys-wasm/tests/actor_lifecycle_tests.rs`

```rust
#[tokio::test]
async fn test_child_stop_cleans_up() {
    // Test: Child::stop() releases all resources
    // Validate: wasm_runtime is None, state is Terminated
}

#[tokio::test]
async fn test_child_stop_calls_cleanup_export() {
    // Test: _cleanup export is called with timeout
    // Validate: Cleanup function executed
}

#[tokio::test]
async fn test_child_stop_timeout() {
    // Test: Child::stop() handles cleanup timeout gracefully
    // Validate: Returns Ok despite timeout, resources still freed
}

#[tokio::test]
async fn test_child_stop_no_leaks() {
    // Test: Repeated start/stop cycles don't leak memory
    // Validate: Memory usage stable over 100 cycles
}

#[tokio::test]
async fn test_child_stop_performance() {
    // Test: Shutdown time <100ms average
    // Validate: Measure shutdown time over 100 iterations
}
```

#### Validation Criteria (Subtask 3)
- ✅ Child::stop() cleans up all resources
- ✅ State transitions: Ready → Stopping → Terminated
- ✅ Average shutdown time <100ms
- ✅ Zero memory leaks (verified via repeated cycles)
- ✅ 10-15 new tests passing
- ✅ Zero warnings

---

### Subtask 4: Error Handling and Edge Cases (3-4 hours)

#### Objectives
- Handle WASM loading failures gracefully
- Handle instantiation errors with context
- Handle resource limit violations
- Handle state transition errors
- Comprehensive error recovery testing

#### Implementation Steps

**Step 4.1: WASM Loading Error Handling** (90 min)

Location: `airssys-wasm/src/actor/child_impl.rs`

```rust
impl ComponentActor {
    async fn load_component_bytes(&self) -> Result<Vec<u8>, WasmError> {
        // Detailed error context for loading failures
        let bytes = storage::load_component(&self.component_id())
            .await
            .map_err(|e| WasmError::component_load_failed(
                format!("Failed to load component {}: {}", self.component_id(), e),
                e
            ))?;
        
        // Validate bytes not empty
        if bytes.is_empty() {
            return Err(WasmError::component_validation_failed(
                format!("Component {} has zero-length WASM binary", self.component_id())
            ));
        }
        
        Ok(bytes)
    }
}
```

**Step 4.2: Instantiation Error Handling** (90 min)

```rust
async fn start(&mut self) -> Result<(), Self::Error> {
    // Wrap instantiation with detailed error context
    let instance = match linker.instantiate_async(&mut store, &module).await {
        Ok(inst) => inst,
        Err(e) => {
            // Log detailed error
            error!(
                component_id = %self.component_id(),
                error = %e,
                "Component instantiation failed"
            );
            
            // Transition to Failed state
            self.set_state(ActorState::Failed(e.to_string()));
            
            return Err(WasmError::instantiation_failed(
                format!("Component {} instantiation failed: {}", self.component_id(), e),
                e
            ));
        }
    };
    
    // Continue with success path...
}
```

**Step 4.3: Resource Limit Violation Handling** (60 min)

```rust
// Detect OOM during execution (for future Task 1.3)
impl ComponentActor {
    fn check_resource_limits(&self) -> Result<(), WasmError> {
        if let Some(runtime) = &self.wasm_runtime {
            let limiter = runtime.store().data();
            
            if limiter.is_near_oom() {
                warn!(
                    component_id = %self.component_id(),
                    usage_percent = limiter.metrics().usage_percentage(),
                    "Component memory usage >= 90%"
                );
            }
        }
        
        Ok(())
    }
}
```

#### Testing Strategy (Subtask 4)

Location: `airssys-wasm/tests/actor_error_handling_tests.rs` (new file)

```rust
#[tokio::test]
async fn test_start_with_invalid_wasm() {
    // Test: Invalid WASM bytes rejected
    // Validate: Returns compilation error
}

#[tokio::test]
async fn test_start_with_missing_component() {
    // Test: Component not found in storage
    // Validate: Returns load error
}

#[tokio::test]
async fn test_start_memory_limit_exceeded() {
    // Test: Component exceeds memory limit during start
    // Validate: Returns OOM error, state is Failed
}

#[tokio::test]
async fn test_stop_after_failed_start() {
    // Test: stop() called after failed start()
    // Validate: Returns Ok, no panic
}

#[tokio::test]
async fn test_double_start_prevented() {
    // Test: Calling start() twice is prevented
    // Validate: Returns error on second call
}
```

#### Validation Criteria (Subtask 4)
- ✅ All error paths tested
- ✅ Error messages include component_id context
- ✅ Failed state transitions handled correctly
- ✅ 8-10 new error handling tests passing
- ✅ Zero warnings

---

### Subtask 5: Integration Testing and Performance Validation (3-4 hours)

#### Objectives
- End-to-end lifecycle testing
- Performance benchmarking (spawn/shutdown times)
- Memory leak detection (repeated cycles)
- Concurrent component testing
- Documentation updates

#### Implementation Steps

**Step 5.1: End-to-End Lifecycle Tests** (120 min)

Location: `airssys-wasm/tests/actor_integration_tests.rs` (new file)

```rust
#[tokio::test]
async fn test_full_lifecycle_start_stop() {
    // Test: Complete lifecycle from Creating to Terminated
    // Validate: All state transitions correct
}

#[tokio::test]
async fn test_multiple_start_stop_cycles() {
    // Test: 100 start/stop cycles
    // Validate: No memory leaks, stable performance
}

#[tokio::test]
async fn test_concurrent_component_starts() {
    // Test: Start 10 components concurrently
    // Validate: All succeed without conflicts
}
```

**Step 5.2: Performance Benchmarking** (90 min)

Location: `airssys-wasm/benches/actor_performance.rs` (new file, optional)

```rust
#[tokio::test]
async fn bench_component_spawn_time() {
    // Benchmark: Measure spawn time over 100 iterations
    // Target: <5ms average
    // Report: P50, P95, P99 latencies
}

#[tokio::test]
async fn bench_component_shutdown_time() {
    // Benchmark: Measure shutdown time over 100 iterations
    // Target: <100ms average
    // Report: P50, P95, P99 latencies
}
```

**Step 5.3: Memory Leak Detection** (60 min)

Location: `airssys-wasm/tests/actor_memory_leak_tests.rs` (new file)

```rust
#[tokio::test]
async fn test_no_memory_leaks_repeated_cycles() {
    // Test: 1000 start/stop cycles
    // Validate: Memory usage stable (if Valgrind/LSAN available)
    // Manual: Monitor process memory with `ps` or `/proc/self/status`
}
```

**Step 5.4: Documentation Updates** (60 min)

Update the following files:
- `.memory-bank/sub_projects/airssys-wasm/progress.md` - Add Task 1.2 completion entry
- `.memory-bank/sub_projects/airssys-wasm/active_context.md` - Update to Task 1.3 focus
- `airssys-wasm/src/actor/child_impl.rs` - Add rustdoc examples
- `.memory-bank/sub_projects/airssys-wasm/tasks/task_004_block_3_actor_system_integration.md` - Mark Task 1.2 complete

#### Validation Criteria (Subtask 5)
- ✅ All integration tests passing
- ✅ Spawn time <5ms average (or documented)
- ✅ Shutdown time <100ms average
- ✅ No memory leaks detected
- ✅ Documentation updated
- ✅ 5-10 new integration tests passing

---

## Summary of Deliverables

### Code Implementation
1. **WasmRuntime** - Real Wasmtime integration (~150 lines)
2. **Child::start()** - Full WASM loading implementation (~200 lines)
3. **Child::stop()** - Complete resource cleanup (~150 lines)
4. **Error Handling** - Comprehensive error paths (~100 lines)
5. **Testing** - Lifecycle, errors, performance, leaks (~400 lines)

**Total Implementation**: ~1,000 lines of production code + tests

### Quality Gates
- ✅ Zero warnings (compiler + clippy)
- ✅ All tests passing (existing 283 + ~50 new = ~333 total)
- ✅ Performance validated (<5ms spawn, <100ms shutdown)
- ✅ No memory leaks detected
- ✅ Code review 9.0+ rating
- ✅ 100% workspace standards compliance (§2.1-§5.1)

### Documentation Updates
- Update `progress.md` with Task 1.2 completion
- Update `active_context.md` to Task 1.3 focus
- Add rustdoc examples to new methods
- Update task file with implementation notes

---

## Effort Estimates

| Subtask | Estimated Hours | Validation Time | Total |
|---------|----------------|-----------------|-------|
| 1. WasmRuntime Integration | 4-5 hours | 1 hour | 5-6 hours |
| 2. Child::start() Implementation | 6-7 hours | 1.5 hours | 7.5-8.5 hours |
| 3. Child::stop() Implementation | 4-5 hours | 1 hour | 5-6 hours |
| 4. Error Handling | 3-4 hours | 1 hour | 4-5 hours |
| 5. Integration Testing | 3-4 hours | 1 hour | 4-5 hours |
| **Total** | **20-25 hours** | **5.5 hours** | **25.5-30.5 hours** |

**Recommended Approach**: Implement incrementally (1 subtask per session), run tests after each, adjust based on actual complexity.

---

## Risk Mitigation

### Risk 1: Block 1 Runtime Integration Complexity
**Impact**: HIGH  
**Probability**: MEDIUM  
**Mitigation**: 
- Use existing `runtime/` module files (engine.rs, loader.rs, limits.rs)
- Reference KNOWLEDGE-WASM-016 lines 209-437 for patterns
- Start with minimal integration, expand incrementally

### Risk 2: Performance Not Meeting Targets (<5ms spawn)
**Impact**: MEDIUM  
**Probability**: MEDIUM  
**Mitigation**: 
- Profile early with `tokio-console` or `perf`
- Optimize WASM loading path (async I/O, caching)
- Consider module caching (already in Block 1 design)
- Document actual performance if target not met initially

### Risk 3: Resource Leak Detection Difficulty
**Impact**: HIGH  
**Probability**: LOW  
**Mitigation**: 
- Use repeated start/stop cycles (1000x)
- Monitor with process memory tools (`ps`, `/proc/self/status`)
- Integrate Valgrind/LSAN if available
- Document leak detection strategy in tests

### Risk 4: WASM Component Storage Not Ready (Block 6 dependency)
**Impact**: LOW  
**Probability**: HIGH (expected)  
**Mitigation**: 
- Create stub `load_component_bytes()` returning test fixture bytes
- Use in-memory test WASM binaries for now
- Document Block 6 integration point clearly with TODO comments
- Ensure abstraction allows easy swap later

### Risk 5: Wasmtime API Changes or Incompatibilities
**Impact**: MEDIUM  
**Probability**: LOW  
**Mitigation**: 
- Pin Wasmtime version in Cargo.toml
- Reference Wasmtime documentation for stable APIs
- Test with actual WASM binaries early
- Keep Wasmtime integration isolated in WasmRuntime

---

## Open Questions for Clarification

### Question 1: WASM Component Storage
**Context**: Block 6 (Component Storage) not implemented yet.  
**Question**: Should I use test fixtures for now, or implement a minimal in-memory storage stub?  
**Options**:
- A) Use hardcoded test WASM bytes in `load_component_bytes()` stub
- B) Create minimal `ComponentStorage` trait with in-memory implementation
- C) Load from filesystem (tests/fixtures/*.wasm) temporarily

**Recommendation**: Option A (simplest, unblocks Task 1.2 immediately)

### Question 2: Module Caching
**Context**: Block 1 has `engine.rs` with module caching logic.  
**Question**: Should I integrate module caching in Task 1.2, or defer to optimization phase?  
**Options**:
- A) Integrate now (better performance, more complexity)
- B) Defer to Phase 5 performance optimization (simpler Task 1.2)

**Recommendation**: Option B (keep Task 1.2 focused, optimize later)

### Question 3: Host Functions Registration
**Context**: Task 1.3 will handle message routing via Actor trait.  
**Question**: Should I add empty `Linker` registration in Task 1.2, or leave it completely stubbed?  
**Options**:
- A) Add empty linker with TODO comments
- B) Add basic host function stubs (e.g., `log`, `get_env`)
- C) Leave completely empty until Task 1.3

**Recommendation**: Option A (shows integration point, doesn't block Task 1.2)

### Question 4: Performance Target Flexibility
**Context**: <5ms spawn time may be challenging with WASM compilation overhead.  
**Question**: If spawn time exceeds 5ms, is it acceptable to document actual performance with optimization plan?  
**Options**:
- A) Block Task 1.2 completion until <5ms achieved
- B) Document actual performance, create technical debt for optimization
- C) Adjust target based on initial measurements

**Recommendation**: Option B (unblock progress, track optimization separately)

### Question 5: Testing Fixtures
**Context**: Need WASM binaries for testing.  
**Question**: Do you have example WASM binaries (.wasm files), or should I create minimal test components?  
**Options**:
- A) Use existing test fixtures (if available)
- B) Create minimal Rust→WASM test components
- C) Use pre-built WASM from external sources

**Recommendation**: Option B (full control, matches production use cases)

---

## Success Criteria

### Functional Requirements
- ✅ Child::start() successfully loads and instantiates WASM components
- ✅ Child::stop() cleanly shuts down and frees all resources
- ✅ State transitions follow documented state machine (Creating → Starting → Ready → Stopping → Terminated)
- ✅ Error handling covers all failure modes (loading, compilation, instantiation, OOM)
- ✅ ResourceLimiter enforces memory and fuel limits

### Performance Requirements
- ✅ Average component spawn time <5ms (or documented with optimization plan)
- ✅ Average component shutdown time <100ms
- ✅ Memory overhead <2MB per component instance
- ✅ Zero memory leaks (1000 start/stop cycles stable)

### Quality Requirements
- ✅ Zero compiler warnings
- ✅ Zero clippy warnings
- ✅ All existing tests passing (283 tests)
- ✅ All new tests passing (~50 new tests)
- ✅ >90% code coverage for new code
- ✅ Code review rating ≥9.0/10
- ✅ 100% workspace standards compliance (§2.1-§5.1)

### Documentation Requirements
- ✅ Rustdoc for all public methods
- ✅ Code examples in rustdoc
- ✅ Memory bank updated (progress.md, active_context.md, task_004)
- ✅ Implementation notes documenting key decisions

---

## Next Steps After Task 1.2

### Task 1.3: Actor Trait Message Handling (16-20 hours)
**Objectives**:
- Implement Actor::handle_message() for component messages
- Add message routing logic (ComponentMessage variants)
- Integrate host function calls from WASM
- Add message serialization/deserialization
- Test inter-component communication patterns

**Prerequisites**: Task 1.2 COMPLETE (Child trait implemented)

### Phase 2: ActorSystem Integration (12-16 hours)
**Objectives**:
- Implement ActorSystem::spawn() for ComponentActor
- Add component registry (tracking active components)
- Implement component lifecycle management
- Add spawn performance optimization
- Test concurrent component spawning

**Prerequisites**: Phase 1 COMPLETE (ComponentActor fully functional)

---

## References

### Primary Documentation
1. **KNOWLEDGE-WASM-016** - Actor System Integration Implementation Guide
   - Location: `.memory-bank/sub_projects/airssys-wasm/docs/knowledges/knowledge_wasm_016_actor_system_integration_implementation_guide.md`
   - Section 2.1 (lines 209-437): Child Trait Implementation
   - Section 3.1 (lines 439-638): Testing Strategies

2. **WASM-TASK-004** - Block 3: Actor System Integration
   - Location: `.memory-bank/sub_projects/airssys-wasm/tasks/task_004_block_3_actor_system_integration.md`
   - Phase 1 Overview (lines 88-133)
   - Task 1.2 Specification (lines 153-206)

### Architecture Decisions
1. **ADR-WASM-006** - Component Isolation and Sandboxing (Actor-based approach)
2. **ADR-WASM-003** - Security Policy Enforcement (Permission manifest validation)
3. **ADR-WASM-010** - Actor System Integration Promotion (Block 9 → Block 3)

### Code Standards
1. **Workspace Shared Patterns** - `.memory-bank/workspace/shared_patterns.md`
   - §2.1: 3-Layer Import Organization
   - §3.2: chrono DateTime<Utc> Standard
   - §4.3: Module Architecture
   - §5.1: Dependency Management

2. **Microsoft Rust Guidelines** - `.memory-bank/workspace/microsoft_rust_guidelines.md`
   - M-DESIGN-FOR-AI: AI-optimized development
   - M-ERRORS-CANONICAL-STRUCTS: Structured errors
   - M-SERVICES-CLONE: Services implement cheap Clone

### External Documentation
1. **Wasmtime Documentation** - https://docs.wasmtime.dev/
2. **airssys-rt Documentation** - `airssys-rt/docs/`
3. **WebAssembly Specification** - https://webassembly.github.io/spec/

---

**End of Implementation Plan**

---

## Plan Metadata

- **Generated By**: @task.plans agent
- **Session ID**: ses_52bdb4978ffeGseg5iBxr7WU77
- **Generated Date**: 2025-11-30
- **Plan Status**: ready-to-start
- **Target Completion**: TBD (20-25 hours estimated)
- **Assignee**: TBD
- **Reviewer**: TBD
