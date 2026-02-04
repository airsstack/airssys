//! SupervisorNode implementation for hierarchical supervision.
//!
//! This module provides the core supervisor implementation with generic strategy,
//! child, and monitor type parameters. SupervisorNode manages child lifecycle,
//! applies restart strategies, and coordinates with monitoring infrastructure.

// Layer 1: Standard library imports
use std::collections::HashMap;
use std::error::Error as StdError;
use std::sync::Arc;
use std::time::Duration;

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use chrono::Utc;
use tokio::sync::Mutex;
use tokio::time::timeout;
use uuid::Uuid;

// Layer 3: Internal module imports
use super::backoff::RestartBackoff;
use super::builder::{ChildrenBatchBuilder, SingleChildBuilder};
use super::error::SupervisorError;
use super::health_monitor::spawn_health_monitor;
use super::strategy::should_restart;
use super::traits::{Child, SupervisionStrategy, Supervisor};
use super::types::{
    ChildHealth, ChildId, ChildSpec, ChildState, RestartPolicy, ShutdownPolicy, StrategyContext,
    SupervisionDecision,
};
use crate::monitoring::{Monitor, SupervisionEvent, SupervisionEventKind};

/// A supervisor with automatic background health monitoring enabled.
///
/// This wrapper type is returned by `with_automatic_health_monitoring()` and
/// manages the lifecycle of the background health checking task. The task
/// will automatically stop when this wrapper is dropped.
///
/// # Type Parameters
///
/// - `S`: Supervision strategy type
/// - `C`: Child type implementing the `Child` trait
/// - `M`: Monitor type for supervision events
///
/// # Usage
///
/// Access the underlying supervisor through the `supervisor` field:
///
/// ```rust,ignore
/// let monitored = supervisor.with_automatic_health_monitoring(...);
/// monitored.supervisor.lock().await.start_child(...).await?;
/// ```
pub struct MonitoredSupervisor<S, C, M>
where
    S: SupervisionStrategy,
    C: Child,
    M: Monitor<SupervisionEvent>,
{
    /// The wrapped supervisor instance
    pub supervisor: Arc<Mutex<SupervisorNode<S, C, M>>>,

    /// Background health monitoring task handle (kept alive)
    _task_handle: tokio::task::JoinHandle<()>,

    /// Shutdown signal sender (kept alive to prevent premature shutdown)
    _shutdown_tx: tokio::sync::oneshot::Sender<()>,
}

/// Child handle with lifecycle state and restart tracking.
///
/// Manages the runtime state of a supervised child including current state,
/// restart history, and lifecycle timestamps.
///
/// # Type Parameters
///
/// - `C`: Child type implementing the `Child` trait
#[derive(Debug)]
pub struct ChildHandle<C>
where
    C: Child,
{
    /// The child instance
    child: C,

    /// Current lifecycle state
    state: ChildState,

    /// Restart policy for this child
    restart_policy: RestartPolicy,

    /// Shutdown policy for this child
    shutdown_policy: ShutdownPolicy,

    /// Number of restarts performed
    restart_count: u32,

    /// Timestamp of last restart
    last_restart: chrono::DateTime<chrono::Utc>,

    /// Initial start timestamp
    #[allow(dead_code)]
    start_time: chrono::DateTime<chrono::Utc>,
}

impl<C> ChildHandle<C>
where
    C: Child,
{
    /// Creates a new child handle.
    pub fn new(child: C, restart_policy: RestartPolicy, shutdown_policy: ShutdownPolicy) -> Self {
        let now = Utc::now();
        Self {
            child,
            state: ChildState::Starting,
            restart_policy,
            shutdown_policy,
            restart_count: 0,
            last_restart: now,
            start_time: now,
        }
    }

    /// Returns the current child state.
    pub fn state(&self) -> &ChildState {
        &self.state
    }

    /// Sets the child state.
    pub fn set_state(&mut self, state: ChildState) {
        self.state = state;
    }

    /// Returns the restart policy.
    pub fn restart_policy(&self) -> &RestartPolicy {
        &self.restart_policy
    }

    /// Returns the shutdown policy.
    pub fn shutdown_policy(&self) -> &ShutdownPolicy {
        &self.shutdown_policy
    }

    /// Returns the restart count.
    pub fn restart_count(&self) -> u32 {
        self.restart_count
    }

    /// Increments the restart count and updates timestamp.
    pub fn record_restart(&mut self) {
        self.restart_count += 1;
        self.last_restart = Utc::now();
    }

    /// Returns mutable reference to the child.
    pub fn child_mut(&mut self) -> &mut C {
        &mut self.child
    }

    /// Returns reference to the child.
    pub fn child(&self) -> &C {
        &self.child
    }
}

/// Generic supervisor node with strategy, child, and monitor type parameters.
///
/// SupervisorNode manages a collection of children with a specific restart strategy
/// and integrates with the monitoring infrastructure to track supervision events.
///
/// # Type Parameters
///
/// - `S`: Supervision strategy (OneForOne, OneForAll, RestForOne)
/// - `C`: Child type implementing the `Child` trait
/// - `M`: Monitor type for supervision events
///
/// # Design Principles
///
/// - **Zero-cost abstractions**: Generic strategy types for compile-time dispatch (§6.2)
/// - **Type safety**: Concrete types over trait objects (M-DI-HIERARCHY)
/// - **Clean separation**: Independent from Actor trait (ADR-RT-004)
/// - **Monitoring integration**: Uses Monitor<SupervisionEvent> from RT-TASK-010
///
/// # Examples
///
/// ```rust,no_run
/// use airssys_rt::supervisor::{SupervisorNode, OneForOne, ChildSpec, RestartPolicy, ShutdownPolicy, Supervisor};
/// use airssys_rt::monitoring::InMemoryMonitor;
/// use std::time::Duration;
///
/// # use airssys_rt::supervisor::Child;
/// # use async_trait::async_trait;
/// # struct MyWorker;
/// # #[derive(Debug)]
/// # struct MyError;
/// # impl std::fmt::Display for MyError {
/// #     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { Ok(()) }
/// # }
/// # impl std::error::Error for MyError {}
/// # #[async_trait]
/// # impl Child for MyWorker {
/// #     type Error = MyError;
/// #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
/// #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
/// # }
/// #
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create supervisor with OneForOne strategy
/// let monitor = InMemoryMonitor::new(Default::default());
/// let mut supervisor = SupervisorNode::<OneForOne, MyWorker, _>::new(
///     OneForOne,
///     monitor,
/// );
///
/// // Add child
/// let child_id = supervisor.start_child(ChildSpec {
///     id: "worker-1".into(),
///     factory: || MyWorker,
///     restart_policy: RestartPolicy::Permanent,
///     shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
///     start_timeout: Duration::from_secs(10),
///     shutdown_timeout: Duration::from_secs(10),
/// }).await?;
/// # Ok(())
/// # }
/// ```
/// Health monitoring configuration for supervised children.
///
/// Defines parameters for periodic health checking and failure threshold management.
///
/// # Design Philosophy
///
/// Following YAGNI principles (§6.1), health monitoring is an optional feature
/// that can be enabled per supervisor instance. This structure encapsulates all
/// health-related configuration and state tracking.
#[derive(Clone)]
pub struct HealthConfig {
    /// How often to check child health
    pub check_interval: Duration,

    /// Timeout for each individual health check
    pub check_timeout: Duration,

    /// Number of consecutive failures before triggering restart
    pub failure_threshold: u32,

    /// Per-child consecutive failure tracking
    consecutive_failures: HashMap<ChildId, u32>,
}

impl std::fmt::Debug for HealthConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HealthConfig")
            .field("check_interval", &self.check_interval)
            .field("check_timeout", &self.check_timeout)
            .field("failure_threshold", &self.failure_threshold)
            .field(
                "consecutive_failures_count",
                &self.consecutive_failures.len(),
            )
            .finish()
    }
}

impl HealthConfig {
    /// Creates a new health configuration.
    ///
    /// # Parameters
    ///
    /// - `check_interval`: How often to perform health checks
    /// - `check_timeout`: Maximum time to wait for each health check
    /// - `failure_threshold`: Consecutive failures before restart
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::time::Duration;
    /// # use airssys_rt::supervisor::node::HealthConfig;
    ///
    /// let config = HealthConfig::new(
    ///     Duration::from_secs(30),  // Check every 30 seconds
    ///     Duration::from_secs(5),   // 5 second timeout
    ///     3,                        // Restart after 3 consecutive failures
    /// );
    /// ```
    pub fn new(check_interval: Duration, check_timeout: Duration, failure_threshold: u32) -> Self {
        Self {
            check_interval,
            check_timeout,
            failure_threshold,
            consecutive_failures: HashMap::new(),
        }
    }

    /// Gets the consecutive failure count for a child.
    pub fn get_failure_count(&self, child_id: &ChildId) -> u32 {
        self.consecutive_failures
            .get(child_id)
            .copied()
            .unwrap_or(0)
    }

    /// Increments the failure count for a child.
    pub fn increment_failure(&mut self, child_id: &ChildId) {
        let count = self
            .consecutive_failures
            .entry(child_id.clone())
            .or_insert(0);
        *count += 1;
    }

    /// Resets the failure count for a child.
    pub fn reset_failure(&mut self, child_id: &ChildId) {
        self.consecutive_failures.remove(child_id);
    }

    /// Checks if a child has exceeded the failure threshold.
    pub fn has_exceeded_threshold(&self, child_id: &ChildId) -> bool {
        self.get_failure_count(child_id) >= self.failure_threshold
    }
}

#[derive(Debug)]
pub struct SupervisorNode<S, C, M>
where
    S: SupervisionStrategy,
    C: Child,
    M: Monitor<SupervisionEvent>,
{
    /// Unique supervisor identifier
    id: Uuid,

    /// Supervision strategy instance
    #[allow(dead_code)]
    // Used by StrategyDecisionLogic trait bound in determine_supervision_decision
    strategy: S,

    /// Managed children indexed by ID
    children: HashMap<ChildId, ChildHandle<C>>,

    /// Per-child restart backoff controllers
    backoff: HashMap<ChildId, RestartBackoff>,

    /// Monitor for supervision events
    monitor: M,

    /// Ordered list of child IDs (for RestForOne strategy)
    child_order: Vec<ChildId>,

    /// Optional health monitoring configuration (YAGNI §6.1 - opt-in feature)
    health_config: Option<HealthConfig>,
}

impl<S, C, M> SupervisorNode<S, C, M>
where
    S: SupervisionStrategy,
    C: Child,
    M: Monitor<SupervisionEvent> + 'static,
{
    /// Creates a new supervisor node with default configuration.
    ///
    /// Uses default restart backoff settings: 5 restarts per 60 seconds.
    /// Health monitoring is disabled by default (YAGNI §6.1).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::supervisor::{SupervisorNode, OneForOne};
    /// use airssys_rt::monitoring::InMemoryMonitor;
    ///
    /// # use airssys_rt::supervisor::Child;
    /// # use async_trait::async_trait;
    /// # use std::time::Duration;
    /// # struct MyWorker;
    /// # #[derive(Debug)]
    /// # struct MyError;
    /// # impl std::fmt::Display for MyError {
    /// #     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { Ok(()) }
    /// # }
    /// # impl std::error::Error for MyError {}
    /// # #[async_trait]
    /// # impl Child for MyWorker {
    /// #     type Error = MyError;
    /// #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
    /// #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
    /// # }
    /// let monitor = InMemoryMonitor::new(Default::default());
    /// let supervisor = SupervisorNode::<OneForOne, MyWorker, _>::new(OneForOne, monitor);
    /// ```
    pub fn new(strategy: S, monitor: M) -> Self {
        Self {
            id: Uuid::new_v4(),
            strategy,
            children: HashMap::new(),
            backoff: HashMap::new(),
            monitor,
            child_order: Vec::new(),
            health_config: None, // Health monitoring disabled by default
        }
    }

    /// Creates a new supervisor node with custom backoff configuration.
    ///
    /// # Note
    ///
    /// This method is deprecated. Use `new()` with per-child backoff instead.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use airssys_rt::supervisor::{SupervisorNode, OneForOne, RestartBackoff};
    /// use airssys_rt::monitoring::InMemoryMonitor;
    /// use std::time::Duration;
    ///
    /// # use airssys_rt::supervisor::Child;
    /// # use async_trait::async_trait;
    /// # struct MyWorker;
    /// # #[derive(Debug)]
    /// # struct MyError;
    /// # impl std::fmt::Display for MyError {
    /// #     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { Ok(()) }
    /// # }
    /// # impl std::error::Error for MyError {}
    /// # #[async_trait]
    /// # impl Child for MyWorker {
    /// #     type Error = MyError;
    /// #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
    /// #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
    /// # }
    /// # let monitor = InMemoryMonitor::new(Default::default());
    /// # let backoff = RestartBackoff::new(10, Duration::from_secs(60));
    /// // Use new() instead
    /// let supervisor = SupervisorNode::<OneForOne, MyWorker, _>::new(OneForOne, monitor);
    /// ```
    #[deprecated(note = "Use new() with per-child backoff configuration")]
    pub fn with_backoff(_strategy: S, _monitor: M, _backoff: RestartBackoff) -> Self {
        unimplemented!("Use new() with per-child backoff configuration instead")
    }

    /// Returns the number of children under supervision.
    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    /// Returns whether a child with the given ID exists.
    pub fn has_child(&self, id: &ChildId) -> bool {
        self.children.contains_key(id)
    }

    /// Returns reference to a child handle if it exists.
    pub fn get_child(&self, id: &ChildId) -> Option<&ChildHandle<C>> {
        self.children.get(id)
    }

    /// Returns mutable reference to a child handle if it exists.
    pub fn get_child_mut(&mut self, id: &ChildId) -> Option<&mut ChildHandle<C>> {
        self.children.get_mut(id)
    }

    /// Returns all child IDs in start order.
    pub fn child_ids(&self) -> &[ChildId] {
        &self.child_order
    }

    // ========================================================================
    // Health Monitoring API (Phase 4a)
    // ========================================================================

    /// Enables health monitoring with the specified configuration.
    ///
    /// Health monitoring allows the supervisor to periodically check child
    /// health and automatically restart unhealthy children. This is an
    /// optional feature (YAGNI §6.1) that can be enabled per supervisor.
    ///
    /// # Parameters
    ///
    /// - `check_interval`: How often to perform health checks
    /// - `check_timeout`: Maximum time to wait for each health check
    /// - `failure_threshold`: Number of consecutive failures before restart
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::supervisor::{SupervisorNode, OneForOne};
    /// use airssys_rt::monitoring::InMemoryMonitor;
    /// use std::time::Duration;
    ///
    /// # use airssys_rt::supervisor::Child;
    /// # use async_trait::async_trait;
    /// # struct MyWorker;
    /// # #[derive(Debug)]
    /// # struct MyError;
    /// # impl std::fmt::Display for MyError {
    /// #     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { Ok(()) }
    /// # }
    /// # impl std::error::Error for MyError {}
    /// # #[async_trait]
    /// # impl Child for MyWorker {
    /// #     type Error = MyError;
    /// #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
    /// #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
    /// # }
    /// let monitor = InMemoryMonitor::new(Default::default());
    /// let mut supervisor = SupervisorNode::<OneForOne, MyWorker, _>::new(OneForOne, monitor);
    ///
    /// // Enable health checks: every 30s, 5s timeout, restart after 3 failures
    /// supervisor.enable_health_checks(
    ///     Duration::from_secs(30),
    ///     Duration::from_secs(5),
    ///     3,
    /// );
    /// ```
    pub fn enable_health_checks(
        &mut self,
        check_interval: Duration,
        check_timeout: Duration,
        failure_threshold: u32,
    ) {
        self.health_config = Some(HealthConfig::new(
            check_interval,
            check_timeout,
            failure_threshold,
        ));
    }

    /// Disables health monitoring.
    ///
    /// Removes the health configuration and clears all failure tracking.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use airssys_rt::supervisor::{SupervisorNode, OneForOne};
    /// # use airssys_rt::monitoring::InMemoryMonitor;
    /// # use std::time::Duration;
    /// # use airssys_rt::supervisor::Child;
    /// # use async_trait::async_trait;
    /// # struct MyWorker;
    /// # #[derive(Debug)]
    /// # struct MyError;
    /// # impl std::fmt::Display for MyError {
    /// #     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { Ok(()) }
    /// # }
    /// # impl std::error::Error for MyError {}
    /// # #[async_trait]
    /// # impl Child for MyWorker {
    /// #     type Error = MyError;
    /// #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
    /// #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
    /// # }
    /// let monitor = InMemoryMonitor::new(Default::default());
    /// let mut supervisor = SupervisorNode::<OneForOne, MyWorker, _>::new(OneForOne, monitor);
    ///
    /// supervisor.enable_health_checks(Duration::from_secs(30), Duration::from_secs(5), 3);
    /// // Later...
    /// supervisor.disable_health_checks();
    /// ```
    pub fn disable_health_checks(&mut self) {
        self.health_config = None;
    }

    /// Returns whether health monitoring is currently enabled.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use airssys_rt::supervisor::{SupervisorNode, OneForOne};
    /// # use airssys_rt::monitoring::InMemoryMonitor;
    /// # use std::time::Duration;
    /// # use airssys_rt::supervisor::Child;
    /// # use async_trait::async_trait;
    /// # struct MyWorker;
    /// # #[derive(Debug)]
    /// # struct MyError;
    /// # impl std::fmt::Display for MyError {
    /// #     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { Ok(()) }
    /// # }
    /// # impl std::error::Error for MyError {}
    /// # #[async_trait]
    /// # impl Child for MyWorker {
    /// #     type Error = MyError;
    /// #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
    /// #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
    /// # }
    /// let monitor = InMemoryMonitor::new(Default::default());
    /// let mut supervisor = SupervisorNode::<OneForOne, MyWorker, _>::new(OneForOne, monitor);
    ///
    /// assert!(!supervisor.is_health_monitoring_enabled());
    ///
    /// supervisor.enable_health_checks(Duration::from_secs(30), Duration::from_secs(5), 3);
    /// assert!(supervisor.is_health_monitoring_enabled());
    /// ```
    pub fn is_health_monitoring_enabled(&self) -> bool {
        self.health_config.is_some()
    }

    /// Gets the health configuration if enabled.
    ///
    /// Returns `None` if health monitoring is not enabled.
    pub fn health_config(&self) -> Option<&HealthConfig> {
        self.health_config.as_ref()
    }

    /// Gets mutable health configuration if enabled.
    ///
    /// Returns `None` if health monitoring is not enabled.
    pub fn health_config_mut(&mut self) -> Option<&mut HealthConfig> {
        self.health_config.as_mut()
    }

    /// Enables automatic background health monitoring (builder pattern).
    ///
    /// This method combines health configuration and automatic background monitoring
    /// in a single call. It configures health checking parameters and automatically
    /// spawns a background task that periodically checks all children.
    ///
    /// **Important**: This method consumes `self` and returns an `Arc<Mutex<Self>>`.
    /// The supervisor is wrapped to allow shared access from the background task.
    /// All subsequent operations must use the returned Arc.
    ///
    /// # Parameters
    ///
    /// - `check_interval`: How often to perform health checks
    /// - `check_timeout`: Maximum time to wait for each health check
    /// - `failure_threshold`: Number of consecutive failures before restart
    ///
    /// # Returns
    ///
    /// An `Arc<Mutex<SupervisorNode>>` with background health monitoring active.
    /// The background task will continue until the supervisor is dropped or
    /// `disable_health_checks()` is called.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use airssys_rt::supervisor::{SupervisorNode, OneForOne};
    /// use airssys_rt::monitoring::InMemoryMonitor;
    /// use std::time::Duration;
    ///
    /// # use airssys_rt::supervisor::Child;
    /// # use async_trait::async_trait;
    /// # struct MyWorker;
    /// # #[derive(Debug)]
    /// # struct MyError;
    /// # impl std::fmt::Display for MyError {
    /// #     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { Ok(()) }
    /// # }
    /// # impl std::error::Error for MyError {}
    /// # #[async_trait]
    /// # impl Child for MyWorker {
    /// #     type Error = MyError;
    /// #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
    /// #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
    /// # }
    /// # async fn example() {
    /// let monitor = InMemoryMonitor::new(Default::default());
    ///
    /// // Builder pattern - automatic background monitoring
    /// let supervisor = SupervisorNode::<OneForOne, MyWorker, _>::new(OneForOne, monitor)
    ///     .with_automatic_health_monitoring(
    ///         Duration::from_secs(30),  // Check every 30 seconds
    ///         Duration::from_secs(5),   // 5 second timeout
    ///         3,                        // Restart after 3 failures
    ///     );
    ///
    /// // Use the wrapped supervisor
    /// // supervisor.lock().await.start_child(...).await;
    ///
    /// // Background health monitoring is automatically running!
    /// # }
    /// ```
    pub fn with_automatic_health_monitoring(
        mut self,
        check_interval: Duration,
        check_timeout: Duration,
        failure_threshold: u32,
    ) -> MonitoredSupervisor<S, C, M>
    where
        S: Send + Sync + 'static,
        C: Send + Sync + 'static,
        M: Send + Sync + 'static,
    {
        // Enable health monitoring configuration
        self.enable_health_checks(check_interval, check_timeout, failure_threshold);

        // Wrap in Arc<Mutex<>> for shared access
        let supervisor_arc = Arc::new(Mutex::new(self));

        // Spawn background health monitoring task
        let (task_handle, shutdown_tx) =
            spawn_health_monitor(Arc::clone(&supervisor_arc), check_interval);

        // Return wrapped supervisor that keeps task alive
        MonitoredSupervisor {
            supervisor: supervisor_arc,
            _task_handle: task_handle,
            _shutdown_tx: shutdown_tx,
        }
    }

    // ========================================================================
    // Health Check Logic (Phase 4b)
    // ========================================================================

    /// Checks the health of a specific child with timeout.
    ///
    /// This method performs a health check on the specified child and updates
    /// consecutive failure tracking. If the failure threshold is exceeded,
    /// the child will be automatically restarted.
    ///
    /// # Parameters
    ///
    /// - `child_id`: The ID of the child to check
    ///
    /// # Returns
    ///
    /// - `Ok(ChildHealth)`: The health status of the child
    /// - `Err(SupervisorError::HealthMonitoringNotEnabled)`: If monitoring is disabled
    /// - `Err(SupervisorError::ChildNotFound)`: If child doesn't exist
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use airssys_rt::supervisor::{SupervisorNode, OneForOne, ChildHealth};
    /// use airssys_rt::monitoring::InMemoryMonitor;
    /// use std::time::Duration;
    ///
    /// # use airssys_rt::supervisor::Child;
    /// # use async_trait::async_trait;
    /// # struct MyWorker;
    /// # #[derive(Debug)]
    /// # struct MyError;
    /// # impl std::fmt::Display for MyError {
    /// #     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { Ok(()) }
    /// # }
    /// # impl std::error::Error for MyError {}
    /// # #[async_trait]
    /// # impl Child for MyWorker {
    /// #     type Error = MyError;
    /// #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
    /// #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
    /// # }
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let monitor = InMemoryMonitor::new(Default::default());
    /// # let mut supervisor = SupervisorNode::<OneForOne, MyWorker, _>::new(OneForOne, monitor);
    /// # let child_id = Default::default();
    /// // Enable health monitoring
    /// supervisor.enable_health_checks(Duration::from_secs(30), Duration::from_secs(5), 3);
    ///
    /// // Check child health
    /// let health = supervisor.check_child_health(&child_id).await?;
    /// match health {
    ///     ChildHealth::Healthy => println!("Child is healthy"),
    ///     ChildHealth::Degraded(reason) => println!("Child degraded: {}", reason),
    ///     ChildHealth::Failed(reason) => println!("Child failed: {}", reason),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn check_child_health(
        &mut self,
        child_id: &ChildId,
    ) -> Result<ChildHealth, SupervisorError> {
        // Ensure health monitoring is enabled
        let config = self.health_config.as_ref().ok_or_else(|| {
            SupervisorError::HealthMonitoringNotEnabled {
                id: child_id.to_string(),
            }
        })?;

        let check_timeout = config.check_timeout;

        // Get child handle
        let child_handle =
            self.children
                .get(child_id)
                .ok_or_else(|| SupervisorError::ChildNotFound {
                    id: child_id.clone(),
                })?;

        // Perform health check with timeout
        let health_result = timeout(check_timeout, child_handle.child().health_check()).await;

        let health = match health_result {
            Ok(health) => health,
            Err(_) => {
                // Timeout occurred
                ChildHealth::Failed(format!("Health check timeout after {check_timeout:?}"))
            }
        };

        // Handle health check result
        self.handle_health_check_result(child_id, &health).await?;

        Ok(health)
    }

    /// Handles the result of a health check and updates failure tracking.
    ///
    /// This method updates consecutive failure counts and triggers restart
    /// if the failure threshold is exceeded.
    async fn handle_health_check_result(
        &mut self,
        child_id: &ChildId,
        health: &ChildHealth,
    ) -> Result<(), SupervisorError> {
        let should_restart = {
            // Get mutable config to update failure tracking
            let config = self.health_config.as_mut().ok_or_else(|| {
                SupervisorError::HealthMonitoringNotEnabled {
                    id: child_id.to_string(),
                }
            })?;

            match health {
                ChildHealth::Healthy => {
                    // Reset failure count on successful health check
                    config.reset_failure(child_id);
                    false
                }
                ChildHealth::Degraded(_) => {
                    // Don't count degraded as failure, but emit event
                    let _ = self
                        .monitor
                        .record(SupervisionEvent {
                            timestamp: Utc::now(),
                            supervisor_id: self.id.to_string(),
                            child_id: Some(child_id.to_string()),
                            event_kind: SupervisionEventKind::ChildFailed {
                                error: format!("Child degraded: {health:?}"),
                                restart_count: 0,
                            },
                            metadata: HashMap::new(),
                        })
                        .await;
                    false
                }
                ChildHealth::Failed(reason) => {
                    // Increment failure count
                    config.increment_failure(child_id);

                    // Check if threshold exceeded
                    let exceeded = config.has_exceeded_threshold(child_id);
                    let failure_count = config.get_failure_count(child_id);

                    // Emit health check failure event
                    let _ = self
                        .monitor
                        .record(SupervisionEvent {
                            timestamp: Utc::now(),
                            supervisor_id: self.id.to_string(),
                            child_id: Some(child_id.to_string()),
                            event_kind: SupervisionEventKind::ChildFailed {
                                error: format!("Health check failed ({failure_count}): {reason}"),
                                restart_count: failure_count,
                            },
                            metadata: HashMap::new(),
                        })
                        .await;

                    exceeded
                }
            }
        };

        // Trigger restart if threshold exceeded
        if should_restart {
            // Reset failure count before restart
            if let Some(config) = self.health_config.as_mut() {
                config.reset_failure(child_id);
            }

            // Restart the child
            self.restart_child(child_id).await?;
        }

        Ok(())
    }

    /// Starts a child with timeout.
    async fn start_child_with_timeout(
        &mut self,
        child_id: &ChildId,
        child_handle: &mut ChildHandle<C>,
        start_timeout: Duration,
    ) -> Result<(), SupervisorError> {
        child_handle.set_state(ChildState::Starting);

        // Start child with timeout
        let result = timeout(start_timeout, child_handle.child_mut().start()).await;

        match result {
            Ok(Ok(())) => {
                child_handle.set_state(ChildState::Running);
                let _ = self
                    .monitor
                    .record(SupervisionEvent {
                        timestamp: Utc::now(),
                        supervisor_id: self.id.to_string(),
                        child_id: Some(child_id.to_string()),
                        event_kind: SupervisionEventKind::ChildStarted,
                        metadata: HashMap::new(),
                    })
                    .await;
                Ok(())
            }
            Ok(Err(e)) => {
                child_handle.set_state(ChildState::Failed);
                Err(SupervisorError::ChildStartFailed {
                    id: child_id.to_string(),
                    source: Box::new(e) as Box<dyn StdError + Send + Sync>,
                })
            }
            Err(_) => {
                child_handle.set_state(ChildState::Failed);
                // Note: ChildStartTimeout doesn't exist, using ChildStartFailed instead
                Err(SupervisorError::ChildStartFailed {
                    id: child_id.to_string(),
                    source: Box::new(std::io::Error::new(
                        std::io::ErrorKind::TimedOut,
                        format!("Child start timeout after {start_timeout:?}"),
                    )) as Box<dyn StdError + Send + Sync>,
                })
            }
        }
    }

    /// Stops a child with timeout according to shutdown policy.
    #[allow(dead_code)] // Will be used in Phase 4 when implementing full lifecycle management
    async fn stop_child_with_timeout(
        &mut self,
        child_id: &ChildId,
        child_handle: &mut ChildHandle<C>,
        shutdown_timeout: Duration,
    ) -> Result<(), SupervisorError> {
        child_handle.set_state(ChildState::Stopping);

        let effective_timeout = match child_handle.shutdown_policy() {
            ShutdownPolicy::Immediate => Duration::from_millis(0),
            ShutdownPolicy::Graceful(duration) => *duration,
            ShutdownPolicy::Infinity => Duration::from_secs(u64::MAX),
        };

        let actual_timeout = effective_timeout.min(shutdown_timeout);

        // Stop child with timeout
        let result = timeout(
            actual_timeout,
            child_handle.child_mut().stop(actual_timeout),
        )
        .await;

        match result {
            Ok(Ok(())) => {
                child_handle.set_state(ChildState::Stopped);
                let _ = self
                    .monitor
                    .record(SupervisionEvent {
                        timestamp: Utc::now(),
                        supervisor_id: self.id.to_string(),
                        child_id: Some(child_id.to_string()),
                        event_kind: SupervisionEventKind::ChildStopped,
                        metadata: HashMap::new(),
                    })
                    .await;
                Ok(())
            }
            Ok(Err(e)) => {
                child_handle.set_state(ChildState::Failed);
                Err(SupervisorError::ChildStopFailed {
                    id: child_id.to_string(),
                    source: Box::new(e) as Box<dyn StdError + Send + Sync>,
                })
            }
            Err(_) => {
                child_handle.set_state(ChildState::Stopped);
                Err(SupervisorError::ShutdownTimeout {
                    id: child_id.to_string(),
                    timeout: actual_timeout,
                })
            }
        }
    }

    /// Gets children started after the given child (for RestForOne strategy).
    #[allow(dead_code)] // Will be used when RestForOne strategy is fully integrated
    fn get_children_started_after(&self, child_id: &ChildId) -> Vec<ChildId> {
        if let Some(index) = self.child_order.iter().position(|id| id == child_id) {
            self.child_order[index + 1..].to_vec()
        } else {
            Vec::new()
        }
    }

    /// Determines supervision decision based on strategy and failure.
    fn determine_supervision_decision(
        &self,
        failed_child_id: &ChildId,
        is_normal_exit: bool,
    ) -> SupervisionDecision {
        let child_handle = match self.children.get(failed_child_id) {
            Some(handle) => handle,
            // If child not found, escalate to parent
            None => {
                return SupervisionDecision::Escalate(format!("Child not found: {failed_child_id}"))
            }
        };

        // Check if we should restart based on restart policy
        if !should_restart(child_handle.restart_policy(), is_normal_exit) {
            return SupervisionDecision::StopChild(failed_child_id.clone());
        }

        // Build strategy context
        let context = StrategyContext::SingleFailure {
            failed_child_id: failed_child_id.clone(),
            all_child_ids: self.child_order.clone(),
        };

        // Apply strategy-specific logic
        S::determine_decision(context)
    }

    /// Create a fluent builder for adding a single child.
    ///
    /// This provides an ergonomic alternative to manual [`ChildSpec`] construction
    /// with sensible defaults for production use. The builder pattern reduces
    /// boilerplate while maintaining full customization capabilities.
    ///
    /// # Examples
    ///
    /// ## Minimal Configuration
    ///
    /// ```rust,no_run
    /// use airssys_rt::supervisor::{SupervisorNode, OneForOne, Child};
    /// use airssys_rt::monitoring::{NoopMonitor, SupervisionEvent};
    /// use async_trait::async_trait;
    /// use std::time::Duration;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # #[derive(Clone)]
    /// # struct MyWorker;
    /// # #[async_trait]
    /// # impl Child for MyWorker {
    /// #     type Error = std::io::Error;
    /// #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
    /// #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
    /// # }
    /// let mut supervisor = SupervisorNode::new(OneForOne, NoopMonitor::<SupervisionEvent>::new());
    ///
    /// // Uses defaults: Permanent restart, 5s graceful shutdown, 30s start timeout
    /// let id = supervisor
    ///     .child("worker")
    ///     .factory(|| MyWorker)
    ///     .spawn()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Full Customization
    ///
    /// ```rust,no_run
    /// use airssys_rt::supervisor::{SupervisorNode, OneForOne, Child};
    /// use airssys_rt::monitoring::{NoopMonitor, SupervisionEvent};
    /// use async_trait::async_trait;
    /// use std::time::Duration;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # #[derive(Clone)]
    /// # struct CriticalWorker;
    /// # #[async_trait]
    /// # impl Child for CriticalWorker {
    /// #     type Error = std::io::Error;
    /// #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
    /// #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
    /// # }
    /// let mut supervisor = SupervisorNode::new(OneForOne, NoopMonitor::<SupervisionEvent>::new());
    ///
    /// let id = supervisor
    ///     .child("critical")
    ///     .factory(|| CriticalWorker)
    ///     .restart_transient()
    ///     .shutdown_graceful(Duration::from_secs(15))
    ///     .start_timeout(Duration::from_secs(60))
    ///     .spawn()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Comparison with Manual ChildSpec
    ///
    /// ```rust,no_run
    /// use airssys_rt::supervisor::{SupervisorNode, OneForOne, Child, ChildSpec, Supervisor};
    /// use airssys_rt::supervisor::{RestartPolicy, ShutdownPolicy};
    /// use airssys_rt::monitoring::{NoopMonitor, SupervisionEvent};
    /// use async_trait::async_trait;
    /// use std::time::Duration;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # #[derive(Clone)]
    /// # struct MyWorker;
    /// # #[async_trait]
    /// # impl Child for MyWorker {
    /// #     type Error = std::io::Error;
    /// #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
    /// #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
    /// # }
    /// let mut supervisor = SupervisorNode::new(OneForOne, NoopMonitor::<SupervisionEvent>::new());
    ///
    /// // Manual approach (10+ lines)
    /// let spec = ChildSpec {
    ///     id: "worker".into(),
    ///     factory: Box::new(|| MyWorker),
    ///     restart_policy: RestartPolicy::Permanent,
    ///     shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
    ///     start_timeout: Duration::from_secs(30),
    ///     shutdown_timeout: Duration::from_secs(10),
    /// };
    /// supervisor.start_child(spec).await?;
    ///
    /// // Builder approach (4 lines - 60% reduction)
    /// let id = supervisor
    ///     .child("worker2")
    ///     .factory(|| MyWorker)
    ///     .spawn()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # See Also
    ///
    /// - [`crate::supervisor::builder::SingleChildBuilder`] for full builder API
    /// - [`ChildSpec`] for manual specification
    /// - [`start_child`](Self::start_child) for spawning with manual spec
    pub fn child(&mut self, id: impl Into<String>) -> SingleChildBuilder<'_, S, C, M> {
        SingleChildBuilder::new(self, id.into())
    }

    /// Create a batch builder for multiple children
    pub fn children(&mut self) -> ChildrenBatchBuilder<'_, S, C, M> {
        ChildrenBatchBuilder::new(self)
    }
}

#[async_trait]
impl<S, C, M> Supervisor for SupervisorNode<S, C, M>
where
    S: SupervisionStrategy + Send + Sync,
    C: Child + Send + Sync,
    M: Monitor<SupervisionEvent> + Send + Sync + 'static,
{
    type Child = C;

    async fn start_child<F>(&mut self, spec: ChildSpec<C, F>) -> Result<ChildId, SupervisorError>
    where
        F: Fn() -> C + Send + Sync + 'static,
    {
        // Create child instance
        let child = (spec.factory)();

        // Create child ID
        let child_id = ChildId::new();

        // Create child handle
        let mut child_handle = ChildHandle::new(child, spec.restart_policy, spec.shutdown_policy);

        // Start child with timeout
        self.start_child_with_timeout(&child_id, &mut child_handle, spec.start_timeout)
            .await?;

        // Add to children map and order
        self.children.insert(child_id.clone(), child_handle);
        self.child_order.push(child_id.clone());

        Ok(child_id)
    }

    async fn stop_child(&mut self, id: &ChildId) -> Result<(), SupervisorError> {
        // Check if child exists
        if !self.children.contains_key(id) {
            return Err(SupervisorError::ChildNotFound { id: id.clone() });
        }

        // Get shutdown policy and stop the child
        let stop_result = {
            let child_handle = match self.children.get_mut(id) {
                Some(handle) => handle,
                None => return Err(SupervisorError::ChildNotFound { id: id.clone() }),
            };

            child_handle.set_state(ChildState::Stopping);

            let shutdown_policy = *child_handle.shutdown_policy();
            let shutdown_timeout = match shutdown_policy {
                ShutdownPolicy::Immediate => Duration::from_millis(0),
                ShutdownPolicy::Graceful(duration) => duration,
                ShutdownPolicy::Infinity => Duration::from_secs(10), // Use default for infinity
            };

            // Stop the child
            child_handle.child_mut().stop(shutdown_timeout).await
        };

        // Update state based on result
        match stop_result {
            Ok(()) => {
                if let Some(handle) = self.children.get_mut(id) {
                    handle.set_state(ChildState::Stopped);
                }
            }
            Err(e) => {
                if let Some(handle) = self.children.get_mut(id) {
                    handle.set_state(ChildState::Failed);
                }
                return Err(SupervisorError::ChildStopFailed {
                    id: id.to_string(),
                    source: Box::new(e) as Box<dyn StdError + Send + Sync>,
                });
            }
        }

        // Remove from tracking
        self.children.remove(id);
        self.child_order.retain(|cid| cid != id);

        Ok(())
    }

    async fn restart_child(&mut self, id: &ChildId) -> Result<(), SupervisorError> {
        // Get child handle
        let child_handle = self
            .children
            .get_mut(id)
            .ok_or_else(|| SupervisorError::ChildNotFound { id: id.clone() })?;

        // Get or create backoff for this child
        let backoff = self
            .backoff
            .entry(id.clone())
            .or_insert_with(|| RestartBackoff::new(5, Duration::from_secs(60)));

        // Check if restart limit exceeded
        if backoff.restart_count() >= 5 {
            return Err(SupervisorError::RestartLimitExceeded {
                id: id.to_string(),
                max_restarts: 5,
                window: Duration::from_secs(60),
            });
        }

        // Calculate and apply backoff delay
        let delay = backoff.calculate_delay();
        if delay > Duration::from_millis(0) {
            tokio::time::sleep(delay).await;
        }

        // Record restart in backoff tracker
        backoff.record_restart();

        // Record restart in child handle
        let restart_count = child_handle.restart_count();
        child_handle.record_restart();

        // Restart: stop then start
        child_handle.set_state(ChildState::Restarting);

        // Stop first (with default timeout)
        let stop_result = child_handle.child_mut().stop(Duration::from_secs(10)).await;

        if let Err(e) = stop_result {
            // Log error but continue with restart
            let _ = self
                .monitor
                .record(SupervisionEvent {
                    timestamp: Utc::now(),
                    supervisor_id: self.id.to_string(),
                    child_id: Some(id.to_string()),
                    event_kind: SupervisionEventKind::ChildFailed {
                        error: format!("Stop failed during restart: {e}"),
                        restart_count,
                    },
                    metadata: HashMap::new(),
                })
                .await;
        }

        // Start again (with default timeout)
        child_handle.set_state(ChildState::Starting);
        let start_result = child_handle.child_mut().start().await;

        match start_result {
            Ok(()) => {
                child_handle.set_state(ChildState::Running);
                let _ = self
                    .monitor
                    .record(SupervisionEvent {
                        timestamp: Utc::now(),
                        supervisor_id: self.id.to_string(),
                        child_id: Some(id.to_string()),
                        event_kind: SupervisionEventKind::ChildRestarted {
                            restart_count: child_handle.restart_count(),
                        },
                        metadata: HashMap::new(),
                    })
                    .await;
                Ok(())
            }
            Err(e) => {
                child_handle.set_state(ChildState::Failed);
                Err(SupervisorError::ChildStartFailed {
                    id: id.to_string(),
                    source: Box::new(e) as Box<dyn StdError + Send + Sync>,
                })
            }
        }
    }

    async fn handle_child_error(
        &mut self,
        id: &ChildId,
        _error: Box<dyn StdError + Send + Sync>,
    ) -> SupervisionDecision {
        // TODO: Parse error to determine if it was a normal exit
        // For now, assume abnormal exit
        let is_normal_exit = false;
        let decision = self.determine_supervision_decision(id, is_normal_exit);

        // Calculate affected count based on decision
        let affected_count = match &decision {
            SupervisionDecision::RestartChild(_) => 1,
            SupervisionDecision::RestartAll(ids) => ids.len(),
            SupervisionDecision::RestartSubset(ids) => ids.len(),
            SupervisionDecision::StopChild(_) => 1,
            SupervisionDecision::StopAll => self.children.len(),
            SupervisionDecision::Escalate(_) => 0,
        };

        // Record supervision decision (fire-and-forget for monitoring)
        let monitor = self.monitor.clone();
        let supervisor_id = self.id.to_string();
        let child_id_str = id.to_string();
        let strategy_name = std::any::type_name::<S>().to_string();

        tokio::spawn(async move {
            let _ = monitor
                .record(SupervisionEvent {
                    timestamp: Utc::now(),
                    supervisor_id,
                    child_id: Some(child_id_str),
                    event_kind: SupervisionEventKind::StrategyApplied {
                        strategy: strategy_name,
                        affected_count,
                    },
                    metadata: HashMap::new(),
                })
                .await;
        });

        decision
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::monitoring::InMemoryMonitor;
    use crate::supervisor::strategy::{OneForOne, RestForOne};
    use crate::supervisor::types::ChildHealth;

    // Test child implementation
    #[derive(Debug)]
    struct TestChild {
        should_fail_start: bool,
        should_fail_stop: bool,
    }

    #[derive(Debug)]
    struct TestChildError;

    impl std::fmt::Display for TestChildError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "TestChildError")
        }
    }

    impl std::error::Error for TestChildError {}

    #[async_trait]
    impl Child for TestChild {
        type Error = TestChildError;

        async fn start(&mut self) -> Result<(), Self::Error> {
            if self.should_fail_start {
                Err(TestChildError)
            } else {
                Ok(())
            }
        }

        async fn stop(&mut self, _timeout: Duration) -> Result<(), Self::Error> {
            if self.should_fail_stop {
                Err(TestChildError)
            } else {
                Ok(())
            }
        }

        async fn health_check(&self) -> ChildHealth {
            ChildHealth::Healthy
        }
    }

    #[tokio::test]
    async fn test_supervisor_node_creation() {
        let monitor = InMemoryMonitor::new(Default::default());
        let supervisor = SupervisorNode::<OneForOne, TestChild, _>::new(OneForOne, monitor);

        assert_eq!(supervisor.child_count(), 0);
    }

    #[tokio::test]
    async fn test_start_child_success() {
        let monitor = InMemoryMonitor::new(Default::default());
        let mut supervisor = SupervisorNode::<OneForOne, TestChild, _>::new(OneForOne, monitor);

        let spec = ChildSpec {
            id: "test-child".into(),
            factory: || TestChild {
                should_fail_start: false,
                should_fail_stop: false,
            },
            restart_policy: RestartPolicy::Permanent,
            shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
            start_timeout: Duration::from_secs(10),
            shutdown_timeout: Duration::from_secs(10),
        };

        let child_id = supervisor.start_child(spec).await.unwrap();

        assert_eq!(supervisor.child_count(), 1);
        assert!(supervisor.has_child(&child_id));

        let handle = supervisor.get_child(&child_id).unwrap();
        assert_eq!(handle.state(), &ChildState::Running);
    }

    #[tokio::test]
    async fn test_start_child_failure() {
        let monitor = InMemoryMonitor::new(Default::default());
        let mut supervisor = SupervisorNode::<OneForOne, TestChild, _>::new(OneForOne, monitor);

        let spec = ChildSpec {
            id: "test-child".into(),
            factory: || TestChild {
                should_fail_start: true,
                should_fail_stop: false,
            },
            restart_policy: RestartPolicy::Permanent,
            shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
            start_timeout: Duration::from_secs(10),
            shutdown_timeout: Duration::from_secs(10),
        };

        let result = supervisor.start_child(spec).await;
        assert!(result.is_err());
        assert_eq!(supervisor.child_count(), 0);
    }

    #[tokio::test]
    async fn test_stop_child_success() {
        let monitor = InMemoryMonitor::new(Default::default());
        let mut supervisor = SupervisorNode::<OneForOne, TestChild, _>::new(OneForOne, monitor);

        let spec = ChildSpec {
            id: "test-child".into(),
            factory: || TestChild {
                should_fail_start: false,
                should_fail_stop: false,
            },
            restart_policy: RestartPolicy::Permanent,
            shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
            start_timeout: Duration::from_secs(10),
            shutdown_timeout: Duration::from_secs(10),
        };

        let child_id = supervisor.start_child(spec).await.unwrap();
        assert_eq!(supervisor.child_count(), 1);

        supervisor.stop_child(&child_id).await.unwrap();
        assert_eq!(supervisor.child_count(), 0);
        assert!(!supervisor.has_child(&child_id));
    }

    #[tokio::test]
    async fn test_stop_nonexistent_child() {
        let monitor = InMemoryMonitor::new(Default::default());
        let mut supervisor = SupervisorNode::<OneForOne, TestChild, _>::new(OneForOne, monitor);

        let child_id = ChildId::new();
        let result = supervisor.stop_child(&child_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_restart_child_success() {
        let monitor = InMemoryMonitor::new(Default::default());
        let mut supervisor = SupervisorNode::<OneForOne, TestChild, _>::new(OneForOne, monitor);

        let spec = ChildSpec {
            id: "test-child".into(),
            factory: || TestChild {
                should_fail_start: false,
                should_fail_stop: false,
            },
            restart_policy: RestartPolicy::Permanent,
            shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
            start_timeout: Duration::from_secs(10),
            shutdown_timeout: Duration::from_secs(10),
        };

        let child_id = supervisor.start_child(spec).await.unwrap();
        let restart_count_before = supervisor.get_child(&child_id).unwrap().restart_count();

        supervisor.restart_child(&child_id).await.unwrap();

        let handle = supervisor.get_child(&child_id).unwrap();
        assert_eq!(handle.state(), &ChildState::Running);
        assert_eq!(handle.restart_count(), restart_count_before + 1);
    }

    #[tokio::test]
    async fn test_child_order_tracking() {
        let monitor = InMemoryMonitor::new(Default::default());
        let mut supervisor = SupervisorNode::<OneForOne, TestChild, _>::new(OneForOne, monitor);

        let mut child_ids = Vec::new();

        for i in 0..3 {
            let spec = ChildSpec {
                id: format!("child-{i}"),
                factory: || TestChild {
                    should_fail_start: false,
                    should_fail_stop: false,
                },
                restart_policy: RestartPolicy::Permanent,
                shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
                start_timeout: Duration::from_secs(10),
                shutdown_timeout: Duration::from_secs(10),
            };

            let child_id = supervisor.start_child(spec).await.unwrap();
            child_ids.push(child_id);
        }

        assert_eq!(supervisor.child_ids(), &child_ids);
    }

    #[tokio::test]
    async fn test_get_children_started_after() {
        let monitor = InMemoryMonitor::new(Default::default());
        let mut supervisor = SupervisorNode::<RestForOne, TestChild, _>::new(RestForOne, monitor);

        let mut child_ids = Vec::new();

        for i in 0..5 {
            let spec = ChildSpec {
                id: format!("child-{i}"),
                factory: || TestChild {
                    should_fail_start: false,
                    should_fail_stop: false,
                },
                restart_policy: RestartPolicy::Permanent,
                shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
                start_timeout: Duration::from_secs(10),
                shutdown_timeout: Duration::from_secs(10),
            };

            let child_id = supervisor.start_child(spec).await.unwrap();
            child_ids.push(child_id);
        }

        // Get children after second child (index 1)
        let children_after = supervisor.get_children_started_after(&child_ids[1]);
        assert_eq!(children_after, &child_ids[2..]);

        // Get children after last child
        let children_after = supervisor.get_children_started_after(&child_ids[4]);
        assert!(children_after.is_empty());
    }

    #[tokio::test]
    async fn test_restart_rate_limiting() {
        let monitor = InMemoryMonitor::new(Default::default());
        let mut supervisor = SupervisorNode::<OneForOne, TestChild, _>::new(OneForOne, monitor);

        let spec = ChildSpec {
            id: "test-child".into(),
            factory: || TestChild {
                should_fail_start: false,
                should_fail_stop: false,
            },
            restart_policy: RestartPolicy::Permanent,
            shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
            start_timeout: Duration::from_secs(10),
            shutdown_timeout: Duration::from_secs(10),
        };

        let child_id = supervisor.start_child(spec).await.unwrap();

        // Default backoff allows 5 restarts per 60 seconds
        // First 5 restarts should succeed
        for _ in 0..5 {
            supervisor.restart_child(&child_id).await.unwrap();
        }

        // 6th restart should fail due to rate limit
        let result = supervisor.restart_child(&child_id).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SupervisorError::RestartLimitExceeded { .. }
        ));
    }

    #[tokio::test]
    async fn test_restart_per_child_backoff_independence() {
        let monitor = InMemoryMonitor::new(Default::default());
        let mut supervisor = SupervisorNode::<OneForOne, TestChild, _>::new(OneForOne, monitor);

        // Start two children
        let spec1 = ChildSpec {
            id: "child-1".into(),
            factory: || TestChild {
                should_fail_start: false,
                should_fail_stop: false,
            },
            restart_policy: RestartPolicy::Permanent,
            shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
            start_timeout: Duration::from_secs(10),
            shutdown_timeout: Duration::from_secs(10),
        };

        let spec2 = ChildSpec {
            id: "child-2".into(),
            factory: || TestChild {
                should_fail_start: false,
                should_fail_stop: false,
            },
            restart_policy: RestartPolicy::Permanent,
            shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
            start_timeout: Duration::from_secs(10),
            shutdown_timeout: Duration::from_secs(10),
        };

        let child1_id = supervisor.start_child(spec1).await.unwrap();
        let child2_id = supervisor.start_child(spec2).await.unwrap();

        // Restart child1 5 times (hit limit)
        for _ in 0..5 {
            supervisor.restart_child(&child1_id).await.unwrap();
        }

        // child1 should be at limit
        let result1 = supervisor.restart_child(&child1_id).await;
        assert!(matches!(
            result1.unwrap_err(),
            SupervisorError::RestartLimitExceeded { .. }
        ));

        // child2 should still be able to restart (independent backoff)
        let result2 = supervisor.restart_child(&child2_id).await;
        assert!(result2.is_ok());
    }

    #[tokio::test]
    async fn test_child_handle_state_transitions() {
        let child = TestChild {
            should_fail_start: false,
            should_fail_stop: false,
        };

        let mut handle = ChildHandle::new(
            child,
            RestartPolicy::Permanent,
            ShutdownPolicy::Graceful(Duration::from_secs(5)),
        );

        assert_eq!(handle.state(), &ChildState::Starting);

        handle.set_state(ChildState::Running);
        assert_eq!(handle.state(), &ChildState::Running);

        handle.set_state(ChildState::Stopping);
        assert_eq!(handle.state(), &ChildState::Stopping);

        handle.set_state(ChildState::Stopped);
        assert_eq!(handle.state(), &ChildState::Stopped);
    }

    #[tokio::test]
    async fn test_child_handle_restart_tracking() {
        let child = TestChild {
            should_fail_start: false,
            should_fail_stop: false,
        };

        let mut handle = ChildHandle::new(
            child,
            RestartPolicy::Permanent,
            ShutdownPolicy::Graceful(Duration::from_secs(5)),
        );

        assert_eq!(handle.restart_count(), 0);

        handle.record_restart();
        assert_eq!(handle.restart_count(), 1);

        handle.record_restart();
        handle.record_restart();
        assert_eq!(handle.restart_count(), 3);
    }

    // ========================================================================
    // Phase 4a: Health Monitoring Configuration Tests
    // ========================================================================

    #[test]
    fn test_health_config_initialization() {
        let config = HealthConfig::new(Duration::from_secs(30), Duration::from_secs(5), 3);

        assert_eq!(config.check_interval, Duration::from_secs(30));
        assert_eq!(config.check_timeout, Duration::from_secs(5));
        assert_eq!(config.failure_threshold, 3);
    }

    #[test]
    fn test_health_config_failure_tracking() {
        let mut config = HealthConfig::new(Duration::from_secs(30), Duration::from_secs(5), 3);

        let child_id = ChildId::new();

        // Initial count should be 0
        assert_eq!(config.get_failure_count(&child_id), 0);
        assert!(!config.has_exceeded_threshold(&child_id));

        // Increment failures
        config.increment_failure(&child_id);
        assert_eq!(config.get_failure_count(&child_id), 1);
        assert!(!config.has_exceeded_threshold(&child_id));

        config.increment_failure(&child_id);
        assert_eq!(config.get_failure_count(&child_id), 2);
        assert!(!config.has_exceeded_threshold(&child_id));

        config.increment_failure(&child_id);
        assert_eq!(config.get_failure_count(&child_id), 3);
        assert!(config.has_exceeded_threshold(&child_id));

        // Reset failures
        config.reset_failure(&child_id);
        assert_eq!(config.get_failure_count(&child_id), 0);
        assert!(!config.has_exceeded_threshold(&child_id));
    }

    #[tokio::test]
    async fn test_health_monitoring_disabled_by_default() {
        let monitor = InMemoryMonitor::new(Default::default());
        let supervisor = SupervisorNode::<OneForOne, TestChild, _>::new(OneForOne, monitor);

        assert!(!supervisor.is_health_monitoring_enabled());
        assert!(supervisor.health_config().is_none());
    }

    #[tokio::test]
    async fn test_enable_health_checks() {
        let monitor = InMemoryMonitor::new(Default::default());
        let mut supervisor = SupervisorNode::<OneForOne, TestChild, _>::new(OneForOne, monitor);

        supervisor.enable_health_checks(Duration::from_secs(30), Duration::from_secs(5), 3);

        assert!(supervisor.is_health_monitoring_enabled());

        let config = supervisor.health_config().unwrap();
        assert_eq!(config.check_interval, Duration::from_secs(30));
        assert_eq!(config.check_timeout, Duration::from_secs(5));
        assert_eq!(config.failure_threshold, 3);
    }

    #[tokio::test]
    async fn test_disable_health_checks() {
        let monitor = InMemoryMonitor::new(Default::default());
        let mut supervisor = SupervisorNode::<OneForOne, TestChild, _>::new(OneForOne, monitor);

        // Enable first
        supervisor.enable_health_checks(Duration::from_secs(30), Duration::from_secs(5), 3);
        assert!(supervisor.is_health_monitoring_enabled());

        // Disable
        supervisor.disable_health_checks();
        assert!(!supervisor.is_health_monitoring_enabled());
        assert!(supervisor.health_config().is_none());
    }

    #[tokio::test]
    async fn test_health_config_accessors() {
        let monitor = InMemoryMonitor::new(Default::default());
        let mut supervisor = SupervisorNode::<OneForOne, TestChild, _>::new(OneForOne, monitor);

        // Initially None
        assert!(supervisor.health_config().is_none());

        // Enable monitoring
        supervisor.enable_health_checks(Duration::from_secs(30), Duration::from_secs(5), 3);

        // Immutable access
        let config = supervisor.health_config().unwrap();
        assert_eq!(config.failure_threshold, 3);

        // Config should be cloneable
        let cloned_config = config.clone();
        assert_eq!(cloned_config.failure_threshold, 3);
    }

    #[tokio::test]
    async fn test_health_config_debug_format() {
        let config = HealthConfig::new(Duration::from_secs(30), Duration::from_secs(5), 3);

        let debug_str = format!("{config:?}");
        assert!(debug_str.contains("HealthConfig"));
        assert!(debug_str.contains("check_interval"));
        assert!(debug_str.contains("30s"));
        assert!(debug_str.contains("check_timeout"));
        assert!(debug_str.contains("5s"));
        assert!(debug_str.contains("failure_threshold"));
        assert!(debug_str.contains("3"));
    }

    // ========================================================================
    // Phase 4b: Health Check Logic Tests
    // ========================================================================

    // Enhanced test child for health check testing
    #[derive(Debug)]
    struct HealthTestChild {
        should_fail_start: bool,
        should_fail_stop: bool,
        health_status: ChildHealth,
    }

    #[async_trait]
    impl Child for HealthTestChild {
        type Error = TestChildError;

        async fn start(&mut self) -> Result<(), Self::Error> {
            if self.should_fail_start {
                Err(TestChildError)
            } else {
                Ok(())
            }
        }

        async fn stop(&mut self, _timeout: Duration) -> Result<(), Self::Error> {
            if self.should_fail_stop {
                Err(TestChildError)
            } else {
                Ok(())
            }
        }

        async fn health_check(&self) -> ChildHealth {
            self.health_status.clone()
        }
    }

    #[tokio::test]
    async fn test_check_child_health_success() {
        let monitor = InMemoryMonitor::new(Default::default());
        let mut supervisor =
            SupervisorNode::<OneForOne, HealthTestChild, _>::new(OneForOne, monitor);

        // Enable health monitoring
        supervisor.enable_health_checks(Duration::from_secs(30), Duration::from_secs(5), 3);

        // Start a healthy child
        let spec = ChildSpec {
            id: "healthy-child".into(),
            factory: || HealthTestChild {
                should_fail_start: false,
                should_fail_stop: false,
                health_status: ChildHealth::Healthy,
            },
            restart_policy: RestartPolicy::Permanent,
            shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
            start_timeout: Duration::from_secs(10),
            shutdown_timeout: Duration::from_secs(10),
        };

        let child_id = supervisor.start_child(spec).await.unwrap();

        // Check health
        let health = supervisor.check_child_health(&child_id).await.unwrap();
        assert!(health.is_healthy());

        // Verify failure count is 0
        let config = supervisor.health_config().unwrap();
        assert_eq!(config.get_failure_count(&child_id), 0);
    }

    #[tokio::test]
    async fn test_check_child_health_degraded() {
        let monitor = InMemoryMonitor::new(Default::default());
        let mut supervisor =
            SupervisorNode::<OneForOne, HealthTestChild, _>::new(OneForOne, monitor);

        supervisor.enable_health_checks(Duration::from_secs(30), Duration::from_secs(5), 3);

        let spec = ChildSpec {
            id: "degraded-child".into(),
            factory: || HealthTestChild {
                should_fail_start: false,
                should_fail_stop: false,
                health_status: ChildHealth::Degraded("High latency".to_string()),
            },
            restart_policy: RestartPolicy::Permanent,
            shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
            start_timeout: Duration::from_secs(10),
            shutdown_timeout: Duration::from_secs(10),
        };

        let child_id = supervisor.start_child(spec).await.unwrap();

        let health = supervisor.check_child_health(&child_id).await.unwrap();
        assert!(health.is_degraded());

        // Degraded doesn't increment failure count
        let config = supervisor.health_config().unwrap();
        assert_eq!(config.get_failure_count(&child_id), 0);
    }

    #[tokio::test]
    async fn test_check_child_health_failed_increments_count() {
        let monitor = InMemoryMonitor::new(Default::default());
        let mut supervisor =
            SupervisorNode::<OneForOne, HealthTestChild, _>::new(OneForOne, monitor);

        supervisor.enable_health_checks(Duration::from_secs(30), Duration::from_secs(5), 3);

        let spec = ChildSpec {
            id: "failing-child".into(),
            factory: || HealthTestChild {
                should_fail_start: false,
                should_fail_stop: false,
                health_status: ChildHealth::Failed("Connection error".to_string()),
            },
            restart_policy: RestartPolicy::Permanent,
            shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
            start_timeout: Duration::from_secs(10),
            shutdown_timeout: Duration::from_secs(10),
        };

        let child_id = supervisor.start_child(spec).await.unwrap();

        // First failure
        let health = supervisor.check_child_health(&child_id).await.unwrap();
        assert!(health.is_failed());
        let config = supervisor.health_config().unwrap();
        assert_eq!(config.get_failure_count(&child_id), 1);
    }

    #[tokio::test]
    async fn test_health_check_failure_threshold_triggers_restart() {
        let monitor = InMemoryMonitor::new(Default::default());
        let mut supervisor =
            SupervisorNode::<OneForOne, HealthTestChild, _>::new(OneForOne, monitor);

        // Set threshold to 3
        supervisor.enable_health_checks(Duration::from_secs(30), Duration::from_secs(5), 3);

        let spec = ChildSpec {
            id: "failing-child".into(),
            factory: || HealthTestChild {
                should_fail_start: false,
                should_fail_stop: false,
                health_status: ChildHealth::Failed("Error".to_string()),
            },
            restart_policy: RestartPolicy::Permanent,
            shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
            start_timeout: Duration::from_secs(10),
            shutdown_timeout: Duration::from_secs(10),
        };

        let child_id = supervisor.start_child(spec).await.unwrap();
        let initial_restart_count = supervisor.get_child(&child_id).unwrap().restart_count();

        // Trigger 3 health check failures
        for i in 1..=3 {
            let _ = supervisor.check_child_health(&child_id).await;
            let config = supervisor.health_config().unwrap();
            let count = config.get_failure_count(&child_id);

            if i < 3 {
                assert_eq!(count, i);
            } else {
                // On 3rd failure, restart is triggered and count is reset
                assert_eq!(count, 0);
            }
        }

        // Verify child was restarted
        let final_restart_count = supervisor.get_child(&child_id).unwrap().restart_count();
        assert_eq!(final_restart_count, initial_restart_count + 1);
    }

    #[tokio::test]
    async fn test_health_check_success_resets_failure_count() {
        let monitor = InMemoryMonitor::new(Default::default());
        let mut supervisor =
            SupervisorNode::<OneForOne, HealthTestChild, _>::new(OneForOne, monitor);

        supervisor.enable_health_checks(Duration::from_secs(30), Duration::from_secs(5), 3);

        // Start with failing child
        let spec = ChildSpec {
            id: "test-child".into(),
            factory: || HealthTestChild {
                should_fail_start: false,
                should_fail_stop: false,
                health_status: ChildHealth::Failed("Error".to_string()),
            },
            restart_policy: RestartPolicy::Permanent,
            shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
            start_timeout: Duration::from_secs(10),
            shutdown_timeout: Duration::from_secs(10),
        };

        let child_id = supervisor.start_child(spec).await.unwrap();

        // Fail twice
        supervisor.check_child_health(&child_id).await.unwrap();
        supervisor.check_child_health(&child_id).await.unwrap();

        let config = supervisor.health_config().unwrap();
        assert_eq!(config.get_failure_count(&child_id), 2);

        // Update child to be healthy (simulate recovery)
        if let Some(handle) = supervisor.get_child_mut(&child_id) {
            handle.child_mut().health_status = ChildHealth::Healthy;
        }

        // Successful check should reset count
        supervisor.check_child_health(&child_id).await.unwrap();
        let config = supervisor.health_config().unwrap();
        assert_eq!(config.get_failure_count(&child_id), 0);
    }

    #[tokio::test]
    async fn test_health_check_without_monitoring_enabled_fails() {
        let monitor = InMemoryMonitor::new(Default::default());
        let mut supervisor =
            SupervisorNode::<OneForOne, HealthTestChild, _>::new(OneForOne, monitor);

        // Don't enable health monitoring

        let spec = ChildSpec {
            id: "test-child".into(),
            factory: || HealthTestChild {
                should_fail_start: false,
                should_fail_stop: false,
                health_status: ChildHealth::Healthy,
            },
            restart_policy: RestartPolicy::Permanent,
            shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
            start_timeout: Duration::from_secs(10),
            shutdown_timeout: Duration::from_secs(10),
        };

        let child_id = supervisor.start_child(spec).await.unwrap();

        // Should return error
        let result = supervisor.check_child_health(&child_id).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SupervisorError::HealthMonitoringNotEnabled { .. }
        ));
    }

    #[tokio::test]
    async fn test_health_check_nonexistent_child_fails() {
        let monitor = InMemoryMonitor::new(Default::default());
        let mut supervisor =
            SupervisorNode::<OneForOne, HealthTestChild, _>::new(OneForOne, monitor);

        supervisor.enable_health_checks(Duration::from_secs(30), Duration::from_secs(5), 3);

        let fake_child_id = ChildId::new();
        let result = supervisor.check_child_health(&fake_child_id).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SupervisorError::ChildNotFound { .. }
        ));
    }

    // ========================================================================
    // Phase 4c: Automatic Background Health Monitoring Tests
    // ========================================================================

    #[tokio::test]
    async fn test_with_automatic_health_monitoring_returns_arc_mutex() {
        let monitor = InMemoryMonitor::new(Default::default());
        let supervisor = SupervisorNode::<OneForOne, HealthTestChild, _>::new(OneForOne, monitor);

        let monitored = supervisor.with_automatic_health_monitoring(
            Duration::from_secs(30),
            Duration::from_secs(5),
            3,
        );

        // Verify we got MonitoredSupervisor with proper Arc<Mutex<SupervisorNode>>
        let sup = monitored.supervisor.lock().await;
        assert!(sup.is_health_monitoring_enabled());
        assert_eq!(sup.child_count(), 0);
    }

    #[tokio::test]
    async fn test_automatic_monitoring_config_is_set() {
        let monitor = InMemoryMonitor::new(Default::default());
        let supervisor = SupervisorNode::<OneForOne, HealthTestChild, _>::new(OneForOne, monitor);

        let monitored = supervisor.with_automatic_health_monitoring(
            Duration::from_secs(30),
            Duration::from_secs(5),
            3,
        );

        let sup = monitored.supervisor.lock().await;
        let config = sup.health_config().unwrap();

        assert_eq!(config.check_interval, Duration::from_secs(30));
        assert_eq!(config.check_timeout, Duration::from_secs(5));
        assert_eq!(config.failure_threshold, 3);
    }

    #[tokio::test]
    async fn test_background_monitoring_checks_children_periodically() {
        let monitor = InMemoryMonitor::new(Default::default());
        let supervisor = SupervisorNode::<OneForOne, HealthTestChild, _>::new(OneForOne, monitor);

        let monitored = supervisor.with_automatic_health_monitoring(
            Duration::from_millis(100), // Fast interval for testing
            Duration::from_secs(5),
            3,
        );

        // Add a healthy child
        let spec = ChildSpec {
            id: "test-child".into(),
            factory: || HealthTestChild {
                should_fail_start: false,
                should_fail_stop: false,
                health_status: ChildHealth::Healthy,
            },
            restart_policy: RestartPolicy::Permanent,
            shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
            start_timeout: Duration::from_secs(10),
            shutdown_timeout: Duration::from_secs(10),
        };

        let child_id = {
            let mut sup = monitored.supervisor.lock().await;
            sup.start_child(spec).await.unwrap()
        };

        // Wait for multiple health check cycles
        tokio::time::sleep(Duration::from_millis(350)).await;

        // Verify child is still healthy and running
        let sup = monitored.supervisor.lock().await;
        let handle = sup.get_child(&child_id).unwrap();
        assert_eq!(handle.state(), &ChildState::Running);

        // Verify failure count is 0 (no failures detected)
        let config = sup.health_config().unwrap();
        assert_eq!(config.get_failure_count(&child_id), 0);
    }

    #[tokio::test]
    async fn test_background_monitoring_detects_and_restarts_failed_child() {
        let monitor = InMemoryMonitor::new(Default::default());
        let supervisor = SupervisorNode::<OneForOne, HealthTestChild, _>::new(OneForOne, monitor);

        let monitored = supervisor.with_automatic_health_monitoring(
            Duration::from_millis(100), // Fast interval for testing
            Duration::from_secs(5),
            2, // Low threshold for faster test
        );

        // Add a child that will fail health checks
        let spec = ChildSpec {
            id: "failing-child".into(),
            factory: || HealthTestChild {
                should_fail_start: false,
                should_fail_stop: false,
                health_status: ChildHealth::Failed("Simulated failure".to_string()),
            },
            restart_policy: RestartPolicy::Permanent,
            shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
            start_timeout: Duration::from_secs(10),
            shutdown_timeout: Duration::from_secs(10),
        };

        let child_id = {
            let mut sup = monitored.supervisor.lock().await;
            sup.start_child(spec).await.unwrap()
        };

        let initial_restart_count = {
            let sup = monitored.supervisor.lock().await;
            sup.get_child(&child_id).unwrap().restart_count()
        };

        // Wait for health checks to detect failures and trigger restart
        // interval.tick() completes immediately on first call, then waits
        // So: tick 0 (immediate) -> tick 1 (100ms) -> tick 2 (200ms) -> tick 3 (300ms)
        // We need at least 2 failed checks (threshold=2) plus processing time
        // Give it longer to be safe: 3 ticks (300ms) + processing (200ms)
        tokio::time::sleep(Duration::from_millis(600)).await;

        // Verify child was restarted
        let sup = monitored.supervisor.lock().await;
        let handle = sup.get_child(&child_id).unwrap();
        let final_restart_count = handle.restart_count();

        // Also check failure count was reset after restart
        let config = sup.health_config().unwrap();
        let failure_count = config.get_failure_count(&child_id);

        // Should have been restarted at least once
        assert!(
            final_restart_count > initial_restart_count,
            "Expected restart count to increase from {initial_restart_count} but got {final_restart_count}. Failure count: {failure_count}"
        );
    }

    #[tokio::test]
    async fn test_background_monitoring_stops_when_monitoring_disabled() {
        let monitor = InMemoryMonitor::new(Default::default());
        let supervisor = SupervisorNode::<OneForOne, HealthTestChild, _>::new(OneForOne, monitor);

        let monitored = supervisor.with_automatic_health_monitoring(
            Duration::from_millis(100),
            Duration::from_secs(5),
            3,
        );

        // Add a child
        let spec = ChildSpec {
            id: "test-child".into(),
            factory: || HealthTestChild {
                should_fail_start: false,
                should_fail_stop: false,
                health_status: ChildHealth::Healthy,
            },
            restart_policy: RestartPolicy::Permanent,
            shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
            start_timeout: Duration::from_secs(10),
            shutdown_timeout: Duration::from_secs(10),
        };

        {
            let mut sup = monitored.supervisor.lock().await;
            sup.start_child(spec).await.unwrap();
        }

        // Wait for health checks to run
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Disable health monitoring
        {
            let mut sup = monitored.supervisor.lock().await;
            sup.disable_health_checks();
        }

        // Wait a bit more
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Verify monitoring is disabled
        let sup = monitored.supervisor.lock().await;
        assert!(!sup.is_health_monitoring_enabled());
    }

    #[tokio::test]
    async fn test_automatic_monitoring_handles_multiple_children() {
        let monitor = InMemoryMonitor::new(Default::default());
        let supervisor = SupervisorNode::<OneForOne, HealthTestChild, _>::new(OneForOne, monitor);

        let monitored = supervisor.with_automatic_health_monitoring(
            Duration::from_millis(100),
            Duration::from_secs(5),
            3,
        );

        // Add multiple children with different health states
        let mut child_ids = Vec::new();

        for i in 0..5 {
            let health_status = if i % 2 == 0 {
                ChildHealth::Healthy
            } else {
                ChildHealth::Degraded("Minor issue".to_string())
            };

            let spec = ChildSpec {
                id: format!("child-{i}"),
                factory: move || HealthTestChild {
                    should_fail_start: false,
                    should_fail_stop: false,
                    health_status: health_status.clone(),
                },
                restart_policy: RestartPolicy::Permanent,
                shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
                start_timeout: Duration::from_secs(10),
                shutdown_timeout: Duration::from_secs(10),
            };

            let child_id = {
                let mut sup = monitored.supervisor.lock().await;
                sup.start_child(spec).await.unwrap()
            };
            child_ids.push(child_id);
        }

        // Wait for health checks
        tokio::time::sleep(Duration::from_millis(250)).await;

        // Verify all children are still running
        let sup = monitored.supervisor.lock().await;
        for child_id in &child_ids {
            let handle = sup.get_child(child_id).unwrap();
            assert_eq!(handle.state(), &ChildState::Running);
        }
    }
}
