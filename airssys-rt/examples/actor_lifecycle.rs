//! Actor lifecycle example demonstrating state transitions and supervision.
//!
//! This example shows:
//! - Actor lifecycle state machine
//! - Error handling and recovery
//! - Restart count tracking
//! - Terminal state detection

use airssys_rt::{Actor, ActorContext, ActorLifecycle, ActorState, ErrorAction, Message};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkMessage {
    id: u32,
    should_fail: bool,
}

impl Message for WorkMessage {
    const MESSAGE_TYPE: &'static str = "work";
}

struct WorkerActor {
    processed: u32,
    max_restarts: u32,
}

#[derive(Debug)]
struct WorkerError {
    reason: String,
}

impl fmt::Display for WorkerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WorkerError: {}", self.reason)
    }
}

impl std::error::Error for WorkerError {}

#[async_trait]
impl Actor for WorkerActor {
    type Message = WorkMessage;
    type Error = WorkerError;

    async fn handle_message<B: airssys_rt::broker::MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        println!("[Worker] Processing work item {}", message.id);

        if message.should_fail {
            return Err(WorkerError {
                reason: format!("Work item {} failed", message.id),
            });
        }

        self.processed += 1;
        context.record_message();
        println!("[Worker] Successfully processed {} items", self.processed);

        Ok(())
    }

    async fn pre_start<B: airssys_rt::broker::MessageBroker<Self::Message>>(
        &mut self,
        _context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        println!("[Worker] Initializing...");
        self.processed = 0;
        Ok(())
    }

    async fn post_stop<B: airssys_rt::broker::MessageBroker<Self::Message>>(
        &mut self,
        _context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        println!(
            "[Worker] Shutting down (processed {} items)",
            self.processed
        );
        Ok(())
    }

    async fn on_error<B: airssys_rt::broker::MessageBroker<Self::Message>>(
        &mut self,
        error: Self::Error,
        _context: &mut ActorContext<Self::Message, B>,
    ) -> ErrorAction {
        println!("[Worker] Error: {error}");
        ErrorAction::Restart
    }
}

async fn simulate_actor_lifecycle<B: airssys_rt::broker::MessageBroker<WorkMessage>>(
    actor: &mut WorkerActor,
    context: &mut ActorContext<WorkMessage, B>,
    lifecycle: &mut ActorLifecycle,
    messages: Vec<WorkMessage>,
) {
    println!("\n--- Starting Actor Lifecycle Simulation ---");

    // Start actor
    println!("\n1. STARTING state");
    println!("   State: {:?}", lifecycle.state());
    if actor.pre_start(context).await.is_ok() {
        lifecycle.transition_to(ActorState::Running);
    }

    // Process messages
    println!("\n2. RUNNING state");
    println!("   State: {:?}", lifecycle.state());
    println!("   Is running: {}", lifecycle.is_running());

    for msg in messages {
        match actor.handle_message(msg.clone(), context).await {
            Ok(()) => {}
            Err(e) => {
                let action = actor.on_error(e, context).await;
                println!("   Supervision decision: {action:?}");

                if action == ErrorAction::Restart {
                    if lifecycle.restart_count() >= actor.max_restarts {
                        println!("   Max restarts reached, stopping actor");
                        lifecycle.transition_to(ActorState::Failed);
                        break;
                    }

                    println!("   Restarting actor...");
                    lifecycle.transition_to(ActorState::Stopping);
                    let _ = actor.post_stop(context).await;

                    lifecycle.transition_to(ActorState::Starting);
                    if actor.pre_start(context).await.is_ok() {
                        lifecycle.transition_to(ActorState::Running);
                    }
                }
            }
        }
    }

    // Stop actor
    if !lifecycle.is_terminal() {
        println!("\n3. STOPPING state");
        lifecycle.transition_to(ActorState::Stopping);
        println!("   State: {:?}", lifecycle.state());

        let _ = actor.post_stop(context).await;
        lifecycle.transition_to(ActorState::Stopped);
    }

    // Final state
    println!("\n4. Final state: {:?}", lifecycle.state());
    println!("   Restart count: {}", lifecycle.restart_count());
    println!("   Is terminal: {}", lifecycle.is_terminal());
}

#[tokio::main]
async fn main() {
    println!("=== Actor Lifecycle Example ===");

    let mut actor = WorkerActor {
        processed: 0,
        max_restarts: 3,
    };

    let address = airssys_rt::util::ActorAddress::named("worker");
    let broker = airssys_rt::broker::in_memory::InMemoryMessageBroker::<WorkMessage>::new();
    let mut context = ActorContext::new(address, broker);
    let mut lifecycle = ActorLifecycle::new();

    // Scenario 1: Normal operation
    println!("\n### Scenario 1: Normal Operation");
    let messages = vec![
        WorkMessage {
            id: 1,
            should_fail: false,
        },
        WorkMessage {
            id: 2,
            should_fail: false,
        },
        WorkMessage {
            id: 3,
            should_fail: false,
        },
    ];
    simulate_actor_lifecycle(&mut actor, &mut context, &mut lifecycle, messages).await;

    // Scenario 2: With failures and restarts
    println!("\n\n### Scenario 2: With Failures and Restarts");
    lifecycle = ActorLifecycle::new();
    let messages = vec![
        WorkMessage {
            id: 1,
            should_fail: false,
        },
        WorkMessage {
            id: 2,
            should_fail: true,
        }, // Fails, triggers restart
        WorkMessage {
            id: 3,
            should_fail: false,
        },
        WorkMessage {
            id: 4,
            should_fail: true,
        }, // Fails, triggers restart
        WorkMessage {
            id: 5,
            should_fail: false,
        },
    ];
    simulate_actor_lifecycle(&mut actor, &mut context, &mut lifecycle, messages).await;

    println!("\n=== Example Complete ===");
}
