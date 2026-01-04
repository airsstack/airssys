//! Correlation tracking trait for request-response patterns.
//!
//! This trait defines the abstraction for correlation tracking, enabling
//! request-response patterns with automatic timeout handling. Implementations
//! can use different concurrency primitives (DashMap, RwLock, etc.).
//!
//! # Architecture
//!
//! ```text
//! CorrelationTrackerTrait (abstraction in core/)
//!     ↓
//!     implements
//!     ↓
//! CorrelationTracker (implementation in host_system/)
//!     ├── DashMap<CorrelationId, PendingRequest>
//!     ├── TimeoutHandler
//!     ├── completed_count: AtomicU64
//!     └── timeout_count: AtomicU64
//! ```
//!
//! # Dependency Management
//!
//! This trait is dependency-free (no external imports), allowing any module
//! to depend on the abstraction without transitive dependencies.

use async_trait::async_trait;
use crate::core::messaging::{CorrelationId, PendingRequest, ResponseMessage};
use crate::core::WasmError;

/// Correlation tracking for request-response patterns.
///
/// Trait defining the contract for correlation tracking, enabling
/// request-response patterns with automatic timeout handling.
///
/// # Thread Safety
///
/// All trait methods must be thread-safe. Implementations typically use
/// concurrent data structures like DashMap or RwLock.
///
/// # Performance
///
/// Implementations should target:
/// - Lookup: <50ns
/// - Insert: ~100ns
/// - Remove: ~100ns
#[async_trait]
pub trait CorrelationTrackerTrait: Send + Sync {
    /// Create new correlation tracker instance.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let tracker = Arc::new(MyCorrelationTracker::new());
    /// ```
    fn new() -> Self
    where
        Self: Sized;

    /// Register pending request with timeout.
    ///
    /// Stores request in the pending map and schedules a timeout task.
    /// If the request is not resolved before timeout, a timeout error will
    /// be sent to the response channel.
    ///
    /// # Arguments
    ///
    /// * `request` - Pending request with correlation ID and response channel
    ///
    /// # Returns
    ///
    /// Ok(()) if registered successfully
    ///
    /// # Errors
    ///
    /// Returns WasmError if correlation ID already exists
    async fn register_pending(&self, request: PendingRequest) -> Result<(), WasmError>;

    /// Resolve pending request with response.
    ///
    /// Removes the request from the pending map and delivers the response
    /// via the oneshot channel. Cancels the timeout task if response arrives
    /// before timeout.
    ///
    /// # Arguments
    ///
    /// * `correlation_id` - Correlation ID of the request
    /// * `response` - Response message to deliver
    ///
    /// # Returns
    ///
    /// Ok(()) if resolved successfully
    ///
    /// # Errors
    ///
    /// Returns WasmError if correlation ID not found
    async fn resolve(
        &self,
        correlation_id: CorrelationId,
        mut response: ResponseMessage,
    ) -> Result<(), WasmError>;

    /// Remove pending request (internal use).
    ///
    /// This method is called by the timeout handler when a request times out.
    /// It removes the request from the pending map so the timeout error
    /// can be sent.
    ///
    /// # Arguments
    ///
    /// * `correlation_id` - Correlation ID to remove
    ///
    /// # Returns
    ///
    /// Some(PendingRequest) if found and removed
    /// None if already resolved
    fn remove_pending(&self, correlation_id: &CorrelationId) -> Option<PendingRequest>;

    /// Cleanup expired requests (background maintenance).
    ///
    /// Removes requests that have exceeded their timeout duration but whose
    /// timeout handlers haven't fired yet.
    ///
    /// # Returns
    ///
    /// Number of expired requests cleaned up
    async fn cleanup_expired(&self) -> usize;

    /// Get number of pending requests (for monitoring).
    ///
    /// Returns the current count of pending requests waiting for responses.
    fn pending_count(&self) -> usize;

    /// Check if correlation ID exists (for testing).
    ///
    /// Returns true if the correlation ID is currently in the pending map.
    fn contains(&self, correlation_id: &CorrelationId) -> bool;

    /// Get number of completed (resolved) requests.
    ///
    /// Returns the total count of requests that were successfully resolved.
    fn completed_count(&self) -> u64;

    /// Get number of timed out requests.
    ///
    /// Returns the total count of requests that expired before receiving
    /// a response.
    fn timeout_count(&self) -> u64;

    /// Remove all pending requests for a specific component.
    ///
    /// When a component is stopped, all its pending requests must be
    /// cleaned up to prevent memory leaks and timeout errors.
    ///
    /// # Arguments
    ///
    /// * `component_id` - Component ID to clean up requests for
    async fn cleanup_pending_for_component(&self, component_id: &crate::core::component::ComponentId);
}
