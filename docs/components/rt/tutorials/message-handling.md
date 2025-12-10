# Tutorial: Message Handling Patterns

**Learning Objectives:**
- Implement request-reply messaging
- Use fire-and-forget pattern
- Handle message routing with brokers
- Implement pub-sub pattern

**Prerequisites:**
- Complete [Your First Actor](./actor-creation.md) tutorial
- Understanding of async/await in Rust
- Familiarity with actor basics

**Estimated time:** 35-40 minutes

---

## What You'll Build

A multi-actor system demonstrating:
- **Request-Reply**: Get responses from actors
- **Fire-and-Forget**: Send notifications without waiting
- **Pub-Sub**: Broadcast to multiple subscribers

**By the end**, you'll understand all core messaging patterns in AirsSys RT.

---

## Step 1: Understand Message Semantics

AirsSys RT supports three message passing semantics:

### Fire-and-Forget (~600ns)
```rust
// Send message, don't wait for response
actor.send(Message::DoWork).await?;
// Continue immediately
```

### Request-Reply (~737ns)
```rust
// Send message, wait for response
let result = actor.ask(Message::GetData).await?;
// Use result
```

### Broadcast (395ns per subscriber)
```rust
// Send to all subscribers of a topic
broker.publish("topic", Message::Update).await?;
// All subscribers receive it
```

Let's implement each pattern!

---

## Step 2: Set Up the Scenario

We'll build a simple order processing system:

**Actors:**
- `OrderProcessor` - Processes orders (request-reply)
- `InventoryChecker` - Checks stock (request-reply)
- `NotificationService` - Sends notifications (fire-and-forget)
- `Analytics` - Tracks metrics (pub-sub subscriber)
- `Logger` - Logs events (pub-sub subscriber)

**Message flow:**
```
OrderProcessor -> (ask) -> InventoryChecker -> (reply) -> OrderProcessor
              \-> (send) -> NotificationService
              \-> (publish) -> [Analytics, Logger]
```

---

## Step 3: Define Message Types

Create comprehensive message types for all actors:

```rust
use airssys_rt::prelude::*;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

// Inventory checker messages - uses oneshot channels for request-reply
#[derive(Debug, Clone)]
pub enum InventoryMessage {
    CheckStock { 
        item: String, 
        quantity: u32,
        reply: oneshot::Sender<InventoryResult>,  // Channel for reply
    },
}

impl Message for InventoryMessage {
    const MESSAGE_TYPE: &'static str = "inventory";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InventoryResult {
    Available { item: String, in_stock: u32 },
    Unavailable { item: String, needed: u32, available: u32 },
}

// Order processor messages - supports both patterns
#[derive(Debug, Clone)]
pub enum OrderMessage {
    ProcessOrder { 
        order_id: String, 
        item: String, 
        quantity: u32,
        reply: oneshot::Sender<OrderResult>,  // Request-reply
    },
    GetOrderStatus { 
        order_id: String,
        reply: oneshot::Sender<OrderResult>,  // Request-reply
    },
}

impl Message for OrderMessage {
    const MESSAGE_TYPE: &'static str = "order";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderResult {
    OrderProcessed { order_id: String, status: String },
    OrderStatus { order_id: String, status: String },
    Error(String),
}

// Notification messages (fire-and-forget - no reply channel)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationMessage {
    OrderPlaced { order_id: String, customer: String },
    OrderShipped { order_id: String },
}

impl Message for NotificationMessage {
    const MESSAGE_TYPE: &'static str = "notification";
}

// Analytics events (pub-sub - no reply channel)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalyticsEvent {
    OrderCreated { order_id: String, amount: f64 },
    OrderCompleted { order_id: String, duration_ms: u64 },
}

impl Message for AnalyticsEvent {
    const MESSAGE_TYPE: &'static str = "analytics";
}
```

**Key design points:**
- **Request-Reply pattern**: Use `oneshot::Sender<T>` in message variants for replies
- **Fire-and-Forget**: Messages without reply channels
- **Message trait**: Only requires `const MESSAGE_TYPE`, no associated Result type

---

## Step 4: Implement Request-Reply Pattern

The inventory checker uses request-reply via oneshot channels:

```rust
use async_trait::async_trait;
use std::collections::HashMap;
use std::fmt;

pub struct InventoryChecker {
    stock: HashMap<String, u32>,
}

#[derive(Debug)]
pub struct InventoryError(String);

impl fmt::Display for InventoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Inventory error: {}", self.0)
    }
}

impl std::error::Error for InventoryError {}

impl InventoryChecker {
    pub fn new() -> Self {
        let mut stock = HashMap::new();
        stock.insert("laptop".to_string(), 10);
        stock.insert("mouse".to_string(), 50);
        stock.insert("keyboard".to_string(), 30);
        
        Self { stock }
    }

    fn check_availability(&self, item: &str, quantity: u32) -> InventoryResult {
        match self.stock.get(item) {
            Some(&available) if available >= quantity => {
                InventoryResult::Available {
                    item: item.to_string(),
                    in_stock: available,
                }
            }
            Some(&available) => {
                InventoryResult::Unavailable {
                    item: item.to_string(),
                    needed: quantity,
                    available,
                }
            }
            None => {
                InventoryResult::Unavailable {
                    item: item.to_string(),
                    needed: quantity,
                    available: 0,
                }
            }
        }
    }
}

#[async_trait]
impl Actor for InventoryChecker {
    type Message = InventoryMessage;
    type Error = InventoryError;

    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        match message {
            InventoryMessage::CheckStock { item, quantity, reply } => {
                println!("  📦 Checking inventory: {item} x{quantity}");
                let result = self.check_availability(&item, quantity);
                context.record_message();
                
                // Send reply through oneshot channel (ignore if receiver dropped)
                let _ = reply.send(result);
                Ok(())
            }
        }
    }
}
```

**Request-Reply pattern:**
- ✅ Actor always returns `Result<(), E>` (unit type)
- ✅ Response sent via `oneshot::Sender<T>` embedded in message
- ✅ Caller waits on `oneshot::Receiver<T>` for response
- ✅ ~737ns roundtrip latency

---

## Step 5: Implement Fire-and-Forget Pattern

The notification service doesn't send responses:

```rust
pub struct NotificationService {
    notifications_sent: usize,
}

#[derive(Debug)]
pub struct NotificationError(String);

impl fmt::Display for NotificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Notification error: {}", self.0)
    }
}

impl std::error::Error for NotificationError {}

impl NotificationService {
    pub fn new() -> Self {
        Self { notifications_sent: 0 }
    }
}

#[async_trait]
impl Actor for NotificationService {
    type Message = NotificationMessage;
    type Error = NotificationError;

    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {  // Returns unit ()
        match message {
            NotificationMessage::OrderPlaced { order_id, customer } => {
                println!("  📧 Notification: Order {order_id} placed by {customer}");
                self.notifications_sent += 1;
            }
            NotificationMessage::OrderShipped { order_id } => {
                println!("  📧 Notification: Order {order_id} shipped");
                self.notifications_sent += 1;
            }
        }
        
        context.record_message();
        Ok(())  // No result to return
    }
}
```

**Fire-and-Forget pattern:**
- ✅ Returns `Result<(), E>` (unit type)
- ✅ Caller doesn't wait for processing
- ✅ ~600ns send latency (no response wait)

---

## Step 6: Implement Pub-Sub Pattern

Analytics and Logger subscribe to events:

```rust
pub struct AnalyticsService {
    events_processed: usize,
}

#[derive(Debug)]
pub struct AnalyticsError(String);

impl fmt::Display for AnalyticsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Analytics error: {}", self.0)
    }
}

impl std::error::Error for AnalyticsError {}

impl AnalyticsService {
    pub fn new() -> Self {
        Self { events_processed: 0 }
    }
}

#[async_trait]
impl Actor for AnalyticsService {
    type Message = AnalyticsEvent;
    type Error = AnalyticsError;

    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        match message {
            AnalyticsEvent::OrderCreated { order_id, amount } => {
                println!("  📊 Analytics: Order {order_id} created (${amount:.2})");
                self.events_processed += 1;
            }
            AnalyticsEvent::OrderCompleted { order_id, duration_ms } => {
                println!("  📊 Analytics: Order {order_id} completed ({duration_ms}ms)");
                self.events_processed += 1;
            }
        }
        
        context.record_message();
        Ok(())
    }
}

pub struct LoggerService {
    logs_written: usize,
}

#[derive(Debug)]
pub struct LoggerError(String);

impl fmt::Display for LoggerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Logger error: {}", self.0)
    }
}

impl std::error::Error for LoggerError {}

impl LoggerService {
    pub fn new() -> Self {
        Self { logs_written: 0 }
    }
}

#[async_trait]
impl Actor for LoggerService {
    type Message = AnalyticsEvent;
    type Error = LoggerError;

    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        match message {
            AnalyticsEvent::OrderCreated { order_id, amount } => {
                println!("  📝 Log: [INFO] Order {order_id} created amount=${amount:.2}");
                self.logs_written += 1;
            }
            AnalyticsEvent::OrderCompleted { order_id, duration_ms } => {
                println!("  📝 Log: [INFO] Order {order_id} completed duration={duration_ms}ms");
                self.logs_written += 1;
            }
        }
        
        context.record_message();
        Ok(())
    }
}
```

**Pub-Sub pattern:**
- ✅ Multiple actors subscribe to same topic
- ✅ Publisher doesn't know subscribers
- ✅ ~395ns per subscriber (linear scaling)

---

## Step 7: Orchestrate the System

Now combine all patterns in the order processor:

```rust
pub struct OrderProcessor {
    orders: HashMap<String, String>,
}

#[derive(Debug)]
pub struct OrderError(String);

impl fmt::Display for OrderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Order error: {}", self.0)
    }
}

impl std::error::Error for OrderError {}

impl OrderProcessor {
    pub fn new() -> Self {
        Self {
            orders: HashMap::new(),
        }
    }
}

#[async_trait]
impl Actor for OrderProcessor {
    type Message = OrderMessage;
    type Error = OrderError;

    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        match message {
            OrderMessage::ProcessOrder { order_id, item, quantity, reply } => {
                println!("\n🛒 Processing order: {order_id} ({item} x{quantity})");

                // 1. Request-Reply: Check inventory
                // (In real implementation, would use actor reference)
                // For now, simulate the response
                println!("  ✓ Inventory check passed");

                // 2. Fire-and-Forget: Send notification
                // (In real implementation, would send to notification actor)
                println!("  ✓ Notification sent");

                // 3. Pub-Sub: Publish analytics event
                // (In real implementation, would use broker.publish())
                println!("  ✓ Analytics event published");

                // Update state
                self.orders.insert(order_id.clone(), "completed".to_string());

                context.record_message();
                
                // Send reply through oneshot channel
                let result = OrderResult::OrderProcessed {
                    order_id,
                    status: "completed".to_string(),
                };
                let _ = reply.send(result);
                Ok(())
            }

            OrderMessage::GetOrderStatus { order_id, reply } => {
                let status = self.orders
                    .get(&order_id)
                    .cloned()
                    .unwrap_or_else(|| "not_found".to_string());

                context.record_message();
                
                let result = OrderResult::OrderStatus { 
                    order_id, 
                    status 
                };
                let _ = reply.send(result);
                Ok(())
            }
        }
    }
}
```

---

## Step 8: Run the Complete System

Test all three patterns together:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Message Handling Patterns Demo ===\n");

    // Create all actors
    let mut order_processor = OrderProcessor::new();
    let mut inventory = InventoryChecker::new();
    let mut notifier = NotificationService::new();
    let mut analytics = AnalyticsService::new();
    let mut logger = LoggerService::new();

    // Setup contexts
    let broker = InMemoryMessageBroker::new();
    
    let mut order_ctx = ActorContext::new(
        ActorAddress::named("order_processor"),
        broker.clone(),
    );
    let mut inv_ctx = ActorContext::new(
        ActorAddress::named("inventory"),
        broker.clone(),
    );
    let mut notif_ctx = ActorContext::new(
        ActorAddress::named("notifier"),
        broker.clone(),
    );
    let mut analytics_ctx = ActorContext::new(
        ActorAddress::named("analytics"),
        broker.clone(),
    );
    let mut logger_ctx = ActorContext::new(
        ActorAddress::named("logger"),
        broker.clone(),
    );

    // Start all actors
    order_processor.pre_start(&mut order_ctx).await?;
    inventory.pre_start(&mut inv_ctx).await?;
    notifier.pre_start(&mut notif_ctx).await?;
    analytics.pre_start(&mut analytics_ctx).await?;
    logger.pre_start(&mut logger_ctx).await?;

    println!("Pattern 1: Request-Reply\n");
    println!("Checking inventory for laptop x2...");
    
    // Create oneshot channel for reply
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    
    let inv_msg = InventoryMessage::CheckStock {
        item: "laptop".to_string(),
        quantity: 2,
        reply: reply_tx,
    };
    
    // Send message to actor
    inventory.handle_message(inv_msg, &mut inv_ctx).await?;
    
    // Wait for reply
    let result = reply_rx.await?;
    println!("  Result: {result:?}\n");

    println!("\nPattern 2: Fire-and-Forget\n");
    println!("Sending notification...");
    let notif_msg = NotificationMessage::OrderPlaced {
        order_id: "ORD-001".to_string(),
        customer: "Alice".to_string(),
    };
    notifier.handle_message(notif_msg, &mut notif_ctx).await?;
    println!("  ✓ Sent (no response)\n");

    println!("\nPattern 3: Pub-Sub\n");
    println!("Publishing analytics event...");
    let event = AnalyticsEvent::OrderCreated {
        order_id: "ORD-001".to_string(),
        amount: 1999.99,
    };
    // Both subscribers receive the event
    analytics.handle_message(event.clone(), &mut analytics_ctx).await?;
    logger.handle_message(event, &mut logger_ctx).await?;
    println!("  ✓ Published to 2 subscribers\n");

    println!("\nPattern Combination: Full Order Flow\n");
    
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    let order_msg = OrderMessage::ProcessOrder {
        order_id: "ORD-002".to_string(),
        item: "laptop".to_string(),
        quantity: 1,
        reply: reply_tx,
    };
    order_processor.handle_message(order_msg, &mut order_ctx).await?;
    let result = reply_rx.await?;
    println!("  Final result: {result:?}\n");

    // Cleanup
    order_processor.post_stop(&mut order_ctx).await?;
    inventory.post_stop(&mut inv_ctx).await?;
    notifier.post_stop(&mut notif_ctx).await?;
    analytics.post_stop(&mut analytics_ctx).await?;
    logger.post_stop(&mut logger_ctx).await?;

    println!("=== Demo Complete ===");
    Ok(())
}
```

---

## Step 9: Run and Observe

```bash
cargo run
```

**Expected output:**

```
=== Message Handling Patterns Demo ===

Pattern 1: Request-Reply

Checking inventory for laptop x2...
  📦 Checking inventory: laptop x2
  Result: Available { item: "laptop", in_stock: 10 }

Pattern 2: Fire-and-Forget

Sending notification...
  📧 Notification: Order ORD-001 placed by Alice
  ✓ Sent (no response)

Pattern 3: Pub-Sub

Publishing analytics event...
  📊 Analytics: Order ORD-001 created ($1999.99)
  📝 Log: [INFO] Order ORD-001 created amount=$1999.99
  ✓ Published to 2 subscribers

Pattern Combination: Full Order Flow

🛒 Processing order: ORD-002 (laptop x1)
  ✓ Inventory check passed
  ✓ Notification sent
  ✓ Analytics event published
  Final result: OrderProcessed { order_id: "ORD-002", status: "completed" }

=== Demo Complete ===
```

---

## Understanding the Patterns

### Request-Reply (~737ns roundtrip)

**When to use:**
- ✅ Need response data
- ✅ Sequential workflow (step depends on result)
- ✅ Synchronous operations

**Example use cases:**
- Database queries
- Validation checks
- Configuration lookups

**Performance:**
- Latency: 737ns (direct), 917ns (via broker)
- Throughput: 1.36M msgs/sec
- Memory: Minimal (stack-allocated response)

### Fire-and-Forget (~600ns)

**When to use:**
- ✅ No response needed
- ✅ Asynchronous notifications
- ✅ Side effects (logging, metrics)

**Example use cases:**
- Sending emails
- Writing logs
- Updating caches

**Performance:**
- Latency: ~600ns (no response wait)
- Throughput: Higher than request-reply
- Memory: No response storage needed

### Pub-Sub (395ns per subscriber)

**When to use:**
- ✅ Multiple subscribers
- ✅ Broadcast notifications
- ✅ Event-driven architecture

**Example use cases:**
- Event sourcing
- Multi-service notifications
- Real-time updates

**Performance:**
- Latency: 395ns per subscriber (linear)
- Scaling: O(n) with subscriber count
- Memory: One message copy per subscriber

---

## Best Practices

### ✅ Choose the Right Pattern

```rust
// Request-Reply: Need the result
let balance = bank_actor.ask(GetBalance { account_id }).await?;

// Fire-and-Forget: Don't need response
logger.send(LogMessage { level: Info, text }).await?;

// Pub-Sub: Multiple subscribers
broker.publish("orders", OrderCreated { order_id }).await?;
```

### ✅ Handle Timeouts

```rust
use tokio::time::{timeout, Duration};

let result = timeout(
    Duration::from_secs(5),
    actor.ask(SlowOperation).await,
).await??;
```

### ✅ Use Type-Safe Replies

```rust
// Clear result types via oneshot channels
#[derive(Debug, Clone)]
enum UserMessage {
    GetUser { 
        id: UserId,
        reply: oneshot::Sender<User>,  // Type-safe reply
    },
}

// Actor always returns Result<(), Error>
async fn handle_message(...) -> Result<(), UserError> {
    // Send typed reply through channel
    let _ = reply.send(user);
    Ok(())
}
```

### ✅ Avoid Blocking

```rust
// ❌ Don't block in message handlers
async fn handle_message(...) {
    std::thread::sleep(Duration::from_secs(1));  // ❌ Blocks actor
}

// ✅ Use async operations
async fn handle_message(...) {
    tokio::time::sleep(Duration::from_secs(1)).await;  // ✅ Async
}
```

---

## Common Mistakes

### ❌ Waiting for Fire-and-Forget

```rust
// ❌ Fire-and-forget shouldn't have reply channel
#[derive(Debug, Clone)]
enum LogMessage {
    Log { 
        text: String,
        reply: oneshot::Sender<()>,  // ❌ Unnecessary
    },
}

// ✅ No reply channel for fire-and-forget
#[derive(Debug, Clone)]
enum LogMessage {
    Log { text: String },  // ✅ No response
}
```

### ❌ Pub-Sub for Request-Reply

```rust
// ❌ Using pub-sub for request-reply
broker.publish("get_user", GetUser { id }).await?;
// How do you get the response? ❌

// ✅ Use direct messaging
let user = user_actor.ask(GetUser { id }).await?;  // ✅ Clear
```

### ❌ Request-Reply in Hot Path

```rust
// ❌ Synchronous in tight loop
for item in items {
    let result = actor.ask(Process { item }).await?;  // ❌ Slow
}

// ✅ Fire-and-forget or batch
for item in items {
    actor.send(Process { item }).await?;  // ✅ Fast
}
```

---

## Next Steps

Congratulations! You've mastered message handling patterns:
- ✅ Request-Reply for synchronous operations
- ✅ Fire-and-Forget for async notifications
- ✅ Pub-Sub for event broadcasting

### Continue Learning:
- **[Supervision Setup Tutorial](./supervision-setup.md)** - Add fault tolerance
- **[Message Passing Guide](../guides/message-passing.md)** - Advanced patterns
- **[Performance Design](../explanation/performance-design.md)** - Optimization strategies

### Explore Examples:
- `examples/actor_basic.rs` - Simple messaging
- `examples/monitoring_basic.rs` - Pub-sub pattern
- [API Reference: Messaging](../reference/api/messaging.md) - Complete messaging API

---

## Quick Reference

### Pattern Selection Guide

| Pattern | Latency | Use When | Example |
|---------|---------|----------|---------|
| **Request-Reply** | 737ns | Need response | Database query |
| **Fire-and-Forget** | 600ns | No response needed | Send notification |
| **Pub-Sub** | 395ns/sub | Multiple receivers | Event broadcast |

### Message Type Template

```rust
// Request-Reply with oneshot channel
#[derive(Debug, Clone)]
enum QueryMessage { 
    GetData { 
        id: String,
        reply: oneshot::Sender<QueryResult>,  // ← Reply channel
    } 
}
impl Message for QueryMessage {
    const MESSAGE_TYPE: &'static str = "query";
}

// Fire-and-Forget (no reply channel)
#[derive(Debug, Clone, Serialize, Deserialize)]
enum CommandMessage { 
    DoWork { task: String }  // ← No reply
}
impl Message for CommandMessage {
    const MESSAGE_TYPE: &'static str = "command";
}

// Pub-Sub (no reply channel)
#[derive(Debug, Clone, Serialize, Deserialize)]
enum EventMessage { 
    DataChanged { id: String }  // ← Broadcast
}
impl Message for EventMessage {
    const MESSAGE_TYPE: &'static str = "event";
}
```

**Ready for fault tolerance?** Continue to [Supervision Setup Tutorial](./supervision-setup.md)!
