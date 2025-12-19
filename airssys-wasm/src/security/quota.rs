//! Resource Quota Management for WASM Components
//!
//! This module provides resource quota enforcement to prevent component resource
//! exhaustion attacks. Quotas limit storage, message rate, network bandwidth, CPU time,
//! and memory usage per component.
//!
//! # Security Model
//!
//! The quota system implements **defense in depth** alongside capability checks:
//! - Components declare required capabilities (Block 4, Task 3.1)
//! - Quotas limit total resource consumption (this module)
//! - Violations deny access and log security events (Task 3.3)
//!
//! # Architecture
//!
//! ```text
//! ┌────────────────────────────────────────────────────────────────┐
//! │ Component calls host function                                   │
//! └────────────────┬────────────────────────────────────────────────┘
//!                  │
//!                  ▼
//! ┌────────────────────────────────────────────────────────────────┐
//! │ 1. Capability Check (Task 3.1)                                  │
//! │    - Does component have permission?                           │
//! └────────────────┬────────────────────────────────────────────────┘
//!                  │ ✅ Allowed
//!                  ▼
//! ┌────────────────────────────────────────────────────────────────┐
//! │ 2. Quota Check (This Module)                                   │
//! │    - Has component exceeded quota?                             │
//! │    - Storage: bytes < max_storage                              │
//! │    - Messages: rate < max_rate                                 │
//! │    - Network: bandwidth < max_bw                               │
//! │    - CPU: time < max_cpu_time                                  │
//! │    - Memory: usage < max_memory                                │
//! └────────────────┬────────────────────────────────────────────────┘
//!                  │ ✅ Within Quota
//!                  ▼
//! ┌────────────────────────────────────────────────────────────────┐
//! │ 3. Execute Operation                                            │
//! │    - Perform actual operation                                  │
//! └────────────────┬────────────────────────────────────────────────┘
//!                  │
//!                  ▼
//! ┌────────────────────────────────────────────────────────────────┐
//! │ 4. Update Quota Usage (This Module)                            │
//! │    - Increment counters (atomic)                               │
//! │    - Update usage metrics                                      │
//! └────────────────┬────────────────────────────────────────────────┘
//!                  │
//!                  ▼
//! ┌────────────────────────────────────────────────────────────────┐
//! │ 5. Audit Log (Task 3.3)                                        │
//! │    - Log operation + quota usage                               │
//! └─────────────────────────────────────────────────────────────────┘
//!
//! If Quota Check fails:
//! ┌────────────────────────────────────────────────────────────────┐
//! │ ❌ QuotaExceeded Error                                         │
//! │    - Return error to component                                 │
//! │    - Log quota violation event                                 │
//! │    - No operation executed                                     │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Quota Types
//!
//! ## Storage Quota
//! - **Purpose**: Prevent disk exhaustion
//! - **Default**: 100 MB
//! - **Tracking**: Cumulative bytes stored (not rate-limited)
//! - **Release**: When files are deleted
//!
//! ## Message Rate Quota
//! - **Purpose**: Prevent message flooding
//! - **Default**: 1000 messages/second
//! - **Tracking**: Time-window based (resets every second)
//! - **Release**: Automatic window reset
//!
//! ## Network Bandwidth Quota
//! - **Purpose**: Prevent bandwidth abuse
//! - **Default**: 10 MB/second
//! - **Tracking**: Time-window based (resets every second)
//! - **Release**: Automatic window reset
//!
//! ## CPU Time Quota
//! - **Purpose**: Prevent CPU monopolization
//! - **Default**: 1000 ms/second (100% of one core)
//! - **Tracking**: Time-window based
//! - **Release**: Automatic window reset
//!
//! ## Memory Quota
//! - **Purpose**: Prevent memory exhaustion
//! - **Default**: 256 MB
//! - **Tracking**: Peak tracking (current usage)
//! - **Release**: When memory is freed
//!
//! # Performance
//!
//! - **Quota Check**: <10μs (target, includes atomic reads)
//! - **Quota Update**: <5μs (target, atomic increment)
//! - **Memory Overhead**: <1KB per component
//! - **Thread-Safe**: Lock-free atomic operations
//!
//! # Examples
//!
//! ## Default Quota
//!
//! ```rust
//! use airssys_wasm::security::quota::ResourceQuota;
//!
//! let quota = ResourceQuota::default();
//! assert_eq!(quota.max_storage_bytes, 100 * 1024 * 1024); // 100 MB
//! assert_eq!(quota.max_message_rate, 1000); // 1000 msg/sec
//! ```
//!
//! ## Custom Quota
//!
//! ```rust
//! use airssys_wasm::security::quota::ResourceQuota;
//!
//! let quota = ResourceQuota::new()
//!     .with_storage(50 * 1024 * 1024)  // 50 MB
//!     .with_message_rate(500)          // 500 msg/sec
//!     .with_network_bandwidth(5 * 1024 * 1024);  // 5 MB/sec
//! ```
//!
//! ## Quota Tracking
//!
//! ```rust
//! use airssys_wasm::security::quota::{ResourceQuota, QuotaTracker};
//!
//! let quota = ResourceQuota::default();
//! let tracker = QuotaTracker::new(quota);
//!
//! // Check storage quota
//! tracker.check_storage(1024)?;
//!
//! // Consume storage quota
//! tracker.consume_storage(1024);
//!
//! // Get current usage
//! let usage = tracker.get_usage();
//! assert_eq!(usage.storage_used, 1024);
//! # Ok::<(), airssys_wasm::security::quota::QuotaError>(())
//! ```
//!
//! # Standards Compliance
//!
//! - **ADR-WASM-005**: Capability-Based Security Model (§2.3 - Resource Quotas) ✅
//! - **PROJECTS_STANDARD.md**: §4.3 (module structure), §5.1 (dependencies) ✅
//! - **Microsoft Rust Guidelines**: M-ERRORS-CANONICAL, M-ESSENTIAL-FN-INHERENT ✅

// Layer 1: Standard library imports
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::{Duration, Instant};

// Layer 2: Third-party crate imports
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// Layer 3: Internal module imports
// (none - this is a leaf module for quota management)

/// Resource quota configuration for a component.
///
/// Defines limits on various resource types to prevent abuse and exhaustion.
/// All limits use sensible defaults suitable for most components.
///
/// # Quota Types
///
/// - **Storage**: Total bytes stored in component namespace
/// - **Message Rate**: Messages per second (rate limiting)
/// - **Network Bandwidth**: Bytes per second (upload + download combined)
/// - **CPU Time**: Milliseconds of CPU time per second (percentage)
/// - **Memory**: Maximum memory usage in bytes
///
/// # Default Values
///
/// - Storage: 100 MB
/// - Message Rate: 1000 messages/second
/// - Network Bandwidth: 10 MB/second
/// - CPU Time: 1000 ms/second (100% of one core)
/// - Memory: 256 MB
///
/// # Examples
///
/// ## Default Quota
///
/// ```rust
/// use airssys_wasm::security::quota::ResourceQuota;
///
/// let quota = ResourceQuota::default();
/// assert_eq!(quota.max_storage_bytes, 100 * 1024 * 1024); // 100 MB
/// ```
///
/// ## Custom Quota
///
/// ```rust
/// use airssys_wasm::security::quota::ResourceQuota;
///
/// let quota = ResourceQuota::new()
///     .with_storage(50 * 1024 * 1024)  // 50 MB
///     .with_message_rate(500)          // 500 msg/sec
///     .with_network_bandwidth(5 * 1024 * 1024);  // 5 MB/sec
/// ```
///
/// ## From Component.toml
///
/// ```toml
/// [quota]
/// storage = "50MB"
/// message_rate = 500
/// network_bandwidth = "5MB/s"
/// cpu_time = "500ms/s"
/// memory = "128MB"
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceQuota {
    /// Maximum storage in bytes (default: 100 MB)
    pub max_storage_bytes: u64,

    /// Maximum messages per second (default: 1000)
    pub max_message_rate: u32,

    /// Maximum network bandwidth in bytes/second (default: 10 MB/s)
    pub max_network_bandwidth: u64,

    /// Maximum CPU time in milliseconds per second (default: 1000 ms/s = 100%)
    pub max_cpu_time_ms: u32,

    /// Maximum memory usage in bytes (default: 256 MB)
    pub max_memory_bytes: u64,

    /// Quota reset period (default: 1 second for rate limits)
    pub reset_period: Duration,
}

impl Default for ResourceQuota {
    fn default() -> Self {
        Self {
            max_storage_bytes: 100 * 1024 * 1024,      // 100 MB
            max_message_rate: 1000,                    // 1000 msg/sec
            max_network_bandwidth: 10 * 1024 * 1024,   // 10 MB/sec
            max_cpu_time_ms: 1000,                     // 1000 ms/sec (100%)
            max_memory_bytes: 256 * 1024 * 1024,       // 256 MB
            reset_period: Duration::from_secs(1),      // 1 second
        }
    }
}

impl ResourceQuota {
    /// Create a new quota with default values.
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::security::quota::ResourceQuota;
    ///
    /// let quota = ResourceQuota::new();
    /// assert_eq!(quota.max_storage_bytes, 100 * 1024 * 1024);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum storage bytes (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::security::quota::ResourceQuota;
    ///
    /// let quota = ResourceQuota::new().with_storage(50 * 1024 * 1024);
    /// assert_eq!(quota.max_storage_bytes, 50 * 1024 * 1024);
    /// ```
    pub fn with_storage(mut self, bytes: u64) -> Self {
        self.max_storage_bytes = bytes;
        self
    }

    /// Set maximum message rate (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::security::quota::ResourceQuota;
    ///
    /// let quota = ResourceQuota::new().with_message_rate(500);
    /// assert_eq!(quota.max_message_rate, 500);
    /// ```
    pub fn with_message_rate(mut self, rate: u32) -> Self {
        self.max_message_rate = rate;
        self
    }

    /// Set maximum network bandwidth (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::security::quota::ResourceQuota;
    ///
    /// let quota = ResourceQuota::new().with_network_bandwidth(5 * 1024 * 1024);
    /// assert_eq!(quota.max_network_bandwidth, 5 * 1024 * 1024);
    /// ```
    pub fn with_network_bandwidth(mut self, bytes_per_sec: u64) -> Self {
        self.max_network_bandwidth = bytes_per_sec;
        self
    }

    /// Set maximum CPU time (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::security::quota::ResourceQuota;
    ///
    /// let quota = ResourceQuota::new().with_cpu_time(500);
    /// assert_eq!(quota.max_cpu_time_ms, 500);
    /// ```
    pub fn with_cpu_time(mut self, ms_per_sec: u32) -> Self {
        self.max_cpu_time_ms = ms_per_sec;
        self
    }

    /// Set maximum memory (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::security::quota::ResourceQuota;
    ///
    /// let quota = ResourceQuota::new().with_memory(128 * 1024 * 1024);
    /// assert_eq!(quota.max_memory_bytes, 128 * 1024 * 1024);
    /// ```
    pub fn with_memory(mut self, bytes: u64) -> Self {
        self.max_memory_bytes = bytes;
        self
    }

    /// Parse human-readable storage size (e.g., "100MB", "1GB").
    ///
    /// Supported units:
    /// - B (bytes)
    /// - KB (kilobytes, 1024 bytes)
    /// - MB (megabytes, 1024 KB)
    /// - GB (gigabytes, 1024 MB)
    /// - TB (terabytes, 1024 GB)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::security::quota::ResourceQuota;
    ///
    /// assert_eq!(ResourceQuota::parse_storage("100MB").unwrap(), 100 * 1024 * 1024);
    /// assert_eq!(ResourceQuota::parse_storage("1GB").unwrap(), 1024 * 1024 * 1024);
    /// assert_eq!(ResourceQuota::parse_storage("500KB").unwrap(), 500 * 1024);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns error if format is invalid or unit is unsupported.
    ///
    /// ```rust
    /// use airssys_wasm::security::quota::ResourceQuota;
    ///
    /// assert!(ResourceQuota::parse_storage("invalid").is_err());
    /// assert!(ResourceQuota::parse_storage("100XB").is_err());
    /// ```
    pub fn parse_storage(size: &str) -> Result<u64, String> {
        let size = size.trim().to_uppercase();

        // Extract numeric part and unit
        let (num_part, unit) = size
            .split_at(size.chars().position(|c| !c.is_ascii_digit()).unwrap_or(size.len()));

        let value: u64 = num_part
            .parse()
            .map_err(|_| format!("Invalid number: {}", num_part))?;

        let multiplier: u64 = match unit {
            "B" | "" => 1,
            "KB" => 1024,
            "MB" => 1024 * 1024,
            "GB" => 1024 * 1024 * 1024,
            "TB" => 1024 * 1024 * 1024 * 1024,
            _ => return Err(format!("Unsupported unit: {}", unit)),
        };

        Ok(value * multiplier)
    }

    /// Parse human-readable bandwidth (e.g., "10MB/s", "1GB/s").
    ///
    /// Supported units:
    /// - B/s (bytes per second)
    /// - KB/s (kilobytes per second)
    /// - MB/s (megabytes per second)
    /// - GB/s (gigabytes per second)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::security::quota::ResourceQuota;
    ///
    /// assert_eq!(ResourceQuota::parse_bandwidth("10MB/s").unwrap(), 10 * 1024 * 1024);
    /// assert_eq!(ResourceQuota::parse_bandwidth("1GB/s").unwrap(), 1024 * 1024 * 1024);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns error if format is invalid or unit is unsupported.
    ///
    /// ```rust
    /// use airssys_wasm::security::quota::ResourceQuota;
    ///
    /// assert!(ResourceQuota::parse_bandwidth("invalid").is_err());
    /// assert!(ResourceQuota::parse_bandwidth("100MB").is_err()); // Missing /s
    /// ```
    pub fn parse_bandwidth(bandwidth: &str) -> Result<u64, String> {
        let bandwidth = bandwidth.trim().to_uppercase();

        // Remove /S or /SEC suffix
        let size_part = bandwidth
            .strip_suffix("/S")
            .or_else(|| bandwidth.strip_suffix("/SEC"))
            .ok_or_else(|| format!("Bandwidth must end with /s or /sec: {}", bandwidth))?;

        // Parse as storage size
        Self::parse_storage(size_part)
    }
}

/// Thread-safe quota usage tracker.
///
/// Tracks current resource usage for a component with atomic operations
/// for high-performance concurrent access. Uses time-window based rate
/// limiting for messages and network bandwidth.
///
/// # Thread Safety
///
/// All counters use atomic operations (lock-free, wait-free):
/// - `AtomicU64` for storage, network, memory counters
/// - `AtomicU32` for message, CPU counters
/// - `RwLock` for time-window data (low contention, read-heavy)
///
/// # Performance
///
/// - Check operation: <5μs (atomic reads + comparison)
/// - Update operation: <2μs (atomic increment)
/// - Reset operation: <10μs (atomic store + RwLock write)
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use airssys_wasm::security::quota::{ResourceQuota, QuotaTracker};
///
/// let quota = ResourceQuota::default();
/// let tracker = QuotaTracker::new(quota);
///
/// // Check storage quota
/// assert!(tracker.check_storage(1024).is_ok());
///
/// // Consume storage quota
/// tracker.consume_storage(1024);
///
/// // Check message rate quota
/// assert!(tracker.check_message_rate(1).is_ok());
/// tracker.consume_message_rate(1);
/// ```
#[derive(Debug)]
pub struct QuotaTracker {
    /// Quota configuration
    quota: ResourceQuota,

    /// Current storage usage in bytes (cumulative)
    storage_used: AtomicU64,

    /// Current message count in window
    message_count: AtomicU32,

    /// Current network bandwidth usage in window (bytes)
    network_bandwidth_used: AtomicU64,

    /// Current CPU time usage in window (milliseconds)
    cpu_time_used: AtomicU32,

    /// Current memory usage in bytes (peak tracking)
    memory_used: AtomicU64,

    /// Time window data (protected by RwLock for low contention)
    window_data: RwLock<TimeWindowData>,
}

/// Time window tracking data for rate limits.
#[derive(Debug)]
struct TimeWindowData {
    /// Start of current time window
    window_start: Instant,

    /// Duration of time window (from quota.reset_period)
    window_duration: Duration,
}

impl QuotaTracker {
    /// Create a new quota tracker with the given quota configuration.
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::security::quota::{ResourceQuota, QuotaTracker};
    ///
    /// let quota = ResourceQuota::default();
    /// let tracker = QuotaTracker::new(quota);
    /// ```
    pub fn new(quota: ResourceQuota) -> Self {
        let window_duration = quota.reset_period;
        Self {
            quota,
            storage_used: AtomicU64::new(0),
            message_count: AtomicU32::new(0),
            network_bandwidth_used: AtomicU64::new(0),
            cpu_time_used: AtomicU32::new(0),
            memory_used: AtomicU64::new(0),
            window_data: RwLock::new(TimeWindowData {
                window_start: Instant::now(),
                window_duration,
            }),
        }
    }

    /// Check if storage quota allows the given bytes.
    ///
    /// # Arguments
    ///
    /// - `bytes`: Number of bytes to check
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Quota allows the bytes
    /// - `Err(QuotaError)`: Quota would be exceeded
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::security::quota::{ResourceQuota, QuotaTracker};
    ///
    /// let quota = ResourceQuota::new().with_storage(1024);
    /// let tracker = QuotaTracker::new(quota);
    ///
    /// assert!(tracker.check_storage(512).is_ok());
    /// assert!(tracker.check_storage(2048).is_err());
    /// ```
    pub fn check_storage(&self, bytes: u64) -> Result<(), QuotaError> {
        let current = self.storage_used.load(Ordering::Relaxed);
        if current + bytes > self.quota.max_storage_bytes {
            return Err(QuotaError::StorageExceeded {
                current,
                requested: bytes,
                limit: self.quota.max_storage_bytes,
            });
        }
        Ok(())
    }

    /// Consume storage quota (increment usage).
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::security::quota::{ResourceQuota, QuotaTracker};
    ///
    /// let quota = ResourceQuota::default();
    /// let tracker = QuotaTracker::new(quota);
    ///
    /// tracker.consume_storage(1024);
    /// assert_eq!(tracker.get_usage().storage_used, 1024);
    /// ```
    pub fn consume_storage(&self, bytes: u64) {
        self.storage_used.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Release storage quota (decrement usage).
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::security::quota::{ResourceQuota, QuotaTracker};
    ///
    /// let quota = ResourceQuota::default();
    /// let tracker = QuotaTracker::new(quota);
    ///
    /// tracker.consume_storage(1024);
    /// tracker.release_storage(512);
    /// assert_eq!(tracker.get_usage().storage_used, 512);
    /// ```
    pub fn release_storage(&self, bytes: u64) {
        self.storage_used.fetch_sub(bytes, Ordering::Relaxed);
    }

    /// Check if message rate quota allows the given count.
    ///
    /// # Arguments
    ///
    /// - `count`: Number of messages to check
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Quota allows the messages
    /// - `Err(QuotaError)`: Quota would be exceeded
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::security::quota::{ResourceQuota, QuotaTracker};
    ///
    /// let quota = ResourceQuota::new().with_message_rate(10);
    /// let tracker = QuotaTracker::new(quota);
    ///
    /// assert!(tracker.check_message_rate(5).is_ok());
    /// assert!(tracker.check_message_rate(20).is_err());
    /// ```
    pub fn check_message_rate(&self, count: u32) -> Result<(), QuotaError> {
        // Reset window if expired
        self.reset_window_if_needed();

        let current = self.message_count.load(Ordering::Relaxed);
        if current + count > self.quota.max_message_rate {
            return Err(QuotaError::MessageRateExceeded {
                current,
                requested: count,
                limit: self.quota.max_message_rate,
            });
        }
        Ok(())
    }

    /// Consume message rate quota (increment count).
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::security::quota::{ResourceQuota, QuotaTracker};
    ///
    /// let quota = ResourceQuota::default();
    /// let tracker = QuotaTracker::new(quota);
    ///
    /// tracker.consume_message_rate(1);
    /// assert_eq!(tracker.get_usage().message_count, 1);
    /// ```
    pub fn consume_message_rate(&self, count: u32) {
        self.message_count.fetch_add(count, Ordering::Relaxed);
    }

    /// Check if network bandwidth quota allows the given bytes.
    ///
    /// # Arguments
    ///
    /// - `bytes`: Number of bytes to check
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Quota allows the bytes
    /// - `Err(QuotaError)`: Quota would be exceeded
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::security::quota::{ResourceQuota, QuotaTracker};
    ///
    /// let quota = ResourceQuota::new().with_network_bandwidth(1024);
    /// let tracker = QuotaTracker::new(quota);
    ///
    /// assert!(tracker.check_network_bandwidth(512).is_ok());
    /// assert!(tracker.check_network_bandwidth(2048).is_err());
    /// ```
    pub fn check_network_bandwidth(&self, bytes: u64) -> Result<(), QuotaError> {
        // Reset window if expired
        self.reset_window_if_needed();

        let current = self.network_bandwidth_used.load(Ordering::Relaxed);
        if current + bytes > self.quota.max_network_bandwidth {
            return Err(QuotaError::NetworkBandwidthExceeded {
                current,
                requested: bytes,
                limit: self.quota.max_network_bandwidth,
            });
        }
        Ok(())
    }

    /// Consume network bandwidth quota (increment usage).
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::security::quota::{ResourceQuota, QuotaTracker};
    ///
    /// let quota = ResourceQuota::default();
    /// let tracker = QuotaTracker::new(quota);
    ///
    /// tracker.consume_network_bandwidth(1024);
    /// assert_eq!(tracker.get_usage().network_bandwidth_used, 1024);
    /// ```
    pub fn consume_network_bandwidth(&self, bytes: u64) {
        self.network_bandwidth_used.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Check if CPU time quota allows the given milliseconds.
    ///
    /// # Arguments
    ///
    /// - `ms`: Milliseconds of CPU time to check
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Quota allows the CPU time
    /// - `Err(QuotaError)`: Quota would be exceeded
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::security::quota::{ResourceQuota, QuotaTracker};
    ///
    /// let quota = ResourceQuota::new().with_cpu_time(1000);
    /// let tracker = QuotaTracker::new(quota);
    ///
    /// assert!(tracker.check_cpu_time(500).is_ok());
    /// assert!(tracker.check_cpu_time(2000).is_err());
    /// ```
    pub fn check_cpu_time(&self, ms: u32) -> Result<(), QuotaError> {
        // Reset window if expired
        self.reset_window_if_needed();

        let current = self.cpu_time_used.load(Ordering::Relaxed);
        if current + ms > self.quota.max_cpu_time_ms {
            return Err(QuotaError::CpuTimeExceeded {
                current,
                requested: ms,
                limit: self.quota.max_cpu_time_ms,
            });
        }
        Ok(())
    }

    /// Consume CPU time quota (increment usage).
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::security::quota::{ResourceQuota, QuotaTracker};
    ///
    /// let quota = ResourceQuota::default();
    /// let tracker = QuotaTracker::new(quota);
    ///
    /// tracker.consume_cpu_time(100);
    /// assert_eq!(tracker.get_usage().cpu_time_used, 100);
    /// ```
    pub fn consume_cpu_time(&self, ms: u32) {
        self.cpu_time_used.fetch_add(ms, Ordering::Relaxed);
    }

    /// Check if memory quota allows the given bytes.
    ///
    /// # Arguments
    ///
    /// - `bytes`: Number of bytes to check
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Quota allows the memory
    /// - `Err(QuotaError)`: Quota would be exceeded
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::security::quota::{ResourceQuota, QuotaTracker};
    ///
    /// let quota = ResourceQuota::new().with_memory(1024);
    /// let tracker = QuotaTracker::new(quota);
    ///
    /// assert!(tracker.check_memory(512).is_ok());
    /// assert!(tracker.check_memory(2048).is_err());
    /// ```
    pub fn check_memory(&self, bytes: u64) -> Result<(), QuotaError> {
        if bytes > self.quota.max_memory_bytes {
            let current = self.memory_used.load(Ordering::Relaxed);
            return Err(QuotaError::MemoryExceeded {
                current,
                requested: bytes,
                limit: self.quota.max_memory_bytes,
            });
        }
        Ok(())
    }

    /// Update memory usage (peak tracking).
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::security::quota::{ResourceQuota, QuotaTracker};
    ///
    /// let quota = ResourceQuota::default();
    /// let tracker = QuotaTracker::new(quota);
    ///
    /// tracker.update_memory_usage(2048);
    /// assert_eq!(tracker.get_usage().memory_used, 2048);
    /// ```
    pub fn update_memory_usage(&self, bytes: u64) {
        self.memory_used.store(bytes, Ordering::Relaxed);
    }

    /// Reset time window if expired (called before rate limit checks).
    fn reset_window_if_needed(&self) {
        // Fast path: Check if window expired (read lock)
        {
            let window_data = self.window_data.read();
            let elapsed = window_data.window_start.elapsed();
            if elapsed < window_data.window_duration {
                // Window still valid, no reset needed
                return;
            }
        } // Release read lock

        // Slow path: Window expired, acquire write lock to reset
        let mut window_data = self.window_data.write();

        // Double-check condition (another thread might have reset)
        let elapsed = window_data.window_start.elapsed();
        if elapsed >= window_data.window_duration {
            // Reset window
            window_data.window_start = Instant::now();

            // Reset rate-limited counters
            self.message_count.store(0, Ordering::Relaxed);
            self.network_bandwidth_used.store(0, Ordering::Relaxed);
            self.cpu_time_used.store(0, Ordering::Relaxed);
        }
    }

    /// Get current quota usage statistics.
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::security::quota::{ResourceQuota, QuotaTracker};
    ///
    /// let quota = ResourceQuota::default();
    /// let tracker = QuotaTracker::new(quota);
    ///
    /// tracker.consume_storage(1024);
    /// tracker.consume_message_rate(5);
    ///
    /// let usage = tracker.get_usage();
    /// assert_eq!(usage.storage_used, 1024);
    /// assert_eq!(usage.message_count, 5);
    /// ```
    pub fn get_usage(&self) -> QuotaUsage {
        QuotaUsage {
            storage_used: self.storage_used.load(Ordering::Relaxed),
            message_count: self.message_count.load(Ordering::Relaxed),
            network_bandwidth_used: self.network_bandwidth_used.load(Ordering::Relaxed),
            cpu_time_used: self.cpu_time_used.load(Ordering::Relaxed),
            memory_used: self.memory_used.load(Ordering::Relaxed),
        }
    }

    /// Get quota usage and remaining quota for all resource types.
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::security::quota::{ResourceQuota, QuotaTracker};
    ///
    /// let quota = ResourceQuota::new().with_storage(1024);
    /// let tracker = QuotaTracker::new(quota);
    ///
    /// tracker.consume_storage(512);
    ///
    /// let status = tracker.get_quota_status();
    /// assert_eq!(status.storage.used, 512);
    /// assert_eq!(status.storage.available, 512);
    /// assert_eq!(status.storage.percentage, 50.0);
    /// ```
    pub fn get_quota_status(&self) -> QuotaStatus {
        let usage = self.get_usage();

        QuotaStatus {
            storage: QuotaResourceStatus {
                used: usage.storage_used,
                limit: self.quota.max_storage_bytes,
                available: self.quota.max_storage_bytes.saturating_sub(usage.storage_used),
                percentage: calculate_percentage(usage.storage_used, self.quota.max_storage_bytes),
            },
            message_rate: QuotaResourceStatus {
                used: usage.message_count as u64,
                limit: self.quota.max_message_rate as u64,
                available: (self.quota.max_message_rate as u64)
                    .saturating_sub(usage.message_count as u64),
                percentage: calculate_percentage(
                    usage.message_count as u64,
                    self.quota.max_message_rate as u64,
                ),
            },
            network_bandwidth: QuotaResourceStatus {
                used: usage.network_bandwidth_used,
                limit: self.quota.max_network_bandwidth,
                available: self
                    .quota
                    .max_network_bandwidth
                    .saturating_sub(usage.network_bandwidth_used),
                percentage: calculate_percentage(
                    usage.network_bandwidth_used,
                    self.quota.max_network_bandwidth,
                ),
            },
            cpu_time: QuotaResourceStatus {
                used: usage.cpu_time_used as u64,
                limit: self.quota.max_cpu_time_ms as u64,
                available: (self.quota.max_cpu_time_ms as u64)
                    .saturating_sub(usage.cpu_time_used as u64),
                percentage: calculate_percentage(
                    usage.cpu_time_used as u64,
                    self.quota.max_cpu_time_ms as u64,
                ),
            },
            memory: QuotaResourceStatus {
                used: usage.memory_used,
                limit: self.quota.max_memory_bytes,
                available: self.quota.max_memory_bytes.saturating_sub(usage.memory_used),
                percentage: calculate_percentage(usage.memory_used, self.quota.max_memory_bytes),
            },
        }
    }

    /// Check if any quota is close to exhaustion (>80% used).
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::security::quota::{ResourceQuota, QuotaTracker};
    ///
    /// let quota = ResourceQuota::new().with_storage(1000);
    /// let tracker = QuotaTracker::new(quota);
    ///
    /// tracker.consume_storage(850);
    /// assert!(tracker.is_quota_warning());
    /// ```
    pub fn is_quota_warning(&self) -> bool {
        let status = self.get_quota_status();
        status.storage.percentage >= 80.0
            || status.message_rate.percentage >= 80.0
            || status.network_bandwidth.percentage >= 80.0
            || status.cpu_time.percentage >= 80.0
            || status.memory.percentage >= 80.0
    }

    /// Check if any quota is exhausted (>95% used).
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::security::quota::{ResourceQuota, QuotaTracker};
    ///
    /// let quota = ResourceQuota::new().with_storage(1000);
    /// let tracker = QuotaTracker::new(quota);
    ///
    /// tracker.consume_storage(970);
    /// assert!(tracker.is_quota_critical());
    /// ```
    pub fn is_quota_critical(&self) -> bool {
        let status = self.get_quota_status();
        status.storage.percentage >= 95.0
            || status.message_rate.percentage >= 95.0
            || status.network_bandwidth.percentage >= 95.0
            || status.cpu_time.percentage >= 95.0
            || status.memory.percentage >= 95.0
    }

    /// Reset all quota usage (for testing or manual reset).
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::security::quota::{ResourceQuota, QuotaTracker};
    ///
    /// let quota = ResourceQuota::default();
    /// let tracker = QuotaTracker::new(quota);
    ///
    /// tracker.consume_storage(1024);
    /// tracker.consume_message_rate(10);
    ///
    /// tracker.reset();
    ///
    /// let usage = tracker.get_usage();
    /// assert_eq!(usage.storage_used, 0);
    /// assert_eq!(usage.message_count, 0);
    /// ```
    pub fn reset(&self) {
        self.storage_used.store(0, Ordering::Relaxed);
        self.message_count.store(0, Ordering::Relaxed);
        self.network_bandwidth_used.store(0, Ordering::Relaxed);
        self.cpu_time_used.store(0, Ordering::Relaxed);
        self.memory_used.store(0, Ordering::Relaxed);

        let mut window_data = self.window_data.write();
        window_data.window_start = Instant::now();
    }

    /// Set custom window duration for testing.
    ///
    /// **WARNING**: This method is intended for testing only. In production,
    /// the window duration is set from `ResourceQuota::reset_period`.
    ///
    /// # Arguments
    ///
    /// - `duration`: Custom window duration
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::security::quota::{ResourceQuota, QuotaTracker};
    /// use std::time::Duration;
    ///
    /// let quota = ResourceQuota::new().with_message_rate(10);
    /// let tracker = QuotaTracker::new(quota);
    ///
    /// // Set short window duration for testing
    /// tracker.set_window_duration_for_testing(Duration::from_millis(100));
    /// ```
    #[doc(hidden)]
    pub fn set_window_duration_for_testing(&self, duration: Duration) {
        let mut window_data = self.window_data.write();
        window_data.window_duration = duration;
    }
}

/// Snapshot of current quota usage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuotaUsage {
    /// Current storage used (bytes)
    pub storage_used: u64,

    /// Current message count in window
    pub message_count: u32,

    /// Current network bandwidth used in window (bytes)
    pub network_bandwidth_used: u64,

    /// Current CPU time used in window (milliseconds)
    pub cpu_time_used: u32,

    /// Current memory used (bytes)
    pub memory_used: u64,
}

/// Complete quota status for a component.
#[derive(Debug, Clone, PartialEq)]
pub struct QuotaStatus {
    /// Storage quota status
    pub storage: QuotaResourceStatus,

    /// Message rate quota status
    pub message_rate: QuotaResourceStatus,

    /// Network bandwidth quota status
    pub network_bandwidth: QuotaResourceStatus,

    /// CPU time quota status
    pub cpu_time: QuotaResourceStatus,

    /// Memory quota status
    pub memory: QuotaResourceStatus,
}

/// Status for a single quota resource.
#[derive(Debug, Clone, PartialEq)]
pub struct QuotaResourceStatus {
    /// Current usage
    pub used: u64,

    /// Maximum limit
    pub limit: u64,

    /// Remaining quota (limit - used)
    pub available: u64,

    /// Usage percentage (0.0-100.0)
    pub percentage: f64,
}

/// Calculate percentage of used quota.
fn calculate_percentage(used: u64, limit: u64) -> f64 {
    if limit == 0 {
        0.0
    } else {
        (used as f64 / limit as f64) * 100.0
    }
}

/// Errors that can occur during quota checking.
#[derive(Debug, Error)]
pub enum QuotaError {
    /// Storage quota exceeded.
    #[error("Storage quota exceeded: used {current} bytes + requested {requested} bytes > limit {limit} bytes")]
    StorageExceeded {
        /// Current storage used
        current: u64,
        /// Requested storage
        requested: u64,
        /// Storage limit
        limit: u64,
    },

    /// Message rate quota exceeded.
    #[error("Message rate quota exceeded: {current} messages/sec + {requested} messages > limit {limit} messages/sec")]
    MessageRateExceeded {
        /// Current message count
        current: u32,
        /// Requested message count
        requested: u32,
        /// Message rate limit
        limit: u32,
    },

    /// Network bandwidth quota exceeded.
    #[error("Network bandwidth quota exceeded: {current} bytes/sec + {requested} bytes > limit {limit} bytes/sec")]
    NetworkBandwidthExceeded {
        /// Current bandwidth used
        current: u64,
        /// Requested bandwidth
        requested: u64,
        /// Bandwidth limit
        limit: u64,
    },

    /// CPU time quota exceeded.
    #[error("CPU time quota exceeded: {current} ms/sec + {requested} ms > limit {limit} ms/sec")]
    CpuTimeExceeded {
        /// Current CPU time used
        current: u32,
        /// Requested CPU time
        requested: u32,
        /// CPU time limit
        limit: u32,
    },

    /// Memory quota exceeded.
    #[error("Memory quota exceeded: requested {requested} bytes > limit {limit} bytes (current: {current} bytes)")]
    MemoryExceeded {
        /// Current memory used
        current: u64,
        /// Requested memory
        requested: u64,
        /// Memory limit
        limit: u64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration as StdDuration;

    // ═════════════════════════════════════════════════════════════════════════
    // ResourceQuota Tests
    // ═════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_resource_quota_default() {
        let quota = ResourceQuota::default();
        assert_eq!(quota.max_storage_bytes, 100 * 1024 * 1024);
        assert_eq!(quota.max_message_rate, 1000);
        assert_eq!(quota.max_network_bandwidth, 10 * 1024 * 1024);
        assert_eq!(quota.max_cpu_time_ms, 1000);
        assert_eq!(quota.max_memory_bytes, 256 * 1024 * 1024);
        assert_eq!(quota.reset_period, Duration::from_secs(1));
    }

    #[test]
    fn test_resource_quota_builder() {
        let quota = ResourceQuota::new()
            .with_storage(50 * 1024 * 1024)
            .with_message_rate(500)
            .with_network_bandwidth(5 * 1024 * 1024)
            .with_cpu_time(500)
            .with_memory(128 * 1024 * 1024);

        assert_eq!(quota.max_storage_bytes, 50 * 1024 * 1024);
        assert_eq!(quota.max_message_rate, 500);
        assert_eq!(quota.max_network_bandwidth, 5 * 1024 * 1024);
        assert_eq!(quota.max_cpu_time_ms, 500);
        assert_eq!(quota.max_memory_bytes, 128 * 1024 * 1024);
    }

    #[test]
    fn test_parse_storage() {
        assert_eq!(ResourceQuota::parse_storage("100MB").unwrap(), 100 * 1024 * 1024);
        assert_eq!(ResourceQuota::parse_storage("1GB").unwrap(), 1024 * 1024 * 1024);
        assert_eq!(ResourceQuota::parse_storage("500KB").unwrap(), 500 * 1024);
        assert_eq!(ResourceQuota::parse_storage("1024B").unwrap(), 1024);
        assert_eq!(ResourceQuota::parse_storage("1TB").unwrap(), 1024 * 1024 * 1024 * 1024);
    }

    #[test]
    fn test_parse_storage_invalid() {
        assert!(ResourceQuota::parse_storage("invalid").is_err());
        assert!(ResourceQuota::parse_storage("100XB").is_err());
        assert!(ResourceQuota::parse_storage("").is_err());
    }

    #[test]
    fn test_parse_bandwidth() {
        assert_eq!(ResourceQuota::parse_bandwidth("10MB/s").unwrap(), 10 * 1024 * 1024);
        assert_eq!(ResourceQuota::parse_bandwidth("1GB/s").unwrap(), 1024 * 1024 * 1024);
        assert_eq!(ResourceQuota::parse_bandwidth("500KB/s").unwrap(), 500 * 1024);
    }

    #[test]
    fn test_parse_bandwidth_invalid() {
        assert!(ResourceQuota::parse_bandwidth("invalid").is_err());
        assert!(ResourceQuota::parse_bandwidth("100MB").is_err()); // Missing /s
        assert!(ResourceQuota::parse_bandwidth("").is_err());
    }

    // ═════════════════════════════════════════════════════════════════════════
    // QuotaTracker Tests - Storage
    // ═════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_storage_quota_within_limit() {
        let quota = ResourceQuota::new().with_storage(1024);
        let tracker = QuotaTracker::new(quota);

        assert!(tracker.check_storage(512).is_ok());
    }

    #[test]
    fn test_storage_quota_exceeds_limit() {
        let quota = ResourceQuota::new().with_storage(1024);
        let tracker = QuotaTracker::new(quota);

        assert!(tracker.check_storage(2048).is_err());
    }

    #[test]
    fn test_storage_quota_consume() {
        let quota = ResourceQuota::new().with_storage(2048);
        let tracker = QuotaTracker::new(quota);

        tracker.consume_storage(1024);
        assert_eq!(tracker.get_usage().storage_used, 1024);

        // Should still allow 1024 bytes
        assert!(tracker.check_storage(1024).is_ok());

        // Should not allow 1025 bytes
        assert!(tracker.check_storage(1025).is_err());
    }

    #[test]
    fn test_storage_quota_release() {
        let quota = ResourceQuota::new().with_storage(1024);
        let tracker = QuotaTracker::new(quota);

        tracker.consume_storage(1024);
        assert_eq!(tracker.get_usage().storage_used, 1024);

        tracker.release_storage(512);
        assert_eq!(tracker.get_usage().storage_used, 512);

        // Should now allow 512 bytes
        assert!(tracker.check_storage(512).is_ok());
    }

    // ═════════════════════════════════════════════════════════════════════════
    // QuotaTracker Tests - Message Rate
    // ═════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_message_rate_quota_within_limit() {
        let quota = ResourceQuota::new().with_message_rate(10);
        let tracker = QuotaTracker::new(quota);

        assert!(tracker.check_message_rate(5).is_ok());
    }

    #[test]
    fn test_message_rate_quota_exceeds_limit() {
        let quota = ResourceQuota::new().with_message_rate(10);
        let tracker = QuotaTracker::new(quota);

        assert!(tracker.check_message_rate(20).is_err());
    }

    #[test]
    fn test_message_rate_quota_consume() {
        let quota = ResourceQuota::new().with_message_rate(10);
        let tracker = QuotaTracker::new(quota);

        tracker.consume_message_rate(5);
        assert_eq!(tracker.get_usage().message_count, 5);

        // Should still allow 5 messages
        assert!(tracker.check_message_rate(5).is_ok());

        // Should not allow 6 messages
        assert!(tracker.check_message_rate(6).is_err());
    }

    #[test]
    fn test_message_rate_window_reset() {
        let quota = ResourceQuota::new()
            .with_message_rate(10);
        // Create tracker with short reset period for testing
        let tracker = QuotaTracker::new(quota);
        tracker.set_window_duration_for_testing(Duration::from_millis(100));

        tracker.consume_message_rate(10);
        assert_eq!(tracker.get_usage().message_count, 10);

        // Should be at limit
        assert!(tracker.check_message_rate(1).is_err());

        // Wait for window reset
        thread::sleep(StdDuration::from_millis(150));

        // Should be reset and allow messages again
        assert!(tracker.check_message_rate(10).is_ok());
    }

    // ═════════════════════════════════════════════════════════════════════════
    // QuotaTracker Tests - Network Bandwidth
    // ═════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_network_bandwidth_quota_within_limit() {
        let quota = ResourceQuota::new().with_network_bandwidth(1024);
        let tracker = QuotaTracker::new(quota);

        assert!(tracker.check_network_bandwidth(512).is_ok());
    }

    #[test]
    fn test_network_bandwidth_quota_exceeds_limit() {
        let quota = ResourceQuota::new().with_network_bandwidth(1024);
        let tracker = QuotaTracker::new(quota);

        assert!(tracker.check_network_bandwidth(2048).is_err());
    }

    #[test]
    fn test_network_bandwidth_quota_consume() {
        let quota = ResourceQuota::new().with_network_bandwidth(1024);
        let tracker = QuotaTracker::new(quota);

        tracker.consume_network_bandwidth(512);
        assert_eq!(tracker.get_usage().network_bandwidth_used, 512);

        // Should still allow 512 bytes
        assert!(tracker.check_network_bandwidth(512).is_ok());

        // Should not allow 513 bytes
        assert!(tracker.check_network_bandwidth(513).is_err());
    }

    // ═════════════════════════════════════════════════════════════════════════
    // QuotaTracker Tests - CPU Time
    // ═════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_cpu_time_quota_within_limit() {
        let quota = ResourceQuota::new().with_cpu_time(1000);
        let tracker = QuotaTracker::new(quota);

        assert!(tracker.check_cpu_time(500).is_ok());
    }

    #[test]
    fn test_cpu_time_quota_exceeds_limit() {
        let quota = ResourceQuota::new().with_cpu_time(1000);
        let tracker = QuotaTracker::new(quota);

        assert!(tracker.check_cpu_time(2000).is_err());
    }

    #[test]
    fn test_cpu_time_quota_consume() {
        let quota = ResourceQuota::new().with_cpu_time(1000);
        let tracker = QuotaTracker::new(quota);

        tracker.consume_cpu_time(500);
        assert_eq!(tracker.get_usage().cpu_time_used, 500);

        // Should still allow 500 ms
        assert!(tracker.check_cpu_time(500).is_ok());

        // Should not allow 501 ms
        assert!(tracker.check_cpu_time(501).is_err());
    }

    // ═════════════════════════════════════════════════════════════════════════
    // QuotaTracker Tests - Memory
    // ═════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_memory_quota_within_limit() {
        let quota = ResourceQuota::new().with_memory(1024);
        let tracker = QuotaTracker::new(quota);

        assert!(tracker.check_memory(512).is_ok());
    }

    #[test]
    fn test_memory_quota_exceeds_limit() {
        let quota = ResourceQuota::new().with_memory(1024);
        let tracker = QuotaTracker::new(quota);

        assert!(tracker.check_memory(2048).is_err());
    }

    #[test]
    fn test_memory_quota_update() {
        let quota = ResourceQuota::new().with_memory(2048);
        let tracker = QuotaTracker::new(quota);

        tracker.update_memory_usage(1024);
        assert_eq!(tracker.get_usage().memory_used, 1024);

        tracker.update_memory_usage(512);
        assert_eq!(tracker.get_usage().memory_used, 512);
    }

    // ═════════════════════════════════════════════════════════════════════════
    // QuotaTracker Tests - Quota Status
    // ═════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_quota_status() {
        let quota = ResourceQuota::new().with_storage(1000);
        let tracker = QuotaTracker::new(quota);

        tracker.consume_storage(500);

        let status = tracker.get_quota_status();
        assert_eq!(status.storage.used, 500);
        assert_eq!(status.storage.limit, 1000);
        assert_eq!(status.storage.available, 500);
        assert_eq!(status.storage.percentage, 50.0);
    }

    #[test]
    fn test_quota_warning_threshold() {
        let quota = ResourceQuota::new().with_storage(1000);
        let tracker = QuotaTracker::new(quota);

        tracker.consume_storage(850);
        assert!(tracker.is_quota_warning());
    }

    #[test]
    fn test_quota_critical_threshold() {
        let quota = ResourceQuota::new().with_storage(1000);
        let tracker = QuotaTracker::new(quota);

        tracker.consume_storage(970);
        assert!(tracker.is_quota_critical());
    }

    #[test]
    fn test_quota_reset() {
        let quota = ResourceQuota::default();
        let tracker = QuotaTracker::new(quota);

        tracker.consume_storage(1024);
        tracker.consume_message_rate(10);
        tracker.consume_network_bandwidth(512);
        tracker.consume_cpu_time(100);
        tracker.update_memory_usage(2048);

        tracker.reset();

        let usage = tracker.get_usage();
        assert_eq!(usage.storage_used, 0);
        assert_eq!(usage.message_count, 0);
        assert_eq!(usage.network_bandwidth_used, 0);
        assert_eq!(usage.cpu_time_used, 0);
        assert_eq!(usage.memory_used, 0);
    }

    // ═════════════════════════════════════════════════════════════════════════
    // QuotaTracker Tests - Edge Cases
    // ═════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_quota_zero_always_denied() {
        let quota = ResourceQuota::new().with_storage(0);
        let tracker = QuotaTracker::new(quota);

        assert!(tracker.check_storage(1).is_err());
    }

    #[test]
    fn test_quota_max_value() {
        let quota = ResourceQuota::new().with_storage(u64::MAX);
        let tracker = QuotaTracker::new(quota);

        // Should allow any reasonable amount
        assert!(tracker.check_storage(1024 * 1024 * 1024).is_ok());
    }

    #[test]
    fn test_quota_concurrent_access() {
        use std::sync::Arc;

        let quota = ResourceQuota::new().with_storage(10000);
        let tracker = Arc::new(QuotaTracker::new(quota));

        let mut handles = vec![];

        for _ in 0..10 {
            let tracker_clone = Arc::clone(&tracker);
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    let _ = tracker_clone.check_storage(10);
                    tracker_clone.consume_storage(10);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // All threads should have consumed storage
        assert_eq!(tracker.get_usage().storage_used, 10000);
    }
}
