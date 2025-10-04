//! Basic actor example demonstrating the core Actor trait.
//!
//! This example shows:
//! - Creating a simple actor with message handling
//! - Actor lifecycle hooks (pre_start, post_stop)
//! - Error handling with ErrorAction
//! - Message processing with ActorContext

use airssys_rt::{Actor, ActorContext, ActorLifecycle, ActorState, ErrorAction, Message};
use async_trait::async_trait;
use std::fmt;

// Define a simple message type
#[derive(Debug, Clone)]
struct CounterMessage {
    delta: i32,
}

impl Message for CounterMessage {
    const MESSAGE_TYPE: &'static str = "counter";
}

// Define an actor
struct CounterActor {
    value: i32,
    max_value: i32,
}

// Define actor error type
#[derive(Debug)]
struct CounterError {
    message: String,
}

impl fmt::Display for CounterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CounterError: {}", self.message)
    }
}

impl std::error::Error for CounterError {}

// Implement the Actor trait
#[async_trait]
impl Actor for CounterActor {
    type Message = CounterMessage;
    type Error = CounterError;

    async fn handle_message(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext<Self::Message>,
    ) -> Result<(), Self::Error> {
        println!(
            "[Actor {}] Processing message with delta: {}",
            context.address().name().unwrap_or("anonymous"),
            message.delta
        );

        self.value += message.delta;

        // Check if value exceeds maximum
        if self.value > self.max_value {
            return Err(CounterError {
                message: format!("Value {} exceeds maximum {}", self.value, self.max_value),
            });
        }

        println!(
            "[Actor {}] New value: {} (messages processed: {})",
            context.address().name().unwrap_or("anonymous"),
            self.value,
            context.message_count()
        );

        // Record message processing
        context.record_message();

        Ok(())
    }

    async fn pre_start(&mut self, context: &mut ActorContext<Self::Message>) -> Result<(), Self::Error> {
        println!(
            "[Actor {}] Starting with initial value: {}",
            context.address().name().unwrap_or("anonymous"),
            self.value
        );
        Ok(())
    }

    async fn post_stop(&mut self, context: &mut ActorContext<Self::Message>) -> Result<(), Self::Error> {
        println!(
            "[Actor {}] Stopping with final value: {} (processed {} messages)",
            context.address().name().unwrap_or("anonymous"),
            self.value,
            context.message_count()
        );
        Ok(())
    }

    async fn on_error(
        &mut self,
        error: Self::Error,
        context: &mut ActorContext<Self::Message>,
    ) -> ErrorAction {
        println!(
            "[Actor {}] Error occurred: {}",
            context.address().name().unwrap_or("anonymous"),
            error
        );

        // Reset value and restart on error
        self.value = 0;
        ErrorAction::Restart
    }
}

#[tokio::main]
async fn main() {
    println!("=== Basic Actor Example ===\n");

    // Create actor instance
    let mut actor = CounterActor {
        value: 0,
        max_value: 100,
    };

    // Create actor context
    let address = airssys_rt::util::ActorAddress::named("counter-actor");
    let mut context = ActorContext::<CounterMessage>::new(address);

    // Create lifecycle tracker
    let mut lifecycle = ActorLifecycle::new();

    // Start the actor
    println!("1. Starting actor...");
    if let Err(e) = actor.pre_start(&mut context).await {
        eprintln!("Failed to start actor: {e}");
        return;
    }
    lifecycle.transition_to(ActorState::Running);
    println!("   Actor state: {:?}\n", lifecycle.state());

    // Process some messages
    println!("2. Processing messages...");
    let messages = vec![
        CounterMessage { delta: 10 },
        CounterMessage { delta: 20 },
        CounterMessage { delta: 15 },
    ];

    for msg in messages {
        match actor.handle_message(msg, &mut context).await {
            Ok(()) => println!("   Message processed successfully"),
            Err(e) => {
                println!("   Error: {e}");
                let action = actor.on_error(e, &mut context).await;
                println!("   Supervisor action: {action:?}");
                
                if action == ErrorAction::Restart {
                    lifecycle.transition_to(ActorState::Starting);
                    let _ = actor.pre_start(&mut context).await;
                    lifecycle.transition_to(ActorState::Running);
                }
            }
        }
    }

    // Stop the actor
    println!("\n3. Stopping actor...");
    lifecycle.transition_to(ActorState::Stopping);
    if let Err(e) = actor.post_stop(&mut context).await {
        eprintln!("Error during shutdown: {e}");
    }
    lifecycle.transition_to(ActorState::Stopped);

    println!("\n4. Final lifecycle state:");
    println!("   State: {:?}", lifecycle.state());
    println!("   Restart count: {}", lifecycle.restart_count());
    println!("   Is terminal: {}", lifecycle.is_terminal());
}
