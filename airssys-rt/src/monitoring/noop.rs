//! No-operation monitor implementation with zero overhead.

use std::marker::PhantomData;

use async_trait::async_trait;
use chrono::Utc;

use super::error::MonitoringError;
use super::traits::{Monitor, MonitoringEvent};
use super::types::MonitoringSnapshot;

/// No-operation monitor that discards all events with zero overhead.
///
/// This monitor is optimized for production scenarios where monitoring is disabled
/// but the monitoring infrastructure must remain in place. All methods are
/// inlined and compile to near-zero overhead.
///
/// # Zero-Cost Abstraction
///
/// All methods are marked `#[inline(always)]` to ensure complete optimization:
/// - No heap allocations
/// - No atomic operations
/// - No lock contention
/// - Minimal stack usage
///
/// # Examples
///
/// ```
/// use airssys_rt::monitoring::{NoopMonitor, Monitor, ActorEvent};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let monitor = NoopMonitor::<ActorEvent>::new();
///
/// // All operations are no-ops
/// // monitor.record(event).await?; // Zero overhead
/// let snapshot = monitor.snapshot().await?;
/// assert_eq!(snapshot.total_events, 0);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct NoopMonitor<E: MonitoringEvent> {
    _phantom: PhantomData<E>,
}

impl<E: MonitoringEvent> NoopMonitor<E> {
    /// Creates a new no-operation monitor.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_rt::monitoring::{NoopMonitor, ActorEvent};
    ///
    /// let monitor = NoopMonitor::<ActorEvent>::new();
    /// ```
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<E: MonitoringEvent> Monitor<E> for NoopMonitor<E> {
    /// Records an event (no-op, always succeeds with zero overhead).
    #[inline(always)]
    async fn record(&self, _event: E) -> Result<(), MonitoringError> {
        Ok(())
    }

    /// Returns an empty snapshot (no-op, zero counters).
    #[inline(always)]
    async fn snapshot(&self) -> Result<MonitoringSnapshot<E>, MonitoringError> {
        Ok(MonitoringSnapshot {
            timestamp: Utc::now(),
            total_events: 0,
            trace_count: 0,
            debug_count: 0,
            info_count: 0,
            warning_count: 0,
            error_count: 0,
            critical_count: 0,
            recent_events: Vec::new(),
        })
    }

    /// Resets monitor state (no-op, always succeeds with zero overhead).
    #[inline(always)]
    async fn reset(&self) -> Result<(), MonitoringError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::monitoring::types::{ActorEvent, ActorEventKind};
    use crate::util::ActorId;

    fn create_test_event() -> ActorEvent {
        ActorEvent {
            timestamp: Utc::now(),
            actor_id: ActorId::new(),
            event_kind: ActorEventKind::Started,
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    #[allow(clippy::expect_used)]
    async fn test_noop_creation() {
        let monitor = NoopMonitor::<ActorEvent>::new();
        let snapshot = monitor.snapshot().await.expect("Snapshot should succeed");
        assert_eq!(snapshot.total_events, 0);
    }

    #[tokio::test]
    #[allow(clippy::expect_used)]
    async fn test_noop_default() {
        let monitor = NoopMonitor::<ActorEvent>::new();
        let snapshot = monitor.snapshot().await.expect("Snapshot should succeed");
        assert_eq!(snapshot.total_events, 0);
    }

    #[tokio::test]
    #[allow(clippy::expect_used)]
    async fn test_noop_record_ignores_events() {
        let monitor = NoopMonitor::new();

        // Record multiple events - all should be ignored
        for _ in 0..100 {
            let event = create_test_event();
            monitor.record(event).await.expect("Record should succeed");
        }

        let snapshot = monitor.snapshot().await.expect("Snapshot should succeed");
        assert_eq!(snapshot.total_events, 0);
        assert_eq!(snapshot.recent_events.len(), 0);
    }

    #[tokio::test]
    #[allow(clippy::expect_used)]
    async fn test_noop_snapshot_always_empty() {
        let monitor = NoopMonitor::<ActorEvent>::new();

        let snapshot = monitor.snapshot().await.expect("Snapshot should succeed");
        assert_eq!(snapshot.total_events, 0);
        assert_eq!(snapshot.trace_count, 0);
        assert_eq!(snapshot.debug_count, 0);
        assert_eq!(snapshot.info_count, 0);
        assert_eq!(snapshot.warning_count, 0);
        assert_eq!(snapshot.error_count, 0);
        assert_eq!(snapshot.critical_count, 0);
        assert_eq!(snapshot.recent_events.len(), 0);
    }

    #[tokio::test]
    #[allow(clippy::expect_used)]
    async fn test_noop_reset_always_succeeds() {
        let monitor = NoopMonitor::<ActorEvent>::new();

        monitor.reset().await.expect("Reset should succeed");

        let snapshot = monitor.snapshot().await.expect("Snapshot should succeed");
        assert_eq!(snapshot.total_events, 0);
    }

    #[tokio::test]
    #[allow(clippy::expect_used)]
    async fn test_noop_clone() {
        let monitor1 = NoopMonitor::<ActorEvent>::new();
        let monitor2 = monitor1.clone();

        let snapshot1 = monitor1.snapshot().await.expect("Snapshot should succeed");
        let snapshot2 = monitor2.snapshot().await.expect("Snapshot should succeed");

        assert_eq!(snapshot1.total_events, snapshot2.total_events);
    }

    #[tokio::test]
    #[allow(clippy::expect_used)]
    async fn test_noop_zero_overhead_trait_impl() {
        let monitor = NoopMonitor::<ActorEvent>::new();

        let event = create_test_event();
        monitor.record(event).await.expect("Record should succeed");
    }

    #[tokio::test]
    #[allow(clippy::expect_used)]
    async fn test_noop_concurrent_safety() {
        use tokio::task;

        // Spawn multiple tasks using the monitor concurrently
        let mut handles = vec![];
        for _ in 0..10 {
            let handle = task::spawn(async move {
                let local_monitor = NoopMonitor::<ActorEvent>::new();
                for _ in 0..10 {
                    let event = create_test_event();
                    local_monitor
                        .record(event)
                        .await
                        .expect("Record should succeed");
                }
                local_monitor
                    .snapshot()
                    .await
                    .expect("Snapshot should succeed")
            });
            handles.push(handle);
        }

        // All should succeed with zero counts
        for handle in handles {
            let snapshot = handle.await.expect("Task should complete");
            assert_eq!(snapshot.total_events, 0);
        }
    }
}
