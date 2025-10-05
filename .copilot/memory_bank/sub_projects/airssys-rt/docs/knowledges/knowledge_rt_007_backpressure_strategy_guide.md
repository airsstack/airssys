# KNOWLEDGE-RT-007: Backpressure Strategy Behavior and Selection Guide

**Knowledge Type**: Technical Explanation  
**Category**: Mailbox System  
**Created**: 2025-10-05  
**Related**: RT-TASK-003, ADR-RT-003, KNOWLEDGE-RT-006  
**Complexity**: Intermediate  

---

## Overview

This document explains the behavioral differences between the three backpressure strategies (`Block`, `Drop`, `Error`) in the airssys-rt mailbox system and provides guidance on when to use each strategy.

## Background

Backpressure strategies determine how the system handles message delivery when a bounded mailbox reaches capacity. The choice of strategy affects:
- Message delivery guarantees
- System throughput
- Sender behavior
- System responsiveness under load

**Related Decision**: See ADR-RT-003 for why we simplified from 4 strategies to 3.

---

## Strategy Comparison

### Block Strategy

**Behavior:**
- **Waits asynchronously** for space to become available in the mailbox
- Uses `sender.send(envelope).await` - async blocking operation
- The sender **will not return** until the message is successfully queued
- No messages are lost (guaranteed delivery)

**Implementation:**
```rust
Self::Block => {
    // Wait for space (async blocking)
    sender
        .send(envelope)
        .await
        .map_err(|_| MailboxError::Closed)?;
    Ok(())
}
```

**When to Use:**
- ✅ **Critical messages** that must be delivered (e.g., system commands, shutdown signals)
- ✅ **Financial transactions** where losing a message is unacceptable
- ✅ **State updates** that must be processed in order
- ✅ **Command/control messages** for actor lifecycle
- ✅ **Coordination messages** between actors in distributed protocols

**Trade-offs:**
- ⚠️ **Can cause sender delays** if receiver is slow
- ⚠️ **Backpressure propagates** up the call chain
- ⚠️ **May create bottlenecks** if many senders are blocked
- ✅ **No data loss** - every message is eventually delivered
- ✅ **Natural flow control** - fast senders slow down to match receiver

**Example Scenario:**
```rust
// Payment processing - cannot afford to lose transactions
let (mailbox, sender) = BoundedMailbox::with_backpressure(
    100,
    BackpressureStrategy::Block
);

// This will wait if mailbox is full, ensuring the payment is processed
sender.send(PaymentMessage { 
    transaction_id: "TX123",
    amount: 100.0 
}).await?;
```

---

### Drop Strategy

**Behavior:**
- **Immediately returns Ok(())** even if message is dropped
- Uses `sender.try_send()` - non-blocking operation
- If mailbox is full, silently discards the incoming message
- Sender continues without waiting

**Implementation:**
```rust
Self::Drop => {
    // Drop incoming message if mailbox is full
    match sender.try_send(envelope) {
        Ok(()) => Ok(()),
        Err(mpsc::error::TrySendError::Full(_)) => {
            // Silently drop the incoming message
            Ok(())
        }
        Err(mpsc::error::TrySendError::Closed(_)) => Err(MailboxError::Closed),
    }
}
```

**When to Use:**
- ✅ **Low-priority messages** that can be safely lost (e.g., debug logs)
- ✅ **Best-effort delivery** scenarios (e.g., metrics, telemetry)
- ✅ **Sampling/monitoring data** where some data loss is acceptable
- ✅ **Fire-and-forget** notifications
- ✅ **Non-critical UI updates** in responsive systems

**Trade-offs:**
- ✅ **Never blocks** the sender - always fast
- ✅ **No backpressure propagation** to sender
- ✅ **High throughput** maintained even under load
- ⚠️ **Silent data loss** when mailbox is full
- ⚠️ **No delivery guarantees**
- ⚠️ **No feedback** to sender about dropped messages

**Example Scenario:**
```rust
// Logging actor - it's okay to drop some log messages under load
let (mailbox, sender) = BoundedMailbox::with_backpressure(
    1000,
    BackpressureStrategy::Drop
);

// Returns immediately, even if log message is dropped
sender.send(LogMessage { 
    level: Debug, 
    msg: "Processing request #42" 
}).await?;
```

---

### Error Strategy

**Behavior:**
- **Immediately returns an error** if mailbox is full
- Uses `sender.try_send()` - non-blocking operation
- Sender receives `MailboxError::Full` and can decide how to handle it
- No messages are lost silently - sender has explicit knowledge

**Implementation:**
```rust
Self::Error => {
    // Return error immediately if full
    sender.try_send(envelope).map_err(|e| match e {
        mpsc::error::TrySendError::Full(_) => MailboxError::Full {
            capacity: sender.max_capacity(),
        },
        mpsc::error::TrySendError::Closed(_) => MailboxError::Closed,
    })
}
```

**When to Use:**
- ✅ **Request/response patterns** where sender needs to know about failure
- ✅ **API calls** where the caller should retry or handle errors
- ✅ **Synchronous operations** that need immediate feedback
- ✅ **Circuit breaker patterns** where failures should be tracked
- ✅ **Rate limiting scenarios** where rejection is part of design

**Trade-offs:**
- ✅ **Explicit failure** - sender knows immediately
- ✅ **No silent data loss** - sender can retry or handle error
- ✅ **Fast feedback** - non-blocking with clear result
- ⚠️ **Sender must handle error** - requires error handling logic
- ⚠️ **No automatic retry** - sender responsibility

**Example Scenario:**
```rust
// API request handler - needs to return error to client
let (mailbox, sender) = BoundedMailbox::with_backpressure(
    50,
    BackpressureStrategy::Error
);

// Returns error if full, allowing HTTP 503 response
match sender.send(ApiRequest { endpoint: "/users" }).await {
    Ok(()) => Ok(HttpResponse::Accepted),
    Err(MailboxError::Full { .. }) => Ok(HttpResponse::ServiceUnavailable),
    Err(e) => Err(e.into()),
}
```

---

## Side-by-Side Comparison

| Aspect | **Block** | **Drop** | **Error** |
|--------|-----------|----------|-----------|
| **Sender behavior** | Waits for space | Returns immediately | Returns immediately |
| **Async operation** | `sender.send().await` | `sender.try_send()` | `sender.try_send()` |
| **When mailbox full** | Suspends until space | Drops message silently | Returns error |
| **Message delivery** | Guaranteed | Best-effort | Explicit failure |
| **Backpressure** | Propagates to sender | Absorbed locally | Explicit to sender |
| **Performance impact** | Can slow sender | Always fast | Always fast |
| **Data loss** | ❌ Never | ✅ Possible (silent) | ❌ Never (explicit) |
| **Sender feedback** | Implicit (delay) | None | Explicit (error) |
| **Error handling** | Not needed | Not needed | Required |
| **Use case** | Critical messages | Low-priority | Request/response |

---

## Priority-Based Strategy Selection

The `for_priority()` method provides automatic strategy selection based on message priority:

```rust
pub fn for_priority(priority: MessagePriority) -> Self {
    match priority {
        MessagePriority::Critical => Self::Block,  // Must deliver
        MessagePriority::High => Self::Block,      // Important
        MessagePriority::Normal => Self::Error,    // Needs feedback
        MessagePriority::Low => Self::Drop,        // Can lose
    }
}
```

**Rationale:**

1. **Critical/High → Block**
   - These messages are important enough to wait for delivery
   - System correctness depends on processing them
   - Acceptable to slow down sender to ensure delivery

2. **Normal → Error**
   - Sender needs to know if delivery failed
   - Can implement retry logic or alternative handling
   - Typical for request/response patterns

3. **Low → Drop**
   - Messages can be safely lost without impacting correctness
   - Throughput more important than individual message delivery
   - Typical for telemetry, logging, monitoring

---

## Real-World Decision Tree

### Ask These Questions:

**1. Can the system function correctly if this message is lost?**
- ❌ No → Use `Block` (critical path)
- ✅ Yes → Continue to question 2

**2. Does the sender need to know if delivery failed?**
- ✅ Yes → Use `Error` (explicit feedback needed)
- ❌ No → Use `Drop` (fire-and-forget)

**3. Is this a request/response pattern?**
- ✅ Yes → Use `Error` (client expects response)
- ❌ No → Continue based on priority

**4. What's more important: delivery guarantee or throughput?**
- Delivery → Use `Block`
- Throughput → Use `Drop`

---

## Real-World Example: Web Server Actor

```rust
// Critical: Shutdown signal - must be delivered
let shutdown_sender = create_mailbox(BackpressureStrategy::Block);
shutdown_sender.send(ShutdownMessage).await?; // Will wait if needed

// Normal: HTTP request - needs error feedback
let request_sender = create_mailbox(BackpressureStrategy::Error);
match request_sender.send(HttpRequest { .. }).await {
    Ok(()) => { /* queued for processing */ }
    Err(MailboxError::Full { .. }) => { /* return 503 Service Unavailable */ }
}

// Low Priority: Request metrics - nice to have, not critical
let metrics_sender = create_mailbox(BackpressureStrategy::Drop);
metrics_sender.send(MetricsMessage { latency: 42 }).await?; // May drop, returns immediately
```

**Under Load (Mailbox Full):**
- **Block**: Shutdown message waits in line → guaranteed delivery → server will shutdown
- **Error**: HTTP request gets error → return 503 to client → client can retry
- **Drop**: Metrics message is silently discarded → lose some stats, but server keeps working

---

## Performance Characteristics

### Block Strategy
- **Latency**: Variable (depends on mailbox availability)
- **Throughput**: Limited by receiver speed
- **CPU**: Low (efficient async waiting)
- **Memory**: Bounded by mailbox capacity

### Drop Strategy
- **Latency**: Constant (always O(1))
- **Throughput**: Maximum (never waits)
- **CPU**: Minimal (quick try_send)
- **Memory**: Bounded by mailbox capacity

### Error Strategy
- **Latency**: Constant (always O(1))
- **Throughput**: High (never waits, but sender must handle errors)
- **CPU**: Minimal (quick try_send)
- **Memory**: Bounded by mailbox capacity

---

## Common Patterns

### Pattern 1: Graceful Degradation
```rust
// Try Error first, fall back to logging on failure
match sender.send(message).await {
    Ok(()) => { /* success */ }
    Err(MailboxError::Full { .. }) => {
        log::warn!("Actor overloaded, dropping message");
        // Optionally send to dead letter queue
    }
}
```

### Pattern 2: Retry with Backoff
```rust
// Use Error strategy with exponential backoff
let mut attempts = 0;
loop {
    match sender.send(message.clone()).await {
        Ok(()) => break,
        Err(MailboxError::Full { .. }) if attempts < 3 => {
            attempts += 1;
            tokio::time::sleep(Duration::from_millis(100 * 2_u64.pow(attempts))).await;
        }
        Err(e) => return Err(e),
    }
}
```

### Pattern 3: Priority Queue Emulation
```rust
// Use Block for high priority, Drop for low priority
let strategy = if message.is_high_priority() {
    BackpressureStrategy::Block
} else {
    BackpressureStrategy::Drop
};

let (mailbox, sender) = BoundedMailbox::with_backpressure(100, strategy);
```

---

## Key Insights

### Block
> *"This message is so important, I'm willing to wait for delivery"*

**Best for**: Critical paths, state consistency, coordination

### Drop
> *"Throughput is more important than delivery guarantees for this message"*

**Best for**: Telemetry, logging, best-effort notifications

### Error
> *"I need to know immediately if delivery failed so I can handle it"*

**Best for**: Request/response, APIs, client-facing operations

---

## Anti-Patterns to Avoid

### ❌ Don't Use Block for High-Volume Telemetry
```rust
// BAD: Will slow down entire system under load
let (mailbox, sender) = BoundedMailbox::with_backpressure(
    100,
    BackpressureStrategy::Block  // ❌ Wrong!
);

for metric in high_frequency_metrics {
    sender.send(metric).await?;  // Will block frequently
}
```

**Fix**: Use `Drop` for telemetry data.

### ❌ Don't Use Drop for Critical State Updates
```rust
// BAD: May lose important state changes
let (mailbox, sender) = BoundedMailbox::with_backpressure(
    10,
    BackpressureStrategy::Drop  // ❌ Wrong!
);

sender.send(AccountBalanceUpdate { new_balance: 1000.0 }).await?;  // Might drop!
```

**Fix**: Use `Block` for state consistency.

### ❌ Don't Use Error Without Handling
```rust
// BAD: Ignoring errors defeats the purpose
let result = sender.send(message).await;  // ❌ Not checking result
```

**Fix**: Handle errors explicitly when using `Error` strategy.

---

## Testing Strategies

### Testing Block Behavior
```rust
#[tokio::test]
async fn test_block_waits_for_space() {
    let (mut mailbox, sender) = BoundedMailbox::with_backpressure(
        1,
        BackpressureStrategy::Block
    );
    
    // Fill mailbox
    sender.send(msg1).await.unwrap();
    
    // Spawn task that will block
    let handle = tokio::spawn(async move {
        sender.send(msg2).await  // Blocks here
    });
    
    // Receive to make space
    mailbox.recv().await;
    
    // Blocked send should now complete
    handle.await.unwrap().unwrap();
}
```

### Testing Drop Behavior
```rust
#[tokio::test]
async fn test_drop_silently_discards() {
    let (mailbox, sender) = BoundedMailbox::with_backpressure(
        1,
        BackpressureStrategy::Drop
    );
    
    // Fill mailbox
    sender.send(msg1).await.unwrap();
    
    // This returns Ok but message is dropped
    sender.send(msg2).await.unwrap();
    
    // Only first message should be in mailbox
    assert_eq!(mailbox.len(), 1);
}
```

### Testing Error Behavior
```rust
#[tokio::test]
async fn test_error_returns_failure() {
    let (mailbox, sender) = BoundedMailbox::with_backpressure(
        1,
        BackpressureStrategy::Error
    );
    
    // Fill mailbox
    sender.send(msg1).await.unwrap();
    
    // This should return error
    let result = sender.send(msg2).await;
    assert!(matches!(result, Err(MailboxError::Full { .. })));
}
```

---

## References

- **ADR-RT-003**: Backpressure Strategy Simplification
- **RT-TASK-003**: Mailbox System Implementation
- **KNOWLEDGE-RT-006**: Mailbox System Implementation Guide
- **§6.1**: YAGNI Principles (workspace/shared_patterns.md)
- **tokio::sync::mpsc**: Channel implementation details

---

## Version History

- **v1.0** (2025-10-05): Initial documentation with Block/Drop/Error strategies
  - Based on ADR-RT-003 simplification decision
  - Removed DropOldest/DropNewest after tokio mpsc limitation discovery
