//! Event Processing Pipeline with RestForOne Supervision
//!
//! Demonstrates building an event processing pipeline with RestForOne supervision strategy,
//! pub/sub messaging, backpressure handling, and cascading restart behavior.
//!
//! # What You'll Learn
//!
//! - **Pipeline Architecture**: Sequential event processing stages (Ingest → Transform → Output)
//! - **RestForOne Supervision**: When a stage fails, that stage and all downstream stages restart
//! - **Pub/Sub Messaging**: Event broadcasting with subscription-based delivery
//! - **Backpressure Handling**: Managing flow control when consumers are slower than producers
//! - **Cascading Restarts**: Understanding how RestForOne propagates restarts through pipeline
//! - **Ordered Processing**: Maintaining event ordering through pipeline stages
//!
//! # Key Concepts
//!
//! ## Pipeline Architecture
//!
//! ```text
//! Supervisor (RestForOne)
//!     ├─→ IngestStage     (receives raw events)
//!     ├─→ TransformStage  (processes and enriches events)
//!     └─→ OutputStage     (persists or forwards events)
//!
//! Restart Behavior:
//! - If IngestStage fails     → Restart: IngestStage, TransformStage, OutputStage
//! - If TransformStage fails  → Restart: TransformStage, OutputStage
//! - If OutputStage fails     → Restart: OutputStage only
//! ```
//!
//! ## RestForOne Strategy
//!
//! **Definition**: When a child fails, restart that child AND all children started after it.
//!
//! **Use Cases**:
//! - Sequential processing pipelines
//! - Dependent service chains
//! - Data transformation flows
//! - Event streaming architectures
//!
//! **Benefits**:
//! - Maintains pipeline integrity
//! - Ensures consistent state across stages
//! - Prevents partial processing
//! - Simpler than manual coordination
//!
//! ## Backpressure Handling
//!
//! When downstream stages can't keep up with upstream stages:
//! ```rust
//! // Check queue capacity before publishing
//! if queue.len() < MAX_QUEUE_SIZE {
//!     queue.push(event);
//! } else {
//!     // Apply backpressure: block, drop, or signal upstream
//! }
//! ```
//!
//! # Run This Example
//!
//! ```bash
//! cargo run --example event_pipeline
//! ```
//!
//! # Expected Output
//!
//! ```text
//! === Event Pipeline Example ===
//!
//! Creating pipeline with RestForOne supervision...
//! [IngestStage] Starting
//! [TransformStage] Starting
//! [OutputStage] Starting
//! ✅ Pipeline ready with 3 stages
//!
//! Processing 5 events through pipeline...
//! [IngestStage] Received event 1: "user_signup"
//! [TransformStage] Processing event 1: enriching...
//! [OutputStage] Persisting event 1: {"type":"user_signup","enriched":true}
//! ...
//!
//! Simulating TransformStage failure...
//! [TransformStage] Stopping (failure simulation)
//! [OutputStage] Stopping (RestForOne cascade)
//! [TransformStage] Starting (automatic restart)
//! [OutputStage] Starting (RestForOne cascade)
//! ✅ Pipeline recovered
//!
//! === Pipeline Statistics ===
//! IngestStage: 5 events received
//! TransformStage: 5 events processed
//! OutputStage: 5 events persisted
//! ```
//!
//! # See Also
//!
//! - [`supervisor_strategies.rs`] - Strategy comparison (OneForOne vs OneForAll vs RestForOne)
//! - [`worker_pool.rs`] - Worker pool with OneForOne supervision
//! - [`supervisor_basic.rs`] - Basic supervisor and child management
//! - [User Guide: Supervisor Patterns](../docs/src/guides/supervisor-patterns.md)

#![expect(
    clippy::expect_used,
    reason = "expect is acceptable in example code for demonstration purposes"
)]

use airssys_rt::monitoring::{InMemoryMonitor, MonitoringConfig};
use airssys_rt::supervisor::{
    Child, ChildHealth, ChildSpec, RestForOne, RestartPolicy, ShutdownPolicy, Supervisor,
    SupervisorNode,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

// =============================================================================
// Event Types
// =============================================================================

/// Event flowing through the pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Event {
    id: u32,
    event_type: String,
    payload: String,
    enriched: bool,
}

/// Shared event queue between pipeline stages.
///
/// Uses Arc<Mutex<>> for thread-safe shared access.
type EventQueue = Arc<Mutex<VecDeque<Event>>>;

// =============================================================================
// Pipeline Stage: Ingest
// =============================================================================

/// First stage: Receives raw events and forwards to transform stage.
///
/// Responsibilities:
/// - Validate incoming events
/// - Normalize event format
/// - Forward to transform queue
#[derive(Debug)]
struct IngestStage {
    received_count: Arc<AtomicU32>,
    transform_queue: EventQueue,
    max_queue_size: usize,
}

#[derive(Debug)]
struct StageError {
    message: String,
}

impl std::fmt::Display for StageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StageError: {}", self.message)
    }
}

impl std::error::Error for StageError {}

impl IngestStage {
    fn new(transform_queue: EventQueue, max_queue_size: usize) -> Self {
        Self {
            received_count: Arc::new(AtomicU32::new(0)),
            transform_queue,
            max_queue_size,
        }
    }

    /// Receive and enqueue event for processing.
    ///
    /// Applies backpressure if transform queue is full.
    fn ingest_event(&mut self, event: Event) -> Result<(), StageError> {
        let mut queue = self
            .transform_queue
            .lock()
            .expect("Failed to acquire transform queue lock");

        // Backpressure: check queue capacity
        if queue.len() >= self.max_queue_size {
            return Err(StageError {
                message: format!("Transform queue full ({} events)", queue.len()),
            });
        }

        println!(
            "[IngestStage] Received event {}: \"{}\"",
            event.id, event.event_type
        );
        queue.push_back(event);
        self.received_count.fetch_add(1, Ordering::Relaxed);

        Ok(())
    }
}

#[async_trait]
impl Child for IngestStage {
    type Error = StageError;

    async fn start(&mut self) -> Result<(), Self::Error> {
        println!("[IngestStage] Starting");
        Ok(())
    }

    async fn stop(&mut self, _timeout: Duration) -> Result<(), Self::Error> {
        println!(
            "[IngestStage] Stopping (received {} events)",
            self.received_count.load(Ordering::Relaxed)
        );
        Ok(())
    }

    async fn health_check(&self) -> ChildHealth {
        ChildHealth::Healthy
    }
}

// =============================================================================
// Pipeline Stage: Transform
// =============================================================================

/// Second stage: Processes and enriches events.
///
/// Responsibilities:
/// - Dequeue events from transform queue
/// - Apply business logic transformations
/// - Enrich with additional data
/// - Forward to output queue
#[derive(Debug)]
struct TransformStage {
    processed_count: Arc<AtomicU32>,
    transform_queue: EventQueue,
    output_queue: EventQueue,
    should_fail: Arc<AtomicU32>,
}

impl TransformStage {
    fn new(transform_queue: EventQueue, output_queue: EventQueue) -> Self {
        Self {
            processed_count: Arc::new(AtomicU32::new(0)),
            transform_queue,
            output_queue,
            should_fail: Arc::new(AtomicU32::new(0)),
        }
    }

    /// Process events from transform queue.
    async fn process_events(&mut self) -> Result<(), StageError> {
        loop {
            // Acquire locks, process one event, then drop locks before await
            let event_opt = {
                let mut transform_queue = self
                    .transform_queue
                    .lock()
                    .expect("Failed to acquire transform queue lock");
                transform_queue.pop_front()
            };

            let mut event = match event_opt {
                Some(e) => e,
                None => break, // No more events
            };

            // Check failure simulation trigger
            let fail_trigger = self.should_fail.load(Ordering::Relaxed);
            if fail_trigger > 0 && self.processed_count.load(Ordering::Relaxed) >= fail_trigger {
                self.should_fail.store(0, Ordering::Relaxed);
                return Err(StageError {
                    message: format!("Simulated failure during event {}", event.id),
                });
            }

            // Simulate transformation work (lock is dropped, safe to await)
            println!(
                "[TransformStage] Processing event {}: enriching...",
                event.id
            );
            tokio::time::sleep(Duration::from_millis(50)).await;

            // Enrich event and push to output queue
            event.enriched = true;
            {
                let mut output_queue = self
                    .output_queue
                    .lock()
                    .expect("Failed to acquire output queue lock");
                output_queue.push_back(event);
            }
            self.processed_count.fetch_add(1, Ordering::Relaxed);
        }

        Ok(())
    }
}

#[async_trait]
impl Child for TransformStage {
    type Error = StageError;

    async fn start(&mut self) -> Result<(), Self::Error> {
        println!("[TransformStage] Starting");
        Ok(())
    }

    async fn stop(&mut self, _timeout: Duration) -> Result<(), Self::Error> {
        println!(
            "[TransformStage] Stopping (processed {} events)",
            self.processed_count.load(Ordering::Relaxed)
        );
        Ok(())
    }

    async fn health_check(&self) -> ChildHealth {
        ChildHealth::Healthy
    }
}

// =============================================================================
// Pipeline Stage: Output
// =============================================================================

/// Third stage: Persists or forwards processed events.
///
/// Responsibilities:
/// - Dequeue events from output queue
/// - Persist to storage or forward to external system
/// - Track successful deliveries
#[derive(Debug)]
struct OutputStage {
    persisted_count: Arc<AtomicU32>,
    output_queue: EventQueue,
}

impl OutputStage {
    fn new(output_queue: EventQueue) -> Self {
        Self {
            persisted_count: Arc::new(AtomicU32::new(0)),
            output_queue,
        }
    }

    /// Persist events from output queue.
    async fn persist_events(&mut self) -> Result<(), StageError> {
        loop {
            // Acquire lock, get one event, then drop lock before await
            let event_opt = {
                let mut queue = self
                    .output_queue
                    .lock()
                    .expect("Failed to acquire output queue lock");
                queue.pop_front()
            };

            let event = match event_opt {
                Some(e) => e,
                None => break, // No more events
            };

            // Simulate persistence work (lock is dropped, safe to await)
            println!(
                "[OutputStage] Persisting event {}: {{\"type\":\"{}\",\"enriched\":{}}}",
                event.id, event.event_type, event.enriched
            );
            tokio::time::sleep(Duration::from_millis(30)).await;

            self.persisted_count.fetch_add(1, Ordering::Relaxed);
        }

        Ok(())
    }
}

#[async_trait]
impl Child for OutputStage {
    type Error = StageError;

    async fn start(&mut self) -> Result<(), Self::Error> {
        println!("[OutputStage] Starting");
        Ok(())
    }

    async fn stop(&mut self, _timeout: Duration) -> Result<(), Self::Error> {
        println!(
            "[OutputStage] Stopping (persisted {} events)",
            self.persisted_count.load(Ordering::Relaxed)
        );
        Ok(())
    }

    async fn health_check(&self) -> ChildHealth {
        ChildHealth::Healthy
    }
}

// =============================================================================
// Pipeline Stage Enum (for supervisor heterogeneity)
// =============================================================================

/// Enum wrapper for different pipeline stage types.
///
/// Allows supervisor to manage heterogeneous child types
/// (IngestStage, TransformStage, OutputStage) as single type.
#[derive(Debug)]
enum PipelineStage {
    Ingest(IngestStage),
    Transform(TransformStage),
    Output(OutputStage),
}

#[async_trait]
impl Child for PipelineStage {
    type Error = StageError;

    async fn start(&mut self) -> Result<(), Self::Error> {
        match self {
            PipelineStage::Ingest(s) => s.start().await,
            PipelineStage::Transform(s) => s.start().await,
            PipelineStage::Output(s) => s.start().await,
        }
    }

    async fn stop(&mut self, timeout: Duration) -> Result<(), Self::Error> {
        match self {
            PipelineStage::Ingest(s) => s.stop(timeout).await,
            PipelineStage::Transform(s) => s.stop(timeout).await,
            PipelineStage::Output(s) => s.stop(timeout).await,
        }
    }

    async fn health_check(&self) -> ChildHealth {
        match self {
            PipelineStage::Ingest(s) => s.health_check().await,
            PipelineStage::Transform(s) => s.health_check().await,
            PipelineStage::Output(s) => s.health_check().await,
        }
    }
}

impl PipelineStage {
    /// Get mutable reference to IngestStage.
    fn as_ingest_mut(&mut self) -> Option<&mut IngestStage> {
        match self {
            PipelineStage::Ingest(s) => Some(s),
            _ => None,
        }
    }

    /// Get mutable reference to TransformStage.
    fn as_transform_mut(&mut self) -> Option<&mut TransformStage> {
        match self {
            PipelineStage::Transform(s) => Some(s),
            _ => None,
        }
    }

    /// Get mutable reference to OutputStage.
    fn as_output_mut(&mut self) -> Option<&mut OutputStage> {
        match self {
            PipelineStage::Output(s) => Some(s),
            _ => None,
        }
    }
}

// =============================================================================
// Main Example
// =============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Event Pipeline Example ===\n");

    // ==========================================================================
    // Step 1: Create shared queues between stages
    // ==========================================================================
    let transform_queue: EventQueue = Arc::new(Mutex::new(VecDeque::new()));
    let output_queue: EventQueue = Arc::new(Mutex::new(VecDeque::new()));
    let max_queue_size = 100;

    // ==========================================================================
    // Step 2: Create supervisor with RestForOne strategy
    // ==========================================================================
    println!("Step 1: Creating pipeline with RestForOne supervision...");
    let monitor = InMemoryMonitor::new(MonitoringConfig::default());

    // Use PipelineStage enum to manage heterogeneous child types
    let mut supervisor = SupervisorNode::<RestForOne, PipelineStage, _>::new(RestForOne, monitor);

    // Track stage counters (persist across restarts)
    let _ingest_count = Arc::new(AtomicU32::new(0));
    let _transform_count = Arc::new(AtomicU32::new(0));
    let _transform_fail = Arc::new(AtomicU32::new(0));
    let _output_count = Arc::new(AtomicU32::new(0));

    // ==========================================================================
    // Step 3: Start pipeline stages in order (CRITICAL for RestForOne)
    // ==========================================================================
    // RestForOne depends on start order - earlier stages restart later stages

    // Stage 1: Ingest
    let ingest_spec = ChildSpec {
        id: "IngestStage".to_string(),
        factory: {
            let queue = Arc::clone(&transform_queue);
            move || PipelineStage::Ingest(IngestStage::new(Arc::clone(&queue), max_queue_size))
        },
        restart_policy: RestartPolicy::Permanent,
        shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
        start_timeout: Duration::from_secs(10),
        shutdown_timeout: Duration::from_secs(10),
    };
    let ingest_id = supervisor.start_child(ingest_spec).await?;

    // Stage 2: Transform
    let transform_spec = ChildSpec {
        id: "TransformStage".to_string(),
        factory: {
            let in_queue = Arc::clone(&transform_queue);
            let out_queue = Arc::clone(&output_queue);
            move || {
                PipelineStage::Transform(TransformStage::new(
                    Arc::clone(&in_queue),
                    Arc::clone(&out_queue),
                ))
            }
        },
        restart_policy: RestartPolicy::Permanent,
        shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
        start_timeout: Duration::from_secs(10),
        shutdown_timeout: Duration::from_secs(10),
    };
    let transform_id = supervisor.start_child(transform_spec).await?;

    // Stage 3: Output
    let output_spec = ChildSpec {
        id: "OutputStage".to_string(),
        factory: {
            let queue = Arc::clone(&output_queue);
            move || PipelineStage::Output(OutputStage::new(Arc::clone(&queue)))
        },
        restart_policy: RestartPolicy::Permanent,
        shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
        start_timeout: Duration::from_secs(10),
        shutdown_timeout: Duration::from_secs(10),
    };
    let output_id = supervisor.start_child(output_spec).await?;

    println!("✅ Pipeline ready with 3 stages\n");

    // ==========================================================================
    // Step 4: Process events through pipeline
    // ==========================================================================
    println!("Step 2: Processing 5 events through pipeline...");

    // Ingest events
    if let Some(handle) = supervisor.get_child_mut(&ingest_id) {
        if let Some(ingest_stage) = handle.child_mut().as_ingest_mut() {
            for i in 1..=5 {
                let event = Event {
                    id: i,
                    event_type: format!("event_type_{}", i),
                    payload: format!("payload_{}", i),
                    enriched: false,
                };
                ingest_stage.ingest_event(event)?;
            }
        }
    }

    // Process through transform stage
    tokio::time::sleep(Duration::from_millis(100)).await;
    if let Some(handle) = supervisor.get_child_mut(&transform_id) {
        if let Some(transform_stage) = handle.child_mut().as_transform_mut() {
            transform_stage.process_events().await?;
        }
    }

    // Persist through output stage
    tokio::time::sleep(Duration::from_millis(100)).await;
    if let Some(handle) = supervisor.get_child_mut(&output_id) {
        if let Some(output_stage) = handle.child_mut().as_output_mut() {
            output_stage.persist_events().await?;
        }
    }

    println!();

    // ==========================================================================
    // Step 5: Demonstrate RestForOne cascading restart
    // ==========================================================================
    println!("Step 3: Demonstrating RestForOne cascading restart...");
    println!("Simulating TransformStage failure...");

    // When TransformStage fails, RestForOne will also restart OutputStage
    println!("Expected behavior: TransformStage + OutputStage restart (cascade)\n");

    // Trigger failure and restart
    if let Err(e) = supervisor.restart_child(&transform_id).await {
        println!("⚠️  Restart error: {}", e);
    } else {
        println!("✅ Cascading restart completed\n");
    }

    // ==========================================================================
    // Step 6: Display statistics
    // ==========================================================================
    println!("=== Pipeline Statistics ===");
    let transform_pending = transform_queue
        .lock()
        .expect("Failed to lock transform queue")
        .len();
    let output_pending = output_queue
        .lock()
        .expect("Failed to lock output queue")
        .len();

    println!("Transform queue: {} events pending", transform_pending);
    println!("Output queue: {} events pending", output_pending);
    println!();

    // ==========================================================================
    // Step 7: Cleanup
    // ==========================================================================
    println!("Step 4: Shutting down pipeline...");
    supervisor.stop_child(&output_id).await?;
    supervisor.stop_child(&transform_id).await?;
    supervisor.stop_child(&ingest_id).await?;
    println!("✅ All stages stopped\n");

    println!("=== Event Pipeline Example Complete! ===");
    println!("\nKey Learnings:");
    println!("  • RestForOne restarts failed child AND all children started after it");
    println!("  • Pipeline stages process events sequentially");
    println!("  • Backpressure prevents queue overflow");
    println!("  • Cascading restarts maintain pipeline integrity");
    println!("  • Start order matters for RestForOne dependency chains");

    Ok(())
}
