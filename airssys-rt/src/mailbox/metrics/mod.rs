//! Mailbox metrics recording and tracking subsystem.
//!
//! Provides trait-based metrics recording for mailbox operations with
//! pluggable implementations.
//!
//! # Design
//!
//! - `MetricsRecorder` trait: Interface for recording metrics
//! - `AtomicMetrics`: Default lock-free implementation using atomics
//! - Future: `AsyncMetrics`, `NoOpMetrics`, `PrometheusMetrics`
//!
//! # Architecture
//!
//! The metrics system uses generic constraints (`R: MetricsRecorder`) instead of
//! trait objects (`dyn MetricsRecorder`) for zero-cost abstractions (§6.2).
//!
//! Mailbox implementations are generic over the metrics recorder:
//! - `BoundedMailbox<M: Message, R: MetricsRecorder>`
//! - `UnboundedMailbox<M: Message, R: MetricsRecorder>`
//!
//! This allows compile-time selection of metrics implementation without
//! runtime dispatch overhead.
//!
//! # Example
//!
//! ```rust
//! use airssys_rt::mailbox::{BoundedMailbox, AtomicMetrics};
//! use std::sync::Arc;
//!
//! # #[derive(Debug, Clone)]
//! # struct MyMessage;
//! # impl airssys_rt::message::Message for MyMessage {
//! #     const MESSAGE_TYPE: &'static str = "my_message";
//! # }
//! # async fn example() {
//! // Uses AtomicMetrics by default
//! let (mailbox, sender) = BoundedMailbox::<MyMessage>::new(100);
//!
//! // Or inject custom metrics
//! let metrics = Arc::new(AtomicMetrics::default());
//! let (mailbox, sender) = BoundedMailbox::with_metrics(100, metrics);
//! # }
//! ```
//!
//! # Future Implementations
//!
//! ## AsyncMetrics (Fire-and-Forget)
//!
//! ```ignore
//! pub struct AsyncMetrics {
//!     tx: mpsc::UnboundedSender<MetricEvent>,
//! }
//!
//! impl MetricsRecorder for AsyncMetrics {
//!     fn record_sent(&self) {
//!         let _ = self.tx.send(MetricEvent::Sent);
//!     }
//!     // Background task aggregates events
//! }
//! ```
//!
//! ## NoOpMetrics (Zero Overhead)
//!
//! ```ignore
//! pub struct NoOpMetrics;
//!
//! impl MetricsRecorder for NoOpMetrics {
//!     fn record_sent(&self) {}  // No-op
//!     fn sent_count(&self) -> u64 { 0 }
//!     // All operations are no-ops
//! }
//! ```
//!
//! ## PrometheusMetrics (Remote Export)
//!
//! ```ignore
//! pub struct PrometheusMetrics {
//!     counter_sent: prometheus::Counter,
//!     counter_received: prometheus::Counter,
//!     gauge_in_flight: prometheus::Gauge,
//! }
//!
//! impl MetricsRecorder for PrometheusMetrics {
//!     fn record_sent(&self) {
//!         self.counter_sent.inc();
//!         self.gauge_in_flight.inc();
//!     }
//!     // Export to Prometheus registry
//! }
//! ```

mod atomic;
mod recorder;

pub use atomic::AtomicMetrics;
pub use recorder::MetricsRecorder;
