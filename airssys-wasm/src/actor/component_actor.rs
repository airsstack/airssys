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
use wasmtime::{Engine, Instance, Store};

// Layer 3: Internal module imports
use crate::core::{
    CapabilitySet, ComponentId, ComponentMetadata, WasmError,
};

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
    pub fn extract(instance: &Instance, store: &mut Store<ComponentResourceLimiter>) -> Result<Self, WasmError> {
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
            func
                .call_async(store, &[], &mut [])
                .await
                .map_err(|e| WasmError::execution_failed(
                    format!("Component _start function failed: {e}")
                ))?;
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
            match tokio::time::timeout(
                timeout,
                func.call_async(store, &[], &mut [])
            ).await {
                Ok(Ok(())) => Ok(()),
                Ok(Err(e)) => Err(WasmError::execution_failed(
                    format!("Component _cleanup function failed: {e}")
                )),
                Err(_) => Err(WasmError::execution_timeout(
                    timeout.as_millis() as u64,
                    None
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
/// - **Option<WasmRuntime>**: Allows safe handling of unloaded state (WASM loads in Child::start())
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
/// # Example
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
/// let actor = ComponentActor::new(component_id.clone(), metadata, caps);
///
/// assert_eq!(actor.component_id(), &component_id);
/// assert_eq!(*actor.state(), ActorState::Creating);
/// assert!(!actor.is_wasm_loaded());
/// ```
#[allow(dead_code)] // Fields will be used in future tasks
pub struct ComponentActor {
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

impl ComponentActor {
    /// Create a new ComponentActor instance.
    ///
    /// Creates a ComponentActor in the `Creating` state. WASM runtime is not
    /// loaded until `Child::start()` is called.
    ///
    /// # Arguments
    ///
    /// * `component_id` - Unique identifier for this component
    /// * `metadata` - Component metadata
    /// * `capabilities` - Security capabilities and permissions
    ///
    /// # Returns
    ///
    /// A new ComponentActor instance in Creating state.
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
    /// let actor = ComponentActor::new(component_id.clone(), metadata, caps);
    ///
    /// assert_eq!(actor.component_id(), &component_id);
    /// assert!(!actor.is_wasm_loaded());
    /// ```
    pub fn new(
        component_id: ComponentId,
        metadata: ComponentMetadata,
        capabilities: CapabilitySet,
    ) -> Self {
        Self {
            component_id,
            wasm_runtime: None,
            capabilities,
            state: ActorState::Creating,
            metadata,
            mailbox_rx: None,
            created_at: Utc::now(),
            started_at: None,
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
        self.started_at
            .map(|started| Utc::now() - started)
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
}

// Actor and Child trait implementations will be in separate files
// to maintain clean separation of concerns (§4.3)

#[cfg(test)]
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
        )
    }

    #[test]
    fn test_component_actor_creation() {
        let component_id = ComponentId::new("test-component");
        let metadata = create_test_metadata();
        let caps = CapabilitySet::new();

        let actor = ComponentActor::new(component_id.clone(), metadata, caps);

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
            assert!(deserialized.is_ok(), "Failed to deserialize HealthStatus: {deserialized:?}");
            
            if let Ok(deserialized) = deserialized {
                assert_eq!(health, deserialized);
            }
        }
    }
}
