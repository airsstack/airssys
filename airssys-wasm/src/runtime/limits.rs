//! Memory and resource limit enforcement for WASM components.
//!
//! This module implements mandatory resource limits for WebAssembly component execution,
//! establishing Layer 2 (WASM Linear Memory Isolation) of the 4-layer defense-in-depth
//! security architecture defined in ADR-WASM-006.
//!
//! # Architecture
//!
//! Resource limits are MANDATORY (ADR-WASM-002) and must be explicitly declared in
//! Component.toml. This module provides:
//!
//! - **ResourceLimits**: Configuration for memory limits with validation
//! - **ComponentResourceLimiter**: Wasmtime integration for runtime enforcement
//! - **MemoryMetrics**: Real-time monitoring of memory usage
//!
//! # Memory Limit Philosophy (ADR-WASM-002)
//!
//! **No Default Values - By Design:**
//! - Engineers MUST explicitly declare memory limits in Component.toml
//! - Forces consideration of actual resource requirements
//! - Prevents "works on my machine" production surprises
//! - Makes resource usage visible and intentional
//!
//! **Memory Range:**
//! - Minimum: 512KB (524,288 bytes) - One WASM page (64KB) feels too restrictive
//! - Maximum: 4MB (4,194,304 bytes) - Intentionally small for lightweight components
//! - Rationale: Encourages efficient component design, predictable resource usage
//!
//! # Security Properties (ADR-WASM-006)
//!
//! - **100% Memory Isolation**: Each component has isolated linear memory
//! - **No Shared Memory**: Components cannot access each other's memory
//! - **OOM Containment**: Out-of-memory errors do not crash host
//! - **Resource Accountability**: Every byte allocated is tracked and attributed
//!
//! # Design Rationale
//!
//! This module implements ADR-WASM-002 Section 2.4.2 (Memory Management):
//! - Mandatory limits prevent unbounded resource usage
//! - Explicit configuration prevents implicit assumptions
//! - Pre-instantiation validation catches configuration errors early
//! - Runtime enforcement prevents limit violations during execution
//!
//! # Usage with Wasmtime (Integration Guide)
//!
//! ## Basic Integration
//!
//! ```rust,ignore
//! use wasmtime::{Engine, Store};
//! use airssys_wasm::runtime::limits::{ResourceLimits, ComponentResourceLimiter};
//!
//! // 1. Create resource limits from Component.toml configuration
//! let limits = ResourceLimits::builder()
//!     .memory_bytes(1024 * 1024)  // 1MB limit
//!     .build()?;
//!
//! // 2. Create resource limiter for runtime enforcement
//! let limiter = ComponentResourceLimiter::new(limits);
//!
//! // 3. Create Wasmtime Store with limiter
//! let mut store = Store::new(&engine, limiter);
//!
//! // 4. Limiter automatically enforces memory limits during component execution
//! // Memory allocations exceeding limit will be denied by Wasmtime
//! ```
//!
//! ## Component.toml Configuration
//!
//! ```toml
//! [component]
//! name = "my-component"
//! version = "0.1.0"
//!
//! [resources.memory]
//! max_bytes = 1048576  # 1MB (MANDATORY - range: 512KB-4MB)
//! ```
//!
//! ## Monitoring Memory Usage
//!
//! ```rust,ignore
//! use wasmtime::{Engine, Store};
//! use airssys_wasm::runtime::limits::{ResourceLimits, ComponentResourceLimiter};
//!
//! let limits = ResourceLimits::builder()
//!     .memory_bytes(1024 * 1024)
//!     .build()?;
//!
//! let limiter = ComponentResourceLimiter::new(limits);
//! let mut store = Store::new(&engine, limiter);
//!
//! // Execute component...
//! // component.call(&mut store, ...)?;
//!
//! // Monitor memory usage after execution
//! let current_usage = store.data().current_memory_bytes();
//! let limit = store.data().limits().max_memory_bytes();
//! let percent = (current_usage as f64 / limit as f64) * 100.0;
//!
//! println!("Memory: {current_usage}/{limit} bytes ({percent:.1}%)");
//! ```
//!
//! ## Handling Out-of-Memory (OOM)
//!
//! ### OOM Detection and Error Handling
//!
//! ```rust,ignore
//! use wasmtime::{Engine, Store};
//! use airssys_wasm::runtime::limits::{ResourceLimits, ComponentResourceLimiter};
//! use airssys_wasm::core::error::WasmError;
//!
//! let limits = ResourceLimits::builder()
//!     .memory_bytes(512 * 1024)  // Small 512KB limit
//!     .build()?;
//!
//! let limiter = ComponentResourceLimiter::new(limits);
//! let mut store = Store::new(&engine, limiter);
//!
//! // Component execution may fail if memory limit exceeded
//! match component.call(&mut store, ...) {
//!     Ok(result) => println!("Success: {result:?}"),
//!     Err(e) if e.to_string().contains("memory") => {
//!         // OOM detected - create detailed error
//!         let requested = 1024 * 1024;  // Example: tried to allocate 1MB
//!         let error = store.data().create_oom_error(requested);
//!         eprintln!("OOM Error: {error}");
//!         // Component MUST be shut down - no recovery possible
//!         return Err(error.into());
//!     }
//!     Err(e) => return Err(e),
//! }
//! ```
//!
//! ### OOM Warning Threshold Detection (90%)
//!
//! ```rust,ignore
//! use airssys_wasm::runtime::limits::ComponentResourceLimiter;
//!
//! let mut store = Store::new(&engine, limiter);
//!
//! // Before each component call, check for OOM warning
//! if store.data().is_near_oom() {
//!     eprintln!("Warning: Memory usage >= 90%, implementing defensive measures");
//!     // Option 1: Reject non-critical requests
//!     // Option 2: Trigger garbage collection (if component supports it)
//!     // Option 3: Scale back cache sizes
//! }
//!
//! // Execute component...
//! component.call(&mut store, ...)?;
//!
//! // After execution, check if peak usage approached OOM
//! if store.data().peak_exceeded_oom_threshold() {
//!     eprintln!("Component approached OOM during execution - consider increasing limit");
//! }
//! ```
//!
//! ### OOM Recovery Patterns
//!
//! **IMPORTANT: OOM is NOT recoverable within the same component instance.**
//!
//! When OOM occurs (`memory_growing()` returns `false`), Wasmtime will:
//! 1. Deny the allocation request
//! 2. Trap the component execution
//! 3. Return an error to the host
//!
//! **Recommended Recovery Procedure:**
//!
//! ```rust,ignore
//! use airssys_wasm::runtime::limits::ComponentResourceLimiter;
//!
//! // 1. Detect OOM error from component execution
//! match component.call(&mut store, request) {
//!     Err(e) if e.to_string().contains("memory") => {
//!         // 2. Log detailed OOM diagnostics
//!         let metrics = store.data().metrics();
//!         eprintln!("OOM: {}/{} bytes used ({:.1}%)",
//!             metrics.current_bytes,
//!             metrics.max_bytes,
//!             metrics.usage_percentage()
//!         );
//!         eprintln!("Peak: {} bytes, Allocations: {}",
//!             metrics.peak_bytes,
//!             metrics.allocation_count
//!         );
//!
//!         // 3. Shut down component gracefully
//!         drop(store);  // Release component resources
//!
//!         // 4. Notify upstream systems (if using airssys-rt)
//!         // supervisor.notify_component_failed(component_id, error)?;
//!
//!         // 5. Decision point:
//!         // - Increase memory limit in Component.toml and restart
//!         // - Replace with more memory-efficient component
//!         // - Reject workload as too large
//!     }
//!     Ok(result) => { /* Success */ }
//!     Err(e) => return Err(e),
//! }
//! ```
//!
//! ### Graceful Degradation (Before OOM)
//!
//! ```rust,ignore
//! use airssys_wasm::runtime::limits::ComponentResourceLimiter;
//!
//! // Monitor memory during execution loop
//! loop {
//!     let metrics = store.data().metrics();
//!
//!     // Implement tiered response based on memory pressure
//!     if metrics.exceeds_threshold(95.0) {
//!         // Critical: Reject all new requests
//!         return Err(WasmError::resource_exhausted("Memory critically low"));
//!     } else if metrics.exceeds_threshold(90.0) {
//!         // Warning: Reduce cache sizes, reject non-critical requests
//!         cache.evict_half();
//!     } else if metrics.exceeds_threshold(75.0) {
//!         // Caution: Stop pre-fetching, reduce batch sizes
//!         disable_prefetch();
//!     }
//!
//!     // Process request...
//!     component.call(&mut store, request)?;
//! }
//! ```
//!
//! ## Integration with airssys-rt (Future)
//!
//! ### Actor-Based Component Hosting with OOM Recovery
//!
//! ```rust,ignore
//! use airssys_rt::actor::Actor;
//! use airssys_rt::supervisor::{SupervisorStrategy, RestartPolicy};
//! use airssys_wasm::runtime::limits::{ResourceLimits, ComponentResourceLimiter};
//!
//! struct ComponentActor {
//!     limiter: ComponentResourceLimiter,
//!     store: Store<ComponentResourceLimiter>,
//!     component_id: String,
//! }
//!
//! impl ComponentActor {
//!     fn new(engine: &Engine, limits: ResourceLimits, component_id: String) -> Self {
//!         let limiter = ComponentResourceLimiter::new(limits);
//!         let store = Store::new(engine, limiter.clone());
//!         Self { limiter, store, component_id }
//!     }
//!
//!     fn memory_metrics(&self) -> MemoryMetrics {
//!         self.limiter.metrics()
//!     }
//!
//!     fn handle_oom(&mut self, requested_bytes: u64) -> WasmError {
//!         let error = self.limiter.create_oom_error(requested_bytes);
//!         eprintln!("Component {} OOM: {}", self.component_id, error);
//!         // Supervisor will handle restart based on RestartPolicy
//!         error
//!     }
//! }
//!
//! // Supervisor configuration for OOM handling
//! let supervisor = Supervisor::builder()
//!     .strategy(SupervisorStrategy::OneForOne)
//!     .restart_policy(RestartPolicy::Transient)  // Restart on OOM
//!     .max_restarts(3)
//!     .within_seconds(60)
//!     .build()?;
//! ```
//!
//! # Examples
//!
//! ```rust,ignore
//! use airssys_wasm::runtime::limits::{ResourceLimits, MemoryConfig};
//!
//! // Create limits from explicit configuration
//! let limits = ResourceLimits::builder()
//!     .max_memory_bytes(2 * 1024 * 1024)  // 2MB
//!     .build()?;
//!
//! // Parse from Component.toml configuration
//! let config = MemoryConfig {
//!     max_memory_bytes: 2 * 1024 * 1024,
//! };
//! let limits = ResourceLimits::try_from(config)?;
//!
//! // Use with Wasmtime engine
//! let mut store = wasmtime::Store::new(&engine, ());
//! store.limiter(|_| ComponentResourceLimiter::new(limits));
//! ```
//!
//! # References
//!
//! - **ADR-WASM-002**: WASM Runtime Engine Selection (Section 2.4.2 Memory Management)
//! - **ADR-WASM-006**: Component Isolation and Sandboxing (Layer 2 architecture)
//! - **WASM-TASK-002**: Block 1 Runtime Engine Implementation
//! - **Workspace Standards**: §2.1 (imports), §6.3 (Microsoft Rust Guidelines)

// Layer 1: Standard library imports
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

// Layer 2: Third-party crate imports
use wasmtime::ResourceLimiter;

// Layer 3: Internal module imports
use crate::core::config::ResourceLimits;
use crate::core::error::WasmError;

/// Wasmtime ResourceLimiter implementation for component memory enforcement.
///
/// This struct integrates with Wasmtime's resource limiting system to enforce
/// memory limits during component execution. It provides real-time tracking of
/// memory usage and enforces the limits defined in ResourceLimits.
///
/// # Thread Safety
///
/// Memory tracking uses atomic operations for thread-safe updates across async
/// execution contexts.
///
/// # Examples
///
/// ```rust,ignore
/// use airssys_wasm::runtime::limits::{ResourceLimits, ComponentResourceLimiter};
/// use wasmtime::Store;
///
/// let limits = ResourceLimits::builder()
///     .max_memory_bytes(2 * 1024 * 1024)
///     .build()?;
///
/// let mut store = Store::new(&engine, ());
/// store.limiter(|_| ComponentResourceLimiter::new(limits));
/// ```
#[derive(Debug, Clone)]
pub struct ComponentResourceLimiter {
    limits: ResourceLimits,
    current_memory_bytes: Arc<AtomicU64>,
    peak_memory_bytes: Arc<AtomicU64>,
    allocation_count: Arc<AtomicU64>,
}

impl ComponentResourceLimiter {
    /// OOM threshold percentage for warnings (90%).
    ///
    /// When memory usage exceeds this threshold, applications should consider
    /// implementing defensive measures (graceful degradation, cleanup, etc.).
    pub const OOM_WARNING_THRESHOLD: f64 = 90.0;

    /// Create a new ComponentResourceLimiter with the specified limits.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let limits = ResourceLimits::builder()
    ///     .max_memory_bytes(2 * 1024 * 1024)
    ///     .build()?;
    /// let limiter = ComponentResourceLimiter::new(limits);
    /// ```
    pub fn new(limits: ResourceLimits) -> Self {
        Self {
            limits,
            current_memory_bytes: Arc::new(AtomicU64::new(0)),
            peak_memory_bytes: Arc::new(AtomicU64::new(0)),
            allocation_count: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Get the current memory usage in bytes.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let usage = limiter.current_memory_bytes();
    /// println!("Current memory usage: {} bytes", usage);
    /// ```
    pub fn current_memory_bytes(&self) -> u64 {
        self.current_memory_bytes.load(Ordering::Relaxed)
    }

    /// Get the peak memory usage in bytes since creation.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let peak = limiter.peak_memory_bytes();
    /// println!("Peak memory usage: {} bytes", peak);
    /// ```
    pub fn peak_memory_bytes(&self) -> u64 {
        self.peak_memory_bytes.load(Ordering::Relaxed)
    }

    /// Get the total number of memory allocation events.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let count = limiter.allocation_count();
    /// println!("Total allocations: {}", count);
    /// ```
    pub fn allocation_count(&self) -> u64 {
        self.allocation_count.load(Ordering::Relaxed)
    }

    /// Get the configured memory limits.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let limits = limiter.limits();
    /// println!("Max memory: {} bytes", limits.max_memory_bytes());
    /// ```
    pub fn limits(&self) -> ResourceLimits {
        self.limits
    }

    /// Reset metrics tracking (for testing or component restart).
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// limiter.reset_metrics();
    /// assert_eq!(limiter.current_memory_bytes(), 0);
    /// assert_eq!(limiter.peak_memory_bytes(), 0);
    /// ```
    pub fn reset_metrics(&self) {
        self.current_memory_bytes.store(0, Ordering::Relaxed);
        self.peak_memory_bytes.store(0, Ordering::Relaxed);
        self.allocation_count.store(0, Ordering::Relaxed);
    }

    /// Check if current memory usage exceeds the OOM warning threshold (90%).
    ///
    /// Returns true if memory usage is at or above 90% of the configured limit.
    /// Applications should implement defensive measures when this returns true.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// if limiter.is_near_oom() {
    ///     eprintln!("Warning: Memory usage >= 90%, consider cleanup");
    ///     // Implement graceful degradation or cleanup
    /// }
    /// ```
    pub fn is_near_oom(&self) -> bool {
        let metrics = self.metrics();
        metrics.exceeds_threshold(Self::OOM_WARNING_THRESHOLD)
    }

    /// Check if peak memory usage exceeded the OOM warning threshold.
    ///
    /// Returns true if peak memory usage ever reached or exceeded 90% of limit.
    /// Useful for post-execution analysis and diagnostics.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// if limiter.peak_exceeded_oom_threshold() {
    ///     eprintln!("Component came close to OOM during execution");
    /// }
    /// ```
    pub fn peak_exceeded_oom_threshold(&self) -> bool {
        let metrics = self.metrics();
        metrics.peak_exceeded_threshold(Self::OOM_WARNING_THRESHOLD)
    }
}

impl ResourceLimiter for ComponentResourceLimiter {
    fn memory_growing(
        &mut self,
        _current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> anyhow::Result<bool> {
        let desired_u64 = desired as u64;
        let max_allowed = self.limits.max_memory_bytes();

        if desired_u64 > max_allowed {
            return Ok(false);
        }

        self.current_memory_bytes
            .store(desired_u64, Ordering::Relaxed);
        self.allocation_count.fetch_add(1, Ordering::Relaxed);

        let mut current_peak = self.peak_memory_bytes.load(Ordering::Relaxed);
        while desired_u64 > current_peak {
            match self.peak_memory_bytes.compare_exchange_weak(
                current_peak,
                desired_u64,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(new_peak) => current_peak = new_peak,
            }
        }

        Ok(true)
    }

    fn table_growing(
        &mut self,
        _current: u32,
        _desired: u32,
        _maximum: Option<u32>,
    ) -> anyhow::Result<bool> {
        Ok(true)
    }
}

/// Real-time memory usage metrics for a component.
///
/// Provides snapshot of current memory usage and limits for monitoring and diagnostics.
///
/// # Examples
///
/// ```rust,ignore
/// let metrics = limiter.metrics();
/// println!("Memory usage: {}/{} bytes ({:.1}%)",
///     metrics.current_bytes,
///     metrics.max_bytes,
///     metrics.usage_percentage()
/// );
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryMetrics {
    pub current_bytes: u64,
    pub max_bytes: u64,
    pub peak_bytes: u64,
    pub allocation_count: u64,
}

impl MemoryMetrics {
    /// Calculate the current memory usage as a percentage of the limit.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let metrics = MemoryMetrics {
    ///     current_bytes: 1024 * 1024,
    ///     max_bytes: 2 * 1024 * 1024,
    ///     peak_bytes: 1536 * 1024,
    ///     allocation_count: 5,
    /// };
    /// assert_eq!(metrics.usage_percentage(), 50.0);
    /// ```
    pub fn usage_percentage(&self) -> f64 {
        if self.max_bytes == 0 {
            0.0
        } else {
            (self.current_bytes as f64 / self.max_bytes as f64) * 100.0
        }
    }

    /// Calculate the peak memory usage as a percentage of the limit.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let metrics = MemoryMetrics {
    ///     current_bytes: 1024 * 1024,
    ///     max_bytes: 2 * 1024 * 1024,
    ///     peak_bytes: 1536 * 1024,
    ///     allocation_count: 5,
    /// };
    /// assert_eq!(metrics.peak_percentage(), 75.0);
    /// ```
    pub fn peak_percentage(&self) -> f64 {
        if self.max_bytes == 0 {
            0.0
        } else {
            (self.peak_bytes as f64 / self.max_bytes as f64) * 100.0
        }
    }

    /// Check if memory usage exceeds a threshold percentage.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// if metrics.exceeds_threshold(90.0) {
    ///     eprintln!("Warning: Memory usage above 90%");
    /// }
    /// ```
    pub fn exceeds_threshold(&self, threshold_percent: f64) -> bool {
        self.usage_percentage() > threshold_percent
    }

    /// Check if peak memory usage exceeded a threshold percentage.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// if metrics.peak_exceeded_threshold(95.0) {
    ///     eprintln!("Warning: Peak memory usage exceeded 95%");
    /// }
    /// ```
    pub fn peak_exceeded_threshold(&self, threshold_percent: f64) -> bool {
        self.peak_percentage() > threshold_percent
    }

    /// Get remaining available memory in bytes.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let remaining = metrics.remaining_bytes();
    /// println!("Remaining memory: {} bytes", remaining);
    /// ```
    pub fn remaining_bytes(&self) -> u64 {
        self.max_bytes.saturating_sub(self.current_bytes)
    }

    /// Check if component is close to the memory limit.
    ///
    /// Returns true if remaining memory is less than the specified margin.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// if metrics.is_near_limit(64 * 1024) {
    ///     eprintln!("Warning: Less than 64KB remaining");
    /// }
    /// ```
    pub fn is_near_limit(&self, margin_bytes: u64) -> bool {
        self.remaining_bytes() < margin_bytes
    }
}

impl ComponentResourceLimiter {
    /// Get current memory metrics snapshot.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let metrics = limiter.metrics();
    /// println!("Current: {} / Max: {} ({:.1}%)",
    ///     metrics.current_bytes,
    ///     metrics.max_bytes,
    ///     metrics.usage_percentage()
    /// );
    /// ```
    pub fn metrics(&self) -> MemoryMetrics {
        MemoryMetrics {
            current_bytes: self.current_memory_bytes(),
            max_bytes: self.limits.max_memory_bytes(),
            peak_bytes: self.peak_memory_bytes(),
            allocation_count: self.allocation_count(),
        }
    }

    /// Create an OutOfMemory error with current metrics.
    ///
    /// This method should be called when OOM is detected to create a detailed
    /// error with memory usage information.
    ///
    /// # Arguments
    ///
    /// * `requested_bytes` - The amount of memory that was requested when OOM occurred
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// if oom_detected {
    ///     let error = limiter.create_oom_error(2 * 1024 * 1024);
    ///     return Err(error);
    /// }
    /// ```
    pub fn create_oom_error(&self, requested_bytes: u64) -> WasmError {
        let limit_bytes = self.limits.max_memory_bytes();
        WasmError::out_of_memory(limit_bytes, requested_bytes)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::core::config::{CpuConfig, MemoryConfig, ResourceConfig};

    #[test]
    fn test_resource_limits_builder_valid() {
        let limits = ResourceLimits::builder()
            .max_memory_bytes(1024 * 1024)
            .max_fuel(10_000)
            .timeout_seconds(30)
            .build()
            .expect("Should build valid limits");

        assert_eq!(limits.max_memory_bytes(), 1024 * 1024);
        assert_eq!(limits.max_fuel(), 10_000);
        assert_eq!(limits.timeout_seconds(), 30);
    }

    #[test]
    fn test_resource_limits_builder_missing_memory() {
        let result = ResourceLimits::builder().build();
        assert!(result.is_err());
        let err = result.expect_err("Should fail without memory limit");
        assert!(err.to_string().contains("MANDATORY"));
    }

    #[test]
    fn test_resource_limits_below_minimum() {
        let result = ResourceLimits::builder()
            .max_memory_bytes(256 * 1024)
            .max_fuel(10_000)
            .timeout_seconds(30)
            .build();

        assert!(result.is_err());
        let err = result.expect_err("Should fail below minimum");
        assert!(err.to_string().contains("below minimum"));
    }

    #[test]
    fn test_resource_limits_above_maximum() {
        let result = ResourceLimits::builder()
            .max_memory_bytes(8 * 1024 * 1024)
            .max_fuel(10_000)
            .timeout_seconds(30)
            .build();

        assert!(result.is_err());
        let err = result.expect_err("Should fail above maximum");
        assert!(err.to_string().contains("exceeds maximum"));
    }

    #[test]
    fn test_resource_limits_at_minimum() {
        let limits = ResourceLimits::builder()
            .max_memory_bytes(ResourceLimits::MIN_MEMORY_BYTES)
            .max_fuel(10_000)
            .timeout_seconds(30)
            .build()
            .expect("Should build at minimum");

        assert_eq!(limits.max_memory_bytes(), ResourceLimits::MIN_MEMORY_BYTES);
    }

    #[test]
    fn test_resource_limits_at_maximum() {
        let limits = ResourceLimits::builder()
            .max_memory_bytes(ResourceLimits::MAX_MEMORY_BYTES)
            .max_fuel(10_000)
            .timeout_seconds(30)
            .build()
            .expect("Should build at maximum");

        assert_eq!(limits.max_memory_bytes(), ResourceLimits::MAX_MEMORY_BYTES);
    }

    #[test]
    fn test_resource_config_to_resource_limits() {
        let memory_config = MemoryConfig {
            max_memory_bytes: 2 * 1024 * 1024,
        };
        let cpu_config = CpuConfig {
            max_fuel: 10_000,
            timeout_seconds: 30,
        };
        let config = ResourceConfig {
            memory: memory_config,
            cpu: cpu_config,
        };

        let limits = ResourceLimits::try_from(config).expect("Should convert config");
        assert_eq!(limits.max_memory_bytes(), 2 * 1024 * 1024);
        assert_eq!(limits.max_fuel(), 10_000);
        assert_eq!(limits.timeout_seconds(), 30);
    }

    #[test]
    fn test_component_resource_limiter_new() {
        let limits = ResourceLimits::builder()
            .max_memory_bytes(2 * 1024 * 1024)
            .max_fuel(10_000)
            .timeout_seconds(30)
            .build()
            .expect("Should build limits");

        let limiter = ComponentResourceLimiter::new(limits);
        assert_eq!(limiter.current_memory_bytes(), 0);
        assert_eq!(limiter.limits().max_memory_bytes(), 2 * 1024 * 1024);
    }

    // Serde deserialization tests (Subtask 2.1.2)

    #[test]
    fn test_memory_config_deserialize_valid() {
        let json = r#"{"max_memory_bytes": 2097152}"#;
        let config: MemoryConfig =
            serde_json::from_str(json).expect("Should deserialize valid config");

        assert_eq!(config.max_memory_bytes, 2 * 1024 * 1024);
    }

    #[test]
    fn test_memory_config_serialize() {
        let config = MemoryConfig {
            max_memory_bytes: 2 * 1024 * 1024,
        };

        let json = serde_json::to_string(&config).expect("Should serialize");
        assert!(json.contains("2097152"));
    }

    #[test]
    fn test_memory_config_validate_below_minimum() {
        let config = MemoryConfig {
            max_memory_bytes: 256 * 1024,
        };

        let result = config.validate();
        assert!(result.is_err());
        let err = result.expect_err("Should fail validation below minimum");
        assert!(err.to_string().contains("below minimum"));
    }

    #[test]
    fn test_memory_config_validate_above_maximum() {
        let config = MemoryConfig {
            max_memory_bytes: 8 * 1024 * 1024,
        };

        let result = config.validate();
        assert!(result.is_err());
        let err = result.expect_err("Should fail validation above maximum");
        assert!(err.to_string().contains("exceeds maximum"));
    }

    #[test]
    fn test_memory_config_validate_at_boundaries() {
        // Test minimum boundary
        let config_min = MemoryConfig {
            max_memory_bytes: ResourceLimits::MIN_MEMORY_BYTES,
        };
        assert!(config_min.validate().is_ok());

        // Test maximum boundary
        let config_max = MemoryConfig {
            max_memory_bytes: ResourceLimits::MAX_MEMORY_BYTES,
        };
        assert!(config_max.validate().is_ok());
    }

    #[test]
    fn test_memory_config_roundtrip() {
        let original = MemoryConfig {
            max_memory_bytes: 1024 * 1024,
        };

        let json = serde_json::to_string(&original).expect("Should serialize");
        let deserialized: MemoryConfig = serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_memory_metrics_usage_percentage() {
        let metrics = MemoryMetrics {
            current_bytes: 1024 * 1024,
            max_bytes: 2 * 1024 * 1024,
            peak_bytes: 1536 * 1024,
            allocation_count: 5,
        };

        assert_eq!(metrics.usage_percentage(), 50.0);
    }

    #[test]
    fn test_memory_metrics_peak_percentage() {
        let metrics = MemoryMetrics {
            current_bytes: 1024 * 1024,
            max_bytes: 2 * 1024 * 1024,
            peak_bytes: 1536 * 1024,
            allocation_count: 5,
        };

        assert_eq!(metrics.peak_percentage(), 75.0);
    }

    #[test]
    fn test_memory_metrics_exceeds_threshold() {
        let metrics = MemoryMetrics {
            current_bytes: 1900 * 1024,
            max_bytes: 2 * 1024 * 1024,
            peak_bytes: 1900 * 1024,
            allocation_count: 3,
        };

        assert!(!metrics.exceeds_threshold(95.0));
        assert!(metrics.exceeds_threshold(90.0));
        assert!(metrics.exceeds_threshold(50.0));
    }

    #[test]
    fn test_memory_metrics_peak_exceeded_threshold() {
        let metrics = MemoryMetrics {
            current_bytes: 1024 * 1024,
            max_bytes: 2 * 1024 * 1024,
            peak_bytes: 1900 * 1024,
            allocation_count: 10,
        };

        assert!(metrics.peak_exceeded_threshold(90.0));
        assert!(!metrics.peak_exceeded_threshold(95.0));
    }

    #[test]
    fn test_memory_metrics_remaining_bytes() {
        let metrics = MemoryMetrics {
            current_bytes: 1024 * 1024,
            max_bytes: 2 * 1024 * 1024,
            peak_bytes: 1024 * 1024,
            allocation_count: 1,
        };

        assert_eq!(metrics.remaining_bytes(), 1024 * 1024);
    }

    #[test]
    fn test_memory_metrics_is_near_limit() {
        let metrics = MemoryMetrics {
            current_bytes: 2 * 1024 * 1024 - 32 * 1024,
            max_bytes: 2 * 1024 * 1024,
            peak_bytes: 2 * 1024 * 1024 - 32 * 1024,
            allocation_count: 5,
        };

        assert!(metrics.is_near_limit(64 * 1024));
        assert!(!metrics.is_near_limit(16 * 1024));
    }

    #[test]
    fn test_component_resource_limiter_peak_tracking() {
        let limits = ResourceLimits::builder()
            .max_memory_bytes(2 * 1024 * 1024)
            .max_fuel(10_000)
            .timeout_seconds(30)
            .build()
            .expect("Should build limits");

        let mut limiter = ComponentResourceLimiter::new(limits);

        limiter
            .memory_growing(0, 512 * 1024, None)
            .expect("Should allow 512KB");
        assert_eq!(limiter.peak_memory_bytes(), 512 * 1024);

        limiter
            .memory_growing(512 * 1024, 1024 * 1024, None)
            .expect("Should allow 1MB");
        assert_eq!(limiter.peak_memory_bytes(), 1024 * 1024);

        limiter
            .memory_growing(1024 * 1024, 768 * 1024, None)
            .expect("Should allow shrinking to 768KB");
        assert_eq!(limiter.peak_memory_bytes(), 1024 * 1024);
    }

    #[test]
    fn test_component_resource_limiter_allocation_count() {
        let limits = ResourceLimits::builder()
            .max_memory_bytes(2 * 1024 * 1024)
            .max_fuel(10_000)
            .timeout_seconds(30)
            .build()
            .expect("Should build limits");

        let mut limiter = ComponentResourceLimiter::new(limits);

        assert_eq!(limiter.allocation_count(), 0);

        limiter
            .memory_growing(0, 512 * 1024, None)
            .expect("Should allow allocation");
        assert_eq!(limiter.allocation_count(), 1);

        limiter
            .memory_growing(512 * 1024, 1024 * 1024, None)
            .expect("Should allow allocation");
        assert_eq!(limiter.allocation_count(), 2);

        limiter
            .memory_growing(1024 * 1024, 1536 * 1024, None)
            .expect("Should allow allocation");
        assert_eq!(limiter.allocation_count(), 3);
    }

    #[test]
    fn test_component_resource_limiter_reset_metrics() {
        let limits = ResourceLimits::builder()
            .max_memory_bytes(2 * 1024 * 1024)
            .max_fuel(10_000)
            .timeout_seconds(30)
            .build()
            .expect("Should build limits");

        let mut limiter = ComponentResourceLimiter::new(limits);

        limiter
            .memory_growing(0, 1024 * 1024, None)
            .expect("Should allow allocation");
        assert_eq!(limiter.current_memory_bytes(), 1024 * 1024);
        assert_eq!(limiter.peak_memory_bytes(), 1024 * 1024);
        assert_eq!(limiter.allocation_count(), 1);

        limiter.reset_metrics();
        assert_eq!(limiter.current_memory_bytes(), 0);
        assert_eq!(limiter.peak_memory_bytes(), 0);
        assert_eq!(limiter.allocation_count(), 0);
    }

    #[test]
    fn test_component_resource_limiter_metrics_snapshot() {
        let limits = ResourceLimits::builder()
            .max_memory_bytes(2 * 1024 * 1024)
            .max_fuel(10_000)
            .timeout_seconds(30)
            .build()
            .expect("Should build limits");

        let mut limiter = ComponentResourceLimiter::new(limits);

        limiter
            .memory_growing(0, 512 * 1024, None)
            .expect("Should allow allocation");
        limiter
            .memory_growing(512 * 1024, 1024 * 1024, None)
            .expect("Should allow allocation");

        let metrics = limiter.metrics();
        assert_eq!(metrics.current_bytes, 1024 * 1024);
        assert_eq!(metrics.max_bytes, 2 * 1024 * 1024);
        assert_eq!(metrics.peak_bytes, 1024 * 1024);
        assert_eq!(metrics.allocation_count, 2);
        assert_eq!(metrics.usage_percentage(), 50.0);
        assert_eq!(metrics.remaining_bytes(), 1024 * 1024);
    }

    #[test]
    fn test_component_resource_limiter_create_oom_error() {
        let limits = ResourceLimits::builder()
            .max_memory_bytes(512 * 1024)
            .max_fuel(10_000)
            .timeout_seconds(30)
            .build()
            .expect("Should build limits");

        let mut limiter = ComponentResourceLimiter::new(limits);

        limiter
            .memory_growing(0, 500 * 1024, None)
            .expect("Should allow allocation");

        let error = limiter.create_oom_error(600 * 1024);
        let error_str = error.to_string();
        assert!(error_str.contains("524288"));
        assert!(error_str.contains("614400"));
    }

    #[test]
    fn test_oom_detection_at_exact_limit() {
        let limits = ResourceLimits::builder()
            .max_memory_bytes(1024 * 1024)
            .max_fuel(10_000)
            .timeout_seconds(30)
            .build()
            .expect("Should build limits");

        let mut limiter = ComponentResourceLimiter::new(limits);

        let result = limiter.memory_growing(0, 1024 * 1024, None);
        assert!(result.is_ok());
        assert!(result.unwrap());
        assert_eq!(limiter.current_memory_bytes(), 1024 * 1024);
    }

    #[test]
    fn test_oom_rejection_one_byte_over() {
        let limits = ResourceLimits::builder()
            .max_memory_bytes(1024 * 1024)
            .max_fuel(10_000)
            .timeout_seconds(30)
            .build()
            .expect("Should build limits");

        let mut limiter = ComponentResourceLimiter::new(limits);

        let result = limiter.memory_growing(0, 1024 * 1024 + 1, None);
        assert!(result.is_ok());
        assert!(!result.unwrap());
        assert_eq!(limiter.current_memory_bytes(), 0);
    }

    #[test]
    fn test_oom_multiple_allocations_leading_to_limit() {
        let limits = ResourceLimits::builder()
            .max_memory_bytes(1024 * 1024)
            .max_fuel(10_000)
            .timeout_seconds(30)
            .build()
            .expect("Should build limits");

        let mut limiter = ComponentResourceLimiter::new(limits);

        assert!(limiter.memory_growing(0, 512 * 1024, None).unwrap());
        assert_eq!(limiter.current_memory_bytes(), 512 * 1024);

        assert!(limiter
            .memory_growing(512 * 1024, 768 * 1024, None)
            .unwrap());
        assert_eq!(limiter.current_memory_bytes(), 768 * 1024);

        assert!(limiter
            .memory_growing(768 * 1024, 1024 * 1024, None)
            .unwrap());
        assert_eq!(limiter.current_memory_bytes(), 1024 * 1024);

        let result = limiter.memory_growing(1024 * 1024, 1024 * 1024 + 64, None);
        assert!(!result.unwrap());
        assert_eq!(limiter.current_memory_bytes(), 1024 * 1024);
    }

    #[test]
    fn test_oom_metrics_accuracy_after_rejection() {
        let limits = ResourceLimits::builder()
            .max_memory_bytes(512 * 1024)
            .max_fuel(10_000)
            .timeout_seconds(30)
            .build()
            .expect("Should build limits");

        let mut limiter = ComponentResourceLimiter::new(limits);

        limiter
            .memory_growing(0, 400 * 1024, None)
            .expect("Should allow allocation");

        let rejected = limiter.memory_growing(400 * 1024, 600 * 1024, None);
        assert!(!rejected.unwrap());

        let metrics = limiter.metrics();
        assert_eq!(metrics.current_bytes, 400 * 1024);
        assert_eq!(metrics.peak_bytes, 400 * 1024);
        assert_eq!(metrics.allocation_count, 1);
    }

    #[test]
    fn test_oom_error_contains_correct_details() {
        let limits = ResourceLimits::builder()
            .max_memory_bytes(1024 * 1024)
            .max_fuel(10_000)
            .timeout_seconds(30)
            .build()
            .expect("Should build limits");

        let limiter = ComponentResourceLimiter::new(limits);

        let requested_bytes = 2 * 1024 * 1024;
        let error = limiter.create_oom_error(requested_bytes);

        let error_str = error.to_string();
        assert!(error_str.contains("1048576"));
        assert!(error_str.contains("2097152"));
        assert!(error_str.to_lowercase().contains("memory"));
    }

    #[test]
    fn test_is_near_oom_threshold_detection() {
        let limits = ResourceLimits::builder()
            .max_memory_bytes(1024 * 1024)
            .max_fuel(10_000)
            .timeout_seconds(30)
            .build()
            .expect("Should build limits");

        let mut limiter = ComponentResourceLimiter::new(limits);

        assert!(!limiter.is_near_oom());

        limiter
            .memory_growing(0, 850 * 1024, None)
            .expect("Should allow 850KB");
        assert!(!limiter.is_near_oom());

        limiter
            .memory_growing(850 * 1024, 922 * 1024, None)
            .expect("Should allow 922KB (90.04%)");
        assert!(limiter.is_near_oom());

        limiter
            .memory_growing(922 * 1024, 980 * 1024, None)
            .expect("Should allow 980KB");
        assert!(limiter.is_near_oom());
    }

    #[test]
    fn test_peak_exceeded_oom_threshold() {
        let limits = ResourceLimits::builder()
            .max_memory_bytes(1024 * 1024)
            .max_fuel(10_000)
            .timeout_seconds(30)
            .build()
            .expect("Should build limits");

        let mut limiter = ComponentResourceLimiter::new(limits);

        limiter
            .memory_growing(0, 950 * 1024, None)
            .expect("Should allow 950KB");
        assert!(limiter.peak_exceeded_oom_threshold());

        limiter
            .memory_growing(950 * 1024, 500 * 1024, None)
            .expect("Should allow shrinking");
        assert!(!limiter.is_near_oom());
        assert!(limiter.peak_exceeded_oom_threshold());
    }
}
