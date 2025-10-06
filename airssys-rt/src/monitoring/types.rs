//! Monitoring event types and configuration structures.

use std::collections::HashMap;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::Serialize;

use super::traits::{EventSeverity, MonitoringEvent};
use crate::util::ActorId;

/// Configuration for monitoring behavior.
///
/// Controls how events are recorded, filtered, and stored in the monitor.
#[derive(Debug, Clone, Serialize)]
pub struct MonitoringConfig {
    /// Whether monitoring is enabled
    pub enabled: bool,

    /// Maximum number of events to keep in history
    pub max_history_size: usize,

    /// Minimum severity level to record (events below this are filtered)
    pub severity_filter: EventSeverity,

    /// Interval for snapshot generation (if using background snapshots)
    #[serde(with = "crate::util::duration_serde")]
    pub snapshot_interval: Duration,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_history_size: 1000,
            severity_filter: EventSeverity::Info,
            snapshot_interval: Duration::from_secs(60),
        }
    }
}

/// Snapshot of monitoring state at a point in time.
///
/// Provides queryable access to monitoring counters and recent event history.
#[derive(Debug, Clone, Serialize)]
pub struct MonitoringSnapshot<E: MonitoringEvent> {
    /// Timestamp when snapshot was taken
    pub timestamp: DateTime<Utc>,

    /// Total number of events recorded
    pub total_events: u64,

    /// Number of trace-level events
    pub trace_count: u64,

    /// Number of debug-level events
    pub debug_count: u64,

    /// Number of info-level events
    pub info_count: u64,

    /// Number of warning-level events
    pub warning_count: u64,

    /// Number of error-level events
    pub error_count: u64,

    /// Number of critical-level events
    pub critical_count: u64,

    /// Recent events (up to max_history_size)
    pub recent_events: Vec<E>,
}

// ============================================================================
// Supervision Events
// ============================================================================

/// Events related to supervisor operations and child management.
#[derive(Debug, Clone, Serialize)]
pub struct SupervisionEvent {
    /// Timestamp when event occurred (§3.2 chrono DateTime<Utc>)
    pub timestamp: DateTime<Utc>,

    /// ID of the supervisor
    pub supervisor_id: String,

    /// ID of the child actor (if applicable)
    pub child_id: Option<String>,

    /// Specific supervision event type
    pub event_kind: SupervisionEventKind,

    /// Additional event metadata
    pub metadata: HashMap<String, String>,
}

impl MonitoringEvent for SupervisionEvent {
    const EVENT_TYPE: &'static str = "supervision";

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn severity(&self) -> EventSeverity {
        match &self.event_kind {
            SupervisionEventKind::ChildStarted => EventSeverity::Info,
            SupervisionEventKind::ChildStopped => EventSeverity::Info,
            SupervisionEventKind::ChildFailed { .. } => EventSeverity::Error,
            SupervisionEventKind::ChildRestarted { .. } => EventSeverity::Warning,
            SupervisionEventKind::RestartLimitExceeded { .. } => EventSeverity::Critical,
            SupervisionEventKind::StrategyApplied { .. } => EventSeverity::Info,
        }
    }
}

/// Specific types of supervision events.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum SupervisionEventKind {
    /// Child actor successfully started
    ChildStarted,

    /// Child actor gracefully stopped
    ChildStopped,

    /// Child actor failed with error
    ChildFailed {
        /// Error message
        error: String,
        /// Current restart count
        restart_count: u32,
    },

    /// Child actor was restarted after failure
    ChildRestarted {
        /// Restart count after this restart
        restart_count: u32,
    },

    /// Restart rate limit exceeded
    RestartLimitExceeded {
        /// Number of restarts attempted
        restart_count: u32,
        /// Time window for restart limit
        #[serde(with = "crate::util::duration_serde")]
        window: Duration,
    },

    /// Supervision strategy was applied
    StrategyApplied {
        /// Name of the strategy (OneForOne, OneForAll, RestForOne)
        strategy: String,
        /// Number of children affected
        affected_count: usize,
    },
}

// ============================================================================
// Actor Events
// ============================================================================

/// Events related to actor lifecycle and message processing.
#[derive(Debug, Clone, Serialize)]
pub struct ActorEvent {
    /// Timestamp when event occurred (§3.2 chrono DateTime<Utc>)
    pub timestamp: DateTime<Utc>,

    /// ID of the actor
    pub actor_id: ActorId,

    /// Specific actor event type
    pub event_kind: ActorEventKind,

    /// Additional event metadata
    pub metadata: HashMap<String, String>,
}

impl MonitoringEvent for ActorEvent {
    const EVENT_TYPE: &'static str = "actor";

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn severity(&self) -> EventSeverity {
        match &self.event_kind {
            ActorEventKind::Spawned => EventSeverity::Debug,
            ActorEventKind::Started => EventSeverity::Info,
            ActorEventKind::MessageReceived { .. } => EventSeverity::Trace,
            ActorEventKind::MessageProcessed { .. } => EventSeverity::Trace,
            ActorEventKind::ErrorOccurred { .. } => EventSeverity::Error,
            ActorEventKind::Stopped => EventSeverity::Info,
        }
    }
}

/// Specific types of actor events.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ActorEventKind {
    /// Actor was spawned
    Spawned,

    /// Actor started execution
    Started,

    /// Actor received a message
    MessageReceived {
        /// Message type name
        message_type: String,
    },

    /// Actor finished processing a message
    MessageProcessed {
        /// Message type name
        message_type: String,
        /// Processing duration in microseconds
        duration_micros: u64,
    },

    /// Error occurred during actor execution
    ErrorOccurred {
        /// Error message
        error: String,
    },

    /// Actor stopped execution
    Stopped,
}

// ============================================================================
// System Events
// ============================================================================

/// Events related to actor system operations.
#[derive(Debug, Clone, Serialize)]
pub struct SystemEvent {
    /// Timestamp when event occurred (§3.2 chrono DateTime<Utc>)
    pub timestamp: DateTime<Utc>,

    /// Specific system event type
    pub event_kind: SystemEventKind,

    /// Additional event metadata
    pub metadata: HashMap<String, String>,
}

impl MonitoringEvent for SystemEvent {
    const EVENT_TYPE: &'static str = "system";

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn severity(&self) -> EventSeverity {
        match &self.event_kind {
            SystemEventKind::Started => EventSeverity::Info,
            SystemEventKind::Shutdown => EventSeverity::Info,
            SystemEventKind::ActorRegistered { .. } => EventSeverity::Debug,
            SystemEventKind::ActorDeregistered { .. } => EventSeverity::Debug,
            SystemEventKind::ConfigurationChanged => EventSeverity::Info,
        }
    }
}

/// Specific types of system events.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum SystemEventKind {
    /// Actor system started
    Started,

    /// Actor system shutting down
    Shutdown,

    /// Actor registered with the system
    ActorRegistered {
        /// ID of the registered actor
        actor_id: ActorId,
    },

    /// Actor deregistered from the system
    ActorDeregistered {
        /// ID of the deregistered actor
        actor_id: ActorId,
    },

    /// System configuration was changed
    ConfigurationChanged,
}

// ============================================================================
// Broker Events
// ============================================================================

/// Events related to message broker operations.
#[derive(Debug, Clone, Serialize)]
pub struct BrokerEvent {
    /// Timestamp when event occurred (§3.2 chrono DateTime<Utc>)
    pub timestamp: DateTime<Utc>,

    /// Specific broker event type
    pub event_kind: BrokerEventKind,

    /// Additional event metadata
    pub metadata: HashMap<String, String>,
}

impl MonitoringEvent for BrokerEvent {
    const EVENT_TYPE: &'static str = "broker";

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn severity(&self) -> EventSeverity {
        match &self.event_kind {
            BrokerEventKind::MessagePublished { .. } => EventSeverity::Trace,
            BrokerEventKind::MessageRouted { .. } => EventSeverity::Trace,
            BrokerEventKind::SubscriberAdded { .. } => EventSeverity::Debug,
            BrokerEventKind::SubscriberRemoved { .. } => EventSeverity::Debug,
            BrokerEventKind::RoutingFailed { .. } => EventSeverity::Error,
            BrokerEventKind::DeadLetter { .. } => EventSeverity::Warning,
        }
    }
}

/// Specific types of broker events.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum BrokerEventKind {
    /// Message published to broker
    MessagePublished {
        /// Message type name
        message_type: String,
    },

    /// Message successfully routed to actor
    MessageRouted {
        /// Destination actor ID
        actor_id: ActorId,
        /// Message type name
        message_type: String,
    },

    /// New subscriber added
    SubscriberAdded {
        /// Subscriber identifier
        subscriber_id: String,
    },

    /// Subscriber removed
    SubscriberRemoved {
        /// Subscriber identifier
        subscriber_id: String,
    },

    /// Failed to route message
    RoutingFailed {
        /// Target actor ID
        actor_id: ActorId,
        /// Failure reason
        reason: String,
    },

    /// Message sent to dead letter queue
    DeadLetter {
        /// Message type name
        message_type: String,
        /// Reason for dead lettering
        reason: String,
    },
}

// ============================================================================
// Mailbox Events
// ============================================================================

/// Events related to mailbox operations and backpressure.
#[derive(Debug, Clone, Serialize)]
pub struct MailboxEvent {
    /// Timestamp when event occurred (§3.2 chrono DateTime<Utc>)
    pub timestamp: DateTime<Utc>,

    /// ID of the actor owning the mailbox
    pub actor_id: ActorId,

    /// Specific mailbox event type
    pub event_kind: MailboxEventKind,

    /// Additional event metadata
    pub metadata: HashMap<String, String>,
}

impl MonitoringEvent for MailboxEvent {
    const EVENT_TYPE: &'static str = "mailbox";

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn severity(&self) -> EventSeverity {
        match &self.event_kind {
            MailboxEventKind::MessageEnqueued { .. } => EventSeverity::Trace,
            MailboxEventKind::MessageDequeued { .. } => EventSeverity::Trace,
            MailboxEventKind::BackpressureApplied { .. } => EventSeverity::Warning,
            MailboxEventKind::CapacityReached => EventSeverity::Warning,
            MailboxEventKind::MessageDropped { .. } => EventSeverity::Error,
        }
    }
}

/// Specific types of mailbox events.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum MailboxEventKind {
    /// Message enqueued in mailbox
    MessageEnqueued {
        /// Current queue size
        queue_size: usize,
    },

    /// Message dequeued from mailbox
    MessageDequeued {
        /// Remaining queue size
        queue_size: usize,
    },

    /// Backpressure strategy applied
    BackpressureApplied {
        /// Strategy applied (Block, Drop, Error)
        strategy: String,
    },

    /// Mailbox capacity reached
    CapacityReached,

    /// Message dropped due to backpressure
    MessageDropped {
        /// Reason for dropping
        reason: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitoring_config_default() {
        let config = MonitoringConfig::default();
        assert!(config.enabled);
        assert_eq!(config.max_history_size, 1000);
        assert_eq!(config.severity_filter, EventSeverity::Info);
        assert_eq!(config.snapshot_interval, Duration::from_secs(60));
    }

    #[test]
    fn test_supervision_event_severity() {
        let event = SupervisionEvent {
            timestamp: Utc::now(),
            supervisor_id: "sup-1".to_string(),
            child_id: Some("child-1".to_string()),
            event_kind: SupervisionEventKind::ChildFailed {
                error: "Connection lost".to_string(),
                restart_count: 1,
            },
            metadata: HashMap::new(),
        };

        assert_eq!(event.severity(), EventSeverity::Error);
        assert_eq!(SupervisionEvent::EVENT_TYPE, "supervision");
    }

    #[test]
    fn test_actor_event_severity() {
        let actor_id = ActorId::new();
        let event = ActorEvent {
            timestamp: Utc::now(),
            actor_id,
            event_kind: ActorEventKind::Started,
            metadata: HashMap::new(),
        };

        assert_eq!(event.severity(), EventSeverity::Info);
        assert_eq!(ActorEvent::EVENT_TYPE, "actor");
    }

    #[test]
    fn test_system_event_severity() {
        let event = SystemEvent {
            timestamp: Utc::now(),
            event_kind: SystemEventKind::Started,
            metadata: HashMap::new(),
        };

        assert_eq!(event.severity(), EventSeverity::Info);
        assert_eq!(SystemEvent::EVENT_TYPE, "system");
    }

    #[test]
    fn test_broker_event_severity() {
        let event = BrokerEvent {
            timestamp: Utc::now(),
            event_kind: BrokerEventKind::RoutingFailed {
                actor_id: ActorId::new(),
                reason: "Actor not found".to_string(),
            },
            metadata: HashMap::new(),
        };

        assert_eq!(event.severity(), EventSeverity::Error);
        assert_eq!(BrokerEvent::EVENT_TYPE, "broker");
    }

    #[test]
    fn test_mailbox_event_severity() {
        let event = MailboxEvent {
            timestamp: Utc::now(),
            actor_id: ActorId::new(),
            event_kind: MailboxEventKind::MessageDropped {
                reason: "Mailbox full".to_string(),
            },
            metadata: HashMap::new(),
        };

        assert_eq!(event.severity(), EventSeverity::Error);
        assert_eq!(MailboxEvent::EVENT_TYPE, "mailbox");
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_supervision_event_kind_serialization() {
        let kind = SupervisionEventKind::ChildFailed {
            error: "Test error".to_string(),
            restart_count: 3,
        };

        let json = serde_json::to_string(&kind).expect("Serialization should succeed");
        assert!(json.contains("ChildFailed"));
        assert!(json.contains("Test error"));
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_actor_event_kind_message_processed() {
        let kind = ActorEventKind::MessageProcessed {
            message_type: "PingMessage".to_string(),
            duration_micros: 150,
        };

        let json = serde_json::to_string(&kind).expect("Serialization should succeed");
        assert!(json.contains("MessageProcessed"));
        assert!(json.contains("150"));
    }

    #[test]
    fn test_monitoring_snapshot_creation() {
        let snapshot = MonitoringSnapshot::<ActorEvent> {
            timestamp: Utc::now(),
            total_events: 100,
            trace_count: 10,
            debug_count: 20,
            info_count: 30,
            warning_count: 25,
            error_count: 10,
            critical_count: 5,
            recent_events: vec![],
        };

        assert_eq!(snapshot.total_events, 100);
        assert_eq!(snapshot.trace_count, 10);
        assert_eq!(snapshot.critical_count, 5);
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_event_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("custom_key".to_string(), "custom_value".to_string());

        let event = SystemEvent {
            timestamp: Utc::now(),
            event_kind: SystemEventKind::ConfigurationChanged,
            metadata,
        };

        assert_eq!(
            event.metadata.get("custom_key").expect("Key should exist"),
            "custom_value"
        );
    }
}
