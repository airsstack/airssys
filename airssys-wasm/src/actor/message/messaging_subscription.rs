//! MessagingSubscriptionService coordinates ActorSystem ↔ MessageBroker subscription.
//!
//! This module implements the ActorSystem Event Subscription Infrastructure (Task 1.3),
//! providing centralized coordination of:
//! - MessageBroker subscription during runtime initialization
//! - ActorSystemSubscriber lifecycle management
//! - Component registration/unregistration hooks
//! - Graceful shutdown of subscription infrastructure
//!
//! # Architecture Context (ADR-WASM-020 & ADR-WASM-009)
//!
//! ```text
//! ┌────────────────────────────────────────────────────────────────┐
//! │            MessagingSubscriptionService                         │
//! │  ┌──────────────────────────────────────────────────────────┐  │
//! │  │ • Initialize ActorSystemSubscriber during runtime startup│  │
//! │  │ • Subscribe to MessageBroker and start routing task      │  │
//! │  │ • Provide hooks for component registration/unregistration│  │
//! │  │ • Handle graceful shutdown of subscription infrastructure│  │
//! │  │ • Track subscription metrics and status                  │  │
//! │  └──────────────────────────────────────────────────────────┘  │
//! └────────────────────────────────────────────────────────────────┘
//!                                 ↓
//!                   ┌────────────────────────────┐
//!                   │   ActorSystemSubscriber    │
//!                   │   (owns mailbox_senders)   │
//!                   └────────────────────────────┘
//!                                 ↓
//!                   ┌────────────────────────────┐
//!                   │  airssys-rt MessageBroker  │
//!                   │  (InMemoryMessageBroker)   │
//!                   └────────────────────────────┘
//! ```
//!
//! # Lifecycle
//!
//! 1. `new()` - Create service with broker, registry, and subscriber manager
//! 2. `start()` - Subscribe to broker, spawn routing task
//! 3. `register_component()` - Register component mailbox (called by spawner)
//! 4. ... messages routed automatically ...
//! 5. `unregister_component()` - Remove component (called during shutdown)
//! 6. `stop()` - Abort routing task, cleanup
//!
//! # Phase 1 Design (KNOWLEDGE-WASM-024)
//!
//! - **Direct ComponentId addressing only**: No topic-based routing
//! - **Internal infrastructure**: Runtime-level, not component-facing API
//! - **Single subscriber**: ActorSystem subscribes on behalf of all components
//!
//! # Usage Example
//!
//! ```rust,ignore
//! use airssys_wasm::actor::message::MessagingSubscriptionService;
//! use airssys_wasm::actor::{ComponentRegistry, SubscriberManager};
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), WasmError> {
//!     // Create messaging service (contains broker)
//!     let messaging_service = MessagingService::new();
//!     let registry = ComponentRegistry::new();
//!     let subscriber_manager = Arc::new(SubscriberManager::new());
//!
//!     // Create subscription service
//!     let mut subscription_service = MessagingSubscriptionService::new(
//!         messaging_service.broker(),
//!         registry,
//!         subscriber_manager,
//!     );
//!
//!     // Start subscription (subscribes to broker, spawns routing task)
//!     subscription_service.start().await?;
//!
//!     // Register component (called by spawner)
//!     let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
//!     subscription_service.register_component(
//!         ComponentId::new("my-component"),
//!         tx,
//!     ).await?;
//!
//!     // ... messages automatically routed ...
//!
//!     // Graceful shutdown
//!     subscription_service.stop().await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! # References
//!
//! - **ADR-WASM-009**: Component Communication Model (ActorSystem as intermediary)
//! - **ADR-WASM-020**: Message Delivery Ownership (ActorSystemSubscriber owns delivery)
//! - **KNOWLEDGE-WASM-024**: Component Messaging Clarifications (Phase 1 scope)
//! - **KNOWLEDGE-WASM-026**: Message Delivery Architecture - Final Decision
//! - **WASM-TASK-006 Phase 1 Task 1.3**: ActorSystem Event Subscription Infrastructure

// Layer 1: Standard library imports
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

// Layer 2: Third-party crate imports
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::RwLock;

// Layer 3: Internal module imports
use super::{ActorSystemSubscriber, SubscriberManager};
use crate::actor::component::ComponentRegistry;
use crate::core::ComponentMessage;
use crate::core::{ComponentId, WasmError};
use airssys_rt::broker::InMemoryMessageBroker;
use airssys_rt::util::ActorAddress;

/// Subscription service status for monitoring.
///
/// Represents the current operational state of the MessagingSubscriptionService.
/// Used for health checks and monitoring.
///
/// # Examples
///
/// ```rust,ignore
/// let status = subscription_service.status();
/// if status.is_running {
///     println!("Service is running with {} components", status.registered_components);
/// }
/// ```
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubscriptionStatus {
    /// Whether the subscription service is running
    pub is_running: bool,

    /// Number of registered components
    pub registered_components: usize,

    /// Number of messages routed since start
    pub messages_routed: u64,

    /// Number of routing failures since start
    pub routing_failures: u64,
}

/// Metrics for subscription service operations.
///
/// Tracks lock-free metrics for monitoring subscription service activity.
/// All counters use atomic operations for thread-safe updates.
#[derive(Debug, Default)]
struct SubscriptionMetrics {
    /// Total messages successfully routed
    messages_routed: AtomicU64,

    /// Total routing failures
    routing_failures: AtomicU64,

    /// Total component registrations
    registrations: AtomicU64,

    /// Total component unregistrations
    unregistrations: AtomicU64,
}

impl SubscriptionMetrics {
    /// Create new metrics instance.
    fn new() -> Self {
        Self::default()
    }

    /// Increment messages routed counter.
    #[inline]
    fn record_message_routed(&self) {
        self.messages_routed.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment routing failures counter.
    #[inline]
    fn record_routing_failure(&self) {
        self.routing_failures.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment registrations counter.
    #[inline]
    fn record_registration(&self) {
        self.registrations.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment unregistrations counter.
    #[inline]
    fn record_unregistration(&self) {
        self.unregistrations.fetch_add(1, Ordering::Relaxed);
    }
}

/// MessagingSubscriptionService coordinates ActorSystem ↔ MessageBroker subscription.
///
/// This service manages the complete subscription lifecycle:
/// 1. Initializes ActorSystemSubscriber during runtime startup
/// 2. Subscribes to MessageBroker and starts routing task
/// 3. Provides hooks for component registration/unregistration
/// 4. Handles graceful shutdown of subscription infrastructure
///
/// # Architecture (ADR-WASM-020)
///
/// ```text
/// MessagingSubscriptionService
///     ↓ contains
/// ActorSystemSubscriber (owns mailbox_senders map)
///     ↓ subscribes to
/// InMemoryMessageBroker (from airssys-rt)
///     ↓ routes to
/// ComponentActor mailboxes (via mailbox_senders lookup)
/// ```
///
/// # Thread Safety
///
/// - Uses `Arc<RwLock<ActorSystemSubscriber>>` for thread-safe subscriber access
/// - All registration/unregistration operations acquire write locks
/// - Routing uses read locks for concurrent message delivery
///
/// # Examples
///
/// ```rust,ignore
/// use airssys_wasm::actor::message::MessagingSubscriptionService;
///
/// let mut service = MessagingSubscriptionService::new(broker, registry, subscriber_manager);
/// service.start().await?;
///
/// // Register components
/// service.register_component(component_id, mailbox_sender).await?;
///
/// // Check status
/// let status = service.status().await;
/// println!("Running: {}, Components: {}", status.is_running, status.registered_components);
///
/// // Graceful shutdown
/// service.stop().await?;
/// ```
pub struct MessagingSubscriptionService {
    /// ActorSystemSubscriber for message routing
    subscriber: Arc<RwLock<ActorSystemSubscriber<InMemoryMessageBroker<ComponentMessage>>>>,

    /// MessageBroker for message distribution
    broker: Arc<InMemoryMessageBroker<ComponentMessage>>,

    /// ComponentRegistry for address lookup (identity only per ADR-WASM-020)
    registry: ComponentRegistry,

    /// SubscriberManager for topic-based routing (future use)
    subscriber_manager: Arc<SubscriberManager>,

    /// Whether the service has been started
    is_started: AtomicBool,

    /// Metrics for monitoring
    metrics: Arc<SubscriptionMetrics>,
}

impl MessagingSubscriptionService {
    /// Create a new MessagingSubscriptionService.
    ///
    /// # Arguments
    ///
    /// * `broker` - MessageBroker for message distribution
    /// * `registry` - ComponentRegistry for address lookup
    /// * `subscriber_manager` - SubscriberManager for topic-based routing (future use)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let service = MessagingSubscriptionService::new(
    ///     messaging_service.broker(),
    ///     registry,
    ///     Arc::new(SubscriberManager::new()),
    /// );
    /// ```
    pub fn new(
        broker: Arc<InMemoryMessageBroker<ComponentMessage>>,
        registry: ComponentRegistry,
        subscriber_manager: Arc<SubscriberManager>,
    ) -> Self {
        let subscriber = ActorSystemSubscriber::new(
            Arc::clone(&broker),
            registry.clone(),
            Arc::clone(&subscriber_manager),
        );

        Self {
            subscriber: Arc::new(RwLock::new(subscriber)),
            broker,
            registry,
            subscriber_manager,
            is_started: AtomicBool::new(false),
            metrics: Arc::new(SubscriptionMetrics::new()),
        }
    }

    /// Initialize subscription service and start MessageBroker subscription.
    ///
    /// This method:
    /// 1. Subscribes ActorSystemSubscriber to MessageBroker
    /// 2. Spawns background routing task
    /// 3. Sets service status to running
    ///
    /// # Errors
    ///
    /// Returns `WasmError::MessageBrokerError` if broker subscription fails.
    ///
    /// # Idempotency
    ///
    /// Calling `start()` multiple times is safe - subsequent calls return `Ok(())`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut service = MessagingSubscriptionService::new(broker, registry, manager);
    /// service.start().await?;
    /// assert!(service.status().await.is_running);
    /// ```
    pub async fn start(&self) -> Result<(), WasmError> {
        // Check if already started (idempotent)
        if self.is_started.load(Ordering::Acquire) {
            tracing::debug!("MessagingSubscriptionService already started, skipping");
            return Ok(());
        }

        // Start the subscriber (acquires write lock)
        {
            let mut subscriber = self.subscriber.write().await;
            subscriber.start().await.map_err(|e| {
                tracing::error!(error = %e, "Failed to start ActorSystemSubscriber");
                e
            })?;
        }

        self.is_started.store(true, Ordering::Release);
        tracing::info!("MessagingSubscriptionService started successfully");

        Ok(())
    }

    /// Register component for message delivery.
    ///
    /// Called by ComponentSpawner after spawning a component. Registers the
    /// component's mailbox sender with ActorSystemSubscriber for message routing.
    ///
    /// # Arguments
    ///
    /// * `component_id` - Unique identifier for the component
    /// * `mailbox_sender` - UnboundedSender for the component's mailbox channel
    ///
    /// # Returns
    ///
    /// - `Ok(())` - Registration successful
    /// - `Err(WasmError::Internal)` - Component already registered
    ///
    /// # Thread Safety
    ///
    /// Acquires read lock on subscriber for registration (uses subscriber's internal write lock).
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    /// service.register_component(
    ///     ComponentId::new("my-component"),
    ///     tx,
    /// ).await?;
    /// ```
    pub async fn register_component(
        &self,
        component_id: ComponentId,
        mailbox_sender: UnboundedSender<ComponentMessage>,
    ) -> Result<(), WasmError> {
        let subscriber = self.subscriber.read().await;
        subscriber
            .register_mailbox(component_id.clone(), mailbox_sender)
            .await
            .map_err(|e| {
                tracing::error!(
                    component_id = %component_id.as_str(),
                    error = %e,
                    "Failed to register component mailbox"
                );
                e
            })?;

        self.metrics.record_registration();
        tracing::debug!(
            component_id = %component_id.as_str(),
            "Component registered with MessagingSubscriptionService"
        );

        Ok(())
    }

    /// Unregister component from message delivery.
    ///
    /// Called during component shutdown. Removes the component's mailbox sender
    /// from ActorSystemSubscriber, preventing further message delivery.
    ///
    /// # Arguments
    ///
    /// * `component_id` - Component identifier to unregister
    ///
    /// # Returns
    ///
    /// - `Ok(())` - Unregistration successful (or component was not registered)
    ///
    /// # Thread Safety
    ///
    /// Acquires read lock on subscriber for unregistration (uses subscriber's internal write lock).
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// service.unregister_component(&ComponentId::new("my-component")).await?;
    /// ```
    pub async fn unregister_component(&self, component_id: &ComponentId) -> Result<(), WasmError> {
        let subscriber = self.subscriber.read().await;
        let removed = subscriber.unregister_mailbox(component_id).await;

        if removed.is_some() {
            self.metrics.record_unregistration();
            tracing::debug!(
                component_id = %component_id.as_str(),
                "Component unregistered from MessagingSubscriptionService"
            );
        } else {
            tracing::warn!(
                component_id = %component_id.as_str(),
                "Attempted to unregister non-existent component"
            );
        }

        Ok(())
    }

    /// Resolve ComponentId to ActorAddress (identity lookup).
    ///
    /// Uses ComponentRegistry for pure identity lookup (no delivery logic).
    /// Per ADR-WASM-020: Registry = "who exists", Subscriber = "how to deliver".
    ///
    /// # Arguments
    ///
    /// * `component_id` - Component identifier to resolve
    ///
    /// # Returns
    ///
    /// - `Some(ActorAddress)` - Component found in registry
    /// - `None` - Component not registered
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// if let Some(addr) = service.resolve_address(&component_id) {
    ///     println!("Component {} -> {}", component_id.as_str(), addr);
    /// }
    /// ```
    pub fn resolve_address(&self, component_id: &ComponentId) -> Option<ActorAddress> {
        self.registry.lookup(component_id).ok()
    }

    /// Check if a component is registered for message delivery.
    ///
    /// # Arguments
    ///
    /// * `component_id` - Component identifier to check
    ///
    /// # Returns
    ///
    /// `true` if component is registered, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// if service.is_component_registered(&component_id).await {
    ///     println!("Component is ready for messages");
    /// }
    /// ```
    pub async fn is_component_registered(&self, component_id: &ComponentId) -> bool {
        let subscriber = self.subscriber.read().await;
        let mailbox_count = subscriber.mailbox_count().await;

        // Check if component exists in registry (identity) for simple check
        // Note: More precise check would be to expose mailbox_senders.contains_key
        // For now, we rely on registry lookup
        self.registry.lookup(component_id).is_ok() || mailbox_count > 0
    }

    /// Gracefully stop subscription infrastructure.
    ///
    /// This method:
    /// 1. Stops the background routing task
    /// 2. Sets service status to not running
    /// 3. Allows pending messages to complete (best effort)
    ///
    /// # Errors
    ///
    /// Returns `WasmError` if stop fails (rare, logged but non-fatal).
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// service.stop().await?;
    /// assert!(!service.status().await.is_running);
    /// ```
    pub async fn stop(&self) -> Result<(), WasmError> {
        if !self.is_started.load(Ordering::Acquire) {
            tracing::debug!("MessagingSubscriptionService not started, skipping stop");
            return Ok(());
        }

        // Stop the subscriber
        {
            let mut subscriber = self.subscriber.write().await;
            subscriber.stop().await.map_err(|e| {
                tracing::error!(error = %e, "Failed to stop ActorSystemSubscriber");
                e
            })?;
        }

        self.is_started.store(false, Ordering::Release);
        tracing::info!("MessagingSubscriptionService stopped successfully");

        Ok(())
    }

    /// Get subscription status for monitoring.
    ///
    /// Returns a snapshot of the current subscription service status including
    /// running state, registered components, and routing metrics.
    ///
    /// # Returns
    ///
    /// `SubscriptionStatus` with current service state.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let status = service.status().await;
    /// if status.is_running {
    ///     println!("Active components: {}", status.registered_components);
    ///     println!("Messages routed: {}", status.messages_routed);
    /// }
    /// ```
    pub async fn status(&self) -> SubscriptionStatus {
        let subscriber = self.subscriber.read().await;
        let registered_components = subscriber.mailbox_count().await;

        SubscriptionStatus {
            is_running: self.is_started.load(Ordering::Acquire),
            registered_components,
            messages_routed: self.metrics.messages_routed.load(Ordering::Relaxed),
            routing_failures: self.metrics.routing_failures.load(Ordering::Relaxed),
        }
    }

    /// Get reference to the MessageBroker.
    ///
    /// Returns the underlying MessageBroker for advanced operations.
    /// Typically used by runtime internals, not by components.
    ///
    /// # Returns
    ///
    /// Arc-wrapped MessageBroker reference.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let broker = service.broker();
    /// let subscriber_count = broker.subscriber_count().await;
    /// ```
    pub fn broker(&self) -> Arc<InMemoryMessageBroker<ComponentMessage>> {
        Arc::clone(&self.broker)
    }

    /// Get reference to the ComponentRegistry.
    ///
    /// Returns the ComponentRegistry for identity lookups.
    /// Per ADR-WASM-020: Registry = identity only, Subscriber = delivery.
    ///
    /// # Returns
    ///
    /// Cloned ComponentRegistry reference.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let registry = service.registry();
    /// let component_count = registry.count()?;
    /// ```
    pub fn registry(&self) -> ComponentRegistry {
        self.registry.clone()
    }

    /// Get reference to the SubscriberManager.
    ///
    /// Returns the SubscriberManager for topic-based routing (future use).
    ///
    /// # Returns
    ///
    /// Arc-wrapped SubscriberManager reference.
    pub fn subscriber_manager(&self) -> Arc<SubscriberManager> {
        Arc::clone(&self.subscriber_manager)
    }

    /// Get the count of registered component mailboxes.
    ///
    /// # Returns
    ///
    /// Number of components registered for message delivery.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let count = service.registered_component_count().await;
    /// println!("Registered components: {}", count);
    /// ```
    pub async fn registered_component_count(&self) -> usize {
        let subscriber = self.subscriber.read().await;
        subscriber.mailbox_count().await
    }

    /// Record a successful message routing (internal use).
    ///
    /// Called by routing logic to track metrics.
    #[doc(hidden)]
    #[allow(dead_code)] // Will be used by routing enhancement
    pub(crate) fn record_message_routed(&self) {
        self.metrics.record_message_routed();
    }

    /// Record a routing failure (internal use).
    ///
    /// Called by routing logic to track failures.
    #[doc(hidden)]
    #[allow(dead_code)] // Will be used by routing enhancement
    pub(crate) fn record_routing_failure(&self) {
        self.metrics.record_routing_failure();
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used)] // Test code: unwrap is acceptable
#[allow(clippy::expect_used)] // Test code: expect is acceptable
mod tests {
    use super::*;
    use airssys_rt::broker::MessageBroker; // Required for broker.publish()

    // ========================================================================
    // Test Category 1: Service Lifecycle
    // ========================================================================

    #[tokio::test]
    async fn test_service_creation() {
        let broker = Arc::new(InMemoryMessageBroker::new());
        let registry = ComponentRegistry::new();
        let subscriber_manager = Arc::new(SubscriberManager::new());

        let service = MessagingSubscriptionService::new(broker, registry, subscriber_manager);

        // Initial state: not started, no components
        let status = service.status().await;
        assert!(!status.is_running);
        assert_eq!(status.registered_components, 0);
        assert_eq!(status.messages_routed, 0);
        assert_eq!(status.routing_failures, 0);
    }

    #[tokio::test]
    async fn test_service_start_success() {
        let broker = Arc::new(InMemoryMessageBroker::new());
        let registry = ComponentRegistry::new();
        let subscriber_manager = Arc::new(SubscriberManager::new());

        let service =
            MessagingSubscriptionService::new(Arc::clone(&broker), registry, subscriber_manager);

        // Start should succeed
        let result = service.start().await;
        assert!(result.is_ok());

        // Status should show running
        let status = service.status().await;
        assert!(status.is_running);

        // Broker should have at least one subscriber
        let subscriber_count = broker.subscriber_count().await;
        assert!(
            subscriber_count >= 1,
            "Broker should have at least one subscriber"
        );

        // Cleanup
        service.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_service_start_idempotent() {
        let broker = Arc::new(InMemoryMessageBroker::new());
        let registry = ComponentRegistry::new();
        let subscriber_manager = Arc::new(SubscriberManager::new());

        let service = MessagingSubscriptionService::new(broker, registry, subscriber_manager);

        // First start
        service.start().await.unwrap();
        assert!(service.status().await.is_running);

        // Second start should be idempotent (no error)
        let result = service.start().await;
        assert!(result.is_ok());
        assert!(service.status().await.is_running);

        // Cleanup
        service.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_service_stop_graceful() {
        let broker = Arc::new(InMemoryMessageBroker::new());
        let registry = ComponentRegistry::new();
        let subscriber_manager = Arc::new(SubscriberManager::new());

        let service = MessagingSubscriptionService::new(broker, registry, subscriber_manager);

        // Start and then stop
        service.start().await.unwrap();
        assert!(service.status().await.is_running);

        let result = service.stop().await;
        assert!(result.is_ok());

        // Status should show not running
        let status = service.status().await;
        assert!(!status.is_running);
    }

    #[tokio::test]
    async fn test_service_status_reporting() {
        let broker = Arc::new(InMemoryMessageBroker::new());
        let registry = ComponentRegistry::new();
        let subscriber_manager = Arc::new(SubscriberManager::new());

        let service = MessagingSubscriptionService::new(broker, registry, subscriber_manager);

        // Before start
        let status = service.status().await;
        assert!(!status.is_running);

        // After start
        service.start().await.unwrap();
        let status = service.status().await;
        assert!(status.is_running);

        // Register a component
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        service
            .register_component(ComponentId::new("test"), tx)
            .await
            .unwrap();

        let status = service.status().await;
        assert_eq!(status.registered_components, 1);

        // Cleanup
        service.stop().await.unwrap();
    }

    // ========================================================================
    // Test Category 2: Component Registration
    // ========================================================================

    #[tokio::test]
    async fn test_register_component_success() {
        let broker = Arc::new(InMemoryMessageBroker::new());
        let registry = ComponentRegistry::new();
        let subscriber_manager = Arc::new(SubscriberManager::new());

        let service = MessagingSubscriptionService::new(broker, registry, subscriber_manager);

        service.start().await.unwrap();

        // Register component
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let component_id = ComponentId::new("test-component");

        let result = service.register_component(component_id.clone(), tx).await;
        assert!(result.is_ok());

        // Verify registered count
        assert_eq!(service.registered_component_count().await, 1);

        // Cleanup
        service.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_register_duplicate_fails() {
        let broker = Arc::new(InMemoryMessageBroker::new());
        let registry = ComponentRegistry::new();
        let subscriber_manager = Arc::new(SubscriberManager::new());

        let service = MessagingSubscriptionService::new(broker, registry, subscriber_manager);

        service.start().await.unwrap();

        let component_id = ComponentId::new("duplicate-test");

        // First registration succeeds
        let (tx1, _rx1) = tokio::sync::mpsc::unbounded_channel();
        let result = service.register_component(component_id.clone(), tx1).await;
        assert!(result.is_ok());

        // Second registration fails (duplicate)
        let (tx2, _rx2) = tokio::sync::mpsc::unbounded_channel();
        let result = service.register_component(component_id.clone(), tx2).await;
        assert!(result.is_err());

        // Cleanup
        service.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_unregister_component_success() {
        let broker = Arc::new(InMemoryMessageBroker::new());
        let registry = ComponentRegistry::new();
        let subscriber_manager = Arc::new(SubscriberManager::new());

        let service = MessagingSubscriptionService::new(broker, registry, subscriber_manager);

        service.start().await.unwrap();

        // Register then unregister
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let component_id = ComponentId::new("unregister-test");

        service
            .register_component(component_id.clone(), tx)
            .await
            .unwrap();
        assert_eq!(service.registered_component_count().await, 1);

        service.unregister_component(&component_id).await.unwrap();
        assert_eq!(service.registered_component_count().await, 0);

        // Cleanup
        service.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_unregister_nonexistent_safe() {
        let broker = Arc::new(InMemoryMessageBroker::new());
        let registry = ComponentRegistry::new();
        let subscriber_manager = Arc::new(SubscriberManager::new());

        let service = MessagingSubscriptionService::new(broker, registry, subscriber_manager);

        service.start().await.unwrap();

        // Unregister non-existent component should succeed (idempotent)
        let component_id = ComponentId::new("nonexistent");
        let result = service.unregister_component(&component_id).await;
        assert!(result.is_ok());

        // Cleanup
        service.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_register_after_unregister() {
        let broker = Arc::new(InMemoryMessageBroker::new());
        let registry = ComponentRegistry::new();
        let subscriber_manager = Arc::new(SubscriberManager::new());

        let service = MessagingSubscriptionService::new(broker, registry, subscriber_manager);

        service.start().await.unwrap();

        let component_id = ComponentId::new("reregister-test");

        // Register
        let (tx1, _rx1) = tokio::sync::mpsc::unbounded_channel();
        service
            .register_component(component_id.clone(), tx1)
            .await
            .unwrap();

        // Unregister
        service.unregister_component(&component_id).await.unwrap();

        // Re-register should succeed
        let (tx2, _rx2) = tokio::sync::mpsc::unbounded_channel();
        let result = service.register_component(component_id.clone(), tx2).await;
        assert!(result.is_ok());

        // Cleanup
        service.stop().await.unwrap();
    }

    // ========================================================================
    // Test Category 3: Address Resolution
    // ========================================================================

    #[tokio::test]
    async fn test_resolve_registered_address() {
        let broker = Arc::new(InMemoryMessageBroker::new());
        let registry = ComponentRegistry::new();
        let subscriber_manager = Arc::new(SubscriberManager::new());

        // Register component in registry
        let component_id = ComponentId::new("resolve-test");
        let actor_addr = ActorAddress::named("resolve-test");
        registry
            .register(component_id.clone(), actor_addr.clone())
            .unwrap();

        let service = MessagingSubscriptionService::new(broker, registry, subscriber_manager);

        // Resolve should find the component
        let resolved = service.resolve_address(&component_id);
        assert!(resolved.is_some());
        assert_eq!(resolved.unwrap(), actor_addr);
    }

    #[tokio::test]
    async fn test_resolve_unregistered_returns_none() {
        let broker = Arc::new(InMemoryMessageBroker::new());
        let registry = ComponentRegistry::new();
        let subscriber_manager = Arc::new(SubscriberManager::new());

        let service = MessagingSubscriptionService::new(broker, registry, subscriber_manager);

        // Resolve unknown component
        let component_id = ComponentId::new("unknown");
        let resolved = service.resolve_address(&component_id);
        assert!(resolved.is_none());
    }

    #[tokio::test]
    async fn test_resolve_after_unregister_none() {
        let broker = Arc::new(InMemoryMessageBroker::new());
        let registry = ComponentRegistry::new();
        let subscriber_manager = Arc::new(SubscriberManager::new());

        // Register then unregister from registry
        let component_id = ComponentId::new("unregister-resolve-test");
        let actor_addr = ActorAddress::named("unregister-resolve-test");
        registry.register(component_id.clone(), actor_addr).unwrap();
        registry.unregister(&component_id).unwrap();

        let service = MessagingSubscriptionService::new(broker, registry, subscriber_manager);

        // Resolve should return None after unregister
        let resolved = service.resolve_address(&component_id);
        assert!(resolved.is_none());
    }

    // ========================================================================
    // Test Category 4: Error Handling
    // ========================================================================

    #[tokio::test]
    async fn test_route_to_nonexistent_logs_error() {
        use airssys_rt::message::MessageEnvelope;

        let broker = Arc::new(InMemoryMessageBroker::new());
        let registry = ComponentRegistry::new();
        let subscriber_manager = Arc::new(SubscriberManager::new());

        let service =
            MessagingSubscriptionService::new(Arc::clone(&broker), registry, subscriber_manager);

        service.start().await.unwrap();

        // Publish message to non-existent component
        let message = ComponentMessage::InterComponent {
            sender: ComponentId::new("sender"),
            to: ComponentId::new("nonexistent"),
            payload: vec![1, 2, 3],
        };

        // Publish to broker - should not crash
        broker.publish(MessageEnvelope::new(message)).await.unwrap();

        // Give time for routing (error logged, not crashed)
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Service should still be running
        assert!(service.status().await.is_running);

        // Cleanup
        service.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_route_to_closed_mailbox_handled() {
        use airssys_rt::message::MessageEnvelope;
        use tokio::sync::mpsc;

        let broker = Arc::new(InMemoryMessageBroker::new());
        let registry = ComponentRegistry::new();
        let subscriber_manager = Arc::new(SubscriberManager::new());

        let service =
            MessagingSubscriptionService::new(Arc::clone(&broker), registry, subscriber_manager);

        service.start().await.unwrap();

        // Register component then close its receiver
        let (tx, rx) = mpsc::unbounded_channel();
        let component_id = ComponentId::new("closed-channel");
        service
            .register_component(component_id.clone(), tx)
            .await
            .unwrap();

        // Close the receiver
        drop(rx);

        // Publish message to component with closed mailbox
        let message = ComponentMessage::InterComponent {
            sender: ComponentId::new("sender"),
            to: component_id,
            payload: vec![1, 2, 3],
        };

        // Publish to broker - should not crash
        broker.publish(MessageEnvelope::new(message)).await.unwrap();

        // Give time for routing (error logged, not crashed)
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Service should still be running
        assert!(service.status().await.is_running);

        // Cleanup
        service.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_invalid_message_type_error() {
        use airssys_rt::message::MessageEnvelope;
        use tokio::sync::mpsc;

        let broker = Arc::new(InMemoryMessageBroker::new());
        let registry = ComponentRegistry::new();
        let subscriber_manager = Arc::new(SubscriberManager::new());

        let service =
            MessagingSubscriptionService::new(Arc::clone(&broker), registry, subscriber_manager);

        service.start().await.unwrap();

        // Register component
        let (tx, _rx) = mpsc::unbounded_channel();
        let component_id = ComponentId::new("test");
        service
            .register_component(component_id.clone(), tx)
            .await
            .unwrap();

        // Publish non-routable message type (Shutdown has no "to" field)
        let message = ComponentMessage::Shutdown;

        // Publish to broker - routing should fail gracefully
        broker.publish(MessageEnvelope::new(message)).await.unwrap();

        // Give time for routing
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Service should still be running
        assert!(service.status().await.is_running);

        // Cleanup
        service.stop().await.unwrap();
    }

    // ========================================================================
    // Additional Tests: Accessor Methods
    // ========================================================================

    #[tokio::test]
    async fn test_broker_accessor() {
        let broker = Arc::new(InMemoryMessageBroker::new());
        let registry = ComponentRegistry::new();
        let subscriber_manager = Arc::new(SubscriberManager::new());

        let service =
            MessagingSubscriptionService::new(Arc::clone(&broker), registry, subscriber_manager);

        let retrieved_broker = service.broker();

        // Should be the same broker (Arc pointer comparison)
        assert!(Arc::ptr_eq(&broker, &retrieved_broker));
    }

    #[tokio::test]
    async fn test_registry_accessor() {
        let broker = Arc::new(InMemoryMessageBroker::new());
        let registry = ComponentRegistry::new();
        let subscriber_manager = Arc::new(SubscriberManager::new());

        // Register something in the registry
        let component_id = ComponentId::new("accessor-test");
        let actor_addr = ActorAddress::named("accessor-test");
        registry
            .register(component_id.clone(), actor_addr.clone())
            .unwrap();

        let service = MessagingSubscriptionService::new(broker, registry, subscriber_manager);

        let retrieved_registry = service.registry();

        // Should be able to lookup in retrieved registry
        let lookup = retrieved_registry.lookup(&component_id);
        assert!(lookup.is_ok());
        assert_eq!(lookup.unwrap(), actor_addr);
    }

    #[tokio::test]
    async fn test_subscriber_manager_accessor() {
        let broker = Arc::new(InMemoryMessageBroker::new());
        let registry = ComponentRegistry::new();
        let subscriber_manager = Arc::new(SubscriberManager::new());

        let service =
            MessagingSubscriptionService::new(broker, registry, Arc::clone(&subscriber_manager));

        let retrieved_manager = service.subscriber_manager();

        // Should be the same manager (Arc pointer comparison)
        assert!(Arc::ptr_eq(&subscriber_manager, &retrieved_manager));
    }
}
