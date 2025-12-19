//! ComponentActor: Bridge between WASM components and airssys-rt actor system.
//!
//! This module implements the `ComponentActor` struct which serves as the core integration
//! point between WebAssembly components and the airssys-rt actor system. ComponentActor
//! implements both `Actor` (for message handling) and `Child` (for WASM lifecycle) traits,
//! establishing the foundational actor-hosted component architecture.
//!
//! # Architecture
//!
//! ComponentActor follows the dual-trait pattern documented in ADR-WASM-006:
//! - **Actor trait**: Handles inter-component messages via mailbox
//! - **Child trait**: Manages WASM runtime lifecycle under supervisor control
//!
//! ```text
//! ┌────────────────────────────────────┐
//! │      ComponentActor                │
//! │  ┌──────────────┐  ┌────────────┐ │
//! │  │ Actor trait  │  │ Child trait│ │
//! │  │ (messaging)  │  │ (lifecycle)│ │
//! │  └──────────────┘  └────────────┘ │
//! │         ↓                ↓         │
//! │  ┌──────────────────────────────┐ │
//! │  │   WasmRuntime (Optional)     │ │
//! │  │   - Wasmtime Engine/Store    │ │
//! │  │   - Component Exports        │ │
//! │  └──────────────────────────────┘ │
//! └────────────────────────────────────┘
//! ```
//!
//! # Lifecycle States
//!
//! ComponentActor follows a well-defined state machine:
//! ```text
//! Creating → Starting → Ready → Stopping → Terminated
//!                ↓                   ↓
//!           Failed(reason)      Failed(reason)
//! ```
//!
//! # Usage Example
//!
//! ```rust,ignore
//! use airssys_wasm::actor::{ComponentActor, ComponentMessage};
//! use airssys_wasm::core::{ComponentId, ComponentSpec, CapabilitySet};
//! use airssys_rt::supervisor::Child;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create ComponentActor
//!     let component_id = ComponentId::new("my-component");
//!     let spec = ComponentSpec::default();
//!     let caps = CapabilitySet::new();
//!     
//!     let mut actor = ComponentActor::new(component_id.clone(), spec, caps);
//!     
//!     // Start WASM runtime (Child::start)
//!     actor.start().await?;
//!     
//!     // Actor is now ready to process messages
//!     
//!     // Graceful shutdown (Child::stop)
//!     actor.stop(Duration::from_secs(5)).await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! # References
//!
//! - **ADR-WASM-006**: Component Isolation and Sandboxing (Actor-based approach)
//! - **ADR-RT-004**: Actor and Child Trait Separation
//! - **KNOWLEDGE-WASM-016**: Actor System Integration Implementation Guide
//! - **Task**: WASM-TASK-004 Phase 1 Task 1.1

// Layer 1: Standard library imports
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

// Layer 2: Third-party crate imports
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::RwLock;
use wasmtime::{Engine, Instance, Store};

// Layer 3: Internal module imports
use crate::core::{CapabilitySet, ComponentId, ComponentMetadata, WasmError};

/// WASM runtime managing Wasmtime engine, store, and component instance.
///
/// WasmRuntime encapsulates all Wasmtime resources for a single component instance,
/// providing controlled access to the engine, store, and exported functions. This
/// struct implements RAII resource cleanup through Drop.
///
/// # Architecture
///
/// ```text
/// ┌─────────────────────────────────┐
/// │       WasmRuntime               │
/// ├─────────────────────────────────┤
/// │ - engine: Engine                │ ← Compilation engine
/// │ - store: Store<ResourceLimiter> │ ← Memory + fuel management
/// │ - instance: Instance            │ ← Component exports
/// │ - exports: WasmExports          │ ← Cached function exports
/// └─────────────────────────────────┘
/// ```
///
/// # Resource Management
///
/// - **Engine**: Shared compilation engine (could be shared across components in future)
/// - **Store**: Per-component memory and fuel tracking with ResourceLimiter
/// - **Instance**: Per-component WASM instance with exports
/// - **Exports**: Cached function handles for performance
///
/// # Lifecycle
///
/// 1. Created during Child::start() after successful instantiation
/// 2. Used during Actor message handling for function calls
/// 3. Dropped during Child::stop() to free all resources
///
/// # Examples
///
/// ```rust,ignore
/// // Created in Child::start()
/// let runtime = WasmRuntime::new(engine, store, instance)?;
///
/// // Used in Actor::handle_message()
/// runtime.exports().call_start(runtime.store_mut()).await?;
///
/// // Dropped in Child::stop()
/// drop(runtime); // Frees all WASM resources
/// ```
#[allow(dead_code)] // Engine will be used in future actor message handling
pub struct WasmRuntime {
    /// Wasmtime compilation engine
    engine: Engine,

    /// Wasmtime store with resource limiter
    store: Store<ComponentResourceLimiter>,

    /// Component instance with exports
    instance: Instance,

    /// Cached function exports
    exports: WasmExports,
}

/// Cached WASM function exports for performance.
///
/// WasmExports stores Optional function handles for common component exports,
/// enabling fast lookup without repeated export resolution. All exports are
/// optional per WASM Component Model conventions.
///
/// # Standard Exports
///
/// - **_start**: Component initialization (called once after instantiation)
/// - **_cleanup**: Graceful shutdown (called before component termination)
/// - **_health**: Health check reporting (periodic monitoring)
/// - **handle-message**: Inter-component message handler (Actor trait)
///
/// # Performance
///
/// Caching exports avoids repeated `get_func()` calls during message handling,
/// reducing Actor message processing latency.
///
/// # Examples
///
/// ```rust,ignore
/// // Extract exports from instance
/// let exports = WasmExports::extract(&instance, &mut store)?;
///
/// // Call optional _start
/// exports.call_start(&mut store).await?;
///
/// // Check if handle-message exists
/// if exports.handle_message.is_some() {
///     // Component supports Actor message handling
/// }
/// ```
pub struct WasmExports {
    /// Optional _start export (component initialization)
    pub start: Option<wasmtime::Func>,

    /// Optional _cleanup export (graceful shutdown)
    pub cleanup: Option<wasmtime::Func>,

    /// Optional _health export (health reporting)
    pub health: Option<wasmtime::Func>,

    /// Optional handle-message export (Actor message handling)
    pub handle_message: Option<wasmtime::Func>,
}

/// Per-component resource limiter implementing Wasmtime ResourceLimiter trait.
///
/// ComponentResourceLimiter enforces memory and fuel limits for individual
/// component instances, preventing resource exhaustion and enabling fair
/// resource sharing across components.
///
/// # Resource Types
///
/// - **Memory**: Linear memory allocation limit (max_memory_bytes)
/// - **Fuel**: CPU execution limit (max_fuel)
/// - **Tables**: WASM table growth (allowed by default)
///
/// # Integration
///
/// Used as the `T` parameter in `Store<T>`, allowing Wasmtime to call
/// resource check callbacks during component execution.
///
/// # Thread Safety
///
/// Uses AtomicU64 for thread-safe current memory tracking, though Wasmtime
/// stores are not currently Send/Sync (future consideration).
///
/// # Examples
///
/// ```rust,ignore
/// let limits = ComponentResourceLimiter::new(
///     64 * 1024 * 1024,  // 64MB memory
///     1_000_000,         // 1M fuel
/// );
///
/// let mut store = Store::new(&engine, limits);
/// store.set_fuel(1_000_000)?;
/// ```
#[allow(dead_code)] // max_fuel will be used for fuel_consumed callbacks in future
pub struct ComponentResourceLimiter {
    /// Maximum memory in bytes
    max_memory: u64,

    /// Maximum fuel (CPU limit)
    max_fuel: u64,

    /// Current memory usage (atomic for thread safety)
    current_memory: Arc<AtomicU64>,
}

impl ComponentResourceLimiter {
    /// Create a new resource limiter with specified limits.
    ///
    /// # Parameters
    ///
    /// * `max_memory` - Maximum memory in bytes
    /// * `max_fuel` - Maximum fuel (CPU limit)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let limiter = ComponentResourceLimiter::new(64 * 1024 * 1024, 1_000_000);
    /// ```
    pub fn new(max_memory: u64, max_fuel: u64) -> Self {
        Self {
            max_memory,
            max_fuel,
            current_memory: Arc::new(AtomicU64::new(0)),
        }
    }
}

impl wasmtime::ResourceLimiter for ComponentResourceLimiter {
    fn memory_growing(
        &mut self,
        current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> anyhow::Result<bool> {
        let new_total = current.saturating_add(desired) as u64;

        if new_total <= self.max_memory {
            self.current_memory.store(new_total, Ordering::Relaxed);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn table_growing(
        &mut self,
        _current: u32,
        _desired: u32,
        _maximum: Option<u32>,
    ) -> anyhow::Result<bool> {
        // Table growth allowed by default (tables don't consume linear memory)
        Ok(true)
    }
}

impl WasmExports {
    /// Extract function exports from a component instance.
    ///
    /// Resolves optional standard exports (_start, _cleanup, _health, handle-message)
    /// and caches them for fast access during component lifecycle and message handling.
    ///
    /// # Parameters
    ///
    /// * `instance` - Component instance to extract exports from
    /// * `store` - Mutable store reference for export resolution
    ///
    /// # Returns
    ///
    /// WasmExports with cached function handles (None for missing exports).
    ///
    /// # Errors
    ///
    /// Returns WasmError if export resolution fails (should not happen for optional exports).
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let exports = WasmExports::extract(&instance, &mut store)?;
    /// if let Some(start_fn) = &exports.start {
    ///     // Component has _start export
    /// }
    /// ```
    pub fn extract(
        instance: &Instance,
        store: &mut Store<ComponentResourceLimiter>,
    ) -> Result<Self, WasmError> {
        Ok(Self {
            start: instance.get_func(&mut *store, "_start"),
            cleanup: instance.get_func(&mut *store, "_cleanup"),
            health: instance.get_func(&mut *store, "_health"),
            handle_message: instance.get_func(&mut *store, "handle-message"),
        })
    }

    /// Call optional _start export.
    ///
    /// Invokes the component's _start export if present. This function is called
    /// once after instantiation to allow component initialization.
    ///
    /// # Parameters
    ///
    /// * `start_fn` - Optional _start function from exports
    /// * `store` - Mutable store reference for function execution
    ///
    /// # Returns
    ///
    /// Ok(()) if _start succeeds or is not present.
    ///
    /// # Errors
    ///
    /// Returns WasmError::ExecutionFailed if _start execution fails.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// WasmExports::call_start_fn(exports.start.as_ref(), &mut store).await?;
    /// ```
    pub async fn call_start_fn(
        start_fn: Option<&wasmtime::Func>,
        store: &mut Store<ComponentResourceLimiter>,
    ) -> Result<(), WasmError> {
        if let Some(func) = start_fn {
            func.call_async(store, &[], &mut []).await.map_err(|e| {
                WasmError::execution_failed(format!("Component _start function failed: {e}"))
            })?;
        }
        Ok(())
    }

    /// Call optional _cleanup export with timeout protection.
    ///
    /// Invokes the component's _cleanup export if present, with a configurable
    /// timeout to prevent hanging during shutdown. Timeout or execution errors
    /// are non-fatal (logged but don't prevent resource cleanup).
    ///
    /// # Parameters
    ///
    /// * `cleanup_fn` - Optional _cleanup function from exports
    /// * `store` - Mutable store reference for function execution
    /// * `timeout` - Maximum time to wait for cleanup completion
    ///
    /// # Returns
    ///
    /// Ok(()) if cleanup succeeds, is not present, or times out (non-fatal).
    ///
    /// # Errors
    ///
    /// Returns WasmError::ExecutionFailed if cleanup execution fails (non-fatal).
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// WasmExports::call_cleanup_fn(exports.cleanup.as_ref(), &mut store, Duration::from_secs(5)).await?;
    /// ```
    pub async fn call_cleanup_fn(
        cleanup_fn: Option<&wasmtime::Func>,
        store: &mut Store<ComponentResourceLimiter>,
        timeout: Duration,
    ) -> Result<(), WasmError> {
        if let Some(func) = cleanup_fn {
            match tokio::time::timeout(timeout, func.call_async(store, &[], &mut [])).await {
                Ok(Ok(())) => Ok(()),
                Ok(Err(e)) => Err(WasmError::execution_failed(format!(
                    "Component _cleanup function failed: {e}"
                ))),
                Err(_) => Err(WasmError::execution_timeout(
                    timeout.as_millis() as u64,
                    None,
                )),
            }
        } else {
            Ok(())
        }
    }
}

impl WasmRuntime {
    /// Create a new WasmRuntime from Wasmtime components.
    ///
    /// Wraps Wasmtime engine, store, and instance into a managed runtime
    /// with cached exports for performance.
    ///
    /// # Parameters
    ///
    /// * `engine` - Wasmtime compilation engine
    /// * `store` - Wasmtime store with resource limiter
    /// * `instance` - Component instance with exports
    ///
    /// # Returns
    ///
    /// WasmRuntime ready for component execution.
    ///
    /// # Errors
    ///
    /// Returns WasmError if export extraction fails.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let runtime = WasmRuntime::new(engine, store, instance)?;
    /// ```
    pub fn new(
        engine: Engine,
        mut store: Store<ComponentResourceLimiter>,
        instance: Instance,
    ) -> Result<Self, WasmError> {
        let exports = WasmExports::extract(&instance, &mut store)?;

        Ok(Self {
            engine,
            store,
            instance,
            exports,
        })
    }

    /// Get mutable reference to the store.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let store = runtime.store_mut();
    /// ```
    pub fn store_mut(&mut self) -> &mut Store<ComponentResourceLimiter> {
        &mut self.store
    }

    /// Get reference to the instance.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let instance = runtime.instance();
    /// ```
    pub fn instance(&self) -> &Instance {
        &self.instance
    }

    /// Get reference to cached exports.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let exports = runtime.exports();
    /// ```
    pub fn exports(&self) -> &WasmExports {
        &self.exports
    }
}

impl Drop for WasmRuntime {
    fn drop(&mut self) {
        // Wasmtime automatically cleans up:
        // - Store drop: frees linear memory
        // - Engine drop: frees compilation cache
        // - Instance drop: frees export handles
        tracing::debug!("WasmRuntime dropped - all WASM resources freed");
    }
}

/// Central component execution unit: Actor for messaging, Child for WASM lifecycle.
///
/// ComponentActor bridges WebAssembly components with the airssys-rt actor system,
/// implementing both `Actor` (message handling) and `Child` (lifecycle management)
/// traits. This dual-trait pattern is foundational to the actor-hosted component
/// architecture.
///
/// # Fields
///
/// - `component_id`: Unique identifier for this component instance
/// - `wasm_runtime`: WASM runtime (None until Child::start() loads WASM)
/// - `capabilities`: Security capabilities and permissions
/// - `state`: Current actor state (Creating → Starting → Ready → Stopping → Terminated)
/// - `metadata`: Component metadata (name, version, author, etc.)
/// - `mailbox_rx`: Mailbox receiver (managed by ActorSystem)
/// - `created_at`: Creation timestamp
/// - `started_at`: Start timestamp (None until Child::start() completes)
///
/// # Design Rationale
///
/// - **`Option<WasmRuntime>`**: Allows safe handling of unloaded state (WASM loads in Child::start())
/// - **State Machine**: ActorState tracks lifecycle transitions for monitoring
/// - **Timestamps**: Track creation and start time for uptime calculations
/// - **Mailbox Integration**: Receiver stored for message processing loop
///
/// # Performance Characteristics
///
/// Target performance (from WASM-TASK-004):
/// - **Spawn time**: <5ms average (includes WASM loading)
/// - **Message throughput**: >10,000 messages/sec per component
/// - **Memory footprint**: <2MB per instance
///
/// # Generic Parameters
///
/// - `S`: Custom state type for this component (default: `()` for no state)
///   - Must be `Send + Sync + 'static` for thread-safe actor execution
///   - State is protected by `Arc<RwLock<S>>` for concurrent access
///
/// # Example (No State)
///
/// ```rust
/// use airssys_wasm::actor::{ComponentActor, ActorState};
/// use airssys_wasm::core::{ComponentId, ComponentMetadata, CapabilitySet, ResourceLimits};
///
/// let component_id = ComponentId::new("example-component");
/// let metadata = ComponentMetadata {
///     name: "example-component".to_string(),
///     version: "1.0.0".to_string(),
///     author: "Test Author".to_string(),
///     description: None,
///     required_capabilities: vec![],
///     resource_limits: ResourceLimits {
///         max_memory_bytes: 64 * 1024 * 1024,
///         max_fuel: 1_000_000,
///         max_execution_ms: 5000,
///         max_storage_bytes: 10 * 1024 * 1024,
///     },
/// };
/// let caps = CapabilitySet::new();
///
/// // Default generic parameter () for no state
/// let actor = ComponentActor::new(component_id.clone(), metadata, caps, ());
///
/// assert_eq!(actor.component_id(), &component_id);
/// assert_eq!(*actor.state(), ActorState::Creating);
/// assert!(!actor.is_wasm_loaded());
/// ```
///
/// # Example (With Custom State)
///
/// ```rust,ignore
/// use airssys_wasm::actor::ComponentActor;
///
/// #[derive(Default)]
/// struct MyComponentState {
///     request_count: u64,
///     last_error: Option<String>,
/// }
///
/// let actor: ComponentActor<MyComponentState> = ComponentActor::new(
///     component_id,
///     metadata,
///     caps,
///     MyComponentState::default(),
/// );
///
/// // Access state
/// actor.with_state_mut(|state| state.request_count += 1).await;
/// ```
#[allow(dead_code)] // Fields will be used in future tasks
pub struct ComponentActor<S = ()>
where
    S: Send + Sync + 'static,
{
    /// Unique component identifier
    component_id: ComponentId,

    /// WASM runtime instance (None until Child::start())
    wasm_runtime: Option<WasmRuntime>,

    /// Component capabilities and permissions
    capabilities: CapabilitySet,

    /// Current actor state
    state: ActorState,

    /// Component metadata
    metadata: ComponentMetadata,

    /// Mailbox receiver (created by ActorSystem)
    mailbox_rx: Option<UnboundedReceiver<ComponentMessage>>,

    /// Creation timestamp
    created_at: DateTime<Utc>,

    /// Start timestamp (None until Child::start())
    started_at: Option<DateTime<Utc>>,

    /// MessageBroker bridge (set during spawn)
    broker: Option<Arc<dyn crate::actor::message::MessageBrokerBridge>>,

    /// Correlation tracker for request-response patterns (Phase 5 Task 5.1)
    correlation_tracker: Option<Arc<crate::actor::message::CorrelationTracker>>,

    /// Custom component state (generic type S)
    ///
    /// Thread-safe state storage using `Arc<RwLock<S>>`. Access via:
    /// - `with_state()` - Read-only access
    /// - `with_state_mut()` - Mutable access
    /// - `state()` - Direct read lock
    /// - `state_mut()` - Direct write lock
    ///
    /// Default generic `()` provides zero-overhead no-state variant.
    custom_state: Arc<RwLock<S>>,

    /// Lifecycle hooks for extensibility (Phase 5 Task 5.2)
    ///
    /// Hooks are called at key lifecycle events:
    /// - `pre_start()` / `post_start()` - Before/after WASM instantiation
    /// - `pre_stop()` / `post_stop()` - Before/after cleanup
    /// - `on_message_received()` - Before message routing
    /// - `on_error()` - On any error
    /// - `on_restart()` - On supervisor restart
    ///
    /// Default: `NoOpHooks` (zero overhead)
    hooks: Box<dyn crate::actor::lifecycle::LifecycleHooks>,

    /// Event callback for monitoring (Phase 5 Task 5.2)
    ///
    /// Optional callback for observability:
    /// - `on_message_received()` - Message arrival
    /// - `on_message_processed()` - Message completion with latency
    /// - `on_error_occurred()` - Error events
    /// - `on_restart_triggered()` - Restart events
    /// - `on_health_changed()` - Health status changes
    ///
    /// Default: None (no callbacks)
    event_callback: Option<Arc<dyn crate::actor::lifecycle::EventCallback>>,

    /// Rate limiter for inter-component messages (DEBT-WASM-004 Item #3).
    ///
    /// Tracks message rate per sender to prevent DoS attacks via message flooding.
    /// Uses sliding window algorithm with default 1000 msg/sec limit.
    rate_limiter: crate::core::rate_limiter::MessageRateLimiter,

    /// Security configuration (DEBT-WASM-004 Item #3).
    ///
    /// Contains security settings including:
    /// - Security mode (Strict, Permissive, Development)
    /// - Audit logging flag
    /// - Max message size (default 1MB)
    /// - Capability check timeout
    security_config: crate::core::config::SecurityConfig,

    /// WASM security context (WASM-TASK-005 Phase 4 Task 4.1).
    ///
    /// Encapsulates component-specific security context including:
    /// - Component ID (maps to ACL identity)
    /// - WASM capability set (Filesystem, Network, Storage)
    ///
    /// This context is **immutable** after component spawn to prevent runtime
    /// privilege escalation. It bridges WASM capability declarations from
    /// Component.toml to airssys-osl ACL/RBAC policies.
    ///
    /// The security context is:
    /// - Initialized during component spawn
    /// - Preserved across supervisor restarts
    /// - Isolated per component (no capability sharing)
    /// - Used by host functions for capability checks
    security_context: crate::security::WasmSecurityContext,
}

/// Actor lifecycle state machine.
///
/// ComponentActor transitions through well-defined states:
/// - **Creating**: Initial state after construction
/// - **Starting**: Child::start() in progress (loading WASM)
/// - **Ready**: WASM loaded, actor processing messages
/// - **Stopping**: Child::stop() in progress (cleanup)
/// - **Terminated**: Actor stopped, resources released
/// - **Failed**: Unrecoverable error occurred
///
/// # State Transitions
///
/// ```text
/// Creating --[Child::start()]--> Starting --[success]--> Ready
///                                    ↓
///                              Failed(reason)
///
/// Ready --[Child::stop()]--> Stopping --[success]--> Terminated
///          ↓                       ↓
///     Failed(reason)          Failed(reason)
/// ```
///
/// # Example
///
/// ```rust
/// use airssys_wasm::actor::ActorState;
///
/// let state = ActorState::Creating;
/// assert_eq!(state, ActorState::Creating);
///
/// let failed = ActorState::Failed("WASM load error".to_string());
/// match failed {
///     ActorState::Failed(reason) => println!("Failed: {}", reason),
///     _ => {}
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActorState {
    /// Initial state after construction
    Creating,

    /// Child::start() in progress (loading WASM)
    Starting,

    /// WASM loaded, actor ready to process messages
    Ready,

    /// Child::stop() in progress (cleanup)
    Stopping,

    /// Actor stopped, resources released
    Terminated,

    /// Unrecoverable error occurred
    Failed(String),
}

/// Component message types for actor communication.
///
/// ComponentMessage defines all message types that ComponentActor can handle.
/// Messages are processed sequentially by the actor's message handler.
///
/// # Message Types
///
/// - **Invoke**: Call a WASM function with arguments
/// - **InvokeResult**: Result of a function invocation (request-response pattern)
/// - **InterComponent**: Message from another component
/// - **Shutdown**: Signal to stop the actor
/// - **HealthCheck**: Request health status
/// - **HealthStatus**: Health status response
///
/// # Multicodec Encoding
///
/// Invoke and InterComponent messages use multicodec-prefixed payloads
/// (ADR-WASM-001) supporting Borsh, CBOR, and JSON codecs.
///
/// # Example
///
/// ```rust
/// use airssys_wasm::actor::ComponentMessage;
/// use airssys_wasm::core::ComponentId;
///
/// // Invoke WASM function
/// let msg = ComponentMessage::Invoke {
///     function: "process_data".to_string(),
///     args: vec![1, 2, 3, 4], // Multicodec-encoded
/// };
///
/// // Inter-component message
/// let sender = ComponentId::new("sender-component");
/// let msg = ComponentMessage::InterComponent {
///     sender,
///     payload: vec![0x70, 0x01, 0x00, 0x01], // Multicodec Borsh
/// };
///
/// // Health check
/// let msg = ComponentMessage::HealthCheck;
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentMessage {
    /// Invoke WASM function with arguments.
    ///
    /// Arguments are multicodec-encoded (ADR-WASM-001).
    Invoke {
        /// Function name to invoke
        function: String,
        /// Multicodec-encoded arguments
        args: Vec<u8>,
    },

    /// Result of invoke (for request-response pattern).
    InvokeResult {
        /// Multicodec-encoded result
        result: Vec<u8>,
        /// Error message if invocation failed
        error: Option<String>,
    },

    /// Message from another component.
    InterComponent {
        /// Sender component ID
        sender: ComponentId,
        /// Multicodec-encoded payload
        payload: Vec<u8>,
    },

    /// Message from another component with correlation ID (request-response).
    ///
    /// Used for request-response patterns where the sender expects a correlated
    /// response message. The correlation_id allows matching responses to requests.
    InterComponentWithCorrelation {
        /// Sender component ID
        sender: ComponentId,
        /// Multicodec-encoded payload
        payload: Vec<u8>,
        /// Correlation ID for request-response pattern
        correlation_id: uuid::Uuid,
    },

    /// Signal to shutdown the actor.
    Shutdown,

    /// Request health status.
    HealthCheck,

    /// Health status response.
    HealthStatus(HealthStatus),
}

/// Component health status for monitoring and supervision.
///
/// HealthStatus represents the operational state of a component, used by both
/// internal health checks and external monitoring systems. This enum supports
/// serialization via Borsh (binary), CBOR (binary), and JSON (text).
///
/// # Serialization Formats
///
/// **Borsh (Recommended):**
/// ```text
/// Healthy:    [0x00]
/// Degraded:   [0x01, len_u32, reason_bytes...]
/// Unhealthy:  [0x02, len_u32, reason_bytes...]
/// ```
///
/// **JSON:**
/// ```json
/// { "status": "healthy" }
/// { "status": "degraded", "reason": "High latency" }
/// { "status": "unhealthy", "reason": "Database unreachable" }
/// ```
///
/// **CBOR:** Binary equivalent of JSON structure
///
/// # Example
///
/// ```rust
/// use airssys_wasm::actor::HealthStatus;
/// use serde_json;
///
/// let status = HealthStatus::Degraded {
///     reason: "High memory usage".to_string(),
/// };
///
/// // JSON serialization
/// let json = serde_json::to_string(&status).unwrap();
/// assert!(json.contains("degraded"));
///
/// // Deserialization
/// let parsed: HealthStatus = serde_json::from_str(&json).unwrap();
/// assert!(matches!(parsed, HealthStatus::Degraded { .. }));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "status", content = "reason", rename_all = "lowercase")]
pub enum HealthStatus {
    /// Component operating normally
    #[serde(rename = "healthy")]
    Healthy,

    /// Component operational but experiencing issues
    #[serde(rename = "degraded")]
    Degraded {
        /// Reason for degraded status
        reason: String,
    },

    /// Component failed or non-functional
    #[serde(rename = "unhealthy")]
    Unhealthy {
        /// Reason for unhealthy status
        reason: String,
    },
}

// Borsh serialization for compact binary format
impl borsh::BorshSerialize for HealthStatus {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        match self {
            HealthStatus::Healthy => borsh::BorshSerialize::serialize(&0u8, writer),
            HealthStatus::Degraded { reason } => {
                borsh::BorshSerialize::serialize(&1u8, writer)?;
                borsh::BorshSerialize::serialize(reason, writer)
            }
            HealthStatus::Unhealthy { reason } => {
                borsh::BorshSerialize::serialize(&2u8, writer)?;
                borsh::BorshSerialize::serialize(reason, writer)
            }
        }
    }
}

impl borsh::BorshDeserialize for HealthStatus {
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        let variant = borsh::BorshDeserialize::deserialize(buf)?;
        match variant {
            0u8 => Ok(HealthStatus::Healthy),
            1u8 => Ok(HealthStatus::Degraded {
                reason: borsh::BorshDeserialize::deserialize(buf)?,
            }),
            2u8 => Ok(HealthStatus::Unhealthy {
                reason: borsh::BorshDeserialize::deserialize(buf)?,
            }),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid HealthStatus variant: {}", variant),
            )),
        }
    }

    fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let mut slice = buf.as_slice();
        <Self as borsh::BorshDeserialize>::deserialize(&mut slice)
    }
}

impl<S> ComponentActor<S>
where
    S: Send + Sync + 'static,
{
    /// Create a new ComponentActor instance with custom state.
    ///
    /// Creates a ComponentActor in the `Creating` state with the provided initial state.
    /// WASM runtime is not loaded until `Child::start()` is called.
    ///
    /// # Arguments
    ///
    /// * `component_id` - Unique identifier for this component
    /// * `metadata` - Component metadata
    /// * `capabilities` - Security capabilities and permissions
    /// * `initial_state` - Initial custom state value (generic type S)
    ///
    /// # Returns
    ///
    /// A new ComponentActor instance in Creating state with initialized custom state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::actor::ComponentActor;
    /// use airssys_wasm::core::{ComponentId, ComponentMetadata, CapabilitySet, ResourceLimits};
    ///
    /// let component_id = ComponentId::new("my-component");
    /// let metadata = ComponentMetadata {
    ///     name: "my-component".to_string(),
    ///     version: "1.0.0".to_string(),
    ///     author: "Test Author".to_string(),
    ///     description: None,
    ///     required_capabilities: vec![],
    ///     resource_limits: ResourceLimits {
    ///         max_memory_bytes: 64 * 1024 * 1024,
    ///         max_fuel: 1_000_000,
    ///         max_execution_ms: 5000,
    ///         max_storage_bytes: 10 * 1024 * 1024,
    ///     },
    /// };
    /// let caps = CapabilitySet::new();
    ///
    /// // No custom state
    /// let actor = ComponentActor::new(component_id.clone(), metadata, caps, ());
    ///
    /// assert_eq!(actor.component_id(), &component_id);
    /// assert!(!actor.is_wasm_loaded());
    /// ```
    ///
    /// # Example (With Custom State)
    ///
    /// ```rust,ignore
    /// #[derive(Default)]
    /// struct MyState {
    ///     request_count: u64,
    /// }
    ///
    /// let actor: ComponentActor<MyState> = ComponentActor::new(
    ///     component_id,
    ///     metadata,
    ///     caps,
    ///     MyState::default(),
    /// );
    /// ```
    pub fn new(
        component_id: ComponentId,
        metadata: ComponentMetadata,
        capabilities: CapabilitySet,
        initial_state: S,
    ) -> Self {
        // Create WASM security context with empty capability set
        // Real capabilities will be set via with_security_context() builder method
        let security_context = crate::security::WasmSecurityContext::new(
            component_id.as_str().to_string(),
            crate::security::WasmCapabilitySet::new(),
        );

        Self {
            component_id,
            wasm_runtime: None,
            capabilities,
            state: ActorState::Creating,
            metadata,
            mailbox_rx: None,
            created_at: Utc::now(),
            started_at: None,
            broker: None,
            correlation_tracker: None,
            custom_state: Arc::new(RwLock::new(initial_state)),
            hooks: Box::new(crate::actor::lifecycle::NoOpHooks),
            event_callback: None,
            rate_limiter: crate::core::rate_limiter::MessageRateLimiter::default(),
            security_config: crate::core::config::SecurityConfig::default(),
            security_context,
        }
    }

    /// Get the component ID.
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::actor::ComponentActor;
    /// use airssys_wasm::core::{ComponentId, ComponentMetadata, CapabilitySet, ResourceLimits};
    ///
    /// let component_id = ComponentId::new("test");
    /// let metadata = ComponentMetadata {
    ///     name: "test".to_string(),
    ///     version: "1.0.0".to_string(),
    ///     author: "Test".to_string(),
    ///     description: None,
    ///     required_capabilities: vec![],
    ///     resource_limits: ResourceLimits {
    ///         max_memory_bytes: 64 * 1024 * 1024,
    ///         max_fuel: 1_000_000,
    ///         max_execution_ms: 5000,
    ///         max_storage_bytes: 10 * 1024 * 1024,
    ///     },
    /// };
    /// let actor = ComponentActor::new(
    ///     component_id.clone(),
    ///     metadata,
    ///     CapabilitySet::new()
    /// );
    ///
    /// assert_eq!(actor.component_id(), &component_id);
    /// ```
    pub fn component_id(&self) -> &ComponentId {
        &self.component_id
    }

    /// Get the current actor state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::actor::{ComponentActor, ActorState};
    /// use airssys_wasm::core::{ComponentId, ComponentMetadata, CapabilitySet, ResourceLimits};
    ///
    /// let metadata = ComponentMetadata {
    ///     name: "test".to_string(),
    ///     version: "1.0.0".to_string(),
    ///     author: "Test".to_string(),
    ///     description: None,
    ///     required_capabilities: vec![],
    ///     resource_limits: ResourceLimits {
    ///         max_memory_bytes: 64 * 1024 * 1024,
    ///         max_fuel: 1_000_000,
    ///         max_execution_ms: 5000,
    ///         max_storage_bytes: 10 * 1024 * 1024,
    ///     },
    /// };
    /// let actor = ComponentActor::new(
    ///     ComponentId::new("test"),
    ///     metadata,
    ///     CapabilitySet::new()
    /// );
    ///
    /// assert_eq!(*actor.state(), ActorState::Creating);
    /// ```
    pub fn state(&self) -> &ActorState {
        &self.state
    }

    /// Set the actor state (internal use by trait implementations).
    ///
    /// This method is public but primarily for internal use by Actor and Child
    /// trait implementations. External code should not normally call this.
    #[doc(hidden)]
    pub fn set_state(&mut self, state: ActorState) {
        self.state = state;
    }

    /// Set the started_at timestamp (internal use by Child::start()).
    ///
    /// This method is public but primarily for internal use by the Child trait
    /// implementation. External code should not normally call this.
    #[doc(hidden)]
    pub fn set_started_at(&mut self, timestamp: Option<DateTime<Utc>>) {
        self.started_at = timestamp;
    }

    /// Clear the WASM runtime (internal use by Child::stop()).
    ///
    /// This method is public but primarily for internal use by the Child trait
    /// implementation. External code should not normally call this.
    #[doc(hidden)]
    pub fn clear_wasm_runtime(&mut self) {
        self.wasm_runtime = None;
    }

    /// Check if WASM runtime is loaded.
    ///
    /// Returns true if Child::start() has successfully loaded the WASM runtime.
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::actor::ComponentActor;
    /// use airssys_wasm::core::{ComponentId, ComponentMetadata, CapabilitySet, ResourceLimits};
    ///
    /// let metadata = ComponentMetadata {
    ///     name: "test".to_string(),
    ///     version: "1.0.0".to_string(),
    ///     author: "Test".to_string(),
    ///     description: None,
    ///     required_capabilities: vec![],
    ///     resource_limits: ResourceLimits {
    ///         max_memory_bytes: 64 * 1024 * 1024,
    ///         max_fuel: 1_000_000,
    ///         max_execution_ms: 5000,
    ///         max_storage_bytes: 10 * 1024 * 1024,
    ///     },
    /// };
    /// let actor = ComponentActor::new(
    ///     ComponentId::new("test"),
    ///     metadata,
    ///     CapabilitySet::new()
    /// );
    ///
    /// // WASM not loaded until Child::start()
    /// assert!(!actor.is_wasm_loaded());
    /// ```
    pub fn is_wasm_loaded(&self) -> bool {
        self.wasm_runtime.is_some()
    }

    /// Calculate component uptime.
    ///
    /// Returns the duration since Child::start() completed successfully.
    /// Returns None if the component has not started yet.
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::actor::ComponentActor;
    /// use airssys_wasm::core::{ComponentId, ComponentMetadata, CapabilitySet, ResourceLimits};
    ///
    /// let metadata = ComponentMetadata {
    ///     name: "test".to_string(),
    ///     version: "1.0.0".to_string(),
    ///     author: "Test".to_string(),
    ///     description: None,
    ///     required_capabilities: vec![],
    ///     resource_limits: ResourceLimits {
    ///         max_memory_bytes: 64 * 1024 * 1024,
    ///         max_fuel: 1_000_000,
    ///         max_execution_ms: 5000,
    ///         max_storage_bytes: 10 * 1024 * 1024,
    ///     },
    /// };
    /// let actor = ComponentActor::new(
    ///     ComponentId::new("test"),
    ///     metadata,
    ///     CapabilitySet::new()
    /// );
    ///
    /// // No uptime before start
    /// assert_eq!(actor.uptime(), None);
    /// ```
    pub fn uptime(&self) -> Option<chrono::Duration> {
        self.started_at.map(|started| Utc::now() - started)
    }

    /// Get the component's security context.
    ///
    /// Returns a reference to the `WasmSecurityContext` containing the component's
    /// unique identifier and granted capabilities. This context is immutable after
    /// component spawn to prevent privilege escalation.
    ///
    /// # Returns
    ///
    /// Reference to the component's `WasmSecurityContext`
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Get component security context
    /// let context = actor.security_context();
    /// println!("Component ID: {}", context.component_id);
    ///
    /// // Convert to ACL entries for capability check
    /// let acl_entries = context.capabilities.to_acl_entries(&context.component_id);
    /// ```
    ///
    /// # Implementation Note (WASM-TASK-005 Phase 4 Task 4.1)
    ///
    /// This method provides read-only access to the security context. The context
    /// is set during component spawn and preserved across supervisor restarts.
    /// Components cannot modify their own security context at runtime.
    pub fn security_context(&self) -> &crate::security::WasmSecurityContext {
        &self.security_context
    }

    /// Set the component's security context (builder pattern).
    ///
    /// Replaces the component's security context with a new one. This is typically
    /// called during component initialization before spawn, or during supervisor
    /// restart to restore the previous security context.
    ///
    /// # Arguments
    ///
    /// * `context` - New security context to set
    ///
    /// # Returns
    ///
    /// `self` for method chaining (builder pattern)
    ///
    /// # Security Considerations
    ///
    /// This method should only be called:
    /// 1. During component initialization (before spawn)
    /// 2. During supervisor restart (restoring previous context)
    ///
    /// **Never** call this method after a component is running, as it would
    /// allow privilege escalation by changing capabilities at runtime.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use airssys_wasm::security::{WasmCapability, WasmCapabilitySet, WasmSecurityContext};
    ///
    /// // Create security context with capabilities
    /// let capabilities = WasmCapabilitySet::new()
    ///     .grant(WasmCapability::Filesystem {
    ///         paths: vec!["/app/data/*".to_string()],
    ///         permissions: vec!["read".to_string()],
    ///     });
    ///
    /// let context = WasmSecurityContext::new(
    ///     "my-component".to_string(),
    ///     capabilities,
    /// );
    ///
    /// // Set security context during initialization
    /// let actor = ComponentActor::new(
    ///     ComponentId::new("my-component"),
    ///     metadata,
    ///     CapabilitySet::new(),
    ///     (),
    /// )
    /// .with_security_context(context);
    /// ```
    ///
    /// # Implementation Note (WASM-TASK-005 Phase 4 Task 4.1)
    ///
    /// This builder method enables fluent API for component construction while
    /// maintaining security context immutability after spawn.
    pub fn with_security_context(mut self, context: crate::security::WasmSecurityContext) -> Self {
        self.security_context = context;
        self
    }

    /// Load WASM component bytes from storage.
    ///
    /// # TODO(Block 6 - Component Storage System)
    ///
    /// This is a stub implementation. Block 6 (Component Storage System) will
    /// provide the actual storage backend integration for loading components
    /// from filesystem, registry, or remote sources.
    ///
    /// For now, returns `WasmError::ComponentNotFound` to indicate storage
    /// integration is pending.
    ///
    /// # Future Implementation
    ///
    /// Block 6 will implement:
    /// - Filesystem-based component loading
    /// - Component registry integration
    /// - Remote component fetching
    /// - Component caching and versioning
    ///
    /// # Returns
    ///
    /// - `Ok(Vec<u8>)`: WASM bytecode (future)
    /// - `Err(WasmError::ComponentNotFound)`: Storage not implemented (current)
    ///
    /// # Errors
    ///
    /// Currently always returns `WasmError::ComponentNotFound`.
    ///
    /// Future errors:
    /// - ComponentNotFound: Component doesn't exist in storage
    /// - StorageError: Storage backend failure
    /// - IoError: Filesystem read error
    /// - SerializationError: Component manifest parse error
    pub(crate) async fn load_component_bytes(&self) -> Result<Vec<u8>, WasmError> {
        // Test mode: Return minimal valid WASM module for testing
        #[cfg(test)]
        {
            // Minimal valid WASM module (empty module with correct magic/version)
            // Magic: \0asm
            // Version: 0x01 0x00 0x00 0x00
            Ok(vec![
                0x00, 0x61, 0x73, 0x6D, // Magic number: \0asm
                0x01, 0x00, 0x00, 0x00, // Version: 1
            ])
        }

        // Production mode: Return error until Block 6 is implemented
        #[cfg(not(test))]
        Err(WasmError::component_not_found(format!(
            "Component storage integration pending (Block 6) - component_id: {}",
            self.component_id.as_str()
        )))
    }

    /// Get reference to component metadata.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let metadata = actor.metadata();
    /// println!("Memory limit: {}", metadata.resource_limits.max_memory_bytes);
    /// ```
    pub(crate) fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    /// Set the WASM runtime (internal use by Child::start()).
    ///
    /// This method is public but primarily for internal use by the Child trait
    /// implementation. External code should not normally call this.
    #[doc(hidden)]
    pub fn set_wasm_runtime(&mut self, runtime: Option<WasmRuntime>) {
        self.wasm_runtime = runtime;
    }

    /// Get mutable reference to WASM runtime (internal use).
    ///
    /// This method is public but primarily for internal use by trait implementations.
    #[doc(hidden)]
    pub fn wasm_runtime_mut(&mut self) -> Option<&mut WasmRuntime> {
        self.wasm_runtime.as_mut()
    }

    /// Get reference to WASM runtime (internal use).
    ///
    /// This method is public but primarily for internal use by trait implementations.
    #[doc(hidden)]
    pub fn wasm_runtime(&self) -> Option<&WasmRuntime> {
        self.wasm_runtime.as_ref()
    }

    /// Set MessageBroker bridge (called by ComponentSpawner).
    ///
    /// This method is called during component spawning to inject the MessageBroker
    /// bridge, enabling the component to publish and subscribe to topics.
    ///
    /// # Arguments
    ///
    /// * `broker` - MessageBrokerBridge implementation
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use std::sync::Arc;
    /// use airssys_wasm::actor::{ComponentActor, MessageBrokerWrapper};
    ///
    /// let mut actor = ComponentActor::new(/* ... */);
    /// let broker_wrapper = Arc::new(MessageBrokerWrapper::new(broker));
    /// actor.set_broker(broker_wrapper);
    /// ```
    pub fn set_broker(&mut self, broker: Arc<dyn crate::actor::message::MessageBrokerBridge>) {
        self.broker = Some(broker);
    }

    /// Publish message to topic.
    ///
    /// Publishes a ComponentMessage to the specified topic via MessageBroker.
    /// All subscribers to the topic will receive the message (fire-and-forget
    /// semantics per ADR-WASM-009).
    ///
    /// # Arguments
    ///
    /// * `topic` - Topic name (e.g., "events", "notifications.user")
    /// * `message` - ComponentMessage to publish
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Message published successfully
    /// - `Err(WasmError::BrokerNotConfigured)`: Broker not set
    /// - `Err(WasmError::MessageBrokerError)`: Publish failed
    ///
    /// # Errors
    ///
    /// Returns `WasmError::BrokerNotConfigured` if set_broker() was not called.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use airssys_wasm::actor::{ComponentActor, ComponentMessage};
    /// use airssys_wasm::core::ComponentId;
    ///
    /// async fn publish_event(actor: &ComponentActor) -> Result<(), WasmError> {
    ///     let message = ComponentMessage::InterComponent {
    ///         sender: ComponentId::new("component-a"),
    ///         payload: vec![1, 2, 3],
    ///     };
    ///     
    ///     actor.publish_message("events.user.login", message).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn publish_message(
        &self,
        topic: &str,
        message: ComponentMessage,
    ) -> Result<(), WasmError> {
        let broker = self.broker.as_ref().ok_or_else(|| {
            WasmError::broker_not_configured(
                "MessageBroker not configured - call set_broker() first",
            )
        })?;

        broker.publish(topic, message).await
    }

    /// Subscribe to topic.
    ///
    /// Subscribes this component to a topic, receiving all messages published
    /// to it. Returns a subscription handle for tracking and unsubscribe operations.
    ///
    /// # Arguments
    ///
    /// * `topic` - Topic name to subscribe to
    ///
    /// # Returns
    ///
    /// - `Ok(SubscriptionHandle)`: Subscription successful
    /// - `Err(WasmError::BrokerNotConfigured)`: Broker not set
    /// - `Err(WasmError::MessageBrokerError)`: Subscribe failed
    ///
    /// # Errors
    ///
    /// Returns `WasmError::BrokerNotConfigured` if set_broker() was not called.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use airssys_wasm::actor::ComponentActor;
    ///
    /// async fn subscribe_to_events(actor: &mut ComponentActor) -> Result<(), WasmError> {
    ///     let handle = actor.subscribe_topic("events.user").await?;
    ///     // Store handle for later unsubscribe
    ///     Ok(())
    /// }
    /// ```
    pub async fn subscribe_topic(
        &mut self,
        topic: &str,
    ) -> Result<crate::actor::message::SubscriptionHandle, WasmError> {
        let broker = self.broker.as_ref().ok_or_else(|| {
            WasmError::broker_not_configured(
                "MessageBroker not configured - call set_broker() first",
            )
        })?;

        broker.subscribe(topic, &self.component_id).await
    }

    /// Set correlation tracker for request-response patterns (Phase 5 Task 5.1).
    ///
    /// Configures the correlation tracker used for managing request-response
    /// correlation IDs and timeouts. Must be called before using send_request().
    ///
    /// # Arguments
    ///
    /// * `tracker` - Shared correlation tracker instance
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use airssys_wasm::actor::{ComponentActor, CorrelationTracker};
    /// use std::sync::Arc;
    ///
    /// let mut actor = ComponentActor::new(/* ... */);
    /// let tracker = Arc::new(CorrelationTracker::new());
    /// actor.set_correlation_tracker(tracker);
    /// ```
    pub fn set_correlation_tracker(
        &mut self,
        tracker: Arc<crate::actor::message::CorrelationTracker>,
    ) {
        self.correlation_tracker = Some(tracker);
    }

    /// Send request with correlation tracking and timeout (Phase 5 Task 5.1).
    ///
    /// Sends a request message to a target component with automatic correlation
    /// ID generation, timeout enforcement, and response callback delivery via
    /// oneshot channel.
    ///
    /// # Arguments
    ///
    /// * `target` - Target component ID
    /// * `request` - Request payload (will be multicodec-encoded)
    /// * `timeout` - Timeout duration
    ///
    /// # Returns
    ///
    /// Oneshot receiver for response (or timeout error)
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - CorrelationTracker not configured (call set_correlation_tracker() first)
    /// - Broker not configured (call set_broker() first)
    /// - Multicodec encoding fails
    /// - Request registration fails (duplicate correlation ID)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use airssys_wasm::actor::ComponentActor;
    /// use airssys_wasm::core::ComponentId;
    /// use std::time::Duration;
    ///
    /// async fn query_user(actor: &ComponentActor, user_id: &str) -> Result<Vec<u8>, WasmError> {
    ///     let target = ComponentId::new("user-service");
    ///     let request = encode_multicodec(&UserQuery { user_id: user_id.to_string() })?;
    ///     
    ///     // Send request with 5 second timeout
    ///     let response_rx = actor.send_request(&target, request, Duration::from_secs(5)).await?;
    ///     
    ///     // Wait for response
    ///     match response_rx.await {
    ///         Ok(response) => {
    ///             match response.result {
    ///                 Ok(payload) => Ok(payload),
    ///                 Err(e) => Err(WasmError::internal(format!("Request failed: {}", e))),
    ///             }
    ///         }
    ///         Err(_) => Err(WasmError::internal("Response channel closed")),
    ///     }
    /// }
    /// ```
    pub async fn send_request(
        &self,
        target: &ComponentId,
        payload: Vec<u8>,
        timeout: std::time::Duration,
    ) -> Result<tokio::sync::oneshot::Receiver<crate::actor::message::ResponseMessage>, WasmError>
    {
        use crate::actor::message::{CorrelationId, RequestMessage};
        use tokio::sync::oneshot;
        use tokio::time::Instant;

        // Verify correlation tracker configured
        let tracker = self.correlation_tracker.as_ref().ok_or_else(|| {
            WasmError::internal(
                "CorrelationTracker not configured - call set_correlation_tracker() first",
            )
        })?;

        // Generate correlation ID
        let correlation_id = CorrelationId::new_v4();

        // Create oneshot channel for response
        let (response_tx, response_rx) = oneshot::channel();

        // Register pending request (this is an internal type, we need to make it public or use a builder pattern)
        // For now, we'll register directly with the internal struct
        let pending_request = crate::actor::message::correlation_tracker::PendingRequest {
            correlation_id,
            response_tx,
            requested_at: Instant::now(),
            timeout: tokio::time::Duration::from_millis(timeout.as_millis() as u64),
            from: self.component_id.clone(),
            to: target.clone(),
        };

        tracker.register_pending(pending_request).await?;

        // Create request message
        let request_msg = RequestMessage::new(
            self.component_id.clone(),
            target.clone(),
            payload,
            timeout.as_millis() as u32,
        );

        // Publish request message via MessageBroker
        let message = ComponentMessage::InterComponentWithCorrelation {
            sender: self.component_id.clone(),
            payload: serde_json::to_vec(&request_msg)
                .map_err(|e| WasmError::internal(format!("Failed to serialize request: {}", e)))?,
            correlation_id,
        };

        self.publish_message("requests", message).await?;

        Ok(response_rx)
    }

    /// Send response to correlated request (Phase 5 Task 5.1).
    ///
    /// Sends a response message matching a previous request's correlation ID.
    /// The response will be delivered to the original requester via the
    /// correlation tracker.
    ///
    /// # Arguments
    ///
    /// * `correlation_id` - Correlation ID from incoming request
    /// * `result` - Response payload or error
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - CorrelationTracker not configured
    /// - Correlation ID not found (already resolved or timed out)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use airssys_wasm::actor::ComponentActor;
    ///
    /// async fn handle_request(
    ///     actor: &ComponentActor,
    ///     correlation_id: uuid::Uuid,
    ///     request_payload: Vec<u8>,
    /// ) -> Result<(), WasmError> {
    ///     // Process request
    ///     let response_payload = process_request(&request_payload)?;
    ///     
    ///     // Send response
    ///     actor.send_response(correlation_id, Ok(response_payload)).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn send_response(
        &self,
        correlation_id: uuid::Uuid,
        result: Result<Vec<u8>, crate::actor::message::RequestError>,
    ) -> Result<(), WasmError> {
        use crate::actor::message::ResponseMessage;
        use chrono::Utc;

        // Verify correlation tracker configured
        let tracker = self.correlation_tracker.as_ref().ok_or_else(|| {
            WasmError::internal(
                "CorrelationTracker not configured - call set_correlation_tracker() first",
            )
        })?;

        // Create response message
        let response = ResponseMessage {
            correlation_id,
            from: self.component_id.clone(),
            to: self.component_id.clone(), // Will be filled by tracker
            result,
            timestamp: Utc::now(),
        };

        // Resolve pending request
        tracker.resolve(correlation_id, response).await?;

        Ok(())
    }

    /// Execute a closure with read-only access to custom state.
    ///
    /// Acquires a read lock on the custom state and executes the provided closure
    /// with a reference to the state. Multiple readers can access state concurrently.
    ///
    /// # Type Parameters
    ///
    /// * `F` - Closure type: `FnOnce(&S) -> R`
    /// * `R` - Return type of the closure
    ///
    /// # Arguments
    ///
    /// * `f` - Closure to execute with state reference
    ///
    /// # Returns
    ///
    /// The value returned by the closure.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// #[derive(Default)]
    /// struct MyState {
    ///     count: u64,
    /// }
    ///
    /// let actor: ComponentActor<MyState> = ComponentActor::new(/* ... */, MyState::default());
    ///
    /// // Read state
    /// let count = actor.with_state(|state| state.count).await;
    /// ```
    pub async fn with_state<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&S) -> R,
    {
        let guard = self.custom_state.read().await;
        f(&*guard)
    }

    /// Execute a closure with mutable access to custom state.
    ///
    /// Acquires a write lock on the custom state and executes the provided closure
    /// with a mutable reference to the state. Only one writer can access state at a time.
    ///
    /// # Type Parameters
    ///
    /// * `F` - Closure type: `FnOnce(&mut S) -> R`
    /// * `R` - Return type of the closure
    ///
    /// # Arguments
    ///
    /// * `f` - Closure to execute with mutable state reference
    ///
    /// # Returns
    ///
    /// The value returned by the closure.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// #[derive(Default)]
    /// struct MyState {
    ///     count: u64,
    /// }
    ///
    /// let actor: ComponentActor<MyState> = ComponentActor::new(/* ... */, MyState::default());
    ///
    /// // Modify state
    /// actor.with_state_mut(|state| {
    ///     state.count += 1;
    /// }).await;
    /// ```
    pub async fn with_state_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut S) -> R,
    {
        let mut guard = self.custom_state.write().await;
        f(&mut *guard)
    }

    /// Get a clone of the custom state.
    ///
    /// Acquires a read lock and returns a clone of the state. Only available
    /// when S implements Clone.
    ///
    /// # Returns
    ///
    /// Cloned copy of the custom state.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// #[derive(Clone, Default)]
    /// struct MyState {
    ///     count: u64,
    /// }
    ///
    /// let actor: ComponentActor<MyState> = ComponentActor::new(/* ... */, MyState::default());
    ///
    /// // Get cloned state
    /// let state_copy = actor.get_state().await;
    /// ```
    pub async fn get_state(&self) -> S
    where
        S: Clone,
    {
        let guard = self.custom_state.read().await;
        (*guard).clone()
    }

    /// Replace the custom state with a new value.
    ///
    /// Acquires a write lock and replaces the entire state with the new value.
    /// Returns the old state value.
    ///
    /// # Arguments
    ///
    /// * `new_state` - New state value to set
    ///
    /// # Returns
    ///
    /// The previous state value.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// #[derive(Default)]
    /// struct MyState {
    ///     count: u64,
    /// }
    ///
    /// let actor: ComponentActor<MyState> = ComponentActor::new(/* ... */, MyState::default());
    ///
    /// // Replace state
    /// let old_state = actor.set_state(MyState { count: 42 }).await;
    /// ```
    pub async fn set_custom_state(&self, new_state: S) -> S {
        let mut guard = self.custom_state.write().await;
        std::mem::replace(&mut *guard, new_state)
    }

    /// Get Arc clone of the custom state RwLock.
    ///
    /// Returns an Arc clone of the underlying RwLock, allowing shared ownership
    /// of the state. Useful for passing state references to other components.
    ///
    /// # Returns
    ///
    /// Arc-wrapped RwLock of the custom state.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let state_ref = actor.state_arc();
    ///
    /// // Can share this Arc with other tasks
    /// tokio::spawn(async move {
    ///     let guard = state_ref.read().await;
    ///     // Use state...
    /// });
    /// ```
    pub fn state_arc(&self) -> Arc<RwLock<S>> {
        Arc::clone(&self.custom_state)
    }

    /// Set custom lifecycle hooks for this component.
    ///
    /// Replaces the default NoOpHooks with a custom implementation. Hooks are
    /// called at key lifecycle events (pre/post-start/stop, on_message, on_error).
    ///
    /// # Arguments
    ///
    /// * `hooks` - Custom LifecycleHooks implementation
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use airssys_wasm::actor::lifecycle::{LifecycleHooks, LifecycleContext, HookResult};
    ///
    /// struct MyHooks;
    ///
    /// impl LifecycleHooks for MyHooks {
    ///     fn pre_start(&mut self, ctx: &LifecycleContext) -> HookResult {
    ///         println!("Component starting: {}", ctx.component_id.as_str());
    ///         HookResult::Ok
    ///     }
    /// }
    ///
    /// let mut actor = ComponentActor::new(/* ... */);
    /// actor.set_lifecycle_hooks(Box::new(MyHooks));
    /// ```
    pub fn set_lifecycle_hooks(&mut self, hooks: Box<dyn crate::actor::lifecycle::LifecycleHooks>) {
        self.hooks = hooks;
    }

    /// Set event callback for monitoring this component.
    ///
    /// Registers an optional callback that receives lifecycle events for
    /// observability (message received/processed, errors, restarts, health changes).
    ///
    /// # Arguments
    ///
    /// * `callback` - EventCallback implementation
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use airssys_wasm::actor::lifecycle::{EventCallback};
    /// use std::sync::Arc;
    ///
    /// struct MyCallback;
    ///
    /// impl EventCallback for MyCallback {
    ///     fn on_message_received(&self, component_id: ComponentId) {
    ///         println!("Message received: {}", component_id.as_str());
    ///     }
    /// }
    ///
    /// let mut actor = ComponentActor::new(/* ... */);
    /// actor.set_event_callback(Arc::new(MyCallback));
    /// ```
    pub fn set_event_callback(
        &mut self,
        callback: Arc<dyn crate::actor::lifecycle::EventCallback>,
    ) {
        self.event_callback = Some(callback);
    }

    /// Get reference to lifecycle hooks (internal use).
    ///
    /// Used by trait implementations to access hooks during lifecycle events.
    #[doc(hidden)]
    pub fn hooks_mut(&mut self) -> &mut Box<dyn crate::actor::lifecycle::LifecycleHooks> {
        &mut self.hooks
    }

    /// Get reference to event callback (internal use).
    ///
    /// Used by trait implementations to fire callback events.
    #[doc(hidden)]
    pub fn event_callback(&self) -> Option<&Arc<dyn crate::actor::lifecycle::EventCallback>> {
        self.event_callback.as_ref()
    }

    /// Get reference to capabilities (internal use for security checks).
    ///
    /// Used by Actor trait message handler for capability-based security enforcement.
    #[doc(hidden)]
    pub fn capabilities(&self) -> &CapabilitySet {
        &self.capabilities
    }

    /// Get reference to rate limiter (internal use for security checks).
    ///
    /// Used by Actor trait message handler for rate limiting enforcement.
    #[doc(hidden)]
    pub fn rate_limiter(&self) -> &crate::core::rate_limiter::MessageRateLimiter {
        &self.rate_limiter
    }

    /// Get reference to security config (internal use for security checks).
    ///
    /// Used by Actor trait message handler for security policy enforcement.
    #[doc(hidden)]
    pub fn security_config(&self) -> &crate::core::config::SecurityConfig {
        &self.security_config
    }

    /// Set custom security configuration.
    ///
    /// This method is primarily intended for testing scenarios where
    /// custom security policies need to be enforced.
    ///
    /// **Note**: This is a testing utility. In production, security config
    /// should be set via the constructor or system configuration.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut actor = ComponentActor::new(...);
    /// actor.set_security_config(SecurityConfig {
    ///     mode: SecurityMode::Strict,
    ///     audit_logging: true,
    ///     max_message_size: 512 * 1024,
    ///     capability_check_timeout_us: 5,
    /// });
    /// ```
    #[doc(hidden)]
    pub fn set_security_config(&mut self, config: crate::core::config::SecurityConfig) {
        self.security_config = config;
    }

    /// Set custom rate limiter.
    ///
    /// This method is primarily intended for testing scenarios where
    /// custom rate limits need to be enforced.
    ///
    /// **Note**: This is a testing utility. In production, rate limiter
    /// should be configured via system configuration.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut actor = ComponentActor::new(...);
    /// actor.set_rate_limiter(MessageRateLimiter::new(RateLimiterConfig {
    ///     messages_per_second: 10,
    ///     window_duration: Duration::from_secs(1),
    /// }));
    /// ```
    #[doc(hidden)]
    pub fn set_rate_limiter(&mut self, limiter: crate::core::rate_limiter::MessageRateLimiter) {
        self.rate_limiter = limiter;
    }

    /// Perform security checks on an incoming message.
    ///
    /// This method extracts the security check logic from handle_message
    /// to allow independent testing of security enforcement.
    ///
    /// Checks performed:
    /// 1. Sender authorization (capability check)
    /// 2. Payload size validation
    /// 3. Rate limiting
    ///
    /// **Note**: This is primarily a testing utility. In production, security
    /// checks are automatically performed by the Actor trait implementation.
    ///
    /// # Arguments
    ///
    /// * `msg` - Message to check
    ///
    /// # Returns
    ///
    /// Ok(()) if all security checks pass, Err otherwise.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let actor = ComponentActor::new(...);
    /// let msg = ComponentMessage::InterComponent {
    ///     sender: ComponentId::new("sender"),
    ///     payload: b"test".to_vec(),
    /// };
    ///
    /// match actor.check_message_security(&msg) {
    ///     Ok(()) => println!("Security checks passed"),
    ///     Err(e) => println!("Security check failed: {}", e),
    /// }
    /// ```
    #[doc(hidden)]
    pub fn check_message_security(
        &self,
        msg: &ComponentMessage,
    ) -> Result<(), crate::core::WasmError> {
        use crate::core::WasmError;

        match msg {
            ComponentMessage::InterComponent { sender, payload }
            | ComponentMessage::InterComponentWithCorrelation {
                sender, payload, ..
            } => {
                let component_id_str = self.component_id().as_str();
                let sender_str = sender.as_str();

                // 2.1. Sender Authorization Check
                if !self.capabilities().allows_receiving_from(sender) {
                    let error_msg = format!(
                        "Component {} not authorized to send to {} (no Messaging capability)",
                        sender_str, component_id_str
                    );
                    return Err(WasmError::capability_denied(
                        crate::core::capability::Capability::Messaging(
                            crate::core::capability::TopicPattern::new("*"),
                        ),
                        error_msg,
                    ));
                }

                // 2.2. Payload Size Validation
                let max_size = self.security_config().max_message_size;
                if payload.len() > max_size {
                    return Err(WasmError::payload_too_large(payload.len(), max_size));
                }

                // 2.3. Rate Limiting Check
                if !self.rate_limiter().check_rate_limit(sender) {
                    return Err(WasmError::rate_limit_exceeded(
                        sender_str.to_string(),
                        crate::core::rate_limiter::DEFAULT_RATE_LIMIT,
                    ));
                }

                Ok(())
            }
            _ => {
                // Other message types don't have security checks
                Ok(())
            }
        }
    }
}

// Actor and Child trait implementations will be in separate files
// to maintain clean separation of concerns (§4.3)

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "unwrap acceptable in test code")]
mod tests {
    use super::*;

    fn create_test_metadata() -> ComponentMetadata {
        ComponentMetadata {
            name: "test-component".to_string(),
            version: "1.0.0".to_string(),
            author: "Test Author".to_string(),
            description: None,
            required_capabilities: vec![],
            resource_limits: crate::core::ResourceLimits {
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

    #[test]
    fn test_component_actor_creation() {
        let component_id = ComponentId::new("test-component");
        let metadata = create_test_metadata();
        let caps = CapabilitySet::new();

        let actor = ComponentActor::new(component_id.clone(), metadata, caps, ());

        assert_eq!(actor.component_id(), &component_id);
        assert!(!actor.is_wasm_loaded());
        assert_eq!(*actor.state(), ActorState::Creating);
        assert_eq!(actor.uptime(), None);
    }

    #[test]
    fn test_component_actor_default_state() {
        let actor = create_test_actor();

        assert_eq!(*actor.state(), ActorState::Creating);
        assert!(!actor.is_wasm_loaded());
        assert!(actor.mailbox_rx.is_none());
        assert!(actor.started_at.is_none());
    }

    #[test]
    fn test_component_id_getter() {
        let component_id = ComponentId::new("getter-test");
        let actor = ComponentActor::new(
            component_id.clone(),
            create_test_metadata(),
            CapabilitySet::new(),
            (),
        );

        assert_eq!(actor.component_id(), &component_id);
    }

    #[test]
    fn test_state_getter() {
        let actor = create_test_actor();
        assert_eq!(*actor.state(), ActorState::Creating);
    }

    #[test]
    fn test_is_wasm_loaded_initial() {
        let actor = create_test_actor();
        assert!(!actor.is_wasm_loaded());
    }

    #[test]
    fn test_uptime_before_start() {
        let actor = create_test_actor();
        assert_eq!(actor.uptime(), None);
    }

    #[test]
    fn test_actor_state_equality() {
        assert_eq!(ActorState::Creating, ActorState::Creating);
        assert_eq!(ActorState::Ready, ActorState::Ready);
        assert_eq!(
            ActorState::Failed("test".to_string()),
            ActorState::Failed("test".to_string())
        );

        assert_ne!(ActorState::Creating, ActorState::Ready);
        assert_ne!(
            ActorState::Failed("a".to_string()),
            ActorState::Failed("b".to_string())
        );
    }

    #[test]
    fn test_actor_state_clone() {
        let state = ActorState::Failed("error".to_string());
        let cloned = state.clone();
        assert_eq!(state, cloned);
    }

    #[test]
    fn test_component_message_invoke() {
        let msg = ComponentMessage::Invoke {
            function: "test_function".to_string(),
            args: vec![1, 2, 3],
        };

        match msg {
            ComponentMessage::Invoke { function, args } => {
                assert_eq!(function, "test_function");
                assert_eq!(args, vec![1, 2, 3]);
            }
            _ => unreachable!("Expected Invoke variant"),
        }
    }

    #[test]
    fn test_component_message_invoke_result() {
        let msg = ComponentMessage::InvokeResult {
            result: vec![4, 5, 6],
            error: Some("test error".to_string()),
        };

        match msg {
            ComponentMessage::InvokeResult { result, error } => {
                assert_eq!(result, vec![4, 5, 6]);
                assert_eq!(error, Some("test error".to_string()));
            }
            _ => unreachable!("Expected InvokeResult variant"),
        }
    }

    #[test]
    fn test_component_message_inter_component() {
        let sender_id = ComponentId::new("sender");
        let msg = ComponentMessage::InterComponent {
            sender: sender_id.clone(),
            payload: vec![7, 8, 9],
        };

        match msg {
            ComponentMessage::InterComponent { sender, payload } => {
                assert_eq!(sender, sender_id);
                assert_eq!(payload, vec![7, 8, 9]);
            }
            _ => unreachable!("Expected InterComponent variant"),
        }
    }

    #[test]
    fn test_component_message_shutdown() {
        let msg = ComponentMessage::Shutdown;
        assert!(matches!(msg, ComponentMessage::Shutdown));
    }

    #[test]
    fn test_component_message_health_check() {
        let msg = ComponentMessage::HealthCheck;
        assert!(matches!(msg, ComponentMessage::HealthCheck));
    }

    #[test]
    fn test_component_message_clone() {
        let msg = ComponentMessage::HealthCheck;
        let cloned = msg.clone();
        assert!(matches!(cloned, ComponentMessage::HealthCheck));
    }

    #[test]
    fn test_health_status_healthy() {
        let health = HealthStatus::Healthy;
        assert_eq!(health, HealthStatus::Healthy);
    }

    #[test]
    fn test_health_status_degraded() {
        let health = HealthStatus::Degraded {
            reason: "High memory usage".to_string(),
        };

        match health {
            HealthStatus::Degraded { reason } => {
                assert_eq!(reason, "High memory usage");
            }
            _ => unreachable!("Expected Degraded variant"),
        }
    }

    #[test]
    fn test_health_status_unhealthy() {
        let health = HealthStatus::Unhealthy {
            reason: "Connection lost".to_string(),
        };

        match health {
            HealthStatus::Unhealthy { reason } => {
                assert_eq!(reason, "Connection lost");
            }
            _ => unreachable!("Expected Unhealthy variant"),
        }
    }

    #[test]
    fn test_health_status_equality() {
        assert_eq!(HealthStatus::Healthy, HealthStatus::Healthy);
        assert_eq!(
            HealthStatus::Degraded {
                reason: "test".to_string()
            },
            HealthStatus::Degraded {
                reason: "test".to_string()
            }
        );

        assert_ne!(
            HealthStatus::Healthy,
            HealthStatus::Unhealthy {
                reason: "failed".to_string()
            }
        );
    }

    #[test]
    fn test_health_status_clone() {
        let health = HealthStatus::Healthy;
        let cloned = health.clone();
        assert_eq!(health, cloned);
    }

    #[test]
    fn test_health_status_serde() {
        let health = HealthStatus::Degraded {
            reason: "test".to_string(),
        };

        let json = serde_json::to_string(&health);
        assert!(json.is_ok(), "Failed to serialize HealthStatus: {json:?}");

        if let Ok(json) = json {
            let deserialized: Result<HealthStatus, _> = serde_json::from_str(&json);
            assert!(
                deserialized.is_ok(),
                "Failed to deserialize HealthStatus: {deserialized:?}"
            );

            if let Ok(deserialized) = deserialized {
                assert_eq!(health, deserialized);
            }
        }
    }

    // ========================================================================
    // Custom State Management Tests (Phase 5 Task 5.2)
    // ========================================================================

    #[derive(Clone, Debug, PartialEq)]
    struct TestState {
        count: u64,
        name: String,
    }

    #[tokio::test]
    async fn test_with_state_read_access() {
        let state = TestState {
            count: 42,
            name: "test".to_string(),
        };
        let actor: ComponentActor<TestState> = ComponentActor::new(
            ComponentId::new("test"),
            create_test_metadata(),
            CapabilitySet::new(),
            state.clone(),
        );

        // Read state using with_state
        let count = actor.with_state(|s| s.count).await;
        assert_eq!(count, 42);

        let name = actor.with_state(|s| s.name.clone()).await;
        assert_eq!(name, "test");
    }

    #[tokio::test]
    async fn test_with_state_mut_write_access() {
        let state = TestState {
            count: 0,
            name: "initial".to_string(),
        };
        let actor: ComponentActor<TestState> = ComponentActor::new(
            ComponentId::new("test"),
            create_test_metadata(),
            CapabilitySet::new(),
            state,
        );

        // Modify state using with_state_mut
        actor
            .with_state_mut(|s| {
                s.count += 1;
                s.name = "modified".to_string();
            })
            .await;

        // Verify modification
        let count = actor.with_state(|s| s.count).await;
        assert_eq!(count, 1);

        let name = actor.with_state(|s| s.name.clone()).await;
        assert_eq!(name, "modified");
    }

    #[tokio::test]
    async fn test_get_state_clone() {
        let state = TestState {
            count: 100,
            name: "original".to_string(),
        };
        let actor: ComponentActor<TestState> = ComponentActor::new(
            ComponentId::new("test"),
            create_test_metadata(),
            CapabilitySet::new(),
            state.clone(),
        );

        // Get cloned state
        let cloned = actor.get_state().await;
        assert_eq!(cloned, state);

        // Modify original via with_state_mut
        actor.with_state_mut(|s| s.count = 200).await;

        // Cloned value unchanged (it was a snapshot)
        assert_eq!(cloned.count, 100);
    }

    #[tokio::test]
    async fn test_set_custom_state_replacement() {
        let initial_state = TestState {
            count: 1,
            name: "first".to_string(),
        };
        let actor: ComponentActor<TestState> = ComponentActor::new(
            ComponentId::new("test"),
            create_test_metadata(),
            CapabilitySet::new(),
            initial_state.clone(),
        );

        // Replace state
        let new_state = TestState {
            count: 2,
            name: "second".to_string(),
        };
        let old_state = actor.set_custom_state(new_state.clone()).await;

        // Old state returned
        assert_eq!(old_state, initial_state);

        // New state active
        let current = actor.get_state().await;
        assert_eq!(current, new_state);
    }

    #[tokio::test]
    async fn test_state_arc_shared_ownership() {
        let state = TestState {
            count: 0,
            name: "shared".to_string(),
        };
        let actor: ComponentActor<TestState> = ComponentActor::new(
            ComponentId::new("test"),
            create_test_metadata(),
            CapabilitySet::new(),
            state,
        );

        // Get Arc reference
        let state_ref = actor.state_arc();

        // Modify via actor
        actor.with_state_mut(|s| s.count = 5).await;

        // Access via Arc reference
        let count = {
            let guard = state_ref.read().await;
            guard.count
        };
        assert_eq!(count, 5);
    }

    #[tokio::test]
    async fn test_concurrent_state_readers() {
        let state = TestState {
            count: 999,
            name: "concurrent".to_string(),
        };
        let actor = Arc::new(ComponentActor::new(
            ComponentId::new("test"),
            create_test_metadata(),
            CapabilitySet::new(),
            state,
        ));

        // Spawn multiple concurrent readers
        let mut handles = vec![];
        for _ in 0..10 {
            let actor_clone = Arc::clone(&actor);
            let handle = tokio::spawn(async move { actor_clone.with_state(|s| s.count).await });
            handles.push(handle);
        }

        // All readers should succeed
        for handle in handles {
            let count = handle.await.unwrap();
            assert_eq!(count, 999);
        }
    }

    #[tokio::test]
    async fn test_state_persistence_across_operations() {
        let state = TestState {
            count: 0,
            name: "counter".to_string(),
        };
        let actor: ComponentActor<TestState> = ComponentActor::new(
            ComponentId::new("test"),
            create_test_metadata(),
            CapabilitySet::new(),
            state,
        );

        // Increment count multiple times
        for i in 1..=10 {
            actor.with_state_mut(|s| s.count += 1).await;
            let count = actor.with_state(|s| s.count).await;
            assert_eq!(count, i);
        }

        // Final count should be 10
        let final_count = actor.with_state(|s| s.count).await;
        assert_eq!(final_count, 10);
    }

    #[tokio::test]
    async fn test_unit_type_state_no_overhead() {
        // Default case: no custom state (unit type ())
        let actor: ComponentActor<()> = ComponentActor::new(
            ComponentId::new("test"),
            create_test_metadata(),
            CapabilitySet::new(),
            (),
        );

        // with_state should work with unit type
        actor.with_state(|_| ()).await;

        // with_state_mut should work with unit type
        actor.with_state_mut(|_| ()).await;

        // State operations are no-op but don't panic
        let state_ref = actor.state_arc();
        assert!(Arc::strong_count(&state_ref) >= 1);
    }

    #[tokio::test]
    async fn test_state_with_complex_type() {
        use std::collections::HashMap;

        #[derive(Clone, Debug)]
        struct ComplexState {
            map: HashMap<String, u64>,
            vec: Vec<String>,
        }

        let mut map = HashMap::new();
        map.insert("key1".to_string(), 100);
        map.insert("key2".to_string(), 200);

        let state = ComplexState {
            map,
            vec: vec!["a".to_string(), "b".to_string()],
        };

        let actor: ComponentActor<ComplexState> = ComponentActor::new(
            ComponentId::new("test"),
            create_test_metadata(),
            CapabilitySet::new(),
            state,
        );

        // Modify complex state
        actor
            .with_state_mut(|s| {
                s.map.insert("key3".to_string(), 300);
                s.vec.push("c".to_string());
            })
            .await;

        // Verify modifications
        let map_len = actor.with_state(|s| s.map.len()).await;
        assert_eq!(map_len, 3);

        let vec_len = actor.with_state(|s| s.vec.len()).await;
        assert_eq!(vec_len, 3);
    }

    #[tokio::test]
    async fn test_state_methods_return_values() {
        let state = TestState {
            count: 50,
            name: "test".to_string(),
        };
        let actor: ComponentActor<TestState> = ComponentActor::new(
            ComponentId::new("test"),
            create_test_metadata(),
            CapabilitySet::new(),
            state,
        );

        // with_state returns computed value
        let doubled = actor.with_state(|s| s.count * 2).await;
        assert_eq!(doubled, 100);

        // with_state_mut returns computed value
        let incremented = actor
            .with_state_mut(|s| {
                s.count += 1;
                s.count
            })
            .await;
        assert_eq!(incremented, 51);
    }
}
