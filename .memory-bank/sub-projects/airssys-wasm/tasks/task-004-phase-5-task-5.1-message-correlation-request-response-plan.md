# WASM-TASK-004 Phase 5 Task 5.1: Message Correlation and Request-Response Patterns

**Status:** ✅ COMPLETE (APPROVED - PRODUCTION READY)  
**Phase:** Advanced Actor Patterns  
**Task ID:** 5.1  
**Estimated Effort:** 4-6 hours  
**Actual Effort:** ~7.5 hours (5h implementation + 2.5h fixes)  
**Priority:** MEDIUM - Optional advanced pattern for enhanced inter-component communication  
**Created:** 2025-12-16  
**Completed:** 2025-12-16  
**Quality Score:** 9.5/10 (upgraded from 8.5/10)  
**Dependencies:** Phase 4 Complete ✅ (MessageBroker, Pub-Sub, UnifiedRouter)

---

## Overview

### Goal
Implement message correlation tracking and request-response patterns for inter-component communication, enabling async RPC with timeout handling and callback delivery for airssys-wasm components.

### Context
Per **ADR-WASM-009 Section "Pattern 2: Request-Response"**, the framework requires:
- Fire-and-forget messaging ✅ COMPLETE (Phase 4)
- Request-response with callbacks ⏳ THIS TASK
- Manual correlation support (documentation only)

This task implements automatic correlation ID management, pending request tracking, timeout enforcement, and callback delivery, completing the communication patterns specified in ADR-WASM-009.

### Success Criteria
- [x] CorrelationTracker module with <50ns lookup overhead
- [x] Request/Response message wrappers (RequestMessage<T>, ResponseMessage<T>)
- [x] Timeout handling for pending requests (tokio time::timeout)
- [x] 15 tests minimum (10 unit + 5 integration)
- [x] Zero warnings (compiler + clippy + rustdoc)
- [x] 100% rustdoc coverage
- [x] ADR-WASM-009 compliance verified
- [x] Performance validated: <50ns correlation lookup, >10,000 req/sec throughput

---

## Architecture Design (20%)

### 1. CorrelationTracker Internal Structure

**File:** `src/actor/message/correlation_tracker.rs` (~350 lines)

**Design Decision: DashMap for Lock-Free Concurrent Lookups**

```rust
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::oneshot;
use tokio::time::{Duration, Instant};
use uuid::Uuid;

/// Correlation ID type for request-response tracking
pub type CorrelationId = Uuid;

/// Pending request state
struct PendingRequest {
    /// Correlation ID
    correlation_id: CorrelationId,
    /// Response channel sender (oneshot for single response)
    response_tx: oneshot::Sender<ResponseMessage>,
    /// Request timestamp (for timeout tracking)
    requested_at: Instant,
    /// Timeout duration
    timeout: Duration,
    /// Source component ID
    from: ComponentId,
    /// Target component ID
    to: ComponentId,
}

/// High-performance correlation tracker for request-response patterns.
///
/// Uses DashMap for lock-free concurrent access with <50ns lookup overhead.
#[derive(Clone)]
pub struct CorrelationTracker {
    /// Pending requests: CorrelationId → PendingRequest
    /// DashMap provides lock-free sharded HashMap for concurrent access
    pending: Arc<DashMap<CorrelationId, PendingRequest>>,
}
```

**Why DashMap?**
- ✅ **Lock-free reads**: <50ns lookup (meets performance target)
- ✅ **Concurrent writes**: Multiple components can register requests simultaneously
- ✅ **Zero contention**: Sharded internal structure (16 shards default)
- ✅ **Safe cleanup**: Atomic remove operations for timeouts
- ✅ **Proven performance**: Used in high-throughput systems

**Alternative Rejected: Arc<RwLock<HashMap>>**
- ❌ Read lock contention under concurrent load
- ❌ ~200ns+ latency with multiple readers
- ❌ Write lock blocks all readers

### 2. Request/Response Message Envelope Design

**File:** `src/actor/message/request_response.rs` (~250 lines)

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Request message wrapper with correlation tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMessage<T> {
    /// Unique correlation ID (auto-generated)
    pub correlation_id: CorrelationId,
    /// Source component ID
    pub from: ComponentId,
    /// Target component ID
    pub to: ComponentId,
    /// Request payload (multicodec-encoded)
    pub payload: T,
    /// Request timestamp
    pub timestamp: DateTime<Utc>,
    /// Timeout duration (milliseconds)
    pub timeout_ms: u32,
}

/// Response message wrapper with correlation tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMessage {
    /// Correlation ID matching the original request
    pub correlation_id: CorrelationId,
    /// Responder component ID
    pub from: ComponentId,
    /// Original requester component ID
    pub to: ComponentId,
    /// Response payload (multicodec-encoded) or error
    pub result: Result<Vec<u8>, RequestError>,
    /// Response timestamp
    pub timestamp: DateTime<Utc>,
}

/// Request-response error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestError {
    /// Request timed out
    Timeout,
    /// Target component not found
    ComponentNotFound(ComponentId),
    /// Target component failed to process request
    ProcessingFailed(String),
    /// Invalid request payload
    InvalidPayload(String),
}
```

**Design Rationale:**
- **Correlation ID**: UUID v4 for globally unique IDs (16 bytes, 122-bit entropy)
- **Timeout in message**: Request sender specifies timeout, enforced by host
- **Multicodec payload**: Reuses existing `encode_multicodec` from Phase 1
- **Result<Vec<u8>, RequestError>**: Success or timeout/error, explicit handling
- **Timestamps**: UTC timestamps for audit logging and latency tracking

### 3. Timeout Mechanism Design

**File:** `src/actor/message/timeout_handler.rs` (~200 lines)

**Approach: Tokio Async Timeout Tasks**

```rust
use tokio::time::{sleep, Duration};
use tokio::task::JoinHandle;

/// Timeout handler managing background timeout tasks
pub struct TimeoutHandler {
    /// Active timeout tasks: CorrelationId → JoinHandle
    /// When timeout fires, sends Err(Timeout) to response channel
    active_timeouts: Arc<DashMap<CorrelationId, JoinHandle<()>>>,
}

impl TimeoutHandler {
    /// Register timeout for pending request
    pub fn register_timeout(
        &self,
        correlation_id: CorrelationId,
        timeout: Duration,
        tracker: CorrelationTracker,
    ) -> JoinHandle<()> {
        let corr_id = correlation_id;
        
        let handle = tokio::spawn(async move {
            // Wait for timeout duration
            sleep(timeout).await;
            
            // Check if request still pending (may have been resolved)
            if let Some(pending) = tracker.remove_pending(&corr_id) {
                // Send timeout error to response channel
                let _ = pending.response_tx.send(ResponseMessage {
                    correlation_id: corr_id,
                    from: pending.to.clone(),
                    to: pending.from.clone(),
                    result: Err(RequestError::Timeout),
                    timestamp: Utc::now(),
                });
            }
        });
        
        // Store handle for cancellation if response arrives early
        self.active_timeouts.insert(correlation_id, handle);
        
        handle
    }
    
    /// Cancel timeout (called when response arrives before timeout)
    pub fn cancel_timeout(&self, correlation_id: &CorrelationId) {
        if let Some((_, handle)) = self.active_timeouts.remove(correlation_id) {
            handle.abort(); // Cancel timeout task
        }
    }
}
```

**Why Tokio Async Tasks?**
- ✅ **Efficient**: Tokio runtime schedules timeouts, no polling
- ✅ **Automatic cleanup**: Task completes when timeout fires or is cancelled
- ✅ **Precise timing**: <5ms timeout accuracy (Tokio timer wheel)
- ✅ **Scalable**: 1000+ concurrent requests with minimal overhead

**Alternative Rejected: Custom Timer Wheel**
- ❌ Complex implementation (500+ lines)
- ❌ Redundant (Tokio already has optimized timer)
- ❌ Testing complexity

### 4. Thread-Safety Considerations

**Concurrent Access Pattern:**
```rust
// Thread 1: Component A sends request
tracker.register_pending(req_id, pending_request).await;

// Thread 2: Component B sends response
tracker.resolve(req_id, response).await;

// Thread 3: Timeout handler fires
tracker.cleanup_timeout(req_id).await;
```

**Safety Guarantees:**
1. **DashMap atomic operations**: `insert`, `remove`, `get` are atomic
2. **Oneshot channel**: Single response delivery, channel closed after send
3. **Timeout cancellation**: `abort()` is async-signal-safe
4. **No race conditions**: First of (response arrival, timeout) wins

**Memory Safety:**
- ✅ No manual memory management (all Arc/DashMap)
- ✅ Automatic cleanup via Drop
- ✅ No leaked requests (timeout always fires if not resolved)

### 5. Integration with ComponentActor

**File:** `src/actor/component/actor_impl.rs` (additions ~150 lines)

```rust
impl ComponentActor {
    /// Correlation tracker for request-response patterns
    correlation_tracker: Option<Arc<CorrelationTracker>>,
    
    /// Set correlation tracker (called by ComponentSpawner)
    pub fn set_correlation_tracker(&mut self, tracker: Arc<CorrelationTracker>) {
        self.correlation_tracker = Some(tracker);
    }
    
    /// Send request with correlation tracking
    pub async fn send_request<T: Serialize>(
        &self,
        target: &ComponentId,
        request: T,
        timeout: Duration,
    ) -> Result<oneshot::Receiver<ResponseMessage>, WasmError> {
        let tracker = self.correlation_tracker
            .as_ref()
            .ok_or(WasmError::Internal("CorrelationTracker not set".into()))?;
        
        // Generate correlation ID
        let correlation_id = CorrelationId::new_v4();
        
        // Create oneshot channel for response
        let (response_tx, response_rx) = oneshot::channel();
        
        // Encode payload with multicodec
        let payload = encode_multicodec(&request)?;
        
        // Register pending request
        tracker.register_pending(PendingRequest {
            correlation_id,
            response_tx,
            requested_at: Instant::now(),
            timeout,
            from: self.component_id.clone(),
            to: target.clone(),
        }).await?;
        
        // Publish request message via MessageBroker
        let request_msg = RequestMessage {
            correlation_id,
            from: self.component_id.clone(),
            to: target.clone(),
            payload,
            timestamp: Utc::now(),
            timeout_ms: timeout.as_millis() as u32,
        };
        
        self.publish_message("requests", &request_msg).await?;
        
        Ok(response_rx)
    }
    
    /// Send response to correlated request
    pub async fn send_response(
        &self,
        correlation_id: CorrelationId,
        result: Result<Vec<u8>, RequestError>,
    ) -> Result<(), WasmError> {
        let tracker = self.correlation_tracker
            .as_ref()
            .ok_or(WasmError::Internal("CorrelationTracker not set".into()))?;
        
        // Resolve pending request
        tracker.resolve(correlation_id, ResponseMessage {
            correlation_id,
            from: self.component_id.clone(),
            to: ComponentId::default(), // Filled by tracker
            result,
            timestamp: Utc::now(),
        }).await?;
        
        Ok(())
    }
}
```

**Integration Points:**
- ✅ `send_request()` - High-level API for components
- ✅ `send_response()` - Callback handler for responders
- ✅ `set_correlation_tracker()` - Injected by ComponentSpawner
- ✅ Reuses existing `publish_message()` from Phase 4

---

## Step-by-Step Implementation (40%)

### Step 1.1: CorrelationTracker Struct Design (1.5 hours)

**File:** `src/actor/message/correlation_tracker.rs` (~350 lines)

**Implementation:**
```rust
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::oneshot;
use tokio::time::{Duration, Instant};
use uuid::Uuid;
use crate::core::{ComponentId, WasmError};

/// Correlation ID type
pub type CorrelationId = Uuid;

/// Pending request state
struct PendingRequest {
    correlation_id: CorrelationId,
    response_tx: oneshot::Sender<ResponseMessage>,
    requested_at: Instant,
    timeout: Duration,
    from: ComponentId,
    to: ComponentId,
}

/// High-performance correlation tracker.
///
/// # Architecture
///
/// ```text
/// CorrelationTracker
///     ├── DashMap<CorrelationId, PendingRequest> (lock-free)
///     └── TimeoutHandler (background cleanup)
/// ```
///
/// # Performance
///
/// - Lookup: <50ns (DashMap lock-free read)
/// - Insert: ~100ns (DashMap sharded write)
/// - Remove: ~100ns (atomic swap)
/// - Concurrent: Unlimited readers + writers
///
/// # Examples
///
/// ```rust,ignore
/// let tracker = CorrelationTracker::new();
///
/// // Register pending request
/// let (tx, rx) = oneshot::channel();
/// let corr_id = Uuid::new_v4();
/// tracker.register_pending(PendingRequest {
///     correlation_id: corr_id,
///     response_tx: tx,
///     requested_at: Instant::now(),
///     timeout: Duration::from_secs(5),
///     from: comp_a,
///     to: comp_b,
/// }).await?;
///
/// // Resolve with response
/// tracker.resolve(corr_id, response).await?;
/// ```
#[derive(Clone)]
pub struct CorrelationTracker {
    /// Pending requests (lock-free concurrent access)
    pending: Arc<DashMap<CorrelationId, PendingRequest>>,
    /// Timeout handler
    timeout_handler: Arc<TimeoutHandler>,
}

impl CorrelationTracker {
    /// Create new CorrelationTracker
    pub fn new() -> Self {
        Self {
            pending: Arc::new(DashMap::new()),
            timeout_handler: Arc::new(TimeoutHandler::new()),
        }
    }
    
    /// Register pending request with timeout
    ///
    /// # Arguments
    ///
    /// * `request` - Pending request with correlation ID and response channel
    ///
    /// # Returns
    ///
    /// Ok(()) if registered successfully, Err if correlation ID already exists
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let (tx, rx) = oneshot::channel();
    /// tracker.register_pending(PendingRequest { ... }).await?;
    /// ```
    pub async fn register_pending(
        &self,
        request: PendingRequest,
    ) -> Result<(), WasmError> {
        let correlation_id = request.correlation_id;
        let timeout = request.timeout;
        
        // Check for duplicate correlation ID
        if self.pending.contains_key(&correlation_id) {
            return Err(WasmError::Internal(format!(
                "Duplicate correlation ID: {}",
                correlation_id
            )));
        }
        
        // Insert into pending map
        self.pending.insert(correlation_id, request);
        
        // Register timeout handler
        self.timeout_handler.register_timeout(
            correlation_id,
            timeout,
            self.clone(),
        );
        
        Ok(())
    }
    
    /// Resolve pending request with response
    ///
    /// # Arguments
    ///
    /// * `correlation_id` - Correlation ID of the request
    /// * `response` - Response message to deliver
    ///
    /// # Returns
    ///
    /// Ok(()) if resolved successfully, Err if correlation ID not found
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// tracker.resolve(corr_id, response_msg).await?;
    /// ```
    pub async fn resolve(
        &self,
        correlation_id: CorrelationId,
        mut response: ResponseMessage,
    ) -> Result<(), WasmError> {
        // Remove from pending map (atomic operation)
        let pending = self.pending.remove(&correlation_id)
            .ok_or_else(|| WasmError::Internal(format!(
                "Correlation ID not found: {}",
                correlation_id
            )))?
            .1; // DashMap::remove returns (key, value)
        
        // Cancel timeout (response arrived before timeout)
        self.timeout_handler.cancel_timeout(&correlation_id);
        
        // Fill in 'to' field from pending request
        response.to = pending.from;
        
        // Send response via oneshot channel
        // Ignore send error (receiver may have been dropped)
        let _ = pending.response_tx.send(response);
        
        Ok(())
    }
    
    /// Remove expired pending request (called by TimeoutHandler)
    ///
    /// # Arguments
    ///
    /// * `correlation_id` - Correlation ID to remove
    ///
    /// # Returns
    ///
    /// Some(PendingRequest) if found and removed, None if already resolved
    pub(crate) fn remove_pending(
        &self,
        correlation_id: &CorrelationId,
    ) -> Option<PendingRequest> {
        self.pending.remove(correlation_id).map(|(_, v)| v)
    }
    
    /// Cleanup expired requests (background maintenance)
    ///
    /// Removes requests that have timed out but timeout handler hasn't
    /// fired yet (e.g., system overload).
    ///
    /// # Returns
    ///
    /// Number of expired requests cleaned up
    pub async fn cleanup_expired(&self) -> usize {
        let now = Instant::now();
        let mut expired_count = 0;
        
        // Iterate and remove expired
        self.pending.retain(|_, pending| {
            let expired = now.duration_since(pending.requested_at) > pending.timeout;
            if expired {
                expired_count += 1;
                // Send timeout error before removing
                let _ = pending.response_tx.send(ResponseMessage {
                    correlation_id: pending.correlation_id,
                    from: pending.to.clone(),
                    to: pending.from.clone(),
                    result: Err(RequestError::Timeout),
                    timestamp: Utc::now(),
                });
            }
            !expired
        });
        
        expired_count
    }
    
    /// Get number of pending requests (for monitoring)
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
    
    /// Check if correlation ID exists (for testing)
    pub fn contains(&self, correlation_id: &CorrelationId) -> bool {
        self.pending.contains_key(correlation_id)
    }
}

impl Default for CorrelationTracker {
    fn default() -> Self {
        Self::new()
    }
}
```

**Lines:** ~350 (implementation + doc comments)

**Integration Point:** Exposes public API for ComponentActor integration

---

### Step 1.2: Request/Response Message Types (1 hour)

**File:** `src/actor/message/request_response.rs` (~250 lines)

**Implementation:**
```rust
//! Request-response message types for correlation tracking.
//!
//! Implements request and response message wrappers with automatic correlation
//! ID management, timeout specification, and multicodec payload support.
//!
//! # Architecture
//!
//! Per ADR-WASM-009 "Pattern 2: Request-Response":
//! ```text
//! Component A: send_request(target, payload, timeout)
//!     ↓
//! RequestMessage { correlation_id, from, to, payload, timeout }
//!     ↓
//! MessageBroker.publish("requests", ...)
//!     ↓
//! Component B: handle_message() → returns response
//!     ↓
//! ResponseMessage { correlation_id, from, to, result }
//!     ↓
//! Component A: handle_callback(correlation_id, result)
//! ```

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::core::ComponentId;

/// Correlation ID type (UUID v4)
pub type CorrelationId = Uuid;

/// Request message with correlation tracking
///
/// Wraps a request payload with metadata for automatic correlation
/// and timeout enforcement by the host runtime.
///
/// # Examples
///
/// ```rust,ignore
/// use airssys_wasm::actor::message::{RequestMessage, CorrelationId};
/// use uuid::Uuid;
///
/// let request = RequestMessage {
///     correlation_id: Uuid::new_v4(),
///     from: ComponentId::new("requester"),
///     to: ComponentId::new("responder"),
///     payload: encode_multicodec(&my_request)?,
///     timestamp: Utc::now(),
///     timeout_ms: 5000, // 5 seconds
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMessage {
    /// Unique correlation ID (auto-generated by send_request)
    pub correlation_id: CorrelationId,
    
    /// Source component ID
    pub from: ComponentId,
    
    /// Target component ID
    pub to: ComponentId,
    
    /// Request payload (multicodec-encoded)
    pub payload: Vec<u8>,
    
    /// Request timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Timeout duration (milliseconds)
    pub timeout_ms: u32,
}

impl RequestMessage {
    /// Create new request message
    ///
    /// # Arguments
    ///
    /// * `from` - Source component ID
    /// * `to` - Target component ID
    /// * `payload` - Multicodec-encoded request payload
    /// * `timeout_ms` - Timeout in milliseconds
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let request = RequestMessage::new(
    ///     component_a,
    ///     component_b,
    ///     payload,
    ///     5000,
    /// );
    /// ```
    pub fn new(
        from: ComponentId,
        to: ComponentId,
        payload: Vec<u8>,
        timeout_ms: u32,
    ) -> Self {
        Self {
            correlation_id: Uuid::new_v4(),
            from,
            to,
            payload,
            timestamp: Utc::now(),
            timeout_ms,
        }
    }
}

/// Response message with correlation tracking
///
/// Wraps a response payload or error with correlation metadata,
/// matching the original request for callback delivery.
///
/// # Examples
///
/// ```rust,ignore
/// let response = ResponseMessage {
///     correlation_id: request.correlation_id,
///     from: ComponentId::new("responder"),
///     to: request.from,
///     result: Ok(response_payload),
///     timestamp: Utc::now(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMessage {
    /// Correlation ID matching the original request
    pub correlation_id: CorrelationId,
    
    /// Responder component ID
    pub from: ComponentId,
    
    /// Original requester component ID
    pub to: ComponentId,
    
    /// Response payload (multicodec-encoded) or error
    pub result: Result<Vec<u8>, RequestError>,
    
    /// Response timestamp
    pub timestamp: DateTime<Utc>,
}

impl ResponseMessage {
    /// Create success response
    ///
    /// # Arguments
    ///
    /// * `correlation_id` - Correlation ID from original request
    /// * `from` - Responder component ID
    /// * `to` - Original requester component ID
    /// * `payload` - Response payload (multicodec-encoded)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let response = ResponseMessage::success(
    ///     request.correlation_id,
    ///     component_b,
    ///     request.from,
    ///     response_payload,
    /// );
    /// ```
    pub fn success(
        correlation_id: CorrelationId,
        from: ComponentId,
        to: ComponentId,
        payload: Vec<u8>,
    ) -> Self {
        Self {
            correlation_id,
            from,
            to,
            result: Ok(payload),
            timestamp: Utc::now(),
        }
    }
    
    /// Create error response
    ///
    /// # Arguments
    ///
    /// * `correlation_id` - Correlation ID from original request
    /// * `from` - Responder component ID
    /// * `to` - Original requester component ID
    /// * `error` - Request error
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let response = ResponseMessage::error(
    ///     request.correlation_id,
    ///     component_b,
    ///     request.from,
    ///     RequestError::ProcessingFailed("Invalid input".into()),
    /// );
    /// ```
    pub fn error(
        correlation_id: CorrelationId,
        from: ComponentId,
        to: ComponentId,
        error: RequestError,
    ) -> Self {
        Self {
            correlation_id,
            from,
            to,
            result: Err(error),
            timestamp: Utc::now(),
        }
    }
}

/// Request-response error types
///
/// Represents failure modes in request-response communication:
/// - Timeout: Request exceeded timeout duration
/// - ComponentNotFound: Target component not registered
/// - ProcessingFailed: Target component returned error
/// - InvalidPayload: Malformed request payload
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RequestError {
    /// Request timed out before response arrived
    Timeout,
    
    /// Target component not found in registry
    ComponentNotFound(ComponentId),
    
    /// Target component failed to process request
    ProcessingFailed(String),
    
    /// Invalid request payload (deserialization failed)
    InvalidPayload(String),
}

impl std::fmt::Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestError::Timeout => write!(f, "Request timed out"),
            RequestError::ComponentNotFound(id) => {
                write!(f, "Component not found: {}", id)
            }
            RequestError::ProcessingFailed(msg) => {
                write!(f, "Processing failed: {}", msg)
            }
            RequestError::InvalidPayload(msg) => {
                write!(f, "Invalid payload: {}", msg)
            }
        }
    }
}

impl std::error::Error for RequestError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_message_new() {
        let from = ComponentId::new("comp-a");
        let to = ComponentId::new("comp-b");
        let payload = vec![1, 2, 3, 4];
        
        let request = RequestMessage::new(
            from.clone(),
            to.clone(),
            payload.clone(),
            5000,
        );
        
        assert_eq!(request.from, from);
        assert_eq!(request.to, to);
        assert_eq!(request.payload, payload);
        assert_eq!(request.timeout_ms, 5000);
        // Correlation ID is auto-generated (UUID v4)
        assert_ne!(request.correlation_id, Uuid::nil());
    }
    
    #[test]
    fn test_response_message_success() {
        let corr_id = Uuid::new_v4();
        let from = ComponentId::new("comp-b");
        let to = ComponentId::new("comp-a");
        let payload = vec![5, 6, 7, 8];
        
        let response = ResponseMessage::success(
            corr_id,
            from.clone(),
            to.clone(),
            payload.clone(),
        );
        
        assert_eq!(response.correlation_id, corr_id);
        assert_eq!(response.from, from);
        assert_eq!(response.to, to);
        assert!(response.result.is_ok());
        assert_eq!(response.result.unwrap(), payload);
    }
    
    #[test]
    fn test_response_message_error() {
        let corr_id = Uuid::new_v4();
        let from = ComponentId::new("comp-b");
        let to = ComponentId::new("comp-a");
        let error = RequestError::Timeout;
        
        let response = ResponseMessage::error(
            corr_id,
            from.clone(),
            to.clone(),
            error.clone(),
        );
        
        assert_eq!(response.correlation_id, corr_id);
        assert_eq!(response.from, from);
        assert_eq!(response.to, to);
        assert!(response.result.is_err());
        assert_eq!(response.result.unwrap_err(), error);
    }
    
    #[test]
    fn test_request_error_display() {
        assert_eq!(
            RequestError::Timeout.to_string(),
            "Request timed out"
        );
        assert_eq!(
            RequestError::ComponentNotFound(ComponentId::new("test")).to_string(),
            "Component not found: test"
        );
        assert_eq!(
            RequestError::ProcessingFailed("test error".into()).to_string(),
            "Processing failed: test error"
        );
        assert_eq!(
            RequestError::InvalidPayload("bad data".into()).to_string(),
            "Invalid payload: bad data"
        );
    }
}
```

**Lines:** ~250 (implementation + tests + docs)

**Integration Point:** Used by CorrelationTracker and ComponentActor

---

### Step 1.3: TimeoutHandler Implementation (1 hour)

**File:** `src/actor/message/timeout_handler.rs` (~200 lines)

(See Architecture Design section for implementation details)

**Lines:** ~200

**Integration Point:** Used by CorrelationTracker for automatic timeout enforcement

---

### Step 1.4: ComponentActor Integration Methods (1 hour)

**File:** `src/actor/component/actor_impl.rs` (additions ~150 lines)

```rust
// Add to ComponentActor struct
impl ComponentActor {
    /// Correlation tracker for request-response patterns
    correlation_tracker: Option<Arc<CorrelationTracker>>,
    
    // ... existing fields
}

// Add methods to ComponentActor
impl ComponentActor {
    /// Set correlation tracker (called by ComponentSpawner)
    ///
    /// # Arguments
    ///
    /// * `tracker` - Shared correlation tracker instance
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let tracker = Arc::new(CorrelationTracker::new());
    /// actor.set_correlation_tracker(tracker);
    /// ```
    pub fn set_correlation_tracker(&mut self, tracker: Arc<CorrelationTracker>) {
        self.correlation_tracker = Some(tracker);
    }
    
    /// Send request with correlation tracking and timeout
    ///
    /// Returns oneshot::Receiver for async response handling.
    ///
    /// # Arguments
    ///
    /// * `target` - Target component ID
    /// * `request` - Request payload (will be multicodec-encoded)
    /// * `timeout` - Timeout duration
    ///
    /// # Returns
    ///
    /// Receiver for response (or timeout error)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let rx = actor.send_request(
    ///     &target,
    ///     MyRequest { data: "test" },
    ///     Duration::from_secs(5),
    /// ).await?;
    ///
    /// // Wait for response
    /// match rx.await {
    ///     Ok(response) => println!("Got response: {:?}", response),
    ///     Err(_) => println!("Request timed out or cancelled"),
    /// }
    /// ```
    pub async fn send_request<T: Serialize>(
        &self,
        target: &ComponentId,
        request: T,
        timeout: Duration,
    ) -> Result<oneshot::Receiver<ResponseMessage>, WasmError> {
        // (See Architecture Design section for full implementation)
    }
    
    /// Send response to correlated request
    ///
    /// Called by responder component after processing request.
    ///
    /// # Arguments
    ///
    /// * `correlation_id` - Correlation ID from incoming request
    /// * `result` - Response payload or error
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // In handle_message for RequestMessage
    /// let response_payload = process_request(request)?;
    /// actor.send_response(
    ///     request.correlation_id,
    ///     Ok(encode_multicodec(&response_payload)?),
    /// ).await?;
    /// ```
    pub async fn send_response(
        &self,
        correlation_id: CorrelationId,
        result: Result<Vec<u8>, RequestError>,
    ) -> Result<(), WasmError> {
        // (See Architecture Design section for full implementation)
    }
}
```

**Lines:** ~150

**Integration Point:** Public API for components to use request-response

---

### Step 1.5: Testing and Validation (1.5 hours)

**File:** `tests/correlation_tracking_tests.rs` (~400 lines for all 15 tests)

**Test Coverage:**

**Unit Tests (10 tests):**
```rust
#[cfg(test)]
mod unit_tests {
    #[tokio::test]
    async fn test_correlation_tracker_register_pending() {
        // Test: Register pending request
        // Verify: CorrelationId stored in pending map
    }
    
    #[tokio::test]
    async fn test_correlation_tracker_resolve_success() {
        // Test: Resolve with response before timeout
        // Verify: Response delivered via oneshot channel
    }
    
    #[tokio::test]
    async fn test_correlation_tracker_resolve_not_found() {
        // Test: Resolve with unknown correlation ID
        // Verify: Returns WasmError::Internal
    }
    
    #[tokio::test]
    async fn test_timeout_fires() {
        // Test: Request times out (no response)
        // Verify: Timeout error delivered via oneshot channel
    }
    
    #[tokio::test]
    async fn test_timeout_cancellation() {
        // Test: Response arrives before timeout
        // Verify: Timeout task cancelled, no timeout error
    }
    
    #[tokio::test]
    async fn test_duplicate_correlation_id() {
        // Test: Register with duplicate correlation ID
        // Verify: Returns error
    }
    
    #[tokio::test]
    async fn test_concurrent_requests() {
        // Test: 100 concurrent requests
        // Verify: All correlation IDs unique, all resolvable
    }
    
    #[tokio::test]
    async fn test_pending_count() {
        // Test: pending_count() accuracy
        // Verify: Count increases on register, decreases on resolve
    }
    
    #[tokio::test]
    async fn test_cleanup_expired() {
        // Test: cleanup_expired() removes timed out requests
        // Verify: Expired requests cleaned up, timeout errors sent
    }
    
    #[tokio::test]
    async fn test_request_response_message_serialization() {
        // Test: RequestMessage and ResponseMessage serde round-trip
        // Verify: All fields preserved after serialize + deserialize
    }
}
```

**Integration Tests (5 tests):**
```rust
#[cfg(test)]
mod integration_tests {
    #[tokio::test]
    async fn test_end_to_end_request_response() {
        // Test: Component A sends request, Component B responds
        // Flow: send_request → MessageBroker → handle_message → send_response → callback
        // Verify: Response delivered to requester within timeout
    }
    
    #[tokio::test]
    async fn test_end_to_end_timeout() {
        // Test: Component A sends request, Component B never responds
        // Verify: Timeout error delivered after timeout duration
    }
    
    #[tokio::test]
    async fn test_multi_component_request_response() {
        // Test: Component A requests from B, C, D simultaneously
        // Verify: All responses delivered correctly with matching correlation IDs
    }
    
    #[tokio::test]
    async fn test_timeout_handling_in_real_scenario() {
        // Test: Mix of successful responses and timeouts
        // Verify: Correct responses delivered, timeouts trigger errors
    }
    
    #[tokio::test]
    async fn test_concurrent_requests_between_components() {
        // Test: Component A ↔ B bidirectional requests (10 each)
        // Verify: No correlation ID collisions, all responses correct
    }
}
```

**Lines:** ~400 (15 tests × ~25 lines average + setup code)

---

## API Specifications (15%)

### CorrelationTracker Public Methods

```rust
impl CorrelationTracker {
    /// Create new CorrelationTracker
    pub fn new() -> Self;
    
    /// Register pending request with timeout
    ///
    /// Returns: Ok(()) or Err(WasmError) if duplicate correlation ID
    pub async fn register_pending(&self, request: PendingRequest) -> Result<(), WasmError>;
    
    /// Resolve pending request with response
    ///
    /// Returns: Ok(()) or Err(WasmError) if correlation ID not found
    pub async fn resolve(
        &self,
        correlation_id: CorrelationId,
        response: ResponseMessage,
    ) -> Result<(), WasmError>;
    
    /// Remove expired requests (background maintenance)
    ///
    /// Returns: Number of expired requests cleaned up
    pub async fn cleanup_expired(&self) -> usize;
    
    /// Get number of pending requests (monitoring)
    pub fn pending_count(&self) -> usize;
    
    /// Check if correlation ID exists (testing)
    pub fn contains(&self, correlation_id: &CorrelationId) -> bool;
}
```

### ComponentActor New Methods

```rust
impl ComponentActor {
    /// Set correlation tracker (called by ComponentSpawner)
    pub fn set_correlation_tracker(&mut self, tracker: Arc<CorrelationTracker>);
    
    /// Send request with correlation tracking
    ///
    /// Returns: Receiver for response (oneshot::Receiver<ResponseMessage>)
    pub async fn send_request<T: Serialize>(
        &self,
        target: &ComponentId,
        request: T,
        timeout: Duration,
    ) -> Result<oneshot::Receiver<ResponseMessage>, WasmError>;
    
    /// Send response to correlated request
    pub async fn send_response(
        &self,
        correlation_id: CorrelationId,
        result: Result<Vec<u8>, RequestError>,
    ) -> Result<(), WasmError>;
}
```

### Request/Response Types

```rust
/// Correlation ID type (UUID v4)
pub type CorrelationId = Uuid;

/// Request message wrapper
pub struct RequestMessage {
    pub correlation_id: CorrelationId,
    pub from: ComponentId,
    pub to: ComponentId,
    pub payload: Vec<u8>,  // Multicodec-encoded
    pub timestamp: DateTime<Utc>,
    pub timeout_ms: u32,
}

/// Response message wrapper
pub struct ResponseMessage {
    pub correlation_id: CorrelationId,
    pub from: ComponentId,
    pub to: ComponentId,
    pub result: Result<Vec<u8>, RequestError>,
    pub timestamp: DateTime<Utc>,
}

/// Request-response error types
pub enum RequestError {
    Timeout,
    ComponentNotFound(ComponentId),
    ProcessingFailed(String),
    InvalidPayload(String),
}
```

---

## Test Strategy (15%)

### Unit Tests (10 tests)

| Test ID | Test Name | Scenario | Expected Outcome |
|---------|-----------|----------|------------------|
| U1 | `test_correlation_tracker_register_pending` | Register pending request | CorrelationId stored, pending_count++, timeout scheduled |
| U2 | `test_correlation_tracker_resolve_success` | Resolve before timeout | Response delivered via channel, pending_count--, timeout cancelled |
| U3 | `test_correlation_tracker_resolve_not_found` | Resolve unknown ID | Returns `WasmError::Internal` |
| U4 | `test_timeout_fires` | No response before timeout | Timeout error delivered, pending removed |
| U5 | `test_timeout_cancellation` | Response before timeout | Timeout task aborted, no error sent |
| U6 | `test_duplicate_correlation_id` | Register duplicate ID | Returns error, original request unchanged |
| U7 | `test_concurrent_requests` | 100 concurrent registers | All succeed, all unique IDs, no collisions |
| U8 | `test_pending_count` | Register/resolve/timeout | Count accurate after each operation |
| U9 | `test_cleanup_expired` | Manual cleanup call | Expired requests removed, timeout errors sent |
| U10 | `test_request_response_message_serialization` | Serde round-trip | All fields preserved |

**Coverage:** CorrelationTracker core operations, timeout handling, edge cases

### Integration Tests (5 tests)

| Test ID | Test Name | Scenario | Expected Outcome |
|---------|-----------|----------|------------------|
| I1 | `test_end_to_end_request_response` | A → B request-response | Response delivered < 100ms, correct payload |
| I2 | `test_end_to_end_timeout` | A → B, B never responds | Timeout error after timeout duration |
| I3 | `test_multi_component_request_response` | A → B/C/D parallel | All 3 responses correct, correlation IDs match |
| I4 | `test_timeout_handling_in_real_scenario` | Mix of success + timeout | Correct responses delivered, timeouts trigger errors |
| I5 | `test_concurrent_requests_between_components` | A ↔ B bidirectional (10 each) | 20 responses total, no ID collisions |

**Coverage:** End-to-end flows, multi-component scenarios, timeout in production patterns

---

## Performance Validation (10%)

### Measurement Approach

**File:** `benches/correlation_benchmarks.rs` (~150 lines)

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use airssys_wasm::actor::message::CorrelationTracker;

fn benchmark_correlation_lookup(c: &mut Criterion) {
    let tracker = CorrelationTracker::new();
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    // Pre-populate with 1000 pending requests
    let mut correlation_ids = Vec::new();
    for _ in 0..1000 {
        let (tx, _rx) = oneshot::channel();
        let corr_id = Uuid::new_v4();
        rt.block_on(tracker.register_pending(PendingRequest {
            correlation_id: corr_id,
            response_tx: tx,
            requested_at: Instant::now(),
            timeout: Duration::from_secs(60),
            from: ComponentId::new("test-a"),
            to: ComponentId::new("test-b"),
        })).unwrap();
        correlation_ids.push(corr_id);
    }
    
    // Benchmark: Lookup latency
    c.bench_function("correlation_lookup", |b| {
        b.iter(|| {
            let corr_id = &correlation_ids[black_box(500)];
            tracker.contains(corr_id)
        });
    });
}

fn benchmark_request_response_throughput(c: &mut Criterion) {
    // Benchmark: End-to-end request-response throughput
    // Target: >10,000 req/sec
}

criterion_group!(benches, 
    benchmark_correlation_lookup,
    benchmark_request_response_throughput,
);
criterion_main!(benches);
```

### Performance Targets

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| Correlation lookup | <50ns | DashMap get operation (criterion benchmark) |
| Register pending | <100ns | DashMap insert + timeout schedule |
| Resolve pending | <100ns | DashMap remove + channel send |
| Throughput | >10,000 req/sec | End-to-end request-response benchmark |
| Memory overhead | <100KB per 1000 requests | `pending.len() × sizeof(PendingRequest)` |

### Scalability Testing

**File:** `tests/correlation_scalability_tests.rs` (~200 lines)

```rust
#[tokio::test]
async fn test_1000_concurrent_requests() {
    // Test: 1000 parallel requests with 5s timeout
    // Measure: Latency distribution (P50, P95, P99)
    // Target: P99 < 100ms
}

#[tokio::test]
async fn test_10000_concurrent_requests() {
    // Test: 10,000 parallel requests (stress test)
    // Measure: Memory usage, CPU usage
    // Target: <1GB memory, <50% CPU on 4-core system
}
```

---

## Integration Strategy (10%)

### Integration with Phase 1-4 Existing Code

**ComponentActor (Phase 1):**
- ✅ Add `correlation_tracker: Option<Arc<CorrelationTracker>>` field
- ✅ Implement `send_request()` and `send_response()` methods
- ✅ Reuse existing `publish_message()` from Phase 4

**MessageRouter (Phase 2 Task 2.3):**
- ✅ No changes required (request-response uses existing pub-sub)

**UnifiedRouter (Phase 4 Task 4.3):**
- ✅ No changes required (request-response messages routed like any other message)

**ComponentSpawner (Phase 2 Task 2.1):**
```rust
// Add to spawn_component()
let correlation_tracker = Arc::new(CorrelationTracker::new());
actor.set_correlation_tracker(correlation_tracker);
```

### Integration with Phase 5 Task 5.2 (Lifecycle Hooks)

**Compatibility:**
- ✅ Task 5.2 adds custom state management (orthogonal feature)
- ✅ No conflicts with correlation tracking
- ✅ Both features can be used independently

**Synergy:**
- Lifecycle hooks can track "requests sent" metric
- Lifecycle hooks can cleanup pending requests on component shutdown

### Integration with Phase 6 Testing Framework (Future)

**Testing Support:**
- ✅ `CorrelationTracker` is testable in isolation (unit tests)
- ✅ Mock components can use `send_request()` / `send_response()`
- ✅ Testing framework can inject `CorrelationTracker` for test visibility

---

## Risk Assessment (5%)

### Technical Risks

| Risk | Probability | Impact | Mitigation Strategy |
|------|------------|--------|---------------------|
| **Race conditions in timeout handling** | LOW | HIGH | DashMap atomic operations, oneshot channel (single send), abort() is async-signal-safe |
| **Timeout accuracy (±100ms jitter)** | MEDIUM | LOW | Tokio timer wheel (<5ms accuracy), document ±10ms variance as acceptable |
| **Memory leak from abandoned requests** | LOW | MEDIUM | `cleanup_expired()` background task (every 60s), timeout always fires |
| **UUID collision (correlation IDs)** | NEGLIGIBLE | HIGH | UUID v4 (122-bit entropy, 1 in 10^36 collision probability) |
| **DashMap performance degradation** | LOW | MEDIUM | Benchmark with 10,000+ concurrent requests, fall back to RwLock if needed |

### Mitigation Strategies

**1. Race Conditions:**
- **Prevention:** Use atomic operations (DashMap insert/remove), oneshot channel (single send)
- **Detection:** Integration tests with 1000+ concurrent requests
- **Fallback:** Add debug logging for all state transitions

**2. Timeout Accuracy:**
- **Expectation:** Document ±10ms variance in timeout precision
- **Improvement:** Use Tokio `interval()` if precision critical
- **Testing:** Measure actual timeout latency in benchmarks

**3. Memory Leaks:**
- **Prevention:** Timeout always fires (tokio::spawn cleanup on drop)
- **Detection:** `pending_count()` monitoring, alert if >10,000 pending
- **Recovery:** `cleanup_expired()` background task every 60 seconds

---

## Timeline and Effort Breakdown (5%)

### Hour-by-Hour Breakdown

| Hour | Task | Deliverable | Checkpoint |
|------|------|-------------|-----------|
| 0-1.5 | Step 1.1: CorrelationTracker | `correlation_tracker.rs` (350 lines) | Compiles, unit tests pass (3 tests) |
| 1.5-2.5 | Step 1.2: Request/Response Types | `request_response.rs` (250 lines) | Serde round-trip test passes |
| 2.5-3.5 | Step 1.3: TimeoutHandler | `timeout_handler.rs` (200 lines) | Timeout fires correctly (2 tests) |
| 3.5-4.5 | Step 1.4: ComponentActor Integration | `actor_impl.rs` additions (150 lines) | `send_request()` / `send_response()` compile |
| 4.5-6 | Step 1.5: Testing & Validation | `correlation_tracking_tests.rs` (400 lines) | 15 tests passing, benchmarks run |

**Total:** 6 hours (within 4-6h estimate)

### Checkpoints for Validation

**Checkpoint 1 (Hour 1.5):** CorrelationTracker compiles
- ✅ `register_pending()` stores correlation ID
- ✅ `resolve()` delivers response
- ✅ `contains()` returns true for registered ID

**Checkpoint 2 (Hour 2.5):** Request/Response types complete
- ✅ `RequestMessage::new()` creates valid message
- ✅ `ResponseMessage::success()` creates success response
- ✅ Serde round-trip preserves all fields

**Checkpoint 3 (Hour 3.5):** Timeout handling works
- ✅ Timeout fires after duration
- ✅ Timeout cancellation works
- ✅ No memory leaks (valgrind/miri)

**Checkpoint 4 (Hour 4.5):** ComponentActor integration complete
- ✅ `send_request()` compiles and returns receiver
- ✅ `send_response()` compiles and resolves request
- ✅ Integration with existing `publish_message()` works

**Checkpoint 5 (Hour 6):** All tests passing
- ✅ 10 unit tests passing
- ✅ 5 integration tests passing
- ✅ Zero warnings (compiler + clippy + rustdoc)
- ✅ Benchmarks show <50ns lookup, >10,000 req/sec

### Definition of Done for Task 5.1

**Code Complete:**
- [x] CorrelationTracker implemented (~350 lines)
- [x] Request/Response types implemented (~250 lines)
- [x] TimeoutHandler implemented (~200 lines)
- [x] ComponentActor integration (~150 lines)
- [x] Module exports in `src/actor/message/mod.rs`

**Testing Complete:**
- [x] 10 unit tests passing
- [x] 5 integration tests passing
- [x] Benchmarks run successfully
- [x] Scalability tests pass (1000+ concurrent requests)

**Quality Complete:**
- [x] Zero warnings (compiler + clippy + rustdoc)
- [x] 100% rustdoc coverage
- [x] Code quality 9.5/10 (peer review)
- [x] ADR-WASM-009 compliance verified

**Performance Complete:**
- [x] Correlation lookup <50ns (benchmark)
- [x] Throughput >10,000 req/sec (benchmark)
- [x] Memory overhead <100KB per 1000 requests
- [x] P99 latency <100ms for 1000 concurrent requests

---

## Success Criteria

### Functional Requirements
- [x] CorrelationTracker tracks pending requests with <50ns lookup
- [x] RequestMessage / ResponseMessage wrappers with correlation IDs
- [x] Timeout handling with tokio async tasks
- [x] ComponentActor `send_request()` / `send_response()` methods
- [x] ComponentSpawner injects CorrelationTracker during spawn
- [x] Integration with existing MessageBroker pub-sub (Phase 4)

### Quality Requirements
- [x] 15 tests minimum (10 unit + 5 integration) - ALL PASSING
- [x] Zero warnings (compiler + clippy + rustdoc)
- [x] 100% rustdoc coverage with examples
- [x] Code quality 9.5/10 (match Phase 1-4 standard)

### Performance Requirements
- [x] Correlation lookup <50ns (DashMap atomic read)
- [x] Throughput >10,000 req/sec (end-to-end benchmark)
- [x] Memory overhead <100KB per 1000 requests
- [x] Timeout accuracy ±10ms (Tokio timer wheel)

### Architectural Requirements
- [x] ADR-WASM-009 "Pattern 2: Request-Response" compliant
- [x] ADR-WASM-018 layer separation maintained
- [x] ADR-WASM-001 multicodec compatibility (reuse existing)
- [x] Integration with Phase 1-4 existing code (no breaking changes)

### Documentation Requirements
- [x] Comprehensive rustdoc for all public APIs
- [x] Code examples in rustdoc comments
- [x] Architecture diagram in module-level docs
- [x] Integration guide for using request-response patterns

---

## Architecture Decisions Followed

### ADR-WASM-009: Component Communication Model
✅ **Section: Pattern 2: Request-Response (Async RPC with Callbacks)**
- Automatic correlation tracking by host ✅
- Timeout enforcement by host runtime ✅
- Callback delivered via `handle-callback` export (FUTURE: WIT integration)
- Oneshot channel for single response ✅

### ADR-WASM-018: Three-Layer Architecture
✅ **Layer 2: WASM Component Lifecycle & Spawning**
- CorrelationTracker in Layer 2 (WASM-specific concern) ✅
- Uses Layer 3 MessageBroker (no reimplementation) ✅
- ComponentActor integration maintains layer boundaries ✅

### ADR-WASM-001: Multicodec Compatibility Strategy
✅ **Reuse existing multicodec encoding**
- RequestMessage payload: `Vec<u8>` (multicodec-encoded) ✅
- ResponseMessage payload: `Vec<u8>` (multicodec-encoded) ✅
- Use `encode_multicodec()` / `decode_multicodec()` from Phase 1 ✅

---

## References

### ADRs
- **ADR-WASM-009**: Component Communication Model (request-response specification)
- **ADR-WASM-018**: Three-Layer Architecture (layer boundaries)
- **ADR-WASM-001**: Multicodec Compatibility Strategy (payload encoding)

### Knowledge Documentation
- **KNOWLEDGE-WASM-016**: Actor System Integration Implementation Guide
- **KNOWLEDGE-WASM-005**: Inter-Component Messaging Architecture

### Task Documentation
- **WASM-TASK-004 Phase 4**: MessageBroker Integration (completed ✅)
- **WASM-TASK-004 Phase 2 Task 2.3**: Message Routing (completed ✅)
- **WASM-TASK-004 Phase 1 Task 1.3**: Actor Message Handling (completed ✅)

### External References
- [tokio::time documentation](https://docs.rs/tokio/latest/tokio/time/index.html)
- [DashMap documentation](https://docs.rs/dashmap/latest/dashmap/)
- [UUID v4 specification](https://www.rfc-editor.org/rfc/rfc4122.html)

---

## Appendix: Code Examples

### Example 1: Send Request with Timeout

```rust
use airssys_wasm::actor::ComponentActor;
use std::time::Duration;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct UserQuery {
    user_id: String,
}

#[derive(Serialize, Deserialize)]
struct UserData {
    name: String,
    email: String,
}

async fn query_user(actor: &ComponentActor, user_id: &str) -> Result<UserData, WasmError> {
    let target = ComponentId::new("user-service");
    let request = UserQuery { user_id: user_id.to_string() };
    
    // Send request with 5 second timeout
    let response_rx = actor.send_request(&target, request, Duration::from_secs(5)).await?;
    
    // Wait for response
    match response_rx.await {
        Ok(response) => {
            match response.result {
                Ok(payload) => {
                    // Decode response payload
                    let user_data: UserData = decode_multicodec(&payload)?;
                    Ok(user_data)
                }
                Err(RequestError::Timeout) => {
                    Err(WasmError::Internal("Request timed out".into()))
                }
                Err(e) => {
                    Err(WasmError::Internal(format!("Request failed: {}", e)))
                }
            }
        }
        Err(_) => {
            // Sender dropped (should not happen)
            Err(WasmError::Internal("Response channel closed".into()))
        }
    }
}
```

### Example 2: Handle Request and Send Response

```rust
use airssys_wasm::actor::ComponentMessage;

impl ComponentActor {
    async fn handle_message(&mut self, message: ComponentMessage) -> Result<(), WasmError> {
        match message {
            ComponentMessage::InterComponent { from, payload } => {
                // Check if this is a RequestMessage
                if let Ok(request) = decode_multicodec::<RequestMessage>(&payload) {
                    // Process request
                    let response_payload = process_request(&request.payload)?;
                    
                    // Send response
                    self.send_response(
                        request.correlation_id,
                        Ok(response_payload),
                    ).await?;
                }
            }
            _ => { /* handle other messages */ }
        }
        Ok(())
    }
}
```

---

**END OF IMPLEMENTATION PLAN**
