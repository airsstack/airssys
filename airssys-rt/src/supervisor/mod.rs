//! Supervisor framework for fault-tolerant actor supervision.
//!
//! This module provides BEAM/Erlang-inspired supervision capabilities for building
//! fault-tolerant systems with automatic failure recovery. Supervisors manage child
//! processes (actors, tasks, or any entity implementing the `Child` trait) and
//! implement restart strategies to recover from failures automatically.
//!
//! # Core Concepts
//!
//! ## Supervision Trees
//!
//! Supervision trees are hierarchical structures where supervisors monitor children:
/// ```text
/// SupervisorTree (Root)
///      ├── Supervisor A (OneForOne)
///      │    ├── Actor 1
///      │    ├── Actor 2
///      │    └── Supervisor B (OneForAll)
///      │         ├── Actor 3
///      │         └── Actor 4
///      └── Supervisor C (RestForOne)
///           ├── DatabasePool
///           ├── CacheService
///           └── ApiHandler
/// ```
///
/// ## Fault Isolation
///
/// - **Failure Containment**: Failures are contained within supervisor boundaries
/// - **Hierarchical Recovery**: Failed supervisors escalate to parent supervisors
/// - **Independent Subtrees**: Different supervision strategies per subtree
/// - **Let It Crash**: Embrace failures, rely on supervision for recovery
///
/// # Architecture
///
/// The supervisor framework is built on several core concepts:
///
/// ## Child Trait
///
/// The [`Child`] trait defines the lifecycle interface for supervised entities.
/// Any entity implementing this trait can be placed under supervision. This includes:
/// - **Actors**: Must explicitly implement `Child` (no blanket implementation, ADR-RT-004)
/// - **Background Tasks**: Custom `Child` implementations for compute workers
/// - **I/O Handlers**: File watchers, network listeners, resource managers
/// - **System Services**: Monitoring daemons, connection pools, caches
///
/// ## Supervision Strategies
///
/// The framework supports three BEAM-inspired restart strategies:
///
/// ### OneForOne (Independent Children)
/// Restart only the failed child, other children continue unaffected.
/// - **Use case**: Independent workers, stateless services
/// - **Isolation**: Maximum failure isolation
/// - **Performance**: Minimal disruption on failure
///
/// ### OneForAll (Interdependent Children)
/// Restart all children when one fails (applications where all parts depend on each other).
/// - **Use case**: Tightly coupled services, shared state systems
/// - **Consistency**: Ensures all children start fresh together
/// - **Overhead**: All children restarted on any failure
///
/// ### RestForOne (Startup Dependencies)
/// Restart the failed child and all children started after it.
/// - **Use case**: Pipeline systems, dependent startup sequences
/// - **Dependencies**: Maintains startup order invariants
/// - **Balance**: More selective than OneForAll, less isolated than OneForOne
///
/// ## Restart Policies
///
/// Children can be configured with different restart policies:
///
/// - **Permanent**: Always restart the child (default for critical services)
/// - **Transient**: Restart only if the child exits abnormally (temporary workers)
/// - **Temporary**: Never restart the child (one-off tasks)
///
/// ## Shutdown Policies
///
/// Control how children are stopped:
///
/// - **Graceful(Duration)**: Wait for child to stop within timeout (default 5s)
/// - **Immediate**: Kill child immediately without cleanup
///
/// # Performance Characteristics
///
/// - **Supervisor overhead**: <100ns per health check cycle
/// - **Restart latency**: ~1-5ms (depends on child initialization)
/// - **Health monitoring**: Configurable interval (default 1s)
/// - **Memory per child**: ~128 bytes (ChildSpec + state tracking)
/// - **Concurrent restarts**: Non-blocking async operations
///
/// # Quick Start Examples
///
/// ## Example 1: Basic Supervisor with OneForOne Strategy
///
/// ```rust,ignore
/// use airssys_rt::supervisor::*;
///
/// // Define a simple worker that implements Child
/// struct Worker { id: u32 }
///
/// #[async_trait::async_trait]
/// impl Child for Worker {
///     type Error = std::io::Error;
///     
///     async fn start(&mut self) -> Result<(), Self::Error> {
///         println!("Worker {} starting", self.id);
///         Ok(())
///     }
///     
///     async fn stop(&mut self, _timeout: Duration) -> Result<(), Self::Error> {
///         println!("Worker {} stopping", self.id);
///         Ok(())
///     }
/// }
///
/// // Create supervisor with OneForOne strategy
/// let supervisor = SupervisorNode::builder()
///     .with_strategy(OneForOne)
///     .build();
///
/// // Add children
/// supervisor.add_child(ChildSpec::new(|| Worker { id: 1 })).await?;
/// supervisor.add_child(ChildSpec::new(|| Worker { id: 2 })).await?;
/// // If Worker 1 fails, only Worker 1 restarts
/// ```
///
/// ## Example 2: Actor Supervision
///
/// ```rust,ignore
/// use airssys_rt::{Actor, ActorContext, supervisor::*};
///
/// // 1. Define your actor
/// struct CounterActor { count: u64 }
///
/// #[async_trait::async_trait]
/// impl Actor for CounterActor {
///     type Message = CounterMsg;
///     type Error = CounterError;
///     
///     async fn handle_message<B: MessageBroker<Self::Message>>(
///         &mut self,
///         msg: Self::Message,
///         _ctx: &mut ActorContext<Self::Message, B>,
///     ) -> Result<(), Self::Error> {
///         self.count += msg.delta;
///         Ok(())
///     }
/// }
///
/// // 2. Explicitly implement Child for supervision
/// #[async_trait::async_trait]
/// impl Child for CounterActor {
///     type Error = CounterError;
///     
///     async fn start(&mut self) -> Result<(), Self::Error> {
///         println!("Counter starting");
///         Ok(())
///     }
///     
///     async fn stop(&mut self, _timeout: Duration) -> Result<(), Self::Error> {
///         println!("Counter stopping");
///         Ok(())
///     }
/// }
///
/// // 3. Supervise the actor
/// let supervisor = SupervisorNode::builder()
///     .with_strategy(OneForOne)
///     .with_restart_policy(RestartPolicy::Permanent)
///     .build();
///
/// supervisor.add_child(ChildSpec::new(|| CounterActor { count: 0 })).await?;
/// ```
///
/// ## Example 3: Health Monitoring
///
/// ```rust,ignore
/// use airssys_rt::supervisor::*;
///
/// struct HealthyService { error_count: u32 }
///
/// #[async_trait::async_trait]
/// impl Child for HealthyService {
///     type Error = std::io::Error;
///     
///     async fn start(&mut self) -> Result<(), Self::Error> {
///         self.error_count = 0;
///         Ok(())
///     }
///     
///     async fn stop(&mut self, _timeout: Duration) -> Result<(), Self::Error> {
///         Ok(())
///     }
///     
///     // Custom health check logic
///     async fn health_check(&self) -> ChildHealth {
///         if self.error_count > 10 {
///             ChildHealth::Failed(format!("Too many errors: {}", self.error_count))
///         } else if self.error_count > 5 {
///             ChildHealth::Degraded(format!("Elevated errors: {}", self.error_count))
///         } else {
///             ChildHealth::Healthy
///         }
///     }
/// }
///
/// // Enable automatic health monitoring
/// let supervisor = SupervisorNode::builder()
///     .with_strategy(OneForOne)
///     .with_health_check_interval(Duration::from_secs(1))
///     .build();
/// ```
// Module declarations
pub mod backoff;
pub mod builder;
pub mod error;
pub mod health_monitor;
pub mod node;
pub mod strategy;
pub mod traits;
pub mod tree;
pub mod types;

// Re-exports for convenient access
pub use backoff::RestartBackoff;
pub use builder::{
    SingleChildBuilder, DEFAULT_RESTART_POLICY, DEFAULT_SHUTDOWN_POLICY, DEFAULT_SHUTDOWN_TIMEOUT,
    DEFAULT_START_TIMEOUT,
};
pub use error::SupervisorError;
pub use node::{ChildHandle, HealthConfig, SupervisorNode};
pub use strategy::{should_restart, should_restart_any, OneForAll, OneForOne, RestForOne};
pub use traits::{Child, SupervisionStrategy, Supervisor};
pub use tree::{SupervisorId, SupervisorTree};
pub use types::{
    ChildHealth, ChildId, ChildSpec, ChildState, RestartPolicy, ShutdownPolicy, StrategyContext,
    SupervisionDecision,
};
