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
// (none needed)

// Layer 2: Third-party crate imports
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedReceiver;

// Layer 3: Internal module imports
use crate::core::{
    CapabilitySet, ComponentId, ComponentMetadata,
};

/// Stub for WASM runtime until Block 1 integration complete.
///
/// TODO(TASK 1.2): Replace with actual WasmRuntime from runtime module.
/// This stub allows Task 1.1 (structure and traits) to proceed independently.
#[derive(Debug)]
pub struct WasmRuntime {
    // Stub implementation - will be replaced with actual Wasmtime runtime
    _placeholder: (),
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
#[derive(Debug, Clone)]
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

/// Component health status.
///
/// Reported by ComponentActor in response to HealthCheck messages.
/// Used by supervisors for proactive failure detection.
///
/// # Variants
///
/// - **Healthy**: Component operating normally
/// - **Degraded**: Component operational but experiencing issues
/// - **Unhealthy**: Component failed or non-functional
///
/// # Example
///
/// ```rust
/// use airssys_wasm::actor::HealthStatus;
///
/// let health = HealthStatus::Healthy;
/// assert!(matches!(health, HealthStatus::Healthy));
///
/// let degraded = HealthStatus::Degraded {
///     reason: "High error rate".to_string(),
/// };
///
/// let unhealthy = HealthStatus::Unhealthy {
///     reason: "WASM not loaded".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthStatus {
    /// Component operating normally
    Healthy,

    /// Component operational but experiencing issues
    Degraded {
        /// Reason for degraded status
        reason: String,
    },

    /// Component failed or non-functional
    Unhealthy {
        /// Reason for unhealthy status
        reason: String,
    },
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
