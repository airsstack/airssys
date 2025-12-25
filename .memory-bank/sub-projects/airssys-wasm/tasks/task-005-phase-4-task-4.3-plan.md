# Implementation Plan: WASM-TASK-005 Phase 4, Task 4.3 - Resource Quota System

**Status:** Ready for Implementation  
**Created:** 2025-12-19  
**Task:** Task 4.3 - Resource Quota System  
**Phase:** Phase 4 - ComponentActor Security Integration  
**Estimated Effort:** 2 days  
**Priority:** 🔒 CRITICAL PATH - Security Layer

---

## Executive Summary

This plan details the implementation of a **per-component resource quota system** that prevents resource exhaustion attacks by enforcing limits on storage, message rate, network bandwidth, CPU time, and memory usage. The system integrates with the existing capability enforcement layer (Task 3.1) and ComponentActor security context (Task 4.1), providing defense-in-depth against malicious or buggy components.

### Key Deliverables

1. **ResourceQuota struct** - Quota configuration (storage, message rate, network, CPU, memory)
2. **Quota tracking system** - Per-ComponentActor quota monitoring (thread-safe, atomic)
3. **Quota enforcement** - Integration with capability check API
4. **Configuration system** - Default + per-component quota overrides
5. **Monitoring API** - Current usage, remaining quota, violation events
6. **Comprehensive tests** - 15+ tests covering enforcement, violations, edge cases

### Success Criteria

- ✅ Components respect quota limits (storage, message rate, network, CPU, memory)
- ✅ Quota violations return clear errors (no silent failures)
- ✅ Quota configuration loaded from Component.toml (per-component overrides)
- ✅ Quota monitoring API functional (usage metrics, remaining quota)
- ✅ 15+ test cases covering quota scenarios (enforcement, violations, resets, edge cases)
- ✅ Performance: <10μs overhead per quota check, <5μs per update
- ✅ Zero compiler/clippy warnings
- ✅ 95%+ code review quality score

---

## Architecture Overview

### System Integration

```text
┌────────────────────────────────────────────────────────────────┐
│ ComponentActor                                                  │
│ ┌────────────────────────────────────────────────────────────┐ │
│ │ WasmSecurityContext (Task 4.1)                             │ │
│ │ - component_id: String                                     │ │
│ │ - capabilities: WasmCapabilitySet                          │ │
│ │ - resource_quota: ResourceQuota ← NEW (Task 4.3)           │ │
│ │ - quota_tracker: Arc<QuotaTracker> ← NEW (Task 4.3)       │ │
│ └────────────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────────┘
                          │
                          │ Host Function Call
                          ▼
┌────────────────────────────────────────────────────────────────┐
│ check_capability_with_quota() ← NEW API (Task 4.3)             │
│ 1. Check capability (Task 3.1)                                 │
│ 2. Check quota (Task 4.3) ← NEW                                │
│ 3. Perform operation                                           │
│ 4. Update quota usage (Task 4.3) ← NEW                         │
│ 5. Audit log                                                   │
└────────────────────────────────────────────────────────────────┘
                          │
                          │ Decision: Allow/Deny
                          ▼
┌────────────────────────────────────────────────────────────────┐
│ Host Function Implementation (Block 8)                          │
│ - Filesystem operations (quota: storage bytes)                 │
│ - Network operations (quota: bandwidth bytes/sec)              │
│ - Message operations (quota: messages/sec)                     │
│ - CPU operations (quota: CPU time ms/sec)                      │
└────────────────────────────────────────────────────────────────┘
```

### Quota Enforcement Flow

```text
┌─────────────┐
│ Component   │
│ calls host  │
│ function    │
└──────┬──────┘
       │
       ▼
┌──────────────────────────────────────┐
│ 1. Capability Check (Task 3.1)       │ ← Existing
│    - Does component have permission? │
└──────┬───────────────────────────────┘
       │ ✅ Allowed
       ▼
┌──────────────────────────────────────┐
│ 2. Quota Check (Task 4.3)            │ ← NEW
│    - Has component exceeded quota?   │
│    - Storage: bytes < max_storage    │
│    - Messages: rate < max_rate       │
│    - Network: bandwidth < max_bw     │
│    - CPU: time < max_cpu_time        │
│    - Memory: usage < max_memory      │
└──────┬───────────────────────────────┘
       │ ✅ Within Quota
       ▼
┌──────────────────────────────────────┐
│ 3. Execute Operation                 │
│    - Perform actual operation        │
└──────┬───────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────┐
│ 4. Update Quota Usage (Task 4.3)     │ ← NEW
│    - Increment counters (atomic)     │
│    - Update usage metrics            │
└──────┬───────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────┐
│ 5. Audit Log (Task 3.3)              │ ← Existing
│    - Log operation + quota usage     │
└──────────────────────────────────────┘

If Quota Check fails:
┌──────────────────────────────────────┐
│ ❌ QuotaExceeded Error               │
│    - Return error to component       │
│    - Log quota violation event       │
│    - No operation executed           │
└──────────────────────────────────────┘
```

---

## Detailed Implementation Plan

### Phase 1: Core ResourceQuota Struct and Tracking (Day 1, 4-5 hours)

#### 1.1 ResourceQuota Struct Design

**File:** `airssys-wasm/src/security/quota.rs`

**Objective:** Define the ResourceQuota configuration struct with all quota types.

**Implementation:**

```rust
//! Resource Quota Management for WASM Components
//!
//! This module provides resource quota enforcement to prevent component
//! resource exhaustion attacks.

// Layer 1: Standard library imports
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicU32, Ordering};
use std::time::{Duration, Instant};

// Layer 2: Third-party crate imports
use serde::{Deserialize, Serialize};
use parking_lot::RwLock;

// Layer 3: Internal module imports
use crate::security::CapabilityCheckError;

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
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum storage bytes (builder pattern).
    pub fn with_storage(mut self, bytes: u64) -> Self {
        self.max_storage_bytes = bytes;
        self
    }

    /// Set maximum message rate (builder pattern).
    pub fn with_message_rate(mut self, rate: u32) -> Self {
        self.max_message_rate = rate;
        self
    }

    /// Set maximum network bandwidth (builder pattern).
    pub fn with_network_bandwidth(mut self, bytes_per_sec: u64) -> Self {
        self.max_network_bandwidth = bytes_per_sec;
        self
    }

    /// Set maximum CPU time (builder pattern).
    pub fn with_cpu_time(mut self, ms_per_sec: u32) -> Self {
        self.max_cpu_time_ms = ms_per_sec;
        self
    }

    /// Set maximum memory (builder pattern).
    pub fn with_memory(mut self, bytes: u64) -> Self {
        self.max_memory_bytes = bytes;
        self
    }

    /// Parse human-readable storage size (e.g., "100MB", "1GB")
    pub fn parse_storage(size: &str) -> Result<u64, String> {
        // Implementation: Parse "100MB", "1GB", "500KB", etc.
        // Returns bytes as u64
        todo!("Parse human-readable storage size")
    }

    /// Parse human-readable bandwidth (e.g., "10MB/s", "1GB/s")
    pub fn parse_bandwidth(bandwidth: &str) -> Result<u64, String> {
        // Implementation: Parse "10MB/s", "1GB/s", etc.
        // Returns bytes/second as u64
        todo!("Parse human-readable bandwidth")
    }
}
```

**Deliverables:**
- ✅ `ResourceQuota` struct with all quota types
- ✅ Default values (100MB storage, 1000 msg/sec, 10MB/s network, 1000ms CPU, 256MB memory)
- ✅ Builder pattern for custom quotas
- ✅ Parsing functions for human-readable sizes
- ✅ Serde serialization support (TOML/JSON)

**Acceptance Criteria:**
- ResourceQuota compiles without warnings
- Default values match specification
- Builder pattern works (fluent API)
- Parse functions handle valid inputs (100MB, 1GB, etc.)

**Estimated Time:** 1-1.5 hours

---

#### 1.2 QuotaTracker Implementation

**File:** `airssys-wasm/src/security/quota.rs` (continued)

**Objective:** Implement thread-safe quota tracking with atomic counters and time-window rate limiting.

**Implementation:**

```rust
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
/// - `AtomicU32` for message counter
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
    pub fn new(quota: ResourceQuota) -> Self {
        Self {
            quota: quota.clone(),
            storage_used: AtomicU64::new(0),
            message_count: AtomicU32::new(0),
            network_bandwidth_used: AtomicU64::new(0),
            cpu_time_used: AtomicU32::new(0),
            memory_used: AtomicU64::new(0),
            window_data: RwLock::new(TimeWindowData {
                window_start: Instant::now(),
                window_duration: quota.reset_period,
            }),
        }
    }

    /// Check if storage quota allows the given bytes.
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
    pub fn consume_storage(&self, bytes: u64) {
        self.storage_used.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Release storage quota (decrement usage).
    pub fn release_storage(&self, bytes: u64) {
        self.storage_used.fetch_sub(bytes, Ordering::Relaxed);
    }

    /// Check if message rate quota allows the given count.
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
    pub fn consume_message_rate(&self, count: u32) {
        self.message_count.fetch_add(count, Ordering::Relaxed);
    }

    /// Check if network bandwidth quota allows the given bytes.
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
    pub fn consume_network_bandwidth(&self, bytes: u64) {
        self.network_bandwidth_used.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Check if CPU time quota allows the given milliseconds.
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
    pub fn consume_cpu_time(&self, ms: u32) {
        self.cpu_time_used.fetch_add(ms, Ordering::Relaxed);
    }

    /// Check if memory quota allows the given bytes.
    pub fn check_memory(&self, bytes: u64) -> Result<(), QuotaError> {
        let current = self.memory_used.load(Ordering::Relaxed);
        if bytes > self.quota.max_memory_bytes {
            return Err(QuotaError::MemoryExceeded {
                current,
                requested: bytes,
                limit: self.quota.max_memory_bytes,
            });
        }
        Ok(())
    }

    /// Update memory usage (peak tracking).
    pub fn update_memory_usage(&self, bytes: u64) {
        self.memory_used.store(bytes, Ordering::Relaxed);
    }

    /// Reset time window if expired (called before rate limit checks).
    fn reset_window_if_needed(&self) {
        let window_data = self.window_data.read();
        let elapsed = window_data.window_start.elapsed();

        if elapsed >= window_data.window_duration {
            drop(window_data); // Release read lock
            
            // Acquire write lock to reset window
            let mut window_data = self.window_data.write();
            
            // Double-check condition (another thread might have reset)
            if window_data.window_start.elapsed() >= window_data.window_duration {
                // Reset window
                window_data.window_start = Instant::now();
                
                // Reset rate-limited counters
                self.message_count.store(0, Ordering::Relaxed);
                self.network_bandwidth_used.store(0, Ordering::Relaxed);
                self.cpu_time_used.store(0, Ordering::Relaxed);
            }
        }
    }

    /// Get current quota usage statistics.
    pub fn get_usage(&self) -> QuotaUsage {
        QuotaUsage {
            storage_used: self.storage_used.load(Ordering::Relaxed),
            message_count: self.message_count.load(Ordering::Relaxed),
            network_bandwidth_used: self.network_bandwidth_used.load(Ordering::Relaxed),
            cpu_time_used: self.cpu_time_used.load(Ordering::Relaxed),
            memory_used: self.memory_used.load(Ordering::Relaxed),
        }
    }

    /// Reset all quota usage (for testing or manual reset).
    pub fn reset(&self) {
        self.storage_used.store(0, Ordering::Relaxed);
        self.message_count.store(0, Ordering::Relaxed);
        self.network_bandwidth_used.store(0, Ordering::Relaxed);
        self.cpu_time_used.store(0, Ordering::Relaxed);
        self.memory_used.store(0, Ordering::Relaxed);
        
        let mut window_data = self.window_data.write();
        window_data.window_start = Instant::now();
    }
}

/// Snapshot of current quota usage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuotaUsage {
    pub storage_used: u64,
    pub message_count: u32,
    pub network_bandwidth_used: u64,
    pub cpu_time_used: u32,
    pub memory_used: u64,
}

/// Errors that can occur during quota checking.
#[derive(Debug, thiserror::Error)]
pub enum QuotaError {
    #[error("Storage quota exceeded: used {current} bytes + requested {requested} bytes > limit {limit} bytes")]
    StorageExceeded {
        current: u64,
        requested: u64,
        limit: u64,
    },

    #[error("Message rate quota exceeded: {current} messages/sec + {requested} messages > limit {limit} messages/sec")]
    MessageRateExceeded {
        current: u32,
        requested: u32,
        limit: u32,
    },

    #[error("Network bandwidth quota exceeded: {current} bytes/sec + {requested} bytes > limit {limit} bytes/sec")]
    NetworkBandwidthExceeded {
        current: u64,
        requested: u64,
        limit: u64,
    },

    #[error("CPU time quota exceeded: {current} ms/sec + {requested} ms > limit {limit} ms/sec")]
    CpuTimeExceeded {
        current: u32,
        requested: u32,
        limit: u32,
    },

    #[error("Memory quota exceeded: requested {requested} bytes > limit {limit} bytes (current: {current} bytes)")]
    MemoryExceeded {
        current: u64,
        requested: u64,
        limit: u64,
    },
}
```

**Deliverables:**
- ✅ `QuotaTracker` struct with atomic counters
- ✅ Time-window based rate limiting (automatic reset)
- ✅ Check methods for all quota types
- ✅ Consume/release methods for quota updates
- ✅ Thread-safe implementation (atomic + RwLock)
- ✅ `QuotaUsage` snapshot struct
- ✅ `QuotaError` enum with detailed error messages

**Acceptance Criteria:**
- QuotaTracker compiles without warnings
- All check methods work correctly
- Time window reset logic works
- Thread-safe (no data races)
- Performance: <5μs check, <2μs update

**Estimated Time:** 2-2.5 hours

---

#### 1.3 Integration with WasmSecurityContext

**File:** `airssys-wasm/src/security/capability.rs` (modify existing)

**Objective:** Extend `WasmSecurityContext` to include resource quota and tracker.

**Implementation:**

```rust
// Modify existing WasmSecurityContext struct

use crate::security::quota::{ResourceQuota, QuotaTracker};

/// WASM component security context (UPDATED for Task 4.3).
///
/// # Changes (Task 4.3)
///
/// - Added `resource_quota: ResourceQuota` field
/// - Added `quota_tracker: Arc<QuotaTracker>` field
#[derive(Debug, Clone)]
pub struct WasmSecurityContext {
    /// Unique identifier for the component
    pub component_id: String,

    /// Set of capabilities granted to the component
    pub capabilities: WasmCapabilitySet,

    /// Resource quota configuration (Task 4.3)
    pub resource_quota: ResourceQuota,

    /// Quota usage tracker (Task 4.3)
    pub quota_tracker: Arc<QuotaTracker>,
}

impl WasmSecurityContext {
    /// Create a new WASM security context (UPDATED for Task 4.3).
    pub fn new(component_id: String, capabilities: WasmCapabilitySet) -> Self {
        let resource_quota = ResourceQuota::default();
        let quota_tracker = Arc::new(QuotaTracker::new(resource_quota.clone()));

        Self {
            component_id,
            capabilities,
            resource_quota,
            quota_tracker,
        }
    }

    /// Create a new security context with custom quota (Task 4.3).
    pub fn with_quota(
        component_id: String,
        capabilities: WasmCapabilitySet,
        resource_quota: ResourceQuota,
    ) -> Self {
        let quota_tracker = Arc::new(QuotaTracker::new(resource_quota.clone()));

        Self {
            component_id,
            capabilities,
            resource_quota,
            quota_tracker,
        }
    }
}
```

**Deliverables:**
- ✅ Updated `WasmSecurityContext` with quota fields
- ✅ Default quota initialization in `new()`
- ✅ Custom quota constructor `with_quota()`

**Acceptance Criteria:**
- WasmSecurityContext compiles without warnings
- Existing tests still pass
- New constructors work correctly

**Estimated Time:** 30 minutes

---

### Phase 2: Enforcement Integration (Day 1, 3-4 hours)

#### 2.1 Quota Check Integration with Capability System

**File:** `airssys-wasm/src/security/enforcement.rs` (modify existing)

**Objective:** Integrate quota checks into the capability enforcement flow.

**Implementation:**

```rust
// Add to enforcement.rs

use crate::security::quota::{QuotaError, QuotaTracker};

/// Enhanced capability check with quota enforcement (Task 4.3).
///
/// This function performs both capability check (Task 3.1) and quota check (Task 4.3)
/// before allowing a host function to proceed.
///
/// # Workflow
///
/// 1. Check capability (Task 3.1) - Does component have permission?
/// 2. Check quota (Task 4.3) - Has component exceeded resource limits?
/// 3. Return result (allow or deny)
///
/// # Note
///
/// This function does NOT update quota usage. Host functions must call
/// `consume_quota()` after successful operation to update usage counters.
///
/// # Examples
///
/// ```rust,ignore
/// // In host function implementation
/// fn filesystem_write(component_id: &str, path: &str, data: &[u8]) -> Result<(), Error> {
///     // Check capability + quota
///     check_capability_with_quota(
///         component_id,
///         path,
///         "write",
///         QuotaType::Storage(data.len() as u64),
///     )?;
///     
///     // Perform operation
///     std::fs::write(path, data)?;
///     
///     // Update quota usage
///     consume_quota(component_id, QuotaType::Storage(data.len() as u64))?;
///     
///     Ok(())
/// }
/// ```
pub fn check_capability_with_quota(
    component_id: &str,
    resource: &str,
    permission: &str,
    quota_type: QuotaType,
) -> Result<(), CapabilityCheckError> {
    // 1. Check capability (existing Task 3.1 logic)
    check_capability(component_id, resource, permission)?;

    // 2. Get component security context (contains quota tracker)
    let context = get_component_context_internal(component_id)?;

    // 3. Check quota based on type
    match quota_type {
        QuotaType::Storage(bytes) => {
            context.quota_tracker.check_storage(bytes)
                .map_err(|e| CapabilityCheckError::AccessDenied {
                    reason: format!("Quota check failed: {}", e),
                })?;
        }
        QuotaType::MessageRate(count) => {
            context.quota_tracker.check_message_rate(count)
                .map_err(|e| CapabilityCheckError::AccessDenied {
                    reason: format!("Quota check failed: {}", e),
                })?;
        }
        QuotaType::NetworkBandwidth(bytes) => {
            context.quota_tracker.check_network_bandwidth(bytes)
                .map_err(|e| CapabilityCheckError::AccessDenied {
                    reason: format!("Quota check failed: {}", e),
                })?;
        }
        QuotaType::CpuTime(ms) => {
            context.quota_tracker.check_cpu_time(ms)
                .map_err(|e| CapabilityCheckError::AccessDenied {
                    reason: format!("Quota check failed: {}", e),
                })?;
        }
        QuotaType::Memory(bytes) => {
            context.quota_tracker.check_memory(bytes)
                .map_err(|e| CapabilityCheckError::AccessDenied {
                    reason: format!("Quota check failed: {}", e),
                })?;
        }
        QuotaType::None => {
            // No quota check needed
        }
    }

    Ok(())
}

/// Consume quota after successful operation (Task 4.3).
///
/// This function updates quota usage counters after a host function
/// successfully completes an operation. Must be called after the
/// operation to maintain accurate quota tracking.
///
/// # Examples
///
/// ```rust,ignore
/// // After successful filesystem write
/// consume_quota(component_id, QuotaType::Storage(bytes_written))?;
///
/// // After successful message send
/// consume_quota(component_id, QuotaType::MessageRate(1))?;
///
/// // After successful network transfer
/// consume_quota(component_id, QuotaType::NetworkBandwidth(bytes_transferred))?;
/// ```
pub fn consume_quota(
    component_id: &str,
    quota_type: QuotaType,
) -> Result<(), CapabilityCheckError> {
    let context = get_component_context_internal(component_id)?;

    match quota_type {
        QuotaType::Storage(bytes) => {
            context.quota_tracker.consume_storage(bytes);
        }
        QuotaType::MessageRate(count) => {
            context.quota_tracker.consume_message_rate(count);
        }
        QuotaType::NetworkBandwidth(bytes) => {
            context.quota_tracker.consume_network_bandwidth(bytes);
        }
        QuotaType::CpuTime(ms) => {
            context.quota_tracker.consume_cpu_time(ms);
        }
        QuotaType::Memory(bytes) => {
            context.quota_tracker.update_memory_usage(bytes);
        }
        QuotaType::None => {
            // No quota consumption
        }
    }

    Ok(())
}

/// Release quota (for operations that failed or were rolled back).
///
/// # Examples
///
/// ```rust,ignore
/// // If operation failed after quota was consumed
/// release_quota(component_id, QuotaType::Storage(bytes))?;
/// ```
pub fn release_quota(
    component_id: &str,
    quota_type: QuotaType,
) -> Result<(), CapabilityCheckError> {
    let context = get_component_context_internal(component_id)?;

    match quota_type {
        QuotaType::Storage(bytes) => {
            context.quota_tracker.release_storage(bytes);
        }
        _ => {
            // Other quota types don't support release
        }
    }

    Ok(())
}

/// Quota type specification for quota checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuotaType {
    /// Storage quota (bytes)
    Storage(u64),

    /// Message rate quota (count)
    MessageRate(u32),

    /// Network bandwidth quota (bytes)
    NetworkBandwidth(u64),

    /// CPU time quota (milliseconds)
    CpuTime(u32),

    /// Memory quota (bytes)
    Memory(u64),

    /// No quota check needed
    None,
}

// Helper function to get component context (internal use)
fn get_component_context_internal(
    component_id: &str,
) -> Result<Arc<WasmSecurityContext>, CapabilityCheckError> {
    global_checker()
        .contexts
        .get(component_id)
        .map(|ctx| Arc::clone(ctx.value()))
        .ok_or_else(|| CapabilityCheckError::ComponentNotFound {
            component_id: component_id.to_string(),
        })
}
```

**Deliverables:**
- ✅ `check_capability_with_quota()` function
- ✅ `consume_quota()` function
- ✅ `release_quota()` function
- ✅ `QuotaType` enum for quota specification
- ✅ Integration with existing capability check system

**Acceptance Criteria:**
- Functions compile without warnings
- Quota checks work correctly
- Quota consumption updates counters
- Integration with Task 3.1 works

**Estimated Time:** 1.5-2 hours

---

#### 2.2 Host Function Integration Pattern

**File:** `airssys-wasm/src/security/host_integration.rs` (create new)

**Objective:** Provide reusable patterns and macros for host function quota enforcement.

**Implementation:**

```rust
//! Host Function Integration Patterns for Quota Enforcement
//!
//! This module provides reusable patterns and macros for integrating
//! quota checks into host function implementations.

/// Macro for host functions with quota enforcement.
///
/// Reduces boilerplate by combining capability check, quota check,
/// operation execution, and quota consumption into a single macro.
///
/// # Syntax
///
/// ```rust,ignore
/// with_quota!(component_id, resource, permission, quota_type => {
///     // Operation code
/// })
/// ```
///
/// # Examples
///
/// ## Filesystem Write
///
/// ```rust,ignore
/// fn filesystem_write(component_id: &str, path: &str, data: &[u8]) -> Result<(), Error> {
///     with_quota!(
///         component_id,
///         path,
///         "write",
///         QuotaType::Storage(data.len() as u64)
///     => {
///         std::fs::write(path, data)?;
///         Ok(())
///     })
/// }
/// ```
///
/// ## Message Send
///
/// ```rust,ignore
/// fn send_message(component_id: &str, target: &str, message: Message) -> Result<(), Error> {
///     with_quota!(
///         component_id,
///         target,
///         "send",
///         QuotaType::MessageRate(1)
///     => {
///         RUNTIME.send_message(target, message)?;
///         Ok(())
///     })
/// }
/// ```
#[macro_export]
macro_rules! with_quota {
    ($component_id:expr, $resource:expr, $permission:expr, $quota_type:expr => $operation:block) => {{
        // 1. Check capability + quota
        $crate::security::enforcement::check_capability_with_quota(
            $component_id,
            $resource,
            $permission,
            $quota_type,
        )?;

        // 2. Execute operation
        let result = (|| $operation)();

        // 3. Consume quota if operation succeeded
        if result.is_ok() {
            $crate::security::enforcement::consume_quota($component_id, $quota_type)?;
        }

        // 4. Return result
        result
    }};
}
```

**Deliverables:**
- ✅ `with_quota!` macro for host functions
- ✅ Documentation and examples

**Acceptance Criteria:**
- Macro compiles without warnings
- Macro works in host function context
- Reduces boilerplate significantly

**Estimated Time:** 1 hour

---

### Phase 3: Configuration and Monitoring (Day 2, 3-4 hours)

#### 3.1 Quota Configuration System

**File:** `airssys-wasm/src/security/parser.rs` (modify existing)

**Objective:** Parse quota configuration from Component.toml.

**Implementation:**

```rust
// Add to parser.rs

use crate::security::quota::ResourceQuota;

/// Component manifest with quota configuration (UPDATED for Task 4.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentManifest {
    pub component: ComponentMetadata,
    pub capabilities: CapabilitiesSection,
    
    /// Resource quota configuration (Task 4.3)
    #[serde(default)]
    pub quota: Option<QuotaSection>,
}

/// Quota configuration section in Component.toml (Task 4.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaSection {
    /// Storage quota (e.g., "100MB", "1GB")
    pub storage: Option<String>,

    /// Message rate quota (messages/second)
    pub message_rate: Option<u32>,

    /// Network bandwidth quota (e.g., "10MB/s", "1GB/s")
    pub network_bandwidth: Option<String>,

    /// CPU time quota (e.g., "1000ms/s", "500ms/s")
    pub cpu_time: Option<String>,

    /// Memory quota (e.g., "256MB", "512MB")
    pub memory: Option<String>,
}

impl QuotaSection {
    /// Convert QuotaSection to ResourceQuota.
    pub fn to_resource_quota(&self) -> Result<ResourceQuota, ParseError> {
        let mut quota = ResourceQuota::default();

        if let Some(ref storage) = self.storage {
            quota.max_storage_bytes = ResourceQuota::parse_storage(storage)
                .map_err(|e| ParseError::InvalidQuota {
                    field: "storage".to_string(),
                    reason: e,
                })?;
        }

        if let Some(message_rate) = self.message_rate {
            quota.max_message_rate = message_rate;
        }

        if let Some(ref bandwidth) = self.network_bandwidth {
            quota.max_network_bandwidth = ResourceQuota::parse_bandwidth(bandwidth)
                .map_err(|e| ParseError::InvalidQuota {
                    field: "network_bandwidth".to_string(),
                    reason: e,
                })?;
        }

        if let Some(ref cpu_time) = self.cpu_time {
            quota.max_cpu_time_ms = parse_cpu_time(cpu_time)
                .map_err(|e| ParseError::InvalidQuota {
                    field: "cpu_time".to_string(),
                    reason: e,
                })?;
        }

        if let Some(ref memory) = self.memory {
            quota.max_memory_bytes = ResourceQuota::parse_storage(memory)
                .map_err(|e| ParseError::InvalidQuota {
                    field: "memory".to_string(),
                    reason: e,
                })?;
        }

        Ok(quota)
    }
}

fn parse_cpu_time(s: &str) -> Result<u32, String> {
    // Parse "1000ms/s", "500ms/s", etc.
    // Returns milliseconds per second as u32
    todo!("Parse CPU time string")
}
```

**Component.toml Example:**

```toml
[component]
name = "data-processor"
version = "1.0.0"

[capabilities]
filesystem.read = ["/app/data/*"]
filesystem.write = ["/app/output/*"]

[quota]
storage = "50MB"
message_rate = 500
network_bandwidth = "5MB/s"
cpu_time = "800ms/s"
memory = "128MB"
```

**Deliverables:**
- ✅ `QuotaSection` struct for TOML parsing
- ✅ `to_resource_quota()` conversion method
- ✅ Parse functions for human-readable formats
- ✅ Default values when quota section is missing

**Acceptance Criteria:**
- Component.toml with quota section parses correctly
- Missing quota section uses defaults
- Invalid quota values return clear errors

**Estimated Time:** 1.5-2 hours

---

#### 3.2 Quota Monitoring API

**File:** `airssys-wasm/src/security/quota.rs` (continued)

**Objective:** Provide API for querying quota usage and remaining quota.

**Implementation:**

```rust
// Add to quota.rs

impl QuotaTracker {
    /// Get quota usage and remaining quota for all resource types.
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
    pub fn is_quota_warning(&self) -> bool {
        let status = self.get_quota_status();
        status.storage.percentage >= 80.0
            || status.message_rate.percentage >= 80.0
            || status.network_bandwidth.percentage >= 80.0
            || status.cpu_time.percentage >= 80.0
            || status.memory.percentage >= 80.0
    }

    /// Check if any quota is exhausted (>95% used).
    pub fn is_quota_critical(&self) -> bool {
        let status = self.get_quota_status();
        status.storage.percentage >= 95.0
            || status.message_rate.percentage >= 95.0
            || status.network_bandwidth.percentage >= 95.0
            || status.cpu_time.percentage >= 95.0
            || status.memory.percentage >= 95.0
    }
}

/// Complete quota status for a component.
#[derive(Debug, Clone, PartialEq)]
pub struct QuotaStatus {
    pub storage: QuotaResourceStatus,
    pub message_rate: QuotaResourceStatus,
    pub network_bandwidth: QuotaResourceStatus,
    pub cpu_time: QuotaResourceStatus,
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

fn calculate_percentage(used: u64, limit: u64) -> f64 {
    if limit == 0 {
        0.0
    } else {
        (used as f64 / limit as f64) * 100.0
    }
}

// Add to enforcement.rs

/// Get quota status for a component (monitoring API).
pub fn get_quota_status(component_id: &str) -> Result<QuotaStatus, CapabilityCheckError> {
    let context = get_component_context_internal(component_id)?;
    Ok(context.quota_tracker.get_quota_status())
}

/// Check if component has quota warning (>80% usage).
pub fn has_quota_warning(component_id: &str) -> Result<bool, CapabilityCheckError> {
    let context = get_component_context_internal(component_id)?;
    Ok(context.quota_tracker.is_quota_warning())
}

/// Check if component has critical quota usage (>95% usage).
pub fn has_quota_critical(component_id: &str) -> Result<bool, CapabilityCheckError> {
    let context = get_component_context_internal(component_id)?;
    Ok(context.quota_tracker.is_quota_critical())
}
```

**Deliverables:**
- ✅ `QuotaStatus` struct with detailed status
- ✅ `QuotaResourceStatus` for individual resources
- ✅ `get_quota_status()` API function
- ✅ Warning/critical threshold checks

**Acceptance Criteria:**
- Quota status API returns accurate data
- Percentage calculations are correct
- Warning/critical thresholds work

**Estimated Time:** 1-1.5 hours

---

### Phase 4: Comprehensive Testing (Day 2, 4-5 hours)

#### 4.1 Unit Tests for QuotaTracker

**File:** `airssys-wasm/src/security/quota.rs` (test module)

**Objective:** Test all quota tracking functionality.

**Test Cases:**

1. **Storage Quota Tests**
   - ✅ Check storage quota (within limit)
   - ✅ Check storage quota (exceeds limit)
   - ✅ Consume storage quota
   - ✅ Release storage quota
   - ✅ Multiple storage operations

2. **Message Rate Tests**
   - ✅ Check message rate quota (within limit)
   - ✅ Check message rate quota (exceeds limit)
   - ✅ Consume message rate quota
   - ✅ Time window reset (auto-reset after duration)

3. **Network Bandwidth Tests**
   - ✅ Check network bandwidth quota (within limit)
   - ✅ Check network bandwidth quota (exceeds limit)
   - ✅ Consume network bandwidth quota
   - ✅ Time window reset

4. **CPU Time Tests**
   - ✅ Check CPU time quota (within limit)
   - ✅ Check CPU time quota (exceeds limit)
   - ✅ Consume CPU time quota
   - ✅ Time window reset

5. **Memory Tests**
   - ✅ Check memory quota (within limit)
   - ✅ Check memory quota (exceeds limit)
   - ✅ Update memory usage (peak tracking)

6. **Edge Cases**
   - ✅ Quota = 0 (always denied)
   - ✅ Quota = u64::MAX (infinite quota)
   - ✅ Concurrent access (thread safety)
   - ✅ Time window boundary conditions

**Estimated Time:** 2-2.5 hours

---

#### 4.2 Integration Tests

**File:** `airssys-wasm/tests/quota_integration_tests.rs`

**Objective:** Test quota enforcement in realistic scenarios.

**Test Cases:**

1. **Component Registration with Quota**
   - ✅ Register component with default quota
   - ✅ Register component with custom quota
   - ✅ Parse quota from Component.toml

2. **Host Function Integration**
   - ✅ Filesystem write with storage quota
   - ✅ Message send with rate limit quota
   - ✅ Network transfer with bandwidth quota

3. **Quota Violation Scenarios**
   - ✅ Storage exhaustion (gradual)
   - ✅ Message rate spike (burst)
   - ✅ Network bandwidth burst

4. **Quota Monitoring**
   - ✅ Get quota status (usage, remaining)
   - ✅ Warning threshold detection (>80%)
   - ✅ Critical threshold detection (>95%)

5. **Multi-Component Isolation**
   - ✅ Component A exceeds quota → Component B unaffected
   - ✅ Independent quota tracking per component

**Estimated Time:** 1.5-2 hours

---

#### 4.3 Performance Benchmarks

**File:** `airssys-wasm/benches/quota_benchmarks.rs`

**Objective:** Verify performance targets are met.

**Benchmarks:**

1. **Quota Check Overhead**
   - ✅ Storage check: <5μs
   - ✅ Message rate check: <5μs
   - ✅ Network bandwidth check: <5μs
   - ✅ CPU time check: <5μs
   - ✅ Memory check: <5μs

2. **Quota Update Overhead**
   - ✅ Storage consume: <2μs
   - ✅ Message rate consume: <2μs
   - ✅ Network bandwidth consume: <2μs

3. **Concurrent Access**
   - ✅ 1000 concurrent checks: <10ms total
   - ✅ No lock contention

**Estimated Time:** 30 minutes

---

## Integration Points

### 1. Task 3.1 (Capability Check API)

**Integration:** Quota checks are layered on top of capability checks.

```rust
// Existing Task 3.1 API
check_capability(component_id, resource, permission)?;

// New Task 4.3 API (wraps Task 3.1)
check_capability_with_quota(
    component_id,
    resource,
    permission,
    QuotaType::Storage(bytes),
)?;
```

**No Breaking Changes:** Existing `check_capability()` continues to work.

---

### 2. Task 4.1 (Security Context Attachment)

**Integration:** `WasmSecurityContext` extended with quota fields.

```rust
// Existing Task 4.1
pub struct WasmSecurityContext {
    pub component_id: String,
    pub capabilities: WasmCapabilitySet,
    // NEW (Task 4.3):
    pub resource_quota: ResourceQuota,
    pub quota_tracker: Arc<QuotaTracker>,
}
```

**Backward Compatibility:** Default quota applied if not specified.

---

### 3. Task 3.3 (Audit Logging)

**Integration:** Quota violations logged via existing audit system.

```rust
// Quota violation audit event
audit_logger.log_quota_violation(WasmQuotaAuditLog {
    component_id,
    quota_type: "storage",
    current_usage: 90 * 1024 * 1024,
    requested: 20 * 1024 * 1024,
    limit: 100 * 1024 * 1024,
    reason: "Storage quota exceeded",
});
```

---

## Dependencies

### Completed Dependencies

- ✅ **Task 3.1**: Capability Check API (required for integration)
- ✅ **Task 4.1**: ComponentActor Security Context (required for quota attachment)
- ✅ **Task 3.3**: Audit Logging (required for quota violation logging)

### No Blocking Dependencies

All prerequisites are complete. Task 4.3 can start immediately.

---

## Risk Analysis

### Technical Risks

#### Risk 1: Time Window Reset Race Conditions

**Probability:** Low  
**Impact:** Medium  
**Mitigation:**
- Use `parking_lot::RwLock` for time window data
- Double-check pattern in `reset_window_if_needed()`
- Atomic operations for counters (lock-free)

**Code Pattern:**
```rust
// Double-check locking pattern
let window_data = self.window_data.read();
if window_data.needs_reset() {
    drop(window_data); // Release read lock
    let mut window_data = self.window_data.write();
    if window_data.needs_reset() { // Double-check
        window_data.reset();
    }
}
```

---

#### Risk 2: Performance Overhead

**Probability:** Low  
**Impact:** High  
**Mitigation:**
- Use atomic operations (lock-free)
- Lazy time window checks (only when needed)
- Benchmark early and often
- Target: <10μs total overhead

**Performance Budget:**
- Capability check: 3-5μs (Task 3.1)
- Quota check: 2-3μs (Task 4.3)
- Quota update: 1-2μs (Task 4.3)
- Total: 6-10μs (within 10μs target)

---

#### Risk 3: Quota Configuration Parsing Errors

**Probability:** Medium  
**Impact:** Low  
**Mitigation:**
- Comprehensive validation
- Clear error messages
- Default values as fallback
- Schema validation for Component.toml

---

### Mitigation Strategies

1. **Atomic Operations:** Use `AtomicU64`/`AtomicU32` for all counters
2. **Benchmarking:** Early performance validation (Day 1)
3. **Testing:** 15+ tests covering edge cases
4. **Documentation:** Clear examples and error messages

---

## Complexity Estimates

### Lines of Code

- **quota.rs**: ~600 lines (ResourceQuota, QuotaTracker, QuotaStatus)
- **enforcement.rs**: ~200 lines (integration functions)
- **parser.rs**: ~100 lines (TOML parsing)
- **tests**: ~800 lines (unit + integration tests)
- **benchmarks**: ~200 lines

**Total:** ~1,900 lines

---

### Test Count

- **Unit tests**: 12-15 tests
- **Integration tests**: 5-8 tests
- **Benchmarks**: 3-5 benchmarks

**Total:** 20-28 tests (exceeds 15+ target)

---

## Timeline Breakdown

### Day 1 (6-8 hours)

#### Morning (3-4 hours)
- ✅ Phase 1.1: ResourceQuota struct (1-1.5 hours)
- ✅ Phase 1.2: QuotaTracker implementation (2-2.5 hours)

#### Afternoon (3-4 hours)
- ✅ Phase 1.3: WasmSecurityContext integration (30 minutes)
- ✅ Phase 2.1: Quota check integration (1.5-2 hours)
- ✅ Phase 2.2: Host function patterns (1 hour)

**End of Day 1 Milestone:**
- Core quota system functional
- Basic integration with capability system
- Manual testing possible

---

### Day 2 (6-8 hours)

#### Morning (3-4 hours)
- ✅ Phase 3.1: Configuration system (1.5-2 hours)
- ✅ Phase 3.2: Monitoring API (1-1.5 hours)

#### Afternoon (3-4 hours)
- ✅ Phase 4.1: Unit tests (2-2.5 hours)
- ✅ Phase 4.2: Integration tests (1.5-2 hours)
- ✅ Phase 4.3: Performance benchmarks (30 minutes)

**End of Day 2 Milestone:**
- Complete quota system
- 15+ tests passing
- Performance targets met
- Documentation complete

---

## References

### Related Tasks

- **Task 3.1**: Capability Check API (integration point)
- **Task 4.1**: ComponentActor Security Context (integration point)
- **Task 3.3**: Audit Logging (integration point)

### ADRs

- **ADR-WASM-005**: Capability-Based Security Model (§2.3 - Resource Quotas)

### Standards

- **PROJECTS_STANDARD.md**: §2.1 (import organization), §4.3 (module structure)
- **Microsoft Rust Guidelines**: M-ERRORS-CANONICAL, M-ESSENTIAL-FN-INHERENT
- **rust.instructions.md**: Safety, testing, performance guidelines

### Code References

- `airssys-wasm/src/security/capability.rs` (WasmSecurityContext)
- `airssys-wasm/src/security/enforcement.rs` (check_capability)
- `airssys-wasm/src/security/parser.rs` (Component.toml parsing)

---

## Approval Checklist

Before implementation begins, verify:

- ✅ Plan reviewed by task requester
- ✅ Architecture aligns with ADR-WASM-005
- ✅ Integration points with Task 3.1, 4.1, 3.3 clear
- ✅ Performance targets feasible (<10μs overhead)
- ✅ Test strategy comprehensive (15+ tests)
- ✅ Timeline realistic (2 days)
- ✅ Deliverables clearly defined
- ✅ Success criteria measurable

---

## Success Criteria (Checklist)

### Functionality
- ✅ Components respect quota limits (storage, message rate, network, CPU, memory)
- ✅ Quota violations return clear errors (no silent failures)
- ✅ Quota configuration loaded from Component.toml
- ✅ Quota monitoring API functional (usage metrics, remaining quota)

### Testing
- ✅ 15+ test cases covering quota scenarios
- ✅ All tests passing (unit + integration)
- ✅ Edge cases tested (quota=0, quota=∞, concurrent access)

### Performance
- ✅ Quota check overhead <10μs per operation
- ✅ Quota update overhead <5μs per operation
- ✅ Memory overhead <1KB per component
- ✅ Thread-safe with minimal contention

### Quality
- ✅ Zero compiler warnings (cargo build)
- ✅ Zero clippy warnings (cargo clippy)
- ✅ Zero rustdoc warnings (cargo doc)
- ✅ 95%+ code review quality score

### Documentation
- ✅ Comprehensive rustdoc for all public types
- ✅ Examples for common usage patterns
- ✅ Integration guide for host functions
- ✅ Component.toml quota configuration examples

---

**Status:** Ready for Approval  
**Next Step:** User approval of plan before implementation begins  

---

**Do you approve this plan? (Yes/No)**

If approved, I will proceed with implementation following this exact plan.
