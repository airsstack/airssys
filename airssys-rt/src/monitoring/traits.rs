//! Core monitoring traits for universal event observation.

use std::fmt::Debug;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::Serialize;

use super::error::MonitoringError;
use super::types::MonitoringSnapshot;

/// Event severity levels for filtering and categorization.
///
/// Ordered from lowest to highest severity for filtering purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum EventSeverity {
    /// Trace-level events for detailed debugging
    Trace,
    /// Debug-level events for development
    Debug,
    /// Informational events for normal operations
    Info,
    /// Warning events for potential issues
    Warning,
    /// Error events for failures
    Error,
    /// Critical events for system-level failures
    Critical,
}

/// Trait for events that can be monitored.
///
/// All event types must implement this trait to be compatible with the Monitor<E> system.
/// This trait provides compile-time type safety and zero-cost abstraction.
///
/// # Examples
/// ```
/// use airssys_rt::monitoring::{MonitoringEvent, EventSeverity};
/// use chrono::{DateTime, Utc};
/// use serde::Serialize;
///
/// #[derive(Debug, Clone, Serialize)]
/// struct MyEvent {
///     timestamp: DateTime<Utc>,
///     message: String,
/// }
///
/// impl MonitoringEvent for MyEvent {
///     const EVENT_TYPE: &'static str = "my_event";
///
///     fn timestamp(&self) -> DateTime<Utc> {
///         self.timestamp
///     }
///
///     fn severity(&self) -> EventSeverity {
///         EventSeverity::Info
///     }
/// }
/// ```
pub trait MonitoringEvent: Send + Sync + Clone + Debug + Serialize + 'static {
    /// Static event type identifier for categorization.
    const EVENT_TYPE: &'static str;

    /// Returns the timestamp when this event occurred.
    ///
    /// Uses chrono DateTime<Utc> following workspace standard §3.2.
    fn timestamp(&self) -> DateTime<Utc>;

    /// Returns the severity level of this event.
    fn severity(&self) -> EventSeverity;
}

/// Generic monitoring trait for observing and tracking events.
///
/// This trait provides a universal interface for monitoring any entity type through
/// generic constraints. Implementations can range from zero-overhead no-op monitors
/// to full-featured in-memory monitors with history tracking.
///
/// # Type Parameters
/// - `E`: The event type that implements `MonitoringEvent`
///
/// # Design Philosophy
/// - **Generic Abstraction**: Works with any event type through MonitoringEvent trait
/// - **Type Safety**: Compile-time verification of event types
/// - **Flexibility**: Multiple implementations (InMemory, Noop, Custom)
/// - **Zero-Cost Option**: NoopMonitor compiles away completely
///
/// # Examples
/// ```rust,ignore
/// // InMemoryMonitor will be available in Phase 2
/// use airssys_rt::monitoring::{Monitor, MonitoringEvent, EventSeverity, MonitoringConfig};
/// use chrono::Utc;
/// # use serde::Serialize;
///
/// # #[derive(Debug, Clone, Serialize)]
/// # struct MyEvent { timestamp: chrono::DateTime<chrono::Utc> }
/// # impl MonitoringEvent for MyEvent {
/// #     const EVENT_TYPE: &'static str = "test";
/// #     fn timestamp(&self) -> chrono::DateTime<chrono::Utc> { self.timestamp }
/// #     fn severity(&self) -> EventSeverity { EventSeverity::Info }
/// # }
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // InMemoryMonitor available in Phase 2
/// // let monitor = InMemoryMonitor::new(MonitoringConfig::default());
/// // monitor.record(MyEvent { timestamp: Utc::now() }).await?;
/// // let snapshot = monitor.snapshot().await?;
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait Monitor<E: MonitoringEvent>: Send + Sync + Clone {
    /// Records a monitoring event.
    ///
    /// # Arguments
    /// - `event`: The event to record
    ///
    /// # Errors
    /// Returns `MonitoringError::RecordError` if the event cannot be recorded.
    async fn record(&self, event: E) -> Result<(), MonitoringError>;

    /// Generates a snapshot of the current monitoring state.
    ///
    /// # Errors
    /// Returns `MonitoringError::SnapshotError` if the snapshot cannot be generated.
    async fn snapshot(&self) -> Result<MonitoringSnapshot<E>, MonitoringError>;

    /// Resets the monitor state, clearing all counters and history.
    ///
    /// # Errors
    /// Returns `MonitoringError::ResetError` if the reset operation fails.
    async fn reset(&self) -> Result<(), MonitoringError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Serialize)]
    struct TestEvent {
        timestamp: DateTime<Utc>,
        message: String,
        severity: EventSeverity,
    }

    impl MonitoringEvent for TestEvent {
        const EVENT_TYPE: &'static str = "test_event";

        fn timestamp(&self) -> DateTime<Utc> {
            self.timestamp
        }

        fn severity(&self) -> EventSeverity {
            self.severity
        }
    }

    #[test]
    fn test_event_severity_ordering() {
        assert!(EventSeverity::Trace < EventSeverity::Debug);
        assert!(EventSeverity::Debug < EventSeverity::Info);
        assert!(EventSeverity::Info < EventSeverity::Warning);
        assert!(EventSeverity::Warning < EventSeverity::Error);
        assert!(EventSeverity::Error < EventSeverity::Critical);
    }

    #[test]
    fn test_event_severity_equality() {
        assert_eq!(EventSeverity::Info, EventSeverity::Info);
        assert_ne!(EventSeverity::Info, EventSeverity::Warning);
    }

    #[test]
    fn test_monitoring_event_implementation() {
        let now = Utc::now();
        let event = TestEvent {
            timestamp: now,
            message: "Test message".to_string(),
            severity: EventSeverity::Info,
        };

        assert_eq!(event.timestamp(), now);
        assert_eq!(event.severity(), EventSeverity::Info);
        assert_eq!(TestEvent::EVENT_TYPE, "test_event");
    }

    #[test]
    fn test_monitoring_event_clone() {
        let event = TestEvent {
            timestamp: Utc::now(),
            message: "Original".to_string(),
            severity: EventSeverity::Debug,
        };

        let cloned = event.clone();
        assert_eq!(event.timestamp(), cloned.timestamp());
        assert_eq!(event.message, cloned.message);
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_event_severity_serialization() {
        let severity = EventSeverity::Warning;
        let json = serde_json::to_string(&severity).expect("Serialization should succeed");
        assert!(json.contains("Warning"));
    }
}
