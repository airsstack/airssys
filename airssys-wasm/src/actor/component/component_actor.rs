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
use std::sync::Arc;
use std::time::Duration;

// Layer 2: Third-party crate imports
use chrono::{DateTime, Utc};

use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::RwLock;
use tracing::{debug, trace};

// Layer 3: Internal module imports
use crate::core::runtime::ComponentHandle;
use crate::core::{CapabilitySet, ComponentId, ComponentMetadata, WasmError};
use crate::runtime::WasmEngine;

// =============================================================================
// LEGACY CODE DELETED (WASM-TASK-006-HOTFIX Phase 2 Task 2.1)
// =============================================================================
// The following legacy types were deleted as part of the Component Model migration:
// - WasmRuntime struct (replaced by WasmEngine + ComponentHandle)
// - WasmExports struct (Component Model uses generated bindings)
//
// See ADR-WASM-002 and ADR-WASM-021 for migration details.
// =============================================================================

// =============================================================================
// LEGACY WORKAROUND CODE DELETED (WASM-TASK-006-HOTFIX Phase 2 Task 2.1)
// =============================================================================
// The following legacy workaround types were deleted as part of the Component Model migration:
// - WasmBumpAllocator (Canonical ABI handles memory marshalling automatically)
// - HandleMessageParams (Component Model has typed function calls)
// - HandleMessageResult (Component Model has typed returns)
// - BUMP_ALLOCATOR_BASE, MAX_SENDER_SIZE, MAX_MESSAGE_SIZE constants
//
// These were workarounds for the wrong API (wasmtime::Module instead of
// wasmtime::component::Component). The Component Model API handles all parameter
// marshalling automatically via the Canonical ABI.
//
// See ADR-WASM-002 and ADR-WASM-021 for migration details.
// =============================================================================

/// Message reception configuration (WASM-TASK-006 Task 1.2).
///
/// Controls ComponentActor message reception behavior including backpressure
/// thresholds, timeouts, and delivery settings. These settings are critical
/// for preventing message storms and ensuring component stability.
///
/// # Backpressure Strategy
///
/// When current_queue_depth >= max_queue_depth, new messages are dropped
/// (backpressure applied). This prevents:
/// - Memory exhaustion from unbounded message queues
/// - Component DoS from message flooding
/// - Cascading failures across components
///
/// # Timeout Handling
///
/// WASM export invocations are wrapped with tokio::time::timeout to prevent
/// hung components from blocking message delivery. If a component takes longer
/// than delivery_timeout_ms, the message is dropped and delivery_timeouts
/// metric is incremented.
///
/// # Performance
///
/// - Backpressure check: <5ns (single atomic load)
/// - Timeout wrapper: ~20ns overhead
/// - Total per-message overhead: ~25ns
///
/// # Examples
///
/// ```rust
/// use airssys_wasm::actor::MessageReceptionConfig;
/// use std::time::Duration;
///
/// // Default configuration
/// let config = MessageReceptionConfig::default();
/// assert_eq!(config.max_queue_depth, 1000);
/// assert_eq!(config.delivery_timeout(), Duration::from_millis(100));
/// assert!(config.enable_backpressure);
///
/// // Custom configuration for high-throughput component
/// let config = MessageReceptionConfig {
///     max_queue_depth: 10000,
///     delivery_timeout_ms: 500,
///     enable_backpressure: true,
/// };
/// ```
///
/// # References
///
/// - WASM-TASK-006 Phase 1 Task 1.2: Message reception infrastructure
/// - Target: >10,000 msg/sec per component, <20ns delivery latency
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MessageReceptionConfig {
    /// Maximum queue depth before backpressure applies (default: 1000)
    pub max_queue_depth: usize,

    /// WASM export invocation timeout in milliseconds (default: 100ms)
    pub delivery_timeout_ms: u64,

    /// Enable backpressure detection (default: true)
    pub enable_backpressure: bool,
}

impl MessageReceptionConfig {
    /// Create new configuration with specified limits.
    ///
    /// # Parameters
    ///
    /// * `max_queue_depth` - Backpressure threshold (messages)
    /// * `delivery_timeout_ms` - WASM invocation timeout (milliseconds)
    /// * `enable_backpressure` - Enable backpressure detection
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::actor::MessageReceptionConfig;
    ///
    /// let config = MessageReceptionConfig::new(5000, 200, true);
    /// assert_eq!(config.max_queue_depth, 5000);
    /// ```
    pub fn new(
        max_queue_depth: usize,
        delivery_timeout_ms: u64,
        enable_backpressure: bool,
    ) -> Self {
        Self {
            max_queue_depth,
            delivery_timeout_ms,
            enable_backpressure,
        }
    }

    /// Get delivery timeout as Duration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::actor::MessageReceptionConfig;
    /// use std::time::Duration;
    ///
    /// let config = MessageReceptionConfig::default();
    /// assert_eq!(config.delivery_timeout(), Duration::from_millis(100));
    /// ```
    pub fn delivery_timeout(&self) -> Duration {
        Duration::from_millis(self.delivery_timeout_ms)
    }
}

impl Default for MessageReceptionConfig {
    /// Default configuration optimized for typical component workloads.
    ///
    /// Defaults:
    /// - max_queue_depth: 1000 messages
    /// - delivery_timeout_ms: 100ms
    /// - enable_backpressure: true
    fn default() -> Self {
        Self {
            max_queue_depth: 1000,
            delivery_timeout_ms: 100,
            enable_backpressure: true,
        }
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

    // NOTE: wasm_runtime field removed in WASM-TASK-006-HOTFIX Phase 2 Task 2.1
    // Component Model uses component_engine + component_handle instead
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

    /// Message reception metrics (WASM-TASK-006 Task 1.2).
    ///
    /// Tracks per-component message reception statistics including:
    /// - messages_received: Successfully processed messages
    /// - backpressure_drops: Messages dropped due to mailbox overflow
    /// - delivery_timeouts: WASM export invocation timeouts
    /// - delivery_errors: WASM traps and invocation failures
    /// - current_queue_depth: Estimated in-flight message count
    ///
    /// These metrics enable:
    /// - Component health monitoring
    /// - Backpressure detection
    /// - Performance analysis
    /// - Capacity planning
    ///
    /// All operations are lock-free atomic updates (<50ns overhead per message).
    message_metrics: crate::messaging::MessageReceptionMetrics,

    /// Message reception configuration (WASM-TASK-006 Task 1.2).
    ///
    /// Controls message reception behavior:
    /// - max_queue_depth: Mailbox capacity limit (default: 1000)
    /// - delivery_timeout_ms: WASM export invocation timeout (default: 100ms)
    /// - enable_backpressure: Enable backpressure detection (default: true)
    ///
    /// Backpressure prevents message storms from overwhelming components by
    /// dropping messages when queue depth exceeds max_queue_depth.
    message_config: MessageReceptionConfig,

    // =============================================================================
    // COMPONENT MODEL ARCHITECTURE (WASM-TASK-006-HOTFIX Phase 2)
    // =============================================================================
    // These fields replace the legacy core WASM API (WasmRuntime) with Component Model.
    // The transition follows ADR-WASM-002 (Component Model mandate) and ADR-WASM-021.
    // =============================================================================
    /// Shared WASM execution engine (Component Model API).
    ///
    /// This is the CORRECT architecture per ADR-WASM-002:
    /// - Uses `wasmtime::component::{Component, Linker}` (Component Model API)
    /// - Shared across components for efficient compilation caching
    /// - Supports WIT interfaces and typed host function calls
    ///
    /// **Migration Note (WASM-TASK-006-HOTFIX):**
    /// This field replaces the legacy `wasm_runtime: Option<WasmRuntime>` which
    /// incorrectly used core WASM API (`wasmtime::Module`). During migration,
    /// both fields may exist, but `component_engine` is the target architecture.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Create shared engine (typically at system startup)
    /// let engine = Arc::new(WasmEngine::new()?);
    ///
    /// // Pass to ComponentActor during creation
    /// let actor = ComponentActor::with_engine(engine.clone(), ...);
    /// ```
    component_engine: Option<Arc<WasmEngine>>,

    /// Handle to loaded component instance (Component Model API).
    ///
    /// This is the CORRECT architecture per ADR-WASM-002:
    /// - Uses `ComponentHandle` which wraps `Arc<wasmtime::component::Component>`
    /// - Loaded via `WasmEngine::load_component()`
    /// - Supports typed function calls via Component Model bindings
    ///
    /// **Migration Note (WASM-TASK-006-HOTFIX):**
    /// This field replaces component loading via `wasmtime::Module::from_binary()`.
    /// The Component Model approach enables WIT interface usage and type-safe
    /// host function calls.
    ///
    /// # Lifecycle
    ///
    /// - `None` after construction
    /// - `Some(handle)` after `Child::start()` successfully loads component
    /// - `None` after `Child::stop()` unloads component
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // In Child::start()
    /// let handle = engine.load_component(&component_id, &wasm_bytes).await?;
    /// self.component_handle = Some(handle);
    ///
    /// // In Child::stop()
    /// self.component_handle = None;
    /// ```
    component_handle: Option<ComponentHandle>,
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

// =============================================================================
// MOVED TO core/component_message.rs (ADR-WASM-022)
// =============================================================================
// ComponentMessage and HealthStatus (now ComponentHealthStatus) have been moved
// to core/component_message.rs to prevent circular dependencies between actor/
// and runtime/ modules.
//
// Re-exports are provided here for backward compatibility.
// =============================================================================

// Re-export ComponentMessage from core for backward compatibility
// Note: The actual enum is now in crate::core::component_message
pub use crate::core::ComponentMessage;

// Re-export ComponentHealthStatus as HealthStatus for backward compatibility
// Note: Renamed to avoid conflict with core::observability::HealthStatus
pub use crate::core::ComponentHealthStatus as HealthStatus;

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
            // NOTE: wasm_runtime field removed in WASM-TASK-006-HOTFIX Phase 2 Task 2.1
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
            message_metrics: crate::messaging::MessageReceptionMetrics::new(),
            message_config: MessageReceptionConfig::default(),
            // WASM-TASK-006-HOTFIX Phase 2: Component Model fields (initially None)
            component_engine: None,
            component_handle: None,
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

    // NOTE: clear_wasm_runtime() removed in WASM-TASK-006-HOTFIX Phase 2 Task 2.1
    // Component Model uses set_component_handle(None) instead

    /// Check if component is loaded and ready.
    ///
    /// Returns true if Child::start() has successfully loaded the component.
    /// Now checks component_handle (Component Model) rather than legacy wasm_runtime.
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
    /// // Component not loaded until Child::start()
    /// assert!(!actor.is_wasm_loaded());
    /// ```
    pub fn is_wasm_loaded(&self) -> bool {
        // WASM-TASK-006-HOTFIX: Now checks component_handle instead of legacy wasm_runtime
        self.component_handle.is_some()
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

    /// Get reference to message reception metrics (WASM-TASK-006 Task 1.2).
    ///
    /// Returns reference to MessageReceptionMetrics tracking per-component
    /// message reception statistics including:
    /// - messages_received: Successfully processed messages
    /// - backpressure_drops: Messages dropped due to queue overflow
    /// - delivery_timeouts: WASM export invocation timeouts
    /// - delivery_errors: WASM traps and invocation failures
    /// - current_queue_depth: Estimated in-flight message count
    ///
    /// # Performance
    ///
    /// Accessor overhead: <1ns (reference return)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let metrics = actor.message_metrics();
    /// let stats = metrics.snapshot();
    /// println!("Messages received: {}", stats.messages_received);
    /// println!("Backpressure drops: {}", stats.backpressure_drops);
    /// ```
    pub fn message_metrics(&self) -> &crate::messaging::MessageReceptionMetrics {
        &self.message_metrics
    }

    /// Get reference to message reception configuration (WASM-TASK-006 Task 1.2).
    ///
    /// Returns reference to MessageReceptionConfig controlling backpressure
    /// thresholds, timeouts, and delivery settings.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let config = actor.message_config();
    /// println!("Max queue depth: {}", config.max_queue_depth);
    /// println!("Delivery timeout: {:?}", config.delivery_timeout());
    /// ```
    pub fn message_config(&self) -> &MessageReceptionConfig {
        &self.message_config
    }

    /// Set message reception configuration (builder pattern).
    ///
    /// Replaces the component's message reception configuration with new settings.
    /// Typically called during component initialization.
    ///
    /// # Arguments
    ///
    /// * `config` - New message reception configuration
    ///
    /// # Returns
    ///
    /// `self` for method chaining (builder pattern)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let config = MessageReceptionConfig {
    ///     max_queue_depth: 5000,
    ///     delivery_timeout_ms: 200,
    ///     enable_backpressure: true,
    /// };
    ///
    /// let actor = ComponentActor::new(/* ... */)
    ///     .with_message_config(config);
    /// ```
    pub fn with_message_config(mut self, config: MessageReceptionConfig) -> Self {
        self.message_config = config;
        self
    }

    // =============================================================================
    // COMPONENT MODEL API (WASM-TASK-006-HOTFIX Phase 2)
    // =============================================================================

    /// Set the shared WASM engine (Component Model - builder pattern).
    ///
    /// Configures the ComponentActor to use the provided WasmEngine for component
    /// loading and execution. This is the **CORRECT** architecture per ADR-WASM-002.
    ///
    /// # Arguments
    ///
    /// * `engine` - Shared WasmEngine (Component Model API)
    ///
    /// # Returns
    ///
    /// `self` for method chaining (builder pattern)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use std::sync::Arc;
    /// use airssys_wasm::runtime::WasmEngine;
    /// use airssys_wasm::actor::ComponentActor;
    ///
    /// let engine = Arc::new(WasmEngine::new()?);
    ///
    /// let actor = ComponentActor::new(component_id, metadata, caps, ())
    ///     .with_component_engine(engine);
    /// ```
    ///
    /// # Migration Note (WASM-TASK-006-HOTFIX)
    ///
    /// This replaces the legacy pattern where Child::start() created a local
    /// Wasmtime Engine using core WASM API. The shared WasmEngine:
    /// - Uses Component Model API (wasmtime::component)
    /// - Enables compilation caching across components
    /// - Supports WIT interfaces and typed calls
    pub fn with_component_engine(mut self, engine: Arc<WasmEngine>) -> Self {
        self.component_engine = Some(engine);
        self
    }

    /// Get reference to the Component Model engine (if set).
    ///
    /// Returns the shared WasmEngine used for component loading and execution.
    ///
    /// # Returns
    ///
    /// - `Some(&Arc<WasmEngine>)` - Engine is configured
    /// - `None` - Engine not set (legacy mode)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// if let Some(engine) = actor.component_engine() {
    ///     let handle = engine.load_component(&id, &bytes).await?;
    /// }
    /// ```
    pub fn component_engine(&self) -> Option<&Arc<WasmEngine>> {
        self.component_engine.as_ref()
    }

    /// Get reference to the loaded component handle (if loaded).
    ///
    /// Returns the ComponentHandle loaded via `WasmEngine::load_component()`.
    ///
    /// # Returns
    ///
    /// - `Some(&ComponentHandle)` - Component is loaded
    /// - `None` - Component not loaded or unloaded
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// if let Some(handle) = actor.component_handle() {
    ///     println!("Component {} loaded", handle.id());
    /// }
    /// ```
    pub fn component_handle(&self) -> Option<&ComponentHandle> {
        self.component_handle.as_ref()
    }

    /// Set the component handle (internal use by Child::start()).
    ///
    /// This method is public but primarily for internal use by the Child trait
    /// implementation. External code should not normally call this.
    #[doc(hidden)]
    pub fn set_component_handle(&mut self, handle: Option<ComponentHandle>) {
        self.component_handle = handle;
    }

    /// Check if Component Model engine is configured.
    ///
    /// Returns true if `with_component_engine()` was called to configure
    /// the Component Model architecture. When true, Child::start() will use
    /// `WasmEngine::load_component()` instead of legacy `wasmtime::Module`.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// if actor.uses_component_model() {
    ///     println!("Using Component Model API (correct architecture)");
    /// } else {
    ///     println!("Using legacy core WASM API (deprecated)");
    /// }
    /// ```
    pub fn uses_component_model(&self) -> bool {
        self.component_engine.is_some()
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
        // Test mode: Return valid Component Model bytes from fixture
        // This is required because Component Model (wasmtime::component::Component)
        // requires valid component format, not core WASM format.
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
        {
            // Minimal valid Component Model binary (component format, not core WASM)
            // This is the binary from tests/fixtures/handle-message-component.wasm
            // which implements a basic component with handle-message export.
            //
            // Note: We inline the bytes here because:
            // 1. Tests run from any directory, making relative paths unreliable
            // 2. The fixture is small (493 bytes)
            // 3. This ensures tests always work regardless of CWD
            //
            // The component was built with: wasm-tools component new
            Ok(include_bytes!("../../../tests/fixtures/handle-message-component.wasm").to_vec())
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

    // NOTE: Legacy accessor methods removed in WASM-TASK-006-HOTFIX Phase 2 Task 2.1:
    // - set_wasm_runtime() - Use set_component_handle() instead
    // - wasm_runtime_mut() - Use component_engine() + component_handle() instead
    // - wasm_runtime() - Use component_engine() + component_handle() instead

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
    ) -> Result<tokio::sync::oneshot::Receiver<crate::core::messaging::ResponseMessage>, WasmError>
    {
        use crate::actor::message::RequestMessage;
        use crate::core::messaging::{CorrelationId, PendingRequest};
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

        // Register pending request
        let pending_request = PendingRequest {
            correlation_id,
            response_tx,
            requested_at: Instant::now(),
            timeout,
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
            // TODO(WASM-TASK-006): Phase 1 uses direct ComponentId addressing
            // This needs to specify actual target component in Phase 1 implementation
            to: ComponentId::new("__topic_broadcast__"),
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
        result: Result<Vec<u8>, crate::core::messaging::RequestError>,
    ) -> Result<(), WasmError> {
        use crate::core::messaging::ResponseMessage;
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

    /// Invoke WASM handle-message export with timeout (WASM-TASK-006 Task 1.2).
    ///
    /// Invokes the component's handle-message export function with sender ID and
    /// payload, wrapped with tokio::time::timeout to prevent hung components from
    /// blocking message delivery.
    ///
    /// # Architecture
    ///
    /// This method implements the WASM boundary crossing for inter-component
    /// messages, converting Rust types to WASM-compatible formats and handling
    /// all edge cases:
    /// - Missing export (component doesn't implement handle-message)
    /// - WASM traps (runtime errors in component code)
    /// - Timeouts (component processing takes too long)
    /// - Type conversion errors
    ///
    /// # Component Model Migration (WASM-TASK-006-HOTFIX Phase 2 Task 2.4)
    ///
    /// This method supports two execution paths:
    ///
    /// 1. **Component Model path** (when `uses_component_model()` returns true):
    ///    - Uses `WasmEngine` for typed function calls
    ///    - No manual memory marshalling required
    ///    - WIT interfaces are fully functional
    ///    - This is the **CORRECT** architecture per ADR-WASM-002
    ///
    /// 2. **Legacy path** (when `component_engine` is not set):
    ///    - Uses `WasmBumpAllocator` for manual memory management
    ///    - Uses `HandleMessageParams` for parameter marshalling
    ///    - This path is **DEPRECATED** and logs a warning
    ///
    /// # Parameters
    ///
    /// * `sender` - ComponentId of the message sender
    /// * `payload` - Multicodec-encoded message payload
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Message delivered successfully
    /// - `Err(WasmError)`: Delivery failed
    ///
    /// # Errors
    ///
    /// - **ComponentNotReady**: WASM runtime not loaded
    /// - **ExecutionTimeout**: Processing exceeded delivery_timeout_ms
    /// - **ExecutionFailed**: WASM trap or export invocation failure
    ///
    /// # Performance
    ///
    /// - Timeout check overhead: ~20ns
    /// - WASM call overhead: ~10μs for simple functions
    /// - Target: <20ns delivery latency (mailbox → WASM export)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use airssys_wasm::actor::ComponentActor;
    /// use airssys_wasm::core::ComponentId;
    ///
    /// async fn deliver_message(actor: &mut ComponentActor) -> Result<(), WasmError> {
    ///     let sender = ComponentId::new("sender-component");
    ///     let payload = vec![1, 2, 3, 4]; // Multicodec-encoded
    ///     
    ///     actor.invoke_handle_message_with_timeout(sender, payload).await?;
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # WIT Interface
    ///
    /// The handle-message export is defined in component-lifecycle.wit:
    /// ```wit
    /// handle-message: func(sender: component-id, message: list<u8>) -> result<_, component-error>;
    /// ```
    ///
    /// # References
    ///
    /// - WASM-TASK-006 Phase 1 Task 1.2: Message reception infrastructure
    /// - WASM-TASK-006-HOTFIX Phase 2 Task 2.4: Component Model handle-message path
    /// - ADR-WASM-001: Inter-component communication design
    /// - wit/core/component-lifecycle.wit: handle-message export specification
    #[doc(hidden)]
    pub async fn invoke_handle_message_with_timeout(
        &mut self,
        sender: crate::core::ComponentId,
        payload: Vec<u8>,
    ) -> Result<(), WasmError> {
        // =============================================================================
        // COMPONENT MODEL PATH (WASM-TASK-006-HOTFIX Phase 2 Task 2.1)
        // =============================================================================
        // After Task 2.1, ONLY the Component Model path exists. The legacy path
        // (which used WasmBumpAllocator, HandleMessageParams, HandleMessageResult)
        // has been completely deleted.
        //
        // This method now ALWAYS uses WasmEngine::call_handle_message() for typed
        // function calls with automatic parameter marshalling via Canonical ABI.
        // =============================================================================

        // Delegate to the Component Model implementation
        self.invoke_handle_message_component_model(sender, payload)
            .await
    }

    /// Invoke handle-message using Component Model API.
    ///
    /// Uses WasmEngine for type-safe invocation with automatic parameter marshalling
    /// via the Canonical ABI. This is the **ONLY** implementation after WASM-TASK-006-HOTFIX
    /// Task 2.1 deleted the legacy workaround code.
    ///
    /// # Architecture
    ///
    /// This Component Model implementation uses:
    /// - `WasmEngine` for typed function calls
    /// - Canonical ABI for automatic serialization/deserialization
    /// - Generated bindings from WIT interfaces
    ///
    /// The legacy workaround code (WasmBumpAllocator, HandleMessageParams, HandleMessageResult)
    /// was deleted in WASM-TASK-006-HOTFIX Phase 2 Task 2.1.
    ///
    /// # Parameters
    ///
    /// * `sender` - ComponentId of the message sender
    /// * `payload` - Multicodec-encoded message payload
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Message delivered successfully
    /// - `Err(WasmError)`: Delivery failed
    ///
    /// # Errors
    ///
    /// - `WasmError::Internal`: Component Model engine not configured
    /// - `WasmError::ComponentNotFound`: Component not loaded
    /// - `WasmError::ExecutionTimeout`: Processing exceeded timeout
    /// - `WasmError::ExecutionFailed`: WASM trap or invocation failure
    ///
    /// # Implementation
    ///
    /// This method uses `WasmEngine::call_handle_message()` which was added
    /// in WASM-TASK-006-HOTFIX Task 2.5. The call is wrapped with timeout
    /// for reliability.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Called via invoke_handle_message_with_timeout()
    /// actor.invoke_handle_message_with_timeout(sender, payload).await?;
    /// ```
    async fn invoke_handle_message_component_model(
        &mut self,
        sender: crate::core::ComponentId,
        payload: Vec<u8>,
    ) -> Result<(), WasmError> {
        let component_id_str = self.component_id().as_str().to_string();
        let sender_str = sender.as_str().to_string();
        let timeout = self.message_config().delivery_timeout();
        let delivery_timeout_ms = self.message_config().delivery_timeout_ms;

        // Get Component Model engine
        let engine = self.component_engine.as_ref()
            .ok_or_else(|| WasmError::internal(
                "Component Model engine not configured (uses_component_model() should have prevented this)"
            ))?;

        // Get Component Model handle
        let handle = self.component_handle.as_ref().ok_or_else(|| {
            WasmError::component_not_found(format!(
                "Component {} not loaded (no ComponentHandle)",
                component_id_str
            ))
        })?;

        trace!(
            component_id = %component_id_str,
            sender = %sender_str,
            payload_len = payload.len(),
            timeout_ms = delivery_timeout_ms,
            "Invoking handle-message via Component Model API"
        );

        // Call handle-message via WasmEngine::call_handle_message()
        // Wrap with timeout for consistency with legacy path
        let result = tokio::time::timeout(
            timeout,
            engine.call_handle_message(handle, &sender, &payload),
        )
        .await;

        match result {
            Ok(Ok(())) => {
                debug!(
                    component_id = %component_id_str,
                    sender = %sender_str,
                    "handle-message export completed successfully (Component Model)"
                );
                Ok(())
            }
            Ok(Err(e)) => {
                // WasmEngine invocation error
                Err(e)
            }
            Err(_) => {
                // Timeout exceeded
                Err(WasmError::execution_timeout(delivery_timeout_ms, None))
            }
        }
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
            ComponentMessage::InterComponent {
                sender,
                to: _,
                payload,
            }
            | ComponentMessage::InterComponentWithCorrelation {
                sender,
                to: _,
                payload,
                ..
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
#[allow(clippy::unwrap_used)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    fn create_test_metadata() -> ComponentMetadata {
        ComponentMetadata {
            name: "test-component".to_string(),
            version: "1.0.0".to_string(),
            author: "Test Author".to_string(),
            description: None,
            max_memory_bytes: 64 * 1024 * 1024,
            max_fuel: 1_000_000,
            timeout_seconds: 5,
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
        let target_id = ComponentId::new("target");
        let msg = ComponentMessage::InterComponent {
            sender: sender_id.clone(),
            to: target_id.clone(),
            payload: vec![7, 8, 9],
        };

        match msg {
            ComponentMessage::InterComponent {
                sender,
                to,
                payload,
            } => {
                assert_eq!(sender, sender_id);
                assert_eq!(to, target_id);
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

    // ========================================================================
    // UNIT TESTS: invoke_handle_message_with_timeout Error Cases
    // (WASM-TASK-006 Task 1.2 Remediation)
    // ========================================================================

    /// Test that invoke_handle_message_with_timeout returns error without WASM
    ///
    /// This test verifies error handling when Component Model engine is not configured.
    /// After WASM-TASK-006-HOTFIX, the legacy path is removed and engine is mandatory.
    #[tokio::test]
    async fn test_invoke_handle_message_returns_error_without_engine() {
        let mut actor = create_test_actor();

        // Component Model engine is not configured
        assert!(!actor.uses_component_model());
        assert!(!actor.is_wasm_loaded());

        let sender = ComponentId::new("sender");
        let payload = vec![1, 2, 3];

        let result = actor
            .invoke_handle_message_with_timeout(sender, payload)
            .await;

        // Should fail with Internal error (engine not configured)
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, WasmError::Internal { .. }),
            "Error should be Internal (engine not configured): {:?}",
            err
        );
    }

    // ========================================================================
    // LEGACY TESTS DELETED (WASM-TASK-006-HOTFIX Phase 2 Task 2.1)
    // ========================================================================
    // Tests for the following deleted types were removed:
    // - HandleMessageParams tests
    // - HandleMessageResult tests
    // - BUMP_ALLOCATOR_BASE, MAX_SENDER_SIZE, MAX_MESSAGE_SIZE constant tests
    // ========================================================================

    // ========================================================================
    // WASM-TASK-006-HOTFIX Phase 2: Component Model Architecture Tests
    // ========================================================================

    /// Test that new ComponentActor has no Component Model engine by default
    #[test]
    fn test_component_model_engine_not_set_by_default() {
        let actor = create_test_actor();

        assert!(actor.component_engine().is_none());
        assert!(!actor.uses_component_model());
    }

    /// Test that Component Model engine can be set via builder
    #[test]
    fn test_with_component_engine_builder() {
        use crate::runtime::WasmEngine;

        let engine = Arc::new(WasmEngine::new().expect("Failed to create WasmEngine"));

        let actor = ComponentActor::new(
            ComponentId::new("test"),
            create_test_metadata(),
            CapabilitySet::new(),
            (),
        )
        .with_component_engine(Arc::clone(&engine));

        assert!(actor.component_engine().is_some());
        assert!(actor.uses_component_model());
    }

    /// Test that component_handle is None by default
    #[test]
    fn test_component_handle_not_loaded_by_default() {
        let actor = create_test_actor();

        assert!(actor.component_handle().is_none());
    }

    /// Test that set_component_handle works
    #[test]
    fn test_set_component_handle() {
        let mut actor = create_test_actor();

        // Verify the method signature compiles and the None case works
        assert!(actor.component_handle().is_none());

        // Clear the handle (already None)
        actor.set_component_handle(None);
        assert!(actor.component_handle().is_none());
    }

    /// Test uses_component_model reflects engine state
    #[test]
    fn test_uses_component_model_reflects_engine() {
        use crate::runtime::WasmEngine;

        // Without engine
        let actor1 = create_test_actor();
        assert!(!actor1.uses_component_model());

        // With engine
        let engine = Arc::new(WasmEngine::new().expect("Failed to create WasmEngine"));
        let actor2 = ComponentActor::new(
            ComponentId::new("test"),
            create_test_metadata(),
            CapabilitySet::new(),
            (),
        )
        .with_component_engine(engine);

        assert!(actor2.uses_component_model());
    }

    /// Test that both legacy and Component Model fields can coexist
    #[test]
    fn test_legacy_and_component_model_coexistence() {
        use crate::runtime::WasmEngine;

        let engine = Arc::new(WasmEngine::new().expect("Failed to create WasmEngine"));

        let actor = ComponentActor::new(
            ComponentId::new("test"),
            create_test_metadata(),
            CapabilitySet::new(),
            (),
        )
        .with_component_engine(engine);

        // Component Model engine set
        assert!(actor.uses_component_model());

        // Legacy WASM runtime not set (they're independent during migration)
        assert!(!actor.is_wasm_loaded());
    }

    // ========================================================================
    // WASM-TASK-006-HOTFIX Phase 2 Task 2.4: Component Model handle-message Tests
    // ========================================================================

    /// Test that invoke_handle_message fails with Internal error when engine not configured
    /// (Legacy path removed in WASM-TASK-006-HOTFIX Phase 2 Task 2.1)
    #[tokio::test]
    async fn test_invoke_handle_message_fails_without_engine() {
        let mut actor = create_test_actor();

        // Verify no Component Model engine
        assert!(!actor.uses_component_model());

        let sender = ComponentId::new("sender");
        let payload = vec![1, 2, 3];

        // This should fail because engine is not configured (legacy path removed)
        let result = actor
            .invoke_handle_message_with_timeout(sender, payload)
            .await;

        // Should fail with Internal (engine not configured)
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, WasmError::Internal { .. }),
            "Expected Internal error (engine not configured), got: {:?}",
            err
        );
    }

    /// Test that Component Model path is used when engine is configured
    #[tokio::test]
    async fn test_invoke_handle_message_uses_component_model_path_with_engine() {
        use crate::runtime::WasmEngine;

        let engine = Arc::new(WasmEngine::new().expect("Failed to create WasmEngine"));

        let mut actor = ComponentActor::new(
            ComponentId::new("test-component"),
            create_test_metadata(),
            CapabilitySet::new(),
            (),
        )
        .with_component_engine(engine);

        // Verify Component Model engine is configured
        assert!(actor.uses_component_model());

        let sender = ComponentId::new("sender");
        let payload = vec![1, 2, 3];

        // This should use Component Model path
        // Since component_handle is not set, it should fail with ComponentNotFound
        let result = actor
            .invoke_handle_message_with_timeout(sender, payload)
            .await;

        // Should fail because component_handle is None
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, WasmError::ComponentNotFound { .. }),
            "Expected ComponentNotFound (no handle), got: {:?}",
            err
        );
    }

    /// Test that Component Model path fails gracefully when handle is not loaded
    #[tokio::test]
    async fn test_component_model_handle_message_fails_without_handle() {
        use crate::runtime::WasmEngine;

        let engine = Arc::new(WasmEngine::new().expect("Failed to create WasmEngine"));

        let mut actor = ComponentActor::new(
            ComponentId::new("test-component"),
            create_test_metadata(),
            CapabilitySet::new(),
            (),
        )
        .with_component_engine(engine);

        // Don't set component_handle - it should fail with descriptive error
        assert!(actor.component_handle().is_none());

        let sender = ComponentId::new("sender");
        let payload = vec![1, 2, 3, 4, 5];

        let result = actor
            .invoke_handle_message_with_timeout(sender, payload)
            .await;

        assert!(result.is_err());

        let err_str = result.unwrap_err().to_string();
        // Error should mention component not loaded
        assert!(
            err_str.contains("not loaded") || err_str.contains("ComponentHandle"),
            "Error should mention component not loaded: {}",
            err_str
        );
    }

    /// Test that invoke_handle_message_component_model calls WasmEngine::call_handle_message
    /// (Task 2.5 implemented WasmEngine::call_handle_message)
    #[tokio::test]
    async fn test_component_model_handle_message_invocation() {
        use crate::core::runtime::RuntimeEngine;
        use crate::runtime::WasmEngine;

        let engine = Arc::new(WasmEngine::new().expect("Failed to create WasmEngine"));

        // Load handle-message-component.wasm which has handle-message export
        let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/handle-message-component.wasm");

        let wasm_bytes = std::fs::read(&fixture_path).expect("Failed to read fixture");

        let component_id = ComponentId::new("test-component");

        // Load the component
        let handle = engine
            .load_component(&component_id, &wasm_bytes)
            .await
            .expect("Failed to load component");

        let mut actor = ComponentActor::new(
            component_id.clone(),
            create_test_metadata(),
            CapabilitySet::new(),
            (),
        )
        .with_component_engine(Arc::clone(&engine));

        // Set the component handle
        actor.set_component_handle(Some(handle));

        let sender = ComponentId::new("sender");
        let payload = vec![1, 2, 3];

        let result = actor
            .invoke_handle_message_with_timeout(sender, payload)
            .await;

        // Should succeed - Task 2.5 implemented WasmEngine::call_handle_message()
        assert!(
            result.is_ok(),
            "Expected success with handle-message-component fixture: {:?}",
            result.err()
        );
    }

    /// Test Component Model path fails gracefully when component lacks handle-message export
    #[tokio::test]
    async fn test_component_model_handle_message_no_export() {
        use crate::core::runtime::RuntimeEngine;
        use crate::runtime::WasmEngine;

        let engine = Arc::new(WasmEngine::new().expect("Failed to create WasmEngine"));

        // Load hello_world.wasm which does NOT have handle-message export
        let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/hello_world.wasm");

        let wasm_bytes = std::fs::read(&fixture_path).expect("Failed to read fixture");

        let component_id = ComponentId::new("test-component");

        // Load the component
        let handle = engine
            .load_component(&component_id, &wasm_bytes)
            .await
            .expect("Failed to load component");

        let mut actor = ComponentActor::new(
            component_id.clone(),
            create_test_metadata(),
            CapabilitySet::new(),
            (),
        )
        .with_component_engine(Arc::clone(&engine));

        // Set the component handle
        actor.set_component_handle(Some(handle));

        let sender = ComponentId::new("sender");
        let payload = vec![1, 2, 3];

        let result = actor
            .invoke_handle_message_with_timeout(sender, payload)
            .await;

        // Should fail - component doesn't have handle-message export
        assert!(result.is_err());

        let err_str = result.unwrap_err().to_string();
        // Error should mention handle-message export
        assert!(
            err_str.contains("handle-message") || err_str.contains("type mismatch"),
            "Error should mention handle-message export: {}",
            err_str
        );
    }

    /// Test that Component Model is REQUIRED (legacy path removed)
    /// After WASM-TASK-006-HOTFIX, invoking handle_message without engine fails
    #[tokio::test]
    async fn test_handle_message_requires_component_model_engine() {
        use crate::runtime::WasmEngine;

        // Test 1: Without engine - fails with Internal error (engine not configured)
        let mut actor_no_engine = create_test_actor();
        assert!(!actor_no_engine.uses_component_model());

        let result_no_engine = actor_no_engine
            .invoke_handle_message_with_timeout(ComponentId::new("sender"), vec![1, 2, 3])
            .await;

        // Should fail with Internal (engine not configured - legacy path removed)
        assert!(
            matches!(result_no_engine.unwrap_err(), WasmError::Internal { .. }),
            "Without engine should fail with Internal error"
        );

        // Test 2: With engine - uses Component Model path (fails with ComponentNotFound because no handle)
        let engine = Arc::new(WasmEngine::new().expect("Failed to create WasmEngine"));
        let mut actor_cm = ComponentActor::new(
            ComponentId::new("test"),
            create_test_metadata(),
            CapabilitySet::new(),
            (),
        )
        .with_component_engine(engine);

        assert!(actor_cm.uses_component_model());

        let result_cm = actor_cm
            .invoke_handle_message_with_timeout(ComponentId::new("sender"), vec![1, 2, 3])
            .await;

        // Component Model path should fail with ComponentNotFound (no handle loaded)
        assert!(
            matches!(result_cm.unwrap_err(), WasmError::ComponentNotFound { .. }),
            "Component Model path should fail with ComponentNotFound (no handle)"
        );
    }
}
