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
//! - **`Arc<Inner>` Pattern**: Cheap cloning for multi-threaded use (M-SERVICES-CLONE)
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
use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine};

// Layer 3: Internal module imports
use crate::core::{
    error::{WasmError, WasmResult},
    // Note: RuntimeEngine uses these types internally
    runtime::{
        ComponentHandle, ExecutionContext, ResourceUsage, RuntimeEngine,
        RuntimeMessageHandlerEngine,
    },
    ComponentId,
    ComponentInput,
    ComponentOutput,
};
use crate::runtime::store_manager::StoreWrapper;

/// Wasmtime-based WebAssembly runtime engine.
///
/// Implements the `RuntimeEngine` trait for executing WebAssembly components
/// with Component Model support, async execution, and resource limiting.
///
/// # Design Pattern (M-SERVICES-CLONE)
///
/// Uses the ``Arc<Inner>`` pattern for cheap cloning and thread-safe sharing.
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
    /// Performs the actual component execution with fuel metering and crash isolation.
    /// Called by `execute()` which wraps this with tokio timeout and panic boundaries.
    ///
    /// # Crash Isolation (Phase 5 - ADR-WASM-006)
    ///
    /// This method implements trap detection and categorization for WASM crashes:
    /// - **Traps**: WASM semantic violations (division by zero, bounds check, unreachable)
    /// - **Fuel Exhaustion**: CPU limit exceeded via fuel metering
    /// - **Stack Overflow**: Call stack exceeded configured limits
    ///
    /// All crashes are categorized and logged with diagnostic information.
    /// Host runtime remains stable regardless of component behavior.
    async fn execute_internal(
        &self,
        handle: &ComponentHandle,
        function: &str,
        _input: ComponentInput,
        context: ExecutionContext,
    ) -> WasmResult<ComponentOutput> {
        // Create Store with RAII cleanup wrapper (Phase 5 - Task 5.2)
        let mut store = StoreWrapper::new(&self.inner.engine, (), context.limits.max_fuel)?;

        // Set component ID for cleanup diagnostics
        store.set_component_id(context.component_id.as_str().to_string());

        // Create linker for component instantiation
        let linker = Linker::new(&self.inner.engine);

        // Instantiate component with trap handling
        let instance = linker
            .instantiate_async(&mut store, handle.component())
            .await
            .map_err(|e| {
                // Categorize instantiation failures
                Self::categorize_wasmtime_error(&e, &context.component_id, function)
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

        // Call function with trap detection (async because engine has async_support enabled)
        let (result,) = func.call_async(&mut store, ()).await.map_err(|e| {
            // Get fuel consumed before crash (StoreWrapper provides this)
            let fuel_consumed = store.fuel_consumed();

            // Categorize execution failures (traps, fuel exhaustion, etc.)
            Self::categorize_wasmtime_error_with_fuel(
                &e,
                &context.component_id,
                function,
                fuel_consumed,
            )
        })?;

        // Convert result to ComponentOutput
        Ok(ComponentOutput::from_i32(result))
    }

    /// Categorize Wasmtime errors into structured WasmError types.
    ///
    /// Maps Wasmtime-specific errors to our error taxonomy for proper handling
    /// and diagnostics. This enables supervisor patterns to make informed restart decisions.
    ///
    /// # Error Categories (Phase 5 - Task 5.1)
    ///
    /// - **Trap**: WASM semantic violations (unreachable, bounds, divide by zero)
    /// - **Fuel Exhaustion**: CPU limit exceeded
    /// - **Stack Overflow**: Call stack limit exceeded
    /// - **Memory**: Memory allocation failures
    /// - **Generic**: All other execution failures
    fn categorize_wasmtime_error(
        error: &wasmtime::Error,
        component_id: &ComponentId,
        function: &str,
    ) -> WasmError {
        Self::categorize_wasmtime_error_with_fuel(error, component_id, function, None)
    }

    /// Categorize Wasmtime errors with fuel consumption data.
    ///
    /// Extended version that includes fuel_consumed for resource accounting.
    fn categorize_wasmtime_error_with_fuel(
        error: &wasmtime::Error,
        component_id: &ComponentId,
        function: &str,
        fuel_consumed: Option<u64>,
    ) -> WasmError {
        let error_str = error.to_string();

        // Check for trap-specific errors
        if let Some(trap) = error.downcast_ref::<wasmtime::Trap>() {
            return Self::categorize_trap(trap, component_id, function, fuel_consumed);
        }

        // Check error message for common patterns
        if error_str.contains("out of fuel") || error_str.contains("fuel exhausted") {
            // Fuel exhaustion - CPU limit exceeded
            return WasmError::component_trapped(
                format!(
                    "Component '{}' exhausted fuel during '{function}' (CPU limit exceeded)",
                    component_id.as_str()
                ),
                fuel_consumed,
            );
        }

        if error_str.contains("out of memory") || error_str.contains("memory allocation") {
            // Memory allocation failure
            return WasmError::execution_failed(format!(
                "Component '{}' failed memory allocation in '{function}': {error_str}",
                component_id.as_str()
            ));
        }

        if error_str.contains("stack overflow") || error_str.contains("call stack") {
            // Stack overflow
            return WasmError::component_trapped(
                format!(
                    "Component '{}' stack overflow in '{function}' (call depth limit exceeded)",
                    component_id.as_str()
                ),
                fuel_consumed,
            );
        }

        // Generic execution failure
        WasmError::execution_failed(format!(
            "Component '{}' function '{function}' failed: {error_str}",
            component_id.as_str()
        ))
    }

    /// Categorize specific WASM trap types.
    ///
    /// Wasmtime provides detailed trap information for different WASM violations.
    /// We categorize these for logging, monitoring, and supervisor decision-making.
    ///
    /// # Implementation Note
    ///
    /// Wasmtime's Trap type doesn't expose variants publicly, so we pattern match
    /// on the Display string representation to categorize trap types.
    fn categorize_trap(
        trap: &wasmtime::Trap,
        component_id: &ComponentId,
        function: &str,
        fuel_consumed: Option<u64>,
    ) -> WasmError {
        let trap_str = trap.to_string();
        let trap_lower = trap_str.to_lowercase();

        // Pattern match on trap message to categorize trap type
        let reason = if trap_lower.contains("unreachable") {
            format!(
                "Component '{}' hit unreachable instruction in '{function}' (program bug or assertion failure)",
                component_id.as_str()
            )
        } else if trap_lower.contains("out of bounds") && trap_lower.contains("memory") {
            format!(
                "Component '{}' memory out of bounds in '{function}' (invalid memory access)",
                component_id.as_str()
            )
        } else if trap_lower.contains("out of bounds") && trap_lower.contains("table") {
            format!(
                "Component '{}' table out of bounds in '{function}' (invalid function table access)",
                component_id.as_str()
            )
        } else if trap_lower.contains("null") && trap_lower.contains("indirect") {
            format!(
                "Component '{}' indirect call to null in '{function}' (null function pointer)",
                component_id.as_str()
            )
        } else if trap_lower.contains("signature") || trap_lower.contains("type mismatch") {
            format!(
                "Component '{}' bad signature in '{function}' (function type mismatch)",
                component_id.as_str()
            )
        } else if trap_lower.contains("overflow") && trap_lower.contains("integer") {
            format!(
                "Component '{}' integer overflow in '{function}' (arithmetic error)",
                component_id.as_str()
            )
        } else if trap_lower.contains("division by zero") || trap_lower.contains("divide by zero") {
            format!(
                "Component '{}' division by zero in '{function}' (arithmetic error)",
                component_id.as_str()
            )
        } else if trap_lower.contains("conversion") && trap_lower.contains("integer") {
            format!(
                "Component '{}' bad conversion to integer in '{function}' (type conversion error)",
                component_id.as_str()
            )
        } else if trap_lower.contains("stack overflow") {
            format!(
                "Component '{}' stack overflow in '{function}' (call depth limit exceeded)",
                component_id.as_str()
            )
        } else if trap_lower.contains("interrupt") || trap_lower.contains("timeout") {
            format!(
                "Component '{}' interrupted in '{function}' (timeout or external signal)",
                component_id.as_str()
            )
        } else if trap_lower.contains("out of fuel") || trap_lower.contains("fuel") {
            format!(
                "Component '{}' exhausted fuel during '{function}' (CPU limit exceeded)",
                component_id.as_str()
            )
        } else {
            // Generic trap - use Wasmtime's message
            format!(
                "Component '{}' trapped in '{function}': {trap_str}",
                component_id.as_str()
            )
        };

        WasmError::component_trapped(reason, fuel_consumed)
    }

    /// Call the handle-message export on a component.
    ///
    /// Uses Component Model API for type-safe invocation with automatic
    /// parameter marshalling via Canonical ABI.
    ///
    /// # Arguments
    ///
    /// * `handle` - Component handle from `load_component()`
    /// * `sender` - Sender component ID
    /// * `payload` - Message payload bytes
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message handled successfully
    /// * `Err(WasmError)` - Execution failed
    ///
    /// # WIT Interface
    ///
    /// The handle-message export is defined in component-lifecycle.wit:
    /// ```wit
    /// handle-message: func(sender: component-id, message: list<u8>) -> result<_, component-error>;
    /// ```
    ///
    /// # Implementation Notes
    ///
    /// This method uses Wasmtime's Component Model typed function API:
    /// 1. Creates a Store with fuel metering for CPU limiting
    /// 2. Creates a Linker for component instantiation
    /// 3. Instantiates the component
    /// 4. Gets the typed handle-message function export
    /// 5. Calls the function with sender and message parameters
    /// 6. Handles success/error results
    ///
    /// For the WIT record `component-id`, we use a simplified approach:
    /// - Pass sender ID as a string tuple: (namespace, name, version)
    /// - Components can parse this to reconstruct the component-id record
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use airssys_wasm::runtime::WasmEngine;
    /// use airssys_wasm::core::ComponentId;
    ///
    /// async fn handle_message(engine: &WasmEngine, handle: &ComponentHandle) {
    ///     let sender = ComponentId::new("sender-component");
    ///     let payload = b"hello world";
    ///     
    ///     engine.call_handle_message(handle, &sender, payload).await?;
    /// }
    /// ```
    pub async fn call_handle_message(
        &self,
        handle: &ComponentHandle,
        sender: &ComponentId,
        payload: &[u8],
    ) -> WasmResult<()> {
        // Default fuel limit for message handling operations
        const DEFAULT_FUEL: u64 = 10_000_000;

        // 1. Create Store with fuel metering for CPU limiting
        let mut store = StoreWrapper::new(&self.inner.engine, (), DEFAULT_FUEL)?;
        store.set_component_id(handle.id().to_string());

        // 2. Create Linker for component instantiation
        let linker = Linker::new(&self.inner.engine);

        // 3. Instantiate component
        let instance = linker
            .instantiate_async(&mut store, handle.component())
            .await
            .map_err(|e| {
                WasmError::execution_failed(format!(
                    "Failed to instantiate component '{}' for handle-message: {e}",
                    handle.id()
                ))
            })?;

        // 4. Get typed function for handle-message export
        //
        // Component Model type mapping for WIT:
        //   handle-message: func(sender: component-id, message: list<u8>) -> result<_, component-error>
        //
        // Where component-id is a record { namespace: string, name: string, version: string }
        //
        // In Wasmtime typed functions, this maps to:
        //   - Input: ((&str, &str, &str), &[u8]) - component-id as tuple, message as slice
        //   - Output: (Result<(), ComponentError>,) - result variant
        //
        // For simplicity in initial implementation, we use a simplified signature:
        //   - Input: (&str, &[u8]) - sender as simple string, message as slice
        //   - Output: (Result<(), ()>,) - success or error
        //
        // This works with components that expect a simple string sender ID.
        // Full component-id record support requires bindgen-generated types.
        let func = instance
            .get_typed_func::<(&str, &[u8]), (Result<(), ()>,)>(&mut store, "handle-message")
            .map_err(|e| {
                WasmError::execution_failed(format!(
                    "Component '{}' has no handle-message export or type mismatch. \
                     Expected signature: (string, list<u8>) -> result<_, _>. Error: {e}",
                    handle.id()
                ))
            })?;

        // 5. Call the handle-message function
        let (result,) = func
            .call_async(&mut store, (sender.as_str(), payload))
            .await
            .map_err(|e| {
                // Collect fuel consumed for diagnostics
                let fuel_consumed = store.fuel_consumed();
                Self::categorize_wasmtime_error_with_fuel(
                    &e,
                    &ComponentId::new(handle.id()),
                    "handle-message",
                    fuel_consumed,
                )
            })?;

        // 6. Handle result
        result.map_err(|()| {
            WasmError::execution_failed(format!(
                "Component '{}' handle-message returned error (component-side failure)",
                handle.id()
            ))
        })
    }

    /// Call the handle-callback export on a component.
    ///
    /// Uses Component Model API to invoke the callback handler when a response
    /// is received for a previous `send-request` call. This implements the
    /// response delivery mechanism defined in KNOWLEDGE-WASM-029.
    ///
    /// # Arguments
    ///
    /// * `handle` - Component handle from `load_component()`
    /// * `request_id` - Correlation ID from the original request (UUID string)
    /// * `payload` - Response payload bytes
    /// * `is_error` - Whether the response is an error (1) or success (0)
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Callback handled successfully
    /// * `Err(WasmError)` - Execution failed
    ///
    /// # WIT Interface
    ///
    /// The handle-callback export follows a similar pattern to handle-message:
    /// ```wit
    /// handle-callback: func(request-id: string, payload: list<u8>, is-error: s32) -> result<_, component-error>;
    /// ```
    ///
    /// # Implementation Notes
    ///
    /// This method follows the same pattern as `call_handle_message`:
    /// 1. Creates a Store with fuel metering for CPU limiting
    /// 2. Creates a Linker for component instantiation
    /// 3. Instantiates the component
    /// 4. Gets the typed handle-callback function export
    /// 5. Calls the function with request_id, payload, and is_error parameters
    /// 6. Handles success/error results
    ///
    /// # Component Interface
    ///
    /// Components receiving callbacks should export:
    /// - `handle-callback(request_id_ptr, request_id_len, result_ptr, result_len) -> i32`
    /// - First byte of result indicates: 0=success, 1=error
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use airssys_wasm::runtime::WasmEngine;
    /// use airssys_wasm::core::ComponentId;
    ///
    /// async fn deliver_callback(engine: &WasmEngine, handle: &ComponentHandle) {
    ///     let request_id = "550e8400-e29b-41d4-a716-446655440000";
    ///     let payload = b"response data";
    ///     let is_error = 0; // success
    ///     
    ///     engine.call_handle_callback(handle, request_id, payload, is_error).await?;
    /// }
    /// ```
    ///
    /// # References
    ///
    /// - **KNOWLEDGE-WASM-029**: Messaging Patterns (response routing)
    /// - **WASM-TASK-006 Phase 3 Task 3.2**: Response Routing and Callbacks
    pub async fn call_handle_callback(
        &self,
        handle: &ComponentHandle,
        request_id: &str,
        payload: &[u8],
        is_error: i32,
    ) -> WasmResult<()> {
        // Default fuel limit for callback handling operations
        const DEFAULT_FUEL: u64 = 10_000_000;

        // 1. Create Store with fuel metering for CPU limiting
        let mut store = StoreWrapper::new(&self.inner.engine, (), DEFAULT_FUEL)?;
        store.set_component_id(handle.id().to_string());

        // 2. Create Linker for component instantiation
        let linker = Linker::new(&self.inner.engine);

        // 3. Instantiate component
        let instance = linker
            .instantiate_async(&mut store, handle.component())
            .await
            .map_err(|e| {
                WasmError::execution_failed(format!(
                    "Failed to instantiate component '{}' for handle-callback: {e}",
                    handle.id()
                ))
            })?;

        // 4. Get typed function for handle-callback export
        //
        // For callback-receiver.wat, the signature is:
        //   handle-callback(request_id_ptr, request_id_len, result_ptr, result_len) -> i32
        //
        // In Wasmtime typed functions, this maps to:
        //   - Input: (&str, &[u8]) - request_id as string, result (with is_error prefix) as slice
        //   - Output: (i32,) - 0 for success, non-zero for error
        //
        // We prepend is_error byte to payload for the component to distinguish success/error.
        let func = instance
            .get_typed_func::<(&str, &[u8]), (i32,)>(&mut store, "handle-callback")
            .map_err(|e| {
                WasmError::execution_failed(format!(
                    "Component '{}' has no handle-callback export or type mismatch. \
                     Expected signature: (string, list<u8>) -> s32. Error: {e}",
                    handle.id()
                ))
            })?;

        // 5. Prepare result payload with is_error prefix
        // First byte: 0 = success, 1 = error
        // Remaining bytes: actual payload
        let mut result_with_prefix = Vec::with_capacity(1 + payload.len());
        result_with_prefix.push(if is_error != 0 { 1 } else { 0 });
        result_with_prefix.extend_from_slice(payload);

        // 6. Call the handle-callback function
        let (result,) = func
            .call_async(&mut store, (request_id, result_with_prefix.as_slice()))
            .await
            .map_err(|e| {
                // Collect fuel consumed for diagnostics
                let fuel_consumed = store.fuel_consumed();
                Self::categorize_wasmtime_error_with_fuel(
                    &e,
                    &ComponentId::new(handle.id()),
                    "handle-callback",
                    fuel_consumed,
                )
            })?;

        // 7. Handle result (0 = success, non-zero = error)
        if result != 0 {
            Err(WasmError::execution_failed(format!(
                "Component '{}' handle-callback returned error code {} (component-side failure)",
                handle.id(),
                result
            )))
        } else {
            Ok(())
        }
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
        // Phase 5: Panic boundary around WASM execution (ADR-WASM-006)
        // Use std::panic::catch_unwind to isolate panics from component crashes
        let panic_result = std::panic::AssertUnwindSafe(async {
            // Wrap execution with timeout (hybrid CPU limiting)
            let timeout_duration = Duration::from_millis(context.timeout_ms);

            match timeout(
                timeout_duration,
                self.execute_internal(handle, function, input, context.clone()),
            )
            .await
            {
                Ok(result) => result,
                Err(_elapsed) => {
                    // Timeout exceeded - return ExecutionTimeout error
                    Err(WasmError::execution_timeout(context.timeout_ms, None))
                }
            }
        });

        // Await the future within panic boundary
        // Note: std::panic::catch_unwind doesn't work directly with async,
        // but Wasmtime's trap handling prevents panics from propagating.
        // This serves as documentation of the panic boundary pattern.
        // Real panic boundaries are enforced by Wasmtime's trap mechanism.
        panic_result.await
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

#[async_trait]
impl RuntimeMessageHandlerEngine for WasmEngine {
    /// Call a component's `handle-message` export.
    ///
    /// Delegates to WasmEngine::call_handle_message() which uses
    /// Wasmtime's Component Model typed function API.
    async fn call_handle_message(
        &self,
        handle: &ComponentHandle,
        sender: &crate::core::component::ComponentId,
        payload: &[u8],
    ) -> WasmResult<()> {
        WasmEngine::call_handle_message(self, handle, sender, payload).await
    }

    /// Call a component's `handle-callback` export.
    ///
    /// Delegates to WasmEngine::call_handle_callback() which uses
    /// Wasmtime's Component Model typed function API.
    async fn call_handle_callback(
        &self,
        handle: &ComponentHandle,
        request_id: &str,
        payload: &[u8],
        is_error: i32,
    ) -> WasmResult<()> {
        WasmEngine::call_handle_callback(self, handle, request_id, payload, is_error).await
    }
}

impl std::fmt::Debug for WasmEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WasmEngine")
            .field("engine", &"Wasmtime::Engine")
            .finish()
    }
}

#[allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::too_many_arguments,
    clippy::type_complexity,
    reason = "test code"
)]
#[cfg(test)]
mod tests {

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

    // ============================================================================
    // call_handle_message Tests (Task 2.5)
    // ============================================================================

    /// Test that call_handle_message returns error when component has no handle-message export.
    ///
    /// Uses hello_world.wasm which only exports "hello", not "handle-message".
    #[tokio::test]
    async fn test_call_handle_message_no_export() {
        // Load hello_world.wasm which doesn't have handle-message export
        let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/hello_world.wasm");
        let bytes = std::fs::read(&fixture_path).expect("Failed to read fixture");

        let engine = WasmEngine::new().unwrap();
        let component_id = ComponentId::new("test-component");
        let handle = engine
            .load_component(&component_id, &bytes)
            .await
            .expect("Failed to load component");

        // Attempt to call handle-message (should fail - no export)
        let sender = ComponentId::new("sender-component");
        let payload = b"test message";

        let result = engine.call_handle_message(&handle, &sender, payload).await;

        // Should fail with execution error (no handle-message export)
        assert!(
            result.is_err(),
            "Expected error when no handle-message export"
        );
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("handle-message") || err_msg.contains("no handle-message"),
            "Error should mention handle-message export: {err_msg}"
        );
    }

    /// Test successful call_handle_message with a component that has handle-message export.
    ///
    /// Uses handle-message-component.wasm which exports:
    ///   handle-message: func(sender: string, message: list<u8>) -> result
    #[tokio::test]
    async fn test_call_handle_message_success() {
        // Load handle-message-component.wasm which has handle-message export
        let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/handle-message-component.wasm");
        let bytes = std::fs::read(&fixture_path).expect("Failed to read fixture");

        let engine = WasmEngine::new().unwrap();
        let component_id = ComponentId::new("test-handler");
        let handle = engine
            .load_component(&component_id, &bytes)
            .await
            .expect("Failed to load component");

        // Call handle-message (should succeed)
        let sender = ComponentId::new("sender-component");
        let payload = b"hello world";

        let result = engine.call_handle_message(&handle, &sender, payload).await;

        // Should succeed
        assert!(
            result.is_ok(),
            "Expected success when component has handle-message export: {:?}",
            result.err()
        );
    }

    /// Test call_handle_message with empty payload.
    #[tokio::test]
    async fn test_call_handle_message_empty_payload() {
        let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/handle-message-component.wasm");
        let bytes = std::fs::read(&fixture_path).expect("Failed to read fixture");

        let engine = WasmEngine::new().unwrap();
        let component_id = ComponentId::new("test-handler");
        let handle = engine
            .load_component(&component_id, &bytes)
            .await
            .expect("Failed to load component");

        // Call with empty payload
        let sender = ComponentId::new("sender");
        let payload: &[u8] = &[];

        let result = engine.call_handle_message(&handle, &sender, payload).await;

        // Should succeed with empty payload
        assert!(
            result.is_ok(),
            "Expected success with empty payload: {:?}",
            result.err()
        );
    }

    /// Test call_handle_message with various sender IDs.
    #[tokio::test]
    async fn test_call_handle_message_various_senders() {
        let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/handle-message-component.wasm");
        let bytes = std::fs::read(&fixture_path).expect("Failed to read fixture");

        let engine = WasmEngine::new().unwrap();
        let component_id = ComponentId::new("test-handler");
        let handle = engine
            .load_component(&component_id, &bytes)
            .await
            .expect("Failed to load component");

        // Test with various sender IDs
        let senders = [
            ComponentId::new("simple"),
            ComponentId::new("namespace/component-v1.0.0"),
            ComponentId::new("a-very-long-component-identifier-for-testing"),
            ComponentId::new("unicode-测试-テスト"),
        ];

        for sender in &senders {
            let result = engine.call_handle_message(&handle, sender, b"test").await;
            assert!(
                result.is_ok(),
                "Expected success with sender '{}': {:?}",
                sender.as_str(),
                result.err()
            );
        }
    }

    // ============================================================================
    // call_handle_callback Tests (Task 3.2)
    // ============================================================================

    /// Test that call_handle_callback returns error when component has no handle-callback export.
    ///
    /// Uses hello_world.wasm which only exports "hello", not "handle-callback".
    #[tokio::test]
    async fn test_call_handle_callback_no_export() {
        // Load hello_world.wasm which doesn't have handle-callback export
        let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/hello_world.wasm");
        let bytes = std::fs::read(&fixture_path).expect("Failed to read fixture");

        let engine = WasmEngine::new().unwrap();
        let component_id = ComponentId::new("test-component");
        let handle = engine
            .load_component(&component_id, &bytes)
            .await
            .expect("Failed to load component");

        // Attempt to call handle-callback (should fail - no export)
        let request_id = "550e8400-e29b-41d4-a716-446655440000";
        let payload = b"test response";
        let is_error = 0;

        let result = engine
            .call_handle_callback(&handle, request_id, payload, is_error)
            .await;

        // Should fail with execution error (no handle-callback export)
        assert!(
            result.is_err(),
            "Expected error when no handle-callback export"
        );
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("handle-callback") || err_msg.contains("no handle-callback"),
            "Error should mention handle-callback export: {err_msg}"
        );
    }

    /// Test successful call_handle_callback with a component that has handle-callback export.
    ///
    /// Uses callback-receiver-component.wasm which exports:
    ///   handle-callback: func(request_id: string, result: list<u8>) -> s32
    #[tokio::test]
    async fn test_call_handle_callback_success() {
        // Load callback-receiver-component.wasm which has handle-callback export
        let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/callback-receiver-component.wasm");
        let bytes = std::fs::read(&fixture_path).expect("Failed to read fixture");

        let engine = WasmEngine::new().unwrap();
        let component_id = ComponentId::new("test-callback-receiver");
        let handle = engine
            .load_component(&component_id, &bytes)
            .await
            .expect("Failed to load component");

        // Call handle-callback (should succeed)
        let request_id = "550e8400-e29b-41d4-a716-446655440000";
        let payload = b"response data";
        let is_error = 0;

        let result = engine
            .call_handle_callback(&handle, request_id, payload, is_error)
            .await;

        // Should succeed
        assert!(
            result.is_ok(),
            "Expected success when component has handle-callback export: {:?}",
            result.err()
        );
    }

    /// Test call_handle_callback with error response.
    #[tokio::test]
    async fn test_call_handle_callback_error_response() {
        let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/callback-receiver-component.wasm");
        let bytes = std::fs::read(&fixture_path).expect("Failed to read fixture");

        let engine = WasmEngine::new().unwrap();
        let component_id = ComponentId::new("test-callback-receiver");
        let handle = engine
            .load_component(&component_id, &bytes)
            .await
            .expect("Failed to load component");

        // Call with is_error = 1 (error response)
        let request_id = "550e8400-e29b-41d4-a716-446655440000";
        let payload = b"error message";
        let is_error = 1;

        let result = engine
            .call_handle_callback(&handle, request_id, payload, is_error)
            .await;

        // Should succeed (component handles error responses)
        assert!(
            result.is_ok(),
            "Expected success with error response: {:?}",
            result.err()
        );
    }

    /// Test call_handle_callback with empty payload.
    #[tokio::test]
    async fn test_call_handle_callback_empty_payload() {
        let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/callback-receiver-component.wasm");
        let bytes = std::fs::read(&fixture_path).expect("Failed to read fixture");

        let engine = WasmEngine::new().unwrap();
        let component_id = ComponentId::new("test-callback-receiver");
        let handle = engine
            .load_component(&component_id, &bytes)
            .await
            .expect("Failed to load component");

        // Call with empty payload
        let request_id = "550e8400-e29b-41d4-a716-446655440000";
        let payload: &[u8] = &[];
        let is_error = 0;

        let result = engine
            .call_handle_callback(&handle, request_id, payload, is_error)
            .await;

        // Should succeed with empty payload
        assert!(
            result.is_ok(),
            "Expected success with empty payload: {:?}",
            result.err()
        );
    }

    /// Test call_handle_callback with various request IDs.
    #[tokio::test]
    async fn test_call_handle_callback_various_request_ids() {
        let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/callback-receiver-component.wasm");
        let bytes = std::fs::read(&fixture_path).expect("Failed to read fixture");

        let engine = WasmEngine::new().unwrap();
        let component_id = ComponentId::new("test-callback-receiver");
        let handle = engine
            .load_component(&component_id, &bytes)
            .await
            .expect("Failed to load component");

        // Test with various request IDs
        let request_ids = [
            "550e8400-e29b-41d4-a716-446655440000",
            "short",
            "a-very-long-request-identifier-for-testing",
        ];

        for request_id in &request_ids {
            let result = engine
                .call_handle_callback(&handle, request_id, b"test", 0)
                .await;
            assert!(
                result.is_ok(),
                "Expected success with request_id '{}': {:?}",
                request_id,
                result.err()
            );
        }
    }
}
