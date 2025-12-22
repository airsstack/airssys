//! MessageBroker integration for inter-component communication.
//!
//! This module provides the `MessagingService` which manages the MessageBroker
//! singleton and coordinates runtime-level message routing for inter-component
//! communication in the airssys-wasm framework.
//!
//! # Phase 1 Scope (KNOWLEDGE-WASM-024)
//!
//! - **Direct ComponentId addressing only**: Components are addressed by ComponentId directly
//! - **No topic-based routing**: Topic-based pub-sub is an optional Phase 2+ enhancement
//! - **ActorSystem handles subscriptions**: Runtime-level subscription, not component-level
//! - **Components never subscribe manually**: All routing handled transparently by runtime
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                  MessagingService                        │
//! │  ┌────────────────────────────────────────────────┐     │
//! │  │  • Initialize MessageBroker singleton          │     │
//! │  │  • Provide broker access to ActorSystem        │     │
//! │  │  • Track messaging metrics                     │     │
//! │  └────────────────────────────────────────────────┘     │
//! └─────────────────────────────────────────────────────────┘
//!                         ↓ provides access to
//! ┌─────────────────────────────────────────────────────────┐
//! │         airssys-rt InMemoryMessageBroker                │
//! │  • Pure pub-sub architecture                            │
//! │  • ~211ns routing (proven baseline)                     │
//! │  • 4.7M msg/sec throughput                              │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! # Usage Example
//!
//! ```rust,ignore
//! use airssys_wasm::runtime::MessagingService;
//!
//! // Initialize messaging service
//! let messaging_service = MessagingService::new();
//!
//! // Get broker for ActorSystem subscription
//! let broker = messaging_service.broker();
//!
//! // Subscribe ActorSystem to broker (runtime-level)
//! let mut stream = broker.subscribe().await?;
//!
//! // Messages published to broker are routed to ComponentActors
//! ```
//!
//! # Performance
//!
//! - **MessageBroker routing**: ≤211ns (proven airssys-rt baseline)
//! - **ActorSystem overhead**: ≤9ns (ComponentId lookup + mailbox send)
//! - **Total routing**: ≤220ns (end-to-end)
//! - **Throughput**: ≥4.7M msg/sec (broker capacity)
//!
//! # References
//!
//! - **ADR-WASM-009**: Component Communication Model (PRIMARY REFERENCE)
//! - **KNOWLEDGE-WASM-024**: Component Messaging Clarifications (Phase 1 scope)
//! - **WASM-TASK-006**: Block 5 - Inter-Component Communication (master task)
//! - **RT-TASK-008**: Message Broker Performance Baseline (211ns proven)

// Layer 1: Standard library imports
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

// Layer 2: Third-party crate imports
use serde::{Deserialize, Serialize};

// Layer 3: airssys-rt imports
use airssys_rt::broker::InMemoryMessageBroker;

// Layer 3: Internal crate imports
use crate::actor::message::{CorrelationId, CorrelationTracker, RequestError, ResponseMessage};
use crate::core::{ComponentId, ComponentMessage, WasmError};
use chrono::Utc;

/// Service managing MessageBroker integration for inter-component communication.
///
/// `MessagingService` is responsible for:
/// - Initializing the MessageBroker singleton
/// - Providing broker access to ActorSystem for subscription
/// - Tracking messaging metrics for monitoring
///
/// # Phase 1 Architecture (KNOWLEDGE-WASM-024)
///
/// - **Direct ComponentId addressing**: Components addressed by ComponentId (no topics)
/// - **Runtime-level subscription**: ActorSystem subscribes to broker at initialization
/// - **Component perspective**: Components never subscribe manually - runtime handles routing
///
/// # Thread Safety
///
/// MessagingService is thread-safe via Arc-wrapped broker. All clones share the same
/// MessageBroker instance, enabling concurrent access from multiple threads.
///
/// # Performance
///
/// Built on proven airssys-rt InMemoryMessageBroker:
/// - ~211ns routing latency (RT-TASK-008 baseline)
/// - 4.7M messages/sec throughput
/// - Minimal memory overhead (<50KB per component)
///
/// # Examples
///
/// ```rust,ignore
/// use airssys_wasm::runtime::MessagingService;
///
/// // Initialize service
/// let service = MessagingService::new();
///
/// // Access broker for ActorSystem subscription
/// let broker = service.broker();
///
/// // Get metrics
/// let stats = service.get_stats().await;
/// println!("Messages routed: {}", stats.messages_published);
/// ```
///
/// # References
///
/// - ADR-WASM-009 (lines 270-412): MessageBroker integration architecture
/// - KNOWLEDGE-WASM-024 (lines 17-183): Phase 1 scope clarifications
#[derive(Clone)]
pub struct MessagingService {
    /// Shared MessageBroker instance for all components
    broker: Arc<InMemoryMessageBroker<ComponentMessage>>,
    
    /// Correlation tracker for request-response patterns
    correlation_tracker: Arc<CorrelationTracker>,
    
    /// Metrics for monitoring messaging activity
    metrics: Arc<MessagingMetrics>,
    
    /// Response router for request-response pattern (Phase 3 Task 3.2)
    response_router: Arc<ResponseRouter>,
}

impl MessagingService {
    /// Create a new MessagingService with initialized broker.
    ///
    /// Initializes the airssys-rt InMemoryMessageBroker singleton which will be
    /// used for all inter-component message routing. The broker uses pure pub-sub
    /// architecture where the ActorSystem subscribes at runtime initialization.
    ///
    /// # Phase 1 Design
    ///
    /// - Creates MessageBroker for direct ComponentId addressing
    /// - No topic routing infrastructure (optional Phase 2+ enhancement)
    /// - ActorSystem will subscribe to this broker (runtime-level subscription)
    ///
    /// # Performance
    ///
    /// - Initialization: <1μs (simple Arc allocation)
    /// - Memory overhead: ~10KB base + ~40KB per subscriber
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let service = MessagingService::new();
    /// assert_eq!(service.get_stats().await.messages_published, 0);
    /// ```
    pub fn new() -> Self {
        let correlation_tracker = Arc::new(CorrelationTracker::new());
        let response_router = Arc::new(ResponseRouter::new(Arc::clone(&correlation_tracker)));
        Self {
            broker: Arc::new(InMemoryMessageBroker::new()),
            correlation_tracker,
            metrics: Arc::new(MessagingMetrics::default()),
            response_router,
        }
    }
    
    /// Get reference to the MessageBroker.
    ///
    /// Returns an Arc-wrapped MessageBroker that can be shared across threads.
    /// The ActorSystem uses this to subscribe to all published messages at
    /// runtime initialization.
    ///
    /// # Usage Pattern
    ///
    /// ```text
    /// 1. WasmEngine initializes MessagingService
    /// 2. ComponentRegistry gets broker via this method
    /// 3. ActorSystemSubscriber subscribes to broker
    /// 4. Components publish messages → broker → ActorSystem routes
    /// ```
    ///
    /// # Thread Safety
    ///
    /// The returned Arc can be cloned and passed to multiple threads. All clones
    /// reference the same MessageBroker instance, ensuring consistent routing.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let service = MessagingService::new();
    /// let broker = service.broker();
    ///
    /// // Subscribe ActorSystem (runtime-level)
    /// let mut stream = broker.subscribe().await?;
    /// ```
    pub fn broker(&self) -> Arc<InMemoryMessageBroker<ComponentMessage>> {
        Arc::clone(&self.broker)
    }
    
    /// Get reference to the CorrelationTracker.
    ///
    /// Returns an Arc-wrapped CorrelationTracker for request-response correlation
    /// tracking. Used by the `send-request` host function to register pending
    /// requests with automatic timeout handling.
    ///
    /// # Usage Pattern
    ///
    /// ```text
    /// 1. SendRequestHostFunction calls correlation_tracker()
    /// 2. Registers PendingRequest with oneshot response channel
    /// 3. Publishes request message via broker
    /// 4. Waits for response or timeout
    /// 5. Response delivered via oneshot channel
    /// ```
    ///
    /// # Thread Safety
    ///
    /// The returned Arc can be cloned and passed to multiple threads. All clones
    /// reference the same CorrelationTracker instance.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let service = MessagingService::new();
    /// let tracker = service.correlation_tracker();
    ///
    /// // Register pending request
    /// tracker.register_pending(pending).await?;
    /// ```
    ///
    /// # References
    ///
    /// - ADR-WASM-009 (Pattern 2): Request-Response with correlation tracking
    /// - WASM-TASK-006 Phase 3 Task 3.1: send-request Host Function
    pub fn correlation_tracker(&self) -> Arc<CorrelationTracker> {
        Arc::clone(&self.correlation_tracker)
    }
    
    /// Get current messaging statistics.
    ///
    /// Returns a snapshot of messaging metrics including messages published,
    /// subscribers active, and routing performance. Statistics are thread-safe
    /// via atomic operations.
    ///
    /// # Metrics Tracked
    ///
    /// - `messages_published`: Total messages published to broker
    /// - `active_subscribers`: Current number of subscribers (ActorSystem + monitors)
    /// - `routing_failures`: Count of failed routing attempts
    ///
    /// # Performance
    ///
    /// - Overhead: <50ns (atomic loads only)
    /// - No locks acquired
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let service = MessagingService::new();
    /// let stats = service.get_stats().await;
    ///
    /// println!("Messages published: {}", stats.messages_published);
    /// println!("Active subscribers: {}", stats.active_subscribers);
    /// ```
    pub async fn get_stats(&self) -> MessagingStats {
        MessagingStats {
            messages_published: self.metrics.messages_published.load(Ordering::Relaxed),
            active_subscribers: self.broker.subscriber_count().await,
            routing_failures: self.metrics.routing_failures.load(Ordering::Relaxed),
            requests_sent: self.metrics.requests_sent.load(Ordering::Relaxed),
            requests_pending: self.metrics.requests_pending.load(Ordering::Relaxed),
            responses_routed: self.response_router.responses_routed_count(),
        }
    }
    
    /// Record a message publication (internal use by broker integration).
    ///
    /// This method is called internally when messages are published through the
    /// broker. It increments the messages_published counter for monitoring.
    ///
    /// # Thread Safety
    ///
    /// Uses atomic operations for lock-free metric updates.
    #[doc(hidden)]
    #[allow(dead_code)] // Phase 2: Will be used by ActorSystemSubscriber
    pub(crate) fn record_publish(&self) {
        self.metrics.messages_published.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record a routing failure (internal use by ActorSystemSubscriber).
    ///
    /// This method is called when message routing fails (e.g., component not found).
    /// It increments the routing_failures counter for monitoring and alerting.
    ///
    /// # Thread Safety
    ///
    /// Uses atomic operations for lock-free metric updates.
    #[doc(hidden)]
    #[allow(dead_code)] // Phase 2: Will be used by UnifiedRouter
    pub(crate) fn record_routing_failure(&self) {
        self.metrics.routing_failures.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record a request sent (internal use by SendRequestHostFunction).
    ///
    /// Increments requests_sent counter and pending requests count.
    ///
    /// # Thread Safety
    ///
    /// Uses atomic operations for lock-free metric updates.
    #[doc(hidden)]
    pub(crate) fn record_request_sent(&self) {
        self.metrics.requests_sent.fetch_add(1, Ordering::Relaxed);
        self.metrics.requests_pending.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record a request completed (response received or timeout).
    ///
    /// Decrements pending requests count.
    ///
    /// # Thread Safety
    ///
    /// Uses atomic operations for lock-free metric updates.
    #[doc(hidden)]
    #[allow(dead_code)] // Phase 3 Task 3.2: Will be used by response handler
    pub(crate) fn record_request_completed(&self) {
        // Saturating sub to prevent underflow
        let current = self.metrics.requests_pending.load(Ordering::Relaxed);
        if current > 0 {
            self.metrics.requests_pending.fetch_sub(1, Ordering::Relaxed);
        }
    }
    
    /// Get count of pending requests.
    ///
    /// Returns current number of requests awaiting responses.
    #[doc(hidden)]
    #[allow(dead_code)] // Phase 3: Will be used by monitoring
    pub(crate) fn pending_requests(&self) -> u64 {
        self.metrics.requests_pending.load(Ordering::Relaxed)
    }
    
    /// Get reference to the ResponseRouter.
    ///
    /// Returns an Arc-wrapped ResponseRouter for routing responses from
    /// `handle-message` return values back to requesters via `handle-callback`.
    ///
    /// # Usage Pattern
    ///
    /// ```text
    /// 1. Component A calls send-request → pending registered in tracker
    /// 2. Component B receives message → handle-message invoked
    /// 3. Component B returns response → response_router.route_response()
    /// 4. Component A receives callback → handle-callback invoked
    /// ```
    ///
    /// # Thread Safety
    ///
    /// The returned Arc can be cloned and passed to multiple threads. All clones
    /// reference the same ResponseRouter instance.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let service = MessagingService::new();
    /// let router = service.response_router();
    ///
    /// // Route response from handle-message return value
    /// router.route_response(correlation_id, Ok(payload)).await?;
    /// ```
    ///
    /// # References
    ///
    /// - KNOWLEDGE-WASM-029: Messaging Patterns (no send-response; response IS return value)
    /// - WASM-TASK-006 Phase 3 Task 3.2: Response Routing and Callbacks
    pub fn response_router(&self) -> Arc<ResponseRouter> {
        Arc::clone(&self.response_router)
    }
}

impl Default for MessagingService {
    fn default() -> Self {
        Self::new()
    }
}

/// Messaging metrics for monitoring.
///
/// Tracks messaging activity using atomic counters for thread-safe, lock-free
/// metric collection. Metrics are exposed via `MessagingService::get_stats()`.
///
/// # Thread Safety
///
/// All counters use AtomicU64 with Relaxed ordering, providing:
/// - Lock-free updates
/// - Eventually consistent reads
/// - Minimal performance overhead (<10ns per update)
///
/// # Metrics
///
/// - `messages_published`: Total messages published to broker
/// - `routing_failures`: Count of failed routing attempts (component not found)
///
/// # Examples
///
/// ```rust,ignore
/// let metrics = MessagingMetrics::default();
/// metrics.messages_published.fetch_add(1, Ordering::Relaxed);
/// ```
#[derive(Debug, Default)]
struct MessagingMetrics {
    /// Total messages published to broker
    messages_published: AtomicU64,
    
    /// Total routing failures (component not found, mailbox full, etc.)
    routing_failures: AtomicU64,
    
    /// Total request-response messages sent (Phase 3 Task 3.1)
    requests_sent: AtomicU64,
    
    /// Current number of pending requests awaiting responses
    requests_pending: AtomicU64,
}

/// Snapshot of messaging statistics.
///
/// Represents a point-in-time view of messaging metrics. Statistics are
/// eventually consistent due to Relaxed atomic ordering.
///
/// # Examples
///
/// ```rust,ignore
/// let stats = messaging_service.get_stats().await;
/// println!("Messages: {}, Subscribers: {}, Failures: {}",
///     stats.messages_published,
///     stats.active_subscribers,
///     stats.routing_failures
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagingStats {
    /// Total messages published to broker
    pub messages_published: u64,
    
    /// Current number of active subscribers
    pub active_subscribers: usize,
    
    /// Total routing failures
    pub routing_failures: u64,
    
    /// Total request-response messages sent (Phase 3 Task 3.1)
    pub requests_sent: u64,
    
    /// Current number of pending requests awaiting responses
    pub requests_pending: u64,
    
    /// Total responses routed successfully (Phase 3 Task 3.2)
    pub responses_routed: u64,
}

/// Response router for request-response messaging pattern.
///
/// `ResponseRouter` handles routing responses from `handle-message` return values
/// back to requesting components via `handle-callback`. It implements the core
/// pattern defined in KNOWLEDGE-WASM-029:
///
/// - **No `send-response` host function**: Response IS the return value from `handle-message`
/// - **Correlation-based routing**: Uses `CorrelationTracker` to match responses to requests
/// - **Callback invocation**: Routes response to requester via `handle-callback` export
///
/// # Architecture
///
/// ```text
/// Component A                   Component B
/// send-request ──────────────► handle-message
///       │                            │
///       │ correlation_id             │ return value
///       ▼                            ▼
/// CorrelationTracker           ResponseRouter
///       │                            │
///       │◄────── route_response ─────┘
///       │
///       ▼
/// handle-callback ◄───────── response routed
/// ```
///
/// # Thread Safety
///
/// ResponseRouter is thread-safe via Arc-wrapped CorrelationTracker with DashMap.
/// All operations are lock-free with O(1) complexity.
///
/// # Performance
///
/// - Response routing: ~150ns (DashMap lookup + oneshot send)
/// - Callback invocation: ~300ns (WASM export call)
/// - Total: ~450ns end-to-end response delivery
///
/// # References
///
/// - **KNOWLEDGE-WASM-029**: Messaging Patterns (response IS return value)
/// - **ADR-WASM-009**: Component Communication Model (Pattern 2: Request-Response)
/// - **WASM-TASK-006 Phase 3 Task 3.2**: Response Routing and Callbacks
#[derive(Clone)]
pub struct ResponseRouter {
    /// Correlation tracker for pending request lookup
    correlation_tracker: Arc<CorrelationTracker>,
    
    /// Metrics for monitoring response routing
    metrics: Arc<ResponseRouterMetrics>,
}

/// Metrics for response routing.
#[derive(Debug, Default)]
struct ResponseRouterMetrics {
    /// Total responses routed successfully
    responses_routed: AtomicU64,
    
    /// Responses that failed to route (no pending request)
    responses_orphaned: AtomicU64,
    
    /// Responses that were error results
    error_responses: AtomicU64,
}

impl ResponseRouter {
    /// Create a new ResponseRouter with the given correlation tracker.
    ///
    /// # Arguments
    ///
    /// * `correlation_tracker` - Shared correlation tracker for request-response matching
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use airssys_wasm::runtime::ResponseRouter;
    /// use airssys_wasm::actor::message::CorrelationTracker;
    /// use std::sync::Arc;
    ///
    /// let tracker = Arc::new(CorrelationTracker::new());
    /// let router = ResponseRouter::new(tracker);
    /// ```
    pub fn new(correlation_tracker: Arc<CorrelationTracker>) -> Self {
        Self {
            correlation_tracker,
            metrics: Arc::new(ResponseRouterMetrics::default()),
        }
    }
    
    /// Route a response to the requesting component.
    ///
    /// Looks up the pending request by correlation ID and delivers the response
    /// via the oneshot channel established during `send-request`. The
    /// CorrelationTracker handles channel delivery and cleanup.
    ///
    /// # Arguments
    ///
    /// * `correlation_id` - Correlation ID from the original request
    /// * `result` - Response result (Ok for success payload, Err for error)
    /// * `from` - Component ID that produced the response
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Response routed successfully
    /// * `Err(WasmError)` - Routing failed (no pending request, already resolved)
    ///
    /// # Errors
    ///
    /// - `WasmError::Internal` - Correlation ID not found (already resolved or timeout)
    ///
    /// # Performance
    ///
    /// ~150ns (DashMap lookup + oneshot send)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let router = messaging_service.response_router();
    ///
    /// // After handle-message returns, route the response
    /// router.route_response(
    ///     correlation_id,
    ///     Ok(response_payload),
    ///     ComponentId::new("responder"),
    /// ).await?;
    /// ```
    pub async fn route_response(
        &self,
        correlation_id: CorrelationId,
        result: Result<Vec<u8>, RequestError>,
        from: ComponentId,
    ) -> Result<(), WasmError> {
        // Track error responses
        if result.is_err() {
            self.metrics.error_responses.fetch_add(1, Ordering::Relaxed);
        }
        
        // Create ResponseMessage
        let response = ResponseMessage {
            correlation_id,
            from,
            to: ComponentId::new(""), // Will be filled by CorrelationTracker::resolve()
            result,
            timestamp: Utc::now(),
        };
        
        // Resolve via correlation tracker (delivers to oneshot channel)
        match self.correlation_tracker.resolve(correlation_id, response).await {
            Ok(()) => {
                self.metrics.responses_routed.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
            Err(e) => {
                self.metrics.responses_orphaned.fetch_add(1, Ordering::Relaxed);
                Err(e)
            }
        }
    }
    
    /// Check if a correlation ID has a pending request.
    ///
    /// Useful for determining whether a response should be routed or ignored.
    /// Fire-and-forget messages won't have pending requests.
    ///
    /// # Arguments
    ///
    /// * `correlation_id` - Correlation ID to check
    ///
    /// # Returns
    ///
    /// `true` if there's a pending request for this correlation ID
    pub fn has_pending_request(&self, correlation_id: &CorrelationId) -> bool {
        self.correlation_tracker.contains(correlation_id)
    }
    
    /// Get the number of responses routed successfully.
    pub fn responses_routed_count(&self) -> u64 {
        self.metrics.responses_routed.load(Ordering::Relaxed)
    }
    
    /// Get the number of orphaned responses (no pending request).
    pub fn responses_orphaned_count(&self) -> u64 {
        self.metrics.responses_orphaned.load(Ordering::Relaxed)
    }
    
    /// Get the number of error responses.
    pub fn error_responses_count(&self) -> u64 {
        self.metrics.error_responses.load(Ordering::Relaxed)
    }
    
    /// Get snapshot of response router metrics.
    pub fn get_stats(&self) -> ResponseRouterStats {
        ResponseRouterStats {
            responses_routed: self.metrics.responses_routed.load(Ordering::Relaxed),
            responses_orphaned: self.metrics.responses_orphaned.load(Ordering::Relaxed),
            error_responses: self.metrics.error_responses.load(Ordering::Relaxed),
        }
    }
}

/// Snapshot of response router statistics.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ResponseRouterStats {
    /// Total responses routed successfully
    pub responses_routed: u64,
    
    /// Responses that failed to route (no pending request)
    pub responses_orphaned: u64,
    
    /// Responses that were error results
    pub error_responses: u64,
}

/// Message reception metrics for ComponentActor (WASM-TASK-006 Task 1.2).
///
/// Tracks metrics specific to message reception and delivery to WASM components,
/// including backpressure events, timeouts, and processing latency. All counters
/// use atomic operations for thread-safe, lock-free updates.
///
/// # Architecture
///
/// Each ComponentActor maintains its own MessageReceptionMetrics instance to track
/// per-component reception behavior. These metrics are essential for:
/// - Monitoring component health and performance
/// - Detecting backpressure conditions
/// - Identifying slow or failing components
/// - Capacity planning and resource allocation
///
/// # Thread Safety
///
/// All counters use AtomicU64 with Relaxed ordering, providing:
/// - Lock-free updates (<10ns overhead)
/// - Eventually consistent reads
/// - Safe concurrent access from multiple threads
///
/// # Performance Impact
///
/// Total overhead per message: ~50ns
/// - messages_received increment: ~10ns
/// - Conditional backpressure check: ~20ns
/// - Conditional timeout check: ~20ns
///
/// # Examples
///
/// ```rust,ignore
/// use airssys_wasm::runtime::MessageReceptionMetrics;
///
/// let metrics = MessageReceptionMetrics::default();
///
/// // Record successful message reception
/// metrics.record_message_received();
///
/// // Record backpressure event
/// metrics.record_backpressure_drop();
///
/// // Get snapshot of metrics
/// let stats = metrics.snapshot();
/// println!("Received: {}, Dropped: {}, Timeouts: {}",
///     stats.messages_received,
///     stats.backpressure_drops,
///     stats.delivery_timeouts
/// );
/// ```
///
/// # References
///
/// - WASM-TASK-006 Phase 1 Task 1.2: Message reception infrastructure
/// - Performance targets: >10,000 msg/sec per component, <20ns delivery latency
#[derive(Debug, Default)]
pub struct MessageReceptionMetrics {
    /// Total messages successfully received and processed
    pub messages_received: AtomicU64,
    
    /// Messages dropped due to backpressure (mailbox full)
    pub backpressure_drops: AtomicU64,
    
    /// Messages that timed out during WASM export invocation
    pub delivery_timeouts: AtomicU64,
    
    /// WASM export invocation errors (traps, not found, etc.)
    pub delivery_errors: AtomicU64,
    
    /// Current estimated queue depth (in-flight messages)
    pub current_queue_depth: AtomicU64,
}

impl MessageReceptionMetrics {
    /// Create new metrics with all counters at zero.
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Record successful message reception.
    ///
    /// Increments messages_received counter atomically.
    ///
    /// # Performance
    ///
    /// ~10ns overhead (single atomic fetch_add)
    #[inline]
    pub fn record_message_received(&self) {
        self.messages_received.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record backpressure drop event.
    ///
    /// Increments backpressure_drops counter atomically.
    ///
    /// # Performance
    ///
    /// ~10ns overhead (single atomic fetch_add)
    #[inline]
    pub fn record_backpressure_drop(&self) {
        self.backpressure_drops.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record delivery timeout.
    ///
    /// Increments delivery_timeouts counter atomically.
    ///
    /// # Performance
    ///
    /// ~10ns overhead (single atomic fetch_add)
    #[inline]
    pub fn record_delivery_timeout(&self) {
        self.delivery_timeouts.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record delivery error.
    ///
    /// Increments delivery_errors counter atomically.
    ///
    /// # Performance
    ///
    /// ~10ns overhead (single atomic fetch_add)
    #[inline]
    pub fn record_delivery_error(&self) {
        self.delivery_errors.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Update current queue depth estimate.
    ///
    /// Sets current_queue_depth to the provided value atomically.
    ///
    /// # Parameters
    ///
    /// * `depth` - Current number of in-flight messages
    ///
    /// # Performance
    ///
    /// ~5ns overhead (single atomic store)
    #[inline]
    pub fn set_queue_depth(&self, depth: u64) {
        self.current_queue_depth.store(depth, Ordering::Relaxed);
    }
    
    /// Get current queue depth estimate.
    ///
    /// Returns the last recorded queue depth value.
    ///
    /// # Performance
    ///
    /// ~3ns overhead (single atomic load)
    #[inline]
    pub fn get_queue_depth(&self) -> u64 {
        self.current_queue_depth.load(Ordering::Relaxed)
    }
    
    /// Get snapshot of all metrics.
    ///
    /// Returns MessageReceptionStats with point-in-time values of all counters.
    ///
    /// # Performance
    ///
    /// ~30ns overhead (5 atomic loads)
    pub fn snapshot(&self) -> MessageReceptionStats {
        MessageReceptionStats {
            messages_received: self.messages_received.load(Ordering::Relaxed),
            backpressure_drops: self.backpressure_drops.load(Ordering::Relaxed),
            delivery_timeouts: self.delivery_timeouts.load(Ordering::Relaxed),
            delivery_errors: self.delivery_errors.load(Ordering::Relaxed),
            current_queue_depth: self.current_queue_depth.load(Ordering::Relaxed),
        }
    }
}

/// Snapshot of message reception statistics.
///
/// Point-in-time view of MessageReceptionMetrics counters. Values are eventually
/// consistent due to Relaxed atomic ordering.
///
/// # Examples
///
/// ```rust,ignore
/// let metrics = MessageReceptionMetrics::default();
/// metrics.record_message_received();
/// metrics.record_message_received();
///
/// let stats = metrics.snapshot();
/// assert_eq!(stats.messages_received, 2);
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MessageReceptionStats {
    /// Total messages successfully received
    pub messages_received: u64,
    
    /// Messages dropped due to backpressure
    pub backpressure_drops: u64,
    
    /// Messages that timed out during delivery
    pub delivery_timeouts: u64,
    
    /// WASM export invocation errors
    pub delivery_errors: u64,
    
    /// Current estimated queue depth
    pub current_queue_depth: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_messaging_service_new() {
        let service = MessagingService::new();
        
        // Broker should be initialized
        assert_eq!(Arc::strong_count(&service.broker), 1);
        
        // Metrics should start at zero
        assert_eq!(service.metrics.messages_published.load(Ordering::Relaxed), 0);
        assert_eq!(service.metrics.routing_failures.load(Ordering::Relaxed), 0);
    }
    
    #[test]
    fn test_messaging_service_broker_access() {
        let service = MessagingService::new();
        let broker = service.broker();
        
        // Broker should be the same instance
        assert_eq!(Arc::strong_count(&service.broker), 2); // service + broker variable
        
        // Multiple calls should return the same broker
        let broker2 = service.broker();
        assert_eq!(Arc::strong_count(&service.broker), 3);
        
        drop(broker);
        drop(broker2);
        assert_eq!(Arc::strong_count(&service.broker), 1);
    }
    
    #[tokio::test]
    async fn test_messaging_service_stats() {
        let service = MessagingService::new();
        
        // Initial stats should be zero
        let stats = service.get_stats().await;
        assert_eq!(stats.messages_published, 0);
        assert_eq!(stats.active_subscribers, 0);
        assert_eq!(stats.routing_failures, 0);
    }
    
    #[test]
    fn test_record_publish() {
        let service = MessagingService::new();
        
        service.record_publish();
        assert_eq!(service.metrics.messages_published.load(Ordering::Relaxed), 1);
        
        service.record_publish();
        assert_eq!(service.metrics.messages_published.load(Ordering::Relaxed), 2);
    }
    
    #[test]
    fn test_record_routing_failure() {
        let service = MessagingService::new();
        
        service.record_routing_failure();
        assert_eq!(service.metrics.routing_failures.load(Ordering::Relaxed), 1);
        
        service.record_routing_failure();
        assert_eq!(service.metrics.routing_failures.load(Ordering::Relaxed), 2);
    }
    
    #[test]
    fn test_messaging_service_clone() {
        let service = MessagingService::new();
        let service_clone = service.clone();
        
        // Should share same broker and metrics
        assert_eq!(Arc::strong_count(&service.broker), 2);
        assert_eq!(Arc::strong_count(&service.metrics), 2);
        
        // Metrics should be shared
        service.record_publish();
        assert_eq!(service_clone.metrics.messages_published.load(Ordering::Relaxed), 1);
    }
    
    #[test]
    fn test_default_trait() {
        let service1 = MessagingService::new();
        let service2 = MessagingService::default();
        
        // Both should be initialized correctly
        assert_eq!(service1.metrics.messages_published.load(Ordering::Relaxed), 0);
        assert_eq!(service2.metrics.messages_published.load(Ordering::Relaxed), 0);
    }
    
    // ============================================================================
    // Phase 3 Task 3.1 Tests - CorrelationTracker and Request Metrics
    // ============================================================================
    
    #[test]
    fn test_correlation_tracker_access() {
        let service = MessagingService::new();
        let tracker = service.correlation_tracker();
        
        // Tracker should be initialized
        assert_eq!(tracker.pending_count(), 0);
        
        // Multiple calls should return the same tracker
        // Note: strong_count is 4 because ResponseRouter also holds an Arc<CorrelationTracker>
        // service.correlation_tracker + service.response_router.correlation_tracker + tracker + tracker2
        let tracker2 = service.correlation_tracker();
        assert_eq!(Arc::strong_count(&service.correlation_tracker), 4);
        
        drop(tracker);
        drop(tracker2);
        // After dropping both, only service.correlation_tracker and response_router remain
        assert_eq!(Arc::strong_count(&service.correlation_tracker), 2);
    }
    
    #[test]
    fn test_record_request_sent() {
        let service = MessagingService::new();
        
        // Initial values
        assert_eq!(service.metrics.requests_sent.load(Ordering::Relaxed), 0);
        assert_eq!(service.metrics.requests_pending.load(Ordering::Relaxed), 0);
        
        // Record first request
        service.record_request_sent();
        assert_eq!(service.metrics.requests_sent.load(Ordering::Relaxed), 1);
        assert_eq!(service.metrics.requests_pending.load(Ordering::Relaxed), 1);
        
        // Record second request
        service.record_request_sent();
        assert_eq!(service.metrics.requests_sent.load(Ordering::Relaxed), 2);
        assert_eq!(service.metrics.requests_pending.load(Ordering::Relaxed), 2);
    }
    
    #[test]
    fn test_record_request_completed() {
        let service = MessagingService::new();
        
        // Send 2 requests
        service.record_request_sent();
        service.record_request_sent();
        assert_eq!(service.metrics.requests_pending.load(Ordering::Relaxed), 2);
        
        // Complete 1 request
        service.record_request_completed();
        assert_eq!(service.metrics.requests_pending.load(Ordering::Relaxed), 1);
        assert_eq!(service.metrics.requests_sent.load(Ordering::Relaxed), 2); // Unchanged
        
        // Complete remaining request
        service.record_request_completed();
        assert_eq!(service.metrics.requests_pending.load(Ordering::Relaxed), 0);
        
        // Don't go negative
        service.record_request_completed();
        assert_eq!(service.metrics.requests_pending.load(Ordering::Relaxed), 0);
    }
    
    #[test]
    fn test_pending_requests() {
        let service = MessagingService::new();
        
        assert_eq!(service.pending_requests(), 0);
        
        service.record_request_sent();
        assert_eq!(service.pending_requests(), 1);
        
        service.record_request_sent();
        assert_eq!(service.pending_requests(), 2);
        
        service.record_request_completed();
        assert_eq!(service.pending_requests(), 1);
    }
    
    #[tokio::test]
    async fn test_get_stats_includes_request_metrics() {
        let service = MessagingService::new();
        
        // Initial stats
        let stats = service.get_stats().await;
        assert_eq!(stats.requests_sent, 0);
        assert_eq!(stats.requests_pending, 0);
        
        // After sending requests
        service.record_request_sent();
        service.record_request_sent();
        
        let stats = service.get_stats().await;
        assert_eq!(stats.requests_sent, 2);
        assert_eq!(stats.requests_pending, 2);
        
        // After completing one
        service.record_request_completed();
        
        let stats = service.get_stats().await;
        assert_eq!(stats.requests_sent, 2);
        assert_eq!(stats.requests_pending, 1);
    }
    
    // ============================================================================
    // Phase 3 Task 3.2 Tests - ResponseRouter
    // ============================================================================
    
    #[test]
    fn test_response_router_new() {
        let tracker = Arc::new(CorrelationTracker::new());
        let router = ResponseRouter::new(tracker);
        
        // Initial metrics should be zero
        assert_eq!(router.responses_routed_count(), 0);
        assert_eq!(router.responses_orphaned_count(), 0);
        assert_eq!(router.error_responses_count(), 0);
    }
    
    #[test]
    fn test_response_router_clone() {
        let tracker = Arc::new(CorrelationTracker::new());
        let router = ResponseRouter::new(tracker);
        let _router_clone = router.clone();
        
        // Should share same metrics
        assert_eq!(Arc::strong_count(&router.metrics), 2);
        
        // Verify both reference same tracker
        assert_eq!(Arc::strong_count(&router.correlation_tracker), 2);
    }
    
    #[test]
    fn test_response_router_has_pending_request_false() {
        let tracker = Arc::new(CorrelationTracker::new());
        let router = ResponseRouter::new(tracker);
        
        let correlation_id = uuid::Uuid::new_v4();
        assert!(!router.has_pending_request(&correlation_id));
    }
    
    #[tokio::test]
    async fn test_response_router_has_pending_request_true() {
        use crate::actor::message::PendingRequest;
        use tokio::sync::oneshot;
        use tokio::time::{Duration, Instant};
        
        let tracker = Arc::new(CorrelationTracker::new());
        let router = ResponseRouter::new(Arc::clone(&tracker));
        
        let correlation_id = uuid::Uuid::new_v4();
        let (tx, _rx) = oneshot::channel();
        
        let pending = PendingRequest {
            correlation_id,
            response_tx: tx,
            requested_at: Instant::now(),
            timeout: Duration::from_secs(30),
            from: ComponentId::new("requester"),
            to: ComponentId::new("responder"),
        };
        
        tracker.register_pending(pending).await.unwrap();
        
        assert!(router.has_pending_request(&correlation_id));
    }
    
    #[tokio::test]
    async fn test_response_router_route_response_success() {
        use crate::actor::message::PendingRequest;
        use tokio::sync::oneshot;
        use tokio::time::{Duration, Instant};
        
        let tracker = Arc::new(CorrelationTracker::new());
        let router = ResponseRouter::new(Arc::clone(&tracker));
        
        let correlation_id = uuid::Uuid::new_v4();
        let (tx, rx) = oneshot::channel();
        
        let pending = PendingRequest {
            correlation_id,
            response_tx: tx,
            requested_at: Instant::now(),
            timeout: Duration::from_secs(30),
            from: ComponentId::new("requester"),
            to: ComponentId::new("responder"),
        };
        
        tracker.register_pending(pending).await.unwrap();
        
        // Route successful response
        let result = router.route_response(
            correlation_id,
            Ok(vec![1, 2, 3]),
            ComponentId::new("responder"),
        ).await;
        
        assert!(result.is_ok());
        assert_eq!(router.responses_routed_count(), 1);
        assert_eq!(router.responses_orphaned_count(), 0);
        assert_eq!(router.error_responses_count(), 0);
        
        // Verify response was delivered
        let response = rx.await.unwrap();
        assert_eq!(response.correlation_id, correlation_id);
        assert!(response.result.is_ok());
    }
    
    #[tokio::test]
    async fn test_response_router_route_response_error() {
        use crate::actor::message::{PendingRequest, RequestError};
        use tokio::sync::oneshot;
        use tokio::time::{Duration, Instant};
        
        let tracker = Arc::new(CorrelationTracker::new());
        let router = ResponseRouter::new(Arc::clone(&tracker));
        
        let correlation_id = uuid::Uuid::new_v4();
        let (tx, rx) = oneshot::channel();
        
        let pending = PendingRequest {
            correlation_id,
            response_tx: tx,
            requested_at: Instant::now(),
            timeout: Duration::from_secs(30),
            from: ComponentId::new("requester"),
            to: ComponentId::new("responder"),
        };
        
        tracker.register_pending(pending).await.unwrap();
        
        // Route error response
        let result = router.route_response(
            correlation_id,
            Err(RequestError::ComponentNotFound(ComponentId::new("target"))),
            ComponentId::new("responder"),
        ).await;
        
        assert!(result.is_ok());
        assert_eq!(router.responses_routed_count(), 1);
        assert_eq!(router.responses_orphaned_count(), 0);
        assert_eq!(router.error_responses_count(), 1);
        
        // Verify response was delivered as error
        let response = rx.await.unwrap();
        assert!(response.result.is_err());
    }
    
    #[tokio::test]
    async fn test_response_router_orphaned_response() {
        let tracker = Arc::new(CorrelationTracker::new());
        let router = ResponseRouter::new(tracker);
        
        // Try to route response for non-existent request
        let correlation_id = uuid::Uuid::new_v4();
        let result = router.route_response(
            correlation_id,
            Ok(vec![1, 2, 3]),
            ComponentId::new("responder"),
        ).await;
        
        assert!(result.is_err()); // Should fail - no pending request
        assert_eq!(router.responses_routed_count(), 0);
        assert_eq!(router.responses_orphaned_count(), 1);
    }
    
    #[test]
    fn test_response_router_get_stats() {
        let tracker = Arc::new(CorrelationTracker::new());
        let router = ResponseRouter::new(tracker);
        
        let stats = router.get_stats();
        assert_eq!(stats.responses_routed, 0);
        assert_eq!(stats.responses_orphaned, 0);
        assert_eq!(stats.error_responses, 0);
    }
    
    #[test]
    fn test_response_router_access() {
        let service = MessagingService::new();
        let router = service.response_router();
        
        // Router should be initialized
        assert_eq!(router.responses_routed_count(), 0);
        
        // Multiple calls should return the same router
        let router2 = service.response_router();
        assert_eq!(Arc::strong_count(&service.response_router), 3);
        
        drop(router);
        drop(router2);
        assert_eq!(Arc::strong_count(&service.response_router), 1);
    }
    
    #[tokio::test]
    async fn test_get_stats_includes_responses_routed() {
        use crate::actor::message::PendingRequest;
        use tokio::sync::oneshot;
        use tokio::time::{Duration, Instant};
        
        let service = MessagingService::new();
        let tracker = service.correlation_tracker();
        
        // Initial stats
        let stats = service.get_stats().await;
        assert_eq!(stats.responses_routed, 0);
        
        // Register and route a response
        let correlation_id = uuid::Uuid::new_v4();
        let (tx, _rx) = oneshot::channel();
        
        let pending = PendingRequest {
            correlation_id,
            response_tx: tx,
            requested_at: Instant::now(),
            timeout: Duration::from_secs(30),
            from: ComponentId::new("requester"),
            to: ComponentId::new("responder"),
        };
        
        tracker.register_pending(pending).await.unwrap();
        
        let router = service.response_router();
        router.route_response(
            correlation_id,
            Ok(vec![1, 2, 3]),
            ComponentId::new("responder"),
        ).await.unwrap();
        
        let stats = service.get_stats().await;
        assert_eq!(stats.responses_routed, 1);
    }
}
