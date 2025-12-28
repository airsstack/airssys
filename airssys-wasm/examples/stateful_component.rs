//! # Stateful Component Example
//!
//! **Purpose**: Demonstrate ComponentActor with custom state management
//! **Demonstrates**: State initialization, state access patterns, concurrent state updates
//! **Run**: `cargo run --example stateful_component`
//!
//! This example shows how to:
//! - Define custom state structs
//! - Initialize ComponentActor with state
//! - Read state with `with_state()`
//! - Modify state with `with_state_mut()`
//! - Clone state with `get_state()`
//! - Share state with `state_arc()`

// Layer 1: Standard library imports
use std::time::Duration;

// Layer 2: Third-party crate imports
use chrono::{DateTime, Utc};
use tokio::time::sleep;

// Layer 3: Internal module imports
use airssys_wasm::actor::ComponentActor;
use airssys_wasm::core::{CapabilitySet, ComponentId, ComponentMetadata, ResourceLimits};

/// Custom state tracking message processing statistics
#[derive(Debug, Clone, Default)]
struct MessageStats {
    /// Total messages processed
    message_count: u64,

    /// Last message timestamp
    last_message_at: Option<DateTime<Utc>>,

    /// Error count
    error_count: u64,

    /// Processing times (last 10 messages)
    processing_times_ms: Vec<u64>,
}

impl MessageStats {
    /// Record a successful message processing
    fn record_success(&mut self, processing_time_ms: u64) {
        self.message_count += 1;
        self.last_message_at = Some(Utc::now());

        // Keep last 10 processing times
        self.processing_times_ms.push(processing_time_ms);
        if self.processing_times_ms.len() > 10 {
            self.processing_times_ms.remove(0);
        }
    }

    /// Record an error
    fn record_error(&mut self) {
        self.error_count += 1;
    }

    /// Calculate average processing time
    fn avg_processing_time_ms(&self) -> f64 {
        if self.processing_times_ms.is_empty() {
            0.0
        } else {
            let sum: u64 = self.processing_times_ms.iter().sum();
            sum as f64 / self.processing_times_ms.len() as f64
        }
    }
}

/// Create example component metadata
fn create_metadata() -> ComponentMetadata {
    ComponentMetadata {
        name: "stateful-example".to_string(),
        version: "1.0.0".to_string(),
        author: "Example Author".to_string(),
        description: Some("Stateful ComponentActor demonstration".to_string()),
        max_memory_bytes: 64 * 1024 * 1024,
        max_fuel: 1_000_000,
        timeout_seconds: 5,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Stateful Component Example ===\n");

    // Step 1: Create ComponentActor with custom state
    let component_id = ComponentId::new("stateful-example");
    let metadata = create_metadata();
    let capabilities = CapabilitySet::new();

    // Initialize with default state
    let initial_state = MessageStats::default();

    let actor: ComponentActor<MessageStats> =
        ComponentActor::new(component_id.clone(), metadata, capabilities, initial_state);

    println!("✓ Created ComponentActor with MessageStats state");
    println!("Component ID: {}\n", component_id.as_str());

    // Step 2: Read initial state (read-only)
    println!("--- Initial State (Read-Only) ---");
    actor
        .with_state(|state| {
            println!("Message count: {}", state.message_count);
            println!("Error count: {}", state.error_count);
            println!("Last message: {:?}", state.last_message_at);
        })
        .await;

    // Step 3: Simulate processing messages (mutable state)
    println!("\n--- Simulating Message Processing ---");

    for i in 1..=5 {
        let start = Utc::now();

        // Simulate work
        sleep(Duration::from_millis(10 + i * 5)).await;

        let processing_time = (Utc::now() - start).num_milliseconds() as u64;

        // Update state
        actor
            .with_state_mut(|state| {
                state.record_success(processing_time);
            })
            .await;

        println!("✓ Processed message {} ({}ms)", i, processing_time);
    }

    // Step 4: Simulate an error
    println!("\n✗ Simulating error condition...");
    actor
        .with_state_mut(|state| {
            state.record_error();
        })
        .await;

    // Step 5: Read final state
    println!("\n--- Final State ---");
    let final_count = actor
        .with_state(|state| {
            println!("Total messages: {}", state.message_count);
            println!("Error count: {}", state.error_count);
            println!(
                "Avg processing time: {:.2}ms",
                state.avg_processing_time_ms()
            );

            if let Some(last_msg) = state.last_message_at {
                println!("Last message: {}", last_msg.format("%H:%M:%S"));
            }

            state.message_count
        })
        .await;

    // Step 6: Clone state (requires Clone trait)
    println!("\n--- Cloning State ---");
    let state_copy = actor.get_state().await;
    println!("Cloned state - message count: {}", state_copy.message_count);
    println!("(Original and clone are independent)");

    // Step 7: Share state Arc (for concurrent access)
    println!("\n--- Sharing State Arc ---");
    let state_arc = actor.state_arc();
    println!("Created Arc<RwLock<MessageStats>> reference");

    // Spawn a background task that reads state
    let background_task = tokio::spawn(async move {
        sleep(Duration::from_millis(50)).await;

        let guard = state_arc.read().await;
        println!(
            "  [Background task] Read message count: {}",
            guard.message_count
        );
    });

    // Main task continues (concurrent access)
    actor
        .with_state(|state| {
            println!("  [Main task] Read message count: {}", state.message_count);
        })
        .await;

    // Wait for background task
    background_task.await?;

    // Step 8: Replace entire state
    println!("\n--- Replacing State ---");
    let new_state = MessageStats {
        message_count: 100,
        last_message_at: Some(Utc::now()),
        error_count: 5,
        processing_times_ms: vec![10, 15, 20],
    };

    let old_state = actor.set_custom_state(new_state).await;
    println!("Replaced state:");
    println!("  Old message count: {}", old_state.message_count);

    let current_count = actor.with_state(|state| state.message_count).await;
    println!("  New message count: {}", current_count);

    // Step 9: State management patterns summary
    println!("\n--- State Management Patterns ---");
    println!("1. with_state()      - Read-only access (shared lock)");
    println!("2. with_state_mut()  - Mutable access (exclusive lock)");
    println!("3. get_state()       - Clone entire state (requires Clone)");
    println!("4. set_custom_state()- Replace entire state");
    println!("5. state_arc()       - Share Arc<RwLock<S>> for concurrent access");

    println!("\n--- Concurrency Characteristics ---");
    println!("Multiple readers: Allowed (RwLock read lock)");
    println!("Multiple writers: Blocked (RwLock write lock)");
    println!("Reader + writer: Writer waits (RwLock semantics)");

    println!("\n--- Performance Notes ---");
    println!("State access: 37-39ns (Task 6.2 benchmarks)");
    println!("State is thread-safe: Arc<RwLock<S>>");
    println!("No locks held during async operations");

    println!("\n=== Example Complete ===");
    println!(
        "Summary: Processed {} messages with {} errors",
        final_count, 1
    );

    Ok(())
}
