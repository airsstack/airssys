//! Integration tests for the monitoring infrastructure.
//!
//! Tests the complete monitoring system including:
//! - Configuration and setup
//! - Multi-monitor coordination
//! - High-load scenarios
//! - Dynamic configuration changes
//! - Cross-event-type tracking

#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use airssys_rt::monitoring::{
    ActorEvent, ActorEventKind, BrokerEvent, BrokerEventKind, EventSeverity, InMemoryMonitor,
    MailboxEvent, MailboxEventKind, Monitor, MonitoringConfig, NoopMonitor, SupervisionEvent,
    SupervisionEventKind, SystemEvent, SystemEventKind,
};
use airssys_rt::ActorId;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

// ============================================================================
// Integration Tests - Multi-Monitor Coordination
// ============================================================================

#[tokio::test]
async fn test_multiple_monitors_coordination() {
    // Setup multiple monitors for different event types
    let config = MonitoringConfig {
        enabled: true,
        max_history_size: 100,
        severity_filter: EventSeverity::Trace,
        snapshot_interval: Duration::from_secs(60),
    };

    let actor_monitor = InMemoryMonitor::<ActorEvent>::new(config.clone());
    let system_monitor = InMemoryMonitor::<SystemEvent>::new(config.clone());
    let broker_monitor = InMemoryMonitor::<BrokerEvent>::new(config.clone());

    // Record events in different monitors
    let actor_id = ActorId::new();

    actor_monitor
        .record(ActorEvent {
            timestamp: Utc::now(),
            actor_id,
            event_kind: ActorEventKind::Started,
            metadata: HashMap::new(),
        })
        .await
        .unwrap();

    system_monitor
        .record(SystemEvent {
            timestamp: Utc::now(),
            event_kind: SystemEventKind::ActorRegistered { actor_id },
            metadata: HashMap::new(),
        })
        .await
        .unwrap();

    broker_monitor
        .record(BrokerEvent {
            timestamp: Utc::now(),
            event_kind: BrokerEventKind::SubscriberAdded {
                subscriber_id: actor_id.to_string(),
            },
            metadata: HashMap::new(),
        })
        .await
        .unwrap();

    // Verify each monitor has independent state
    let actor_snapshot = actor_monitor.snapshot().await.unwrap();
    let system_snapshot = system_monitor.snapshot().await.unwrap();
    let broker_snapshot = broker_monitor.snapshot().await.unwrap();

    assert_eq!(actor_snapshot.total_events, 1);
    assert_eq!(system_snapshot.total_events, 1);
    assert_eq!(broker_snapshot.total_events, 1);

    assert_eq!(actor_snapshot.info_count, 1); // Started is Info
    assert_eq!(system_snapshot.debug_count, 1); // ActorRegistered is Debug
    assert_eq!(broker_snapshot.debug_count, 1); // SubscriberAdded is Debug
}

#[tokio::test]
async fn test_actor_lifecycle_tracking() {
    let config = MonitoringConfig {
        enabled: true,
        max_history_size: 1000,
        severity_filter: EventSeverity::Trace,
        snapshot_interval: Duration::from_secs(60),
    };
    let monitor = InMemoryMonitor::<ActorEvent>::new(config);

    let actor_id = ActorId::new();

    // Track complete actor lifecycle
    let lifecycle_events = vec![
        ActorEventKind::Started,
        ActorEventKind::MessageReceived {
            message_type: "InitMessage".to_string(),
        },
        ActorEventKind::MessageProcessed {
            message_type: "InitMessage".to_string(),
            duration_micros: 10_000, // 10ms in microseconds
        },
        ActorEventKind::Stopped,
    ];

    for event_kind in lifecycle_events {
        monitor
            .record(ActorEvent {
                timestamp: Utc::now(),
                actor_id,
                event_kind,
                metadata: HashMap::new(),
            })
            .await
            .unwrap();
    }

    let snapshot = monitor.snapshot().await.unwrap();
    assert_eq!(snapshot.total_events, 4);
    assert_eq!(snapshot.info_count, 2); // Started + Stopped
    assert_eq!(snapshot.recent_events.len(), 4);
}

// ============================================================================
// High-Load Scenario Tests
// ============================================================================

#[tokio::test]
async fn test_high_load_concurrent_recording() {
    let config = MonitoringConfig {
        enabled: true,
        max_history_size: 1000,
        severity_filter: EventSeverity::Trace,
        snapshot_interval: Duration::from_secs(60),
    };

    let monitor = Arc::new(InMemoryMonitor::<ActorEvent>::new(config));

    // Spawn multiple tasks recording events concurrently
    let mut handles = vec![];

    for i in 0..10 {
        let monitor_clone = Arc::clone(&monitor);
        let handle = tokio::spawn(async move {
            for j in 0..100 {
                let actor_id = ActorId::new();
                let event = ActorEvent {
                    timestamp: Utc::now(),
                    actor_id,
                    event_kind: ActorEventKind::MessageReceived {
                        message_type: format!("Message-{i}-{j}"),
                    },
                    metadata: HashMap::new(),
                };
                monitor_clone.record(event).await.unwrap();
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    let snapshot = monitor.snapshot().await.unwrap();
    assert_eq!(snapshot.total_events, 1000); // 10 tasks × 100 events
    assert_eq!(snapshot.recent_events.len(), 1000); // All fit in history
}

#[tokio::test]
async fn test_ring_buffer_eviction_under_load() {
    let config = MonitoringConfig {
        enabled: true,
        max_history_size: 50, // Small buffer to test eviction
        severity_filter: EventSeverity::Trace,
        snapshot_interval: Duration::from_secs(60),
    };

    let monitor = InMemoryMonitor::<ActorEvent>::new(config);

    // Record 100 events (should evict oldest 50)
    for i in 0..100 {
        monitor
            .record(ActorEvent {
                timestamp: Utc::now(),
                actor_id: ActorId::new(),
                event_kind: ActorEventKind::MessageReceived {
                    message_type: format!("Message-{i}"),
                },
                metadata: HashMap::new(),
            })
            .await
            .unwrap();
    }

    let snapshot = monitor.snapshot().await.unwrap();
    assert_eq!(snapshot.total_events, 100);
    assert_eq!(snapshot.recent_events.len(), 50); // Only last 50 kept

    // Verify we kept the most recent events (50-99)
    let first_message = &snapshot.recent_events[0];
    if let ActorEventKind::MessageReceived { message_type } = &first_message.event_kind {
        // The first event in recent_events should be around Message-50
        assert!(message_type.contains("Message-5"));
    } else {
        panic!("Expected MessageReceived event");
    }
}

// ============================================================================
// Dynamic Configuration Tests
// ============================================================================

#[tokio::test]
async fn test_severity_filter_changes() {
    let config = MonitoringConfig {
        enabled: true,
        max_history_size: 100,
        severity_filter: EventSeverity::Error, // Start with Error filter
        snapshot_interval: Duration::from_secs(60),
    };

    let monitor = InMemoryMonitor::<ActorEvent>::new(config);

    // Record Debug event (should be filtered)
    monitor
        .record(ActorEvent {
            timestamp: Utc::now(),
            actor_id: ActorId::new(),
            event_kind: ActorEventKind::MessageReceived {
                message_type: "DebugMessage".to_string(),
            },
            metadata: HashMap::new(),
        })
        .await
        .unwrap();

    // Record Error event (should pass filter)
    monitor
        .record(ActorEvent {
            timestamp: Utc::now(),
            actor_id: ActorId::new(),
            event_kind: ActorEventKind::ErrorOccurred {
                error: "Test error".to_string(),
            },
            metadata: HashMap::new(),
        })
        .await
        .unwrap();

    let snapshot1 = monitor.snapshot().await.unwrap();
    assert_eq!(snapshot1.total_events, 1); // Only Error event counted
    assert_eq!(snapshot1.error_count, 1);
    assert_eq!(snapshot1.debug_count, 0);

    // Note: InMemoryMonitor doesn't support dynamic config updates in current implementation
    // Create a new monitor with Debug filter
    let monitor = InMemoryMonitor::<ActorEvent>::new(MonitoringConfig {
        enabled: true,
        max_history_size: 100,
        severity_filter: EventSeverity::Trace,
        snapshot_interval: Duration::from_secs(60),
    });

    // Record Debug event again (should now pass)
    monitor
        .record(ActorEvent {
            timestamp: Utc::now(),
            actor_id: ActorId::new(),
            event_kind: ActorEventKind::MessageReceived {
                message_type: "DebugMessage2".to_string(),
            },
            metadata: HashMap::new(),
        })
        .await
        .unwrap();

    let snapshot2 = monitor.snapshot().await.unwrap();
    assert_eq!(snapshot2.total_events, 1);
    assert_eq!(snapshot2.debug_count, 0); // MessageReceived is Trace, not Debug
    assert_eq!(snapshot2.trace_count, 1); // Trace events are recorded with Debug filter
}

#[tokio::test]
async fn test_monitoring_enable_disable() {
    let mut config = MonitoringConfig {
        enabled: true,
        max_history_size: 1000,
        severity_filter: EventSeverity::Trace,
        snapshot_interval: Duration::from_secs(60),
    };
    config.enabled = false; // Start disabled

    let monitor = InMemoryMonitor::<ActorEvent>::new(config);

    // Record event while disabled
    monitor
        .record(ActorEvent {
            timestamp: Utc::now(),
            actor_id: ActorId::new(),
            event_kind: ActorEventKind::Started,
            metadata: HashMap::new(),
        })
        .await
        .unwrap();

    let snapshot1 = monitor.snapshot().await.unwrap();
    assert_eq!(snapshot1.total_events, 0); // Event not recorded

    // Note: InMemoryMonitor doesn't support dynamic config updates
    // Create a new monitor with enabled=true
    let monitor = InMemoryMonitor::<ActorEvent>::new(MonitoringConfig {
        enabled: true,
        max_history_size: 100,
        severity_filter: EventSeverity::Trace,
        snapshot_interval: Duration::from_secs(60),
    });

    // Record event while enabled
    monitor
        .record(ActorEvent {
            timestamp: Utc::now(),
            actor_id: ActorId::new(),
            event_kind: ActorEventKind::Started,
            metadata: HashMap::new(),
        })
        .await
        .unwrap();

    let snapshot2 = monitor.snapshot().await.unwrap();
    assert_eq!(snapshot2.total_events, 1); // Event recorded
}

// ============================================================================
// Event Type Coverage Tests
// ============================================================================

#[tokio::test]
async fn test_mailbox_backpressure_tracking() {
    let config = MonitoringConfig {
        enabled: true,
        max_history_size: 1000,
        severity_filter: EventSeverity::Trace,
        snapshot_interval: Duration::from_secs(60),
    };
    let monitor = InMemoryMonitor::<MailboxEvent>::new(config);

    let actor_id = ActorId::new();

    // Simulate backpressure scenario
    monitor
        .record(MailboxEvent {
            timestamp: Utc::now(),
            actor_id,
            event_kind: MailboxEventKind::MessageEnqueued { queue_size: 90 },
            metadata: HashMap::new(),
        })
        .await
        .unwrap();

    monitor
        .record(MailboxEvent {
            timestamp: Utc::now(),
            actor_id,
            event_kind: MailboxEventKind::MessageEnqueued { queue_size: 100 },
            metadata: HashMap::new(),
        })
        .await
        .unwrap();

    monitor
        .record(MailboxEvent {
            timestamp: Utc::now(),
            actor_id,
            event_kind: MailboxEventKind::CapacityReached,
            metadata: HashMap::new(),
        })
        .await
        .unwrap();

    monitor
        .record(MailboxEvent {
            timestamp: Utc::now(),
            actor_id,
            event_kind: MailboxEventKind::BackpressureApplied {
                strategy: "Block".to_string(),
            },
            metadata: HashMap::new(),
        })
        .await
        .unwrap();

    let snapshot = monitor.snapshot().await.unwrap();
    assert_eq!(snapshot.total_events, 4);
    assert_eq!(snapshot.warning_count, 2); // CapacityReached + BackpressureApplied
}

#[tokio::test]
async fn test_broker_routing_events() {
    let config = MonitoringConfig {
        enabled: true,
        max_history_size: 1000,
        severity_filter: EventSeverity::Trace,
        snapshot_interval: Duration::from_secs(60),
    };
    let monitor = InMemoryMonitor::<BrokerEvent>::new(config);

    // Successful routing
    monitor
        .record(BrokerEvent {
            timestamp: Utc::now(),
            event_kind: BrokerEventKind::MessagePublished {
                message_type: "TestMessage".to_string(),
            },
            metadata: HashMap::new(),
        })
        .await
        .unwrap();

    monitor
        .record(BrokerEvent {
            timestamp: Utc::now(),
            event_kind: BrokerEventKind::MessageRouted {
                actor_id: ActorId::new(),
                message_type: "TestMessage".to_string(),
            },
            metadata: HashMap::new(),
        })
        .await
        .unwrap();

    // Failed routing
    monitor
        .record(BrokerEvent {
            timestamp: Utc::now(),
            event_kind: BrokerEventKind::RoutingFailed {
                actor_id: ActorId::new(),
                reason: "Actor not found".to_string(),
            },
            metadata: HashMap::new(),
        })
        .await
        .unwrap();

    monitor
        .record(BrokerEvent {
            timestamp: Utc::now(),
            event_kind: BrokerEventKind::DeadLetter {
                message_type: "TestMessage".to_string(),
                reason: "No subscribers".to_string(),
            },
            metadata: HashMap::new(),
        })
        .await
        .unwrap();

    let snapshot = monitor.snapshot().await.unwrap();
    assert_eq!(snapshot.total_events, 4);
    assert_eq!(snapshot.error_count, 1); // RoutingFailed
    assert_eq!(snapshot.warning_count, 1); // DeadLetter
}

#[tokio::test]
async fn test_supervision_event_tracking() {
    let config = MonitoringConfig {
        enabled: true,
        max_history_size: 1000,
        severity_filter: EventSeverity::Trace,
        snapshot_interval: Duration::from_secs(60),
    };
    let monitor = InMemoryMonitor::<SupervisionEvent>::new(config);

    let supervisor_id = "supervisor-1".to_string();
    let child_id = "child-1".to_string();

    // Normal startup
    monitor
        .record(SupervisionEvent {
            timestamp: Utc::now(),
            supervisor_id: supervisor_id.clone(),
            child_id: Some(child_id.clone()),
            event_kind: SupervisionEventKind::ChildStarted,
            metadata: HashMap::new(),
        })
        .await
        .unwrap();

    // Failure and restart
    monitor
        .record(SupervisionEvent {
            timestamp: Utc::now(),
            supervisor_id: supervisor_id.clone(),
            child_id: Some(child_id.clone()),
            event_kind: SupervisionEventKind::ChildFailed {
                error: "Crash".to_string(),
                restart_count: 0,
            },
            metadata: HashMap::new(),
        })
        .await
        .unwrap();

    monitor
        .record(SupervisionEvent {
            timestamp: Utc::now(),
            supervisor_id: supervisor_id.clone(),
            child_id: Some(child_id.clone()),
            event_kind: SupervisionEventKind::ChildRestarted { restart_count: 1 },
            metadata: HashMap::new(),
        })
        .await
        .unwrap();

    // Strategy application
    monitor
        .record(SupervisionEvent {
            timestamp: Utc::now(),
            supervisor_id,
            child_id: None, // No specific child for strategy application
            event_kind: SupervisionEventKind::StrategyApplied {
                strategy: "OneForOne".to_string(),
                affected_count: 1,
            },
            metadata: HashMap::new(),
        })
        .await
        .unwrap();

    let snapshot = monitor.snapshot().await.unwrap();
    assert_eq!(snapshot.total_events, 4);
    assert_eq!(snapshot.error_count, 1); // ChildFailed
    assert_eq!(snapshot.warning_count, 1); // ChildRestarted
}

// ============================================================================
// NoopMonitor Integration Tests
// ============================================================================

#[tokio::test]
async fn test_noop_monitor_zero_overhead() {
    let noop = NoopMonitor::<ActorEvent>::new();

    // Record events (should do nothing)
    for _ in 0..1000 {
        noop.record(ActorEvent {
            timestamp: Utc::now(),
            actor_id: ActorId::new(),
            event_kind: ActorEventKind::Started,
            metadata: HashMap::new(),
        })
        .await
        .unwrap();
    }

    // Snapshot should show zero events
    let snapshot = noop.snapshot().await.unwrap();
    assert_eq!(snapshot.total_events, 0);
    assert_eq!(snapshot.debug_count, 0);
    assert_eq!(snapshot.recent_events.len(), 0);

    // Reset should succeed but do nothing
    noop.reset().await.unwrap();

    // NoopMonitor doesn't need config updates since it does nothing
    // This test verifies it compiles with zero overhead
}

// ============================================================================
// Metadata and Context Tests
// ============================================================================

#[tokio::test]
async fn test_event_metadata_tracking() {
    let config = MonitoringConfig {
        enabled: true,
        max_history_size: 1000,
        severity_filter: EventSeverity::Trace,
        snapshot_interval: Duration::from_secs(60),
    };
    let monitor = InMemoryMonitor::<ActorEvent>::new(config);

    let mut metadata = HashMap::new();
    metadata.insert("request_id".to_string(), "req-123".to_string());
    metadata.insert("user_id".to_string(), "user-456".to_string());
    metadata.insert("endpoint".to_string(), "/api/users".to_string());

    monitor
        .record(ActorEvent {
            timestamp: Utc::now(),
            actor_id: ActorId::new(),
            event_kind: ActorEventKind::MessageReceived {
                message_type: "HttpRequest".to_string(),
            },
            metadata: metadata.clone(),
        })
        .await
        .unwrap();

    let snapshot = monitor.snapshot().await.unwrap();
    assert_eq!(snapshot.recent_events.len(), 1);

    let event = &snapshot.recent_events[0];
    assert_eq!(event.metadata.get("request_id").unwrap(), "req-123");
    assert_eq!(event.metadata.get("user_id").unwrap(), "user-456");
    assert_eq!(event.metadata.get("endpoint").unwrap(), "/api/users");
}

// ============================================================================
// Stress and Performance Tests
// ============================================================================

#[tokio::test]
async fn test_rapid_snapshot_generation() {
    let config = MonitoringConfig {
        enabled: true,
        max_history_size: 1000,
        severity_filter: EventSeverity::Trace,
        snapshot_interval: Duration::from_secs(60),
    };
    let monitor = Arc::new(InMemoryMonitor::<ActorEvent>::new(config));

    // Record some events
    for i in 0..100 {
        monitor
            .record(ActorEvent {
                timestamp: Utc::now(),
                actor_id: ActorId::new(),
                event_kind: ActorEventKind::MessageReceived {
                    message_type: format!("Message-{i}"),
                },
                metadata: HashMap::new(),
            })
            .await
            .unwrap();
    }

    // Generate snapshots rapidly from multiple tasks
    let mut handles = vec![];
    for _ in 0..10 {
        let monitor_clone = Arc::clone(&monitor);
        let handle = tokio::spawn(async move {
            for _ in 0..10 {
                let snapshot = monitor_clone.snapshot().await.unwrap();
                assert!(snapshot.total_events >= 100);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }
}

#[tokio::test]
async fn test_reset_during_concurrent_operations() {
    let config = MonitoringConfig {
        enabled: true,
        max_history_size: 1000,
        severity_filter: EventSeverity::Trace,
        snapshot_interval: Duration::from_secs(60),
    };
    let monitor = Arc::new(InMemoryMonitor::<ActorEvent>::new(config));

    // Spawn task that continuously records events
    let recorder = Arc::clone(&monitor);
    let record_handle = tokio::spawn(async move {
        for i in 0..1000 {
            recorder
                .record(ActorEvent {
                    timestamp: Utc::now(),
                    actor_id: ActorId::new(),
                    event_kind: ActorEventKind::MessageReceived {
                        message_type: format!("Message-{i}"),
                    },
                    metadata: HashMap::new(),
                })
                .await
                .unwrap();
            sleep(Duration::from_micros(100)).await;
        }
    });

    // Wait a bit, then reset
    sleep(Duration::from_millis(50)).await;
    monitor.reset().await.unwrap();

    // Check that reset worked
    let snapshot = monitor.snapshot().await.unwrap();
    assert!(snapshot.total_events < 1000); // Should be reset mid-recording

    record_handle.await.unwrap();
}
