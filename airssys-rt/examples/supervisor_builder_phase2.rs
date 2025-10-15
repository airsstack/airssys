//! Phase 2: Batch Supervisor Builder Pattern - Comprehensive Examples
//!
//! This example demonstrates the ChildrenBatchBuilder API for spawning multiple
//! children with shared defaults and per-child customization.
//!
//! ## Features Demonstrated
//!
//! 1. **Batch Spawning with Shared Defaults**
//!    - Configure shared restart/shutdown policies for all children
//!    - Efficient batch operations with fail-fast semantics
//!
//! 2. **Per-Child Customization**
//!    - Override specific settings for individual children using BatchChildCustomizer
//!    - Mix default and customized children in same batch
//!
//! 3. **Two Return Types**
//!    - `spawn_all()` → Vec<ChildId> for ordered results
//!    - `spawn_all_map()` → HashMap<String, ChildId> for name-based lookup
//!
//! 4. **Fail-Fast Atomic Semantics**
//!    - All children spawn or none spawn (rollback on error)
//!    - Predictable error handling for batch operations

use airssys_rt::{
    Child, NoopMonitor, OneForOne, RestartPolicy, ShutdownPolicy, SupervisorError, SupervisorNode,
};
use async_trait::async_trait;
use std::time::Duration;
use tokio::time::sleep;

/// Simple worker actor that simulates work
#[derive(Clone)]
struct Worker {
    id: String,
    task_duration_ms: u64,
}

impl Worker {
    fn new(id: &str, task_duration_ms: u64) -> Self {
        Self {
            id: id.to_string(),
            task_duration_ms,
        }
    }
}

#[async_trait]
impl Child for Worker {
    type Error = std::io::Error;

    async fn start(&mut self) -> Result<(), Self::Error> {
        println!(
            "[{}] Worker starting (task duration: {}ms)...",
            self.id, self.task_duration_ms
        );
        sleep(Duration::from_millis(100)).await;
        println!("[{}] Worker started successfully", self.id);
        Ok(())
    }

    async fn stop(&mut self, timeout: Duration) -> Result<(), Self::Error> {
        println!("[{}] Worker stopping (timeout: {:?})...", self.id, timeout);
        sleep(Duration::from_millis(50)).await;
        println!("[{}] Worker stopped", self.id);
        Ok(())
    }
}

/// Example 1: Basic batch spawning with shared defaults
async fn example_1_basic_batch() -> Result<(), SupervisorError> {
    println!("\n=== Example 1: Basic Batch Spawning ===\n");

    let mut supervisor: SupervisorNode<OneForOne, Worker, _> =
        SupervisorNode::new(OneForOne, NoopMonitor::new());

    // Spawn multiple workers with shared restart policy
    let child_ids = supervisor
        .children()
        .restart_policy(RestartPolicy::Permanent)
        .child("worker-1", || Worker::new("worker-1", 1000))
        .child("worker-2", || Worker::new("worker-2", 1500))
        .child("worker-3", || Worker::new("worker-3", 2000))
        .spawn_all()
        .await?;

    println!("\n✓ Spawned {} workers successfully", child_ids.len());
    println!("  Child IDs: {child_ids:?}");

    Ok(())
}

/// Example 2: Batch with per-child customization
async fn example_2_custom_children() -> Result<(), SupervisorError> {
    println!("\n=== Example 2: Per-Child Customization ===\n");

    let mut supervisor: SupervisorNode<OneForOne, Worker, _> =
        SupervisorNode::new(OneForOne, NoopMonitor::new());

    // Mix default and customized children in same batch
    let child_ids = supervisor
        .children()
        .restart_policy(RestartPolicy::Permanent) // Shared default
        .shutdown_policy(ShutdownPolicy::Graceful(Duration::from_secs(5))) // Shared default
        .child("worker-1", || Worker::new("worker-1", 1000)) // Uses shared defaults
        .child_with("worker-2", || Worker::new("worker-2", 1500))
        .restart_policy(RestartPolicy::Temporary) // Override restart policy
        .shutdown_timeout(Duration::from_secs(3)) // Override shutdown timeout
        .done()
        .child("worker-3", || Worker::new("worker-3", 2000)) // Uses shared defaults
        .child_with("worker-4", || Worker::new("worker-4", 2500))
        .shutdown_policy(ShutdownPolicy::Immediate) // Override shutdown policy
        .done()
        .spawn_all()
        .await?;

    println!(
        "\n✓ Spawned {} workers with mixed configurations",
        child_ids.len()
    );
    println!("  worker-1: Permanent restart, 5s graceful shutdown (defaults)");
    println!("  worker-2: Temporary restart, 3s shutdown timeout (custom)");
    println!("  worker-3: Permanent restart, 5s graceful shutdown (defaults)");
    println!("  worker-4: Permanent restart, immediate shutdown (custom)");

    Ok(())
}

/// Example 3: Using spawn_all_map for name-based lookup
async fn example_3_spawn_all_map() -> Result<(), SupervisorError> {
    println!("\n=== Example 3: Name-Based Child Lookup ===\n");

    let mut supervisor: SupervisorNode<OneForOne, Worker, _> =
        SupervisorNode::new(OneForOne, NoopMonitor::new());

    // Spawn batch and get HashMap for name-based lookup
    let children = supervisor
        .children()
        .restart_policy(RestartPolicy::Transient)
        .child("database-worker", || Worker::new("database-worker", 500))
        .child("cache-worker", || Worker::new("cache-worker", 300))
        .child("queue-worker", || Worker::new("queue-worker", 800))
        .spawn_all_map()
        .await?;

    println!(
        "\n✓ Spawned {} workers with name-based lookup",
        children.len()
    );

    // Access children by name
    if let Some(db_id) = children.get("database-worker") {
        println!("  database-worker ID: {db_id}");
    }
    if let Some(cache_id) = children.get("cache-worker") {
        println!("  cache-worker ID: {cache_id}");
    }
    if let Some(queue_id) = children.get("queue-worker") {
        println!("  queue-worker ID: {queue_id}");
    }

    Ok(())
}

/// Example 4: Large batch with dynamic configuration
async fn example_4_large_batch() -> Result<(), SupervisorError> {
    println!("\n=== Example 4: Large Batch with Dynamic Configuration ===\n");

    let mut supervisor: SupervisorNode<OneForOne, Worker, _> =
        SupervisorNode::new(OneForOne, NoopMonitor::new());

    // Build a large batch dynamically
    let worker_count = 10;
    let mut builder = supervisor
        .children()
        .restart_policy(RestartPolicy::Permanent)
        .shutdown_policy(ShutdownPolicy::Graceful(Duration::from_secs(3)));

    // Add workers dynamically
    for i in 1..=worker_count {
        let name = format!("worker-{i}");
        let duration = 500 + (i as u64 * 100);
        builder = builder.child(name.clone(), move || Worker::new(&name, duration));
    }

    let child_ids = builder.spawn_all().await?;

    println!("\n✓ Spawned {worker_count} workers dynamically");
    println!("  Total children: {}", child_ids.len());

    Ok(())
}

/// Example 5: Advanced batch with multiple customizers
async fn example_5_advanced_customization() -> Result<(), SupervisorError> {
    println!("\n=== Example 5: Advanced Customization Patterns ===\n");

    let mut supervisor: SupervisorNode<OneForOne, Worker, _> =
        SupervisorNode::new(OneForOne, NoopMonitor::new());

    // Batch with sophisticated per-child configuration
    let children = supervisor
        .children()
        .restart_policy(RestartPolicy::Permanent)
        .shutdown_policy(ShutdownPolicy::Graceful(Duration::from_secs(5)))
        .start_timeout(Duration::from_secs(30))
        .shutdown_timeout(Duration::from_secs(10))
        // Critical worker with tight timeouts
        .child_with("critical-worker", || Worker::new("critical-worker", 100))
        .restart_policy(RestartPolicy::Permanent)
        .shutdown_policy(ShutdownPolicy::Graceful(Duration::from_secs(2)))
        .start_timeout(Duration::from_secs(10))
        .shutdown_timeout(Duration::from_secs(5))
        .done()
        // Regular workers using defaults
        .child("worker-1", || Worker::new("worker-1", 1000))
        .child("worker-2", || Worker::new("worker-2", 1500))
        // Temporary worker with immediate shutdown
        .child_with("temp-worker", || Worker::new("temp-worker", 500))
        .restart_policy(RestartPolicy::Temporary)
        .shutdown_policy(ShutdownPolicy::Immediate)
        .done()
        // Background worker with relaxed timeouts
        .child_with("background-worker", || {
            Worker::new("background-worker", 3000)
        })
        .restart_policy(RestartPolicy::Transient)
        .start_timeout(Duration::from_secs(60))
        .shutdown_timeout(Duration::from_secs(20))
        .done()
        .spawn_all_map()
        .await?;

    println!(
        "\n✓ Spawned {} workers with sophisticated configurations",
        children.len()
    );
    println!("  critical-worker: Tight timeouts for critical operations");
    println!("  worker-1, worker-2: Standard configuration");
    println!("  temp-worker: Temporary restart, immediate shutdown");
    println!("  background-worker: Relaxed timeouts for long-running tasks");

    Ok(())
}

/// Example 6: Empty batch handling
async fn example_6_empty_batch() -> Result<(), SupervisorError> {
    println!("\n=== Example 6: Empty Batch Handling ===\n");

    let mut supervisor: SupervisorNode<OneForOne, Worker, _> =
        SupervisorNode::new(OneForOne, NoopMonitor::new());

    // Empty batch returns empty Vec
    let child_ids = supervisor
        .children()
        .restart_policy(RestartPolicy::Permanent)
        .spawn_all()
        .await?;

    println!("✓ Empty batch handled correctly");
    println!("  Spawned children: {}", child_ids.len());
    assert!(child_ids.is_empty());

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), SupervisorError> {
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║  Phase 2: Batch Supervisor Builder Pattern - Examples          ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");

    // Run all examples
    example_1_basic_batch().await?;
    example_2_custom_children().await?;
    example_3_spawn_all_map().await?;
    example_4_large_batch().await?;
    example_5_advanced_customization().await?;
    example_6_empty_batch().await?;

    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║  All Examples Completed Successfully                            ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    Ok(())
}
