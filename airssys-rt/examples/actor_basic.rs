//! Basic Actor Example - Core Actor Trait Fundamentals
//!
//! This example demonstrates the fundamentals of creating and using actors in AirsSys RT.
//! It shows the complete actor lifecycle, message handling, and error recovery patterns.
//!
//! # What You'll Learn
//!
//! - Creating a simple actor with message handling
//! - Actor lifecycle hooks (`pre_start`, `post_stop`, `on_error`)
//! - Error handling with `ErrorAction` (Stop, Resume, Restart, Escalate)
//! - Message processing with `ActorContext`
//! - Lifecycle state transitions (Starting → Running → Stopping → Stopped)
//!
//! # Key Concepts
//!
//! **Actor Trait**: The core trait that all actors must implement
//! - `handle_message()`: Process incoming messages (REQUIRED)
//! - `pre_start()`: Initialize actor before receiving messages (optional)
//! - `post_stop()`: Cleanup when actor stops (optional)
//! - `on_error()`: Handle errors and return supervision decision (optional)
//!
//! **ActorContext**: Provides actor metadata and messaging capabilities
//! - Access to actor address and ID
//! - Message statistics (count, last message time)
//! - Ability to send messages to other actors
//!
//! **ErrorAction**: Supervision decision after errors
//! - `Stop`: Stop the actor permanently
//! - `Resume`: Continue processing (ignore error)
//! - `Restart`: Restart actor (call `pre_start` again)
//! - `Escalate`: Propagate error to parent supervisor
//!
//! # Run This Example
//!
//! ```bash
//! cargo run --example actor_basic
//! ```
//!
//! # Expected Output
//!
//! ```text
//! === Basic Actor Example ===
//!
//! 1. Starting actor...
//! [Actor counter-actor] Starting with initial value: 0
//!    Actor state: Running
//!
//! 2. Processing messages...
//! [Actor counter-actor] Processing message with delta: 10
//! [Actor counter-actor] New value: 10 (messages processed: 0)
//!    Message processed successfully
//! [Actor counter-actor] Processing message with delta: 20
//! [Actor counter-actor] New value: 30 (messages processed: 1)
//!    Message processed successfully
//! [Actor counter-actor] Processing message with delta: 15
//! [Actor counter-actor] New value: 45 (messages processed: 2)
//!    Message processed successfully
//!
//! 3. Stopping actor...
//! [Actor counter-actor] Stopping with final value: 45 (processed 3 messages)
//!
//! 4. Final lifecycle state:
//!    State: Stopped
//!    Restart count: 0
//!    Is terminal: true
//! ```
//!
//! # See Also
//!
//! - [`actor_lifecycle.rs`](actor_lifecycle.rs) - Detailed lifecycle transition examples
//! - [`actor_patterns.rs`](actor_patterns.rs) - Advanced actor patterns
//! - [`getting_started.rs`](getting_started.rs) - Complete quickstart guide
//! - [Actor Development Guide](../docs/src/guides/actor-development.md) - Comprehensive patterns

use airssys_rt::{Actor, ActorContext, ActorLifecycle, ActorState, ErrorAction, Message};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;

// =============================================================================
// Message Definition
// =============================================================================

// Define a simple message type that our actor will process.
// Messages must implement Clone + Serialize + Deserialize for message passing.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CounterMessage {
    delta: i32, // Value to add to the counter
}

// Implement the Message trait to integrate with the actor system.
// MESSAGE_TYPE is used for message routing and monitoring.
impl Message for CounterMessage {
    const MESSAGE_TYPE: &'static str = "counter";
}

// =============================================================================
// Actor Definition
// =============================================================================

// Define an actor that maintains a counter with a maximum value.
// Actors encapsulate state and process messages sequentially.
struct CounterActor {
    value: i32,     // Current counter value (private state)
    max_value: i32, // Maximum allowed value (boundary condition)
}

// =============================================================================
// Error Type Definition
// =============================================================================

// Define actor-specific error type for domain errors.
// This error is returned when business logic fails (e.g., exceeding max value).
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

// =============================================================================
// Actor Trait Implementation
// =============================================================================

// Implement the Actor trait to define message handling and lifecycle behavior.
// This is the core trait that makes CounterActor part of the actor system.
#[async_trait]
impl Actor for CounterActor {
    type Message = CounterMessage; // Messages this actor can handle
    type Error = CounterError; // Errors this actor can return

    /// Process incoming messages (REQUIRED method).
    ///
    /// This is the heart of the actor - defines how it responds to messages.
    /// Messages are processed sequentially (one at a time) ensuring thread safety.
    async fn handle_message<B: airssys_rt::broker::MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        println!(
            "[Actor {}] Processing message with delta: {}",
            context.address().name().unwrap_or("anonymous"),
            message.delta
        );

        // Update actor state (safe because messages are processed sequentially)
        self.value += message.delta;

        // Business logic: Check boundary condition
        // When exceeded, return error which triggers supervision
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

        // Record message statistics in context (for monitoring)
        context.record_message();

        Ok(())
    }

    /// Initialize actor before receiving messages (optional hook).
    ///
    /// Called when actor starts or restarts. Use for setup logic like:
    /// - Loading initial state from database
    /// - Establishing connections
    /// - Subscribing to topics
    async fn pre_start<B: airssys_rt::broker::MessageBroker<Self::Message>>(
        &mut self,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        println!(
            "[Actor {}] Starting with initial value: {}",
            context.address().name().unwrap_or("anonymous"),
            self.value
        );
        Ok(())
    }

    /// Cleanup when actor stops (optional hook).
    ///
    /// Called when actor is stopping. Use for cleanup logic like:
    /// - Closing connections
    /// - Flushing buffers
    /// - Saving state
    async fn post_stop<B: airssys_rt::broker::MessageBroker<Self::Message>>(
        &mut self,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        println!(
            "[Actor {}] Stopping with final value: {} (processed {} messages)",
            context.address().name().unwrap_or("anonymous"),
            self.value,
            context.message_count()
        );
        Ok(())
    }

    /// Handle errors and return supervision decision (optional hook).
    ///
    /// Called when handle_message returns an error. The ErrorAction determines
    /// how the supervisor should handle the failure:
    /// - Stop: Terminate actor permanently
    /// - Resume: Continue processing (ignore error)
    /// - Restart: Call pre_start() and resume processing
    /// - Escalate: Propagate error to parent supervisor
    async fn on_error<B: airssys_rt::broker::MessageBroker<Self::Message>>(
        &mut self,
        error: Self::Error,
        context: &mut ActorContext<Self::Message, B>,
    ) -> ErrorAction {
        println!(
            "[Actor {}] Error occurred: {}",
            context.address().name().unwrap_or("anonymous"),
            error
        );

        // Reset value and restart on error (recovery strategy)
        self.value = 0;
        ErrorAction::Restart
    }
}

// =============================================================================
// Main Example
// =============================================================================

// =============================================================================
// Main Example
// =============================================================================

#[tokio::main]
async fn main() {
    println!("=== Basic Actor Example ===\n");

    // Step 1: Create actor instance with initial state
    //
    // Actors encapsulate state and are created like normal Rust structs.
    // The state is private and only accessible through message handling.
    let mut actor = CounterActor {
        value: 0,       // Start at zero
        max_value: 100, // Fail if value exceeds 100
    };

    // Step 2: Create actor context
    //
    // ActorContext provides:
    // - Actor identity (address and ID)
    // - Message statistics (count, timestamps)
    // - MessageBroker for sending messages to other actors
    let address = airssys_rt::util::ActorAddress::named("counter-actor");
    let broker = airssys_rt::broker::in_memory::InMemoryMessageBroker::<CounterMessage>::new();
    let mut context = ActorContext::new(address, broker);

    // Step 3: Create lifecycle tracker
    //
    // ActorLifecycle tracks state transitions:
    // Created → Starting → Running → Stopping → Stopped
    let mut lifecycle = ActorLifecycle::new();

    // ==========================================================================
    // Step 4: Start the actor (pre_start lifecycle hook)
    // ==========================================================================
    println!("1. Starting actor...");
    if let Err(e) = actor.pre_start(&mut context).await {
        eprintln!("Failed to start actor: {e}");
        return;
    }
    lifecycle.transition_to(ActorState::Running);
    println!("   Actor state: {:?}\n", lifecycle.state());

    // ==========================================================================
    // Step 5: Process messages (handle_message)
    // ==========================================================================
    println!("2. Processing messages...");
    let messages = vec![
        CounterMessage { delta: 10 }, // value: 0 → 10
        CounterMessage { delta: 20 }, // value: 10 → 30
        CounterMessage { delta: 15 }, // value: 30 → 45
    ];

    for msg in messages {
        match actor.handle_message(msg, &mut context).await {
            Ok(()) => println!("   Message processed successfully"),
            Err(e) => {
                // Error occurred - invoke error handler
                println!("   Error: {e}");
                let action = actor.on_error(e, &mut context).await;
                println!("   Supervisor action: {action:?}");

                // Handle ErrorAction::Restart
                if action == ErrorAction::Restart {
                    lifecycle.transition_to(ActorState::Starting);
                    let _ = actor.pre_start(&mut context).await;
                    lifecycle.transition_to(ActorState::Running);
                }
            }
        }
    }

    // ==========================================================================
    // Step 6: Stop the actor (post_stop lifecycle hook)
    // ==========================================================================
    println!("\n3. Stopping actor...");
    lifecycle.transition_to(ActorState::Stopping);
    if let Err(e) = actor.post_stop(&mut context).await {
        eprintln!("Error during shutdown: {e}");
    }
    lifecycle.transition_to(ActorState::Stopped);

    // ==========================================================================
    // Step 7: Display final lifecycle state
    // ==========================================================================
    println!("\n4. Final lifecycle state:");
    println!("   State: {:?}", lifecycle.state());
    println!("   Restart count: {}", lifecycle.restart_count());
    println!("   Is terminal: {}", lifecycle.is_terminal());
}
