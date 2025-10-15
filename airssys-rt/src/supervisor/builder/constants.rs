//! Default configuration constants for builder patterns.
//!
//! This module defines sensible defaults for child supervision configuration,
//! balancing production safety, fault tolerance, and system responsiveness.

use std::time::Duration;

use crate::supervisor::{RestartPolicy, ShutdownPolicy};

/// Default restart policy: Permanent (always restart on failure).
///
/// # Rationale
///
/// Supervisors exist primarily for fault tolerance. The `Permanent` restart
/// policy ensures children are always restarted after failure, which is the
/// most common and safest default for production systems.
///
/// Users can override this with [`SingleChildBuilder::restart_transient`] or
/// [`SingleChildBuilder::restart_temporary`] for different failure semantics.
///
/// # Examples
///
/// ```rust
/// use airssys_rt::supervisor::builder::DEFAULT_RESTART_POLICY;
/// use airssys_rt::supervisor::RestartPolicy;
///
/// assert_eq!(DEFAULT_RESTART_POLICY, RestartPolicy::Permanent);
/// ```
pub const DEFAULT_RESTART_POLICY: RestartPolicy = RestartPolicy::Permanent;

/// Default shutdown policy: Graceful with 5 second timeout.
///
/// # Rationale
///
/// A 5-second graceful shutdown provides a reasonable balance:
/// - Long enough for most cleanup operations (flush buffers, close connections)
/// - Short enough to maintain system responsiveness during restarts
/// - Typical production default in similar systems (Kubernetes, systemd)
///
/// Children that need more time can override via
/// [`SingleChildBuilder::shutdown_graceful`] or
/// [`SingleChildBuilder::shutdown_policy`].
///
/// # Examples
///
/// ```rust
/// use airssys_rt::supervisor::builder::DEFAULT_SHUTDOWN_POLICY;
/// use airssys_rt::supervisor::ShutdownPolicy;
/// use std::time::Duration;
///
/// assert_eq!(
///     DEFAULT_SHUTDOWN_POLICY,
///     ShutdownPolicy::Graceful(Duration::from_secs(5))
/// );
/// ```
pub const DEFAULT_SHUTDOWN_POLICY: ShutdownPolicy =
    ShutdownPolicy::Graceful(Duration::from_secs(5));

/// Default start timeout: 30 seconds.
///
/// # Rationale
///
/// A 30-second start timeout accommodates slow initialization scenarios:
/// - Database connection establishment
/// - Configuration loading from external services
/// - Warm-up periods for caches
/// - TLS handshakes and authentication
///
/// This is long enough for most real-world startup sequences while preventing
/// indefinite hangs. Children with faster startup can complete earlier; only
/// truly slow operations need the full timeout.
///
/// Override via [`SingleChildBuilder::start_timeout`] if needed.
///
/// # Examples
///
/// ```rust
/// use airssys_rt::supervisor::builder::DEFAULT_START_TIMEOUT;
/// use std::time::Duration;
///
/// assert_eq!(DEFAULT_START_TIMEOUT, Duration::from_secs(30));
/// ```
pub const DEFAULT_START_TIMEOUT: Duration = Duration::from_secs(30);

/// Default shutdown timeout: 10 seconds.
///
/// # Rationale
///
/// A 10-second shutdown timeout provides reasonable cleanup time without
/// blocking system shutdown for too long. This is separate from the graceful
/// shutdown policy timeout (5s) and represents the maximum time the supervisor
/// will wait for the shutdown process to complete.
///
/// Override via [`SingleChildBuilder::shutdown_timeout`] if needed.
///
/// # Examples
///
/// ```rust
/// use airssys_rt::supervisor::builder::DEFAULT_SHUTDOWN_TIMEOUT;
/// use std::time::Duration;
///
/// assert_eq!(DEFAULT_SHUTDOWN_TIMEOUT, Duration::from_secs(10));
/// ```
pub const DEFAULT_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(10);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_restart_policy_is_permanent() {
        assert_eq!(DEFAULT_RESTART_POLICY, RestartPolicy::Permanent);
    }

    #[test]
    fn test_default_shutdown_policy_is_graceful_5s() {
        assert_eq!(
            DEFAULT_SHUTDOWN_POLICY,
            ShutdownPolicy::Graceful(Duration::from_secs(5))
        );
    }

    #[test]
    fn test_default_start_timeout_is_30s() {
        assert_eq!(DEFAULT_START_TIMEOUT.as_secs(), 30);
    }

    #[test]
    fn test_default_shutdown_timeout_is_10s() {
        assert_eq!(DEFAULT_SHUTDOWN_TIMEOUT.as_secs(), 10);
    }

    #[test]
    fn test_shutdown_timeout_exceeds_graceful_timeout() {
        // Shutdown timeout should be greater than graceful shutdown policy timeout
        // to allow for the graceful shutdown to complete
        if let ShutdownPolicy::Graceful(graceful_timeout) = DEFAULT_SHUTDOWN_POLICY {
            assert!(
                DEFAULT_SHUTDOWN_TIMEOUT > graceful_timeout,
                "Shutdown timeout should exceed graceful policy timeout"
            );
        }
    }

    #[test]
    fn test_all_defaults_are_production_ready() {
        // Validate that all defaults represent production-safe values
        assert!(
            DEFAULT_START_TIMEOUT.as_secs() >= 10,
            "Start timeout should be at least 10 seconds for production"
        );
        assert!(
            DEFAULT_SHUTDOWN_TIMEOUT.as_secs() >= 5,
            "Shutdown timeout should be at least 5 seconds for cleanup"
        );
    }
}
