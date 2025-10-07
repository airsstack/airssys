//! In-memory monitor implementation with lock-free atomic counters.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use chrono::Utc;

use super::error::MonitoringError;
use super::traits::{EventSeverity, Monitor, MonitoringEvent};
use super::types::{MonitoringConfig, MonitoringSnapshot};

/// In-memory monitor implementation with lock-free atomic counters.
///
/// This implementation provides efficient concurrent event recording using atomic
/// operations for counters and a bounded ring buffer for event history.
///
/// # Architecture
///
/// Uses the M-SERVICES-CLONE pattern with Arc<Inner> for cheap cloning:
/// - Atomic counters for lock-free event counting
/// - RwLock for ring buffer (read-heavy optimization)
/// - Bounded memory with configurable history size
///
/// # Examples
///
/// ```
/// use airssys_rt::monitoring::{InMemoryMonitor, Monitor, MonitoringConfig, ActorEvent, ActorEventKind};
/// use chrono::Utc;
/// use std::collections::HashMap;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = MonitoringConfig::default();
/// let monitor = InMemoryMonitor::new(config);
///
/// // Record an event
/// let event = ActorEvent {
///     timestamp: Utc::now(),
///     actor_id: airssys_rt::util::ActorId::new(),
///     event_kind: ActorEventKind::Started,
///     metadata: HashMap::new(),
/// };
/// monitor.record(event).await?;
///
/// // Get snapshot
/// let snapshot = monitor.snapshot().await?;
/// assert_eq!(snapshot.total_events, 1);
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct InMemoryMonitor<E: MonitoringEvent> {
    inner: Arc<InMemoryMonitorInner<E>>,
}

/// Inner state for InMemoryMonitor following M-SERVICES-CLONE pattern.
#[derive(Debug)]
struct InMemoryMonitorInner<E: MonitoringEvent> {
    config: MonitoringConfig,

    // Lock-free atomic counters for concurrent access
    total_events: AtomicU64,
    trace_count: AtomicU64,
    debug_count: AtomicU64,
    info_count: AtomicU64,
    warning_count: AtomicU64,
    error_count: AtomicU64,
    critical_count: AtomicU64,

    // Ring buffer for event history (read-heavy optimization with RwLock)
    history: RwLock<VecDeque<E>>,
}

impl<E: MonitoringEvent> InMemoryMonitor<E> {
    /// Creates a new in-memory monitor with the given configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_rt::monitoring::{InMemoryMonitor, MonitoringConfig, ActorEvent};
    ///
    /// let config = MonitoringConfig::default();
    /// let monitor = InMemoryMonitor::<ActorEvent>::new(config);
    /// ```
    pub fn new(config: MonitoringConfig) -> Self {
        Self {
            inner: Arc::new(InMemoryMonitorInner {
                config,
                total_events: AtomicU64::new(0),
                trace_count: AtomicU64::new(0),
                debug_count: AtomicU64::new(0),
                info_count: AtomicU64::new(0),
                warning_count: AtomicU64::new(0),
                error_count: AtomicU64::new(0),
                critical_count: AtomicU64::new(0),
                history: RwLock::new(VecDeque::new()),
            }),
        }
    }

    /// Increments the appropriate severity counter atomically.
    fn increment_severity_counter(&self, severity: EventSeverity) {
        match severity {
            EventSeverity::Trace => self.inner.trace_count.fetch_add(1, Ordering::Relaxed),
            EventSeverity::Debug => self.inner.debug_count.fetch_add(1, Ordering::Relaxed),
            EventSeverity::Info => self.inner.info_count.fetch_add(1, Ordering::Relaxed),
            EventSeverity::Warning => self.inner.warning_count.fetch_add(1, Ordering::Relaxed),
            EventSeverity::Error => self.inner.error_count.fetch_add(1, Ordering::Relaxed),
            EventSeverity::Critical => self.inner.critical_count.fetch_add(1, Ordering::Relaxed),
        };
    }
}

impl<E: MonitoringEvent> Clone for InMemoryMonitor<E> {
    /// Cheap clone using Arc (M-SERVICES-CLONE pattern).
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

#[async_trait]
impl<E: MonitoringEvent> Monitor<E> for InMemoryMonitor<E> {
    async fn record(&self, event: E) -> Result<(), MonitoringError> {
        // Early return if monitoring is disabled
        if !self.inner.config.enabled {
            return Ok(());
        }

        let severity = event.severity();

        // Early return if below severity threshold
        if severity < self.inner.config.severity_filter {
            return Ok(());
        }

        // Increment total events counter (lock-free)
        self.inner.total_events.fetch_add(1, Ordering::Relaxed);

        // Increment severity-specific counter (lock-free)
        self.increment_severity_counter(severity);

        // Add to ring buffer history
        let mut history =
            self.inner.history.write().map_err(|e| {
                MonitoringError::record(format!("Failed to acquire write lock: {e}"))
            })?;

        // Enforce ring buffer size limit
        if history.len() >= self.inner.config.max_history_size {
            history.pop_front();
        }

        history.push_back(event);

        Ok(())
    }

    async fn snapshot(&self) -> Result<MonitoringSnapshot<E>, MonitoringError> {
        // Read all atomic counters
        let total_events = self.inner.total_events.load(Ordering::Relaxed);
        let trace_count = self.inner.trace_count.load(Ordering::Relaxed);
        let debug_count = self.inner.debug_count.load(Ordering::Relaxed);
        let info_count = self.inner.info_count.load(Ordering::Relaxed);
        let warning_count = self.inner.warning_count.load(Ordering::Relaxed);
        let error_count = self.inner.error_count.load(Ordering::Relaxed);
        let critical_count = self.inner.critical_count.load(Ordering::Relaxed);

        // Read history with RwLock
        let history =
            self.inner.history.read().map_err(|e| {
                MonitoringError::snapshot(format!("Failed to acquire read lock: {e}"))
            })?;

        let recent_events = history.iter().cloned().collect();

        Ok(MonitoringSnapshot {
            timestamp: Utc::now(),
            total_events,
            trace_count,
            debug_count,
            info_count,
            warning_count,
            error_count,
            critical_count,
            recent_events,
        })
    }

    async fn reset(&self) -> Result<(), MonitoringError> {
        // Reset all atomic counters to 0
        self.inner.total_events.store(0, Ordering::Relaxed);
        self.inner.trace_count.store(0, Ordering::Relaxed);
        self.inner.debug_count.store(0, Ordering::Relaxed);
        self.inner.info_count.store(0, Ordering::Relaxed);
        self.inner.warning_count.store(0, Ordering::Relaxed);
        self.inner.error_count.store(0, Ordering::Relaxed);
        self.inner.critical_count.store(0, Ordering::Relaxed);

        // Clear ring buffer history
        let mut history =
            self.inner.history.write().map_err(|e| {
                MonitoringError::reset(format!("Failed to acquire write lock: {e}"))
            })?;

        history.clear();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::monitoring::types::{ActorEvent, ActorEventKind};
    use crate::util::ActorId;

    fn create_test_event(_severity: EventSeverity) -> ActorEvent {
        ActorEvent {
            timestamp: Utc::now(),
            actor_id: ActorId::new(),
            event_kind: ActorEventKind::Started,
            metadata: HashMap::new(),
        }
    }

    fn create_test_event_with_kind(kind: ActorEventKind) -> ActorEvent {
        ActorEvent {
            timestamp: Utc::now(),
            actor_id: ActorId::new(),
            event_kind: kind,
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    #[allow(clippy::expect_used)]
    async fn test_monitor_creation() {
        let config = MonitoringConfig::default();
        let monitor = InMemoryMonitor::<ActorEvent>::new(config);

        let snapshot = monitor.snapshot().await.expect("Snapshot should succeed");
        assert_eq!(snapshot.total_events, 0);
        assert_eq!(snapshot.recent_events.len(), 0);
    }

    #[tokio::test]
    #[allow(clippy::expect_used)]
    async fn test_record_single_event() {
        let config = MonitoringConfig::default();
        let monitor = InMemoryMonitor::new(config);

        let event = create_test_event(EventSeverity::Info);
        monitor.record(event).await.expect("Record should succeed");

        let snapshot = monitor.snapshot().await.expect("Snapshot should succeed");
        assert_eq!(snapshot.total_events, 1);
        assert_eq!(snapshot.info_count, 1);
        assert_eq!(snapshot.recent_events.len(), 1);
    }

    #[tokio::test]
    #[allow(clippy::expect_used)]
    async fn test_record_multiple_events() {
        let config = MonitoringConfig::default();
        let monitor = InMemoryMonitor::new(config);

        for _ in 0..10 {
            let event = create_test_event(EventSeverity::Info);
            monitor.record(event).await.expect("Record should succeed");
        }

        let snapshot = monitor.snapshot().await.expect("Snapshot should succeed");
        assert_eq!(snapshot.total_events, 10);
        assert_eq!(snapshot.info_count, 10);
        assert_eq!(snapshot.recent_events.len(), 10);
    }

    #[tokio::test]
    #[allow(clippy::expect_used)]
    async fn test_severity_filtering() {
        let config = MonitoringConfig {
            severity_filter: EventSeverity::Warning,
            ..MonitoringConfig::default()
        };
        let monitor = InMemoryMonitor::new(config);

        // These should be filtered out
        let info_event = create_test_event_with_kind(ActorEventKind::Started);
        monitor
            .record(info_event)
            .await
            .expect("Record should succeed");

        // This should be recorded
        let error_event = create_test_event_with_kind(ActorEventKind::ErrorOccurred {
            error: "test error".to_string(),
        });
        monitor
            .record(error_event)
            .await
            .expect("Record should succeed");

        let snapshot = monitor.snapshot().await.expect("Snapshot should succeed");
        assert_eq!(snapshot.total_events, 1); // Only error event
        assert_eq!(snapshot.error_count, 1);
        assert_eq!(snapshot.info_count, 0);
    }

    #[tokio::test]
    #[allow(clippy::expect_used)]
    async fn test_ring_buffer_overflow() {
        let config = MonitoringConfig {
            max_history_size: 5,
            ..MonitoringConfig::default()
        };
        let monitor = InMemoryMonitor::new(config);

        // Record more events than max_history_size
        for _ in 0..10 {
            let event = create_test_event(EventSeverity::Info);
            monitor.record(event).await.expect("Record should succeed");
        }

        let snapshot = monitor.snapshot().await.expect("Snapshot should succeed");
        assert_eq!(snapshot.total_events, 10); // Counter tracks all
        assert_eq!(snapshot.recent_events.len(), 5); // History limited to 5
    }

    #[tokio::test]
    #[allow(clippy::expect_used)]
    async fn test_reset_functionality() {
        let config = MonitoringConfig::default();
        let monitor = InMemoryMonitor::new(config);

        // Record some events
        for _ in 0..5 {
            let event = create_test_event(EventSeverity::Info);
            monitor.record(event).await.expect("Record should succeed");
        }

        // Reset
        monitor.reset().await.expect("Reset should succeed");

        let snapshot = monitor.snapshot().await.expect("Snapshot should succeed");
        assert_eq!(snapshot.total_events, 0);
        assert_eq!(snapshot.info_count, 0);
        assert_eq!(snapshot.recent_events.len(), 0);
    }

    #[tokio::test]
    #[allow(clippy::expect_used)]
    async fn test_clone_implementation() {
        let config = MonitoringConfig::default();
        let monitor1 = InMemoryMonitor::new(config);

        // Record event on first monitor
        let event = create_test_event(EventSeverity::Info);
        monitor1.record(event).await.expect("Record should succeed");

        // Clone monitor
        let monitor2 = monitor1.clone();

        // Both should see the same state
        let snapshot1 = monitor1.snapshot().await.expect("Snapshot should succeed");
        let snapshot2 = monitor2.snapshot().await.expect("Snapshot should succeed");

        assert_eq!(snapshot1.total_events, snapshot2.total_events);
        assert_eq!(snapshot1.info_count, snapshot2.info_count);
    }

    #[tokio::test]
    #[allow(clippy::expect_used)]
    async fn test_disabled_monitoring() {
        let config = MonitoringConfig {
            enabled: false,
            ..MonitoringConfig::default()
        };
        let monitor = InMemoryMonitor::new(config);

        let event = create_test_event(EventSeverity::Info);
        monitor.record(event).await.expect("Record should succeed");

        let snapshot = monitor.snapshot().await.expect("Snapshot should succeed");
        assert_eq!(snapshot.total_events, 0); // Should not record
    }

    #[tokio::test]
    #[allow(clippy::expect_used)]
    async fn test_severity_counters() {
        let config = MonitoringConfig {
            severity_filter: EventSeverity::Trace, // Allow all event levels
            ..MonitoringConfig::default()
        };
        let monitor = InMemoryMonitor::new(config);

        // Record events with different severities
        let events = vec![
            create_test_event_with_kind(ActorEventKind::Spawned), // Debug
            create_test_event_with_kind(ActorEventKind::Started), // Info
            create_test_event_with_kind(ActorEventKind::ErrorOccurred {
                error: "test".to_string(),
            }), // Error
        ];

        for event in events {
            monitor.record(event).await.expect("Record should succeed");
        }

        let snapshot = monitor.snapshot().await.expect("Snapshot should succeed");
        assert_eq!(snapshot.total_events, 3);
        assert_eq!(snapshot.debug_count, 1);
        assert_eq!(snapshot.info_count, 1);
        assert_eq!(snapshot.error_count, 1);
    }

    #[tokio::test]
    #[allow(clippy::expect_used)]
    async fn test_concurrent_recording() {
        use tokio::task;

        let config = MonitoringConfig::default();
        let monitor = InMemoryMonitor::new(config);

        // Spawn multiple tasks recording concurrently
        let mut handles = vec![];
        for _ in 0..10 {
            let monitor_clone = monitor.clone();
            let handle = task::spawn(async move {
                for _ in 0..10 {
                    let event = create_test_event(EventSeverity::Info);
                    monitor_clone
                        .record(event)
                        .await
                        .expect("Record should succeed");
                }
            });
            handles.push(handle);
        }

        // Wait for all tasks
        for handle in handles {
            handle.await.expect("Task should complete");
        }

        let snapshot = monitor.snapshot().await.expect("Snapshot should succeed");
        assert_eq!(snapshot.total_events, 100); // 10 tasks * 10 events
        assert_eq!(snapshot.info_count, 100);
    }
}
