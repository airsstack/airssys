//! Actor integration abstractions for WASM components.
//!
//! These types define the message envelope, supervision strategies, lifecycle states,
//! and metadata needed for integrating WASM components with the airssys-rt actor system.
//! They provide the foundation for Block 3 (Actor System Integration) without implementation
//! details, following YAGNI principles (§6.1).
//!
//! # Design Rationale
//!
//! - **ActorMessage**: Uses ComponentOutput for payload to align with component execution model.
//!   Timestamps use chrono::DateTime<Utc> per §3.2 for consistency.
//! - **SupervisionStrategy**: Covers standard Erlang-style supervision (restart/stop/escalate).
//! - **ActorState**: Complete lifecycle from initialization to termination.
//! - **ActorMetadata**: Tracks essential runtime state for monitoring and supervision.
//!
//! All types are Clone + Serialize/Deserialize for message passing and persistence.
//! No internal dependencies beyond core (zero circular deps).
//!
//! # References
//!
//! - ADR-WASM-010: Implementation Strategy (Block 3 foundational)
//! - KNOWLEDGE-WASM-005: Messaging Architecture (actor integration patterns)
//! - airssys-rt: MessageBroker and SupervisorNode contracts

// Layer 1: Standard library

// Layer 2: External crates
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Layer 3: Internal (core only)
use crate::core::component::{ComponentId, ComponentOutput};

/// Message envelope for actor-based messaging between WASM components.
///
/// This struct wraps messages sent via the airssys-rt MessageBroker, providing
/// routing information, correlation for request-response patterns, and a payload
/// from component execution. Used in Block 3 for ComponentActor communication.
///
/// # Example
///
/// ```
/// use airssys_wasm::core::component::{ComponentId, ComponentOutput};
/// use airssys_wasm::core::actor::ActorMessage;
/// use chrono::Utc;
/// use std::collections::HashMap;
///
/// let message = ActorMessage {
///     from: ComponentId::new("component-a"),
///     to: ComponentId::new("component-b"),
///     message_id: "msg-123".to_string(),
///     correlation_id: Some("req-456".to_string()),
///     payload: ComponentOutput {
///         data: vec![1, 2, 3],
///         codec: 0x55,
///         metadata: HashMap::new(),
///     },
///     timestamp: Utc::now(),
/// };
///
/// assert_eq!(message.from.as_str(), "component-a");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorMessage {
    /// Source component ID.
    pub from: ComponentId,

    /// Destination component ID.
    pub to: ComponentId,

    /// Unique message identifier (for deduplication).
    pub message_id: String,

    /// Optional correlation ID for request-response patterns.
    pub correlation_id: Option<String>,

    /// Message payload from component execution.
    pub payload: ComponentOutput,

    /// UTC timestamp when message was created.
    pub timestamp: DateTime<Utc>,
}

impl ActorMessage {
    /// Create a new fire-and-forget message.
    pub fn fire_and_forget(from: ComponentId, to: ComponentId, payload: ComponentOutput) -> Self {
        Self {
            from,
            to,
            message_id: Uuid::new_v4().to_string(),
            correlation_id: None,
            payload,
            timestamp: Utc::now(),
        }
    }

    /// Create a new request message (expects response).
    pub fn request(
        from: ComponentId,
        to: ComponentId,
        payload: ComponentOutput,
        correlation_id: impl Into<String>,
    ) -> Self {
        Self {
            from,
            to,
            message_id: Uuid::new_v4().to_string(),
            correlation_id: Some(correlation_id.into()),
            payload,
            timestamp: Utc::now(),
        }
    }

    /// Check if this is a request message (has correlation ID).
    pub fn is_request(&self) -> bool {
        self.correlation_id.is_some()
    }

    /// Get the message age in milliseconds.
    pub fn age_ms(&self) -> u64 {
        let now = Utc::now();
        (now - self.timestamp).num_milliseconds() as u64
    }
}

/// Supervision strategy for component actors.
///
/// Defines how the SupervisorNode should respond to component failures.
/// Aligns with airssys-rt supervision patterns for Block 3 integration.
///
/// # Example
///
/// ```
/// use airssys_wasm::core::actor::SupervisionStrategy;
///
/// let strategy = SupervisionStrategy::Restart;
/// assert_eq!(format!("{:?}", strategy), "Restart");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SupervisionStrategy {
    /// Restart the failed component actor.
    Restart,

    /// Stop the failed component actor permanently.
    Stop,

    /// Escalate failure to parent supervisor.
    Escalate,
}

impl SupervisionStrategy {
    /// Check if this strategy restarts on failure.
    pub fn restarts(&self) -> bool {
        matches!(self, Self::Restart)
    }

    /// Check if this strategy stops on failure.
    pub fn stops(&self) -> bool {
        matches!(self, Self::Stop)
    }
}

/// Actor lifecycle state for WASM component actors.
///
/// Represents the state machine for ComponentActor instances in the airssys-rt system.
/// Used for monitoring and supervision decisions in Block 3.
///
/// # Example
///
/// ```
/// use airssys_wasm::core::actor::ActorState;
///
/// let state = ActorState::Ready;
/// assert!(matches!(state, ActorState::Ready));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActorState {
    /// Actor is initializing (Component::init() running).
    Initializing,

    /// Actor is ready to receive messages.
    Ready,

    /// Actor is processing a message.
    Processing,

    /// Actor is temporarily suspended (e.g., resource limits).
    Suspended,

    /// Actor is shutting down (Component::shutdown() running).
    Terminating,

    /// Actor has terminated (either normally or due to failure).
    Terminated,
}

impl ActorState {
    /// Check if actor is active (can process messages).
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Ready | Self::Processing)
    }

    /// Check if actor is terminal (cannot recover).
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Terminated)
    }
}

/// Metadata for a component actor instance.
///
/// Tracks runtime state for supervision, monitoring, and observability.
/// Used in Block 3 for SupervisorNode integration and Block 10 for metrics.
///
/// # Example
///
/// ```
/// use airssys_wasm::core::actor::{ActorMetadata, ActorState};
/// use airssys_wasm::core::component::ComponentId;
///
/// let metadata = ActorMetadata {
///     actor_id: "actor-123".to_string(),
///     component_id: ComponentId::new("my-component"),
///     mailbox_size: 5,
///     state: ActorState::Ready,
///     restart_count: 0,
/// };
///
/// assert_eq!(metadata.state, ActorState::Ready);
/// assert_eq!(metadata.restart_count, 0);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorMetadata {
    /// Unique actor instance ID (generated by airssys-rt).
    pub actor_id: String,

    /// Associated WASM component ID.
    pub component_id: ComponentId,

    /// Current mailbox size (pending messages).
    pub mailbox_size: usize,

    /// Current actor state.
    pub state: ActorState,

    /// Number of restarts due to failures.
    pub restart_count: u32,
}

impl ActorMetadata {
    /// Create new metadata for an initializing actor.
    pub fn new(actor_id: impl Into<String>, component_id: ComponentId) -> Self {
        Self {
            actor_id: actor_id.into(),
            component_id,
            mailbox_size: 0,
            state: ActorState::Initializing,
            restart_count: 0,
        }
    }

    /// Update mailbox size after message operations.
    pub fn update_mailbox_size(&mut self, size: usize) {
        self.mailbox_size = size;
    }

    /// Transition to a new state (e.g., after lifecycle event).
    pub fn transition_state(&mut self, new_state: ActorState) {
        self.state = new_state;
    }

    /// Increment restart count (after supervision restart).
    pub fn increment_restart(&mut self) {
        self.restart_count += 1;
    }

    /// Check if actor has exceeded restart limits (e.g., >3 restarts).
    pub fn has_exceeded_restarts(&self, max_restarts: u32) -> bool {
        self.restart_count > max_restarts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use std::collections::HashMap;  // Local to tests for metadata

    #[test]
    fn test_actor_message_creation() {
        let from = ComponentId::new("sender");
        let to = ComponentId::new("receiver");
        let payload = ComponentOutput {
            data: vec![1, 2, 3],
            codec: 0x55,
            metadata: HashMap::new(),
        };

        let msg = ActorMessage::fire_and_forget(from.clone(), to.clone(), payload.clone());
        assert_eq!(msg.from, from);
        assert_eq!(msg.to, to);
        assert!(!msg.message_id.is_empty());
        assert!(msg.correlation_id.is_none());
        assert_eq!(msg.payload, payload);
        assert!(msg.timestamp <= Utc::now());
    }

    #[test]
    fn test_actor_message_request() {
        let from = ComponentId::new("client");
        let to = ComponentId::new("server");
        let payload = ComponentOutput {
            data: vec![],
            codec: 0,
            metadata: HashMap::new(),
        };
        let msg = ActorMessage::request(from, to, payload, "req-001");

        assert!(msg.is_request());
        assert_eq!(msg.correlation_id, Some("req-001".to_string()));
    }

    #[test]
    fn test_actor_message_age() {
        let timestamp = Utc::now() - chrono::Duration::milliseconds(100);
        let msg = ActorMessage {
            from: ComponentId::new("test"),
            to: ComponentId::new("test"),
            message_id: "test".to_string(),
            correlation_id: None,
            payload: ComponentOutput {
                data: vec![],
                codec: 0,
                metadata: HashMap::new(),
            },
            timestamp,
        };

        let age = msg.age_ms();
        assert!(
            (99..=101).contains(&age),
            "Expected age ~100ms, got {age}ms (timing tolerance)"
        );
    }

    #[test]
    fn test_supervision_strategy() {
        assert!(SupervisionStrategy::Restart.restarts());
        assert!(!SupervisionStrategy::Restart.stops());
        assert!(SupervisionStrategy::Stop.stops());
        assert!(!SupervisionStrategy::Stop.restarts());
    }

    #[test]
    fn test_actor_state() {
        assert!(ActorState::Ready.is_active());
        assert!(ActorState::Processing.is_active());
        assert!(!ActorState::Terminated.is_active());
        assert!(ActorState::Terminated.is_terminal());
        assert!(!ActorState::Initializing.is_terminal());
    }

    #[test]
    fn test_actor_metadata_creation() {
        let metadata = ActorMetadata::new("actor-001", ComponentId::new("comp-001"));
        assert_eq!(metadata.actor_id, "actor-001");
        assert_eq!(metadata.component_id.as_str(), "comp-001");
        assert_eq!(metadata.mailbox_size, 0);
        assert_eq!(metadata.state, ActorState::Initializing);
        assert_eq!(metadata.restart_count, 0);
    }

    #[test]
    fn test_actor_metadata_updates() {
        let mut metadata = ActorMetadata::new("actor-001", ComponentId::new("comp-001"));
        metadata.update_mailbox_size(10);
        assert_eq!(metadata.mailbox_size, 10);

        metadata.transition_state(ActorState::Ready);
        assert_eq!(metadata.state, ActorState::Ready);

        metadata.increment_restart();
        assert_eq!(metadata.restart_count, 1);

        assert!(!metadata.has_exceeded_restarts(2));
        metadata.increment_restart();
        metadata.increment_restart();
        assert!(metadata.has_exceeded_restarts(2));
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_actor_message_serialization() {
        use std::collections::HashMap;
        
        let msg = ActorMessage::fire_and_forget(
            ComponentId::new("sender"),
            ComponentId::new("receiver"),
            ComponentOutput {
                data: vec![1, 2, 3],
                codec: 0x0200, // JSON multicodec
                metadata: HashMap::new(),
            },
        );

        let json = serde_json::to_value(&msg).expect("serialization should succeed");
        assert_eq!(json["from"], "sender");
        assert_eq!(json["to"], "receiver");

        let deserialized: ActorMessage = serde_json::from_value(json).expect("deserialization should succeed");
        assert_eq!(deserialized.from.as_str(), "sender");
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_actor_metadata_serialization() {
        let metadata = ActorMetadata {
            actor_id: "actor-001".to_string(),
            component_id: ComponentId::new("comp-001"),
            mailbox_size: 5,
            state: ActorState::Ready,
            restart_count: 1,
        };

        let json = serde_json::to_value(&metadata).expect("serialization should succeed");
        assert_eq!(json["actor_id"], "actor-001");
        assert_eq!(json["component_id"], "comp-001");
        assert_eq!(json["mailbox_size"], 5);
        assert_eq!(json["state"], "Ready");
        assert_eq!(json["restart_count"], 1);

        let deserialized: ActorMetadata = serde_json::from_value(json).expect("deserialization should succeed");
        assert_eq!(deserialized.actor_id, "actor-001");
        assert_eq!(deserialized.state, ActorState::Ready);
    }
}
