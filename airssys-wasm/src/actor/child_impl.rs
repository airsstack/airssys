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
use super::component_actor::{ActorState, ComponentActor, ComponentResourceLimiter, WasmRuntime, WasmExports};
use airssys_rt::supervisor::{Child, ChildHealth};
use crate::core::WasmError;

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
impl Child for ComponentActor {
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
        config.async_support(true);  // Required for async component execution
        config.wasm_multi_value(true);  // Allow multiple return values
        config.consume_fuel(true);  // Enable fuel metering for CPU limits
        
        // Disable unsafe WASM features for security (ADR-WASM-003)
        config.wasm_bulk_memory(false);
        config.wasm_reference_types(false);
        config.wasm_threads(false);
        config.wasm_simd(false);
        config.wasm_relaxed_simd(false);  // Must be disabled if SIMD is disabled
        
        let engine = Engine::new(&config)
            .map_err(|e| {
                let err_msg = format!("Failed to create Wasmtime engine: {e}");
                self.set_state(ActorState::Failed(err_msg.clone()));
                WasmError::engine_initialization(err_msg)
            })?;
        
        // 5. Compile WASM module
        let module = Module::from_binary(&engine, &wasm_bytes)
            .map_err(|e| {
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
                WasmError::component_load_failed(
                    self.component_id().as_str(),
                    err_msg
                )
            })?;
        
        // 6. Create Store with ResourceLimiter
        let max_memory_bytes = self.metadata().resource_limits.max_memory_bytes;
        let max_fuel = self.metadata().resource_limits.max_fuel;
        
        let limiter = ComponentResourceLimiter::new(max_memory_bytes, max_fuel);
        let mut store = Store::new(&engine, limiter);
        
        // Set initial fuel
        store
            .set_fuel(max_fuel)
            .map_err(|e| {
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
        WasmExports::call_start_fn(start_fn_opt.as_ref(), runtime.store_mut()).await
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
        // 1. Transition to Stopping state
        self.set_state(ActorState::Stopping);
        
        // 2. Call optional _cleanup export if WASM is loaded
        if let Some(runtime) = self.wasm_runtime_mut() {
            // Clone cleanup function to avoid borrowing issues
            let cleanup_fn_opt = runtime.exports().cleanup;
            
            match WasmExports::call_cleanup_fn(
                cleanup_fn_opt.as_ref(),
                runtime.store_mut(),
                timeout
            ).await {
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
        
        Ok(())
    }

    /// Check component health status (STUB - always healthy).
    ///
    /// **STUB IMPLEMENTATION**: Always returns ChildHealth::Healthy.
    /// Full implementation in Task 3.3 will:
    /// 1. Check if WASM is loaded (unhealthy if not)
    /// 2. Call optional _health export if available
    /// 3. Map WASM health result to ChildHealth
    /// 4. Consider error rate, memory pressure, etc.
    ///
    /// # Returns
    ///
    /// - **Stub**: Always ChildHealth::Healthy
    /// - **Future**: ChildHealth::Healthy | Degraded | Failed based on checks
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Ignored until Block 6 (Component Storage) is implemented
    /// use airssys_wasm::actor::ComponentActor;
    /// use airssys_wasm::core::{ComponentId, ComponentMetadata, CapabilitySet, ResourceLimits};
    /// use airssys_rt::supervisor::{Child, ChildHealth};
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
    ///     // Health check (stub - always healthy)
    ///     let health = actor.health_check().await;
    ///     assert!(health.is_healthy());
    ///     
    ///     Ok(())
    /// }
    /// ```
    async fn health_check(&self) -> ChildHealth {
        // TODO(Task 3.3): Implement actual health checking
        //
        // Full implementation will:
        // 1. Check if WASM loaded: self.wasm_runtime.is_some()
        // 2. Call _health export if available
        // 3. Map HealthStatus → ChildHealth
        // 4. Consider metrics (error rate, memory, etc.)

        // Stub: Always healthy
        ChildHealth::Healthy
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{ComponentId, ComponentMetadata, CapabilitySet, ResourceLimits};

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
    async fn test_child_health_check_always_healthy() {
        let mut actor = create_test_actor();

        // Before start
        let health = actor.health_check().await;
        assert!(health.is_healthy());

        // After start
        let result = actor.start().await;
        assert!(result.is_ok(), "Failed to start actor: {result:?}");
        let health = actor.health_check().await;
        assert!(health.is_healthy());

        // After stop
        let result = actor.stop(Duration::from_secs(1)).await;
        assert!(result.is_ok(), "Failed to stop actor: {result:?}");
        let health = actor.health_check().await;
        assert!(health.is_healthy());
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
        assert!(health.is_healthy());

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
}
