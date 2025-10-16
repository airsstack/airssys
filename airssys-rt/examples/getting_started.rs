//! Getting Started Example - Your First Actor
//!
//! This example demonstrates the complete workflow from the Getting Started guide.
//! It creates a simple counter actor that handles increment, decrement, and query operations.
//!
//! Run with: cargo run --example getting_started

use airssys_rt::prelude::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;

// Step 2: Define Your Messages
#[derive(Debug, Clone, Serialize, Deserialize)]
enum CounterMessage {
    Increment,
    Decrement,
    GetValue,
    Shutdown,
}

impl Message for CounterMessage {
    const MESSAGE_TYPE: &'static str = "counter";
}

// Step 3: Implement Your Actor
struct CounterActor {
    value: i32,
}

// Define error type
#[derive(Debug)]
struct CounterError(String);

impl fmt::Display for CounterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Counter error: {}", self.0)
    }
}

impl std::error::Error for CounterError {}

#[async_trait]
impl Actor for CounterActor {
    type Message = CounterMessage;
    type Error = CounterError;

    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        match message {
            CounterMessage::Increment => {
                self.value += 1;
                println!("Counter incremented to: {}", self.value);
            }
            CounterMessage::Decrement => {
                self.value -= 1;
                println!("Counter decremented to: {}", self.value);
            }
            CounterMessage::GetValue => {
                println!("Current value: {}", self.value);
            }
            CounterMessage::Shutdown => {
                println!("Shutting down counter actor");
                return Err(CounterError("Shutdown requested".to_string()));
            }
        }

        // Record that we processed a message
        context.record_message();
        Ok(())
    }
}

// Step 4: Create and Run Your Actor
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Getting Started Example ===\n");

    // Create actor instance
    let mut actor = CounterActor { value: 0 };

    // Create actor context with address and message broker
    let address = ActorAddress::named("counter");
    let broker = InMemoryMessageBroker::<CounterMessage>::new();
    let mut context = ActorContext::new(address, broker);

    // Create lifecycle tracker
    let mut lifecycle = ActorLifecycle::new();

    // Start the actor
    println!("1. Starting actor...");
    actor.pre_start(&mut context).await?;
    lifecycle.transition_to(ActorState::Running);
    println!("   Actor is running\n");

    // Process messages
    println!("2. Sending messages...");
    let messages = vec![
        CounterMessage::Increment,
        CounterMessage::Increment,
        CounterMessage::GetValue,
        CounterMessage::Decrement,
        CounterMessage::GetValue,
    ];

    for msg in messages {
        match actor.handle_message(msg, &mut context).await {
            Ok(()) => {
                println!(
                    "   ✓ Message processed (total: {})",
                    context.message_count()
                );
            }
            Err(e) => {
                println!("   ✗ Error: {e}");
                let action = actor.on_error(e, &mut context).await;
                match action {
                    ErrorAction::Resume => {
                        println!("   Resuming after error...");
                    }
                    ErrorAction::Stop => {
                        lifecycle.transition_to(ActorState::Stopping);
                        break;
                    }
                    ErrorAction::Restart => {
                        lifecycle.transition_to(ActorState::Starting);
                        actor.pre_start(&mut context).await?;
                        lifecycle.transition_to(ActorState::Running);
                    }
                    ErrorAction::Escalate => {
                        println!("   Escalating to supervisor...");
                        break;
                    }
                }
            }
        }
    }

    // Graceful shutdown
    println!("\n3. Shutting down...");
    let shutdown_result = actor
        .handle_message(CounterMessage::Shutdown, &mut context)
        .await;
    if shutdown_result.is_err() {
        lifecycle.transition_to(ActorState::Stopping);
        actor.post_stop(&mut context).await?;
        lifecycle.transition_to(ActorState::Stopped);
    }

    println!("\n4. Final state:");
    println!("   State: {:?}", lifecycle.state());
    println!("   Messages processed: {}", context.message_count());
    println!("   Restart count: {}", lifecycle.restart_count());

    println!("\n=== Example Complete ===");
    Ok(())
}
