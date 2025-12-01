# KNOWLEDGE-RT-012: Pub-Sub MessageBroker Pattern

**Created:** 2025-10-06  
**Status:** Active  
**Category:** Architecture Pattern  
**Related:** ADR-006, DEBT-RT-005, RT-TASK-004, RT-TASK-006  
**Complexity:** High  
**Impact:** Critical - Foundation for actor messaging  

---

## Problem Context

The original MessageBroker design used direct routing semantics with `send(recipient, message)` method. This created tight coupling between transport layer and routing logic, preventing essential features like:

- ❌ Message monitoring and observability
- ❌ Dead letter queue handling
- ❌ Circuit breakers and resilience patterns
- ❌ Distributed broker implementations (Redis, NATS)
- ❌ Multiple subscribers to message streams
- ❌ Extensibility hooks for logging/metrics/persistence

**Root Cause**: MessageBroker was treated as a router instead of a transport layer (pub-sub bus).

---

## Pub-Sub Architecture Solution

### **Core Principle**

**MessageBroker = Pure Pub-Sub Transport Layer**

```
Publishers → [MessageBroker] → Subscribers
             (Event Bus)
```

### **Separation of Concerns**

| Component | Responsibility | Does NOT Handle |
|-----------|---------------|-----------------|
| **MessageBroker** | Publish/Subscribe transport | Routing to specific actors |
| **ActorRegistry** | Address → Mailbox mapping | Message transport |
| **ActorSystem** | Subscribe & route messages | Direct message delivery |
| **ActorContext** | Publish messages (actor API) | Address resolution |

---

## Trait Definition

### **MessageBroker Trait (Updated)**

```rust
use std::error::Error;
use async_trait::async_trait;
use tokio::sync::mpsc;
use std::time::Duration;

/// Message stream from broker subscriptions
pub struct MessageStream<M: Message> {
    receiver: mpsc::UnboundedReceiver<MessageEnvelope<M>>,
}

impl<M: Message> MessageStream<M> {
    /// Create a new message stream
    pub fn new(receiver: mpsc::UnboundedReceiver<MessageEnvelope<M>>) -> Self {
        Self { receiver }
    }
    
    /// Receive next message from stream
    pub async fn recv(&mut self) -> Option<MessageEnvelope<M>> {
        self.receiver.recv().await
    }
    
    /// Try to receive without blocking
    pub fn try_recv(&mut self) -> Result<MessageEnvelope<M>, TryRecvError> {
        self.receiver.try_recv()
    }
}

#[async_trait]
pub trait MessageBroker<M: Message>: Send + Sync + Clone {
    type Error: Error + Send + Sync + 'static;
    
    /// Publish a message to the broker bus
    /// 
    /// Messages are broadcast to all subscribers. This is the fundamental
    /// operation for actor-to-actor communication.
    /// 
    /// # Extensibility Hooks
    /// 
    /// Implementations can add hooks for:
    /// - Logging and tracing
    /// - Metrics collection
    /// - Message persistence
    /// - Circuit breakers
    /// - Rate limiting
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let envelope = MessageEnvelope::new(message)
    ///     .with_sender(sender_address)
    ///     .with_recipient(recipient_address);
    /// 
    /// broker.publish(envelope).await?;
    /// ```
    async fn publish(&self, envelope: MessageEnvelope<M>) -> Result<(), Self::Error>;
    
    /// Subscribe to message events on the broker
    /// 
    /// Returns a stream of all messages published to the broker. Multiple
    /// subscribers can listen to the same message stream independently.
    /// 
    /// # Use Cases
    /// 
    /// - **ActorSystem**: Subscribes to route messages to actors
    /// - **Monitor**: Subscribes for observability and metrics
    /// - **Audit**: Subscribes for compliance logging
    /// - **Dead Letter**: Subscribes to capture undeliverable messages
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let mut stream = broker.subscribe().await?;
    /// 
    /// while let Some(envelope) = stream.recv().await {
    ///     // Process message
    ///     route_to_actor(envelope).await?;
    /// }
    /// ```
    async fn subscribe(&self) -> Result<MessageStream<M>, Self::Error>;
    
    /// Publish a request and wait for correlated reply
    /// 
    /// This is a convenience method that combines publish with correlation
    /// tracking for request-reply patterns.
    /// 
    /// # Arguments
    /// 
    /// * `envelope` - The request message to publish
    /// * `timeout` - How long to wait for a reply
    /// 
    /// # Returns
    /// 
    /// - `Ok(Some(reply))` - Reply received within timeout
    /// - `Ok(None)` - Timeout expired with no reply
    /// - `Err(e)` - Broker error occurred
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let request = MessageEnvelope::new(MyRequest { data: 42 })
    ///     .with_sender(requester)
    ///     .with_recipient(responder)
    ///     .with_correlation_id(request_id);
    /// 
    /// let reply = broker.publish_request::<MyReply>(
    ///     request,
    ///     Duration::from_secs(5)
    /// ).await?;
    /// 
    /// match reply {
    ///     Some(envelope) => println!("Got reply: {:?}", envelope.message),
    ///     None => println!("Request timeout"),
    /// }
    /// ```
    async fn publish_request<R: Message>(
        &self,
        envelope: MessageEnvelope<M>,
        timeout: Duration,
    ) -> Result<Option<MessageEnvelope<R>>, Self::Error>;
}
```

---

## Implementation Pattern: InMemoryMessageBroker

### **Data Structure**

```rust
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use dashmap::DashMap;

pub struct InMemoryMessageBroker<M, S>
where
    M: Message,
    S: MailboxSender<M> + Clone,
{
    inner: Arc<InMemoryMessageBrokerInner<M, S>>,
}

struct InMemoryMessageBrokerInner<M, S> {
    // Actor lifecycle management
    registry: ActorRegistry<M, S>,
    
    // Pub-sub subscribers (NEW)
    subscribers: RwLock<Vec<mpsc::UnboundedSender<MessageEnvelope<M>>>>,
    
    // Request-reply correlation tracking
    pending_requests: DashMap<CorrelationId, PendingRequest<M>>,
    
    // Extensibility hooks (optional)
    config: BrokerConfig,
    metrics: Arc<BrokerMetrics>,
}

impl<M, S> Clone for InMemoryMessageBroker<M, S>
where
    M: Message,
    S: MailboxSender<M> + Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}
```

### **Publish Implementation with Extensibility Hooks**

```rust
#[async_trait]
impl<M, S> MessageBroker<M> for InMemoryMessageBroker<M, S>
where
    M: Message,
    S: MailboxSender<M> + Clone,
{
    type Error = BrokerError;
    
    async fn publish(&self, envelope: MessageEnvelope<M>) -> Result<(), Self::Error> {
        // HOOK 1: Logging
        log::trace!(
            "Publishing message: id={}, sender={:?}, recipient={:?}",
            envelope.metadata.message_id,
            envelope.metadata.sender,
            envelope.metadata.reply_to
        );
        
        // HOOK 2: Metrics
        self.inner.metrics.messages_published.fetch_add(1, Ordering::Relaxed);
        
        // HOOK 3: Persistence (optional)
        if self.inner.config.persist_messages {
            self.persist_message(&envelope).await?;
        }
        
        // HOOK 4: Circuit breaker check
        if let Some(ref recipient) = envelope.metadata.reply_to {
            if self.inner.config.enable_circuit_breaker {
                self.check_circuit_breaker(recipient)?;
            }
        }
        
        // HOOK 5: Rate limiting (optional)
        if self.inner.config.enable_rate_limiting {
            self.check_rate_limit(&envelope).await?;
        }
        
        // Broadcast to all subscribers
        let subscribers = self.inner.subscribers.read().await;
        let mut failed_subscribers = Vec::new();
        
        for (idx, sender) in subscribers.iter().enumerate() {
            if let Err(_) = sender.send(envelope.clone()) {
                // Subscriber channel closed - mark for removal
                failed_subscribers.push(idx);
            }
        }
        
        // Clean up disconnected subscribers
        if !failed_subscribers.is_empty() {
            drop(subscribers); // Release read lock
            let mut subscribers = self.inner.subscribers.write().await;
            
            // Remove in reverse order to maintain indices
            for idx in failed_subscribers.into_iter().rev() {
                subscribers.swap_remove(idx);
            }
        }
        
        Ok(())
    }
    
    async fn subscribe(&self) -> Result<MessageStream<M>, Self::Error> {
        let (tx, rx) = mpsc::unbounded_channel();
        
        let mut subscribers = self.inner.subscribers.write().await;
        subscribers.push(tx);
        
        log::debug!("New subscriber registered (total: {})", subscribers.len());
        
        Ok(MessageStream::new(rx))
    }
    
    async fn publish_request<R: Message>(
        &self,
        envelope: MessageEnvelope<M>,
        timeout: Duration,
    ) -> Result<Option<MessageEnvelope<R>>, Self::Error> {
        let correlation_id = envelope.metadata.correlation_id
            .ok_or(BrokerError::MissingCorrelationId)?;
        
        // Create reply channel
        let (tx, mut rx) = mpsc::unbounded_channel();
        
        // Register pending request
        let pending = PendingRequest {
            sender: envelope.metadata.sender.clone(),
            reply_channel: tx,
            sent_at: Utc::now(),
        };
        
        self.inner.pending_requests.insert(correlation_id.clone(), pending);
        
        // Publish request
        self.publish(envelope).await?;
        
        // Wait for reply with timeout
        let result = tokio::time::timeout(timeout, rx.recv()).await;
        
        // Cleanup
        self.inner.pending_requests.remove(&correlation_id);
        
        match result {
            Ok(Some(reply)) => {
                // Deserialize reply (type safety check)
                let reply_envelope: MessageEnvelope<R> = 
                    serde_json::from_value(serde_json::to_value(reply)?)?;
                Ok(Some(reply_envelope))
            }
            Ok(None) => Ok(None), // Channel closed
            Err(_) => Ok(None),   // Timeout
        }
    }
}
```

---

## ActorSystem Integration Pattern

### **ActorSystem with Message Router**

```rust
pub struct ActorSystem<M, S, B>
where
    M: Message,
    S: MailboxSender<M> + Clone,
    B: MessageBroker<M>,
{
    inner: Arc<ActorSystemInner<M, S, B>>,
}

struct ActorSystemInner<M, S, B> {
    config: SystemConfig,
    broker: B,                      // ← Injected via DI
    registry: ActorRegistry<M, S>,  // ← Created internally
    actors: RwLock<HashMap<ActorId, ActorMetadata>>,
    state: RwLock<SystemState>,
    router_handle: RwLock<Option<JoinHandle<()>>>,
}

impl<M, S, B> ActorSystem<M, S, B>
where
    M: Message,
    S: MailboxSender<M> + Clone,
    B: MessageBroker<M>,
{
    /// Create new actor system with dependency injection
    pub async fn new(config: SystemConfig, broker: B) -> Result<Self, SystemError> {
        let system = Self {
            inner: Arc::new(ActorSystemInner {
                config,
                broker,
                registry: ActorRegistry::new(),
                actors: RwLock::new(HashMap::new()),
                state: RwLock::new(SystemState::Running),
                router_handle: RwLock::new(None),
            }),
        };
        
        // Subscribe to broker and start message router
        system.start_message_router().await?;
        
        Ok(system)
    }
    
    /// Start background task to route messages from broker to actors
    async fn start_message_router(&self) -> Result<(), SystemError> {
        let mut stream = self.inner.broker.subscribe().await
            .map_err(|e| SystemError::broker_error(e.to_string()))?;
        
        let inner = Arc::clone(&self.inner);
        
        let handle = tokio::spawn(async move {
            log::info!("Message router started");
            
            while let Some(envelope) = stream.recv().await {
                // Check system state
                let state = inner.state.read().await;
                if *state != SystemState::Running {
                    log::warn!("System not running, dropping message");
                    continue;
                }
                drop(state);
                
                // Extract recipient
                let recipient = match &envelope.metadata.reply_to {
                    Some(addr) => addr.clone(),
                    None => {
                        log::warn!("Message without recipient: {:?}", envelope.metadata.message_id);
                        // TODO: Dead letter queue
                        continue;
                    }
                };
                
                // Resolve actor via registry
                match inner.registry.resolve(&recipient) {
                    Ok(sender) => {
                        // Forward to actor's mailbox
                        if let Err(e) = sender.send(envelope.clone()).await {
                            log::error!(
                                "Failed to deliver to {}: {:?}",
                                recipient,
                                e
                            );
                            // TODO: Dead letter queue
                        } else {
                            log::trace!("Delivered message to {}", recipient);
                        }
                    }
                    Err(e) => {
                        log::warn!("Actor not found {}: {:?}", recipient, e);
                        // TODO: Dead letter queue
                    }
                }
            }
            
            log::info!("Message router stopped");
        });
        
        *self.inner.router_handle.write().await = Some(handle);
        
        Ok(())
    }
    
    /// Stop message router on shutdown
    pub async fn shutdown(&self) -> Result<(), SystemError> {
        // Update state
        *self.inner.state.write().await = SystemState::ShuttingDown;
        
        // Stop router
        if let Some(handle) = self.inner.router_handle.write().await.take() {
            handle.abort();
        }
        
        // Shutdown all actors
        // ... (actor shutdown logic)
        
        *self.inner.state.write().await = SystemState::Stopped;
        
        Ok(())
    }
}
```

---

## ActorContext Publisher Pattern

### **ActorContext with Broker Publish**

```rust
pub struct ActorContext<M, B>
where
    M: Message,
    B: MessageBroker<M>,
{
    address: ActorAddress,
    broker: B,  // ← For publishing messages
    metadata: ContextMetadata,
    _marker: PhantomData<M>,
}

impl<M, B> ActorContext<M, B>
where
    M: Message,
    B: MessageBroker<M>,
{
    pub fn new(address: ActorAddress, broker: B) -> Self {
        Self {
            address,
            broker,
            metadata: ContextMetadata::default(),
            _marker: PhantomData,
        }
    }
    
    /// Send message to another actor (publishes to broker)
    pub async fn send(
        &self,
        recipient: ActorAddress,
        message: M,
    ) -> Result<(), B::Error> {
        let envelope = MessageEnvelope::new(message)
            .with_sender(self.address.clone())
            .with_recipient(recipient);
        
        // Publish to broker (pub-sub pattern)
        self.broker.publish(envelope).await
    }
    
    /// Request-reply pattern
    pub async fn request<R: Message>(
        &self,
        recipient: ActorAddress,
        message: M,
        timeout: Duration,
    ) -> Result<Option<R>, B::Error> {
        let correlation_id = CorrelationId::new();
        
        let envelope = MessageEnvelope::new(message)
            .with_sender(self.address.clone())
            .with_recipient(recipient)
            .with_correlation_id(correlation_id);
        
        let reply = self.broker.publish_request::<R>(envelope, timeout).await?;
        
        Ok(reply.map(|env| env.message))
    }
}
```

---

## Benefits of Pub-Sub Architecture

### ✅ **1. Extensibility at Transport Layer**

```rust
async fn publish(&self, envelope: MessageEnvelope<M>) -> Result<()> {
    // Easy to add new capabilities without changing actors
    self.logger.log_message(&envelope);
    self.metrics.record_published();
    self.persistence.save(&envelope).await?;
    self.circuit_breaker.check(&envelope.reply_to)?;
    
    // Then broadcast
    self.broadcast_to_subscribers(envelope).await
}
```

### ✅ **2. Multiple Independent Subscribers**

```rust
// Routing
let routing_stream = broker.subscribe().await?;
tokio::spawn(route_messages(routing_stream, registry));

// Monitoring
let monitor_stream = broker.subscribe().await?;
tokio::spawn(monitor_messages(monitor_stream, metrics));

// Audit
let audit_stream = broker.subscribe().await?;
tokio::spawn(audit_messages(audit_stream, storage));
```

### ✅ **3. Dead Letter Queue Support**

```rust
match registry.resolve(&recipient) {
    Ok(sender) => {
        if let Err(_) = sender.send(envelope).await {
            dead_letter_queue.push(envelope).await;
        }
    }
    Err(_) => {
        dead_letter_queue.push(envelope).await;
    }
}
```

### ✅ **4. Distributed Broker Implementations**

```rust
// Redis pub-sub
impl<M: Message> MessageBroker<M> for RedisMessageBroker {
    async fn publish(&self, envelope: MessageEnvelope<M>) -> Result<()> {
        let data = serde_json::to_vec(&envelope)?;
        self.redis.publish("actor-messages", data).await?;
        Ok(())
    }
    
    async fn subscribe(&self) -> Result<MessageStream<M>> {
        // Listen to Redis pub-sub channel
        let stream = self.redis.subscribe("actor-messages").await?;
        Ok(MessageStream::from_redis(stream))
    }
}

// NATS pub-sub
impl<M: Message> MessageBroker<M> for NatsMessageBroker {
    async fn publish(&self, envelope: MessageEnvelope<M>) -> Result<()> {
        let data = serde_json::to_vec(&envelope)?;
        self.client.publish("actors", data).await?;
        Ok(())
    }
    
    async fn subscribe(&self) -> Result<MessageStream<M>> {
        let sub = self.client.subscribe("actors").await?;
        Ok(MessageStream::from_nats(sub))
    }
}
```

### ✅ **5. Testing and Mocking**

```rust
// Mock broker for tests
pub struct MockMessageBroker<M: Message> {
    published: Arc<Mutex<Vec<MessageEnvelope<M>>>>,
    subscribers: Arc<Mutex<Vec<mpsc::UnboundedSender<MessageEnvelope<M>>>>>,
}

#[async_trait]
impl<M: Message> MessageBroker<M> for MockMessageBroker<M> {
    async fn publish(&self, envelope: MessageEnvelope<M>) -> Result<()> {
        self.published.lock().unwrap().push(envelope.clone());
        
        for sender in self.subscribers.lock().unwrap().iter() {
            let _ = sender.send(envelope.clone());
        }
        
        Ok(())
    }
    
    async fn subscribe(&self) -> Result<MessageStream<M>> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.subscribers.lock().unwrap().push(tx);
        Ok(MessageStream::new(rx))
    }
}

// Easy assertions in tests
#[tokio::test]
async fn test_actor_sends_message() {
    let broker = MockMessageBroker::new();
    let context = ActorContext::new(address, broker.clone());
    
    context.send(recipient, message).await.unwrap();
    
    let published = broker.get_published();
    assert_eq!(published.len(), 1);
    assert_eq!(published[0].metadata.reply_to, Some(recipient));
}
```

---

## Implementation Checklist

### **Phase 0: Update MessageBroker Trait**
- [ ] Add `MessageStream<M>` type
- [ ] Add `publish()` method (replace `send()`)
- [ ] Add `subscribe()` method
- [ ] Add `publish_request()` method
- [ ] Update trait documentation with pub-sub semantics
- [ ] Update all trait tests

### **Phase 1: Update InMemoryMessageBroker**
- [ ] Add `subscribers: RwLock<Vec<UnboundedSender>>` field
- [ ] Implement `publish()` with broadcast to subscribers
- [ ] Implement `subscribe()` with subscriber registration
- [ ] Implement `publish_request()` with correlation tracking
- [ ] Add extensibility hooks (logging, metrics placeholders)
- [ ] Handle disconnected subscribers cleanup
- [ ] Add comprehensive pub-sub tests (~15 tests)

### **Phase 2: Update ActorSystem**
- [ ] Add `router_handle: RwLock<Option<JoinHandle<()>>>` field
- [ ] Implement `start_message_router()` background task
- [ ] Subscribe to broker in `new()`
- [ ] Route messages via `ActorRegistry::resolve()`
- [ ] Add TODO markers for dead letter queue
- [ ] Implement `shutdown()` to stop router
- [ ] Add router tests (~10 tests)

### **Phase 3: Update ActorContext (RT-TASK-007)**
- [ ] Change `send()` to use `broker.publish()`
- [ ] Change `request()` to use `broker.publish_request()`
- [ ] Update all context tests
- [ ] Update examples with new API

---

## Performance Considerations

### **Memory Overhead**
- Each subscriber: 1 unbounded channel (~48 bytes + buffer)
- Typical setup: 3-4 subscribers (routing + monitoring + audit) = ~200 bytes
- **Impact**: Negligible (<1KB per broker instance)

### **Latency**
- Direct routing: 0 hops (actor → mailbox)
- Pub-sub routing: 1 hop (actor → broker → router → mailbox)
- **Overhead**: ~1-2μs per message (channel send/recv)
- **Trade-off**: Minimal latency for massive extensibility gain

### **Throughput**
- Tokio mpsc unbounded: ~10M messages/sec on modern CPU
- Broadcast to 3-4 subscribers: ~3M messages/sec
- **Bottleneck**: Actor processing, not broker transport

---

## Related Documentation

- **ADR-006**: MessageBroker Pub-Sub Architecture Decision
- **DEBT-RT-005**: Technical debt analysis and resolution plan
- **RT-TASK-004**: Message Broker Core implementation task
- **RT-TASK-006**: Actor System Framework integration
- **KNOWLEDGE-RT-011**: Actor System Integration Guide (needs update)

---

**Status**: Implementation guide ready  
**Next Action**: Implement Phase 0 (update MessageBroker trait)  
**Estimated Time**: 8-12 hours for complete pub-sub implementation  
**Priority**: CRITICAL - Blocks RT-TASK-006 Phase 2
