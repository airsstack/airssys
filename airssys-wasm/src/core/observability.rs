//! Observability abstractions for monitoring and metrics.
//!
//! This module provides traits and types for collecting metrics, tracking
//! observability events, and monitoring component health. The abstractions
//! enable pluggable observability backends while maintaining a consistent
//! interface across the framework.
//!
//! # Architecture
//!
//! The observability architecture includes:
//!
//! - **Metrics Collection**: Counter, gauge, histogram, and timing metrics
//! - **Event Tracking**: Observable events with severity levels
//! - **Health Monitoring**: Component health status and failure tracking
//! - **Snapshot Support**: Point-in-time metrics snapshots
//!
//! # Examples
//!
//! ```rust
//! use std::collections::HashMap;
//! use chrono::Utc;
//! use airssys_wasm::core::{
//!     ComponentId,
//!     observability::{Metric, MetricType, EventSeverity, ObservabilityEvent, HealthStatus}
//! };
//!
//! let metric = Metric {
//!     component_id: ComponentId::new("my-component"),
//!     metric_type: MetricType::Counter {
//!         name: "requests".to_string(),
//!         value: 42,
//!     },
//!     labels: HashMap::new(),
//!     timestamp: Utc::now(),
//! };
//!
//! let event = ObservabilityEvent {
//!     component_id: ComponentId::new("my-component"),
//!     event_type: "startup".to_string(),
//!     severity: EventSeverity::Info,
//!     message: "Component initialized".to_string(),
//!     metadata: HashMap::new(),
//!     timestamp: Utc::now(),
//! };
//! ```
//!
//! # References
//!
//! - **ADR-WASM-012**: Comprehensive Core Abstractions Strategy
//! - **WASM-TASK-000 Phase 10**: Observability abstractions design
//! - **Workspace Standards**: §3.2 (chrono DateTime<Utc>), §6.1 (YAGNI)

use std::collections::HashMap;

use chrono::{DateTime, Utc};

use super::{ComponentId, WasmResult};

/// Metrics collector trait for observability.
///
/// Implementors of this trait provide metric collection and snapshot capabilities.
/// The trait is designed to be simple and composable, allowing different backends
/// (in-memory, time-series databases, monitoring services) to plug in.
///
/// # Examples
///
/// ```rust,ignore
/// use airssys_wasm::core::WasmResult;
/// use airssys_wasm::core::observability::{MetricsCollector, Metric, MetricsSnapshot};
///
/// struct InMemoryCollector {
///     metrics: Vec<Metric>,
/// }
///
/// impl MetricsCollector for InMemoryCollector {
///     fn record_metric(&self, metric: Metric) -> WasmResult<()> {
///         Ok(())
///     }
///     
///     fn snapshot(&self) -> MetricsSnapshot {
///         MetricsSnapshot::new()
///     }
/// }
/// ```
pub trait MetricsCollector: Send + Sync {
    /// Records a metric value.
    ///
    /// # Errors
    ///
    /// Returns `WasmError` if metric recording fails (e.g., backend unavailable,
    /// invalid metric format).
    fn record_metric(&self, metric: Metric) -> WasmResult<()>;

    /// Returns a snapshot of current metrics.
    ///
    /// The snapshot captures the state of all metrics at the time of the call.
    /// Implementations may aggregate, filter, or transform metrics as appropriate.
    fn snapshot(&self) -> MetricsSnapshot;
}

/// Metric type classification.
///
/// Categorizes metrics by their type and value semantics. Each variant carries
/// its specific data and follows standard observability metric conventions.
///
/// # Examples
///
/// ```rust
/// use airssys_wasm::core::observability::MetricType;
///
/// let counter = MetricType::Counter {
///     name: "http_requests_total".to_string(),
///     value: 1542,
/// };
///
/// let gauge = MetricType::Gauge {
///     name: "memory_usage_bytes".to_string(),
///     value: 1024.5,
/// };
///
/// let histogram = MetricType::Histogram {
///     name: "request_duration_seconds".to_string(),
///     value: 0.234,
/// };
///
/// let timing = MetricType::Timing {
///     name: "operation_duration".to_string(),
///     duration_ms: 150,
/// };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum MetricType {
    /// Counter metric - monotonically increasing value.
    ///
    /// Counters represent cumulative values that only increase (e.g., total requests,
    /// total errors). Use counters for values that accumulate over time.
    Counter {
        /// Metric name identifier.
        name: String,
        /// Current counter value.
        value: u64,
    },

    /// Gauge metric - value that can increase or decrease.
    ///
    /// Gauges represent point-in-time measurements that can go up or down
    /// (e.g., memory usage, active connections, queue depth).
    Gauge {
        /// Metric name identifier.
        name: String,
        /// Current gauge value.
        value: f64,
    },

    /// Histogram metric - distribution of values.
    ///
    /// Histograms track the distribution of measurements (e.g., request latencies,
    /// response sizes). The backend typically calculates percentiles and buckets.
    Histogram {
        /// Metric name identifier.
        name: String,
        /// Sample value.
        value: f64,
    },

    /// Timing metric - duration measurement in milliseconds.
    ///
    /// Timing metrics specifically measure durations. While similar to histograms,
    /// they explicitly represent time and use millisecond precision.
    Timing {
        /// Metric name identifier.
        name: String,
        /// Duration in milliseconds.
        duration_ms: u64,
    },
}

impl MetricType {
    /// Returns the metric name.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::observability::MetricType;
    ///
    /// let metric = MetricType::Counter {
    ///     name: "requests".to_string(),
    ///     value: 42,
    /// };
    ///
    /// assert_eq!(metric.name(), "requests");
    /// ```
    pub fn name(&self) -> &str {
        match self {
            Self::Counter { name, .. } => name,
            Self::Gauge { name, .. } => name,
            Self::Histogram { name, .. } => name,
            Self::Timing { name, .. } => name,
        }
    }
}

/// Metric value with metadata.
///
/// Represents a single metric observation with its associated component,
/// labels, and timestamp. Labels enable dimensional metrics and filtering.
///
/// # Examples
///
/// ```rust
/// use std::collections::HashMap;
/// use chrono::Utc;
/// use airssys_wasm::core::{ComponentId, observability::{Metric, MetricType}};
///
/// let mut labels = HashMap::new();
/// labels.insert("method".to_string(), "GET".to_string());
/// labels.insert("status".to_string(), "200".to_string());
///
/// let metric = Metric {
///     component_id: ComponentId::new("my-component"),
///     metric_type: MetricType::Counter {
///         name: "http_requests".to_string(),
///         value: 1,
///     },
///     labels,
///     timestamp: Utc::now(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Metric {
    /// Component that generated this metric.
    pub component_id: ComponentId,

    /// Metric type and value.
    pub metric_type: MetricType,

    /// Key-value labels for dimensional metrics.
    pub labels: HashMap<String, String>,

    /// Timestamp when metric was recorded.
    pub timestamp: DateTime<Utc>,
}

impl Metric {
    /// Creates a new metric.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// use airssys_wasm::core::{ComponentId, observability::{Metric, MetricType}};
    ///
    /// let metric = Metric::new(
    ///     ComponentId::new("my-component"),
    ///     MetricType::Counter {
    ///         name: "operations".to_string(),
    ///         value: 10,
    ///     },
    ///     HashMap::new(),
    /// );
    /// ```
    pub fn new(
        component_id: ComponentId,
        metric_type: MetricType,
        labels: HashMap<String, String>,
    ) -> Self {
        Self {
            component_id,
            metric_type,
            labels,
            timestamp: Utc::now(),
        }
    }

    /// Returns the metric name.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// use airssys_wasm::core::{ComponentId, observability::{Metric, MetricType}};
    ///
    /// let metric = Metric::new(
    ///     ComponentId::new("my-component"),
    ///     MetricType::Gauge {
    ///         name: "memory_usage".to_string(),
    ///         value: 512.0,
    ///     },
    ///     HashMap::new(),
    /// );
    ///
    /// assert_eq!(metric.name(), "memory_usage");
    /// ```
    pub fn name(&self) -> &str {
        self.metric_type.name()
    }
}

/// Metrics snapshot at a point in time.
///
/// Captures the state of all component metrics at a specific timestamp.
/// Used for exporting metrics to monitoring systems or generating reports.
///
/// # Examples
///
/// ```rust
/// use std::collections::HashMap;
/// use airssys_wasm::core::{ComponentId, observability::MetricsSnapshot};
///
/// let snapshot = MetricsSnapshot::new();
/// assert!(snapshot.component_metrics.is_empty());
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct MetricsSnapshot {
    /// Metrics organized by component.
    pub component_metrics: HashMap<ComponentId, Vec<Metric>>,

    /// Snapshot timestamp.
    pub timestamp: DateTime<Utc>,
}

impl MetricsSnapshot {
    /// Creates a new empty metrics snapshot.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::observability::MetricsSnapshot;
    ///
    /// let snapshot = MetricsSnapshot::new();
    /// ```
    pub fn new() -> Self {
        Self {
            component_metrics: HashMap::new(),
            timestamp: Utc::now(),
        }
    }

    /// Adds a metric to the snapshot.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// use airssys_wasm::core::{ComponentId, observability::{Metric, MetricType, MetricsSnapshot}};
    ///
    /// let mut snapshot = MetricsSnapshot::new();
    /// let component_id = ComponentId::new("my-component");
    /// let metric = Metric::new(
    ///     component_id.clone(),
    ///     MetricType::Counter { name: "test".to_string(), value: 1 },
    ///     HashMap::new(),
    /// );
    ///
    /// snapshot.add_metric(metric);
    /// assert_eq!(snapshot.component_metrics.len(), 1);
    /// ```
    pub fn add_metric(&mut self, metric: Metric) {
        self.component_metrics
            .entry(metric.component_id.clone())
            .or_default()
            .push(metric);
    }

    /// Returns metrics for a specific component.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// use airssys_wasm::core::{ComponentId, observability::{Metric, MetricType, MetricsSnapshot}};
    ///
    /// let mut snapshot = MetricsSnapshot::new();
    /// let component_id = ComponentId::new("my-component");
    /// let metric = Metric::new(
    ///     component_id.clone(),
    ///     MetricType::Counter { name: "test".to_string(), value: 1 },
    ///     HashMap::new(),
    /// );
    ///
    /// snapshot.add_metric(metric);
    /// assert_eq!(snapshot.metrics_for(&component_id).len(), 1);
    /// ```
    pub fn metrics_for(&self, component_id: &ComponentId) -> &[Metric] {
        self.component_metrics
            .get(component_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }
}

impl Default for MetricsSnapshot {
    fn default() -> Self {
        Self::new()
    }
}

/// Observability event for monitoring.
///
/// Represents a notable occurrence within a component that should be tracked
/// for monitoring, debugging, or auditing purposes. Events carry severity
/// levels and structured metadata.
///
/// # Examples
///
/// ```rust
/// use std::collections::HashMap;
/// use chrono::Utc;
/// use airssys_wasm::core::{ComponentId, observability::{ObservabilityEvent, EventSeverity}};
///
/// let event = ObservabilityEvent {
///     component_id: ComponentId::new("my-component"),
///     event_type: "component_started".to_string(),
///     severity: EventSeverity::Info,
///     message: "Component initialized successfully".to_string(),
///     metadata: HashMap::new(),
///     timestamp: Utc::now(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservabilityEvent {
    /// Component that generated this event.
    pub component_id: ComponentId,

    /// Event type identifier.
    pub event_type: String,

    /// Event severity level.
    pub severity: EventSeverity,

    /// Human-readable event message.
    pub message: String,

    /// Structured metadata for the event.
    pub metadata: HashMap<String, String>,

    /// Event timestamp.
    pub timestamp: DateTime<Utc>,
}

impl ObservabilityEvent {
    /// Creates a new observability event.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// use airssys_wasm::core::{ComponentId, observability::{ObservabilityEvent, EventSeverity}};
    ///
    /// let event = ObservabilityEvent::new(
    ///     ComponentId::new("my-component"),
    ///     "test_event",
    ///     EventSeverity::Info,
    ///     "Test message",
    ///     HashMap::new(),
    /// );
    /// ```
    pub fn new(
        component_id: ComponentId,
        event_type: impl Into<String>,
        severity: EventSeverity,
        message: impl Into<String>,
        metadata: HashMap<String, String>,
    ) -> Self {
        Self {
            component_id,
            event_type: event_type.into(),
            severity,
            message: message.into(),
            metadata,
            timestamp: Utc::now(),
        }
    }
}

/// Event severity level.
///
/// Classifies observability events by their severity, enabling filtering
/// and alert configuration. Severity levels follow standard logging conventions.
///
/// # Examples
///
/// ```rust
/// use airssys_wasm::core::observability::EventSeverity;
///
/// assert!(EventSeverity::Critical > EventSeverity::Error);
/// assert!(EventSeverity::Error > EventSeverity::Warning);
/// assert!(EventSeverity::Warning > EventSeverity::Info);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub enum EventSeverity {
    /// Debug-level events for detailed troubleshooting.
    Debug,

    /// Informational events for normal operations.
    Info,

    /// Warning events for potentially problematic situations.
    Warning,

    /// Error events for failures that don't halt the component.
    Error,

    /// Critical events for failures requiring immediate attention.
    Critical,
}

impl EventSeverity {
    /// Returns the severity as a string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::observability::EventSeverity;
    ///
    /// assert_eq!(EventSeverity::Info.as_str(), "info");
    /// assert_eq!(EventSeverity::Critical.as_str(), "critical");
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Critical => "critical",
        }
    }
}

/// Component health status.
///
/// Reports the health state of a component, including failure tracking
/// and diagnostic messages. Used for health checks and supervisory decisions.
///
/// # Examples
///
/// ```rust
/// use chrono::Utc;
/// use airssys_wasm::core::{ComponentId, observability::HealthStatus};
///
/// let healthy = HealthStatus {
///     component_id: ComponentId::new("my-component"),
///     is_healthy: true,
///     last_check: Utc::now(),
///     failure_count: 0,
///     message: None,
/// };
///
/// let unhealthy = HealthStatus {
///     component_id: ComponentId::new("my-component"),
///     is_healthy: false,
///     last_check: Utc::now(),
///     failure_count: 3,
///     message: Some("Connection timeout".to_string()),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthStatus {
    /// Component being monitored.
    pub component_id: ComponentId,

    /// Current health state.
    pub is_healthy: bool,

    /// Last health check timestamp.
    pub last_check: DateTime<Utc>,

    /// Number of consecutive failures.
    pub failure_count: u32,

    /// Optional diagnostic message.
    pub message: Option<String>,
}

impl HealthStatus {
    /// Creates a healthy status.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::{ComponentId, observability::HealthStatus};
    ///
    /// let status = HealthStatus::healthy(ComponentId::new("my-component"));
    /// assert!(status.is_healthy);
    /// assert_eq!(status.failure_count, 0);
    /// ```
    pub fn healthy(component_id: ComponentId) -> Self {
        Self {
            component_id,
            is_healthy: true,
            last_check: Utc::now(),
            failure_count: 0,
            message: None,
        }
    }

    /// Creates an unhealthy status.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::{ComponentId, observability::HealthStatus};
    ///
    /// let status = HealthStatus::unhealthy(
    ///     ComponentId::new("my-component"),
    ///     3,
    ///     Some("Connection failed"),
    /// );
    /// assert!(!status.is_healthy);
    /// assert_eq!(status.failure_count, 3);
    /// ```
    pub fn unhealthy(
        component_id: ComponentId,
        failure_count: u32,
        message: Option<impl Into<String>>,
    ) -> Self {
        Self {
            component_id,
            is_healthy: false,
            last_check: Utc::now(),
            failure_count,
            message: message.map(Into::into),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_type_name() {
        let counter = MetricType::Counter {
            name: "test_counter".to_string(),
            value: 42,
        };
        assert_eq!(counter.name(), "test_counter");

        let gauge = MetricType::Gauge {
            name: "test_gauge".to_string(),
            value: 42.5,
        };
        assert_eq!(gauge.name(), "test_gauge");

        let histogram = MetricType::Histogram {
            name: "test_histogram".to_string(),
            value: 0.5,
        };
        assert_eq!(histogram.name(), "test_histogram");

        let timing = MetricType::Timing {
            name: "test_timing".to_string(),
            duration_ms: 100,
        };
        assert_eq!(timing.name(), "test_timing");
    }

    #[test]
    fn test_metric_new() {
        let component_id = ComponentId::new("test-component");
        let metric_type = MetricType::Counter {
            name: "requests".to_string(),
            value: 10,
        };
        let labels = HashMap::new();

        let metric = Metric::new(component_id.clone(), metric_type, labels);

        assert_eq!(metric.component_id, component_id);
        assert_eq!(metric.name(), "requests");
    }

    #[test]
    fn test_metrics_snapshot_new() {
        let snapshot = MetricsSnapshot::new();
        assert!(snapshot.component_metrics.is_empty());
    }

    #[test]
    fn test_metrics_snapshot_add_metric() {
        let mut snapshot = MetricsSnapshot::new();
        let component_id = ComponentId::new("test-component");
        let metric = Metric::new(
            component_id.clone(),
            MetricType::Counter {
                name: "test".to_string(),
                value: 1,
            },
            HashMap::new(),
        );

        snapshot.add_metric(metric);
        assert_eq!(snapshot.component_metrics.len(), 1);
        assert_eq!(snapshot.metrics_for(&component_id).len(), 1);
    }

    #[test]
    fn test_observability_event_new() {
        let component_id = ComponentId::new("test-component");
        let event = ObservabilityEvent::new(
            component_id.clone(),
            "test_event",
            EventSeverity::Info,
            "Test message",
            HashMap::new(),
        );

        assert_eq!(event.component_id, component_id);
        assert_eq!(event.event_type, "test_event");
        assert_eq!(event.severity, EventSeverity::Info);
        assert_eq!(event.message, "Test message");
    }

    #[test]
    fn test_event_severity_ordering() {
        assert!(EventSeverity::Critical > EventSeverity::Error);
        assert!(EventSeverity::Error > EventSeverity::Warning);
        assert!(EventSeverity::Warning > EventSeverity::Info);
        assert!(EventSeverity::Info > EventSeverity::Debug);
    }

    #[test]
    fn test_event_severity_as_str() {
        assert_eq!(EventSeverity::Debug.as_str(), "debug");
        assert_eq!(EventSeverity::Info.as_str(), "info");
        assert_eq!(EventSeverity::Warning.as_str(), "warning");
        assert_eq!(EventSeverity::Error.as_str(), "error");
        assert_eq!(EventSeverity::Critical.as_str(), "critical");
    }

    #[test]
    fn test_health_status_healthy() {
        let component_id = ComponentId::new("test-component");
        let status = HealthStatus::healthy(component_id.clone());

        assert_eq!(status.component_id, component_id);
        assert!(status.is_healthy);
        assert_eq!(status.failure_count, 0);
        assert!(status.message.is_none());
    }

    #[test]
    fn test_health_status_unhealthy() {
        let component_id = ComponentId::new("test-component");
        let status = HealthStatus::unhealthy(
            component_id.clone(),
            5,
            Some("Connection timeout"),
        );

        assert_eq!(status.component_id, component_id);
        assert!(!status.is_healthy);
        assert_eq!(status.failure_count, 5);
        assert_eq!(status.message, Some("Connection timeout".to_string()));
    }
}
