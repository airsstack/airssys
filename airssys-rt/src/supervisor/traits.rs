//! Core traits for the supervisor framework.
//!
//! This module defines the fundamental traits that enable supervision:
//! - `Child`: Lifecycle interface for supervised entities
//! - `Supervisor`: Interface for supervisor implementation
//! - `SupervisionStrategy`: Interface for restart strategy implementation
//!
//! # Architecture
//!
//! The Child trait is intentionally separate from the Actor trait (ADR-RT-004)
//! to enable supervision of ANY entity type, not just actors. Actors that need
//! to be supervised must explicitly implement the Child trait, maintaining clean
//! separation between message-passing behavior and supervision lifecycle.

// Layer 1: Standard library imports
use std::error::Error;
use std::time::Duration;

// Layer 2: Third-party crate imports
use async_trait::async_trait;

// Layer 3: Internal module imports
use super::error::SupervisorError;
use super::types::{ChildHealth, ChildId, ChildSpec, StrategyContext, SupervisionDecision};

/// Child trait for entities that can be supervised.
///
/// Any entity implementing this trait can be placed under supervisor management,
/// enabling fault-tolerant hierarchical supervision trees. This trait is
/// intentionally separate and independent from `Actor` to allow supervision of
/// diverse entity types including actors, background tasks, I/O handlers, and
/// system services.
///
/// # Design Philosophy
///
/// - **Universal Interface**: Any process-like entity can be supervised
/// - **BEAM Alignment**: Matches Erlang/OTP supervisor behavior model
/// - **True Independence**: No coupling to Actor, MessageBroker, or ActorContext
/// - **Composability**: Mix actors and non-actors in supervision trees
///
/// # Lifecycle Methods
///
/// - `start()`: Initialize and start the child process (REQUIRED)
/// - `stop()`: Gracefully shutdown with timeout (REQUIRED)
/// - `health_check()`: Report health status (OPTIONAL, default: Healthy)
///
/// # Actor Supervision
///
/// Actors are **NOT** automatically Children. To supervise an actor, you must
/// explicitly implement the Child trait for that actor type. This maintains
/// clean separation between message-passing behavior (Actor) and supervision
/// lifecycle (Child).
///
/// # Examples
///
/// ## Example 1: Actor with Explicit Child Implementation
///
/// ```rust,no_run
/// use airssys_rt::{Actor, ActorContext, supervisor::{Child, ChildHealth}};
/// use async_trait::async_trait;
/// use std::time::Duration;
///
/// // Define an actor
/// struct CounterActor {
///     count: u32,
/// }
///
/// #[derive(Debug, Clone)]
/// struct CounterMsg { delta: u32 }
/// impl airssys_rt::Message for CounterMsg {
///     const MESSAGE_TYPE: &'static str = "counter";
/// }
///
/// #[derive(Debug)]
/// struct CounterError;
/// impl std::fmt::Display for CounterError {
///     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
///         write!(f, "Counter error")
///     }
/// }
/// impl std::error::Error for CounterError {}
///
/// #[async_trait]
/// impl Actor for CounterActor {
///     type Message = CounterMsg;
///     type Error = CounterError;
///     
///     async fn handle_message<B: airssys_rt::broker::MessageBroker<Self::Message>>(
///         &mut self,
///         msg: Self::Message,
///         _ctx: &mut ActorContext<Self::Message, B>,
///     ) -> Result<(), Self::Error> {
///         self.count += msg.delta;
///         Ok(())
///     }
/// }
///
/// // Explicitly implement Child for supervision
/// #[async_trait]
/// impl Child for CounterActor {
///     type Error = CounterError;
///     
///     async fn start(&mut self) -> Result<(), Self::Error> {
///         // Initialize actor for supervision
///         println!("Counter starting");
///         Ok(())
///     }
///     
///     async fn stop(&mut self, _timeout: Duration) -> Result<(), Self::Error> {
///         // Cleanup when supervised actor stops
///         println!("Counter stopping");
///         Ok(())
///     }
/// }
///
/// // ✅ CounterActor can now be supervised!
/// ```
///
/// ## Example 2: Non-Actor Background Task
///
/// ```rust
/// use airssys_rt::supervisor::{Child, ChildHealth};
/// use async_trait::async_trait;
/// use std::time::Duration;
///
/// struct BackgroundWorker {
///     name: String,
///     running: bool,
/// }
///
/// #[derive(Debug)]
/// struct WorkerError;
///
/// impl std::fmt::Display for WorkerError {
///     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
///         write!(f, "Worker error")
///     }
/// }
///
/// impl std::error::Error for WorkerError {}
///
/// #[async_trait]
/// impl Child for BackgroundWorker {
///     type Error = WorkerError;
///     
///     async fn start(&mut self) -> Result<(), Self::Error> {
///         println!("[{}] Starting worker", self.name);
///         self.running = true;
///         Ok(())
///     }
///     
///     async fn stop(&mut self, _timeout: Duration) -> Result<(), Self::Error> {
///         println!("[{}] Stopping worker", self.name);
///         self.running = false;
///         Ok(())
///     }
///     
///     async fn health_check(&self) -> ChildHealth {
///         if self.running {
///             ChildHealth::Healthy
///         } else {
///             ChildHealth::Failed("Worker not running".into())
///         }
///     }
/// }
/// ```
#[async_trait]
pub trait Child: Send + Sync + 'static {
    /// Error type for child lifecycle operations
    type Error: Error + Send + Sync + 'static;

    /// Start the child process.
    ///
    /// This method should initialize all resources and begin operation.
    /// Implementations should be idempotent where possible - calling
    /// start() on an already-started child should either succeed or
    /// return an appropriate error.
    ///
    /// # Errors
    ///
    /// Returns error if initialization fails. The supervisor will handle
    /// the error according to the configured `RestartPolicy`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use airssys_rt::supervisor::Child;
    /// # use async_trait::async_trait;
    /// # use std::time::Duration;
    /// # struct MyChild { initialized: bool }
    /// # #[derive(Debug)]
    /// # struct MyError;
    /// # impl std::fmt::Display for MyError {
    /// #     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { Ok(()) }
    /// # }
    /// # impl std::error::Error for MyError {}
    /// #[async_trait]
    /// impl Child for MyChild {
    ///     type Error = MyError;
    ///     
    ///     async fn start(&mut self) -> Result<(), Self::Error> {
    ///         // Initialize resources
    ///         self.initialized = true;
    ///         println!("Child started");
    ///         Ok(())
    ///     }
    /// #   async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
    /// }
    /// ```
    async fn start(&mut self) -> Result<(), Self::Error>;

    /// Stop the child process gracefully.
    ///
    /// This method should perform graceful shutdown within the given timeout.
    /// After timeout expires, the supervisor may forcefully terminate the
    /// child depending on the `ShutdownPolicy` configuration.
    ///
    /// # Parameters
    ///
    /// - `timeout`: Maximum time to wait for graceful shutdown
    ///
    /// # Errors
    ///
    /// Returns error if shutdown fails or times out. Errors are logged
    /// but typically don't affect supervision decisions since the child
    /// is being stopped anyway.
    ///
    /// # Examples
    ///
    /// See the strategy examples for complete implementations:
    /// - [`crate::supervisor::OneForOne`]
    /// - [`crate::supervisor::OneForAll`]
    /// - [`crate::supervisor::RestForOne`]
    async fn stop(&mut self, timeout: Duration) -> Result<(), Self::Error>;

    /// Check the health status of the child.
    ///
    /// Used by supervisors to detect degraded or failing children before
    /// they completely fail. This enables proactive restart or recovery
    /// strategies.
    ///
    /// # Default Implementation
    ///
    /// The default implementation returns `ChildHealth::Healthy`. Override
    /// this method to provide custom health checking logic.
    ///
    /// # Returns
    ///
    /// - `ChildHealth::Healthy`: Child is operating normally
    /// - `ChildHealth::Degraded(reason)`: Child is operational but degraded
    /// - `ChildHealth::Failed(reason)`: Child has failed and needs restart
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use airssys_rt::supervisor::{Child, ChildHealth};
    /// # use async_trait::async_trait;
    /// # use std::time::Duration;
    /// # struct MyChild { error_count: u32, error_threshold: u32 }
    /// # #[derive(Debug)]
    /// # struct MyError;
    /// # impl std::fmt::Display for MyError {
    /// #     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { Ok(()) }
    /// # }
    /// # impl std::error::Error for MyError {}
    /// #[async_trait]
    /// impl Child for MyChild {
    ///     type Error = MyError;
    /// #   async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
    /// #   async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
    ///     
    ///     async fn health_check(&self) -> ChildHealth {
    ///         if self.error_count > self.error_threshold {
    ///             ChildHealth::Failed(format!("Error count: {}", self.error_count))
    ///         } else if self.error_count > self.error_threshold / 2 {
    ///             ChildHealth::Degraded(format!("Elevated errors: {}", self.error_count))
    ///         } else {
    ///             ChildHealth::Healthy
    ///         }
    ///     }
    /// }
    /// ```
    async fn health_check(&self) -> ChildHealth {
        ChildHealth::Healthy
    }
}

// NOTE: No blanket implementation for Actor → Child
//
// Child and Actor are intentionally independent traits. Actors that need
// to be supervised must explicitly implement the Child trait. This maintains
// true separation of concerns:
//
// - Actor trait: Message handling and actor-specific behavior
// - Child trait: Lifecycle management for supervision
//
// This design enables supervision of ANY entity (actors, tasks, services, etc.)
// without coupling the Child trait to Actor-specific concepts like ActorContext
// or MessageBroker.
//
// Example: Explicit Child implementation for an Actor
// ```rust
// #[async_trait]
// impl Child for MyActor {
//     type Error = MyError;
//
//     async fn start(&mut self) -> Result<(), Self::Error> {
//         // Custom initialization logic for supervision
//         self.initialize_resources()
//     }
//
//     async fn stop(&mut self, timeout: Duration) -> Result<(), Self::Error> {
//         // Custom cleanup logic for supervision
//         self.cleanup_resources(timeout)
//     }
// }
// ```

/// Supervisor trait for managing supervised children.
///
/// Supervisors implement fault-tolerant child process management using
/// supervision strategies and restart policies. This trait will be
/// fully implemented in RT-TASK-007 Phase 3.
///
/// # Type Parameters
///
/// - `C`: Child type implementing the `Child` trait
/// - `S`: Supervision strategy type implementing `SupervisionStrategy`
/// - `M`: Monitor type for supervision events
///
/// # Examples
///
/// ```rust,ignore
/// // Will be implemented in Phase 3
/// use airssys_rt::supervisor::{Supervisor, SupervisorNode, OneForOne};
///
/// let supervisor = SupervisorNode::<OneForOne, MyChild, MyMonitor>::new(
///     OneForOne,
///     monitor,
/// );
/// ```
#[async_trait]
pub trait Supervisor: Send + Sync + 'static {
    /// Child type managed by this supervisor
    type Child: Child;

    /// Start a child process from a specification.
    ///
    /// # Errors
    ///
    /// Returns error if child factory fails or child startup fails.
    async fn start_child<F>(
        &mut self,
        spec: ChildSpec<Self::Child, F>,
    ) -> Result<ChildId, SupervisorError>
    where
        F: Fn() -> Self::Child + Send + Sync + 'static;

    /// Stop a specific child process.
    ///
    /// # Errors
    ///
    /// Returns error if child is not found or shutdown fails.
    async fn stop_child(&mut self, id: &ChildId) -> Result<(), SupervisorError>;

    /// Restart a specific child process.
    ///
    /// # Errors
    ///
    /// Returns error if restart fails or restart limits are exceeded.
    async fn restart_child(&mut self, id: &ChildId) -> Result<(), SupervisorError>;

    /// Handle a child error and return supervision decision.
    ///
    /// Delegates to the configured supervision strategy to determine
    /// the appropriate action (restart child, restart all, escalate, etc.).
    async fn handle_child_error(
        &mut self,
        id: &ChildId,
        error: Box<dyn Error + Send + Sync>,
    ) -> SupervisionDecision;
}

/// Supervision strategy trait.
///
/// Defines how a supervisor should respond to child failures. Implementations
/// of this trait determine restart behavior (OneForOne, OneForAll, RestForOne).
///
/// Each strategy implements a decision-making algorithm that determines which
/// children should be affected when a child fails.
///
/// # Context-Based Decision Making
///
/// The strategy receives a `StrategyContext` enum that provides type-safe context
/// for different scenarios (single failure, manual restart, shutdown). This design:
///
/// - Prevents invalid parameter combinations
/// - Makes extension easy (new variants don't break existing code)
/// - Self-documents the purpose of each decision scenario
///
/// # Examples
///
/// ```rust
/// use airssys_rt::supervisor::{SupervisionStrategy, SupervisionDecision, ChildId, StrategyContext};
///
/// # struct OneForOne;
/// # impl SupervisionStrategy for OneForOne {
/// #     fn determine_decision(context: StrategyContext) -> SupervisionDecision {
/// #         match context {
/// #             StrategyContext::SingleFailure { failed_child_id, .. } => {
/// #                 SupervisionDecision::RestartChild(failed_child_id)
/// #             }
/// #             _ => SupervisionDecision::StopAll,
/// #         }
/// #     }
/// # }
/// // OneForOne strategy: restart only the failed child
/// let failed_id = ChildId::new();
/// let context = StrategyContext::SingleFailure {
///     failed_child_id: failed_id.clone(),
///     all_child_ids: vec![failed_id.clone()],
/// };
/// let decision = OneForOne::determine_decision(context);
/// ```
pub trait SupervisionStrategy: Send + Sync + 'static {
    /// Determines what action to take based on the supervision context.
    ///
    /// # Parameters
    ///
    /// - `context`: The supervision context (failure, manual restart, or shutdown)
    ///
    /// # Returns
    ///
    /// A `SupervisionDecision` indicating what action to take
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::supervisor::{StrategyContext, SupervisionDecision, ChildId};
    ///
    /// # struct MyStrategy;
    /// # impl airssys_rt::supervisor::SupervisionStrategy for MyStrategy {
    /// #     fn determine_decision(context: StrategyContext) -> SupervisionDecision {
    /// match context {
    ///     StrategyContext::SingleFailure { failed_child_id, all_child_ids } => {
    ///         // Strategy-specific logic here
    /// #         SupervisionDecision::RestartChild(failed_child_id)
    ///     }
    ///     StrategyContext::ManualRestart { child_id } => {
    ///         // Handle manual restart
    /// #         SupervisionDecision::RestartChild(child_id)
    ///     }
    ///     StrategyContext::Shutdown { all_child_ids } => {
    ///         // Handle shutdown
    /// #         SupervisionDecision::StopAll
    ///     }
    /// }
    /// #     }
    /// # }
    /// ```
    fn determine_decision(context: StrategyContext) -> SupervisionDecision;
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::actor::Actor;
    use crate::{ActorContext, Message};
    use std::fmt;

    // Test child implementation
    struct TestChild {
        started: bool,
        stopped: bool,
    }

    #[derive(Debug)]
    struct TestError;

    impl fmt::Display for TestError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Test error")
        }
    }

    impl Error for TestError {}

    #[async_trait]
    impl Child for TestChild {
        type Error = TestError;

        async fn start(&mut self) -> Result<(), Self::Error> {
            self.started = true;
            Ok(())
        }

        async fn stop(&mut self, _timeout: Duration) -> Result<(), Self::Error> {
            self.stopped = true;
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_child_lifecycle() {
        let mut child = TestChild {
            started: false,
            stopped: false,
        };

        // Test start
        child.start().await.unwrap();
        assert!(child.started);

        // Test health check (default)
        let health = child.health_check().await;
        assert!(health.is_healthy());

        // Test stop
        child.stop(Duration::from_secs(1)).await.unwrap();
        assert!(child.stopped);
    }

    // Test actor implementation
    struct TestActor {
        count: u32,
    }

    #[derive(Debug, Clone)]
    struct TestMessage;

    impl Message for TestMessage {
        const MESSAGE_TYPE: &'static str = "test";
    }

    #[derive(Debug)]
    struct TestActorError;

    impl fmt::Display for TestActorError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Test actor error")
        }
    }

    impl Error for TestActorError {}

    #[async_trait]
    impl Actor for TestActor {
        type Message = TestMessage;
        type Error = TestActorError;

        async fn handle_message<B: crate::broker::MessageBroker<Self::Message>>(
            &mut self,
            _msg: Self::Message,
            _ctx: &mut ActorContext<Self::Message, B>,
        ) -> Result<(), Self::Error> {
            self.count += 1;
            Ok(())
        }
    }

    // Explicitly implement Child for TestActor (no automatic blanket impl)
    #[async_trait]
    impl Child for TestActor {
        type Error = TestActorError;

        async fn start(&mut self) -> Result<(), Self::Error> {
            // TestActor specific initialization
            self.count = 0;
            Ok(())
        }

        async fn stop(&mut self, _timeout: Duration) -> Result<(), Self::Error> {
            // TestActor specific cleanup
            Ok(())
        }

        async fn health_check(&self) -> ChildHealth {
            ChildHealth::Healthy
        }
    }

    #[tokio::test]
    async fn test_actor_child_explicit_impl() {
        let mut actor = TestActor { count: 0 };

        // Actor implements Child explicitly
        actor.start().await.unwrap();

        let health = actor.health_check().await;
        assert!(health.is_healthy());

        actor.stop(Duration::from_secs(1)).await.unwrap();
    }

    #[test]
    fn test_child_trait_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<TestChild>();
    }
}
