//! # Pub-Sub Broadcasting Example
//!
//! **Purpose:** Demonstrates pub-sub broadcasting pattern concepts
//! **Demonstrates:** ComponentRegistry, topic-based routing, fanout simulation
//! **Run:** `cargo run --example pubsub_component`
//!
//! This example shows:
//! - Creating a ComponentRegistry for multiple subscribers
//! - Simulating topic-based message routing
//! - Demonstrating fanout (one message → multiple subscribers)

// Layer 1: Standard library imports
use std::collections::HashMap;

// Layer 2: Third-party crate imports
// (none)

// Layer 3: Internal module imports
use airssys_rt::util::ActorAddress;
use airssys_wasm::actor::ComponentRegistry;
use airssys_wasm::core::ComponentId;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Pub-Sub Broadcasting Demo ===\n");

    // Step 1: Create infrastructure
    println!("--- Setting Up Infrastructure ---");
    let registry = ComponentRegistry::new();
    let topic = "sensor.temperature".to_string();
    println!("✓ Created ComponentRegistry");
    println!("✓ Topic: {}\n", topic);

    // Step 2: Register subscriber components
    println!("--- Registering Subscribers ---");
    let mut subscribers = Vec::new();
    let mut topic_subscriptions: HashMap<String, Vec<ComponentId>> = HashMap::new();

    for i in 1..=5 {
        let subscriber_id = ComponentId::new(format!("subscriber-{}", i));
        let subscriber_addr = ActorAddress::named(format!("subscriber-actor-{}", i));

        // Register component
        registry.register(subscriber_id.clone(), subscriber_addr.clone())?;

        // Track topic subscription
        topic_subscriptions
            .entry(topic.clone())
            .or_default()
            .push(subscriber_id.clone());

        subscribers.push((subscriber_id, subscriber_addr));
        println!("  ✓ Registered and subscribed: subscriber-{}", i);
    }

    println!("\n✓ All {} subscribers registered and subscribed\n", subscribers.len());

    // Step 3: Simulate publishing events
    println!("--- Publishing Events ---");

    for event_num in 1..=3 {
        let payload = format!("Temperature: {}°C", 25 + event_num - 1);
        println!("\nEvent {}:", event_num);
        println!("  Topic: {}", topic);
        println!("  Payload: \"{}\"", payload);

        // Get all subscribers for this topic
        if let Some(topic_subscribers) = topic_subscriptions.get(&topic) {
            println!("  Fanout: {} subscribers", topic_subscribers.len());

            // Simulate routing to each subscriber
            for subscriber_id in topic_subscribers {
                let subscriber_addr = registry.lookup(subscriber_id)?;
                println!(
                    "    ✓ Would route to {} (ActorAddress: {})",
                    subscriber_id.as_str(),
                    subscriber_addr.name().unwrap_or("unnamed")
                );
            }

            println!("  ✓ Published (fanout to {} subscribers)", topic_subscribers.len());
        }

        // Simulate delivery delay
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    // Step 4: Verify subscriber registration
    println!("\n--- Verifying Subscribers ---");
    println!("Registered components: {}", registry.count()?);
    println!("Subscribers to '{}': {}", topic, topic_subscriptions.get(&topic).map(|v| v.len()).unwrap_or(0));

    for (subscriber_id, _) in &subscribers {
        println!("  ✓ {} is registered", subscriber_id.as_str());
    }

    // Step 5: Summary
    println!("\n=== Demo Complete ===");
    println!("\n✅ Pub-sub broadcasting pattern demonstrated successfully!");
    println!("   - 5 subscribers registered to topic");
    println!("   - 3 events published");
    println!("   - Fanout simulated (1 message → 5 subscribers)");
    println!("   - Registry lookups: O(1) for each subscriber");

    println!("\n📊 Performance:");
    println!("   - Fanout to 100 subscribers: 85.2µs (Task 6.2 benchmark)");
    println!("   - Per-subscriber latency: ~852ns");
    println!("   - Registry lookup: 36ns O(1)");
    println!("   - Source: benches/messaging_benchmarks.rs, benches/scalability_benchmarks.rs");

    println!("\n📝 Key Concepts:");
    println!("   - ComponentRegistry provides O(1) component lookup");
    println!("   - Topic-based routing enables fanout to multiple subscribers");
    println!("   - MessageBroker (from airssys-rt) handles actual message delivery");
    println!("   - Subscriber isolation ensures one failure doesn't affect others");

    Ok(())
}
