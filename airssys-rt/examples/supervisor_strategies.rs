//! Supervision strategy comparison example.
//!
//! This example demonstrates all three supervision strategies and how they
//! behave differently when a child fails:
//! - **OneForOne**: Restart only the failed child
//! - **OneForAll**: Restart all children when one fails
//! - **RestForOne**: Restart the failed child and all children started after it
//!
//! Run with:
//! ```bash
//! cargo run --example supervisor_strategies
//! ```

use airssys_rt::monitoring::{InMemoryMonitor, MonitoringConfig};
use airssys_rt::supervisor::{
    Child, ChildHealth, ChildSpec, OneForAll, OneForOne, RestForOne, RestartPolicy, ShutdownPolicy,
    Supervisor, SupervisorNode,
};
use async_trait::async_trait;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// A worker that tracks restarts and can be triggered to fail.
#[derive(Debug, Clone)]
struct Worker {
    id: String,
    restart_count: Arc<AtomicU32>,
    should_fail: Arc<AtomicBool>,
}

#[derive(Debug)]
struct WorkerError {
    message: String,
}

impl std::fmt::Display for WorkerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for WorkerError {}

#[async_trait]
impl Child for Worker {
    type Error = WorkerError;

    async fn start(&mut self) -> Result<(), Self::Error> {
        let count = self.restart_count.fetch_add(1, Ordering::Relaxed) + 1;
        println!("  [{}] Starting (restart #{})...", self.id, count);

        // Check if this worker should fail on startup
        if self.should_fail.swap(false, Ordering::Relaxed) {
            return Err(WorkerError {
                message: format!("{} intentionally failed", self.id),
            });
        }

        Ok(())
    }

    async fn stop(&mut self, _timeout: Duration) -> Result<(), Self::Error> {
        println!("  [{}] Stopping...", self.id);
        Ok(())
    }

    async fn health_check(&self) -> ChildHealth {
        ChildHealth::Healthy
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Supervision Strategy Comparison ===\n");

    // Example 1: OneForOne Strategy
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Example 1: OneForOne Strategy");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("When a child fails, ONLY that child is restarted.\n");

    demonstrate_one_for_one().await?;

    println!("\n");
    sleep(Duration::from_secs(1)).await;

    // Example 2: OneForAll Strategy
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Example 2: OneForAll Strategy");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("When a child fails, ALL children are restarted.\n");

    demonstrate_one_for_all().await?;

    println!("\n");
    sleep(Duration::from_secs(1)).await;

    // Example 3: RestForOne Strategy
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Example 3: RestForOne Strategy");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("When a child fails, restart that child and all children started AFTER it.\n");

    demonstrate_rest_for_one().await?;

    println!("\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Strategy Selection Guide");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!(
        "
📋 OneForOne:
   Use when: Children are independent
   Example: HTTP request handlers, independent workers
   Behavior: Minimal disruption - only failed child restarts

📋 OneForAll:
   Use when: Children are interdependent (shared state/resources)
   Example: Database pool, cache cluster, service mesh
   Behavior: Clean slate - all children restart together

📋 RestForOne:
   Use when: Children have startup dependencies
   Example: Config → Database → Cache → API Server
   Behavior: Respects dependencies - maintains startup order
"
    );

    println!("=== All Strategy Examples Complete! ===\n");

    Ok(())
}

async fn demonstrate_one_for_one() -> Result<(), Box<dyn std::error::Error>> {
    let monitor = InMemoryMonitor::new(MonitoringConfig::default());
    let mut supervisor = SupervisorNode::<OneForOne, Worker, _>::new(OneForOne, monitor);

    // Create 3 workers
    let workers = vec![
        (
            "worker-A",
            Arc::new(AtomicU32::new(0)),
            Arc::new(AtomicBool::new(false)),
        ),
        (
            "worker-B",
            Arc::new(AtomicU32::new(0)),
            Arc::new(AtomicBool::new(false)),
        ),
        (
            "worker-C",
            Arc::new(AtomicU32::new(0)),
            Arc::new(AtomicBool::new(false)),
        ),
    ];

    println!("Starting 3 workers:");
    let mut child_ids = Vec::new();
    for (id, count, fail_flag) in &workers {
        let id_str = id.to_string();
        let count_clone = Arc::clone(count);
        let fail_clone = Arc::clone(fail_flag);

        let spec = ChildSpec {
            id: (*id).into(),
            factory: move || Worker {
                id: id_str.clone(),
                restart_count: Arc::clone(&count_clone),
                should_fail: Arc::clone(&fail_clone),
            },
            restart_policy: RestartPolicy::Permanent,
            shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
            start_timeout: Duration::from_secs(10),
            shutdown_timeout: Duration::from_secs(10),
        };

        let child_id = supervisor.start_child(spec).await?;
        child_ids.push(child_id);
    }

    println!("\n✅ All workers started\n");
    sleep(Duration::from_millis(300)).await;

    // Trigger worker-B to fail
    println!("Triggering worker-B to fail and restart...");
    workers[1].2.store(true, Ordering::Relaxed);
    let _ = supervisor.restart_child(&child_ids[1]).await;

    sleep(Duration::from_millis(300)).await;

    // Show results
    println!("\n📊 Restart Statistics:");
    for (id, count, _) in &workers {
        let restarts = count.load(Ordering::Relaxed);
        println!(
            "  {} restarted: {} times {}",
            id,
            restarts,
            if *id == "worker-B" {
                "← Failed worker"
            } else {
                ""
            }
        );
    }

    println!("\n💡 Result: Only worker-B was restarted (OneForOne behavior)");

    // Cleanup
    for child_id in child_ids {
        let _ = supervisor.stop_child(&child_id).await;
    }

    Ok(())
}

async fn demonstrate_one_for_all() -> Result<(), Box<dyn std::error::Error>> {
    let monitor = InMemoryMonitor::new(MonitoringConfig::default());
    let mut supervisor = SupervisorNode::<OneForAll, Worker, _>::new(OneForAll, monitor);

    // Create 3 workers
    let workers = vec![
        (
            "worker-A",
            Arc::new(AtomicU32::new(0)),
            Arc::new(AtomicBool::new(false)),
        ),
        (
            "worker-B",
            Arc::new(AtomicU32::new(0)),
            Arc::new(AtomicBool::new(false)),
        ),
        (
            "worker-C",
            Arc::new(AtomicU32::new(0)),
            Arc::new(AtomicBool::new(false)),
        ),
    ];

    println!("Starting 3 workers:");
    let mut child_ids = Vec::new();
    for (id, count, fail_flag) in &workers {
        let id_str = id.to_string();
        let count_clone = Arc::clone(count);
        let fail_clone = Arc::clone(fail_flag);

        let spec = ChildSpec {
            id: (*id).into(),
            factory: move || Worker {
                id: id_str.clone(),
                restart_count: Arc::clone(&count_clone),
                should_fail: Arc::clone(&fail_clone),
            },
            restart_policy: RestartPolicy::Permanent,
            shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
            start_timeout: Duration::from_secs(10),
            shutdown_timeout: Duration::from_secs(10),
        };

        let child_id = supervisor.start_child(spec).await?;
        child_ids.push(child_id);
    }

    println!("\n✅ All workers started\n");
    sleep(Duration::from_millis(300)).await;

    // Trigger worker-B to fail
    println!("Triggering worker-B to fail...");
    workers[1].2.store(true, Ordering::Relaxed);
    let _ = supervisor.restart_child(&child_ids[1]).await;

    sleep(Duration::from_millis(300)).await;

    // Show results
    println!("\n📊 Restart Statistics:");
    for (id, count, _) in &workers {
        let restarts = count.load(Ordering::Relaxed);
        println!(
            "  {} restarted: {} times {}",
            id,
            restarts,
            if *id == "worker-B" {
                "← Failed worker"
            } else {
                "← Also restarted!"
            }
        );
    }

    println!("\n💡 Result: ALL workers were restarted (OneForAll behavior)");

    // Cleanup
    for child_id in child_ids {
        let _ = supervisor.stop_child(&child_id).await;
    }

    Ok(())
}

async fn demonstrate_rest_for_one() -> Result<(), Box<dyn std::error::Error>> {
    let monitor = InMemoryMonitor::new(MonitoringConfig::default());
    let mut supervisor = SupervisorNode::<RestForOne, Worker, _>::new(RestForOne, monitor);

    // Create 3 workers (startup order matters!)
    let workers = vec![
        (
            "worker-A",
            Arc::new(AtomicU32::new(0)),
            Arc::new(AtomicBool::new(false)),
        ),
        (
            "worker-B",
            Arc::new(AtomicU32::new(0)),
            Arc::new(AtomicBool::new(false)),
        ),
        (
            "worker-C",
            Arc::new(AtomicU32::new(0)),
            Arc::new(AtomicBool::new(false)),
        ),
    ];

    println!("Starting 3 workers in order (A → B → C):");
    let mut child_ids = Vec::new();
    for (id, count, fail_flag) in &workers {
        let id_str = id.to_string();
        let count_clone = Arc::clone(count);
        let fail_clone = Arc::clone(fail_flag);

        let spec = ChildSpec {
            id: (*id).into(),
            factory: move || Worker {
                id: id_str.clone(),
                restart_count: Arc::clone(&count_clone),
                should_fail: Arc::clone(&fail_clone),
            },
            restart_policy: RestartPolicy::Permanent,
            shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
            start_timeout: Duration::from_secs(10),
            shutdown_timeout: Duration::from_secs(10),
        };

        let child_id = supervisor.start_child(spec).await?;
        child_ids.push(child_id);
    }

    println!("\n✅ All workers started in order\n");
    sleep(Duration::from_millis(300)).await;

    // Trigger worker-B to fail
    println!("Triggering worker-B to fail (middle of the chain)...");
    workers[1].2.store(true, Ordering::Relaxed);
    let _ = supervisor.restart_child(&child_ids[1]).await;

    sleep(Duration::from_millis(300)).await;

    // Show results
    println!("\n📊 Restart Statistics:");
    for (id, count, _) in &workers {
        let restarts = count.load(Ordering::Relaxed);
        let marker = match *id {
            "worker-A" => "← Started BEFORE B (not restarted)",
            "worker-B" => "← Failed worker (restarted)",
            "worker-C" => "← Started AFTER B (also restarted)",
            _ => "",
        };
        println!("  {} restarted: {} times {}", id, restarts, marker);
    }

    println!("\n💡 Result: Worker-B and worker-C restarted (RestForOne behavior)");
    println!("   Worker-A was not affected (started before failed child)");

    // Cleanup
    for child_id in child_ids {
        let _ = supervisor.stop_child(&child_id).await;
    }

    Ok(())
}
