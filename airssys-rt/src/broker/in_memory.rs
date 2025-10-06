//! Pure pub-sub in-memory message broker.
//!
//! This broker implements a pure publish-subscribe pattern with no actor registry.
//! Subscribers receive all published messages and are responsible for routing.

// Layer 1: Standard library imports
use std::sync::Arc;
use std::time::Duration;

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use dashmap::DashMap;
use tokio::sync::{mpsc, oneshot, RwLock};
use tokio::time::timeout;

// Layer 3: Internal module imports
use super::error::BrokerError;
use super::traits::{MessageBroker, MessageStream};
use crate::message::{Message, MessageEnvelope};
use crate::util::ActorAddress;

/// Pure pub-sub in-memory message broker.
///
/// This broker maintains no actor registry - it simply broadcasts messages
/// to all subscribers. Subscribers (like ActorSystem) are responsible for
/// routing messages to specific actors.
///
/// # Architecture
///
/// ```text
/// Publisher → publish() → Broker → Subscribers (ActorSystem, Monitors, etc.)
/// ```
///
/// # Example
///
/// ```ignore
/// let broker = InMemoryMessageBroker::new();
///
/// // Subscribe (ActorSystem does this)
/// let mut stream = broker.subscribe().await?;
/// tokio::spawn(async move {
///     while let Some(envelope) = stream.recv().await {
///         route_to_actor(envelope).await;
///     }
/// });
///
/// // Publish message
/// broker.publish(envelope).await?;
/// ```
#[derive(Clone)]
pub struct InMemoryMessageBroker<M: Message> {
    inner: Arc<InMemoryBrokerInner<M>>,
}

struct InMemoryBrokerInner<M: Message> {
    /// Pub-sub subscribers: each receives all published messages
    subscribers: RwLock<Vec<mpsc::UnboundedSender<MessageEnvelope<M>>>>,

    /// Pending request-reply channels: correlation_id -> response sender
    pending_requests: DashMap<uuid::Uuid, oneshot::Sender<Vec<u8>>>,
}

impl<M: Message> InMemoryMessageBroker<M> {
    /// Create a new pure pub-sub broker.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(InMemoryBrokerInner {
                subscribers: RwLock::new(Vec::new()),
                pending_requests: DashMap::new(),
            }),
        }
    }

    /// Get the number of active subscribers.
    pub async fn subscriber_count(&self) -> usize {
        self.inner.subscribers.read().await.len()
    }

    /// Check if a message is a reply to a pending request and route it.
    ///
    /// Returns `true` if the message was routed as a reply, `false` otherwise.
    fn try_route_reply(&self, envelope: &MessageEnvelope<M>) -> bool
    where
        M: serde::Serialize,
    {
        if let Some(correlation_id) = &envelope.correlation_id {
            if let Some((_, tx)) = self.inner.pending_requests.remove(correlation_id) {
                // Serialize and send through oneshot channel
                if let Ok(serialized) = serde_json::to_vec(envelope) {
                    let _ = tx.send(serialized);
                    return true;
                }
            }
        }
        false
    }
}

#[async_trait]
impl<M: Message + serde::Serialize> MessageBroker<M> for InMemoryMessageBroker<M> {
    type Error = BrokerError;

    async fn publish(&self, envelope: MessageEnvelope<M>) -> Result<(), Self::Error> {
        // Check if this is a reply to a pending request
        if self.try_route_reply(&envelope) {
            return Ok(());
        }

        // Broadcast to all subscribers
        let subscribers = self.inner.subscribers.read().await;

        for subscriber in subscribers.iter() {
            // Send clone to each subscriber (ignore closed channels)
            let _ = subscriber.send(envelope.clone());
        }

        Ok(())
    }

    async fn subscribe(&self) -> Result<MessageStream<M>, Self::Error> {
        let (tx, rx) = mpsc::unbounded_channel();

        let mut subscribers = self.inner.subscribers.write().await;
        subscribers.push(tx);

        Ok(MessageStream::new(rx))
    }

    async fn publish_request<R: Message + for<'de> serde::Deserialize<'de>>(
        &self,
        mut envelope: MessageEnvelope<M>,
        timeout_duration: Duration,
    ) -> Result<Option<MessageEnvelope<R>>, Self::Error> {
        // Generate correlation ID
        let correlation_id = uuid::Uuid::new_v4();
        envelope.correlation_id = Some(correlation_id);

        // Create oneshot channel for reply
        let (tx, rx) = oneshot::channel();

        // Register pending request
        self.inner.pending_requests.insert(correlation_id, tx);

        // Publish request
        self.publish(envelope.clone()).await?;

        // Wait for reply with timeout
        let target = envelope.reply_to.unwrap_or_else(ActorAddress::anonymous);

        match timeout(timeout_duration, rx).await {
            Ok(Ok(serialized)) => {
                let response: MessageEnvelope<R> =
                    serde_json::from_slice(&serialized).map_err(|e| BrokerError::RouteError {
                        message_type: R::MESSAGE_TYPE,
                        reason: format!("Failed to deserialize reply: {e}"),
                    })?;
                Ok(Some(response))
            }
            Ok(Err(_)) => {
                self.inner.pending_requests.remove(&correlation_id);
                Ok(None)
            }
            Err(_) => {
                self.inner.pending_requests.remove(&correlation_id);
                Err(BrokerError::RequestTimeout {
                    target,
                    timeout: timeout_duration,
                })
            }
        }
    }
}

impl<M: Message> Default for InMemoryMessageBroker<M> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::MessagePriority;

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    struct TestMessage {
        data: String,
    }

    impl Message for TestMessage {
        const MESSAGE_TYPE: &'static str = "test_message";

        fn priority(&self) -> MessagePriority {
            MessagePriority::Normal
        }
    }

    type TestBroker = InMemoryMessageBroker<TestMessage>;

    #[tokio::test]
    async fn test_pub_sub_basic() {
        let broker = TestBroker::new();

        // Subscribe
        let mut stream = broker.subscribe().await.unwrap();

        // Publish
        let msg = TestMessage {
            data: "hello".to_string(),
        };
        let envelope = MessageEnvelope::new(msg);

        broker.publish(envelope).await.unwrap();

        // Receive
        let received = stream.recv().await.unwrap();
        assert_eq!(received.payload.data, "hello");
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let broker = TestBroker::new();

        // Create 3 subscribers
        let mut stream1 = broker.subscribe().await.unwrap();
        let mut stream2 = broker.subscribe().await.unwrap();
        let mut stream3 = broker.subscribe().await.unwrap();

        assert_eq!(broker.subscriber_count().await, 3);

        // Publish one message
        let msg = TestMessage {
            data: "broadcast".to_string(),
        };
        broker.publish(MessageEnvelope::new(msg)).await.unwrap();

        // All subscribers should receive it
        let r1 = stream1.recv().await.unwrap();
        let r2 = stream2.recv().await.unwrap();
        let r3 = stream3.recv().await.unwrap();

        assert_eq!(r1.payload.data, "broadcast");
        assert_eq!(r2.payload.data, "broadcast");
        assert_eq!(r3.payload.data, "broadcast");
    }

    #[tokio::test]
    async fn test_publish_without_subscribers() {
        let broker = TestBroker::new();

        // Publishing without subscribers should not error
        let msg = TestMessage {
            data: "nobody listening".to_string(),
        };
        let result = broker.publish(MessageEnvelope::new(msg)).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_request_reply() {
        let broker = TestBroker::new();

        // Subscribe and spawn responder task
        let mut stream = broker.subscribe().await.unwrap();
        let broker_clone = broker.clone();

        tokio::spawn(async move {
            if let Some(request) = stream.recv().await {
                if let Some(corr_id) = request.correlation_id {
                    // Send reply with same correlation_id
                    let reply = TestMessage {
                        data: "pong".to_string(),
                    };
                    let mut reply_envelope = MessageEnvelope::new(reply);
                    reply_envelope.correlation_id = Some(corr_id);

                    let _ = broker_clone.publish(reply_envelope).await;
                }
            }
        });

        // Send request
        let request = TestMessage {
            data: "ping".to_string(),
        };
        let envelope = MessageEnvelope::new(request);

        let response: Option<MessageEnvelope<TestMessage>> = broker
            .publish_request(envelope, Duration::from_secs(1))
            .await
            .unwrap();

        assert!(response.is_some());
        assert_eq!(response.unwrap().payload.data, "pong");
    }

    #[tokio::test]
    async fn test_request_timeout() {
        let broker = TestBroker::new();

        // Subscribe but don't reply
        let _stream = broker.subscribe().await.unwrap();

        let request = TestMessage {
            data: "no reply".to_string(),
        };

        let result: Result<Option<MessageEnvelope<TestMessage>>, _> = broker
            .publish_request(MessageEnvelope::new(request), Duration::from_millis(10))
            .await;

        assert!(matches!(result, Err(BrokerError::RequestTimeout { .. })));
    }
}
