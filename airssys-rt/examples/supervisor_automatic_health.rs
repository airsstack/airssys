//! Example demonstrating automatic background health monitoring for supervisors.
//!
//! This example shows how to use the builder pattern to enable automatic
//! background health checking. When you call `with_automatic_health_monitoring()`,
//! a background task is automatically spawned to periodically check all children.
//!
//! Run with:
//! ```bash
//! cargo run --example supervisor_automatic_health
//! ```

use airssys_rt::monitoring::{InMemoryMonitor, MonitoringConfig};
use airssys_rt::supervisor::{
    Child, ChildSpec, OneForOne, RestartPolicy, ShutdownPolicy, Supervisor, SupervisorNode,
};
use async_trait::async_trait;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// Example worker that tracks health check attempts
#[derive(Debug)]
struct HealthyWorker {
    id: String,
    health_check_count: Arc<AtomicU32>,
}

#[derive(Debug)]
struct WorkerError;

impl std::fmt::Display for WorkerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WorkerError")
    }
}

impl std::error::Error for WorkerError {}

#[async_trait]
impl Child for HealthyWorker {
    type Error = WorkerError;

    async fn start(&mut self) -> Result<(), Self::Error> {
        println!("[{}] Starting worker", self.id);
        Ok(())
    }

    async fn stop(&mut self, _timeout: Duration) -> Result<(), Self::Error> {
        println!("[{}] Stopping worker", self.id);
        Ok(())
    }

    async fn health_check(&self) -> airssys_rt::supervisor::ChildHealth {
        let count = self.health_check_count.fetch_add(1, Ordering::Relaxed) + 1;
        println!("[{}] Health check #{} - Healthy!", self.id, count);
        airssys_rt::supervisor::ChildHealth::Healthy
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Automatic Background Health Monitoring Example ===\n");

    // Create monitoring
    let monitor = InMemoryMonitor::new(MonitoringConfig::default());

    // Create supervisor with AUTOMATIC background health monitoring (builder pattern)
    println!("Creating supervisor with automatic health monitoring...");
    let supervisor = SupervisorNode::<OneForOne, HealthyWorker, _>::new(OneForOne, monitor)
        .with_automatic_health_monitoring(
            Duration::from_secs(2), // Check every 2 seconds
            Duration::from_secs(1), // 1 second timeout per check
            3,                      // Restart after 3 consecutive failures
        );

    println!("✅ Supervisor created with automatic background monitoring!\n");

    // Create shared health check counters
    let counter1 = Arc::new(AtomicU32::new(0));
    let counter2 = Arc::new(AtomicU32::new(0));

    // Start first worker
    println!("Starting worker-1...");
    let spec1 = ChildSpec {
        id: "worker-1".into(),
        factory: {
            let counter = Arc::clone(&counter1);
            move || HealthyWorker {
                id: "worker-1".to_string(),
                health_check_count: Arc::clone(&counter),
            }
        },
        restart_policy: RestartPolicy::Permanent,
        shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
        start_timeout: Duration::from_secs(10),
        shutdown_timeout: Duration::from_secs(10),
    };

    let child1_id = supervisor
        .supervisor
        .lock()
        .await
        .start_child(spec1)
        .await?;
    println!("✅ Worker-1 started with ID: {child1_id}\n");

    // Wait a bit to see some health checks
    println!("Waiting 5 seconds to observe automatic health checks...\n");
    sleep(Duration::from_secs(5)).await;

    // Start second worker
    println!("\nStarting worker-2...");
    let spec2 = ChildSpec {
        id: "worker-2".into(),
        factory: {
            let counter = Arc::clone(&counter2);
            move || HealthyWorker {
                id: "worker-2".to_string(),
                health_check_count: Arc::clone(&counter),
            }
        },
        restart_policy: RestartPolicy::Permanent,
        shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
        start_timeout: Duration::from_secs(10),
        shutdown_timeout: Duration::from_secs(10),
    };

    let child2_id = supervisor
        .supervisor
        .lock()
        .await
        .start_child(spec2)
        .await?;
    println!("✅ Worker-2 started with ID: {child2_id}\n");

    // Wait to see health checks for both workers
    println!("Waiting 7 more seconds to observe health checks for both workers...\n");
    sleep(Duration::from_secs(7)).await;

    // Show statistics
    let count1 = counter1.load(Ordering::Relaxed);
    let count2 = counter2.load(Ordering::Relaxed);

    println!("\n=== Health Check Statistics ===");
    println!("Worker-1: {count1} health checks performed");
    println!("Worker-2: {count2} health checks performed");

    println!("\n✅ Background health monitoring was AUTOMATIC!");
    println!("✅ No manual health check loop required!");
    println!("✅ Safe by default - children are continuously monitored!");

    // Cleanup
    println!("\n=== Cleanup ===");
    {
        let mut sup = supervisor.supervisor.lock().await;
        println!("Stopping worker-1...");
        sup.stop_child(&child1_id).await?;
        println!("Stopping worker-2...");
        sup.stop_child(&child2_id).await?;
    }

    println!("\n✅ Example completed successfully!");

    Ok(())
}
