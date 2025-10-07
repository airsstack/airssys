//! Basic Monitoring Example
//!
//! Demonstrates fundamental monitoring capabilities using InMemoryMonitor:
//! - Creating monitors with configuration
//! - Recording various event types
//! - Taking snapshots for observability
//! - Severity filtering
//! - Resetting monitor state
//!
//! Run with: cargo run --example monitoring_basic

use std::collections::HashMap;
use std::time::Duration;

use chrono::Utc;

use airssys_rt::monitoring::{
    ActorEvent, ActorEventKind, BrokerEvent, BrokerEventKind, EventSeverity, InMemoryMonitor,
    MailboxEvent, MailboxEventKind, Monitor, MonitoringConfig, SystemEvent, SystemEventKind,
};
use airssys_rt::util::ActorId;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== AirsSys RT - Basic Monitoring Example ===\n");

    // Example 1: Basic InMemoryMonitor setup
    basic_monitoring_setup().await?;

    // Example 2: Severity filtering
    severity_filtering_example().await?;

    // Example 3: Multiple event types
    multiple_event_types_example().await?;

    // Example 4: Snapshot and reset
    snapshot_and_reset_example().await?;

    println!("\n=== All examples completed successfully! ===");
    Ok(())
}

/// Example 1: Basic monitoring setup and event recording
async fn basic_monitoring_setup() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Example 1: Basic Monitoring Setup ---");

    // Create monitor with default configuration
    let config = MonitoringConfig::default();
    let monitor = InMemoryMonitor::new(config);

    // Record a few actor events
    for i in 1..=5 {
        let event = ActorEvent {
            timestamp: Utc::now(),
            actor_id: ActorId::new(),
            event_kind: ActorEventKind::Started,
            metadata: HashMap::from([("iteration".to_string(), i.to_string())]),
        };
        monitor.record(event).await?;
    }

    // Take a snapshot
    let snapshot = monitor.snapshot().await?;
    println!("  Total events recorded: {}", snapshot.total_events);
    println!("  Info-level events: {}", snapshot.info_count);
    println!(
        "  Recent events in history: {}\n",
        snapshot.recent_events.len()
    );

    Ok(())
}

/// Example 2: Severity filtering demonstration
async fn severity_filtering_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Example 2: Severity Filtering ---");

    // Configure monitor to only record Warning and above
    let config = MonitoringConfig {
        severity_filter: EventSeverity::Warning,
        ..MonitoringConfig::default()
    };
    let monitor = InMemoryMonitor::new(config);

    // Try to record events at different severity levels
    let debug_event = ActorEvent {
        timestamp: Utc::now(),
        actor_id: ActorId::new(),
        event_kind: ActorEventKind::Spawned, // Debug severity
        metadata: HashMap::new(),
    };

    let info_event = ActorEvent {
        timestamp: Utc::now(),
        actor_id: ActorId::new(),
        event_kind: ActorEventKind::Started, // Info severity
        metadata: HashMap::new(),
    };

    let error_event = ActorEvent {
        timestamp: Utc::now(),
        actor_id: ActorId::new(),
        event_kind: ActorEventKind::ErrorOccurred {
            error: "Connection timeout".to_string(),
        }, // Error severity
        metadata: HashMap::new(),
    };

    monitor.record(debug_event).await?; // Filtered out
    monitor.record(info_event).await?; // Filtered out
    monitor.record(error_event).await?; // Recorded

    let snapshot = monitor.snapshot().await?;
    println!(
        "  Events recorded with Warning+ filter: {}",
        snapshot.total_events
    );
    println!("  Debug events (filtered): {}", snapshot.debug_count);
    println!("  Info events (filtered): {}", snapshot.info_count);
    println!("  Error events (recorded): {}\n", snapshot.error_count);

    Ok(())
}

/// Example 3: Recording multiple event types
async fn multiple_event_types_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Example 3: Multiple Event Types ---");

    let config = MonitoringConfig {
        severity_filter: EventSeverity::Trace, // Record all events
        max_history_size: 50,
        ..MonitoringConfig::default()
    };

    let actor_monitor = InMemoryMonitor::new(config.clone());
    let system_monitor = InMemoryMonitor::new(config.clone());
    let broker_monitor = InMemoryMonitor::new(config.clone());
    let mailbox_monitor = InMemoryMonitor::new(config);

    // Record ActorEvent
    actor_monitor
        .record(ActorEvent {
            timestamp: Utc::now(),
            actor_id: ActorId::new(),
            event_kind: ActorEventKind::MessageProcessed {
                message_type: "GreetingMessage".to_string(),
                duration_micros: 1250,
            },
            metadata: HashMap::new(),
        })
        .await?;

    // Record SystemEvent
    system_monitor
        .record(SystemEvent {
            timestamp: Utc::now(),
            event_kind: SystemEventKind::Started,
            metadata: HashMap::from([("version".to_string(), "0.1.0".to_string())]),
        })
        .await?;

    // Record BrokerEvent
    broker_monitor
        .record(BrokerEvent {
            timestamp: Utc::now(),
            event_kind: BrokerEventKind::MessageRouted {
                actor_id: ActorId::new(),
                message_type: "RequestMessage".to_string(),
            },
            metadata: HashMap::new(),
        })
        .await?;

    // Record MailboxEvent
    mailbox_monitor
        .record(MailboxEvent {
            timestamp: Utc::now(),
            actor_id: ActorId::new(),
            event_kind: MailboxEventKind::MessageEnqueued { queue_size: 5 },
            metadata: HashMap::new(),
        })
        .await?;

    println!(
        "  Actor events: {}",
        actor_monitor.snapshot().await?.total_events
    );
    println!(
        "  System events: {}",
        system_monitor.snapshot().await?.total_events
    );
    println!(
        "  Broker events: {}",
        broker_monitor.snapshot().await?.total_events
    );
    println!(
        "  Mailbox events: {}\n",
        mailbox_monitor.snapshot().await?.total_events
    );

    Ok(())
}

/// Example 4: Snapshot and reset operations
async fn snapshot_and_reset_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Example 4: Snapshot and Reset ---");

    let config = MonitoringConfig {
        enabled: true,
        max_history_size: 10,
        severity_filter: EventSeverity::Info,
        snapshot_interval: Duration::from_secs(5),
    };
    let monitor = InMemoryMonitor::new(config);

    // Record some events
    for i in 1..=15 {
        let event = SystemEvent {
            timestamp: Utc::now(),
            event_kind: SystemEventKind::ConfigurationChanged,
            metadata: HashMap::from([
                ("check_number".to_string(), i.to_string()),
                ("type".to_string(), "periodic_check".to_string()),
            ]),
        };
        monitor.record(event).await?;
    }

    // Take snapshot before reset
    let snapshot1 = monitor.snapshot().await?;
    println!("  Before reset:");
    println!("    Total events: {}", snapshot1.total_events);
    println!("    Events in history: {}", snapshot1.recent_events.len());
    println!("    (Note: history limited to max_history_size=10)");

    // Reset monitor
    monitor.reset().await?;

    // Take snapshot after reset
    let snapshot2 = monitor.snapshot().await?;
    println!("  After reset:");
    println!("    Total events: {}", snapshot2.total_events);
    println!("    Events in history: {}\n", snapshot2.recent_events.len());

    Ok(())
}
