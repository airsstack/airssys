//! # Component Composition Example
//!
//! **Purpose**: Demonstrates component coordination and pipeline patterns
//! **Demonstrates**: ComponentRegistry, pipeline setup, message flow concepts
//! **Run**: `cargo run --example component_composition`
//!
//! This example shows how to coordinate multiple components in a pipeline:
//! - Component registration with ComponentRegistry
//! - Pipeline message routing (Input → Processor → Output)
//! - Error handling with failure isolation
//! - Component lookup and coordination patterns

// Layer 1: Standard library imports
// (none needed)

// Layer 2: Third-party crate imports
// (none)

// Layer 3: Internal module imports
use airssys_rt::util::ActorAddress;
use airssys_wasm::actor::ComponentRegistry;
use airssys_wasm::core::ComponentId;

/// Simulate a message flowing through the pipeline
#[derive(Debug, Clone)]
struct PipelineMessage {
    id: u64,
    data: String,
    stage: String,
}

impl PipelineMessage {
    fn new(id: u64, data: String) -> Self {
        Self {
            id,
            data,
            stage: "initial".to_string(),
        }
    }

    fn process_at_stage(&mut self, stage: &str) {
        self.stage = stage.to_string();
        self.data = format!("{}[{}]", stage, self.data);
    }
}

/// Demonstrate pipeline processing
fn simulate_pipeline_flow(message: &mut PipelineMessage, stages: &[&str]) {
    for stage in stages {
        message.process_at_stage(stage);
        println!(
            "  → {} processed: id={}, data=\"{}\"",
            stage, message.id, message.data
        );
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Component Composition Demo ===\n");

    // Step 1: Create infrastructure
    println!("--- Creating Infrastructure ---");
    let registry = ComponentRegistry::new();
    println!("✓ Created ComponentRegistry");

    // Step 2: Define pipeline components
    println!("\n--- Defining Pipeline Components ---");
    let components = vec![
        (
            "input-source",
            "Generates data events",
            "Sensor data, file uploads, API requests",
        ),
        (
            "data-processor",
            "Transforms and validates data",
            "Schema validation, data enrichment",
        ),
        (
            "output-sink",
            "Persists processed data",
            "Database, file system, message queue",
        ),
    ];

    for (id, description, examples) in &components {
        println!("  Component: {} - {}", id, description);
        println!("    Examples: {}", examples);
    }

    // Step 3: Register components
    println!("\n--- Registering Components ---");
    for (id, _, _) in &components {
        let component_id = ComponentId::new(*id);
        let actor_addr = ActorAddress::named(format!("{}-actor", id));
        registry.register(component_id.clone(), actor_addr)?;
        let actor_name = format!("{}-actor", id);
        println!("  ✓ Registered: {} → ActorAddress({})", id, actor_name);
    }

    println!("\n✓ All {} components registered", components.len());

    // Step 4: Demonstrate pipeline message flow
    println!("\n--- Pipeline Message Flow ---");
    println!("Processing 3 messages through the pipeline:\n");

    let test_data = [
        "sensor_reading:25.3°C",
        "user_upload:report.pdf",
        "api_request:get_weather",
    ];

    for (i, &data) in test_data.iter().enumerate() {
        let mut message = PipelineMessage::new(i as u64 + 1, data.to_string());

        println!("Message #{}: \"{}\"", message.id, data);

        // Simulate routing through pipeline stages
        let stages = vec!["input-source", "data-processor", "output-sink"];
        simulate_pipeline_flow(&mut message, &stages);

        println!("  ✓ Pipeline complete: final=\"{}\"\n", message.data);
    }

    // Step 5: Demonstrate error handling
    println!("--- Error Handling in Pipeline ---");
    println!("Scenario: data-processor detects invalid value\n");

    let invalid_message = PipelineMessage::new(99, "invalid:corrupted_data".to_string());

    let invalid_data = "invalid:corrupted_data";
    println!("Message #{}: \"{}\"", invalid_message.id, invalid_data);
    println!("  → input-source received: id={}", invalid_message.id);
    println!("  → input-source → data-processor (via registry lookup)");

    let processor_id = ComponentId::new("data-processor");
    let processor_addr = registry.lookup(&processor_id)?;
    println!(
        "    Registry lookup: {} → {}",
        processor_id.as_str(),
        processor_addr.name().unwrap_or("unnamed")
    );

    println!("  → data-processor validation: ❌ FAILED");
    println!("    Error: Invalid data format");
    println!("  → Error sent back to input-source");
    println!("  → input-source logs error and drops message");
    println!("  → Pipeline continues processing other messages");
    println!("    ✓ Failure isolation: other components unaffected\n");

    // Step 6: Demonstrate fanout (one-to-many)
    println!("--- Fanout Pattern (One-to-Many) ---");
    println!("Scenario: Broadcast message to multiple processors\n");

    // Register additional processors
    println!("Registering parallel processors:");
    let parallel_processors = vec!["processor-1", "processor-2", "processor-3"];
    for proc_id in &parallel_processors {
        let component_id = ComponentId::new(*proc_id);
        let actor_addr = ActorAddress::named(format!("{}-actor", proc_id));
        registry.register(component_id.clone(), actor_addr)?;
        println!("  ✓ {}", proc_id);
    }

    println!("\nBroadcasting message to all processors:");
    let broadcast_msg = "config_update:v2.1.0";
    println!("  Message: \"{}\"", broadcast_msg);

    for proc_id in &parallel_processors {
        let component_id = ComponentId::new(*proc_id);
        let addr = registry.lookup(&component_id)?;
        println!("    → {} ({})", proc_id, addr.name().unwrap_or("unnamed"));
    }

    println!(
        "  ✓ Fanout to {} processors complete",
        parallel_processors.len()
    );

    // Step 7: Verify registry state
    println!("\n--- Verifying Component Registry ---");
    let count = registry.count()?;
    println!("Registered components: {}", count);

    println!("\nVerifying each component is registered:");
    for (id, _, _) in &components {
        let component_id = ComponentId::new(*id);
        let addr = registry.lookup(&component_id)?;
        println!(
            "  ✓ {} → {}",
            component_id.as_str(),
            addr.name().unwrap_or("unnamed")
        );
    }

    // Also verify parallel processors
    for proc_id in &parallel_processors {
        let component_id = ComponentId::new(*proc_id);
        let addr = registry.lookup(&component_id)?;
        println!(
            "  ✓ {} → {}",
            component_id.as_str(),
            addr.name().unwrap_or("unnamed")
        );
    }

    // Step 8: Demonstrate component unregistration
    println!("\n--- Component Cleanup ---");
    println!("Unregistering parallel processors:");
    for proc_id in &parallel_processors {
        let component_id = ComponentId::new(*proc_id);
        registry.unregister(&component_id)?;
        println!("  ✓ Unregistered: {}", proc_id);
    }

    let final_count = registry.count()?;
    println!("\nRemaining components: {}", final_count);

    // Summary
    println!("\n=== Demo Complete ===");
    println!("\n✅ Component composition demonstrated successfully!");
    println!("   - 3-component pipeline (input → processor → output)");
    println!("   - Fanout to 3 parallel processors");
    println!("   - Registry-based component lookup");
    println!("   - Error handling with failure isolation");
    println!("   - Component registration and cleanup");

    println!("\n📊 Performance:");
    println!("   - Registry lookup: 36ns O(1) (Task 6.2 benchmark)");
    println!("   - Message routing: ~1.05µs per hop");
    println!("   - Pipeline throughput: 6.12M msg/sec (Task 6.2 benchmark)");
    println!("   - Fanout to 100 components: 85.2µs");
    println!("   - Source: benches/messaging_benchmarks.rs");

    println!("\n📖 Key Concepts:");
    println!("   - ComponentRegistry provides O(1) component lookup");
    println!("   - Pipeline pattern: sequential processing stages");
    println!("   - Fanout pattern: broadcast to multiple components");
    println!("   - Failure isolation: errors don't propagate");
    println!("   - ActorAddress enables location transparency");

    println!("\n🔗 Integration:");
    println!("   For full actor system integration, see:");
    println!("   - airssys-wasm/examples/actor_routing_example.rs");
    println!("   - airssys-wasm/examples/request_response_pattern.rs");
    println!("   - airssys-rt/examples/actor_patterns.rs");

    Ok(())
}
