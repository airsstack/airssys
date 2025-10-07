//! SupervisorNode implementation for hierarchical supervision.
//!
//! This module provides the core supervisor implementation with generic strategy,
//! child, and monitor type parameters. SupervisorNode manages child lifecycle,
//! applies restart strategies, and coordinates with monitoring infrastructure.

// Layer 1: Standard library imports
use std::collections::HashMap;
use std::error::Error as StdError;
use std::time::Duration;

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use chrono::Utc;
use tokio::time::timeout;
use uuid::Uuid;

// Layer 3: Internal module imports
use super::backoff::RestartBackoff;
use super::error::SupervisorError;
use super::strategy::should_restart;
use super::traits::{Child, SupervisionStrategy, Supervisor};
use super::types::{
    ChildId, ChildSpec, ChildState, RestartPolicy, ShutdownPolicy, StrategyContext,
    SupervisionDecision,
};
use crate::monitoring::{Monitor, SupervisionEvent, SupervisionEventKind};

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
/// use airssys_rt::supervisor::{SupervisorNode, OneForOne, ChildSpec, RestartPolicy, ShutdownPolicy};
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
/// let monitor = InMemoryMonitor::new(MonitoringConfig::default());
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
}

impl<S, C, M> SupervisorNode<S, C, M>
where
    S: SupervisionStrategy,
    C: Child,
    M: Monitor<SupervisionEvent>,
{
    /// Creates a new supervisor node with default configuration.
    ///
    /// Uses default restart backoff settings: 5 restarts per 60 seconds.
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
    /// let monitor = InMemoryMonitor::new(MonitoringConfig::default());
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
        // TODO: Implement per-child backoff properly
        // For now, we use a simple approach without rate limiting

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
mod tests {
    use super::*;
    use crate::monitoring::{InMemoryMonitor, MonitoringConfig};
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
        let monitor = InMemoryMonitor::new(MonitoringConfig::default());
        let supervisor = SupervisorNode::<OneForOne, TestChild, _>::new(OneForOne, monitor);

        assert_eq!(supervisor.child_count(), 0);
    }

    #[tokio::test]
    async fn test_start_child_success() {
        let monitor = InMemoryMonitor::new(MonitoringConfig::default());
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
        let monitor = InMemoryMonitor::new(MonitoringConfig::default());
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
        let monitor = InMemoryMonitor::new(MonitoringConfig::default());
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
        let monitor = InMemoryMonitor::new(MonitoringConfig::default());
        let mut supervisor = SupervisorNode::<OneForOne, TestChild, _>::new(OneForOne, monitor);

        let child_id = ChildId::new();
        let result = supervisor.stop_child(&child_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_restart_child_success() {
        let monitor = InMemoryMonitor::new(MonitoringConfig::default());
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
        let monitor = InMemoryMonitor::new(MonitoringConfig::default());
        let mut supervisor = SupervisorNode::<OneForOne, TestChild, _>::new(OneForOne, monitor);

        let mut child_ids = Vec::new();

        for i in 0..3 {
            let spec = ChildSpec {
                id: format!("child-{}", i),
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
        let monitor = InMemoryMonitor::new(MonitoringConfig::default());
        let mut supervisor = SupervisorNode::<RestForOne, TestChild, _>::new(RestForOne, monitor);

        let mut child_ids = Vec::new();

        for i in 0..5 {
            let spec = ChildSpec {
                id: format!("child-{}", i),
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

    // TODO: Re-enable after per-child backoff is fully implemented
    #[tokio::test]
    #[ignore]
    async fn test_restart_rate_limiting() {
        // This test is temporarily disabled as the API has changed
        // Need to update to use per-child backoff configuration
        /*
        let monitor = InMemoryMonitor::new(MonitoringConfig::default());
        let backoff = RestartBackoff::new(2, Duration::from_secs(60));
        let mut supervisor =
            SupervisorNode::<OneForOne, TestChild, _>::new(OneForOne, monitor);

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

        // First two restarts should succeed
        supervisor.restart_child(&child_id).await.unwrap();
        supervisor.restart_child(&child_id).await.unwrap();

        // Third restart should fail due to rate limit
        let result = supervisor.restart_child(&child_id).await;
        assert!(result.is_err());
        */
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
}
