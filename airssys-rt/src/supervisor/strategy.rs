//! Supervision strategy implementations.
//!
//! This module implements the three BEAM/Erlang-inspired supervision strategies:
//! - **OneForOne**: Restart only the failed child
//! - **OneForAll**: Restart all children when one fails
//! - **RestForOne**: Restart the failed child and all children started after it
//!
//! # Strategy Selection Guide
//!
//! ## OneForOne
//! Use when children are independent. One child's failure doesn't affect others.
//! **Example:** HTTP request handlers, independent background tasks
//!
//! ## OneForAll
//! Use when children are interdependent. If one fails, all need to restart.
//! **Example:** Database connection pool, service mesh components
//!
//! ## RestForOne
//! Use when children have startup dependencies. Later children depend on earlier ones.
//! **Example:** Config loader → Database → API server (restart order matters)
//!
//! # Examples
//!
//! ```rust
//! use airssys_rt::supervisor::{SupervisorNode, OneForOne, OneForAll, RestForOne};
//! use airssys_rt::monitoring::InMemoryMonitor;
//!
//! # use airssys_rt::supervisor::Child;
//! # use async_trait::async_trait;
//! # use std::time::Duration;
//! # struct MyWorker;
//! # #[derive(Debug)]
//! # struct MyError;
//! # impl std::fmt::Display for MyError {
//! #     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { Ok(()) }
//! # }
//! # impl std::error::Error for MyError {}
//! # #[async_trait]
//! # impl Child for MyWorker {
//! #     type Error = MyError;
//! #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
//! #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
//! # }
//! # fn example() {
//! let monitor = InMemoryMonitor::new(Default::default());
//!
//! // Independent children - use OneForOne
//! let supervisor1 = SupervisorNode::<OneForOne, MyWorker, _>::new(OneForOne, monitor.clone());
//!
//! // Interdependent children - use OneForAll
//! let supervisor2 = SupervisorNode::<OneForAll, MyWorker, _>::new(OneForAll, monitor.clone());
//!
//! // Dependent startup order - use RestForOne
//! let supervisor3 = SupervisorNode::<RestForOne, MyWorker, _>::new(RestForOne, monitor);
//! # }
//! ```

// Layer 1: Standard library imports
// (none needed for Phase 2)

// Layer 2: Third-party crate imports
// (none needed for Phase 2)

// Layer 3: Internal module imports
use super::traits::SupervisionStrategy;
use super::types::{RestartPolicy, StrategyContext, SupervisionDecision};

/// OneForOne supervision strategy.
///
/// When a child fails, only that specific child is restarted. Other children
/// continue running unaffected. This is the most common strategy for independent
/// processes that don't share state or resources.
///
/// # BEAM/OTP Alignment
///
/// Matches Erlang's `:one_for_one` supervision strategy.
///
/// # Decision Logic
///
/// 1. Child fails with error
/// 2. Check child's `RestartPolicy`:
///    - `Permanent`: Always restart
///    - `Transient`: Restart only if error is abnormal (not normal exit)
///    - `Temporary`: Never restart, just stop the child
/// 3. If restart allowed, check restart rate limits
/// 4. Restart only the failed child
///
/// # Examples
///
/// ```rust
/// use airssys_rt::supervisor::{SupervisorNode, OneForOne, ChildSpec, RestartPolicy, ShutdownPolicy, Supervisor};
/// use airssys_rt::monitoring::InMemoryMonitor;
/// use std::time::Duration;
///
/// # use airssys_rt::supervisor::Child;
/// # use async_trait::async_trait;
/// # struct HttpHandler;
/// # #[derive(Debug)]
/// # struct HttpError;
/// # impl std::fmt::Display for HttpError {
/// #     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { Ok(()) }
/// # }
/// # impl std::error::Error for HttpError {}
/// # #[async_trait]
/// # impl Child for HttpHandler {
/// #     type Error = HttpError;
/// #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
/// #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
/// # }
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Strategy for independent HTTP handlers
/// let monitor = InMemoryMonitor::new(Default::default());
/// let mut supervisor = SupervisorNode::<OneForOne, HttpHandler, _>::new(OneForOne, monitor);
///
/// // Add independent children
/// supervisor.start_child(ChildSpec {
///     id: "handler-1".into(),
///     factory: || HttpHandler,
///     restart_policy: RestartPolicy::Permanent,
///     shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
///     start_timeout: Duration::from_secs(10),
///     shutdown_timeout: Duration::from_secs(10),
/// }).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OneForOne;

impl SupervisionStrategy for OneForOne {
    fn determine_decision(context: StrategyContext) -> SupervisionDecision {
        // OneForOne: Restart only the failed child
        match context {
            StrategyContext::SingleFailure {
                failed_child_id, ..
            } => SupervisionDecision::RestartChild(failed_child_id),
            StrategyContext::ManualRestart { child_id } => {
                SupervisionDecision::RestartChild(child_id)
            }
            StrategyContext::Shutdown { .. } => SupervisionDecision::StopAll,
        }
    }
}

/// OneForAll supervision strategy.
///
/// When any child fails, ALL children are stopped and then restarted in their
/// original start order. This strategy is for tightly coupled processes where
/// one failure indicates the entire subsystem needs to restart.
///
/// # BEAM/OTP Alignment
///
/// Matches Erlang's `:one_for_all` supervision strategy.
///
/// # Decision Logic
///
/// 1. Child fails with error
/// 2. Check if ANY child's `RestartPolicy` allows restart:
///    - If all are `Temporary`, stop all without restart
///    - If any is `Permanent` or `Transient` (with abnormal error), restart all
/// 3. Stop all children (in reverse start order)
/// 4. Restart all children (in original start order)
///
/// # Use Cases
///
/// - **Database connection pools**: All connections share state
/// - **Service mesh components**: Interdependent services
/// - **Stateful protocols**: TCP connections with shared session state
///
/// # Examples
///
/// ```rust
/// use airssys_rt::supervisor::{SupervisorNode, OneForAll, ChildSpec, RestartPolicy, ShutdownPolicy, Supervisor};
/// use airssys_rt::monitoring::InMemoryMonitor;
/// use std::time::Duration;
///
/// # use airssys_rt::supervisor::Child;
/// # use async_trait::async_trait;
/// # struct PoolWorker;
/// # #[derive(Debug)]
/// # struct PoolError;
/// # impl std::fmt::Display for PoolError {
/// #     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { Ok(()) }
/// # }
/// # impl std::error::Error for PoolError {}
/// # #[async_trait]
/// # impl Child for PoolWorker {
/// #     type Error = PoolError;
/// #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
/// #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
/// # }
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Strategy for interdependent services
/// let monitor = InMemoryMonitor::new(Default::default());
/// let mut supervisor = SupervisorNode::<OneForAll, PoolWorker, _>::new(OneForAll, monitor);
///
/// // Add interdependent children
/// supervisor.start_child(ChildSpec {
///     id: "db-pool".into(),
///     factory: || PoolWorker,
///     restart_policy: RestartPolicy::Permanent,
///     shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
///     start_timeout: Duration::from_secs(10),
///     shutdown_timeout: Duration::from_secs(10),
/// }).await?;
///
/// supervisor.start_child(ChildSpec {
///     id: "cache-manager".into(),
///     factory: || PoolWorker,
///     restart_policy: RestartPolicy::Permanent,
///     shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
///     start_timeout: Duration::from_secs(10),
///     shutdown_timeout: Duration::from_secs(10),
/// }).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OneForAll;

impl SupervisionStrategy for OneForAll {
    fn determine_decision(context: StrategyContext) -> SupervisionDecision {
        // OneForAll: Restart all children
        match context {
            StrategyContext::SingleFailure { all_child_ids, .. } => {
                SupervisionDecision::RestartAll(all_child_ids)
            }
            StrategyContext::ManualRestart { child_id } => {
                // Manual restart of one child in OneForAll: restart all
                // (In practice, this might not be triggered, but being defensive)
                SupervisionDecision::RestartChild(child_id)
            }
            StrategyContext::Shutdown { .. } => SupervisionDecision::StopAll,
        }
    }
}

/// RestForOne supervision strategy.
///
/// When a child fails, that child and all children started AFTER it are stopped
/// and restarted in their original order. This strategy handles startup dependencies
/// where later children depend on earlier ones being healthy.
///
/// # BEAM/OTP Alignment
///
/// Matches Erlang's `:rest_for_one` supervision strategy.
///
/// # Decision Logic
///
/// 1. Child fails with error
/// 2. Identify all children started after the failed child
/// 3. Check restart policies for failed child and subsequent children
/// 4. Stop failed child and all subsequent children (reverse order)
/// 5. Restart failed child and subsequent children (original order)
///
/// # Use Cases
///
/// - **Dependency chains**: Config → Database → API Server
/// - **Resource initialization**: Logger → Metrics → Application
/// - **Service startup sequences**: Auth Service → User Service → API Gateway
///
/// # Examples
///
/// ```rust
/// use airssys_rt::supervisor::{SupervisorNode, RestForOne, ChildSpec, RestartPolicy, ShutdownPolicy, Supervisor};
/// use airssys_rt::monitoring::InMemoryMonitor;
/// use std::time::Duration;
///
/// # use airssys_rt::supervisor::Child;
/// # use async_trait::async_trait;
/// # struct ServiceWorker;
/// # #[derive(Debug)]
/// # struct ServiceError;
/// # impl std::fmt::Display for ServiceError {
/// #     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { Ok(()) }
/// # }
/// # impl std::error::Error for ServiceError {}
/// # #[async_trait]
/// # impl Child for ServiceWorker {
/// #     type Error = ServiceError;
/// #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
/// #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
/// # }
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Strategy for dependent startup sequence
/// let monitor = InMemoryMonitor::new(Default::default());
/// let mut supervisor = SupervisorNode::<RestForOne, ServiceWorker, _>::new(RestForOne, monitor);
///
/// // Order matters! Later children depend on earlier ones
/// supervisor.start_child(ChildSpec {
///     id: "config-loader".into(),      // Started first
///     factory: || ServiceWorker,
///     restart_policy: RestartPolicy::Permanent,
///     shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
///     start_timeout: Duration::from_secs(10),
///     shutdown_timeout: Duration::from_secs(10),
/// }).await?;
///
/// supervisor.start_child(ChildSpec {
///     id: "database".into(),            // Depends on config
///     factory: || ServiceWorker,
///     restart_policy: RestartPolicy::Permanent,
///     shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
///     start_timeout: Duration::from_secs(10),
///     shutdown_timeout: Duration::from_secs(10),
/// }).await?;
///
/// supervisor.start_child(ChildSpec {
///     id: "api-server".into(),          // Depends on database
///     factory: || ServiceWorker,
///     restart_policy: RestartPolicy::Permanent,
///     shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
///     start_timeout: Duration::from_secs(10),
///     shutdown_timeout: Duration::from_secs(10),
/// }).await?;
/// # Ok(())
/// # }
///
/// // If database fails, it and api-server will restart
/// // If api-server fails, only it restarts
/// // If config-loader fails, all three restart
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RestForOne;

impl SupervisionStrategy for RestForOne {
    fn determine_decision(context: StrategyContext) -> SupervisionDecision {
        // RestForOne: Restart failed child and all children started after it
        match context {
            StrategyContext::SingleFailure {
                failed_child_id,
                all_child_ids,
            } => {
                if let Some(index) = all_child_ids.iter().position(|id| id == &failed_child_id) {
                    // Include the failed child and all after it
                    SupervisionDecision::RestartSubset(all_child_ids[index..].to_vec())
                } else {
                    // Child not found, just restart it
                    SupervisionDecision::RestartChild(failed_child_id)
                }
            }
            StrategyContext::ManualRestart { child_id } => {
                // Manual restart: just restart the specified child
                SupervisionDecision::RestartChild(child_id)
            }
            StrategyContext::Shutdown { .. } => SupervisionDecision::StopAll,
        }
    }
}

/// Determine if a child should be restarted based on its restart policy.
///
/// # Restart Policy Logic
///
/// - **Permanent**: Always restart, regardless of error
/// - **Transient**: Restart only if error is abnormal (not normal exit)
/// - **Temporary**: Never restart
///
/// # Parameters
///
/// - `policy`: The child's configured restart policy
/// - `is_normal_exit`: Whether the child exited normally (no error)
///
/// # Returns
///
/// `true` if the child should be restarted, `false` otherwise
///
/// # Examples
///
/// ```rust
/// use airssys_rt::supervisor::{RestartPolicy, should_restart};
///
/// // Permanent child always restarts
/// assert!(should_restart(&RestartPolicy::Permanent, false));
/// assert!(should_restart(&RestartPolicy::Permanent, true));
///
/// // Transient child restarts only on abnormal exit
/// assert!(should_restart(&RestartPolicy::Transient, false));
/// assert!(!should_restart(&RestartPolicy::Transient, true));
///
/// // Temporary child never restarts
/// assert!(!should_restart(&RestartPolicy::Temporary, false));
/// assert!(!should_restart(&RestartPolicy::Temporary, true));
/// ```
pub fn should_restart(policy: &RestartPolicy, is_normal_exit: bool) -> bool {
    match policy {
        RestartPolicy::Permanent => true,
        RestartPolicy::Transient => !is_normal_exit,
        RestartPolicy::Temporary => false,
    }
}

/// Determine if at least one child in a set should be restarted.
///
/// Used by custom strategies to decide if a supervision group
/// should restart. If all children are Temporary, no restart occurs.
///
/// **Note**: This is a utility function for custom strategy implementations.
/// The built-in strategies (OneForOne, OneForAll, RestForOne) don't use this
/// directly as restart policy checking happens at the supervisor node level.
///
/// # Parameters
///
/// - `children_policies`: Iterator over restart policies
/// - `is_normal_exit`: Whether the triggering child exited normally
///
/// # Returns
///
/// `true` if at least one child policy allows restart, `false` otherwise
///
/// # Examples
///
/// ```rust
/// use airssys_rt::supervisor::{RestartPolicy, should_restart_any};
///
/// let policies = vec![RestartPolicy::Permanent, RestartPolicy::Temporary];
///
/// // At least one Permanent child, so group should restart
/// assert!(should_restart_any(policies.iter(), false));
///
/// let temp_only = vec![RestartPolicy::Temporary, RestartPolicy::Temporary];
///
/// // All Temporary, so group should not restart
/// assert!(!should_restart_any(temp_only.iter(), false));
/// ```
pub fn should_restart_any<'a, I>(mut children_policies: I, is_normal_exit: bool) -> bool
where
    I: Iterator<Item = &'a RestartPolicy>,
{
    children_policies.any(|policy| should_restart(policy, is_normal_exit))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_should_restart_permanent() {
        // Permanent always restarts
        assert!(should_restart(&RestartPolicy::Permanent, false));
        assert!(should_restart(&RestartPolicy::Permanent, true));
    }

    #[test]
    fn test_should_restart_transient() {
        // Transient restarts only on abnormal exit
        assert!(should_restart(&RestartPolicy::Transient, false));
        assert!(!should_restart(&RestartPolicy::Transient, true));
    }

    #[test]
    fn test_should_restart_temporary() {
        // Temporary never restarts
        assert!(!should_restart(&RestartPolicy::Temporary, false));
        assert!(!should_restart(&RestartPolicy::Temporary, true));
    }

    #[test]
    fn test_should_restart_any_with_permanent() {
        let policies = [RestartPolicy::Permanent, RestartPolicy::Temporary];

        // At least one Permanent child
        assert!(should_restart_any(policies.iter(), false));
        assert!(should_restart_any(policies.iter(), true));
    }

    #[test]
    fn test_should_restart_any_with_transient() {
        let policies = [RestartPolicy::Transient, RestartPolicy::Temporary];

        // Transient restarts on abnormal exit
        assert!(should_restart_any(policies.iter(), false));
        assert!(!should_restart_any(policies.iter(), true));
    }

    #[test]
    fn test_should_restart_any_all_temporary() {
        let policies = [RestartPolicy::Temporary, RestartPolicy::Temporary];

        // All Temporary, no restart
        assert!(!should_restart_any(policies.iter(), false));
        assert!(!should_restart_any(policies.iter(), true));
    }

    #[test]
    fn test_should_restart_any_empty() {
        let policies: Vec<RestartPolicy> = vec![];

        // No children, no restart
        assert!(!should_restart_any(policies.iter(), false));
        assert!(!should_restart_any(policies.iter(), true));
    }

    #[test]
    fn test_one_for_one_strategy_marker() {
        // Test that OneForOne is a valid marker type
        let strategy = OneForOne;
        let strategy2 = OneForOne;

        assert_eq!(strategy, strategy2);
        assert_eq!(format!("{strategy:?}"), "OneForOne");
    }

    #[test]
    fn test_one_for_all_strategy_marker() {
        // Test that OneForAll is a valid marker type
        let strategy = OneForAll;
        let strategy2 = OneForAll;

        assert_eq!(strategy, strategy2);
        assert_eq!(format!("{strategy:?}"), "OneForAll");
    }

    #[test]
    fn test_rest_for_one_strategy_marker() {
        // Test that RestForOne is a valid marker type
        let strategy = RestForOne;
        let strategy2 = RestForOne;

        assert_eq!(strategy, strategy2);
        assert_eq!(format!("{strategy:?}"), "RestForOne");
    }

    #[test]
    fn test_strategy_markers_are_copy() {
        // Verify all strategies implement Copy
        let s1 = OneForOne;
        let _s2 = s1; // Copy
        let _s3 = s1; // Still valid

        let s1 = OneForAll;
        let _s2 = s1; // Copy
        let _s3 = s1; // Still valid

        let s1 = RestForOne;
        let _s2 = s1; // Copy
        let _s3 = s1; // Still valid
    }

    #[test]
    fn test_strategy_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<OneForOne>();
        assert_send_sync::<OneForAll>();
        assert_send_sync::<RestForOne>();
    }
}
