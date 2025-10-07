//! Basic supervisor usage example.
//!
//! This example demonstrates the core supervisor functionality:
//! - Creating a supervisor with a supervision strategy
//! - Starting and stopping children
//! - Basic error handling and restart behavior
//! - Child lifecycle management
//!
//! Run with:
//! ```bash
//! cargo run --example supervisor_basic
//! ```

use airssys_rt::monitoring::{InMemoryMonitor, MonitoringConfig};
use airssys_rt::supervisor::{
    Child, ChildHealth, ChildSpec, OneForOne, RestartPolicy, ShutdownPolicy, Supervisor,
    SupervisorNode,
};
use async_trait::async_trait;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// A simple worker that can be supervised.
#[derive(Debug)]
struct SimpleWorker {
    id: String,
    start_count: Arc<AtomicU32>,
    stop_count: Arc<AtomicU32>,
    should_fail: Arc<AtomicBool>,
}

#[derive(Debug)]
struct WorkerError {
    message: String,
}

impl std::fmt::Display for WorkerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WorkerError: {}", self.message)
    }
}

impl std::error::Error for WorkerError {}

#[async_trait]
impl Child for SimpleWorker {
    type Error = WorkerError;

    async fn start(&mut self) -> Result<(), Self::Error> {
        let count = self.start_count.fetch_add(1, Ordering::Relaxed) + 1;
        println!("[{}] Starting (start count: {})", self.id, count);

        // Simulate startup failure if flag is set
        if self.should_fail.load(Ordering::Relaxed) {
            self.should_fail.store(false, Ordering::Relaxed); // Only fail once
            return Err(WorkerError {
                message: format!("{} startup failed", self.id),
            });
        }

        Ok(())
    }

    async fn stop(&mut self, _timeout: Duration) -> Result<(), Self::Error> {
        let count = self.stop_count.fetch_add(1, Ordering::Relaxed) + 1;
        println!("[{}] Stopping (stop count: {})", self.id, count);
        Ok(())
    }

    async fn health_check(&self) -> ChildHealth {
        ChildHealth::Healthy
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Basic Supervisor Example ===\n");

    // Step 1: Create a supervisor with OneForOne strategy
    println!("Step 1: Creating supervisor with OneForOne strategy");
    let monitor = InMemoryMonitor::new(MonitoringConfig::default());
    let mut supervisor = SupervisorNode::<OneForOne, SimpleWorker, _>::new(OneForOne, monitor);
    println!("✅ Supervisor created\n");

    // Step 2: Define child specifications
    println!("Step 2: Defining child specifications");

    let worker1_start_count = Arc::new(AtomicU32::new(0));
    let worker1_stop_count = Arc::new(AtomicU32::new(0));
    let worker1_should_fail = Arc::new(AtomicBool::new(false));

    let spec1 = ChildSpec {
        id: "worker-1".into(),
        factory: {
            let start_count = Arc::clone(&worker1_start_count);
            let stop_count = Arc::clone(&worker1_stop_count);
            let should_fail = Arc::clone(&worker1_should_fail);
            move || SimpleWorker {
                id: "worker-1".to_string(),
                start_count: Arc::clone(&start_count),
                stop_count: Arc::clone(&stop_count),
                should_fail: Arc::clone(&should_fail),
            }
        },
        restart_policy: RestartPolicy::Permanent, // Always restart on failure
        shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
        start_timeout: Duration::from_secs(10),
        shutdown_timeout: Duration::from_secs(10),
    };

    let worker2_start_count = Arc::new(AtomicU32::new(0));
    let worker2_stop_count = Arc::new(AtomicU32::new(0));
    let worker2_should_fail = Arc::new(AtomicBool::new(false));

    let spec2 = ChildSpec {
        id: "worker-2".into(),
        factory: {
            let start_count = Arc::clone(&worker2_start_count);
            let stop_count = Arc::clone(&worker2_stop_count);
            let should_fail = Arc::clone(&worker2_should_fail);
            move || SimpleWorker {
                id: "worker-2".to_string(),
                start_count: Arc::clone(&start_count),
                stop_count: Arc::clone(&stop_count),
                should_fail: Arc::clone(&should_fail),
            }
        },
        restart_policy: RestartPolicy::Transient, // Only restart on abnormal exit
        shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
        start_timeout: Duration::from_secs(10),
        shutdown_timeout: Duration::from_secs(10),
    };

    println!("✅ Child specifications defined\n");

    // Step 3: Start children
    println!("Step 3: Starting children");
    let child1_id = supervisor.start_child(spec1).await?;
    println!("✅ Worker-1 started with ID: {child1_id}");

    let child2_id = supervisor.start_child(spec2).await?;
    println!("✅ Worker-2 started with ID: {child2_id}\n");

    // Step 4: Check child states
    println!("Step 4: Checking child states");
    let child_count = supervisor.child_count();
    println!("Active children: {}", child_count);
    for child_id in supervisor.child_ids() {
        if let Some(handle) = supervisor.get_child(child_id) {
            println!("  - {}: {:?}", child_id, handle.state());
        }
    }
    println!();

    // Step 5: Simulate child restart (worker-1 with Permanent policy)
    println!("Step 5: Testing automatic restart (Permanent policy)");
    println!("Simulating worker-1 failure...");
    worker1_should_fail.store(true, Ordering::Relaxed);

    match supervisor.restart_child(&child1_id).await {
        Ok(_) => {
            println!("✅ Worker-1 restart initiated");
            // The restart will fail once, then succeed on retry due to Permanent policy
        }
        Err(e) => println!("⚠️  Worker-1 restart error: {}", e),
    }

    sleep(Duration::from_millis(500)).await;
    println!();

    // Step 6: Stop a child
    println!("Step 6: Gracefully stopping worker-2");
    supervisor.stop_child(&child2_id).await?;
    println!("✅ Worker-2 stopped\n");

    // Step 7: Check final states
    println!("Step 7: Final state check");
    let child_count = supervisor.child_count();
    println!("Active children: {}", child_count);
    for child_id in supervisor.child_ids() {
        if let Some(handle) = supervisor.get_child(child_id) {
            println!("  - {}: {:?}", child_id, handle.state());
        }
    }
    println!();

    // Step 8: Display statistics
    println!("=== Worker Statistics ===");
    println!(
        "Worker-1: started {} times, stopped {} times",
        worker1_start_count.load(Ordering::Relaxed),
        worker1_stop_count.load(Ordering::Relaxed)
    );
    println!(
        "Worker-2: started {} times, stopped {} times",
        worker2_start_count.load(Ordering::Relaxed),
        worker2_stop_count.load(Ordering::Relaxed)
    );
    println!();

    // Step 9: Cleanup - stop all children
    println!("Step 9: Stopping all remaining children");
    supervisor.stop_child(&child1_id).await?;
    println!("✅ All children stopped\n");

    println!("=== Basic Supervisor Example Complete! ===");
    println!("\nKey Learnings:");
    println!("  • Supervisors manage child lifecycle (start, stop, restart)");
    println!("  • RestartPolicy controls restart behavior (Permanent, Transient, Temporary)");
    println!("  • OneForOne strategy restarts only the failed child");
    println!("  • Child specifications define how children are created and managed");

    Ok(())
}
