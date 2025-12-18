//! Example: Actor Address Routing
//!
//! Demonstrates:
//! - Creating ComponentRegistry and MessageRouter
//! - Registering component addresses manually
//! - Routing messages via MessageRouter
//! - Error handling for component-not-found

// Layer 1: Standard library imports
use std::sync::Arc;

// Layer 2: Third-party crate imports
// (none)

// Layer 3: Internal module imports
use airssys_rt::broker::InMemoryMessageBroker;
use airssys_rt::util::ActorAddress;
use airssys_wasm::actor::{ComponentMessage, ComponentRegistry, MessageRouter};
use airssys_wasm::core::ComponentId;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Actor Address Routing Example\n");

    // Create ComponentRegistry
    let registry = ComponentRegistry::new();
    println!("✓ Created ComponentRegistry");

    // Create MessageBroker
    let broker = Arc::new(InMemoryMessageBroker::new());
    println!("✓ Created MessageBroker");

    // Create MessageRouter
    let router = MessageRouter::new(registry.clone(), broker);
    println!("✓ Created MessageRouter\n");

    // Manually register some component addresses
    // (In real usage, ComponentSpawner does this automatically)
    let component_a = ComponentId::new("component-a");
    let component_b = ComponentId::new("component-b");
    let component_c = ComponentId::new("component-c");

    registry.register(component_a.clone(), ActorAddress::named("actor-a"))?;
    registry.register(component_b.clone(), ActorAddress::named("actor-b"))?;
    registry.register(component_c.clone(), ActorAddress::named("actor-c"))?;
    println!("✓ Registered 3 components (A, B, C)\n");

    // Example 1: Send message to single component
    println!("📨 Example 1: Single Message Routing");
    println!("→ Routing HealthCheck to component-b...");
    router
        .send_message(&component_b, ComponentMessage::HealthCheck)
        .await?;
    println!("✓ Message routed successfully\n");

    // Example 2: Error handling - nonexistent component
    println!("❌ Example 2: Error Handling");
    let nonexistent = ComponentId::new("nonexistent");
    println!("→ Attempting to route to nonexistent component...");
    match router
        .send_message(&nonexistent, ComponentMessage::HealthCheck)
        .await
    {
        Ok(_) => println!("✗ Unexpected success"),
        Err(e) => println!("✓ Expected error: {}", e),
    }
    println!();

    // Example 3: Broadcast to multiple components
    println!("📡 Example 3: Broadcast Messaging");
    let targets = vec![
        component_a.clone(),
        component_b.clone(),
        component_c.clone(),
    ];
    println!(
        "→ Broadcasting HealthCheck to {} components...",
        targets.len()
    );
    router
        .broadcast_message(&targets, ComponentMessage::HealthCheck)
        .await?;
    println!("✓ Broadcast completed\n");

    // Example 4: Try-broadcast with mixed results
    println!("🔄 Example 4: Best-Effort Broadcast");
    let mixed_targets = vec![
        component_a.clone(),
        ComponentId::new("nonexistent-1"),
        component_b.clone(),
        ComponentId::new("nonexistent-2"),
        component_c.clone(),
    ];
    println!(
        "→ Try-broadcast to {} targets (including nonexistent)...",
        mixed_targets.len()
    );
    let results = router
        .try_broadcast_message(&mixed_targets, ComponentMessage::HealthCheck)
        .await;

    for (comp_id, result) in results {
        match result {
            Ok(_) => println!("   ✓ {}: delivered", comp_id.as_str()),
            Err(e) => println!("   ✗ {}: {}", comp_id.as_str(), e),
        }
    }
    println!();

    // Example 5: Registry statistics
    println!("📊 Example 5: Registry Statistics");
    println!("   Total components: {}", router.component_count()?);
    println!(
        "   Component A exists: {}",
        router.component_exists(&component_a)
    );
    println!(
        "   Component B exists: {}",
        router.component_exists(&component_b)
    );
    println!(
        "   Component C exists: {}",
        router.component_exists(&component_c)
    );
    println!(
        "   Nonexistent exists: {}",
        router.component_exists(&nonexistent)
    );
    println!();

    // Example 6: Unregister component
    println!("🗑️  Example 6: Component Lifecycle");
    println!("→ Unregistering component-b...");
    registry.unregister(&component_b)?;
    println!("✓ Component unregistered");
    println!(
        "   Component B exists: {}",
        router.component_exists(&component_b)
    );
    println!("   Total components: {}", router.component_count()?);

    println!("\n✅ All examples completed successfully!");
    println!("\n💡 Key Takeaways:");
    println!("   • MessageRouter provides high-level routing API");
    println!("   • O(1) lookup performance via ComponentRegistry");
    println!("   • Graceful error handling for missing components");
    println!("   • Broadcast supports both fail-fast and best-effort modes");
    println!("   • Router automatically tracks registry changes");

    Ok(())
}
