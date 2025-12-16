# Request-Response Communication Pattern

**Category:** How-To Guide (Task-Oriented)  
**Purpose:** This guide shows you how to implement request-response communication between ComponentActors.

## Overview

Request-response is a fundamental communication pattern where one component sends a request and waits for a response from another component. This pattern is essential for synchronous-style interactions in distributed component systems.

**Key Features:**
- Unique correlation ID per request (UUID v4)
- Automatic timeout handling
- Type-safe response delivery via oneshot channel
- Lock-free correlation tracking (DashMap-based)

**Performance:** Request-response latency is 3.18µs (measured in Task 6.2 `messaging_benchmarks.rs` benchmark `bench_correlation_tracking_overhead`, macOS M1, 100 samples).

## When to Use Request-Response

**Use request-response when:**
- You need a reply to a specific request
- The response is required before proceeding
- You want to enforce request timeouts
- One-to-one communication is sufficient

**Don't use request-response when:**
- Broadcasting to multiple components is needed (use pub-sub)
- Fire-and-forget messaging is sufficient
- The response may arrive much later (use event-driven pattern)

## Prerequisites

Before implementing request-response, you should understand:
- ComponentActor basics (see [Your First ComponentActor](../tutorials/your-first-component-actor.md))
- ComponentRegistry for component lookup
- Basic async/await in Rust

## Implementation Steps

### Step 1: Set Up CorrelationTracker

The `CorrelationTracker` manages pending requests and matches responses to their originating requests.

```rust
use airssys_wasm::actor::message::CorrelationTracker;

// Create tracker (usually one per system or per component)
let tracker = CorrelationTracker::new();
```

**Performance Note:** Tracker construction is extremely fast (~7.8ns, measured in Task 6.2 `bench_correlation_tracker_construction`).

### Step 2: Create Request Component

The requester component initiates requests and waits for responses.

```rust
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::oneshot;
use uuid::Uuid;

use airssys_wasm::actor::message::{
    CorrelationTracker, PendingRequest, RequestMessage,
};
use airssys_wasm::core::ComponentId;

struct RequesterComponent {
    tracker: CorrelationTracker,
    target_id: ComponentId,
}

impl RequesterComponent {
    async fn send_request(&self, payload: Vec<u8>) -> Result<Vec<u8>, String> {
        // Generate unique correlation ID
        let corr_id = Uuid::new_v4();
        
        // Create oneshot channel for response
        let (tx, rx) = oneshot::channel();
        
        // Register pending request
        let pending = PendingRequest {
            correlation_id: corr_id,
            response_tx: tx,
            requested_at: tokio::time::Instant::now(),
            timeout: Duration::from_secs(5),
            from: ComponentId::new("requester"),
            to: self.target_id.clone(),
        };
        
        self.tracker.register_pending(pending).await
            .map_err(|e| format!("Failed to register request: {}", e))?;
        
        // Create request message
        let request = RequestMessage::new(
            ComponentId::new("requester"),
            self.target_id.clone(),
            payload,
            5000, // 5s timeout in ms
        );
        
        // Send request via MessageRouter (implementation depends on your setup)
        // router.send_message(&self.target_id, ComponentMessage::Request(request)).await?;
        
        // Wait for response (with timeout)
        match tokio::time::timeout(Duration::from_secs(5), rx).await {
            Ok(Ok(response)) => {
                // Response received
                if response.is_success {
                    Ok(response.payload)
                } else {
                    Err(format!("Request failed: {:?}", response.error_code))
                }
            }
            Ok(Err(_)) => Err("Response channel closed".to_string()),
            Err(_) => Err("Request timeout".to_string()),
        }
    }
}
```

### Step 3: Create Responder Component

The responder component receives requests and sends responses with matching correlation IDs.

```rust
use airssys_wasm::actor::message::{RequestMessage, ResponseMessage};

struct ResponderComponent {
    // Component state
}

impl ResponderComponent {
    async fn handle_request(&self, request: RequestMessage) -> ResponseMessage {
        // Process request
        let result = self.process_payload(&request.payload);
        
        // Create response with same correlation ID
        match result {
            Ok(payload) => ResponseMessage::success(
                request.correlation_id,
                request.to.clone(),  // responder is now "from"
                request.from.clone(), // requester is now "to"
                payload,
            ),
            Err(error_msg) => ResponseMessage::error(
                request.correlation_id,
                request.to.clone(),
                request.from.clone(),
                1000, // error code
                error_msg,
            ),
        }
    }
    
    fn process_payload(&self, payload: &[u8]) -> Result<Vec<u8>, String> {
        // Your business logic here
        Ok(payload.to_vec())
    }
}
```

### Step 4: Integrate with MessageRouter

Connect the request-response pattern to your message routing infrastructure.

```rust
use airssys_wasm::actor::{ComponentMessage, MessageRouter};

// In your component's Actor::handle_message implementation
async fn handle_message(message: ComponentMessage) -> Result<(), WasmError> {
    match message {
        ComponentMessage::Request(request) => {
            // Responder: Handle request and send response
            let response = self.handle_request(request).await;
            
            // Send response back to requester
            router.send_message(
                &response.to,
                ComponentMessage::Response(response)
            ).await?;
        }
        ComponentMessage::Response(response) => {
            // Requester: Resolve pending request
            tracker.resolve(response.correlation_id, response).await?;
        }
        _ => {}
    }
    Ok(())
}
```

## Timeout Handling

### Setting Timeout Duration

```rust
// Short timeout for fast operations
let timeout = Duration::from_millis(100);

// Standard timeout for network operations
let timeout = Duration::from_secs(5);

// Long timeout for batch processing
let timeout = Duration::from_secs(30);
```

### Handling Timeout Errors

```rust
match self.send_request(payload).await {
    Ok(response) => {
        // Process response
    }
    Err(e) if e.contains("timeout") => {
        // Handle timeout specifically
        log::warn!("Request timed out, retrying...");
        // Implement retry logic
    }
    Err(e) => {
        // Handle other errors
        log::error!("Request failed: {}", e);
    }
}
```

## Error Scenarios

### Target Component Stopped

If the target component is stopped or unregistered:

```rust
match tracker.register_pending(pending).await {
    Ok(_) => {}
    Err(WasmError::ComponentNotFound(_)) => {
        // Component doesn't exist
        return Err("Target component not available".to_string());
    }
    Err(e) => return Err(format!("Registration failed: {}", e)),
}
```

### Invalid Response

If the response correlation ID doesn't match any pending request:

```rust
// CorrelationTracker automatically handles this
// tracker.resolve() returns error if correlation ID not found
match tracker.resolve(corr_id, response).await {
    Ok(_) => {}
    Err(WasmError::InvalidCorrelationId(_)) => {
        log::warn!("Received response for unknown request: {}", corr_id);
    }
    Err(e) => log::error!("Resolution failed: {}", e),
}
```

### Response Channel Closed

If the requester component stops before receiving the response:

```rust
// oneshot::Sender::send returns Err if receiver is dropped
match pending.response_tx.send(response) {
    Ok(_) => {}
    Err(_) => {
        // Requester no longer waiting (component stopped or timeout)
        log::debug!("Response channel closed for correlation {}", corr_id);
    }
}
```

## Performance Characteristics

Based on Task 6.2 benchmarks (`messaging_benchmarks.rs`):

| Operation | Latency | Benchmark |
|-----------|---------|-----------|
| Full request-response cycle | 3.18µs | `bench_correlation_tracking_overhead` |
| CorrelationTracker construction | 7.8ns | `bench_correlation_tracker_construction` |
| RequestMessage construction | <10µs | `bench_request_message_construction` |
| Message routing overhead | ~211ns | Inferred from routing benchmarks |

**Test Conditions:** macOS M1, 100 samples, 95% confidence interval

### Optimization Tips

1. **Reuse CorrelationTracker:** Create one tracker and share it across components (it's Clone and thread-safe)
2. **Batch Requests:** If sending multiple requests, don't wait for each response before sending the next
3. **Tune Timeouts:** Use shorter timeouts for fast operations to fail fast
4. **Cleanup Expired:** Call `tracker.cleanup_expired()` periodically to free memory from timed-out requests

## Complete Example

See [examples/request_response_pattern.rs](../../../examples/request_response_pattern.rs) for a complete, runnable example demonstrating:
- Requester and responder components
- Correlation tracking
- Timeout handling
- Error scenarios

Run the example:
```bash
cargo run --example request_response_pattern
```

## Best Practices

1. **Always Set Timeouts:** Never use infinite timeouts in production
2. **Handle All Error Cases:** Timeout, channel closed, component not found
3. **Use Unique Correlation IDs:** UUID v4 ensures global uniqueness
4. **Log Request Flows:** Include correlation ID in all log messages for traceability
5. **Monitor Latency:** Track P50, P99, and P99.9 latency in production

## Common Mistakes

1. **Forgetting to Register Pending Request:** Must call `tracker.register_pending()` before sending request
2. **Mismatched Correlation IDs:** Response must use exact correlation ID from request
3. **Blocking on Response:** Use `tokio::time::timeout` to prevent indefinite waiting
4. **Memory Leaks:** Not calling `cleanup_expired()` causes expired requests to accumulate

## Related Patterns

- **Pub-Sub Broadcasting:** For one-to-many communication (see [Pub-Sub Broadcasting](./pubsub-broadcasting.md))
- **Fire-and-Forget:** For asynchronous operations that don't need responses
- **State Management:** For sharing state across components (see [State Management Patterns](../explanation/state-management-patterns.md))

## References

- **ADR-WASM-009:** Component Communication Model (Pattern 2: Request-Response)
- **Task 6.2 Benchmarks:** `benches/messaging_benchmarks.rs`
- **API Reference:** [Message Routing](../reference/message-routing.md)
- **Implementation:** `airssys-wasm/src/actor/message/correlation_tracker.rs`
