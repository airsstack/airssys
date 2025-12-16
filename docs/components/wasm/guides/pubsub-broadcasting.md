# Pub-Sub Broadcasting Pattern

**Category:** How-To Guide (Task-Oriented)  
**Purpose:** This guide shows you how to implement publish-subscribe broadcasting for one-to-many component communication.

## Overview

Pub-sub (publish-subscribe) is a messaging pattern where publishers broadcast messages to multiple subscribers without knowing who the subscribers are. This decoupling enables flexible, scalable communication in component-based systems.

**Key Features:**
- Topic-based message routing
- Multiple subscribers per topic
- Subscriber isolation (crash doesn't affect others)
- Dynamic subscription management

**Performance:** Fanout to 100 subscribers completes in 85.2µs (measured in Task 6.2 `messaging_benchmarks.rs` benchmark `bench_pubsub_fanout_100`, macOS M1, 100 samples).

## When to Use Pub-Sub

**Use pub-sub when:**
- Broadcasting to multiple components is needed
- Publishers shouldn't know about subscribers
- Dynamic subscriber addition/removal is required
- Event-driven architecture is preferred

**Don't use pub-sub when:**
- One-to-one communication is sufficient (use request-response)
- Guaranteed delivery to specific component is required
- Response from subscriber is needed (use request-response instead)

## Prerequisites

Before implementing pub-sub, you should understand:
- ComponentActor basics (see [Your First ComponentActor](../tutorials/your-first-component-actor.md))
- MessageBroker integration with airssys-rt
- ComponentRegistry for component lookup

## Implementation Steps

### Step 1: Set Up MessageBroker

The `MessageBroker` handles topic-based message routing and subscriber management.

```rust
use std::sync::Arc;

use airssys_rt::broker::InMemoryMessageBroker;
use airssys_wasm::actor::ComponentMessage;

// Create broker (usually one per system)
let broker = Arc::new(InMemoryMessageBroker::<ComponentMessage>::new());
```

**Note:** `InMemoryMessageBroker` is provided by `airssys-rt` and supports all pub-sub operations.

### Step 2: Create Publisher Component

The publisher broadcasts messages to a topic without knowing who the subscribers are.

```rust
use airssys_wasm::actor::{ComponentMessage, MessageRouter};
use airssys_wasm::core::ComponentId;

struct PublisherComponent {
    router: MessageRouter<InMemoryMessageBroker<ComponentMessage>>,
    topic: String,
}

impl PublisherComponent {
    async fn publish_event(&self, event_data: Vec<u8>) -> Result<(), String> {
        // Create event message
        let message = ComponentMessage::Custom {
            topic: self.topic.clone(),
            payload: event_data,
        };
        
        // Broadcast to all subscribers via broker
        // The broker handles fanout automatically
        self.router.broadcast_to_topic(&self.topic, message).await
            .map_err(|e| format!("Failed to publish: {}", e))?;
        
        Ok(())
    }
}
```

### Step 3: Create Subscriber Components

Subscribers register interest in topics and receive all messages published to those topics.

```rust
struct SubscriberComponent {
    id: ComponentId,
    subscribed_topics: Vec<String>,
}

impl SubscriberComponent {
    fn new(id: ComponentId, topics: Vec<String>) -> Self {
        Self {
            id,
            subscribed_topics: topics,
        }
    }
    
    async fn handle_event(&self, topic: &str, payload: Vec<u8>) {
        // Process event
        println!("Subscriber {} received event on topic '{}': {} bytes",
            self.id.as_str(), topic, payload.len());
        
        // Your business logic here
    }
}

// In Actor::handle_message implementation
async fn handle_message(message: ComponentMessage) -> Result<(), WasmError> {
    match message {
        ComponentMessage::Custom { topic, payload } => {
            if self.subscribed_topics.contains(&topic) {
                self.handle_event(&topic, payload).await;
            }
        }
        _ => {}
    }
    Ok(())
}
```

### Step 4: Subscribe to Topics

Components subscribe to topics by registering with the broker.

```rust
use airssys_rt::broker::MessageBroker;

// Subscribe to topic
let topic = "sensor.temperature".to_string();
broker.subscribe(&topic, subscriber_addr.clone()).await
    .map_err(|e| format!("Failed to subscribe: {}", e))?;

// Subscribe to multiple topics
for topic in &subscriber.subscribed_topics {
    broker.subscribe(topic, subscriber_addr.clone()).await?;
}
```

### Step 5: Unsubscribe from Topics

Components can unsubscribe when they no longer want to receive messages.

```rust
// Unsubscribe from specific topic
broker.unsubscribe(&topic, &subscriber_addr).await?;

// Unsubscribe from all topics (component shutdown)
for topic in &subscriber.subscribed_topics {
    broker.unsubscribe(topic, &subscriber_addr).await.ok();
}
```

## Topic Naming Conventions

Use hierarchical topic names for better organization:

```rust
// Good: hierarchical, specific
"sensor.temperature.room1"
"sensor.humidity.room2"
"actuator.light.room1"

// Bad: flat, unclear
"temp"
"data"
"event"
```

**Benefits of hierarchical topics:**
- Easier to manage subscriptions
- Supports wildcard subscriptions (if implemented)
- Better logging and debugging

## Subscriber Isolation

One of the key benefits of pub-sub is subscriber isolation: if one subscriber crashes, others continue to receive messages.

```rust
// If subscriber B crashes, subscribers A and C still receive messages
let subscribers = vec![
    ComponentId::new("subscriber-a"),
    ComponentId::new("subscriber-b"), // crashes
    ComponentId::new("subscriber-c"),
];

// Broker handles delivery failures gracefully
// Failed deliveries are logged but don't affect other subscribers
```

**Implementation Detail:** The broker sends messages to each subscriber independently. If delivery to one subscriber fails, the broker continues with remaining subscribers.

## Performance Characteristics

Based on Task 6.2 benchmarks (`messaging_benchmarks.rs`):

| Operation | Latency | Benchmark |
|-----------|---------|-----------|
| Fanout to 10 subscribers | ~8.5µs | `bench_pubsub_fanout_10` |
| Fanout to 100 subscribers | 85.2µs | `bench_pubsub_fanout_100` |
| Subscribe (register 10) | <500µs | `bench_subscription_management` |
| Broadcast single message | ~1.05µs | Inferred from routing |

**Test Conditions:** macOS M1, 100 samples, 95% confidence interval

### Scalability Characteristics

- **Linear fanout:** 85.2µs for 100 subscribers = ~852ns per subscriber
- **Constant subscription:** Registration overhead is O(1)
- **Concurrent delivery:** Messages sent in parallel (limited by available threads)

### Optimization Tips

1. **Batch Publications:** If publishing multiple events, batch them when possible
2. **Filter Topics:** Use specific topics to reduce unnecessary message delivery
3. **Subscriber Count:** Monitor subscriber count per topic for capacity planning
4. **Async Delivery:** Broker uses async delivery to minimize blocking

## Broadcasting Patterns

### Simple Broadcast

Send one message to all subscribers:

```rust
publisher.publish_event("temperature".to_string(), vec![25, 0, 0, 0]).await?;
```

### Filtered Broadcast

Send different messages based on subscriber criteria:

```rust
// Publisher sends raw data
publisher.publish_event("sensor.raw", raw_data).await?;

// Subscriber filters based on local criteria
impl SubscriberComponent {
    async fn handle_event(&self, topic: &str, payload: Vec<u8>) {
        if topic == "sensor.raw" && self.should_process(&payload) {
            // Process only relevant events
        }
    }
    
    fn should_process(&self, payload: &[u8]) -> bool {
        // Filter logic
        true
    }
}
```

### Multi-Topic Broadcast

Publish to multiple topics simultaneously:

```rust
async fn publish_multi_topic(&self, topics: &[String], payload: Vec<u8>) {
    for topic in topics {
        self.broker.broadcast_to_topic(topic, 
            ComponentMessage::Custom {
                topic: topic.clone(),
                payload: payload.clone(),
            }
        ).await.ok();
    }
}
```

## Message Delivery Guarantees

**Pub-sub provides:**
- **At-most-once delivery:** Each subscriber receives each message at most once
- **Best-effort delivery:** Messages are delivered if subscriber is available and responsive
- **No ordering guarantees:** Messages may arrive out of order across different subscribers

**Pub-sub does NOT guarantee:**
- **Exactly-once delivery:** Use request-response for guaranteed delivery
- **Message persistence:** Messages are not stored if no subscribers are available
- **Ordered delivery:** Use sequence numbers if ordering is critical

## Error Handling

### Publisher Errors

```rust
match self.publish_event(data).await {
    Ok(_) => {}
    Err(e) if e.contains("broker unavailable") => {
        // Broker is down, queue message or drop
        log::error!("Broker unavailable: {}", e);
    }
    Err(e) => {
        log::error!("Publish failed: {}", e);
    }
}
```

### Subscriber Errors

```rust
// In subscriber's handle_message
async fn handle_message(message: ComponentMessage) -> Result<(), WasmError> {
    match message {
        ComponentMessage::Custom { topic, payload } => {
            // Wrap processing in error boundary
            if let Err(e) = self.handle_event(&topic, payload).await {
                log::error!("Event processing failed for topic {}: {}", topic, e);
                // Don't propagate error to broker (isolate failures)
            }
        }
        _ => {}
    }
    Ok(())
}
```

## Complete Example

See [examples/pubsub_component.rs](../../../examples/pubsub_component.rs) for a complete, runnable example demonstrating:
- Publisher broadcasting to multiple subscribers
- Dynamic subscription management
- Subscriber isolation
- Fanout performance

Run the example:
```bash
cargo run --example pubsub_component
```

## Best Practices

1. **Use Hierarchical Topics:** Organize topics in namespaces for clarity
2. **Isolate Failures:** Catch errors in subscriber handlers to prevent affecting others
3. **Monitor Fanout Time:** Track P99 fanout latency for performance regression
4. **Cleanup Subscriptions:** Always unsubscribe during component shutdown
5. **Avoid Circular Dependencies:** Publishers should not subscribe to their own topics

## Common Mistakes

1. **Forgetting to Subscribe:** Subscribers must explicitly subscribe to receive messages
2. **Blocking in Handlers:** Subscriber handlers should be fast to avoid delaying other subscribers
3. **Leaking Subscriptions:** Not unsubscribing causes memory leaks in broker
4. **Assuming Ordering:** Messages may arrive out of order; use sequence numbers if needed
5. **Ignoring Errors:** Publisher should handle broker unavailability gracefully

## Advanced: Wildcard Subscriptions

If your broker supports wildcard subscriptions:

```rust
// Subscribe to all temperature sensors
broker.subscribe("sensor.temperature.*", addr).await?;

// Subscribe to all sensors in room1
broker.subscribe("sensor.*.room1", addr).await?;

// Subscribe to everything (use with caution)
broker.subscribe("*", addr).await?;
```

**Note:** Wildcard support depends on broker implementation. Check documentation for your specific broker.

## Related Patterns

- **Request-Response:** For one-to-one communication with replies (see [Request-Response Pattern](./request-response-pattern.md))
- **Event Sourcing:** For event-driven architectures with persistent event logs
- **Message Filtering:** For subscriber-side filtering of events
- **State Management:** For sharing state across components (see [State Management Patterns](../explanation/state-management-patterns.md))

## References

- **ADR-WASM-009:** Component Communication Model (Pattern 3: Pub-Sub)
- **Task 6.2 Benchmarks:** `benches/messaging_benchmarks.rs`
- **API Reference:** [Message Routing](../reference/message-routing.md)
- **airssys-rt:** MessageBroker implementation
