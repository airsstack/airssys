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
//! use airssys_rt::supervisor::{OneForOne, OneForAll, RestForOne};
//!
//! // Independent children - use OneForOne
//! let supervisor = SupervisorNode::new(OneForOne, monitor);
//!
//! // Interdependent children - use OneForAll
//! let supervisor = SupervisorNode::new(OneForAll, monitor);
//!
//! // Dependent startup order - use RestForOne
//! let supervisor = SupervisorNode::new(RestForOne, monitor);
//! ```

// Layer 1: Standard library imports
use std::collections::HashMap;

// Layer 2: Third-party crate imports
// (none needed for Phase 2)

// Layer 3: Internal module imports
use super::traits::SupervisionStrategy;
use super::types::{ChildId, RestartPolicy};

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
/// use airssys_rt::supervisor::{OneForOne, ChildSpec, RestartPolicy};
///
/// // Strategy for independent HTTP handlers
/// let supervisor = SupervisorNode::new(OneForOne, monitor);
///
/// // Add independent children
/// supervisor.start_child(ChildSpec {
///     id: "handler-1".into(),
///     restart_policy: RestartPolicy::Permanent,
///     // ... other config
/// }).await?;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OneForOne;

impl SupervisionStrategy for OneForOne {
    // Implementation will be added in Phase 3 when SupervisorNode is complete
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
/// use airssys_rt::supervisor::{OneForAll, ChildSpec, RestartPolicy};
///
/// // Strategy for interdependent services
/// let supervisor = SupervisorNode::new(OneForAll, monitor);
///
/// // Add interdependent children
/// supervisor.start_child(ChildSpec {
///     id: "db-pool".into(),
///     restart_policy: RestartPolicy::Permanent,
///     // ... other config
/// }).await?;
///
/// supervisor.start_child(ChildSpec {
///     id: "cache-manager".into(),
///     restart_policy: RestartPolicy::Permanent,
///     // ... other config
/// }).await?;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OneForAll;

impl SupervisionStrategy for OneForAll {
    // Implementation will be added in Phase 3 when SupervisorNode is complete
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
/// use airssys_rt::supervisor::{RestForOne, ChildSpec, RestartPolicy};
///
/// // Strategy for dependent startup sequence
/// let supervisor = SupervisorNode::new(RestForOne, monitor);
///
/// // Order matters! Later children depend on earlier ones
/// supervisor.start_child(ChildSpec {
///     id: "config-loader".into(),      // Started first
///     restart_policy: RestartPolicy::Permanent,
///     // ... other config
/// }).await?;
///
/// supervisor.start_child(ChildSpec {
///     id: "database".into(),            // Depends on config
///     restart_policy: RestartPolicy::Permanent,
///     // ... other config
/// }).await?;
///
/// supervisor.start_child(ChildSpec {
///     id: "api-server".into(),          // Depends on database
///     restart_policy: RestartPolicy::Permanent,
///     // ... other config
/// }).await?;
///
/// // If database fails, it and api-server will restart
/// // If api-server fails, only it restarts
/// // If config-loader fails, all three restart
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RestForOne;

impl SupervisionStrategy for RestForOne {
    // Implementation will be added in Phase 3 when SupervisorNode is complete
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
/// Used by OneForAll strategy to decide if the entire supervision group
/// should restart. If all children are Temporary, no restart occurs.
///
/// # Parameters
///
/// - `children_policies`: Map of child IDs to their restart policies
/// - `is_normal_exit`: Whether the triggering child exited normally
///
/// # Returns
///
/// `true` if at least one child policy allows restart, `false` otherwise
///
/// # Examples
///
/// ```rust
/// use airssys_rt::supervisor::{RestartPolicy, ChildId, should_restart_any};
/// use std::collections::HashMap;
///
/// let mut policies = HashMap::new();
/// policies.insert(ChildId::new(), RestartPolicy::Permanent);
/// policies.insert(ChildId::new(), RestartPolicy::Temporary);
///
/// // At least one Permanent child, so group should restart
/// assert!(should_restart_any(&policies, false));
///
/// let mut temp_only = HashMap::new();
/// temp_only.insert(ChildId::new(), RestartPolicy::Temporary);
/// temp_only.insert(ChildId::new(), RestartPolicy::Temporary);
///
/// // All Temporary, so group should not restart
/// assert!(!should_restart_any(&temp_only, false));
/// ```
pub fn should_restart_any(
    children_policies: &HashMap<ChildId, RestartPolicy>,
    is_normal_exit: bool,
) -> bool {
    children_policies
        .values()
        .any(|policy| should_restart(policy, is_normal_exit))
}

#[cfg(test)]
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
        let mut policies = HashMap::new();
        policies.insert(ChildId::new(), RestartPolicy::Permanent);
        policies.insert(ChildId::new(), RestartPolicy::Temporary);

        // At least one Permanent child
        assert!(should_restart_any(&policies, false));
        assert!(should_restart_any(&policies, true));
    }

    #[test]
    fn test_should_restart_any_with_transient() {
        let mut policies = HashMap::new();
        policies.insert(ChildId::new(), RestartPolicy::Transient);
        policies.insert(ChildId::new(), RestartPolicy::Temporary);

        // Transient restarts on abnormal exit
        assert!(should_restart_any(&policies, false));
        assert!(!should_restart_any(&policies, true));
    }

    #[test]
    fn test_should_restart_any_all_temporary() {
        let mut policies = HashMap::new();
        policies.insert(ChildId::new(), RestartPolicy::Temporary);
        policies.insert(ChildId::new(), RestartPolicy::Temporary);

        // All Temporary, no restart
        assert!(!should_restart_any(&policies, false));
        assert!(!should_restart_any(&policies, true));
    }

    #[test]
    fn test_should_restart_any_empty() {
        let policies = HashMap::new();

        // No children, no restart
        assert!(!should_restart_any(&policies, false));
        assert!(!should_restart_any(&policies, true));
    }

    #[test]
    fn test_one_for_one_strategy_marker() {
        // Test that OneForOne is a valid marker type
        let strategy = OneForOne;
        let strategy2 = OneForOne;

        assert_eq!(strategy, strategy2);
        assert_eq!(format!("{:?}", strategy), "OneForOne");
    }

    #[test]
    fn test_one_for_all_strategy_marker() {
        // Test that OneForAll is a valid marker type
        let strategy = OneForAll;
        let strategy2 = OneForAll;

        assert_eq!(strategy, strategy2);
        assert_eq!(format!("{:?}", strategy), "OneForAll");
    }

    #[test]
    fn test_rest_for_one_strategy_marker() {
        // Test that RestForOne is a valid marker type
        let strategy = RestForOne;
        let strategy2 = RestForOne;

        assert_eq!(strategy, strategy2);
        assert_eq!(format!("{:?}", strategy), "RestForOne");
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
