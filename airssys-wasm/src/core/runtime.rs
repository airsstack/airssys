//! Runtime execution abstractions for WebAssembly components.
//!
//! This module provides the core abstractions for WASM component execution,
//! including the runtime engine trait, execution context, and state management.
//! These types define the contract between the `core` module and the `runtime`
//! implementation block (Block 1).
//!
//! # Design Rationale
//!
//! Following ADR-WASM-012 (Core Abstractions Strategy), these abstractions prevent
//! circular dependencies by defining trait contracts that implementation blocks fulfill.
//! The `RuntimeEngine` trait allows `core` types to reference runtime execution
//! without depending on the Wasmtime-specific implementation.
//!
//! # Architecture
//!
//! ```text
//! core/runtime.rs (abstractions)
//!        ↓ trait contract
//! runtime/engine.rs (implementation with Wasmtime)
//! ```
//!
//! # Example
//!
//! ```rust
//! use std::collections::HashMap;
//! use airssys_wasm::core::{
//!     RuntimeEngine, ExecutionContext, ComponentId, ResourceLimits,
//!     CapabilitySet, ComponentInput,
//! };
//!
//! async fn execute_component<E: RuntimeEngine>(
//!     engine: &E,
//!     component_id: ComponentId,
//!     bytes: &[u8],
//! ) -> Result<(), Box<dyn std::error::Error>> {
//!     // Load component
//!     let handle = engine.load_component(&component_id, bytes).await?;
//!     
//!     // Create execution context
//!     let context = ExecutionContext {
//!         component_id: component_id.clone(),
//!         limits: ResourceLimits {
//!             max_memory_bytes: 64 * 1024 * 1024,
//!             max_fuel: 1_000_000,
//!             max_execution_ms: 5000,
//!             max_storage_bytes: 10 * 1024 * 1024,
//!         },
//!         capabilities: CapabilitySet::new(),
//!         timeout_ms: 5000,
//!     };
//!     
//!     // Execute function
//!     let input = ComponentInput {
//!         data: b"input data".to_vec(),
//!         codec: 0,
//!         metadata: HashMap::new(),
//!     };
//!     let output = engine.execute(&handle, "process", input, context).await?;
//!     
//!     println!("Output: {} bytes", output.data.len());
//!     Ok(())
//! }
//! ```

// Layer 1: Standard library imports
use std::sync::Arc;

// Layer 2: External crate imports
use async_trait::async_trait;
use wasmtime::component::Component;

// Layer 3: Internal module imports
use super::{
    capability::CapabilitySet,
    component::{ComponentId, ComponentInput, ComponentOutput},
    config::ResourceLimits,
    error::WasmResult,
};

/// Opaque handle to a loaded WebAssembly component.
///
/// This type is returned by [`RuntimeEngine::load_component`] and used to reference
/// the loaded component in subsequent operations. The actual implementation is
/// provided by the runtime engine (e.g., Wasmtime's `Component` handle).
///
/// # Design Rationale (Option A - WASM-TASK-002 Phase 3)
///
/// This handle stores `Arc<Component>` directly for simplicity:
/// - Component handle owns the component (no caching complexity)
/// - Simpler architecture following YAGNI principles
/// - Efficient cloning via Arc (cheap reference counting)
/// - No need for engine-level caching in Block 1
///
/// Using an opaque type (rather than exposing Wasmtime's types directly) allows:
/// - Core abstractions to remain independent of runtime implementation
/// - Future runtime engine swapping without breaking core API
/// - Implementation-specific optimizations (pooling, etc.) in future blocks
///
/// # Example
///
/// ```rust,ignore
/// let handle = engine.load_component(&component_id, bytes).await?;
/// let output = engine.execute(&handle, "process", input, context).await?;
/// ```
#[derive(Clone)]
pub struct ComponentHandle {
    /// Component identifier for logging and debugging.
    id: String,

    /// Compiled Wasmtime component (Arc for cheap cloning).
    component: Arc<Component>,
}

impl ComponentHandle {
    /// Create a new component handle with compiled component.
    ///
    /// This is called internally by runtime implementations after successful
    /// component compilation.
    #[doc(hidden)]
    pub fn new(id: impl Into<String>, component: Arc<Component>) -> Self {
        Self {
            id: id.into(),
            component,
        }
    }

    /// Get the component ID associated with this handle.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get reference to the underlying Wasmtime component.
    ///
    /// This is used internally by the runtime engine for instantiation.
    #[doc(hidden)]
    pub fn component(&self) -> &Component {
        &self.component
    }
}

impl std::fmt::Debug for ComponentHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComponentHandle")
            .field("id", &self.id)
            .field("component", &"Arc<Component>")
            .finish()
    }
}

/// Core runtime engine trait for WASM component execution.
///
/// This trait defines the contract for executing WebAssembly components.
/// It is implemented by `runtime::WasmEngine` using Wasmtime as the underlying
/// execution engine.
///
/// # Design
///
/// The trait is designed to be:
/// - **Async-first**: All operations support async/await for non-blocking execution
/// - **Send + Sync**: Thread-safe for concurrent execution across multiple components
/// - **Error-propagating**: Uses `WasmResult<T>` for structured error handling
/// - **Resource-aware**: Tracks memory, fuel, and execution time
///
/// # Implementation Notes
///
/// Implementors must ensure:
/// - Component loading is idempotent (same bytes → same component)
/// - Execution respects `ResourceLimits` and `timeout_ms`
/// - Capability checking delegates to security middleware
/// - Resource usage tracking is accurate and low-overhead
///
/// # Example
///
/// ```rust,ignore
/// struct MyEngine { /* ... */ }
///
/// #[async_trait]
/// impl RuntimeEngine for MyEngine {
///     async fn load_component(
///         &self,
///         component_id: &ComponentId,
///         bytes: &[u8],
///     ) -> WasmResult<ComponentHandle> {
///         // Parse and validate component
///         // Cache compiled module
///         // Return handle for future execution
///         todo!("Implementation in runtime/engine.rs")
///     }
///     
///     // ... other methods
/// }
/// ```
#[async_trait]
pub trait RuntimeEngine: Send + Sync {
    /// Load a component from bytes.
    ///
    /// Parses, validates, and compiles a WebAssembly component from raw bytes.
    /// The component is cached internally for efficient repeated execution.
    ///
    /// # Parameters
    ///
    /// - `component_id`: Unique identifier for the component
    /// - `bytes`: Component binary data (WASM Component Model format)
    ///
    /// # Returns
    ///
    /// - `Ok(ComponentHandle)`: Opaque handle for future execution
    /// - `Err(WasmError)`: Component loading failed (invalid format, compilation error, etc.)
    ///
    /// # Errors
    ///
    /// - `WasmError::ComponentLoadFailed`: Invalid component format or compilation failure
    /// - `WasmError::InvalidConfiguration`: Component exceeds configured limits
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let component_bytes = std::fs::read("component.wasm")?;
    /// let component_id = ComponentId::new("my-component");
    /// let handle = engine.load_component(&component_id, &component_bytes).await?;
    /// ```
    async fn load_component(
        &self,
        component_id: &ComponentId,
        bytes: &[u8],
    ) -> WasmResult<ComponentHandle>;

    /// Execute a component function.
    ///
    /// Invokes a function exported by the component with the given input data.
    /// Execution respects resource limits and capabilities specified in the context.
    ///
    /// # Parameters
    ///
    /// - `handle`: Component handle from `load_component`
    /// - `function`: Name of the exported function to invoke
    /// - `input`: Input data for the function
    /// - `context`: Execution context with limits and capabilities
    ///
    /// # Returns
    ///
    /// - `Ok(ComponentOutput)`: Function result data
    /// - `Err(WasmError)`: Execution failed (trap, timeout, capability denied, etc.)
    ///
    /// # Errors
    ///
    /// - `WasmError::ExecutionFailed`: Function invocation failed
    /// - `WasmError::ComponentTrapped`: Component execution trapped (panic, unreachable, etc.)
    /// - `WasmError::ExecutionTimeout`: Execution exceeded `context.timeout_ms`
    /// - `WasmError::ResourceLimitExceeded`: Memory or fuel limit exceeded
    /// - `WasmError::CapabilityDenied`: Function requires capabilities not in context
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let input = ComponentInput::new(b"hello");
    /// let context = ExecutionContext {
    ///     component_id: component_id.clone(),
    ///     limits: ResourceLimits::default(),
    ///     capabilities: CapabilitySet::new(),
    ///     timeout_ms: 5000,
    /// };
    ///
    /// let output = engine.execute(&handle, "process", input, context).await?;
    /// println!("Result: {:?}", output.data());
    /// ```
    async fn execute(
        &self,
        handle: &ComponentHandle,
        function: &str,
        input: ComponentInput,
        context: ExecutionContext,
    ) -> WasmResult<ComponentOutput>;

    /// Get resource usage statistics.
    ///
    /// Returns current resource consumption for a loaded component.
    /// Used for monitoring, debugging, and resource management.
    ///
    /// # Parameters
    ///
    /// - `handle`: Component handle from `load_component`
    ///
    /// # Returns
    ///
    /// `ResourceUsage` struct with memory, fuel, and timing statistics.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let usage = engine.resource_usage(&handle);
    /// println!("Memory: {} bytes", usage.memory_bytes);
    /// println!("Fuel: {} units", usage.fuel_consumed);
    /// println!("Time: {} ms", usage.execution_time_ms);
    /// ```
    fn resource_usage(&self, handle: &ComponentHandle) -> ResourceUsage;
}

/// Execution context passed to runtime engine.
///
/// Contains all information needed for secure, resource-controlled component execution.
/// Passed to `RuntimeEngine::execute()` to configure execution environment.
///
/// # Fields
///
/// - `component_id`: Component identity for logging and debugging
/// - `limits`: Resource limits (memory, fuel)
/// - `capabilities`: Granted capabilities for permission checking
/// - `timeout_ms`: Maximum execution time in milliseconds
///
/// # Example
///
/// ```rust
/// use airssys_wasm::core::{
///     ExecutionContext, ComponentId, ResourceLimits, CapabilitySet, Capability, PathPattern,
/// };
///
/// let context = ExecutionContext {
///     component_id: ComponentId::new("my-component"),
///     limits: ResourceLimits {
///         max_memory_bytes: 64 * 1024 * 1024,
///         max_fuel: 1_000_000,
///         max_execution_ms: 5000,
///         max_storage_bytes: 10 * 1024 * 1024,
///     },
///     capabilities: CapabilitySet::from_vec(vec![
///         Capability::FileRead(PathPattern::new("data/*")),
///     ]),
///     timeout_ms: 10_000, // 10 seconds
/// };
/// ```
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Component identifier for this execution.
    ///
    /// Used for logging, debugging, and security audit trails.
    pub component_id: ComponentId,

    /// Resource limits for this execution.
    ///
    /// Defines maximum memory and fuel consumption.
    /// See `ResourceLimits` for default values and configuration.
    pub limits: ResourceLimits,

    /// Granted capabilities for this execution.
    ///
    /// Security middleware validates function calls against this set.
    /// Empty set means component has no special permissions.
    pub capabilities: CapabilitySet,

    /// Maximum execution time in milliseconds.
    ///
    /// Execution is terminated if this timeout is exceeded.
    /// Prevents infinite loops and resource exhaustion.
    ///
    /// # Recommended Values
    ///
    /// - Fast operations: 100-1000ms
    /// - I/O operations: 5000-10000ms
    /// - Long-running tasks: 30000-60000ms
    pub timeout_ms: u64,
}

/// Runtime state machine.
///
/// Tracks the lifecycle state of a component execution.
/// Used for monitoring, debugging, and error handling.
///
/// # State Transitions
///
/// ```text
/// Idle → Loading → Executing → Completed
///                         ↓
///                    Trapped/TimedOut
/// ```
///
/// # Example
///
/// ```rust
/// use airssys_wasm::core::ExecutionState;
///
/// let state = ExecutionState::Idle;
/// assert_eq!(state, ExecutionState::Idle);
///
/// match state {
///     ExecutionState::Idle => println!("Ready to execute"),
///     ExecutionState::Executing => println!("Running..."),
///     ExecutionState::Completed => println!("Done"),
///     _ => println!("Error state"),
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionState {
    /// Component is idle, ready to execute.
    Idle,

    /// Component is being loaded and compiled.
    Loading,

    /// Component function is currently executing.
    Executing,

    /// Component execution trapped (panic, unreachable, etc.).
    Trapped,

    /// Component execution exceeded timeout.
    TimedOut,

    /// Component execution completed successfully.
    Completed,
}

/// Resource usage statistics.
///
/// Tracks resource consumption during component execution.
/// Used for monitoring, profiling, and resource management.
///
/// # Example
///
/// ```rust
/// use airssys_wasm::core::ResourceUsage;
///
/// let usage = ResourceUsage {
///     memory_bytes: 1024 * 1024, // 1 MB
///     fuel_consumed: 1000000,
///     execution_time_ms: 150,
/// };
///
/// if usage.memory_bytes > 10 * 1024 * 1024 {
///     println!("Warning: High memory usage");
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    /// Current memory usage in bytes.
    ///
    /// Includes linear memory, tables, and runtime overhead.
    pub memory_bytes: u64,

    /// Fuel consumed during execution.
    ///
    /// Fuel is a Wasmtime mechanism for metering execution cost.
    /// Higher fuel = more computation performed.
    pub fuel_consumed: u64,

    /// Execution time in milliseconds.
    ///
    /// Wall-clock time from function invocation to completion.
    pub execution_time_ms: u64,
}

#[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic, clippy::indexing_slicing, clippy::too_many_arguments, clippy::type_complexity, reason = "test code")]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::capability::{Capability, PathPattern};

    fn test_resource_limits() -> ResourceLimits {
        ResourceLimits {
            max_memory_bytes: 64 * 1024 * 1024,
            max_fuel: 1_000_000,
            timeout_seconds: 5,
        }
    }

    #[test]
    fn test_execution_context_creation() {
        let context = ExecutionContext {
            component_id: ComponentId::new("test-component"),
            limits: test_resource_limits(),
            capabilities: CapabilitySet::new(),
            timeout_ms: 5000,
        };

        assert_eq!(context.component_id.as_str(), "test-component");
        assert_eq!(context.timeout_ms, 5000);
        assert!(context.capabilities.is_empty());
    }

    #[test]
    fn test_execution_state_transitions() {
        let state = ExecutionState::Idle;
        assert_eq!(state, ExecutionState::Idle);

        let state = ExecutionState::Loading;
        assert_eq!(state, ExecutionState::Loading);

        let state = ExecutionState::Executing;
        assert_eq!(state, ExecutionState::Executing);

        let state = ExecutionState::Completed;
        assert_eq!(state, ExecutionState::Completed);
    }

    #[test]
    fn test_execution_state_error_cases() {
        let state = ExecutionState::Trapped;
        assert_eq!(state, ExecutionState::Trapped);

        let state = ExecutionState::TimedOut;
        assert_eq!(state, ExecutionState::TimedOut);
    }

    #[test]
    fn test_resource_usage_tracking() {
        let usage = ResourceUsage {
            memory_bytes: 1024,
            fuel_consumed: 500,
            execution_time_ms: 100,
        };

        assert_eq!(usage.memory_bytes, 1024);
        assert_eq!(usage.fuel_consumed, 500);
        assert_eq!(usage.execution_time_ms, 100);
    }

    #[test]
    fn test_execution_context_with_capabilities() {
        let mut capabilities = CapabilitySet::new();
        capabilities.grant(Capability::FileRead(PathPattern::new("data/*")));

        let context = ExecutionContext {
            component_id: ComponentId::new("secure-component"),
            limits: test_resource_limits(),
            capabilities,
            timeout_ms: 10_000,
        };

        assert!(!context.capabilities.is_empty());
        assert!(context
            .capabilities
            .has(&Capability::FileRead(PathPattern::new("data/*"))));
    }

    #[test]
    fn test_execution_context_clone() {
        let context = ExecutionContext {
            component_id: ComponentId::new("clone-test"),
            limits: test_resource_limits(),
            capabilities: CapabilitySet::new(),
            timeout_ms: 3000,
        };

        let cloned = context.clone();
        assert_eq!(cloned.component_id.as_str(), "clone-test");
        assert_eq!(cloned.timeout_ms, 3000);
    }

    #[test]
    fn test_resource_usage_clone() {
        let usage = ResourceUsage {
            memory_bytes: 2048,
            fuel_consumed: 1000,
            execution_time_ms: 200,
        };

        let cloned = usage.clone();
        assert_eq!(cloned.memory_bytes, 2048);
        assert_eq!(cloned.fuel_consumed, 1000);
        assert_eq!(cloned.execution_time_ms, 200);
    }
}
