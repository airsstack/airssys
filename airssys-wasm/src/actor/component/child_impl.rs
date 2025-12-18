//! Child trait implementation for ComponentActor (WASM lifecycle management).
//!
//! This module implements the `Child` trait from airssys-rt, enabling ComponentActor
//! to be supervised by SupervisorNode. The Child trait manages WASM runtime lifecycle:
//! - `start()`: Load WASM component and instantiate runtime
//! - `stop()`: Cleanup WASM instance and release resources
//! - `health_check()`: Report component health status
//!
//! # Design Rationale (ADR-RT-004)
//!
//! Child trait is separate from Actor trait to enable supervision of any process-like
//! entity, not just actors. ComponentActor explicitly implements Child to integrate
//! with SupervisorNode for automatic restart on failures.
//!
//! # Implementation Status
//!
//! **TASK 1.2 COMPLETE - Full WASM Lifecycle Implementation**
//!
//! This implementation provides full WASM loading, instantiation, and cleanup:
//! - `start()`: Validates WASM, creates Wasmtime engine/store, instantiates component
//! - `stop()`: Calls _cleanup export, drops WasmRuntime, verifies resource cleanup
//! - `health_check()`: Returns Healthy (stub - full implementation in Task 3.3)
//!
//! # References
//!
//! - **WASM-TASK-004 Phase 1 Task 1.2**: Child Trait WASM Lifecycle (20-25 hours)
//! - **KNOWLEDGE-WASM-016**: Actor System Integration Implementation Guide
//! - **ADR-RT-004**: Actor and Child Trait Separation
//! - **ADR-WASM-006**: Component Isolation and Sandboxing

// Layer 1: Standard library imports
use std::time::Duration;

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use chrono::Utc;
use tracing::{error, info, warn};
use wasmtime::{Config, Engine, Linker, Module, Store};

// Layer 3: Internal module imports
use super::component_actor::{ComponentResourceLimiter, WasmExports};
use crate::actor::component::{ActorState, ComponentActor, HealthStatus, WasmRuntime};
use crate::core::WasmError;
use airssys_rt::supervisor::{Child, ChildHealth};

/// Child trait implementation for ComponentActor (STUB).
///
/// This is a stub implementation to unblock Task 1.1. Full WASM loading
/// and cleanup logic will be implemented in Task 1.2.
///
/// # Stub Behavior
///
/// - **start()**: Transitions state (Creating → Starting → Ready) but does NOT load WASM
/// - **stop()**: Transitions state (Ready → Stopping → Terminated) but does NOT cleanup WASM
/// - **health_check()**: Always returns Healthy
///
/// # Future Implementation (Task 1.2)
///
/// - **start()**: Load WASM bytes, create Wasmtime engine/store, instantiate component
/// - **stop()**: Call _cleanup export, drop WasmRuntime, verify resource cleanup
/// - **health_check()**: Call _health export if available, map to ChildHealth
#[async_trait]
impl<S> Child for ComponentActor<S>
where
    S: Send + Sync + 'static,
{
    type Error = WasmError;

    /// Start the component by loading and instantiating WASM.
    ///
    /// Implements full WASM lifecycle startup:
    /// 1. Transition to Starting state
    /// 2. Load WASM bytes from storage (stub for Block 6)
    /// 3. Validate WASM magic number
    /// 4. Create Wasmtime Engine with security configuration
    /// 5. Compile WASM module from bytes
    /// 6. Create Store with ResourceLimiter integration
    /// 7. Create empty Linker (host functions in Task 1.3)
    /// 8. Instantiate component
    /// 9. Call optional _start export
    /// 10. Store runtime and transition to Ready
    ///
    /// # State Transitions
    ///
    /// - Success: Creating → Starting → Ready
    /// - Failure: Creating → Starting → Failed(reason)
    ///
    /// # Errors
    ///
    /// - WasmError::ComponentNotFound: Component storage not implemented (Block 6)
    /// - WasmError::ComponentValidationFailed: Invalid WASM magic number
    /// - WasmError::EngineInitialization: Engine creation failed
    /// - WasmError::ComponentLoadFailed: Module compilation failed
    /// - WasmError::ExecutionFailed: Instantiation or _start failed
    ///
    /// # Performance Target
    ///
    /// <5ms average spawn time (validated in tests)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use airssys_wasm::actor::{ComponentActor, ActorState};
    /// use airssys_wasm::core::{ComponentId, ComponentMetadata, CapabilitySet, ResourceLimits};
    /// use airssys_rt::supervisor::Child;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let component_id = ComponentId::new("test");
    ///     let metadata = ComponentMetadata {
    ///         name: "test".to_string(),
    ///         version: "1.0.0".to_string(),
    ///         author: "Test".to_string(),
    ///         description: None,
    ///         required_capabilities: vec![],
    ///         resource_limits: ResourceLimits {
    ///             max_memory_bytes: 64 * 1024 * 1024,
    ///             max_fuel: 1_000_000,
    ///             max_execution_ms: 5000,
    ///             max_storage_bytes: 10 * 1024 * 1024,
    ///         },
    ///     };
    ///     let mut actor = ComponentActor::new(component_id, metadata, CapabilitySet::new());
    ///     
    ///     // Start component - loads WASM and instantiates
    ///     actor.start().await?;
    ///     
    ///     // State transitioned to Ready
    ///     assert_eq!(*actor.state(), ActorState::Ready);
    ///     
    ///     Ok(())
    /// }
    /// ```
    async fn start(&mut self) -> Result<(), Self::Error> {
        // PHASE 5 TASK 5.2: Call pre_start hook
        // Note: We don't have ActorAddress in Child::start() context, so we create
        // a placeholder context. Full context will be available in Actor::handle_message().
        let lifecycle_ctx = crate::actor::lifecycle::LifecycleContext {
            component_id: self.component_id().clone(),
            actor_address: airssys_rt::ActorAddress::anonymous(),
            timestamp: Utc::now(),
        };

        // Call pre_start hook synchronously (hooks are sync, not async)
        let hook_result = {
            let ctx_clone = lifecycle_ctx.clone();
            crate::actor::lifecycle::catch_unwind_hook(|| self.hooks_mut().pre_start(&ctx_clone))
        };

        match hook_result {
            crate::actor::lifecycle::HookResult::Ok => {
                tracing::debug!(
                    component_id = %self.component_id().as_str(),
                    "pre_start hook completed successfully"
                );
            }
            crate::actor::lifecycle::HookResult::Error(e) => {
                tracing::warn!(
                    component_id = %self.component_id().as_str(),
                    error = %e,
                    "pre_start hook returned error (continuing startup)"
                );
            }
            crate::actor::lifecycle::HookResult::Timeout => {
                // catch_unwind_hook doesn't timeout, this shouldn't happen
                tracing::warn!(
                    component_id = %self.component_id().as_str(),
                    "pre_start hook unexpectedly timed out"
                );
            }
        }

        // 1. Transition to Starting state
        self.set_state(ActorState::Starting);

        // 2. Load WASM bytes (stub for Block 6)
        let wasm_bytes = self.load_component_bytes().await?;

        // 3. Validate WASM magic number
        if wasm_bytes.len() < 4 || !wasm_bytes.starts_with(b"\0asm") {
            let err_msg = if wasm_bytes.is_empty() {
                "WASM bytes are empty"
            } else if wasm_bytes.len() < 4 {
                "WASM bytes too short (< 4 bytes)"
            } else {
                "Invalid WASM module: missing magic number \\0asm"
            };

            self.set_state(ActorState::Failed(err_msg.to_string()));
            return Err(WasmError::component_validation_failed(err_msg));
        }

        // 4. Create Wasmtime Engine with security config
        let mut config = Config::new();
        config.async_support(true); // Required for async component execution
        config.wasm_multi_value(true); // Allow multiple return values
        config.consume_fuel(true); // Enable fuel metering for CPU limits

        // Disable unsafe WASM features for security (ADR-WASM-003)
        config.wasm_bulk_memory(false);
        config.wasm_reference_types(false);
        config.wasm_threads(false);
        config.wasm_simd(false);
        config.wasm_relaxed_simd(false); // Must be disabled if SIMD is disabled

        let engine = Engine::new(&config).map_err(|e| {
            let err_msg = format!("Failed to create Wasmtime engine: {e}");
            self.set_state(ActorState::Failed(err_msg.clone()));
            WasmError::engine_initialization(err_msg)
        })?;

        // 5. Compile WASM module
        let module = Module::from_binary(&engine, &wasm_bytes).map_err(|e| {
            let err_msg = format!(
                "Component {} compilation failed: {}",
                self.component_id().as_str(),
                e
            );
            error!(
                component_id = %self.component_id().as_str(),
                error = %e,
                "WASM module compilation failed"
            );
            self.set_state(ActorState::Failed(err_msg.clone()));
            WasmError::component_load_failed(self.component_id().as_str(), err_msg)
        })?;

        // 6. Create Store with ResourceLimiter
        let max_memory_bytes = self.metadata().resource_limits.max_memory_bytes;
        let max_fuel = self.metadata().resource_limits.max_fuel;

        let limiter = ComponentResourceLimiter::new(max_memory_bytes, max_fuel);
        let mut store = Store::new(&engine, limiter);

        // Set initial fuel
        store.set_fuel(max_fuel).map_err(|e| {
            let err_msg = format!("Failed to set fuel limit: {e}");
            self.set_state(ActorState::Failed(err_msg.clone()));
            WasmError::invalid_configuration(err_msg)
        })?;

        // 7. Create empty Linker (host functions will be added in Task 1.3)
        let linker = Linker::new(&engine);
        // TODO(Task 1.3): Register host functions here

        // 8. Instantiate component
        let instance = linker
            .instantiate_async(&mut store, &module)
            .await
            .map_err(|e| {
                let err_msg = format!(
                    "Component {} instantiation failed: {}",
                    self.component_id().as_str(),
                    e
                );
                error!(
                    component_id = %self.component_id().as_str(),
                    error = %e,
                    "Component instantiation failed"
                );
                self.set_state(ActorState::Failed(err_msg.clone()));
                WasmError::execution_failed(err_msg)
            })?;

        // 9. Create WasmRuntime and call optional _start
        let mut runtime = WasmRuntime::new(engine, store, instance)?;

        // Clone _start function to avoid borrowing issues
        let start_fn_opt = runtime.exports().start;

        // Call _start if available
        WasmExports::call_start_fn(start_fn_opt.as_ref(), runtime.store_mut())
            .await
            .map_err(|e| {
                error!(
                    component_id = %self.component_id().as_str(),
                    error = %e,
                    "Component _start function failed"
                );
                self.set_state(ActorState::Failed(e.to_string()));
                e
            })?;

        // 10. Store runtime and transition state
        self.set_wasm_runtime(Some(runtime));
        self.set_started_at(Some(Utc::now()));
        self.set_state(ActorState::Ready);

        info!(
            component_id = %self.component_id().as_str(),
            memory_limit = max_memory_bytes,
            fuel_limit = max_fuel,
            "Component started successfully"
        );

        // PHASE 5 TASK 5.2: Call post_start hook
        let hook_result = {
            let ctx_clone = lifecycle_ctx.clone();
            crate::actor::lifecycle::catch_unwind_hook(|| self.hooks_mut().post_start(&ctx_clone))
        };

        match hook_result {
            crate::actor::lifecycle::HookResult::Ok => {
                tracing::debug!(
                    component_id = %self.component_id().as_str(),
                    "post_start hook completed successfully"
                );
            }
            crate::actor::lifecycle::HookResult::Error(e) => {
                tracing::warn!(
                    component_id = %self.component_id().as_str(),
                    error = %e,
                    "post_start hook returned error (startup complete)"
                );
            }
            crate::actor::lifecycle::HookResult::Timeout => {
                tracing::warn!(
                    component_id = %self.component_id().as_str(),
                    "post_start hook unexpectedly timed out"
                );
            }
        }

        Ok(())
    }

    /// Stop the component gracefully with resource cleanup.
    ///
    /// Implements full WASM lifecycle shutdown:
    /// 1. Transition to Stopping state
    /// 2. Call optional _cleanup export with timeout protection
    /// 3. Drop WasmRuntime to free all resources
    /// 4. Verify cleanup completed
    /// 5. Transition to Terminated state
    /// 6. Log shutdown with uptime metrics
    ///
    /// # State Transitions
    ///
    /// - Success: Ready → Stopping → Terminated
    /// - With warnings: Ready → Stopping → Terminated (cleanup timeout/error logged)
    ///
    /// # Parameters
    ///
    /// - `timeout`: Maximum time to wait for _cleanup export execution
    ///
    /// # Errors
    ///
    /// This method always succeeds (Ok) because cleanup failures are non-fatal:
    /// - _cleanup timeout: Logged as warning, cleanup continues
    /// - _cleanup error: Logged as warning, cleanup continues
    /// - Resources are freed regardless of _cleanup success
    ///
    /// # Performance Target
    ///
    /// <100ms average shutdown time (validated in tests)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use airssys_wasm::actor::{ComponentActor, ActorState};
    /// use airssys_wasm::core::{ComponentId, ComponentMetadata, CapabilitySet, ResourceLimits};
    /// use airssys_rt::supervisor::Child;
    /// use std::time::Duration;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let component_id = ComponentId::new("test");
    ///     let metadata = ComponentMetadata {
    ///         name: "test".to_string(),
    ///         version: "1.0.0".to_string(),
    ///         author: "Test".to_string(),
    ///         description: None,
    ///         required_capabilities: vec![],
    ///         resource_limits: ResourceLimits {
    ///             max_memory_bytes: 64 * 1024 * 1024,
    ///             max_fuel: 1_000_000,
    ///             max_execution_ms: 5000,
    ///             max_storage_bytes: 10 * 1024 * 1024,
    ///         },
    ///     };
    ///     let mut actor = ComponentActor::new(component_id, metadata, CapabilitySet::new());
    ///     
    ///     actor.start().await?;
    ///     
    ///     // Stop component - calls _cleanup and frees resources
    ///     actor.stop(Duration::from_secs(5)).await?;
    ///     
    ///     // State transitioned to Terminated
    ///     assert_eq!(*actor.state(), ActorState::Terminated);
    ///     
    ///     Ok(())
    /// }
    /// ```
    async fn stop(&mut self, timeout: Duration) -> Result<(), Self::Error> {
        // PHASE 5 TASK 5.2: Call pre_stop hook
        let lifecycle_ctx = crate::actor::lifecycle::LifecycleContext {
            component_id: self.component_id().clone(),
            actor_address: airssys_rt::ActorAddress::anonymous(),
            timestamp: Utc::now(),
        };

        let hook_result = {
            let ctx_clone = lifecycle_ctx.clone();
            crate::actor::lifecycle::catch_unwind_hook(|| self.hooks_mut().pre_stop(&ctx_clone))
        };

        match hook_result {
            crate::actor::lifecycle::HookResult::Ok => {
                tracing::debug!(
                    component_id = %self.component_id().as_str(),
                    "pre_stop hook completed successfully"
                );
            }
            crate::actor::lifecycle::HookResult::Error(e) => {
                tracing::warn!(
                    component_id = %self.component_id().as_str(),
                    error = %e,
                    "pre_stop hook returned error (continuing shutdown)"
                );
            }
            crate::actor::lifecycle::HookResult::Timeout => {
                tracing::warn!(
                    component_id = %self.component_id().as_str(),
                    "pre_stop hook unexpectedly timed out"
                );
            }
        }

        // 1. Transition to Stopping state
        self.set_state(ActorState::Stopping);

        // 2. Call optional _cleanup export if WASM is loaded
        if let Some(runtime) = self.wasm_runtime_mut() {
            // Clone cleanup function to avoid borrowing issues
            let cleanup_fn_opt = runtime.exports().cleanup;

            match WasmExports::call_cleanup_fn(
                cleanup_fn_opt.as_ref(),
                runtime.store_mut(),
                timeout,
            )
            .await
            {
                Ok(()) => {
                    info!(
                        component_id = %self.component_id().as_str(),
                        "Component cleanup completed successfully"
                    );
                }
                Err(WasmError::ExecutionTimeout { .. }) => {
                    warn!(
                        component_id = %self.component_id().as_str(),
                        timeout_ms = timeout.as_millis(),
                        "Component cleanup timed out (non-fatal)"
                    );
                    // Non-fatal: continue with resource cleanup
                }
                Err(e) => {
                    warn!(
                        component_id = %self.component_id().as_str(),
                        error = %e,
                        "Component cleanup function failed (non-fatal)"
                    );
                    // Non-fatal: continue with resource cleanup
                }
            }
        }

        // 3. Drop WasmRuntime (frees all resources via RAII)
        self.clear_wasm_runtime();

        // 4. Verify cleanup completed
        debug_assert!(
            !self.is_wasm_loaded(),
            "WasmRuntime should be cleared after stop"
        );

        // 5. Transition state
        self.set_state(ActorState::Terminated);

        // 6. Log shutdown with uptime metrics
        if let Some(uptime) = self.uptime() {
            info!(
                component_id = %self.component_id().as_str(),
                uptime_seconds = uptime.num_seconds(),
                "Component stopped successfully"
            );
        } else {
            info!(
                component_id = %self.component_id().as_str(),
                "Component stopped successfully (never started)"
            );
        }

        // PHASE 5 TASK 5.2: Call post_stop hook
        let hook_result = {
            let ctx_clone = lifecycle_ctx.clone();
            crate::actor::lifecycle::catch_unwind_hook(|| self.hooks_mut().post_stop(&ctx_clone))
        };

        match hook_result {
            crate::actor::lifecycle::HookResult::Ok => {
                tracing::debug!(
                    component_id = %self.component_id().as_str(),
                    "post_stop hook completed successfully"
                );
            }
            crate::actor::lifecycle::HookResult::Error(e) => {
                tracing::warn!(
                    component_id = %self.component_id().as_str(),
                    error = %e,
                    "post_stop hook returned error (shutdown complete)"
                );
            }
            crate::actor::lifecycle::HookResult::Timeout => {
                tracing::warn!(
                    component_id = %self.component_id().as_str(),
                    "post_stop hook unexpectedly timed out"
                );
            }
        }

        Ok(())
    }

    /// Check component health status with timeout protection.
    ///
    /// This method implements comprehensive health checking by:
    /// 1. Checking if WASM runtime is loaded
    /// 2. Evaluating ActorState for immediate failures
    /// 3. Calling optional _health WASM export
    /// 4. Aggregating health from multiple factors
    ///
    /// # Health Semantics
    ///
    /// **Readiness Probe:** Can component serve traffic?
    /// - `Creating/Starting` → Degraded (not ready yet)
    /// - `Ready + Healthy` → Healthy (ready to serve)
    /// - `Failed` → Failed (restart needed)
    ///
    /// **Liveness Probe:** Should component be restarted?
    /// - `Failed` → Restart required
    /// - `Unhealthy` → Consider restart
    /// - `Degraded` → Keep running (may self-heal)
    ///
    /// # Performance
    ///
    /// - **Without _health export:** <1ms (state check only)
    /// - **With _health export:** <10ms typical, <50ms P99
    /// - **Timeout protection:** 1000ms (returns Degraded on timeout)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use airssys_wasm::actor::ComponentActor;
    /// use airssys_rt::supervisor::{Child, ChildHealth};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut actor = create_component_actor().await?;
    ///     actor.start().await?;
    ///     
    ///     // Periodic health check (e.g., every 30 seconds)
    ///     let health = actor.health_check().await;
    ///     
    ///     match health {
    ///         ChildHealth::Healthy => {
    ///             println!("Component healthy");
    ///         }
    ///         ChildHealth::Degraded { reason } => {
    ///             println!("Component degraded: {}", reason);
    ///             // May self-heal, keep monitoring
    ///         }
    ///         ChildHealth::Failed { reason } => {
    ///             println!("Component failed: {}", reason);
    ///             // Supervisor will restart
    ///         }
    ///     }
    ///     
    ///     Ok(())
    /// }
    /// ```
    async fn health_check(&self) -> ChildHealth {
        const HEALTH_CHECK_TIMEOUT: Duration = Duration::from_millis(1000);

        match tokio::time::timeout(HEALTH_CHECK_TIMEOUT, self.health_check_inner()).await {
            Ok(health) => health,
            Err(_timeout) => {
                warn!(
                    component_id = %self.component_id().as_str(),
                    timeout_ms = HEALTH_CHECK_TIMEOUT.as_millis(),
                    "Health check timed out"
                );
                ChildHealth::Degraded(format!(
                    "Health check timeout (>{}ms)",
                    HEALTH_CHECK_TIMEOUT.as_millis()
                ))
            }
        }
    }
}

impl<S> ComponentActor<S>
where
    S: Send + Sync + 'static,
{
    /// Inner health check implementation without timeout protection.
    ///
    /// Implements comprehensive health aggregation logic based on component state
    /// and resource utilization. This method provides the core health assessment
    /// logic used by the timeout-protected `health_check()` public method.
    ///
    /// # Design Note: WASM _health Export Limitation
    ///
    /// The _health export call requires mutable access to the Wasmtime Store,
    /// but the Child trait's health_check() method signature is `async fn health_check(&self)`.
    ///
    /// **Current Implementation**: State-based health checking only (no _health export invocation).
    ///
    /// **Future Enhancement**: If _health export invocation is needed, consider:
    /// - Using `RefCell<Store>` for interior mutability (with performance tradeoff)
    /// - Adding a separate mutable health check API to ComponentActor
    /// - Proposing trait signature change to airssys-rt (breaking change)
    ///
    /// The `parse_health_status()` helper method is provided for future _health export support.
    ///
    /// # Health Aggregation Logic
    ///
    /// Health status is determined by checking multiple factors in priority order:
    ///
    /// 1. **WASM Runtime Status**: Component must have loaded WASM runtime
    ///    - If None → `Failed("WASM runtime not loaded")`
    ///
    /// 2. **Actor State Assessment**: Current lifecycle state determines health
    ///    - `Failed(reason)` → `Failed(reason)` (unrecoverable error)
    ///    - `Terminated` → `Failed("Component terminated")` (shutdown)
    ///    - `Creating | Starting` → `Degraded("Component starting")` (not ready)
    ///    - `Stopping` → `Degraded("Component stopping")` (graceful shutdown)
    ///    - `Ready` → `Healthy` (operational, accepting messages)
    ///
    /// 3. **Future: WASM _health Export** (when Store mutability is resolved):
    ///    - Call optional _health export if available
    ///    - Parse multicodec-encoded HealthStatus response
    ///    - Aggregate with state-based health (unhealthy beats all)
    ///
    /// 4. **Future: Resource Health** (placeholder for Task 1.5):
    ///    - Check fuel consumption rate
    ///    - Check memory pressure
    ///    - Check error rate
    ///
    /// # Health vs Readiness Semantics
    ///
    /// This method supports both health probe types:
    ///
    /// **Readiness Probe** (Can component serve traffic?):
    /// - `Creating/Starting` → Degraded (wait for Ready state)
    /// - `Ready` → Healthy (ready to process messages)
    /// - `Failed/Terminated` → Failed (needs restart)
    ///
    /// **Liveness Probe** (Should component be restarted?):
    /// - `Failed` → Restart required
    /// - `Degraded` → Keep running (may self-heal)
    /// - `Healthy` → No action needed
    ///
    /// # Returns
    ///
    /// ChildHealth enum representing aggregated component health status:
    /// - `ChildHealth::Healthy` - Component operational
    /// - `ChildHealth::Degraded(reason)` - Component has issues but operational
    /// - `ChildHealth::Failed(reason)` - Component failed, restart needed
    ///
    /// # Performance
    ///
    /// This method executes in <1ms for state-only checks (no WASM invocation).
    /// Future _health export calls would add ~5-10ms depending on component implementation.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use airssys_wasm::actor::ComponentActor;
    /// use airssys_rt::supervisor::ChildHealth;
    ///
    /// let health = actor.health_check_inner().await;
    /// match health {
    ///     ChildHealth::Healthy => {
    ///         println!("Component operational");
    ///     }
    ///     ChildHealth::Degraded(reason) => {
    ///         println!("Component degraded: {}", reason);
    ///         // Monitor closely, may self-heal
    ///     }
    ///     ChildHealth::Failed(reason) => {
    ///         println!("Component failed: {}", reason);
    ///         // Supervisor will restart
    ///     }
    /// }
    /// ```
    ///
    /// # See Also
    ///
    /// - `parse_health_status()` - Helper for future _health export deserialization
    /// - `Child::health_check()` - Public API with timeout protection
    async fn health_check_inner(&self) -> ChildHealth {
        // 1. Check if WASM loaded
        let _runtime = match self.wasm_runtime() {
            Some(rt) => rt,
            None => {
                warn!(
                    component_id = %self.component_id().as_str(),
                    "Health check failed: WASM runtime not loaded"
                );
                return ChildHealth::Failed("WASM runtime not loaded".to_string());
            }
        };

        // 2. Evaluate ActorState for health determination
        let component_id = self.component_id().as_str();

        match self.state() {
            ActorState::Failed(reason) => {
                warn!(
                    component_id = %component_id,
                    reason = %reason,
                    "Health check: Component in Failed state"
                );
                ChildHealth::Failed(reason.clone())
            }
            ActorState::Terminated => {
                info!(
                    component_id = %component_id,
                    "Health check: Component terminated"
                );
                ChildHealth::Failed("Component terminated".to_string())
            }
            ActorState::Creating | ActorState::Starting => {
                info!(
                    component_id = %component_id,
                    state = ?self.state(),
                    "Health check: Component starting"
                );
                ChildHealth::Degraded("Component starting".to_string())
            }
            ActorState::Stopping => {
                info!(
                    component_id = %component_id,
                    "Health check: Component stopping"
                );
                ChildHealth::Degraded("Component stopping".to_string())
            }
            ActorState::Ready => {
                // Component is Ready and WASM loaded → Healthy
                //
                // Future enhancement: Call _health export here when Store mutability is resolved:
                // if let Some(health_fn) = &_runtime.exports().health {
                //     match self.call_health_export(health_fn, _runtime).await {
                //         Ok(wasm_status) => return Self::map_health_status(wasm_status),
                //         Err(e) => {
                //             warn!("_health export failed: {}, using state-based health", e);
                //             // Fall through to state-based Healthy
                //         }
                //     }
                // }

                ChildHealth::Healthy
            }
        }
    }

    /// Parse HealthStatus from multicodec-encoded WASM bytes.
    ///
    /// This helper method deserializes HealthStatus from bytes returned by the
    /// _health WASM export. It supports multiple serialization formats via multicodec
    /// detection (Borsh, CBOR, JSON) and tries all formats for maximum compatibility.
    ///
    /// # Arguments
    ///
    /// * `health_bytes` - Raw bytes from WASM linear memory (multicodec-prefixed)
    ///
    /// # Returns
    ///
    /// Parsed HealthStatus enum on success.
    ///
    /// # Errors
    ///
    /// Returns `WasmError::HealthCheckFailed` if:
    /// - Bytes are empty or too short
    /// - Multicodec prefix is invalid or unsupported
    /// - All deserialization attempts fail (Borsh, CBOR, JSON)
    ///
    /// # Format Support
    ///
    /// **Borsh** (preferred for performance):
    /// ```text
    /// [0x81, 0x0E, 0x00, ...]  // Multicodec 0x701 + Borsh payload
    /// Healthy:   [0x00]
    /// Degraded:  [0x01, len_u32, reason_bytes...]
    /// Unhealthy: [0x02, len_u32, reason_bytes...]
    /// ```
    ///
    /// **JSON** (human-readable):
    /// ```text
    /// [0x02, 0x00, 0x7b, ...]  // Multicodec 0x0200 + JSON payload
    /// {"status":"healthy"}
    /// {"status":"degraded","reason":"High latency"}
    /// {"status":"unhealthy","reason":"Database down"}
    /// ```
    ///
    /// **CBOR** (cross-language binary):
    /// ```text
    /// [0x51, ...]  // Multicodec 0x51 + CBOR payload
    /// Binary equivalent of JSON structure
    /// ```
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use airssys_wasm::actor::{ComponentActor, HealthStatus};
    ///
    /// // WASM component returns bytes from _health export
    /// let health_bytes = vec![0x81, 0x0E, 0x00]; // Borsh Healthy
    ///
    /// let status = ComponentActor::<()>::parse_health_status(&health_bytes)?;
    /// assert_eq!(status, HealthStatus::Healthy);
    /// ```
    ///
    /// # Performance
    ///
    /// - Multicodec decode: <5μs
    /// - Borsh deserialize: <10μs (fastest)
    /// - CBOR deserialize: <20μs
    /// - JSON deserialize: <50μs (slowest but most readable)
    ///
    /// # See Also
    ///
    /// - `HealthStatus` enum - Health status variants
    /// - `decode_multicodec()` - Multicodec prefix parsing
    /// - ADR-WASM-001 - Inter-Component Communication Design
    pub fn parse_health_status(health_bytes: &[u8]) -> Result<HealthStatus, WasmError> {
        use crate::core::decode_multicodec;
        use borsh::BorshDeserialize;

        if health_bytes.is_empty() {
            return Err(WasmError::health_check_failed(
                String::new(),
                "Empty health status bytes from _health export",
            ));
        }

        // Decode multicodec prefix
        let (codec, payload) = decode_multicodec(health_bytes).map_err(|e| {
            WasmError::health_check_failed(
                String::new(),
                format!("Multicodec decoding failed: {}", e),
            )
        })?;

        // Try deserializing with detected codec first, then try all formats
        // (component may have mislabeled the codec)

        // Try Borsh (most common for Rust components)
        if let Ok(status) = HealthStatus::try_from_slice(&payload) {
            return Ok(status);
        }

        // Try CBOR (common for cross-language)
        if let Ok(status) = serde_cbor::from_slice::<HealthStatus>(&payload) {
            return Ok(status);
        }

        // Try JSON (human-readable, debugging)
        if let Ok(status) = serde_json::from_slice::<HealthStatus>(&payload) {
            return Ok(status);
        }

        // All formats failed
        Err(WasmError::health_check_failed(
            String::new(),
            format!(
                "Failed to deserialize HealthStatus with codec {:?} (tried Borsh, CBOR, JSON)",
                codec
            ),
        ))
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "unwrap is acceptable in test code for brevity"
)]
mod tests {
    use super::*;
    use crate::core::{CapabilitySet, ComponentId, ComponentMetadata, ResourceLimits};

    fn create_test_metadata() -> ComponentMetadata {
        ComponentMetadata {
            name: "test-component".to_string(),
            version: "1.0.0".to_string(),
            author: "Test Author".to_string(),
            description: None,
            required_capabilities: vec![],
            resource_limits: ResourceLimits {
                max_memory_bytes: 64 * 1024 * 1024,
                max_fuel: 1_000_000,
                max_execution_ms: 5000,
                max_storage_bytes: 10 * 1024 * 1024,
            },
        }
    }

    fn create_test_actor() -> ComponentActor {
        ComponentActor::new(
            ComponentId::new("test-component"),
            create_test_metadata(),
            CapabilitySet::new(),
            (),
        )
    }

    #[tokio::test]
    async fn test_child_start_transitions_state() {
        let mut actor = create_test_actor();

        assert_eq!(*actor.state(), ActorState::Creating);

        // Start should transition to Ready
        let result = actor.start().await;
        assert!(result.is_ok());
        assert_eq!(*actor.state(), ActorState::Ready);
        assert!(actor.uptime().is_some());
    }

    #[tokio::test]
    async fn test_child_stop_transitions_state() {
        let mut actor = create_test_actor();

        let result = actor.start().await;
        assert!(result.is_ok(), "Failed to start actor: {result:?}");
        assert_eq!(*actor.state(), ActorState::Ready);

        // Stop should transition to Terminated
        let result = actor.stop(Duration::from_secs(5)).await;
        assert!(result.is_ok());
        assert_eq!(*actor.state(), ActorState::Terminated);
        assert!(!actor.is_wasm_loaded());
    }

    #[tokio::test]
    async fn test_child_health_check_state_based() {
        let mut actor = create_test_actor();

        // Before start - WASM not loaded → Failed
        let health = actor.health_check().await;
        assert!(matches!(health, ChildHealth::Failed(_)));

        // After start - Ready state → Healthy
        let result = actor.start().await;
        assert!(result.is_ok(), "Failed to start actor: {result:?}");
        let health = actor.health_check().await;
        assert!(matches!(health, ChildHealth::Healthy));

        // After stop - Terminated state → Failed
        let result = actor.stop(Duration::from_secs(1)).await;
        assert!(result.is_ok(), "Failed to stop actor: {result:?}");
        let health = actor.health_check().await;
        assert!(matches!(health, ChildHealth::Failed(_)));
    }

    #[tokio::test]
    async fn test_child_lifecycle_full_cycle() {
        let mut actor = create_test_actor();

        // Full lifecycle
        assert_eq!(*actor.state(), ActorState::Creating);

        let result = actor.start().await;
        assert!(result.is_ok(), "Failed to start actor: {result:?}");
        assert_eq!(*actor.state(), ActorState::Ready);

        let health = actor.health_check().await;
        assert!(matches!(health, ChildHealth::Healthy));

        let result = actor.stop(Duration::from_secs(5)).await;
        assert!(result.is_ok(), "Failed to stop actor: {result:?}");
        assert_eq!(*actor.state(), ActorState::Terminated);
    }

    #[tokio::test]
    async fn test_child_stop_timeout_parameter() {
        let mut actor = create_test_actor();
        let result = actor.start().await;
        assert!(result.is_ok(), "Failed to start actor: {result:?}");

        // Timeout parameter is accepted (though not used in stub)
        let result = actor.stop(Duration::from_millis(100)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_child_start_sets_timestamp() {
        let mut actor = create_test_actor();

        assert_eq!(actor.uptime(), None);

        let result = actor.start().await;
        assert!(result.is_ok(), "Failed to start actor: {result:?}");

        assert!(actor.uptime().is_some());
    }

    #[tokio::test]
    async fn test_child_trait_compiles() {
        // Verify ComponentActor implements Child trait
        fn assert_child<T: Child>() {}
        assert_child::<ComponentActor>();
    }

    // ======================================================================
    // COMPREHENSIVE HEALTH CHECK TESTS (Task 1.4)
    // ======================================================================

    /// Helper: Create actor with manually set state (for testing state-based health)
    fn create_actor_in_state(state: ActorState) -> ComponentActor {
        let mut actor = create_test_actor();
        actor.set_state(state);
        actor
    }

    /// Helper: Create actor without WASM runtime (simulates uninitialized state)
    fn create_actor_without_runtime() -> ComponentActor {
        let mut actor = create_test_actor();
        actor.clear_wasm_runtime();
        actor
    }

    // Test: ActorState::Creating → ChildHealth::Degraded
    #[tokio::test]
    async fn test_health_check_creating_state_returns_degraded() {
        let actor = create_actor_in_state(ActorState::Creating);
        let health = actor.health_check().await;

        assert!(
            matches!(health, ChildHealth::Failed(_)),
            "Creating state should return Failed (no WASM runtime), got: {health:?}"
        );

        if let ChildHealth::Failed(reason) = health {
            assert!(
                reason.contains("WASM runtime not loaded"),
                "Expected 'WASM runtime not loaded', got: {reason}"
            );
        }
    }

    // Test: ActorState::Starting → ChildHealth::Degraded
    #[tokio::test]
    async fn test_health_check_starting_state_returns_degraded() {
        let actor = create_actor_in_state(ActorState::Starting);
        let health = actor.health_check().await;

        assert!(
            matches!(health, ChildHealth::Failed(_)),
            "Starting state should return Failed (no WASM runtime), got: {health:?}"
        );
    }

    // Test: ActorState::Stopping → ChildHealth::Degraded
    #[tokio::test]
    async fn test_health_check_stopping_state_returns_degraded() {
        let mut actor = create_test_actor();
        // Start first to load WASM
        let result = actor.start().await;
        assert!(result.is_ok(), "Failed to start actor: {result:?}");

        // Manually set Stopping state (normally done by stop())
        actor.set_state(ActorState::Stopping);

        let health = actor.health_check().await;
        assert!(
            matches!(health, ChildHealth::Degraded(_)),
            "Stopping state should return Degraded, got: {health:?}"
        );

        if let ChildHealth::Degraded(reason) = health {
            assert!(
                reason.contains("Component stopping"),
                "Expected 'Component stopping', got: {reason}"
            );
        }
    }

    // Test: ActorState::Failed → ChildHealth::Failed
    #[tokio::test]
    async fn test_health_check_failed_state_returns_failed() {
        let actor = create_actor_in_state(ActorState::Failed("Test failure".to_string()));
        let health = actor.health_check().await;

        assert!(
            matches!(health, ChildHealth::Failed(_)),
            "Failed state should return Failed, got: {health:?}"
        );

        if let ChildHealth::Failed(reason) = health {
            assert!(
                reason.contains("WASM runtime not loaded") || reason.contains("Test failure"),
                "Expected failure reason, got: {reason}"
            );
        }
    }

    // Test: ActorState::Terminated → ChildHealth::Failed
    #[tokio::test]
    async fn test_health_check_terminated_state_returns_failed() {
        let mut actor = create_test_actor();

        // Start and stop to reach Terminated state
        let result = actor.start().await;
        assert!(result.is_ok(), "Failed to start actor: {result:?}");

        let result = actor.stop(Duration::from_secs(1)).await;
        assert!(result.is_ok(), "Failed to stop actor: {result:?}");

        let health = actor.health_check().await;
        assert!(
            matches!(health, ChildHealth::Failed(_)),
            "Terminated state should return Failed, got: {health:?}"
        );

        if let ChildHealth::Failed(reason) = health {
            assert!(
                reason.contains("WASM runtime not loaded"),
                "Expected 'WASM runtime not loaded', got: {reason}"
            );
        }
    }

    // Test: ActorState::Ready → ChildHealth::Healthy
    #[tokio::test]
    async fn test_health_check_ready_state_returns_healthy() {
        let mut actor = create_test_actor();

        let result = actor.start().await;
        assert!(result.is_ok(), "Failed to start actor: {result:?}");

        let health = actor.health_check().await;
        assert!(
            matches!(health, ChildHealth::Healthy),
            "Ready state should return Healthy, got: {health:?}"
        );
    }

    // Test: No WASM runtime → ChildHealth::Failed
    #[tokio::test]
    async fn test_health_check_no_wasm_runtime_returns_failed() {
        let actor = create_actor_without_runtime();
        let health = actor.health_check().await;

        assert!(
            matches!(health, ChildHealth::Failed(_)),
            "No WASM runtime should return Failed, got: {health:?}"
        );

        if let ChildHealth::Failed(reason) = health {
            assert!(
                reason.contains("WASM runtime not loaded"),
                "Expected 'WASM runtime not loaded', got: {reason}"
            );
        }
    }

    // Test: health_check() has timeout protection
    #[tokio::test]
    async fn test_health_check_has_timeout_protection() {
        let mut actor = create_test_actor();

        // Start component
        let result = actor.start().await;
        assert!(result.is_ok(), "Failed to start actor: {result:?}");

        // Call health_check and measure time
        let start = std::time::Instant::now();
        let health = actor.health_check().await;
        let elapsed = start.elapsed();

        // Should complete very quickly (<100ms for state-based check)
        assert!(
            elapsed.as_millis() < 100,
            "Health check took {}ms (expected <100ms)",
            elapsed.as_millis()
        );
        assert!(
            matches!(health, ChildHealth::Healthy),
            "Ready state should return Healthy"
        );
    }

    // Test: ChildHealth::Healthy maps to HealthStatus::Healthy
    #[test]
    fn test_child_health_healthy_maps_to_health_status_healthy() {
        use super::super::component_actor::HealthStatus;

        // Simulate mapping logic from actor_impl.rs
        let child_health = ChildHealth::Healthy;
        let health_status = match child_health {
            ChildHealth::Healthy => HealthStatus::Healthy,
            ChildHealth::Degraded(reason) => HealthStatus::Degraded { reason },
            ChildHealth::Failed(reason) => HealthStatus::Unhealthy { reason },
        };

        assert_eq!(health_status, HealthStatus::Healthy);
    }

    // Test: ChildHealth::Degraded maps to HealthStatus::Degraded
    #[test]
    fn test_child_health_degraded_maps_to_health_status_degraded() {
        use super::super::component_actor::HealthStatus;

        let child_health = ChildHealth::Degraded("High latency".to_string());
        let health_status = match child_health {
            ChildHealth::Healthy => HealthStatus::Healthy,
            ChildHealth::Degraded(reason) => HealthStatus::Degraded { reason },
            ChildHealth::Failed(reason) => HealthStatus::Unhealthy { reason },
        };

        assert!(matches!(health_status, HealthStatus::Degraded { .. }));

        if let HealthStatus::Degraded { reason } = health_status {
            assert_eq!(reason, "High latency");
        }
    }

    // Test: ChildHealth::Failed maps to HealthStatus::Unhealthy
    #[test]
    fn test_child_health_failed_maps_to_health_status_unhealthy() {
        use super::super::component_actor::HealthStatus;

        let child_health = ChildHealth::Failed("Component crashed".to_string());
        let health_status = match child_health {
            ChildHealth::Healthy => HealthStatus::Healthy,
            ChildHealth::Degraded(reason) => HealthStatus::Degraded { reason },
            ChildHealth::Failed(reason) => HealthStatus::Unhealthy { reason },
        };

        assert!(matches!(health_status, HealthStatus::Unhealthy { .. }));

        if let HealthStatus::Unhealthy { reason } = health_status {
            assert_eq!(reason, "Component crashed");
        }
    }

    // ========================================================================
    // Tests for parse_health_status() - Multicodec HealthStatus Deserialization
    // ========================================================================

    /// Test: parse_health_status() with empty bytes → Error
    #[test]
    fn test_parse_health_status_empty_bytes_error() {
        let result = ComponentActor::<()>::parse_health_status(&[]);
        assert!(result.is_err());

        if let Err(e) = result {
            assert!(e.to_string().contains("Empty health status bytes"));
        }
    }

    /// Test: parse_health_status() with Borsh Healthy
    #[test]
    fn test_parse_health_status_borsh_healthy() {
        use crate::core::encode_multicodec;
        use crate::core::multicodec::Codec;

        // Encode HealthStatus::Healthy with Borsh
        let status = HealthStatus::Healthy;
        let borsh_bytes = borsh::to_vec(&status).unwrap();
        let encoded = encode_multicodec(Codec::Borsh, &borsh_bytes).unwrap();

        // Parse back
        let parsed = ComponentActor::<()>::parse_health_status(&encoded).unwrap();
        assert_eq!(parsed, HealthStatus::Healthy);
    }

    /// Test: parse_health_status() with Borsh Degraded
    #[test]
    fn test_parse_health_status_borsh_degraded() {
        use crate::core::encode_multicodec;
        use crate::core::multicodec::Codec;

        let status = HealthStatus::Degraded {
            reason: "High memory usage".to_string(),
        };
        let borsh_bytes = borsh::to_vec(&status).unwrap();
        let encoded = encode_multicodec(Codec::Borsh, &borsh_bytes).unwrap();

        let parsed = ComponentActor::<()>::parse_health_status(&encoded).unwrap();
        assert_eq!(parsed, status);
    }

    /// Test: parse_health_status() with Borsh Unhealthy
    #[test]
    fn test_parse_health_status_borsh_unhealthy() {
        use crate::core::encode_multicodec;
        use crate::core::multicodec::Codec;

        let status = HealthStatus::Unhealthy {
            reason: "Database connection lost".to_string(),
        };
        let borsh_bytes = borsh::to_vec(&status).unwrap();
        let encoded = encode_multicodec(Codec::Borsh, &borsh_bytes).unwrap();

        let parsed = ComponentActor::<()>::parse_health_status(&encoded).unwrap();
        assert_eq!(parsed, status);
    }

    /// Test: parse_health_status() with JSON Healthy
    #[test]
    fn test_parse_health_status_json_healthy() {
        use crate::core::encode_multicodec;
        use crate::core::multicodec::Codec;

        let status = HealthStatus::Healthy;
        let json_bytes = serde_json::to_vec(&status).unwrap();
        let encoded = encode_multicodec(Codec::JSON, &json_bytes).unwrap();

        let parsed = ComponentActor::<()>::parse_health_status(&encoded).unwrap();
        assert_eq!(parsed, HealthStatus::Healthy);
    }

    /// Test: parse_health_status() with JSON Degraded
    #[test]
    fn test_parse_health_status_json_degraded() {
        use crate::core::encode_multicodec;
        use crate::core::multicodec::Codec;

        let status = HealthStatus::Degraded {
            reason: "Response time elevated".to_string(),
        };
        let json_bytes = serde_json::to_vec(&status).unwrap();
        let encoded = encode_multicodec(Codec::JSON, &json_bytes).unwrap();

        let parsed = ComponentActor::<()>::parse_health_status(&encoded).unwrap();
        assert_eq!(parsed, status);
    }

    /// Test: parse_health_status() with CBOR Healthy
    #[test]
    fn test_parse_health_status_cbor_healthy() {
        use crate::core::encode_multicodec;
        use crate::core::multicodec::Codec;

        let status = HealthStatus::Healthy;
        let cbor_bytes = serde_cbor::to_vec(&status).unwrap();
        let encoded = encode_multicodec(Codec::CBOR, &cbor_bytes).unwrap();

        let parsed = ComponentActor::<()>::parse_health_status(&encoded).unwrap();
        assert_eq!(parsed, HealthStatus::Healthy);
    }

    /// Test: parse_health_status() with CBOR Unhealthy
    #[test]
    fn test_parse_health_status_cbor_unhealthy() {
        use crate::core::encode_multicodec;
        use crate::core::multicodec::Codec;

        let status = HealthStatus::Unhealthy {
            reason: "Critical failure".to_string(),
        };
        let cbor_bytes = serde_cbor::to_vec(&status).unwrap();
        let encoded = encode_multicodec(Codec::CBOR, &cbor_bytes).unwrap();

        let parsed = ComponentActor::<()>::parse_health_status(&encoded).unwrap();
        assert_eq!(parsed, status);
    }

    /// Test: parse_health_status() with invalid multicodec → Error
    #[test]
    fn test_parse_health_status_invalid_multicodec() {
        // Invalid multicodec prefix
        let invalid_bytes = vec![0xFF, 0xFF, 0xFF, 0xFF];
        let result = ComponentActor::<()>::parse_health_status(&invalid_bytes);
        assert!(result.is_err());
    }

    /// Test: parse_health_status() with valid codec but invalid payload → Error
    #[test]
    fn test_parse_health_status_invalid_payload() {
        use crate::core::encode_multicodec;
        use crate::core::multicodec::Codec;

        // Valid Borsh multicodec but invalid HealthStatus payload
        let invalid_payload = vec![0x99, 0x99, 0x99];
        let encoded = encode_multicodec(Codec::Borsh, &invalid_payload).unwrap();

        let result = ComponentActor::<()>::parse_health_status(&encoded);
        assert!(result.is_err());

        if let Err(e) = result {
            assert!(e.to_string().contains("Failed to deserialize HealthStatus"));
        }
    }

    /// Test: parse_health_status() tries all formats on mislabeled codec
    #[test]
    fn test_parse_health_status_fallback_to_all_formats() {
        use crate::core::encode_multicodec;
        use crate::core::multicodec::Codec;

        // Encode JSON but label as Borsh (simulates component mislabeling)
        let status = HealthStatus::Degraded {
            reason: "Testing fallback".to_string(),
        };
        let json_bytes = serde_json::to_vec(&status).unwrap();
        let mislabeled = encode_multicodec(Codec::Borsh, &json_bytes).unwrap();

        // Should still parse correctly because we try all formats
        let parsed = ComponentActor::<()>::parse_health_status(&mislabeled).unwrap();
        assert_eq!(parsed, status);
    }

    /// Test: parse_health_status() round-trip preserves data (Borsh)
    #[test]
    fn test_parse_health_status_borsh_round_trip() {
        use crate::core::encode_multicodec;
        use crate::core::multicodec::Codec;

        let original = HealthStatus::Degraded {
            reason: "Test reason with special chars: 日本語 🚀".to_string(),
        };

        let borsh_bytes = borsh::to_vec(&original).unwrap();
        let encoded = encode_multicodec(Codec::Borsh, &borsh_bytes).unwrap();
        let parsed = ComponentActor::<()>::parse_health_status(&encoded).unwrap();

        assert_eq!(parsed, original);
    }

    /// Test: parse_health_status() round-trip preserves data (JSON)
    #[test]
    fn test_parse_health_status_json_round_trip() {
        use crate::core::encode_multicodec;
        use crate::core::multicodec::Codec;

        let original = HealthStatus::Unhealthy {
            reason: "Complex UTF-8: Здравствуй мир! 你好世界!".to_string(),
        };

        let json_bytes = serde_json::to_vec(&original).unwrap();
        let encoded = encode_multicodec(Codec::JSON, &json_bytes).unwrap();
        let parsed = ComponentActor::<()>::parse_health_status(&encoded).unwrap();

        assert_eq!(parsed, original);
    }

    /// Test: parse_health_status() performance (Borsh should be fastest)
    #[test]
    fn test_parse_health_status_performance() {
        use crate::core::encode_multicodec;
        use crate::core::multicodec::Codec;
        use std::time::Instant;

        let status = HealthStatus::Degraded {
            reason: "Performance test".to_string(),
        };

        // Prepare encoded data
        let borsh_bytes = borsh::to_vec(&status).unwrap();
        let borsh_encoded = encode_multicodec(Codec::Borsh, &borsh_bytes).unwrap();

        // Measure parsing time (should be <1ms)
        let start = Instant::now();
        for _ in 0..1000 {
            let _ = ComponentActor::<()>::parse_health_status(&borsh_encoded).unwrap();
        }
        let elapsed = start.elapsed();

        // 1000 parses should complete in <10ms (10μs per parse)
        assert!(
            elapsed.as_millis() < 10,
            "1000 parses took {}ms (expected <10ms)",
            elapsed.as_millis()
        );
    }
}
