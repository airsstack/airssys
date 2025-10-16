//! Actor Lifecycle and State Transitions
//!
//! Demonstrates comprehensive actor lifecycle management with state machine transitions,
//! error recovery, restart strategies, and terminal state handling.
//!
//! # What You'll Learn
//!
//! - **Lifecycle States**: Understanding the actor state machine (Created → Starting → Running → Stopping → Stopped/Failed)
//! - **State Transitions**: How to safely transition between lifecycle states
//! - **Error Recovery**: Implementing restart strategies with ErrorAction::Restart
//! - **Restart Limits**: Tracking and enforcing maximum restart counts
//! - **Terminal States**: Detecting and handling terminal states (Stopped, Failed)
//! - **Supervision Integration**: How lifecycle integrates with supervisor restart policies
//!
//! # Key Concepts
//!
//! ## ActorLifecycle State Machine
//!
//! ```text
//! Created ─→ Starting ─→ Running ─→ Stopping ─→ Stopped
//!               ↓          ↓                      
//!            Failed ←──────┘                      
//!               ↑
//!               └─── (max restarts exceeded)
//! ```
//!
//! - **Created**: Initial state after construction
//! - **Starting**: Actor is initializing (pre_start hook)
//! - **Running**: Actor is processing messages (handle_message)
//! - **Stopping**: Actor is shutting down (post_stop hook)
//! - **Stopped**: Normal termination (terminal state)
//! - **Failed**: Error termination (terminal state)
//!
//! ## ErrorAction and Restart Strategy
//!
//! When `handle_message()` returns an error, the actor's `on_error()` hook decides:
//! - `ErrorAction::Restart`: Transition to Stopping → Starting → Running
//! - `ErrorAction::Stop`: Transition to Stopping → Stopped
//! - `ErrorAction::Escalate`: Pass error to supervisor
//!
//! ## Restart Count Tracking
//!
//! `ActorLifecycle::restart_count()` tracks restarts to prevent infinite restart loops.
//! Common pattern: enforce max_restarts limit, transition to Failed if exceeded.
//!
//! # Run This Example
//!
//! ```bash
//! cargo run --example actor_lifecycle
//! ```
//!
//! # Expected Output
//!
//! ```text
//! === Actor Lifecycle Example ===
//!
//! ### Scenario 1: Normal Operation
//! [Worker] Initializing...
//! [Worker] Processing work item 1
//! [Worker] Successfully processed 1 items
//! ...
//! Final state: Stopped
//! Restart count: 0
//!
//! ### Scenario 2: With Failures and Restarts
//! [Worker] Error: WorkerError: Work item 2 failed
//! Supervision decision: Restart
//! Restarting actor...
//! [Worker] Shutting down (processed 1 items)
//! [Worker] Initializing...
//! ...
//! Final state: Stopped
//! Restart count: 2
//! ```
//!
//! # See Also
//!
//! - [`supervisor_basic.rs`] - Supervisor-managed actor lifecycle
//! - [`supervisor_strategies.rs`] - OneForOne, OneForAll, RestForOne restart strategies
//! - [`actor_basic.rs`] - Core Actor trait implementation
//! - [User Guide: Actor Development](../docs/src/guides/actor-development.md)

use airssys_rt::{Actor, ActorContext, ActorLifecycle, ActorState, ErrorAction, Message};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;

// =============================================================================
// Message and Actor Definitions
// =============================================================================

/// Work message that can optionally fail for testing error handling.
///
/// The `should_fail` flag simulates transient failures that trigger
/// the actor's error recovery and restart logic.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkMessage {
    id: u32,
    should_fail: bool,
}

impl Message for WorkMessage {
    const MESSAGE_TYPE: &'static str = "work";
}

/// Worker actor that processes work items and tracks restart behavior.
///
/// Demonstrates:
/// - State reset on restart (processed count resets in pre_start)
/// - Restart limit enforcement (max_restarts)
/// - Error recovery strategy (ErrorAction::Restart)
struct WorkerActor {
    processed: u32,    // Counter reset on each restart
    max_restarts: u32, // Restart limit before transitioning to Failed state
}

/// Domain-specific error for worker operations.
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

    /// Process work items, simulating failures based on message flag.
    ///
    /// Returns error if `should_fail` is true, triggering the actor's
    /// error recovery logic (see on_error method).
    async fn handle_message<B: airssys_rt::broker::MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        println!("[Worker] Processing work item {}", message.id);

        // Simulate transient failure
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

    /// Initialize actor state when entering Running state.
    ///
    /// CRITICAL: This is called on EVERY restart, not just initial start.
    /// Reset state here to ensure clean restart behavior.
    async fn pre_start<B: airssys_rt::broker::MessageBroker<Self::Message>>(
        &mut self,
        _context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        println!("[Worker] Initializing...");
        // Reset processed counter on each restart
        self.processed = 0;
        Ok(())
    }

    /// Clean up actor resources when leaving Running state.
    ///
    /// Called when transitioning to Stopped state (both normal shutdown
    /// and pre-restart cleanup).
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

    /// Decide recovery action when message processing fails.
    ///
    /// This example always returns ErrorAction::Restart, but real actors
    /// might escalate to supervisor for certain error types.
    async fn on_error<B: airssys_rt::broker::MessageBroker<Self::Message>>(
        &mut self,
        error: Self::Error,
        _context: &mut ActorContext<Self::Message, B>,
    ) -> ErrorAction {
        println!("[Worker] Error: {error}");
        ErrorAction::Restart // Always attempt restart
    }
}

// =============================================================================
// Lifecycle Simulation Helper
// =============================================================================

/// Simulates complete actor lifecycle with error handling and restarts.
///
/// This function demonstrates the full lifecycle state machine:
/// 1. Starting → Running (pre_start)
/// 2. Running → Error → Stopping → Starting → Running (restart cycle)
/// 3. Running → Stopping → Stopped (normal shutdown)
/// 4. Enforce max_restarts limit → Failed (error termination)
///
/// # Parameters
///
/// - `actor`: The WorkerActor instance to manage
/// - `context`: Actor context for messaging
/// - `lifecycle`: Lifecycle state tracker
/// - `messages`: Messages to process during Running state
async fn simulate_actor_lifecycle<B: airssys_rt::broker::MessageBroker<WorkMessage>>(
    actor: &mut WorkerActor,
    context: &mut ActorContext<WorkMessage, B>,
    lifecycle: &mut ActorLifecycle,
    messages: Vec<WorkMessage>,
) {
    println!("\n--- Starting Actor Lifecycle Simulation ---");

    // ==========================================================================
    // Step 1: STARTING state (pre_start hook)
    // ==========================================================================
    println!("\n1. STARTING state");
    println!("   State: {:?}", lifecycle.state());
    if actor.pre_start(context).await.is_ok() {
        lifecycle.transition_to(ActorState::Running);
    }

    // ==========================================================================
    // Step 2: RUNNING state (process messages with error handling)
    // ==========================================================================
    println!("\n2. RUNNING state");
    println!("   State: {:?}", lifecycle.state());
    println!("   Is running: {}", lifecycle.is_running());

    for msg in messages {
        match actor.handle_message(msg.clone(), context).await {
            Ok(()) => {
                // Message processed successfully
            }
            Err(e) => {
                // Error occurred - invoke error handler
                let action = actor.on_error(e, context).await;
                println!("   Supervision decision: {action:?}");

                // Handle ErrorAction::Restart with max_restarts limit
                if action == ErrorAction::Restart {
                    if lifecycle.restart_count() >= actor.max_restarts {
                        println!("   Max restarts reached, stopping actor");
                        lifecycle.transition_to(ActorState::Failed);
                        break;
                    }

                    // Perform restart cycle: Stopping → Starting → Running
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

    // ==========================================================================
    // Step 3: STOPPING → STOPPED state (normal shutdown)
    // ==========================================================================
    if !lifecycle.is_terminal() {
        println!("\n3. STOPPING state");
        lifecycle.transition_to(ActorState::Stopping);
        println!("   State: {:?}", lifecycle.state());

        let _ = actor.post_stop(context).await;
        lifecycle.transition_to(ActorState::Stopped);
    }

    // ==========================================================================
    // Step 4: Display final state and statistics
    // ==========================================================================
    println!("\n4. Final state: {:?}", lifecycle.state());
    println!("   Restart count: {}", lifecycle.restart_count());
    println!("   Is terminal: {}", lifecycle.is_terminal());
}

// =============================================================================
// Main Example - Two Scenarios
// =============================================================================

#[tokio::main]
async fn main() {
    println!("=== Actor Lifecycle Example ===");

    // Create actor instance with restart limit
    let mut actor = WorkerActor {
        processed: 0,
        max_restarts: 3, // Allow up to 3 restarts before transitioning to Failed
    };

    // Create actor context and lifecycle tracker
    let address = airssys_rt::util::ActorAddress::named("worker");
    let broker = airssys_rt::broker::in_memory::InMemoryMessageBroker::<WorkMessage>::new();
    let mut context = ActorContext::new(address, broker);
    let mut lifecycle = ActorLifecycle::new();

    // ==========================================================================
    // Scenario 1: Normal Operation (no failures)
    // ==========================================================================
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

    // ==========================================================================
    // Scenario 2: With Failures and Restarts
    // ==========================================================================
    println!("\n\n### Scenario 2: With Failures and Restarts");
    lifecycle = ActorLifecycle::new(); // Reset lifecycle for new scenario
    let messages = vec![
        WorkMessage {
            id: 1,
            should_fail: false,
        },
        WorkMessage {
            id: 2,
            should_fail: true, // ← Triggers restart #1
        },
        WorkMessage {
            id: 3,
            should_fail: false,
        },
        WorkMessage {
            id: 4,
            should_fail: true, // ← Triggers restart #2
        },
        WorkMessage {
            id: 5,
            should_fail: false,
        },
    ];
    simulate_actor_lifecycle(&mut actor, &mut context, &mut lifecycle, messages).await;

    println!("\n=== Example Complete ===");
}
