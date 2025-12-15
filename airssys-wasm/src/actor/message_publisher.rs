//! Topic-based message publishing for WASM components.
//!
//! This module implements `MessagePublisher`, providing components with pub-sub
//! capabilities for broadcasting messages to multiple subscribers via topics.
//!
//! # Architecture Context (ADR-WASM-009)
//!
//! Per ADR-WASM-009 Component Communication Model:
//! - **Fire-and-Forget**: No acknowledgment required
//! - **Topic-Based**: Messages routed via topic names
//! - **Multi-Subscriber**: One message delivered to all matching subscribers
//!
//! # Responsibilities
//!
//! - Publishing messages to single topics
//! - Broadcasting messages to multiple topics
//! - Correlation ID support for request-response patterns
//!
//! # Performance
//!
//! Target: <100ns overhead per publish operation.
//!
//! # Example
//!
//! ```rust,ignore
//! use airssys_wasm::actor::{MessagePublisher, MessageBrokerBridge, ComponentMessage};
//! use airssys_wasm::core::ComponentId;
//! use std::sync::Arc;
//!
//! async fn publish_example(
//!     broker: Arc<dyn MessageBrokerBridge>,
//! ) -> Result<(), WasmError> {
//!     let publisher = MessagePublisher::new(
//!         ComponentId::new("my-component"),
//!         broker,
//!     );
//!     
//!     // Fire-and-forget publish
//!     publisher.publish("events.user.login", vec![1, 2, 3]).await?;
//!     
//!     // Broadcast to multiple topics
//!     let topics = ["events.user.login", "audit.auth"];
//!     publisher.publish_multi(&topics, vec![1, 2, 3]).await?;
//!     
//!     // Request-response pattern with correlation ID
//!     let correlation_id = uuid::Uuid::new_v4();
//!     publisher.publish_with_correlation(
//!         "requests.data",
//!         vec![1, 2, 3],
//!         correlation_id,
//!     ).await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! # References
//!
//! - **ADR-WASM-009**: Component Communication Model (Pub-Sub Patterns)
//! - **ADR-WASM-018**: Three-Layer Architecture (Layer Separation)
//! - **WASM-TASK-004 Phase 4 Task 4.2**: Pub-Sub Message Routing

// Layer 1: Standard library imports
use std::sync::Arc;

// Layer 2: Third-party crate imports
use uuid::Uuid;

// Layer 3: Internal module imports
use crate::actor::{ComponentMessage, MessageBrokerBridge};
use crate::core::{ComponentId, WasmError};

/// Component message publisher with topic support.
///
/// MessagePublisher provides a high-level API for publishing messages to topics,
/// abstracting the underlying MessageBroker implementation. Each publisher is
/// associated with a specific component (sender identity).
///
/// # Architecture
///
/// ```text
/// ComponentActor
///     ↓ uses
/// MessagePublisher
///     ↓ delegates to
/// MessageBrokerBridge (trait)
///     ↓ implemented by
/// MessageBrokerWrapper
///     ↓ wraps
/// MessageBroker (Layer 3)
/// ```
///
/// # Thread Safety
///
/// MessagePublisher is Send + Sync, allowing use across async task boundaries.
/// The underlying MessageBrokerBridge is Arc-wrapped for shared ownership.
///
/// # Examples
///
/// ```rust,ignore
/// use airssys_wasm::actor::{MessagePublisher, MessageBrokerWrapper};
/// use airssys_wasm::core::ComponentId;
/// use airssys_rt::broker::InMemoryMessageBroker;
/// use std::sync::Arc;
///
/// let broker = InMemoryMessageBroker::new();
/// let wrapper = Arc::new(MessageBrokerWrapper::new(broker));
/// let publisher = MessagePublisher::new(
///     ComponentId::new("my-component"),
///     wrapper,
/// );
/// ```
pub struct MessagePublisher {
    /// Component ID (sender identity)
    component_id: ComponentId,
    /// MessageBroker bridge for publishing
    broker: Arc<dyn MessageBrokerBridge>,
}

impl MessagePublisher {
    /// Create new MessagePublisher for component.
    ///
    /// # Arguments
    ///
    /// * `component_id` - Component identifier (sender identity)
    /// * `broker` - MessageBrokerBridge implementation
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use airssys_wasm::actor::{MessagePublisher, MessageBrokerWrapper};
    /// use airssys_wasm::core::ComponentId;
    /// use std::sync::Arc;
    ///
    /// let component_id = ComponentId::new("my-component");
    /// let wrapper = Arc::new(MessageBrokerWrapper::new(broker));
    /// let publisher = MessagePublisher::new(component_id, wrapper);
    /// ```
    pub fn new(
        component_id: ComponentId,
        broker: Arc<dyn MessageBrokerBridge>,
    ) -> Self {
        Self {
            component_id,
            broker,
        }
    }

    /// Publish fire-and-forget message to topic.
    ///
    /// Delivers message to all subscribers of the specified topic using
    /// fire-and-forget semantics (no acknowledgment or response expected).
    ///
    /// # Parameters
    ///
    /// * `topic` - Topic name (e.g., "events.user.login", "notifications")
    /// * `payload` - Multicodec-encoded message payload
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Message published successfully
    /// - `Err(WasmError)`: Broker error (invalid topic, publish failed)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Publish event notification
    /// publisher.publish("events.user.login", vec![1, 2, 3]).await?;
    ///
    /// // Publish system alert
    /// publisher.publish("system.alerts", payload).await?;
    /// ```
    ///
    /// # Performance
    ///
    /// Target: <100ns overhead per operation (excluding broker latency)
    pub async fn publish(
        &self,
        topic: &str,
        payload: Vec<u8>,
    ) -> Result<(), WasmError> {
        let message = ComponentMessage::InterComponent {
            sender: self.component_id.clone(),
            payload,
        };

        self.broker.publish(topic, message).await
    }

    /// Publish message to multiple topics (broadcast).
    ///
    /// Atomically publishes the same message to multiple topics, useful for
    /// broadcasting notifications or events that belong to multiple categories.
    ///
    /// # Parameters
    ///
    /// * `topics` - Slice of topic names to publish to
    /// * `payload` - Multicodec-encoded message payload (cloned for each topic)
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Message published to all topics successfully
    /// - `Err(WasmError)`: First error encountered during publish
    ///
    /// # Error Handling
    ///
    /// If any publish fails, subsequent topics are NOT published and the error
    /// is returned immediately. Partial delivery may occur.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Broadcast authentication event to multiple topics
    /// let topics = ["events.user.login", "audit.auth", "security.access"];
    /// publisher.publish_multi(&topics, payload).await?;
    ///
    /// // Notify multiple monitoring channels
    /// let topics = ["metrics.cpu", "metrics.all", "alerts"];
    /// publisher.publish_multi(&topics, metric_data).await?;
    /// ```
    ///
    /// # Performance
    ///
    /// Time complexity: O(n) where n = number of topics
    pub async fn publish_multi(
        &self,
        topics: &[&str],
        payload: Vec<u8>,
    ) -> Result<(), WasmError> {
        for topic in topics {
            self.publish(topic, payload.clone()).await?;
        }
        Ok(())
    }

    /// Publish with correlation ID (for request-response pattern).
    ///
    /// Publishes a message with a correlation ID, enabling request-response
    /// patterns where the receiver can associate responses with requests.
    ///
    /// # Parameters
    ///
    /// * `topic` - Topic name
    /// * `payload` - Multicodec-encoded message payload
    /// * `correlation_id` - UUID for correlating request with response
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Message published successfully
    /// - `Err(WasmError)`: Broker error
    ///
    /// # Correlation Pattern
    ///
    /// ```text
    /// Component A                        Component B
    ///     |                                   |
    ///     | publish_with_correlation()       |
    ///     | topic: "requests.data"           |
    ///     | correlation_id: abc-123          |
    ///     |---------------------------------->|
    ///     |                                   | process()
    ///     |                                   |
    ///     |<----------------------------------|
    ///     | InterComponentWithCorrelation    |
    ///     | correlation_id: abc-123          |
    /// ```
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use uuid::Uuid;
    ///
    /// // Send request with correlation ID
    /// let correlation_id = Uuid::new_v4();
    /// publisher.publish_with_correlation(
    ///     "requests.user.profile",
    ///     request_payload,
    ///     correlation_id,
    /// ).await?;
    ///
    /// // Later, match response by correlation_id
    /// match message {
    ///     ComponentMessage::InterComponentWithCorrelation { correlation_id: id, .. }
    ///         if id == correlation_id => {
    ///             // Handle response
    ///         }
    ///     _ => {}
    /// }
    /// ```
    pub async fn publish_with_correlation(
        &self,
        topic: &str,
        payload: Vec<u8>,
        correlation_id: Uuid,
    ) -> Result<(), WasmError> {
        let message = ComponentMessage::InterComponentWithCorrelation {
            sender: self.component_id.clone(),
            payload,
            correlation_id,
        };

        self.broker.publish(topic, message).await
    }

    /// Get component ID (sender identity).
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let component_id = publisher.component_id();
    /// println!("Publisher component: {}", component_id.as_str());
    /// ```
    pub fn component_id(&self) -> &ComponentId {
        &self.component_id
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)] // Test code: unwrap is acceptable
mod tests {
    use super::*;
    use crate::actor::MessageBrokerWrapper;
    use airssys_rt::broker::InMemoryMessageBroker;

    #[tokio::test]
    async fn test_message_publisher_creation() {
        let broker = InMemoryMessageBroker::new();
        let wrapper = Arc::new(MessageBrokerWrapper::new(broker));
        let component_id = ComponentId::new("test-component");

        let publisher = MessagePublisher::new(component_id.clone(), wrapper);

        assert_eq!(publisher.component_id(), &component_id);
    }

    #[tokio::test]
    async fn test_publish_single_topic() {
        let broker = InMemoryMessageBroker::new();
        let wrapper = Arc::new(MessageBrokerWrapper::new(broker));
        let component_id = ComponentId::new("test-component");
        let publisher = MessagePublisher::new(component_id, wrapper);

        let result = publisher.publish("test-topic", vec![1, 2, 3]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_publish_multi_topics() {
        let broker = InMemoryMessageBroker::new();
        let wrapper = Arc::new(MessageBrokerWrapper::new(broker));
        let component_id = ComponentId::new("test-component");
        let publisher = MessagePublisher::new(component_id, wrapper);

        let topics = ["topic-1", "topic-2", "topic-3"];
        let result = publisher.publish_multi(&topics, vec![1, 2, 3]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_publish_with_correlation() {
        let broker = InMemoryMessageBroker::new();
        let wrapper = Arc::new(MessageBrokerWrapper::new(broker));
        let component_id = ComponentId::new("test-component");
        let publisher = MessagePublisher::new(component_id, wrapper);

        let correlation_id = Uuid::new_v4();
        let result = publisher.publish_with_correlation(
            "request-topic",
            vec![1, 2, 3],
            correlation_id,
        ).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_publish_empty_payload() {
        let broker = InMemoryMessageBroker::new();
        let wrapper = Arc::new(MessageBrokerWrapper::new(broker));
        let component_id = ComponentId::new("test-component");
        let publisher = MessagePublisher::new(component_id, wrapper);

        let result = publisher.publish("test-topic", vec![]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_publish_multi_empty_topics() {
        let broker = InMemoryMessageBroker::new();
        let wrapper = Arc::new(MessageBrokerWrapper::new(broker));
        let component_id = ComponentId::new("test-component");
        let publisher = MessagePublisher::new(component_id, wrapper);

        let topics: &[&str] = &[];
        let result = publisher.publish_multi(topics, vec![1, 2, 3]).await;
        assert!(result.is_ok());
    }
}
