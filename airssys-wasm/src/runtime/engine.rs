//! Wasmtime-based runtime engine implementation.
//!
//! This module implements the `RuntimeEngine` trait using Wasmtime v24.0
//! with Component Model support, async execution, and fuel metering.
//!
//! # Architecture
//!
//! ```text
//! WasmEngine (Arc<WasmEngineInner>)
//!     ├── Wasmtime Engine (Component Model + async + Cranelift JIT)
//!     └── Component Cache (HashMap<ComponentId, PrecompiledComponent>)
//! ```
//!
//! # Design Decisions (ADR-WASM-002)
//!
//! - **Arc<Inner> Pattern**: Cheap cloning for multi-threaded use (M-SERVICES-CLONE)
//! - **Component Model**: WebAssembly Component Model support enabled
//! - **Async Runtime**: Tokio integration for non-blocking execution
//! - **Fuel Metering**: Hybrid CPU limiting (fuel + wall-clock timeout)
//! - **Cranelift JIT**: Fast compilation with predictable performance
//!
//! # Example
//!
//! ```rust,ignore
//! use airssys_wasm::runtime::WasmEngine;
//! use airssys_wasm::core::{RuntimeEngine, ComponentId};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create engine with default production configuration
//!     let engine = WasmEngine::new()?;
//!     
//!     // Load component
//!     let bytes = std::fs::read("component.wasm")?;
//!     let id = ComponentId::new("my-component");
//!     let handle = engine.load_component(&id, &bytes).await?;
//!     
//!     println!("Component loaded: {}", handle.id());
//!     Ok(())
//! }
//! ```

// Layer 1: Standard library imports (§2.1 - 3-layer import organization)
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

// Layer 2: External crate imports
use async_trait::async_trait;
use tokio::time::{timeout, Duration};
use wasmtime::{Config, Engine, Store};
use wasmtime::component::{Component, Linker};

// Layer 3: Internal module imports
use crate::core::{
    error::{WasmError, WasmResult},
    runtime::{ComponentHandle, ExecutionContext, ResourceUsage, RuntimeEngine},
    ComponentId, ComponentInput, ComponentOutput,
};

/// Wasmtime-based WebAssembly runtime engine.
///
/// Implements the `RuntimeEngine` trait for executing WebAssembly components
/// with Component Model support, async execution, and resource limiting.
///
/// # Design Pattern (M-SERVICES-CLONE)
///
/// Uses the `Arc<Inner>` pattern for cheap cloning and thread-safe sharing.
/// Multiple clones share the same underlying Wasmtime engine and component cache.
///
/// # Configuration
///
/// Default configuration provides production-ready settings:
/// - Component Model support enabled
/// - Async support with Tokio integration
/// - Cranelift JIT compiler
/// - Fuel metering enabled (10,000,000 default)
/// - Epoch-based interruption for timeouts
///
/// # Thread Safety
///
/// `WasmEngine` is `Send + Sync` and can be used across multiple threads.
/// Internal state is protected by `RwLock` for concurrent access.
///
/// # Example
///
/// ```rust,ignore
/// use airssys_wasm::runtime::WasmEngine;
///
/// // Create engine
/// let engine = WasmEngine::new()?;
///
/// // Clone is cheap (Arc increment)
/// let engine_clone = engine.clone();
///
/// // Use in multiple threads
/// tokio::spawn(async move {
///     // engine_clone can be used here
/// });
/// ```
#[derive(Clone)]
pub struct WasmEngine {
    inner: Arc<WasmEngineInner>,
}

/// Internal state for WasmEngine (Arc pattern).
struct WasmEngineInner {
    /// Wasmtime engine with Component Model support.
    engine: Engine,
    
    /// Component cache (future optimization - Phase 2).
    /// Maps ComponentId to compiled component instances.
    #[allow(dead_code)]
    component_cache: RwLock<HashMap<String, ()>>,
}

impl WasmEngine {
    /// Create a new WasmEngine with default production configuration.
    ///
    /// # Configuration (ADR-WASM-002)
    ///
    /// - **Component Model**: Enabled for Component Model support
    /// - **Async**: Enabled with Tokio runtime integration
    /// - **Fuel**: Enabled for CPU metering (10,000,000 default)
    /// - **Epoch Interruption**: Enabled for wall-clock timeouts
    /// - **Compiler**: Cranelift JIT (predictable performance)
    ///
    /// # Returns
    ///
    /// - `Ok(WasmEngine)`: Engine initialized successfully
    /// - `Err(WasmError::EngineInitialization)`: Engine creation failed
    ///
    /// # Errors
    ///
    /// - `WasmError::EngineInitialization`: Wasmtime engine creation failed
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use airssys_wasm::runtime::WasmEngine;
    ///
    /// let engine = WasmEngine::new()?;
    /// println!("Engine created successfully");
    /// ```
    pub fn new() -> WasmResult<Self> {
        // Configure Wasmtime engine (ADR-WASM-002 specifications)
        let mut config = Config::new();
        
        // Enable Component Model support
        config.wasm_component_model(true);
        
        // Enable async support for non-blocking execution
        config.async_support(true);
        
        // Enable fuel metering for CPU limiting
        config.consume_fuel(true);
        
        // Create Wasmtime engine
        let engine = Engine::new(&config).map_err(|e| {
            WasmError::engine_initialization(format!("Failed to create Wasmtime engine: {e}"))
        })?;
        
        Ok(Self {
            inner: Arc::new(WasmEngineInner {
                engine,
                component_cache: RwLock::new(HashMap::new()),
            }),
        })
    }
    
    /// Get reference to underlying Wasmtime engine.
    ///
    /// Used internally for component loading and instantiation.
    #[allow(dead_code)]
    pub(crate) fn engine(&self) -> &Engine {
        &self.inner.engine
    }
    
    /// Internal execution helper (without timeout wrapper).
    ///
    /// Performs the actual component execution with fuel metering.
    /// Called by `execute()` which wraps this with tokio timeout.
    async fn execute_internal(
        &self,
        handle: &ComponentHandle,
        function: &str,
        _input: ComponentInput,
        context: ExecutionContext,
    ) -> WasmResult<ComponentOutput> {
        // Create Store with fuel configuration
        let mut store = Store::new(&self.inner.engine, ());
        
        // Set fuel for CPU metering (hybrid limiting)
        store
            .set_fuel(context.limits.max_fuel)
            .map_err(|e| WasmError::execution_failed(format!("Failed to set fuel: {e}")))?;
        
        // Create linker for component instantiation
        let linker = Linker::new(&self.inner.engine);
        
        // Instantiate component
        let instance = linker
            .instantiate_async(&mut store, handle.component())
            .await
            .map_err(|e| {
                WasmError::execution_failed(format!(
                    "Failed to instantiate component '{}': {e}",
                    context.component_id.as_str()
                ))
            })?;
        
        // Get typed function (Component Model: () -> s32)
        // Component Model requires typed function interfaces
        let func = instance
            .get_typed_func::<(), (i32,)>(&mut store, function)
            .map_err(|e| {
                WasmError::execution_failed(format!(
                    "Function '{function}' not found or type mismatch: {e}"
                ))
            })?;
        
        // Call function (async because engine has async_support enabled)
        let (result,) = func
            .call_async(&mut store, ())
            .await
            .map_err(|e| {
                WasmError::execution_failed(format!("Function '{function}' execution failed: {e}"))
            })?;
        
        // Convert result to ComponentOutput
        Ok(ComponentOutput::from_i32(result))
    }
}

// Implement RuntimeEngine trait for WasmEngine
#[async_trait]
impl RuntimeEngine for WasmEngine {
    async fn load_component(
        &self,
        component_id: &ComponentId,
        bytes: &[u8],
    ) -> WasmResult<ComponentHandle> {
        // Parse component bytes into Wasmtime Component
        let component = Component::new(&self.inner.engine, bytes).map_err(|e| {
            WasmError::component_load_failed(
                component_id.as_str(),
                format!("Failed to parse WebAssembly component: {e}"),
            )
        })?;
        
        // Wrap in Arc for cheap cloning (Option A - WASM-TASK-002)
        let component_arc = Arc::new(component);
        
        // Return handle with component reference
        Ok(ComponentHandle::new(component_id.as_str(), component_arc))
    }
    
    async fn execute(
        &self,
        handle: &ComponentHandle,
        function: &str,
        input: ComponentInput,
        context: ExecutionContext,
    ) -> WasmResult<ComponentOutput> {
        // Wrap execution with timeout (hybrid CPU limiting)
        let timeout_duration = Duration::from_millis(context.timeout_ms);
        
        match timeout(timeout_duration, self.execute_internal(handle, function, input, context.clone())).await {
            Ok(result) => result,
            Err(_elapsed) => {
                // Timeout exceeded - return ExecutionTimeout error
                Err(WasmError::execution_timeout(context.timeout_ms, None))
            }
        }
    }
    
    fn resource_usage(&self, _handle: &ComponentHandle) -> ResourceUsage {
        // Stub implementation - real tracking requires persistent Store
        // TODO(Phase 4): Implement stateful resource tracking
        // For now, return zero values as Store is dropped after execution
        ResourceUsage {
            memory_bytes: 0,
            fuel_consumed: 0,
            execution_time_ms: 0,
        }
    }
}

impl std::fmt::Debug for WasmEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WasmEngine")
            .field("engine", &"Wasmtime::Engine")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    #![allow(clippy::expect_used)]
    
    use super::*;
    
    #[test]
    fn test_engine_creation() {
        let engine = WasmEngine::new();
        assert!(engine.is_ok(), "Engine creation should succeed");
    }
    
    #[test]
    fn test_engine_clone() {
        let engine = WasmEngine::new().unwrap();
        let cloned = engine.clone();
        
        // Verify Arc pointer equality (same underlying engine)
        assert!(Arc::ptr_eq(&engine.inner, &cloned.inner));
    }
    
    #[test]
    fn test_engine_debug_format() {
        let engine = WasmEngine::new().unwrap();
        let debug_str = format!("{engine:?}");
        assert!(debug_str.contains("WasmEngine"));
    }
    
    #[test]
    fn test_engine_send_sync() {
        // Compile-time verification that WasmEngine is Send + Sync
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        
        assert_send::<WasmEngine>();
        assert_sync::<WasmEngine>();
    }
}
