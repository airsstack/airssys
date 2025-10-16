//! Basic Supervisor Usage - Child Lifecycle Management
//!
//! Demonstrates core supervisor functionality for managing child processes with
//! automatic restart strategies, health monitoring, and graceful shutdown.
//!
//! # What You'll Learn
//!
//! - **Supervisor Creation**: Creating a SupervisorNode with OneForOne strategy
//! - **Child Specifications**: Defining ChildSpec with factory, restart policy, and shutdown policy
//! - **Lifecycle Management**: Starting, stopping, and restarting children
//! - **Restart Policies**: Permanent (always restart), Transient (restart on abnormal exit), Temporary (never restart)
//! - **Health Monitoring**: Implementing health_check() for child health tracking
//! - **Graceful Shutdown**: Using ShutdownPolicy and shutdown timeouts
//!
//! # Key Concepts
//!
//! ## Supervisor Strategies
//!
//! - **OneForOne**: When one child fails, only that child is restarted (used in this example)
//! - **OneForAll**: When one child fails, all children are restarted (see supervisor_strategies.rs)
//! - **RestForOne**: When one child fails, that child and all children started after it are restarted
//!
//! ## Restart Policies
//!
//! - **RestartPolicy::Permanent**: Child is always restarted, regardless of exit reason
//! - **RestartPolicy::Transient**: Child is restarted only if it terminates abnormally (error)
//! - **RestartPolicy::Temporary**: Child is never restarted, even on failure
//!
//! ## Child Lifecycle
//!
//! ```text
//! Created → Starting → Running → Stopping → Stopped
//!             ↑          ↓
//!             └─────────┘ (restart on failure)
//! ```
//!
//! ## ChildSpec Components
//!
//! - **id**: Unique identifier for the child
//! - **factory**: Closure that creates new child instance (called on start and restart)
//! - **restart_policy**: Determines when child should be restarted
//! - **shutdown_policy**: Graceful vs brutal shutdown with timeout
//! - **start_timeout**: Maximum time allowed for start() to complete
//! - **shutdown_timeout**: Maximum time allowed for stop() to complete
//!
//! # Run This Example
//!
//! ```bash
//! cargo run --example supervisor_basic
//! ```
//!
//! # Expected Output
//!
//! ```text
//! === Basic Supervisor Example ===
//!
//! Step 1: Creating supervisor with OneForOne strategy
//! ✅ Supervisor created
//!
//! Step 3: Starting children
//! [worker-1] Starting (start count: 1)
//! ✅ Worker-1 started with ID: worker-1
//! [worker-2] Starting (start count: 1)
//! ✅ Worker-2 started with ID: worker-2
//!
//! Step 5: Testing automatic restart (Permanent policy)
//! [worker-1] Stopping (stop count: 1)
//! [worker-1] Starting (start count: 2)  ← Automatic restart
//! ✅ Worker-1 restart initiated
//!
//! === Worker Statistics ===
//! Worker-1: started 2 times, stopped 2 times
//! Worker-2: started 1 times, stopped 1 times
//! ```
//!
//! # See Also
//!
//! - [`supervisor_strategies.rs`] - OneForOne, OneForAll, RestForOne strategy comparison
//! - [`supervisor_builder_phase1.rs`] - Fluent builder API for supervisors
//! - [`monitoring_supervisor.rs`] - Monitoring integration with supervisors
//! - [User Guide: Supervisor Patterns](../docs/src/guides/supervisor-patterns.md)

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

// =============================================================================
// Child Implementation - Simple Supervised Worker
// =============================================================================

/// A simple worker that can be supervised.
///
/// Demonstrates the Child trait implementation required for supervision:
/// - `start()`: Initialize the child (called on start and restart)
/// - `stop()`: Clean shutdown with timeout enforcement
/// - `health_check()`: Report child health status for monitoring
///
/// Uses Arc<Atomic*> for counters to track lifecycle calls across restarts
/// (factory creates new instances, but counters persist).
#[derive(Debug)]
struct SimpleWorker {
    id: String,
    start_count: Arc<AtomicU32>,  // Shared counter across all instances
    stop_count: Arc<AtomicU32>,   // Shared counter across all instances
    should_fail: Arc<AtomicBool>, // Shared failure flag for testing
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

    /// Initialize child on start or restart.
    ///
    /// CRITICAL: This is called for BOTH initial start AND every restart.
    /// Initialize resources, connections, and state here.
    async fn start(&mut self) -> Result<(), Self::Error> {
        let count = self.start_count.fetch_add(1, Ordering::Relaxed) + 1;
        println!("[{}] Starting (start count: {})", self.id, count);

        // Simulate startup failure for testing restart behavior
        if self.should_fail.load(Ordering::Relaxed) {
            self.should_fail.store(false, Ordering::Relaxed); // Only fail once
            return Err(WorkerError {
                message: format!("{} startup failed", self.id),
            });
        }

        Ok(())
    }

    /// Gracefully stop child with timeout enforcement.
    ///
    /// Clean up resources, close connections, flush buffers.
    /// Supervisor enforces the timeout from ShutdownPolicy.
    async fn stop(&mut self, _timeout: Duration) -> Result<(), Self::Error> {
        let count = self.stop_count.fetch_add(1, Ordering::Relaxed) + 1;
        println!("[{}] Stopping (stop count: {})", self.id, count);
        Ok(())
    }

    /// Report child health status for monitoring.
    ///
    /// Called periodically by monitoring system. Return:
    /// - ChildHealth::Healthy: Child is functioning normally
    /// - ChildHealth::Unhealthy(reason): Child needs attention or restart
    async fn health_check(&self) -> ChildHealth {
        ChildHealth::Healthy
    }
}

// =============================================================================
// Main Example - Supervisor Lifecycle Demonstration
// =============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Basic Supervisor Example ===\n");

    // ==========================================================================
    // Step 1: Create supervisor with OneForOne strategy
    // ==========================================================================
    // OneForOne: When one child fails, only that child is restarted
    // (Other children continue running unaffected)
    println!("Step 1: Creating supervisor with OneForOne strategy");
    let monitor = InMemoryMonitor::new(MonitoringConfig::default());
    let mut supervisor = SupervisorNode::<OneForOne, SimpleWorker, _>::new(OneForOne, monitor);
    println!("✅ Supervisor created\n");

    // ==========================================================================
    // Step 2: Define child specifications
    // ==========================================================================
    // ChildSpec defines:
    // - How to create child instances (factory closure)
    // - When to restart (RestartPolicy)
    // - How to shutdown (ShutdownPolicy)
    // - Timeout constraints
    println!("Step 2: Defining child specifications");

    // Worker 1: Permanent restart policy (always restart on failure)
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

    // Worker 2: Transient restart policy (only restart on abnormal exit)
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

    // ==========================================================================
    // Step 3: Start children under supervision
    // ==========================================================================
    // Supervisor calls spec.factory() to create instances, then child.start()
    println!("Step 3: Starting children");
    let child1_id = supervisor.start_child(spec1).await?;
    println!("✅ Worker-1 started with ID: {child1_id}");

    let child2_id = supervisor.start_child(spec2).await?;
    println!("✅ Worker-2 started with ID: {child2_id}\n");

    // ==========================================================================
    // Step 4: Inspect child states
    // ==========================================================================
    println!("Step 4: Checking child states");
    let child_count = supervisor.child_count();
    println!("Active children: {child_count}");
    for child_id in supervisor.child_ids() {
        if let Some(handle) = supervisor.get_child(child_id) {
            println!("  - {}: {:?}", child_id, handle.state());
        }
    }
    println!();

    // ==========================================================================
    // Step 5: Test automatic restart (Permanent policy)
    // ==========================================================================
    // Worker-1 has RestartPolicy::Permanent, so it will restart on failure
    println!("Step 5: Testing automatic restart (Permanent policy)");
    println!("Simulating worker-1 failure...");
    worker1_should_fail.store(true, Ordering::Relaxed);

    match supervisor.restart_child(&child1_id).await {
        Ok(_) => {
            println!("✅ Worker-1 restart initiated");
            // The restart will fail once, then succeed on retry due to Permanent policy
        }
        Err(e) => println!("⚠️  Worker-1 restart error: {e}"),
    }

    sleep(Duration::from_millis(500)).await;
    println!();

    // ==========================================================================
    // Step 6: Graceful child shutdown
    // ==========================================================================
    // Gracefully stop worker-2 (calls child.stop() with timeout enforcement)
    println!("Step 6: Gracefully stopping worker-2");
    supervisor.stop_child(&child2_id).await?;
    println!("✅ Worker-2 stopped\n");

    // ==========================================================================
    // Step 7: Final state inspection
    // ==========================================================================
    println!("Step 7: Final state check");
    let child_count = supervisor.child_count();
    println!("Active children: {child_count}");
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
