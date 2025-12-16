//! Event callbacks for ComponentActor monitoring and observability.
//!
//! This module provides the `EventCallback` trait which enables external systems
//! to monitor component lifecycle events for observability, metrics collection,
//! and alerting purposes.
//!
//! # Design Principles
//!
//! - **Non-blocking**: Callbacks fire synchronously but should be fast (<10μs)
//! - **Immutable**: Callbacks use `&self` (no component state mutation)
//! - **Optional**: Callback registration is optional via `Option<Arc<dyn EventCallback>>`
//! - **Fire-and-forget**: No return values, errors logged internally
//!
//! # Performance
//!
//! - Callback dispatch: <10μs per event
//! - No-op callbacks: ~1-2μs (trait object indirection only)
//!
//! # Examples
//!
//! ```rust,ignore
//! use airssys_wasm::actor::lifecycle::{EventCallback, HealthStatus};
//! use airssys_wasm::core::{ComponentId, WasmError};
//! use std::time::Duration;
//!
//! struct MetricsCollector {
//!     message_count: AtomicU64,
//! }
//!
//! impl EventCallback for MetricsCollector {
//!     fn on_message_received(&self, component_id: ComponentId) {
//!         self.message_count.fetch_add(1, Ordering::Relaxed);
//!     }
//!     
//!     fn on_message_processed(&self, component_id: ComponentId, latency: Duration) {
//!         println!("Message processed in {:?}", latency);
//!     }
//! }
//! ```

// Layer 1: Standard library imports
use std::time::Duration;

// Layer 2: Third-party crate imports
// (none)

// Layer 3: Internal module imports
use crate::actor::lifecycle::RestartReason;
use crate::actor::HealthStatus;
use crate::core::{ComponentId, WasmError};

/// Event callback for monitoring component lifecycle.
///
/// EventCallback provides a trait for external monitoring systems to observe
/// component events without modifying component behavior. All methods have
/// default no-op implementations.
///
/// # Design Notes
///
/// - **Immutable &self**: Callbacks don't modify component state
/// - **`Arc<dyn EventCallback>`**: Optional registration, Arc for sharing
/// - **Non-blocking**: No async, fires immediately (keep callbacks fast)
/// - **Error context**: Full error and reason passed to callbacks
/// - **Latency tracking**: Measure message processing time
///
/// # Examples
///
/// ```rust,ignore
/// use airssys_wasm::actor::lifecycle::EventCallback;
/// use std::sync::atomic::{AtomicU64, Ordering};
///
/// struct SimpleLogger {
///     message_count: AtomicU64,
/// }
///
/// impl EventCallback for SimpleLogger {
///     fn on_message_received(&self, component_id: ComponentId) {
///         let count = self.message_count.fetch_add(1, Ordering::Relaxed);
///         println!("Component {} received message #{}", component_id.as_str(), count);
///     }
/// }
/// ```
pub trait EventCallback: Send + Sync {
    /// Called when component receives a message.
    ///
    /// Fired immediately when ComponentMessage arrives, before hook processing
    /// or WASM invocation. Use for request counting, rate limit checks.
    ///
    /// # Arguments
    ///
    /// * `component_id` - ID of component receiving message
    ///
    /// # Performance
    ///
    /// Should complete in <5μs. Heavy processing should be deferred.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// fn on_message_received(&self, component_id: ComponentId) {
    ///     self.metrics.increment_counter("messages_received");
    /// }
    /// ```
    fn on_message_received(&self, _component_id: ComponentId) {}
    
    /// Called when component finishes processing message (with latency).
    ///
    /// Fired after WASM invocation completes successfully. Includes latency
    /// measurement for performance tracking.
    ///
    /// # Arguments
    ///
    /// * `component_id` - ID of component that processed message
    /// * `latency` - Time elapsed from message received to processed
    ///
    /// # Performance
    ///
    /// Should complete in <5μs. Heavy processing should be deferred.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// fn on_message_processed(&self, component_id: ComponentId, latency: Duration) {
    ///     self.metrics.record_histogram("message_latency_ms", latency.as_millis() as f64);
    /// }
    /// ```
    fn on_message_processed(&self, _component_id: ComponentId, _latency: Duration) {}
    
    /// Called when error occurs in component.
    ///
    /// Fired whenever a WasmError occurs (execution error, timeout, etc).
    /// Use for error logging, alerting, error rate tracking.
    ///
    /// # Arguments
    ///
    /// * `component_id` - ID of component where error occurred
    /// * `error` - WasmError that occurred
    ///
    /// # Performance
    ///
    /// Should complete in <10μs. Alert fanout should be async.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// fn on_error_occurred(&self, component_id: ComponentId, error: &WasmError) {
    ///     eprintln!("Component {} error: {:?}", component_id.as_str(), error);
    ///     self.metrics.increment_counter("component_errors");
    /// }
    /// ```
    fn on_error_occurred(&self, _component_id: ComponentId, _error: &WasmError) {}
    
    /// Called when supervisor restarts component.
    ///
    /// Fired when supervisor triggers component restart (crash, health check,
    /// manual). Use for restart tracking, alerting on frequent restarts.
    ///
    /// # Arguments
    ///
    /// * `component_id` - ID of component being restarted
    /// * `reason` - Why the restart was triggered
    ///
    /// # Performance
    ///
    /// Should complete in <10μs. Alert fanout should be async.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// fn on_restart_triggered(&self, component_id: ComponentId, reason: RestartReason) {
    ///     println!("Component {} restarting: {:?}", component_id.as_str(), reason);
    ///     self.restart_tracker.record_restart(component_id, reason);
    /// }
    /// ```
    fn on_restart_triggered(&self, _component_id: ComponentId, _reason: RestartReason) {}
    
    /// Called when component health status changes.
    ///
    /// Fired when health check reports Healthy → Degraded, Degraded → Unhealthy,
    /// or any other status transition. Use for health monitoring dashboards.
    ///
    /// # Arguments
    ///
    /// * `component_id` - ID of component with health change
    /// * `new_health` - New health status
    ///
    /// # Performance
    ///
    /// Should complete in <10μs. Dashboard updates should be async.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// fn on_health_changed(&self, component_id: ComponentId, new_health: HealthStatus) {
    ///     match new_health {
    ///         HealthStatus::Unhealthy { reason } => {
    ///             self.alerting.send_alert(component_id, reason);
    ///         }
    ///         _ => {}
    ///     }
    /// }
    /// ```
    fn on_health_changed(&self, _component_id: ComponentId, _new_health: HealthStatus) {}
}

/// Default no-op implementation.
///
/// NoOpEventCallback provides a zero-overhead default implementation where
/// all callbacks are no-ops. Components that don't need event callbacks can
/// use this implementation (or leave event_callback as None).
///
/// # Examples
///
/// ```rust
/// use airssys_wasm::actor::lifecycle::{EventCallback, NoOpEventCallback};
/// use std::sync::Arc;
///
/// let callback: Arc<dyn EventCallback> = Arc::new(NoOpEventCallback);
/// // All events are no-ops (minimal overhead)
/// ```
#[derive(Debug)]
pub struct NoOpEventCallback;

impl EventCallback for NoOpEventCallback {}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;

    struct TestCallback {
        message_received_count: Arc<AtomicU64>,
        message_processed_count: Arc<AtomicU64>,
        error_count: Arc<AtomicU64>,
        restart_count: Arc<AtomicU64>,
        health_change_count: Arc<AtomicU64>,
    }

    impl TestCallback {
        fn new() -> Self {
            Self {
                message_received_count: Arc::new(AtomicU64::new(0)),
                message_processed_count: Arc::new(AtomicU64::new(0)),
                error_count: Arc::new(AtomicU64::new(0)),
                restart_count: Arc::new(AtomicU64::new(0)),
                health_change_count: Arc::new(AtomicU64::new(0)),
            }
        }
    }

    impl EventCallback for TestCallback {
        fn on_message_received(&self, _component_id: ComponentId) {
            self.message_received_count.fetch_add(1, Ordering::Relaxed);
        }
        
        fn on_message_processed(&self, _component_id: ComponentId, _latency: Duration) {
            self.message_processed_count.fetch_add(1, Ordering::Relaxed);
        }
        
        fn on_error_occurred(&self, _component_id: ComponentId, _error: &WasmError) {
            self.error_count.fetch_add(1, Ordering::Relaxed);
        }
        
        fn on_restart_triggered(&self, _component_id: ComponentId, _reason: RestartReason) {
            self.restart_count.fetch_add(1, Ordering::Relaxed);
        }
        
        fn on_health_changed(&self, _component_id: ComponentId, _new_health: HealthStatus) {
            self.health_change_count.fetch_add(1, Ordering::Relaxed);
        }
    }

    #[test]
    fn test_noop_callback() {
        let callback = NoOpEventCallback;
        let component_id = ComponentId::new("test");
        
        // All callbacks should be no-ops
        callback.on_message_received(component_id.clone());
        callback.on_message_processed(component_id.clone(), Duration::from_millis(10));
        callback.on_error_occurred(component_id.clone(), &WasmError::internal("test"));
        callback.on_restart_triggered(component_id.clone(), RestartReason::Manual);
        callback.on_health_changed(component_id, HealthStatus::Healthy);
    }

    #[test]
    fn test_noop_callback_as_trait_object() {
        let callback: Arc<dyn EventCallback> = Arc::new(NoOpEventCallback);
        let component_id = ComponentId::new("test");
        
        callback.on_message_received(component_id);
    }

    #[test]
    fn test_custom_callback() {
        let callback = TestCallback::new();
        let component_id = ComponentId::new("test");
        
        // Test on_message_received
        callback.on_message_received(component_id.clone());
        callback.on_message_received(component_id.clone());
        assert_eq!(callback.message_received_count.load(Ordering::Relaxed), 2);
        
        // Test on_message_processed
        callback.on_message_processed(component_id.clone(), Duration::from_millis(10));
        assert_eq!(callback.message_processed_count.load(Ordering::Relaxed), 1);
        
        // Test on_error_occurred
        callback.on_error_occurred(component_id.clone(), &WasmError::internal("test"));
        assert_eq!(callback.error_count.load(Ordering::Relaxed), 1);
        
        // Test on_restart_triggered
        callback.on_restart_triggered(component_id.clone(), RestartReason::Crashed("panic".to_string()));
        assert_eq!(callback.restart_count.load(Ordering::Relaxed), 1);
        
        // Test on_health_changed
        callback.on_health_changed(component_id, HealthStatus::Healthy);
        assert_eq!(callback.health_change_count.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_custom_callback_as_trait_object() {
        let callback: Arc<dyn EventCallback> = Arc::new(TestCallback::new());
        let component_id = ComponentId::new("test");
        
        callback.on_message_received(component_id);
    }

    #[test]
    fn test_callback_thread_safety() {
        let callback = Arc::new(TestCallback::new());
        let component_id = ComponentId::new("test");
        
        // Simulate concurrent callback invocations
        let handles: Vec<_> = (0..10).map(|_| {
            let callback = Arc::clone(&callback);
            let component_id = component_id.clone();
            std::thread::spawn(move || {
                callback.on_message_received(component_id);
            })
        }).collect();
        
        
        #[allow(clippy::unwrap_used, reason = "test thread should not panic")]
        for handle in handles {
            handle.join().unwrap();
        }
        
        assert_eq!(callback.message_received_count.load(Ordering::Relaxed), 10);
    }
}
