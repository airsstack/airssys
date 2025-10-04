//! Actor lifecycle management with state transitions.
//!
//! Provides actor state machine and lifecycle tracking for supervision.

// Layer 1: Standard library imports
// (none)

// Layer 2: Third-party crate imports
use chrono::{DateTime, Utc}; // §3.2 MANDATORY

// Layer 3: Internal module imports
// (none)

/// Actor state in the lifecycle state machine.
///
/// # State Transitions
///
/// ```text
/// Starting -> Running -> Stopping -> Stopped
///     |          |           |
///     v          v           v
///   Failed     Failed     Failed
/// ```
///
/// # Examples
///
/// ```rust
/// use airssys_rt::ActorState;
///
/// let state = ActorState::Starting;
/// assert_eq!(state, ActorState::Starting);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorState {
    /// Actor is initializing (pre_start in progress).
    Starting,

    /// Actor is running and processing messages.
    Running,

    /// Actor is shutting down (post_stop in progress).
    Stopping,

    /// Actor has stopped successfully.
    Stopped,

    /// Actor has failed and requires supervisor intervention.
    Failed,
}

impl Default for ActorState {
    fn default() -> Self {
        Self::Starting
    }
}

/// Actor lifecycle tracker with state management.
///
/// Tracks actor state transitions, restart count, and timing information
/// for supervision and monitoring.
///
/// # Examples
///
/// ```rust
/// use airssys_rt::{ActorLifecycle, ActorState};
///
/// let mut lifecycle = ActorLifecycle::new();
/// assert_eq!(lifecycle.state(), ActorState::Starting);
///
/// lifecycle.transition_to(ActorState::Running);
/// assert_eq!(lifecycle.state(), ActorState::Running);
/// assert_eq!(lifecycle.restart_count(), 0);
/// ```
#[derive(Debug, Clone)]
pub struct ActorLifecycle {
    state: ActorState,
    last_state_change: DateTime<Utc>,
    restart_count: u32,
}

impl ActorLifecycle {
    /// Create a new lifecycle tracker in Starting state.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::{ActorLifecycle, ActorState};
    ///
    /// let lifecycle = ActorLifecycle::new();
    /// assert_eq!(lifecycle.state(), ActorState::Starting);
    /// assert_eq!(lifecycle.restart_count(), 0);
    /// ```
    pub fn new() -> Self {
        Self {
            state: ActorState::Starting,
            last_state_change: Utc::now(), // §3.2
            restart_count: 0,
        }
    }

    /// Transition to a new state.
    ///
    /// Updates the state and records the transition timestamp.
    /// Increments restart_count when transitioning to Starting (for restarts).
    ///
    /// # Arguments
    ///
    /// * `new_state` - The target state
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::{ActorLifecycle, ActorState};
    ///
    /// let mut lifecycle = ActorLifecycle::new();
    ///
    /// lifecycle.transition_to(ActorState::Running);
    /// assert_eq!(lifecycle.state(), ActorState::Running);
    ///
    /// // Transition to Starting increments restart count
    /// lifecycle.transition_to(ActorState::Starting);
    /// assert_eq!(lifecycle.restart_count(), 1);
    /// ```
    pub fn transition_to(&mut self, new_state: ActorState) {
        self.state = new_state;
        self.last_state_change = Utc::now(); // §3.2

        // Increment restart count when transitioning to Starting (except initial)
        if new_state == ActorState::Starting && self.restart_count > 0 {
            self.restart_count += 1;
        } else if new_state == ActorState::Starting {
            self.restart_count = 1;
        }
    }

    /// Get the current actor state.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::{ActorLifecycle, ActorState};
    ///
    /// let lifecycle = ActorLifecycle::new();
    /// assert_eq!(lifecycle.state(), ActorState::Starting);
    /// ```
    pub fn state(&self) -> ActorState {
        self.state
    }

    /// Get the timestamp of the last state change.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::ActorLifecycle;
    ///
    /// let lifecycle = ActorLifecycle::new();
    /// let timestamp = lifecycle.last_state_change();
    /// assert!(timestamp <= chrono::Utc::now());
    /// ```
    pub fn last_state_change(&self) -> DateTime<Utc> {
        self.last_state_change
    }

    /// Get the number of times this actor has been restarted.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::{ActorLifecycle, ActorState};
    ///
    /// let mut lifecycle = ActorLifecycle::new();
    /// assert_eq!(lifecycle.restart_count(), 0);
    ///
    /// lifecycle.transition_to(ActorState::Running);
    /// lifecycle.transition_to(ActorState::Starting);
    /// assert_eq!(lifecycle.restart_count(), 1);
    /// ```
    pub fn restart_count(&self) -> u32 {
        self.restart_count
    }

    /// Check if the actor is in a terminal state (Stopped or Failed).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::{ActorLifecycle, ActorState};
    ///
    /// let mut lifecycle = ActorLifecycle::new();
    /// assert!(!lifecycle.is_terminal());
    ///
    /// lifecycle.transition_to(ActorState::Stopped);
    /// assert!(lifecycle.is_terminal());
    /// ```
    pub fn is_terminal(&self) -> bool {
        matches!(self.state, ActorState::Stopped | ActorState::Failed)
    }

    /// Check if the actor is currently running.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::{ActorLifecycle, ActorState};
    ///
    /// let mut lifecycle = ActorLifecycle::new();
    /// assert!(!lifecycle.is_running());
    ///
    /// lifecycle.transition_to(ActorState::Running);
    /// assert!(lifecycle.is_running());
    /// ```
    pub fn is_running(&self) -> bool {
        self.state == ActorState::Running
    }
}

impl Default for ActorLifecycle {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifecycle_new() {
        let lifecycle = ActorLifecycle::new();
        assert_eq!(lifecycle.state(), ActorState::Starting);
        assert_eq!(lifecycle.restart_count(), 0);
        assert!(!lifecycle.is_terminal());
        assert!(!lifecycle.is_running());
    }

    #[test]
    fn test_lifecycle_default() {
        let lifecycle = ActorLifecycle::default();
        assert_eq!(lifecycle.state(), ActorState::Starting);
    }

    #[test]
    fn test_state_transition() {
        let mut lifecycle = ActorLifecycle::new();

        lifecycle.transition_to(ActorState::Running);
        assert_eq!(lifecycle.state(), ActorState::Running);
        assert!(lifecycle.is_running());

        lifecycle.transition_to(ActorState::Stopping);
        assert_eq!(lifecycle.state(), ActorState::Stopping);
        assert!(!lifecycle.is_running());
    }

    #[test]
    fn test_restart_count_increment() {
        let mut lifecycle = ActorLifecycle::new();
        assert_eq!(lifecycle.restart_count(), 0);

        // First transition to Starting sets count to 1
        lifecycle.transition_to(ActorState::Running);
        lifecycle.transition_to(ActorState::Starting);
        assert_eq!(lifecycle.restart_count(), 1);

        // Subsequent transitions increment
        lifecycle.transition_to(ActorState::Running);
        lifecycle.transition_to(ActorState::Starting);
        assert_eq!(lifecycle.restart_count(), 2);
    }

    #[test]
    fn test_terminal_states() {
        let mut lifecycle = ActorLifecycle::new();
        assert!(!lifecycle.is_terminal());

        lifecycle.transition_to(ActorState::Running);
        assert!(!lifecycle.is_terminal());

        lifecycle.transition_to(ActorState::Stopped);
        assert!(lifecycle.is_terminal());

        lifecycle = ActorLifecycle::new();
        lifecycle.transition_to(ActorState::Failed);
        assert!(lifecycle.is_terminal());
    }

    #[test]
    fn test_is_running() {
        let mut lifecycle = ActorLifecycle::new();
        assert!(!lifecycle.is_running());

        lifecycle.transition_to(ActorState::Running);
        assert!(lifecycle.is_running());

        lifecycle.transition_to(ActorState::Stopping);
        assert!(!lifecycle.is_running());
    }

    #[test]
    fn test_last_state_change_updates() {
        let mut lifecycle = ActorLifecycle::new();
        let first_timestamp = lifecycle.last_state_change();

        std::thread::sleep(std::time::Duration::from_millis(10));
        lifecycle.transition_to(ActorState::Running);
        let second_timestamp = lifecycle.last_state_change();

        assert!(second_timestamp > first_timestamp);
    }

    #[test]
    fn test_state_equality() {
        assert_eq!(ActorState::Starting, ActorState::Starting);
        assert_ne!(ActorState::Running, ActorState::Stopped);
    }

    #[test]
    fn test_state_default() {
        assert_eq!(ActorState::default(), ActorState::Starting);
    }

    #[test]
    fn test_lifecycle_clone() {
        let lifecycle = ActorLifecycle::new();
        let cloned = lifecycle.clone();
        assert_eq!(lifecycle.state(), cloned.state());
        assert_eq!(lifecycle.restart_count(), cloned.restart_count());
    }
}
