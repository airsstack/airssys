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
use std::sync::Arc;
use std::time::Duration;

use super::correlation_trait::CorrelationTrackerTrait;

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

/// Object-safe timeout handling trait for dependency injection.
///
/// This trait provides an object-safe interface for timeout handling,
/// enabling storage as `Arc<dyn TimeoutHandlerObjectSafeTrait>`
/// in structs that require dependency injection.
///
/// # When to Use This Trait
///
/// - Use this trait when you need to store timeouts in a field
/// - Use this trait when you need to pass timeouts as `Arc<dyn Trait>`
/// - Use this trait for dependency injection patterns
///
/// # When to Use TimeoutHandlerTrait Instead
///
/// - Use `TimeoutHandlerTrait` when you need maximum performance
/// - Use `TimeoutHandlerTrait` when you don't need `dyn` (static dispatch)
/// - Use `TimeoutHandlerTrait` in generic functions with type parameters
///
/// # Design Notes
///
/// This trait is a wrapper around `TimeoutHandlerTrait` that uses
/// `Arc<dyn CorrelationTrackerTrait>` instead of generics, making
/// it object-safe at the cost of dynamic dispatch.
///
/// # Thread Safety
///
/// All trait methods must be thread-safe.
#[async_trait]
pub trait TimeoutHandlerObjectSafeTrait: Send + Sync {
    /// Register timeout for pending request.
    ///
    /// Spawns a background task that waits for timeout duration.
    /// If request is not resolved before timeout, sends a timeout error
    /// to the response channel.
    ///
    /// # Arguments
    ///
    /// * `correlation_id` - Correlation ID of request
    /// * `timeout` - Timeout duration
    /// * `tracker` - CorrelationTracker to remove request on timeout (as trait object)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use airssys_wasm::core::{correlation_trait::CorrelationTrackerTrait, timeout_trait::TimeoutHandlerObjectSafeTrait};
    /// use airssys_wasm::host_system::{correlation_impl::CorrelationTracker, timeout_impl::TimeoutHandler};
    /// use std::sync::Arc;
    ///
    /// let tracker = Arc::new(CorrelationTracker::new());
    /// let handler = Arc::new(TimeoutHandler::new());
    ///
    /// let correlation_id = CorrelationId::new();
    /// let timeout = Duration::from_secs(30);
    ///
    /// handler.register_timeout(correlation_id, timeout, tracker);
    /// ```
    fn register_timeout(
        &self,
        correlation_id: CorrelationId,
        timeout: Duration,
        tracker: Arc<dyn CorrelationTrackerTrait>,
    );

    /// Cancel timeout (called when response arrives before timeout).
    ///
    /// Aborts timeout task to prevent unnecessary timeout error.
    /// If timeout has already fired, this is a no-op.
    ///
    /// # Arguments
    ///
    /// * `correlation_id` - Correlation ID of request
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use airssys_wasm::core::timeout_trait::TimeoutHandlerObjectSafeTrait;
    /// use airssys_wasm::host_system::timeout_impl::TimeoutHandler;
    /// use std::sync::Arc;
    ///
    /// let handler = Arc::new(TimeoutHandler::new());
    ///
    /// handler.cancel_timeout(&correlation_id);
    /// ```
    fn cancel_timeout(&self, correlation_id: &CorrelationId);

    /// Get number of active timeouts (for monitoring).
    ///
    /// Returns the current count of active timeout tasks.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use airssys_wasm::core::timeout_trait::TimeoutHandlerObjectSafeTrait;
    /// use airssys_wasm::host_system::timeout_impl::TimeoutHandler;
    /// use std::sync::Arc;
    ///
    /// let handler = Arc::new(TimeoutHandler::new());
    ///
    /// let count = handler.active_count();
    /// println!("Active timeouts: {}", count);
    /// ```
    fn active_count(&self) -> usize;
}
