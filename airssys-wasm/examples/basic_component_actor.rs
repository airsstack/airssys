//! # Basic ComponentActor Example
//!
//! **Purpose**: Demonstrate minimal ComponentActor construction and inspection
//! **Demonstrates**: ComponentActor creation, state inspection, basic lifecycle
//! **Run**: `cargo run --example basic_component_actor`
//!
//! This example shows the simplest possible ComponentActor usage:
//! - Create a ComponentActor with no custom state
//! - Inspect component metadata and state
//! - Understand the component lifecycle states

// Layer 1: Standard library imports
// (none needed)

// Layer 2: Third-party crate imports
use chrono::Utc;

// Layer 3: Internal module imports
use airssys_wasm::actor::{ActorState, ComponentActor};
use airssys_wasm::core::{CapabilitySet, ComponentId, ComponentMetadata, ResourceLimits};

/// Create example component metadata
fn create_example_metadata() -> ComponentMetadata {
    ComponentMetadata {
        name: "basic-example".to_string(),
        version: "1.0.0".to_string(),
        author: "Example Author".to_string(),
        description: Some("Basic ComponentActor demonstration".to_string()),
        required_capabilities: vec![],
        resource_limits: ResourceLimits {
            max_memory_bytes: 64 * 1024 * 1024,  // 64MB
            max_fuel: 1_000_000,                 // 1M fuel
            max_execution_ms: 5000,              // 5s timeout
            max_storage_bytes: 10 * 1024 * 1024, // 10MB storage
        },
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Basic ComponentActor Example ===\n");

    // Step 1: Create component ID
    let component_id = ComponentId::new("basic-example");
    println!("✓ Created component ID: {}", component_id.as_str());

    // Step 2: Create metadata
    let metadata = create_example_metadata();
    println!("✓ Created metadata:");
    println!("  - Name: {} v{}", metadata.name, metadata.version);
    println!("  - Author: {}", metadata.author);
    println!(
        "  - Memory limit: {} MB",
        metadata.resource_limits.max_memory_bytes / (1024 * 1024)
    );
    println!("  - Fuel limit: {}", metadata.resource_limits.max_fuel);

    // Step 3: Create capabilities (empty for basic example)
    let capabilities = CapabilitySet::new();
    println!("✓ Created capabilities (empty set)");

    // Step 4: Construct ComponentActor with no custom state (default: ())
    let start_time = Utc::now();
    let actor = ComponentActor::new(
        component_id.clone(),
        metadata,
        capabilities,
        (), // No custom state
    );
    println!("✓ ComponentActor constructed\n");

    // Step 5: Inspect component state
    println!("--- Component State Inspection ---");
    println!("Component ID: {}", actor.component_id().as_str());

    // Check current lifecycle state
    match actor.state() {
        ActorState::Creating => println!("Lifecycle state: Creating (initial state)"),
        ActorState::Starting => println!("Lifecycle state: Starting (WASM loading)"),
        ActorState::Ready => println!("Lifecycle state: Ready (active)"),
        ActorState::Stopping => println!("Lifecycle state: Stopping (cleanup)"),
        ActorState::Terminated => println!("Lifecycle state: Terminated (stopped)"),
        ActorState::Failed(reason) => println!("Lifecycle state: Failed ({})", reason),
    }

    // Check WASM runtime status
    if actor.is_wasm_loaded() {
        println!("WASM runtime: Loaded");
    } else {
        println!("WASM runtime: Not loaded (will load in Child::start())");
    }

    // Check uptime
    match actor.uptime() {
        Some(uptime) => println!("Uptime: {} seconds", uptime.num_seconds()),
        None => println!("Uptime: None (component not started)"),
    }

    // Calculate construction time
    let construction_time = Utc::now() - start_time;
    println!(
        "\nConstruction time: {}µs",
        construction_time.num_microseconds().unwrap_or(0)
    );
    println!("(Target: 286ns from Task 6.2 benchmarks)");

    // Step 6: Demonstrate state transitions (conceptual)
    println!("\n--- Lifecycle State Machine ---");
    println!("ComponentActor lifecycle transitions:");
    println!("  Creating → Starting → Ready → Stopping → Terminated");
    println!("              ↓            ↓         ↓");
    println!("          Failed       Failed    Failed");
    println!("\nCurrent state: {:?}", actor.state());
    println!("Next step would be: Child::start() to load WASM runtime");

    // Step 7: Show expected next steps
    println!("\n--- Next Steps ---");
    println!("To use this component in a real application:");
    println!("1. Call Child::start() to load WASM runtime");
    println!("2. Component transitions to ActorState::Ready");
    println!("3. Send messages via mailbox");
    println!("4. Call Child::stop() for graceful shutdown");

    // Step 8: Performance characteristics
    println!("\n--- Performance Characteristics ---");
    println!("ComponentActor construction: 286ns (Task 6.2 benchmark)");
    println!("Full lifecycle (start+stop): 1.49µs (Task 6.2 benchmark)");
    println!("Memory footprint: < 2MB per instance (target)");

    println!("\n=== Example Complete ===");

    Ok(())
}
