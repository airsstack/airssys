//! Supervisor Monitoring Integration Preview
//!
//! Demonstrates how RT-TASK-007 (Supervisor Framework) will integrate
//! with the monitoring infrastructure. This is a conceptual preview showing
//! how supervision events will be recorded and monitored.
//!
//! **Note:** Full supervisor functionality requires RT-TASK-007 completion.
//! This example shows the monitoring interface that supervisors will use.
//!
//! Run with: cargo run --example monitoring_supervisor

use std::collections::HashMap;
use std::time::Duration;

use chrono::Utc;

use airssys_rt::monitoring::{
    EventSeverity, InMemoryMonitor, Monitor, MonitoringConfig, SupervisionEvent,
    SupervisionEventKind,
};
use airssys_rt::util::ActorId;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== AirsSys RT - Supervisor Monitoring Integration Preview ===\n");
    println!("NOTE: This is a conceptual preview. Full functionality requires RT-TASK-007.\n");

    // Example 1: Basic supervisor monitoring
    basic_supervisor_monitoring().await?;

    // Example 2: Restart strategy monitoring
    restart_strategy_monitoring().await?;

    // Example 3: Supervision tree monitoring
    supervision_tree_monitoring().await?;

    // Example 4: Failure analysis from snapshots
    failure_analysis_example().await?;

    println!("\n=== All supervisor monitoring examples completed! ===");
    Ok(())
}

/// Example 1: Basic supervisor event monitoring
async fn basic_supervisor_monitoring() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Example 1: Basic Supervisor Monitoring ---");

    let config = MonitoringConfig {
        severity_filter: EventSeverity::Debug, // Capture all supervisor events
        ..MonitoringConfig::default()
    };
    let monitor = InMemoryMonitor::new(config);

    let supervisor_id = ActorId::new().to_string();
    let child_id = ActorId::new().to_string();

    // Supervisor starts - using ChildStarted for the supervisor itself
    monitor
        .record(SupervisionEvent {
            timestamp: Utc::now(),
            supervisor_id: supervisor_id.clone(),
            child_id: None,
            event_kind: SupervisionEventKind::ChildStarted,
            metadata: HashMap::from([("strategy".to_string(), "one_for_one".to_string())]),
        })
        .await?;

    // Child actor spawned
    monitor
        .record(SupervisionEvent {
            timestamp: Utc::now(),
            supervisor_id: supervisor_id.clone(),
            child_id: Some(child_id.clone()),
            event_kind: SupervisionEventKind::ChildStarted,
            metadata: HashMap::from([("child_name".to_string(), "worker-1".to_string())]),
        })
        .await?;

    // Child encounters error
    monitor
        .record(SupervisionEvent {
            timestamp: Utc::now(),
            supervisor_id: supervisor_id.clone(),
            child_id: Some(child_id.clone()),
            event_kind: SupervisionEventKind::ChildFailed {
                error: "Network connection lost".to_string(),
                restart_count: 0,
            },
            metadata: HashMap::new(),
        })
        .await?;

    let snapshot = monitor.snapshot().await?;
    println!("  Supervision events recorded: {}", snapshot.total_events);
    println!("  Debug-level events: {}", snapshot.debug_count);
    println!("  Info-level events: {}", snapshot.info_count);
    println!("  Error-level events: {}\n", snapshot.error_count);

    Ok(())
}

/// Example 2: Monitoring restart strategies
async fn restart_strategy_monitoring() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Example 2: Restart Strategy Monitoring ---");

    let config = MonitoringConfig::default();
    let monitor = InMemoryMonitor::new(config);

    let supervisor_id = ActorId::new().to_string();
    let child_id = ActorId::new().to_string();

    // Simulate multiple restart attempts
    for attempt in 1..=3 {
        // Child fails
        monitor
            .record(SupervisionEvent {
                timestamp: Utc::now(),
                supervisor_id: supervisor_id.clone(),
                child_id: Some(child_id.clone()),
                event_kind: SupervisionEventKind::ChildFailed {
                    error: format!("Attempt {attempt}: Service unavailable"),
                    restart_count: attempt - 1,
                },
                metadata: HashMap::from([("attempt".to_string(), attempt.to_string())]),
            })
            .await?;

        // Strategy applied
        monitor
            .record(SupervisionEvent {
                timestamp: Utc::now(),
                supervisor_id: supervisor_id.clone(),
                child_id: Some(child_id.clone()),
                event_kind: SupervisionEventKind::StrategyApplied {
                    strategy: "one_for_one".to_string(),
                    affected_count: 1,
                },
                metadata: HashMap::new(),
            })
            .await?;

        // Child restarted
        monitor
            .record(SupervisionEvent {
                timestamp: Utc::now(),
                supervisor_id: supervisor_id.clone(),
                child_id: Some(child_id.clone()),
                event_kind: SupervisionEventKind::ChildRestarted {
                    restart_count: attempt,
                },
                metadata: HashMap::from([("restart_count".to_string(), attempt.to_string())]),
            })
            .await?;
    }

    let snapshot = monitor.snapshot().await?;
    println!("  Total supervision events: {}", snapshot.total_events);
    println!(
        "  Recent events tracked: {}\n",
        snapshot.recent_events.len()
    );

    Ok(())
}

/// Example 3: Supervision tree monitoring
async fn supervision_tree_monitoring() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Example 3: Supervision Tree Monitoring ---");

    let config = MonitoringConfig::default();
    let monitor = InMemoryMonitor::new(config);

    let root_supervisor = ActorId::new().to_string();
    let mid_supervisor = ActorId::new().to_string();
    let worker1 = ActorId::new().to_string();
    let worker2 = ActorId::new().to_string();

    // Root supervisor starts (using ChildStarted for root)
    monitor
        .record(SupervisionEvent {
            timestamp: Utc::now(),
            supervisor_id: root_supervisor.clone(),
            child_id: None,
            event_kind: SupervisionEventKind::ChildStarted,
            metadata: HashMap::from([
                ("level".to_string(), "root".to_string()),
                ("strategy".to_string(), "one_for_all".to_string()),
            ]),
        })
        .await?;

    // Middle-level supervisor started
    monitor
        .record(SupervisionEvent {
            timestamp: Utc::now(),
            supervisor_id: root_supervisor.clone(),
            child_id: Some(mid_supervisor.clone()),
            event_kind: SupervisionEventKind::ChildStarted,
            metadata: HashMap::from([("type".to_string(), "supervisor".to_string())]),
        })
        .await?;

    // Workers under mid-level supervisor
    for (idx, worker_id) in [worker1, worker2].iter().enumerate() {
        monitor
            .record(SupervisionEvent {
                timestamp: Utc::now(),
                supervisor_id: mid_supervisor.clone(),
                child_id: Some(worker_id.clone()),
                event_kind: SupervisionEventKind::ChildStarted,
                metadata: HashMap::from([
                    ("type".to_string(), "worker".to_string()),
                    ("worker_number".to_string(), (idx + 1).to_string()),
                ]),
            })
            .await?;
    }

    let snapshot = monitor.snapshot().await?;
    println!("  Supervision tree events: {}", snapshot.total_events);
    println!("  This demonstrates hierarchical supervisor monitoring\n");

    Ok(())
}

/// Example 4: Failure analysis from monitoring snapshots
async fn failure_analysis_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Example 4: Failure Analysis from Snapshots ---");

    let config = MonitoringConfig {
        max_history_size: 50,
        ..MonitoringConfig::default()
    };
    let monitor = InMemoryMonitor::new(config);

    let supervisor_id = ActorId::new().to_string();
    let problematic_child = ActorId::new().to_string();
    let stable_child = ActorId::new().to_string();

    // Simulate a problematic child with multiple failures
    for i in 1..=5 {
        monitor
            .record(SupervisionEvent {
                timestamp: Utc::now(),
                supervisor_id: supervisor_id.clone(),
                child_id: Some(problematic_child.clone()),
                event_kind: SupervisionEventKind::ChildFailed {
                    error: format!("Crash #{i}"),
                    restart_count: i - 1,
                },
                metadata: HashMap::from([("child_type".to_string(), "problematic".to_string())]),
            })
            .await?;
    }

    // Stable child runs fine
    monitor
        .record(SupervisionEvent {
            timestamp: Utc::now(),
            supervisor_id: supervisor_id.clone(),
            child_id: Some(stable_child.clone()),
            event_kind: SupervisionEventKind::ChildStarted,
            metadata: HashMap::from([("child_type".to_string(), "stable".to_string())]),
        })
        .await?;

    // Max retries exceeded (using RestartLimitExceeded)
    monitor
        .record(SupervisionEvent {
            timestamp: Utc::now(),
            supervisor_id: supervisor_id.clone(),
            child_id: Some(problematic_child.clone()),
            event_kind: SupervisionEventKind::RestartLimitExceeded {
                restart_count: 5,
                window: Duration::from_secs(60),
            },
            metadata: HashMap::from([(
                "reason".to_string(),
                "Persistent failure after 5 attempts".to_string(),
            )]),
        })
        .await?;

    let snapshot = monitor.snapshot().await?;
    println!("  Total events: {}", snapshot.total_events);
    println!("  Error events: {}", snapshot.error_count);
    println!("  Critical events: {}", snapshot.critical_count);
    println!("\n  Analysis: Monitoring data enables:");
    println!("    - Identifying problematic actors");
    println!("    - Tracking restart patterns");
    println!("    - Detecting cascading failures");
    println!("    - Optimizing supervision strategies\n");

    Ok(())
}
