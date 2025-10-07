//! Type definitions for supervisor framework.
//!
//! This module provides all core types used by the supervisor framework,
//! including child specifications, restart policies, health status, and
//! supervision decisions.

// Layer 1: Standard library imports
use std::fmt;
use std::time::Duration;

// Layer 2: Third-party crate imports
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a child process.
///
/// Child IDs are UUID-based to ensure uniqueness across the supervisor tree.
///
/// # Examples
///
/// ```rust
/// use airssys_rt::supervisor::ChildId;
///
/// let id1 = ChildId::new();
/// let id2 = ChildId::new();
///
/// assert_ne!(id1, id2);  // Each ID is unique
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChildId(Uuid);

impl ChildId {
    /// Creates a new unique child ID.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::supervisor::ChildId;
    ///
    /// let id = ChildId::new();
    /// println!("Child ID: {}", id);
    /// ```
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Returns the underlying UUID.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for ChildId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ChildId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for ChildId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

/// Child process specification.
///
/// Defines how a child should be started, stopped, and restarted. This
/// struct uses generic type parameters to avoid `Box<dyn Trait>` patterns
/// (§6.2), enabling zero-cost abstractions.
///
/// # Type Parameters
///
/// - `C`: Child type implementing the `Child` trait
/// - `F`: Factory function type that creates new child instances
///
/// # Examples
///
/// ```rust
/// use airssys_rt::supervisor::{ChildSpec, RestartPolicy, ShutdownPolicy};
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
/// let spec = ChildSpec {
///     id: "worker-1".into(),
///     factory: || MyWorker,
///     restart_policy: RestartPolicy::Permanent,
///     shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
///     start_timeout: Duration::from_secs(10),
///     shutdown_timeout: Duration::from_secs(10),
/// };
/// ```
#[derive(Debug)]
pub struct ChildSpec<C, F>
where
    F: Fn() -> C + Send + Sync + 'static,
{
    /// Unique identifier for this child (for logging and monitoring)
    pub id: String,

    /// Factory function that creates new child instances.
    ///
    /// Generic function type avoids `Box<dyn Fn()>` for zero-cost abstraction (§6.2).
    pub factory: F,

    /// Restart policy determining when to restart this child
    pub restart_policy: RestartPolicy,

    /// Shutdown policy determining how to stop this child
    pub shutdown_policy: ShutdownPolicy,

    /// Maximum time to wait for child startup
    pub start_timeout: Duration,

    /// Maximum time to wait for child shutdown
    pub shutdown_timeout: Duration,
}

/// Restart policy for supervised children.
///
/// Determines when a child should be restarted after termination.
/// Based on Erlang/OTP supervisor restart policies.
///
/// # Examples
///
/// ```rust
/// use airssys_rt::supervisor::RestartPolicy;
///
/// let permanent = RestartPolicy::Permanent;
/// assert!(permanent.should_restart(true));   // Restart on error
/// assert!(permanent.should_restart(false));  // Restart on normal exit
///
/// let transient = RestartPolicy::Transient;
/// assert!(transient.should_restart(true));   // Restart on error
/// assert!(!transient.should_restart(false)); // Don't restart on normal exit
///
/// let temporary = RestartPolicy::Temporary;
/// assert!(!temporary.should_restart(true));  // Never restart
/// assert!(!temporary.should_restart(false)); // Never restart
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RestartPolicy {
    /// Always restart the child, regardless of exit reason.
    ///
    /// Use for critical services that must always be running.
    Permanent,

    /// Restart only if the child exits abnormally (with an error).
    ///
    /// Use for workers that may complete successfully and shouldn't restart.
    Transient,

    /// Never restart the child, regardless of exit reason.
    ///
    /// Use for one-shot tasks or temporary processes.
    Temporary,
}

impl RestartPolicy {
    /// Returns `true` if this policy should restart on the given exit condition.
    ///
    /// # Parameters
    ///
    /// - `is_error`: `true` if the child exited with an error, `false` for normal exit
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::supervisor::RestartPolicy;
    ///
    /// assert!(RestartPolicy::Permanent.should_restart(true));
    /// assert!(RestartPolicy::Permanent.should_restart(false));
    ///
    /// assert!(RestartPolicy::Transient.should_restart(true));
    /// assert!(!RestartPolicy::Transient.should_restart(false));
    ///
    /// assert!(!RestartPolicy::Temporary.should_restart(true));
    /// assert!(!RestartPolicy::Temporary.should_restart(false));
    /// ```
    pub fn should_restart(&self, is_error: bool) -> bool {
        match self {
            RestartPolicy::Permanent => true,
            RestartPolicy::Transient => is_error,
            RestartPolicy::Temporary => false,
        }
    }
}

/// Shutdown policy for supervised children.
///
/// Determines how a child should be stopped when requested.
///
/// # Examples
///
/// ```rust
/// use airssys_rt::supervisor::ShutdownPolicy;
/// use std::time::Duration;
///
/// let graceful = ShutdownPolicy::Graceful(Duration::from_secs(5));
/// let immediate = ShutdownPolicy::Immediate;
/// let infinity = ShutdownPolicy::Infinity;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShutdownPolicy {
    /// Graceful shutdown with timeout.
    ///
    /// Attempt graceful shutdown, forcefully terminate if timeout expires.
    Graceful(Duration),

    /// Immediate forceful termination.
    ///
    /// No graceful shutdown attempt, terminate immediately.
    Immediate,

    /// Wait indefinitely for graceful shutdown.
    ///
    /// No timeout, wait forever for child to stop gracefully.
    Infinity,
}

impl ShutdownPolicy {
    /// Returns the timeout duration for this policy, if any.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::supervisor::ShutdownPolicy;
    /// use std::time::Duration;
    ///
    /// let graceful = ShutdownPolicy::Graceful(Duration::from_secs(5));
    /// assert_eq!(graceful.timeout(), Some(Duration::from_secs(5)));
    ///
    /// let immediate = ShutdownPolicy::Immediate;
    /// assert_eq!(immediate.timeout(), Some(Duration::ZERO));
    ///
    /// let infinity = ShutdownPolicy::Infinity;
    /// assert_eq!(infinity.timeout(), None);
    /// ```
    pub fn timeout(&self) -> Option<Duration> {
        match self {
            ShutdownPolicy::Graceful(duration) => Some(*duration),
            ShutdownPolicy::Immediate => Some(Duration::ZERO),
            ShutdownPolicy::Infinity => None,
        }
    }

    /// Returns `true` if this is a graceful shutdown policy.
    pub fn is_graceful(&self) -> bool {
        !matches!(self, ShutdownPolicy::Immediate)
    }
}

/// Current state of a supervised child.
///
/// Tracks the lifecycle state of a child process within the supervisor.
///
/// # State Transitions
///
/// ```text
/// Starting → Running → Stopping → Stopped
///     ↓         ↓
/// Failed    Restarting → Starting
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChildState {
    /// Child is in the process of starting
    Starting,

    /// Child is running normally
    Running,

    /// Child is in the process of stopping
    Stopping,

    /// Child has stopped
    Stopped,

    /// Child is being restarted after a failure
    Restarting,

    /// Child has failed and is awaiting supervisor decision
    Failed,
}

impl ChildState {
    /// Returns `true` if the child is in a terminal state.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::supervisor::ChildState;
    ///
    /// assert!(ChildState::Stopped.is_terminal());
    /// assert!(ChildState::Failed.is_terminal());
    /// assert!(!ChildState::Running.is_terminal());
    /// ```
    pub fn is_terminal(&self) -> bool {
        matches!(self, ChildState::Stopped | ChildState::Failed)
    }

    /// Returns `true` if the child is in a transitional state.
    pub fn is_transitional(&self) -> bool {
        matches!(
            self,
            ChildState::Starting | ChildState::Stopping | ChildState::Restarting
        )
    }

    /// Returns `true` if the child is running.
    pub fn is_running(&self) -> bool {
        matches!(self, ChildState::Running)
    }
}

/// Health status of a supervised child.
///
/// Used by `Child::health_check()` to report operational status, enabling
/// proactive failure detection and recovery.
///
/// # Examples
///
/// ```rust
/// use airssys_rt::supervisor::ChildHealth;
///
/// let healthy = ChildHealth::Healthy;
/// assert!(healthy.is_healthy());
///
/// let degraded = ChildHealth::Degraded("High latency".into());
/// assert!(degraded.is_degraded());
/// assert_eq!(degraded.reason(), Some("High latency"));
///
/// let failed = ChildHealth::Failed("Connection lost".into());
/// assert!(failed.is_failed());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChildHealth {
    /// Child is operating normally
    Healthy,

    /// Child is operational but degraded.
    ///
    /// The child can still process work but is showing signs of issues
    /// (e.g., high latency, resource pressure, elevated error rates).
    Degraded(String),

    /// Child has failed and requires restart.
    Failed(String),
}

impl ChildHealth {
    /// Returns `true` if the child is healthy.
    pub fn is_healthy(&self) -> bool {
        matches!(self, ChildHealth::Healthy)
    }

    /// Returns `true` if the child is degraded.
    pub fn is_degraded(&self) -> bool {
        matches!(self, ChildHealth::Degraded(_))
    }

    /// Returns `true` if the child has failed.
    pub fn is_failed(&self) -> bool {
        matches!(self, ChildHealth::Failed(_))
    }

    /// Returns the reason string if degraded or failed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::supervisor::ChildHealth;
    ///
    /// let healthy = ChildHealth::Healthy;
    /// assert_eq!(healthy.reason(), None);
    ///
    /// let degraded = ChildHealth::Degraded("Queue backlog".into());
    /// assert_eq!(degraded.reason(), Some("Queue backlog"));
    /// ```
    pub fn reason(&self) -> Option<&str> {
        match self {
            ChildHealth::Healthy => None,
            ChildHealth::Degraded(reason) | ChildHealth::Failed(reason) => Some(reason),
        }
    }
}

/// Supervision decision made by a supervisor strategy.
///
/// Determines what action to take when a child fails or requires intervention.
///
/// # Examples
///
/// ```rust
/// use airssys_rt::supervisor::{SupervisionDecision, ChildId};
///
/// let child_id = ChildId::new();
/// let decision = SupervisionDecision::RestartChild(child_id);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SupervisionDecision {
    /// Restart only the specified child
    RestartChild(ChildId),

    /// Restart all children in the supervisor
    RestartAll(Vec<ChildId>),

    /// Restart a subset of children
    RestartSubset(Vec<ChildId>),

    /// Stop the specified child without restarting
    StopChild(ChildId),

    /// Stop all children without restarting
    StopAll,

    /// Escalate the error to the parent supervisor
    Escalate(String),
}

/// Handle for a supervised child.
///
/// Internal structure used by supervisors to track child state and metadata.
/// Not part of the public API (will be made `pub(crate)` in Phase 3).
#[derive(Debug)]
pub struct ChildHandle<C> {
    /// Child's unique identifier
    pub id: ChildId,

    /// Child's human-readable name
    pub name: String,

    /// The child instance
    pub child: C,

    /// Current state of the child
    pub state: ChildState,

    /// Number of times this child has been restarted
    pub restart_count: u32,

    /// Timestamp of the last restart (§3.2 - use chrono DateTime<Utc>)
    pub last_restart: Option<DateTime<Utc>>,

    /// Timestamp when child was started (§3.2 - use chrono DateTime<Utc>)
    pub start_time: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_child_id_uniqueness() {
        let id1 = ChildId::new();
        let id2 = ChildId::new();

        assert_ne!(id1, id2);
        assert_ne!(id1.as_uuid(), id2.as_uuid());
    }

    #[test]
    fn test_child_id_display() {
        let id = ChildId::new();
        let display = format!("{}", id);
        assert!(!display.is_empty());
    }

    #[test]
    fn test_child_id_from_uuid() {
        let uuid = Uuid::new_v4();
        let id = ChildId::from(uuid);
        assert_eq!(id.as_uuid(), &uuid);
    }

    #[test]
    fn test_restart_policy_permanent() {
        let policy = RestartPolicy::Permanent;
        assert!(policy.should_restart(true));  // Error exit
        assert!(policy.should_restart(false)); // Normal exit
    }

    #[test]
    fn test_restart_policy_transient() {
        let policy = RestartPolicy::Transient;
        assert!(policy.should_restart(true));   // Error exit
        assert!(!policy.should_restart(false)); // Normal exit
    }

    #[test]
    fn test_restart_policy_temporary() {
        let policy = RestartPolicy::Temporary;
        assert!(!policy.should_restart(true));  // Error exit
        assert!(!policy.should_restart(false)); // Normal exit
    }

    #[test]
    fn test_shutdown_policy_graceful() {
        let timeout = Duration::from_secs(5);
        let policy = ShutdownPolicy::Graceful(timeout);

        assert_eq!(policy.timeout(), Some(timeout));
        assert!(policy.is_graceful());
    }

    #[test]
    fn test_shutdown_policy_immediate() {
        let policy = ShutdownPolicy::Immediate;

        assert_eq!(policy.timeout(), Some(Duration::ZERO));
        assert!(!policy.is_graceful());
    }

    #[test]
    fn test_shutdown_policy_infinity() {
        let policy = ShutdownPolicy::Infinity;

        assert_eq!(policy.timeout(), None);
        assert!(policy.is_graceful());
    }

    #[test]
    fn test_child_state_terminal() {
        assert!(ChildState::Stopped.is_terminal());
        assert!(ChildState::Failed.is_terminal());
        assert!(!ChildState::Running.is_terminal());
        assert!(!ChildState::Starting.is_terminal());
    }

    #[test]
    fn test_child_state_transitional() {
        assert!(ChildState::Starting.is_transitional());
        assert!(ChildState::Stopping.is_transitional());
        assert!(ChildState::Restarting.is_transitional());
        assert!(!ChildState::Running.is_transitional());
        assert!(!ChildState::Stopped.is_transitional());
    }

    #[test]
    fn test_child_state_running() {
        assert!(ChildState::Running.is_running());
        assert!(!ChildState::Starting.is_running());
        assert!(!ChildState::Stopped.is_running());
    }

    #[test]
    fn test_child_health_healthy() {
        let health = ChildHealth::Healthy;
        assert!(health.is_healthy());
        assert!(!health.is_degraded());
        assert!(!health.is_failed());
        assert_eq!(health.reason(), None);
    }

    #[test]
    fn test_child_health_degraded() {
        let health = ChildHealth::Degraded("High latency".into());
        assert!(!health.is_healthy());
        assert!(health.is_degraded());
        assert!(!health.is_failed());
        assert_eq!(health.reason(), Some("High latency"));
    }

    #[test]
    fn test_child_health_failed() {
        let health = ChildHealth::Failed("Connection lost".into());
        assert!(!health.is_healthy());
        assert!(!health.is_degraded());
        assert!(health.is_failed());
        assert_eq!(health.reason(), Some("Connection lost"));
    }
}
