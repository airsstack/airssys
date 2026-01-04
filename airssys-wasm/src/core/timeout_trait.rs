//! Timeout handling trait for pending requests.
//!
//! This trait defines the abstraction for timeout handling, enabling
//! automatic timeout enforcement for request-response patterns. Implementations
//! can use different concurrency primitives (tokio, async-std, etc.).
//!
//! # Architecture
//!
//! ```text
//! TimeoutHandlerTrait (abstraction in core/)
//!     ↓
//!     implements
//!     ↓
//! TimeoutHandler (implementation in host_system/)
//!     ├── DashMap<CorrelationId, JoinHandle>
//!     └── Tokio spawn tasks (one per timeout)
//! ```
//!
//! # Dependency Management
//!
//! This trait is dependency-free (no external imports), allowing any module
//! to depend on the abstraction without transitive dependencies.

use crate::core::messaging::CorrelationId;
use async_trait::async_trait;
use std::time::Duration;

/// Timeout handling for pending requests.
///
/// Trait defining the contract for timeout handling, enabling
/// automatic timeout enforcement for request-response patterns.
///
/// # Thread Safety
///
/// All trait methods must be thread-safe. Implementations typically use
/// concurrent data structures like DashMap or RwLock.
#[async_trait]
pub trait TimeoutHandlerTrait: Send + Sync {
    /// Create new timeout handler instance.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let handler = Arc::new(MyTimeoutHandler::new());
    /// ```
    fn new() -> Self
    where
        Self: Sized;

    /// Register timeout for pending request.
    ///
    /// Spawns a background task that waits for timeout duration.
    /// If request is not resolved before timeout, sends a timeout error
    /// to the response channel.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Type implementing CorrelationTrackerTrait (enables dependency injection)
    ///
    /// # Arguments
    ///
    /// * `correlation_id` - Correlation ID of request
    /// * `timeout` - Timeout duration
    /// * `tracker` - CorrelationTracker to remove request on timeout
    ///
    /// # Design Notes
    ///
    /// Uses generic parameter `T: CorrelationTrackerTrait + 'static` instead of `dyn` to:
    /// - Enable compile-time type checking (static dispatch)
    /// - Zero vtable overhead (monomorphization)
    /// - Maintain DIP architecture (dependency injection)
    /// - Follow PROJECTS_STANDARD.md §6.2 (avoid dyn patterns)
    /// - Support `tokio::spawn` (`'static` lifetime bound required)
    fn register_timeout<T: super::correlation_trait::CorrelationTrackerTrait + 'static>(
        &self,
        correlation_id: CorrelationId,
        timeout: Duration,
        tracker: std::sync::Arc<T>,
    );

    /// Cancel timeout (called when response arrives before timeout).
    ///
    /// Aborts the timeout task to prevent unnecessary timeout error.
    /// If the timeout has already fired, this is a no-op.
    ///
    /// # Arguments
    ///
    /// * `correlation_id` - Correlation ID of the request
    fn cancel_timeout(&self, correlation_id: &CorrelationId);

    /// Get number of active timeouts (for monitoring).
    ///
    /// Returns the current count of active timeout tasks.
    fn active_count(&self) -> usize;
}
