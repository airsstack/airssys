# Inter-Component Messaging Architecture - airssys-wasm

**Document Type:** Knowledge Documentation  
**Created:** 2025-10-18  
**Status:** Complete Messaging Architecture  
**Priority:** Critical - Foundation for Component Communication  
**Related:** KNOWLEDGE-WASM-004 (WIT Interface Definitions)

## Overview

This document provides comprehensive documentation for the **inter-component messaging system** in airssys-wasm. The messaging architecture is built on **actor model principles** with deep integration into the **airssys-rt** runtime system, providing event-driven communication between WASM components.

### Design Philosophy

**Actor-Based Message Passing:**
- Components are actors with mailboxes (managed by airssys-rt)
- Messages delivered via push (no polling required)
- Host runtime acts as message router and supervisor
- Aligns with Erlang/OTP gen_server patterns

**Dual Interaction Patterns:**
- **Fire-and-Forget**: One-way asynchronous notifications (like gen_server:cast)
- **Request-Response**: Async RPC with automatic callbacks (like gen_server:call)

**Integration with airssys-rt:**
- Each component runs as an actor in the actor system
- Component mailboxes managed by airssys-rt supervisor tree
- Message routing through actor system message passing
- Fault tolerance via supervision strategies

## Message Delivery Architecture

### Push-Based Event Model

**No Polling Required:**
```
Traditional Polling (OLD - DEPRECATED):
┌─────────────┐
│ Component A │──┐
└─────────────┘  │
                 │ while true { check_messages() } ← CPU waste!
                 │
┌─────────────┐  │
│    Host     │◄─┘
└─────────────┘

Actor Model (NEW - CURRENT):
┌─────────────┐
│ Component A │◄── handle_message(sender, data) ← Push delivery!
└─────────────┘
       ▲
       │
┌─────────────┐
│    Host     │ Delivers messages when they arrive
│  (Router)   │ Zero polling overhead
└─────────────┘
```

**Key Benefits:**
- ✅ **Zero CPU waste**: No polling loops
- ✅ **Low latency**: Immediate message delivery
- ✅ **Backpressure**: Components can reject messages if overloaded
- ✅ **Actor model**: Natural alignment with airssys-rt
- ✅ **Fault isolation**: Component failures don't affect message router

### Component Exports vs Imports

**Component Exports (What Components Provide):**
```wit
interface component-lifecycle {
    execute: func(...);              // External RPC handler
    handle-message: func(...);       // Internal message handler
    handle-callback: func(...);      // Response handler (optional)
}
```

**Component Imports (What Components Use):**
```wit
interface host-services {
    send-message: func(...);         // Fire-and-forget
    send-request: func(...);         // Request-response
    cancel-request: func(...);       // Cancel pending request
}
```

## Message Flow Sequence Diagrams

### Fire-and-Forget Message Flow

```
┌──────────────┐         ┌──────────────┐         ┌──────────────┐
│ Component A  │         │ Host Runtime │         │ Component B  │
│  (Sender)    │         │   (Router)   │         │  (Receiver)  │
└──────┬───────┘         └──────┬───────┘         └──────┬───────┘
       │                        │                        │
       │  send_message(B, msg)  │                        │
       │───────────────────────>│                        │
       │                        │                        │
       │  Ok(())                │                        │
       │<───────────────────────│                        │
       │                        │                        │
       │                        │  handle_message(A, msg)│
       │                        │───────────────────────>│
       │                        │                        │
       │                        │  Process message       │
       │                        │                        ├─┐
       │                        │                        │ │
       │                        │                        │<┘
       │                        │                        │
       │                        │  Ok(())                │
       │                        │<───────────────────────│
       │                        │                        │
```

**Key Points:**
- Sender calls `send_message` and immediately returns
- Host routes message asynchronously
- Receiver gets `handle_message` invocation (push delivery)
- No response correlation needed

### Request-Response with Callback Flow

```
┌──────────────┐         ┌──────────────┐         ┌──────────────┐
│ Component A  │         │ Host Runtime │         │ Component B  │
│ (Requester)  │         │   (Router)   │         │ (Responder)  │
└──────┬───────┘         └──────┬───────┘         └──────┬───────┘
       │                        │                        │
       │ send_request(B,req,5s) │                        │
       │───────────────────────>│                        │
       │                        │                        │
       │                        │ Register callback      │
       │                        ├─┐ request_id="abc-123" │
       │                        │ │ timeout=5s           │
       │                        │<┘                      │
       │                        │                        │
       │  Ok("abc-123")         │                        │
       │<───────────────────────│                        │
       │                        │                        │
       │                        │  handle_message(A,req) │
       │                        │───────────────────────>│
       │                        │                        │
       │                        │                        │ Process
       │                        │                        ├─┐ request
       │                        │                        │ │
       │                        │                        │<┘
       │                        │                        │
       │                        │  Ok(response_data)     │
       │                        │<───────────────────────│
       │                        │                        │
       │                        │ Route to callback      │
       │                        ├─┐                      │
       │                        │ │                      │
       │                        │<┘                      │
       │                        │                        │
       │ handle_callback(       │                        │
       │   "abc-123",           │                        │
       │   Ok(response_data)    │                        │
       │ )                      │                        │
       │<───────────────────────│                        │
       │                        │                        │
       │ Process response       │                        │
       ├─┐                      │                        │
       │ │                      │                        │
       │<┘                      │                        │
       │                        │                        │
```

**Key Points:**
- Requester gets `request_id` immediately
- Host manages correlation automatically
- Responder returns data via function return
- Host routes response to requester's `handle_callback`
- Timeout enforced by host runtime

### Request Timeout Flow

```
┌──────────────┐         ┌──────────────┐         ┌──────────────┐
│ Component A  │         │ Host Runtime │         │ Component B  │
│ (Requester)  │         │   (Router)   │         │ (Responder)  │
└──────┬───────┘         └──────┬───────┘         └──────┬───────┘
       │                        │                        │
       │ send_request(B,req,2s) │                        │
       │───────────────────────>│                        │
       │                        │                        │
       │  Ok("xyz-789")         │                        │
       │<───────────────────────│                        │
       │                        │                        │
       │                        │  handle_message(A,req) │
       │                        │───────────────────────>│
       │                        │                        │
       │                        │                        │ Processing
       │                        │                        ├─┐ takes too
       │                        │                        │ │ long...
       │                        │                        │ │
       │                        │ ⏰ Timeout (2s)        │ │
       │                        ├─┐                      │ │
       │                        │ │ Remove callback      │ │
       │                        │<┘                      │ │
       │                        │                        │ │
       │ handle_callback(       │                        │ │
       │   "xyz-789",           │                        │ │
       │   Err(Timeout)         │                        │ │
       │ )                      │                        │ │
       │<───────────────────────│                        │ │
       │                        │                        │ │
       │ Handle timeout         │                        │<┘
       ├─┐                      │                        │
       │ │ Retry / Log / Fail   │                        │
       │<┘                      │                        │
       │                        │                        │
       │                        │  Ok(response) [IGNORED]│
       │                        │<───────────────────────│
       │                        │                        │
       │                        │ Callback already       │
       │                        │ removed - discard      │
       │                        ├─┐                      │
       │                        │ │                      │
       │                        │<┘                      │
       │                        │                        │
```

**Key Points:**
- Host enforces timeout automatically
- Timeout error delivered to callback
- Late responses from responder are discarded
- Requester can implement retry logic

### Manual Request-Response Flow (Advanced)

```
┌──────────────┐         ┌──────────────┐         ┌──────────────┐
│ Component A  │         │ Host Runtime │         │ Component B  │
│ (Requester)  │         │   (Router)   │         │ (Responder)  │
└──────┬───────┘         └──────┬───────┘         └──────┬───────┘
       │                        │                        │
       │ Generate correlation   │                        │
       ├─┐ id="manual-456"      │                        │
       │ │                      │                        │
       │<┘                      │                        │
       │                        │                        │
       │ send_message(B, {      │                        │
       │   corr_id:"manual-456",│                        │
       │   request_data         │                        │
       │ })                     │                        │
       │───────────────────────>│                        │
       │                        │                        │
       │ Store pending request  │                        │
       ├─┐ manually              │                        │
       │ │                      │                        │
       │<┘                      │                        │
       │                        │                        │
       │                        │  handle_message(A,msg) │
       │                        │───────────────────────>│
       │                        │                        │
       │                        │                        │ Extract
       │                        │                        ├─┐ corr_id
       │                        │                        │ │ Process
       │                        │                        │<┘
       │                        │                        │
       │                        │                        │ send_message(A, {
       │                        │                        │   corr_id,
       │                        │                        │   response
       │                        │                        │ })
       │                        │<───────────────────────│
       │                        │                        │
       │  handle_message(B,msg) │                        │
       │<───────────────────────│                        │
       │                        │                        │
       │ Extract corr_id        │                        │
       ├─┐ Match with pending   │                        │
       │ │ Process response     │                        │
       │<┘                      │                        │
       │                        │                        │
```

**Key Points:**
- Component manages correlation IDs manually
- Both request and response use `send_message`
- Both sides implement `handle_message` for bidirectional flow
- Full control over correlation logic and timeout strategies

### Multicodec Message Encoding Flow

```
┌──────────────┐         ┌──────────────┐         ┌──────────────┐
│ Component A  │         │ Host Runtime │         │ Component B  │
│   (Rust)     │         │              │         │ (JavaScript) │
└──────┬───────┘         └──────┬───────┘         └──────┬───────┘
       │                        │                        │
       │ Rust struct UserData   │                        │
       ├─┐                      │                        │
       │ │ { id: "123",         │                        │
       │ │   name: "Alice" }    │                        │
       │<┘                      │                        │
       │                        │                        │
       │ Encode with Borsh      │                        │
       ├─┐ + multicodec prefix  │                        │
       │ │ [0x701][binary_data] │                        │
       │<┘                      │                        │
       │                        │                        │
       │ send_message(B, data)  │                        │
       │───────────────────────>│                        │
       │                        │                        │
       │                        │  handle_message(A,data)│
       │                        │───────────────────────>│
       │                        │                        │
       │                        │                        │ Read prefix
       │                        │                        ├─┐ 0x701 = borsh
       │                        │                        │ │
       │                        │                        │<┘
       │                        │                        │
       │                        │                        │ Decode with
       │                        │                        ├─┐ borsh library
       │                        │                        │ │ Get JS object:
       │                        │                        │ │ { id:"123",
       │                        │                        │ │   name:"Alice"}
       │                        │                        │<┘
       │                        │                        │
       │                        │                        │ Process data
       │                        │                        ├─┐
       │                        │                        │ │
       │                        │                        │<┘
       │                        │                        │
```

**Key Points:**
- Multicodec prefix enables language-agnostic serialization
- Receiver identifies format from prefix (varint encoded)
- No out-of-band format negotiation needed
- Supports mixed-language component ecosystems

## Messaging Patterns

### Pattern 1: Fire-and-Forget (One-Way Messaging)

**Use Cases:**
- Event notifications (user logged in, file uploaded, etc.)
- Status updates (progress reports, health checks)
- Pub-sub style broadcasting
- Logging and monitoring events
- Non-critical notifications

**Characteristics:**
- No response expected
- No correlation tracking needed
- Minimal overhead
- Best for high-throughput scenarios
- Fire-and-forget semantics

**WIT Interface:**
```wit
// Sender imports
interface host-services {
    send-message: func(
        target: component-id,
        message: list<u8>    // Multicodec-encoded
    ) -> result<_, messaging-error>;
}

// Receiver exports
interface component-lifecycle {
    handle-message: func(
        sender: component-id,
        message: list<u8>
    ) -> result<_, messaging-error>;
}
```

**Rust Implementation Example:**

```rust
// ========================================
// SENDER Component (Rust)
// ========================================
use airssys_wasm_bindings::host::services::*;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct UserLoggedInEvent {
    user_id: String,
    timestamp: u64,
    ip_address: String,
}

impl Component {
    fn notify_user_login(&self, user_id: &str) -> Result<()> {
        let event = UserLoggedInEvent {
            user_id: user_id.to_string(),
            timestamp: current_time_millis(),
            ip_address: "192.168.1.100".to_string(),
        };
        
        // Encode with multicodec (borsh format)
        let encoded = self.encode_borsh(&event)?;
        
        // Send fire-and-forget message
        send_message(
            &ComponentId::from("analytics-service"),
            &encoded
        ).map_err(|e| anyhow!("Failed to send message: {:?}", e))?;
        
        Ok(())
    }
}

// ========================================
// RECEIVER Component (Rust)
// ========================================
#[export_name = "handle-message"]
pub extern "C" fn handle_message(sender_ptr: *const u8, sender_len: usize,
                                  message_ptr: *const u8, message_len: usize) -> i32 {
    let sender = unsafe { ComponentId::from_ptr(sender_ptr, sender_len) };
    let message_data = unsafe { std::slice::from_raw_parts(message_ptr, message_len) };
    
    match handle_message_impl(sender, message_data) {
        Ok(()) => 0,  // Success
        Err(e) => {
            log_error(&format!("handle-message error: {}", e));
            1  // Error
        }
    }
}

fn handle_message_impl(sender: ComponentId, message: &[u8]) -> Result<()> {
    // Decode multicodec message
    let event: UserLoggedInEvent = decode_borsh(message)?;
    
    // Process event
    log_info(&format!("User {} logged in from {}", event.user_id, event.ip_address));
    
    // Update analytics database
    update_login_stats(&event)?;
    
    Ok(())
}
```

**JavaScript Implementation Example:**

```javascript
// ========================================
// SENDER Component (JavaScript)
// ========================================
import { sendMessage } from 'airssys:host-core/services';
import { encode } from '@msgpack/msgpack';

async function notifyUserLogin(userId) {
    const event = {
        userId: userId,
        timestamp: Date.now(),
        ipAddress: '192.168.1.100'
    };
    
    // Encode with msgpack
    const encoded = encode(event);
    
    // Send fire-and-forget message
    await sendMessage('analytics-service', encoded);
}

// ========================================
// RECEIVER Component (JavaScript)
// ========================================
import { decode } from '@msgpack/msgpack';

export function handleMessage(sender, messageData) {
    try {
        // Decode message
        const event = decode(messageData);
        
        // Process event
        console.log(`User ${event.userId} logged in from ${event.ipAddress}`);
        
        // Update analytics
        updateLoginStats(event);
        
    } catch (error) {
        console.error('handle-message error:', error);
        throw error;
    }
}
```

**Go Implementation Example:**

```go
// ========================================
// SENDER Component (Go)
// ========================================
package main

import (
    "github.com/vmihailenco/msgpack/v5"
    "airssys.io/wasm/bindings/host"
)

type UserLoggedInEvent struct {
    UserID    string `msgpack:"user_id"`
    Timestamp int64  `msgpack:"timestamp"`
    IPAddress string `msgpack:"ip_address"`
}

func (c *Component) NotifyUserLogin(userID string) error {
    event := UserLoggedInEvent{
        UserID:    userID,
        Timestamp: time.Now().UnixMilli(),
        IPAddress: "192.168.1.100",
    }
    
    // Encode with msgpack
    encoded, err := msgpack.Marshal(event)
    if err != nil {
        return fmt.Errorf("encode error: %w", err)
    }
    
    // Send fire-and-forget message
    return host.SendMessage("analytics-service", encoded)
}

// ========================================
// RECEIVER Component (Go)
// ========================================
//export handle_message
func handleMessage(senderPtr, senderLen uint32, msgPtr, msgLen uint32) int32 {
    sender := ptrToComponentID(senderPtr, senderLen)
    message := ptrToBytes(msgPtr, msgLen)
    
    if err := handleMessageImpl(sender, message); err != nil {
        logError("handle-message error: %v", err)
        return 1
    }
    return 0
}

func handleMessageImpl(sender string, message []byte) error {
    var event UserLoggedInEvent
    if err := msgpack.Unmarshal(message, &event); err != nil {
        return fmt.Errorf("decode error: %w", err)
    }
    
    // Process event
    log.Printf("User %s logged in from %s", event.UserID, event.IPAddress)
    
    // Update analytics
    return updateLoginStats(&event)
}
```

**Error Handling:**

```rust
// Component can reject messages
fn handle_message_impl(sender: ComponentId, message: &[u8]) -> Result<()> {
    // Validate message size
    if message.len() > MAX_MESSAGE_SIZE {
        return Err(MessagingError::MessageTooLarge(message.len()));
    }
    
    // Check rate limiting
    if !self.rate_limiter.check(&sender) {
        return Err(MessagingError::RateLimitExceeded);
    }
    
    // Decode and process
    let event: Event = decode_borsh(message)?;
    process_event(event)?;
    
    Ok(())
}
```

### Pattern 2: Request-Response with Callbacks

**Use Cases:**
- Query data from another component
- Request computation results
- Fetch configuration or metadata
- Any operation requiring a response
- RPC-style interactions

**Characteristics:**
- Expects response from target component
- Automatic correlation by host (no manual tracking)
- Timeout enforcement by host
- Callback-based response delivery (NEAR Protocol style)
- Type-safe response handling

**WIT Interface:**
```wit
// Sender imports
interface host-services {
    send-request: func(
        target: component-id,
        request: list<u8>,
        timeout-ms: u64
    ) -> result<request-id, messaging-error>;
    
    cancel-request: func(
        request-id: string
    ) -> result<_, messaging-error>;
}

// Sender exports (callback handler)
interface component-lifecycle {
    handle-callback: func(
        request-id: string,
        result: result<list<u8>, callback-error>
    ) -> result<_, messaging-error>;
}

// Receiver exports (request handler)
interface component-lifecycle {
    handle-message: func(
        sender: component-id,
        message: list<u8>
    ) -> result<_, messaging-error>;
}
```

**Message Flow:**
```
1. Component A calls send-request(B, request, timeout)
   ↓
2. Host registers callback and delivers to Component B
   ↓
3. Component B's handle-message(A, request) is called
   ↓
4. Component B processes and returns result
   ↓
5. Host routes result back to Component A
   ↓
6. Component A's handle-callback(request-id, result) is called
```

**Rust Implementation Example:**

```rust
// ========================================
// REQUESTER Component (Rust)
// ========================================
use airssys_wasm_bindings::host::services::*;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct UserDataRequest {
    user_id: String,
}

#[derive(Serialize, Deserialize)]
struct UserDataResponse {
    user_id: String,
    name: String,
    email: String,
}

pub struct Component {
    pending_requests: HashMap<String, RequestContext>,
}

struct RequestContext {
    operation: String,
    started_at: u64,
}

impl Component {
    pub fn fetch_user_data(&mut self, user_id: &str) -> Result<()> {
        let request = UserDataRequest {
            user_id: user_id.to_string(),
        };
        
        // Encode request
        let encoded = self.encode_borsh(&request)?;
        
        // Send request with 5 second timeout
        let request_id = send_request(
            &ComponentId::from("user-service"),
            &encoded,
            5000  // 5 seconds
        ).map_err(|e| anyhow!("send-request failed: {:?}", e))?;
        
        // Track pending request
        self.pending_requests.insert(request_id.to_string(), RequestContext {
            operation: "fetch-user-data".to_string(),
            started_at: current_time_millis(),
        });
        
        log_info(&format!("Sent user data request: {}", request_id));
        Ok(())
    }
}

// Callback handler export
#[export_name = "handle-callback"]
pub extern "C" fn handle_callback(
    request_id_ptr: *const u8, request_id_len: usize,
    result_ptr: *const u8, result_len: usize,
    is_error: u32
) -> i32 {
    let request_id = unsafe { 
        std::str::from_utf8(std::slice::from_raw_parts(request_id_ptr, request_id_len))
            .unwrap()
    };
    
    let result_data = unsafe { std::slice::from_raw_parts(result_ptr, result_len) };
    
    match handle_callback_impl(request_id, result_data, is_error != 0) {
        Ok(()) => 0,
        Err(e) => {
            log_error(&format!("handle-callback error: {}", e));
            1
        }
    }
}

fn handle_callback_impl(request_id: &str, data: &[u8], is_error: bool) -> Result<()> {
    // Retrieve pending request context
    let context = COMPONENT.pending_requests.remove(request_id)
        .ok_or_else(|| anyhow!("Unknown request ID: {}", request_id))?;
    
    let duration = current_time_millis() - context.started_at;
    
    if is_error {
        // Handle error response
        let error: CallbackError = decode_borsh(data)?;
        log_error(&format!("Request {} failed: {:?}", request_id, error));
        return Err(anyhow!("Request failed: {:?}", error));
    }
    
    // Decode success response
    let response: UserDataResponse = decode_borsh(data)?;
    
    log_info(&format!(
        "Received user data for {} in {}ms: {} <{}>",
        response.user_id, duration, response.name, response.email
    ));
    
    // Process response
    process_user_data(response)?;
    
    Ok(())
}

// ========================================
// RESPONDER Component (Rust)
// ========================================
#[export_name = "handle-message"]
pub extern "C" fn handle_message(sender_ptr: *const u8, sender_len: usize,
                                  message_ptr: *const u8, message_len: usize) -> i32 {
    let sender = unsafe { ComponentId::from_ptr(sender_ptr, sender_len) };
    let message_data = unsafe { std::slice::from_raw_parts(message_ptr, message_len) };
    
    match handle_message_impl(sender, message_data) {
        Ok(response_data) => {
            // Return response data to host for callback routing
            set_response_data(response_data);
            0
        }
        Err(e) => {
            log_error(&format!("handle-message error: {}", e));
            set_error_response(&format!("{:?}", e));
            1
        }
    }
}

fn handle_message_impl(sender: ComponentId, message: &[u8]) -> Result<Vec<u8>> {
    // Decode request
    let request: UserDataRequest = decode_borsh(message)?;
    
    log_info(&format!("Received user data request from {}: {}", sender, request.user_id));
    
    // Fetch user data from database
    let user = fetch_user_from_db(&request.user_id)?;
    
    // Build response
    let response = UserDataResponse {
        user_id: user.id,
        name: user.name,
        email: user.email,
    };
    
    // Encode response
    encode_borsh(&response)
}
```

**JavaScript Implementation Example:**

```javascript
// ========================================
// REQUESTER Component (JavaScript)
// ========================================
import { sendRequest, cancelRequest } from 'airssys:host-core/services';
import { encode, decode } from '@msgpack/msgpack';

class Component {
    constructor() {
        this.pendingRequests = new Map();
    }
    
    async fetchUserData(userId) {
        const request = {
            userId: userId
        };
        
        // Encode and send request with 5 second timeout
        const encoded = encode(request);
        const requestId = await sendRequest('user-service', encoded, 5000);
        
        // Track pending request
        this.pendingRequests.set(requestId, {
            operation: 'fetch-user-data',
            startedAt: Date.now()
        });
        
        console.log(`Sent user data request: ${requestId}`);
    }
}

// Callback handler export
export function handleCallback(requestId, resultData, isError) {
    const context = component.pendingRequests.get(requestId);
    if (!context) {
        throw new Error(`Unknown request ID: ${requestId}`);
    }
    
    component.pendingRequests.delete(requestId);
    const duration = Date.now() - context.startedAt;
    
    if (isError) {
        const error = decode(resultData);
        console.error(`Request ${requestId} failed:`, error);
        throw new Error(`Request failed: ${error.message}`);
    }
    
    // Decode success response
    const response = decode(resultData);
    console.log(`Received user data for ${response.userId} in ${duration}ms: ${response.name} <${response.email}>`);
    
    // Process response
    processUserData(response);
}

// ========================================
// RESPONDER Component (JavaScript)
// ========================================
import { decode, encode } from '@msgpack/msgpack';

export function handleMessage(sender, messageData) {
    // Decode request
    const request = decode(messageData);
    console.log(`Received user data request from ${sender}: ${request.userId}`);
    
    // Fetch user data
    const user = fetchUserFromDB(request.userId);
    
    // Build and encode response
    const response = {
        userId: user.id,
        name: user.name,
        email: user.email
    };
    
    // Return response (host will route to callback)
    return encode(response);
}
```

**Timeout Handling:**

```rust
// Host enforces timeouts automatically
impl HostRuntime {
    async fn enforce_request_timeout(&self, request_id: String, timeout_ms: u64) {
        sleep(Duration::from_millis(timeout_ms)).await;
        
        if let Some(callback) = self.pending_callbacks.remove(&request_id) {
            // Timeout exceeded - deliver error to callback
            let error = CallbackError::Timeout {
                request_id: request_id.clone(),
                timeout_ms,
            };
            
            self.invoke_callback(callback.component_id, &request_id, Err(error)).await;
            
            log_warning(&format!("Request {} timed out after {}ms", request_id, timeout_ms));
        }
    }
}

// Component handles timeout in callback
fn handle_callback_impl(request_id: &str, data: &[u8], is_error: bool) -> Result<()> {
    if is_error {
        let error: CallbackError = decode_borsh(data)?;
        
        match error {
            CallbackError::Timeout { timeout_ms, .. } => {
                log_warning(&format!("Request timed out after {}ms", timeout_ms));
                // Retry logic, fallback, or error propagation
                return retry_with_longer_timeout(request_id);
            }
            CallbackError::TargetNotFound { .. } => {
                log_error("Target component not found");
                return Err(anyhow!("Component unavailable"));
            }
            _ => return Err(anyhow!("Request failed: {:?}", error)),
        }
    }
    
    // ... success handling
}
```

**Request Cancellation:**

```rust
impl Component {
    pub fn cancel_pending_request(&mut self, request_id: &str) -> Result<()> {
        // Cancel at host level
        cancel_request(request_id)
            .map_err(|e| anyhow!("cancel-request failed: {:?}", e))?;
        
        // Remove from local tracking
        self.pending_requests.remove(request_id);
        
        log_info(&format!("Cancelled request: {}", request_id));
        Ok(())
    }
    
    pub fn cleanup_stale_requests(&mut self) {
        let now = current_time_millis();
        let stale_threshold = 30000; // 30 seconds
        
        let stale_ids: Vec<String> = self.pending_requests.iter()
            .filter(|(_, ctx)| now - ctx.started_at > stale_threshold)
            .map(|(id, _)| id.clone())
            .collect();
        
        for request_id in stale_ids {
            let _ = self.cancel_pending_request(&request_id);
        }
    }
}
```

### Pattern 3: Manual Request-Response (Advanced)

**Use Cases:**
- Complex multi-step workflows
- Correlation with external IDs
- Custom timeout strategies
- Advanced error recovery
- When callback pattern is too restrictive

**Characteristics:**
- Full control over correlation
- Manual request/response matching
- Component manages correlation IDs
- Flexible but more boilerplate
- Use when callbacks don't fit

**Implementation Example:**

```rust
// ========================================
// Manual Correlation Pattern
// ========================================
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct ManualRequest {
    correlation_id: String,
    operation: String,
    data: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
struct ManualResponse {
    correlation_id: String,
    result: Vec<u8>,
}

pub struct Component {
    pending_manual_requests: HashMap<String, ManualRequestContext>,
}

struct ManualRequestContext {
    started_at: u64,
    timeout_ms: u64,
    retry_count: u32,
}

impl Component {
    pub fn send_manual_request(&mut self, target: &str, data: Vec<u8>) -> Result<String> {
        // Generate correlation ID
        let correlation_id = Uuid::new_v4().to_string();
        
        let request = ManualRequest {
            correlation_id: correlation_id.clone(),
            operation: "custom-operation".to_string(),
            data,
        };
        
        // Encode and send via send-message (not send-request!)
        let encoded = encode_borsh(&request)?;
        send_message(&ComponentId::from(target), &encoded)?;
        
        // Track manually
        self.pending_manual_requests.insert(correlation_id.clone(), ManualRequestContext {
            started_at: current_time_millis(),
            timeout_ms: 10000,
            retry_count: 0,
        });
        
        Ok(correlation_id)
    }
}

// Handle response via handle-message (not handle-callback!)
fn handle_message_impl(sender: ComponentId, message: &[u8]) -> Result<()> {
    let response: ManualResponse = decode_borsh(message)?;
    
    // Match with pending request
    if let Some(context) = COMPONENT.pending_manual_requests.remove(&response.correlation_id) {
        let duration = current_time_millis() - context.started_at;
        log_info(&format!("Received response for {} in {}ms", response.correlation_id, duration));
        
        // Process response
        process_response(&response.result)?;
    } else {
        log_warning(&format!("Received response for unknown correlation ID: {}", response.correlation_id));
    }
    
    Ok(())
}

// Manual timeout checking
impl Component {
    pub fn check_manual_timeouts(&mut self) {
        let now = current_time_millis();
        let timed_out: Vec<String> = self.pending_manual_requests.iter()
            .filter(|(_, ctx)| now - ctx.started_at > ctx.timeout_ms)
            .map(|(id, _)| id.clone())
            .collect();
        
        for correlation_id in timed_out {
            let context = self.pending_manual_requests.remove(&correlation_id).unwrap();
            log_warning(&format!("Manual request {} timed out after {}ms", 
                correlation_id, context.timeout_ms));
            
            // Custom retry logic
            if context.retry_count < 3 {
                self.retry_manual_request(&correlation_id, context.retry_count + 1);
            } else {
                self.handle_manual_request_failure(&correlation_id);
            }
        }
    }
}
```

**When to Use Manual vs Callback:**

| Scenario | Use Callback | Use Manual |
|----------|-------------|-----------|
| Simple request-response | ✅ | ❌ |
| Need automatic timeout | ✅ | ❌ |
| Standard RPC pattern | ✅ | ❌ |
| Complex multi-step workflow | ❌ | ✅ |
| Custom correlation logic | ❌ | ✅ |
| Retry with exponential backoff | ❌ | ✅ |
| Correlation with external IDs | ❌ | ✅ |

## Host Runtime Implementation

### Message Routing Engine

**airssys-rt Integration:**

```rust
use airssys_rt::{Actor, ActorSystem, Mailbox, Message};
use airssys_wasm::ComponentInstance;

pub struct ComponentActor {
    instance: ComponentInstance,
    mailbox: Mailbox<ComponentMessage>,
}

pub enum ComponentMessage {
    External(ExternalRpc),
    Internal(InternalMessage),
    Callback(CallbackResponse),
}

impl Actor for ComponentActor {
    type Message = ComponentMessage;
    
    async fn handle(&mut self, msg: Self::Message) -> Result<()> {
        match msg {
            ComponentMessage::External(rpc) => {
                // Call execute() export
                let result = self.instance.execute(&rpc.operation, &rpc.context).await?;
                rpc.reply_channel.send(result)?;
            }
            
            ComponentMessage::Internal(internal) => {
                // Call handle-message() export
                self.instance.handle_message(&internal.sender, &internal.data).await?;
            }
            
            ComponentMessage::Callback(callback) => {
                // Call handle-callback() export
                self.instance.handle_callback(&callback.request_id, callback.result).await?;
            }
        }
        
        Ok(())
    }
}

pub struct MessageRouter {
    actor_system: ActorSystem,
    component_actors: HashMap<ComponentId, ActorRef<ComponentMessage>>,
    pending_callbacks: HashMap<String, PendingCallback>,
}

impl MessageRouter {
    pub async fn route_message(&self, from: ComponentId, to: ComponentId, data: Vec<u8>) -> Result<()> {
        let target_actor = self.component_actors.get(&to)
            .ok_or_else(|| Error::ComponentNotFound(to.clone()))?;
        
        // Send internal message to target actor
        target_actor.send(ComponentMessage::Internal(InternalMessage {
            sender: from,
            data,
        })).await?;
        
        Ok(())
    }
    
    pub async fn route_request(&mut self, from: ComponentId, to: ComponentId, 
                                 request: Vec<u8>, timeout_ms: u64) -> Result<String> {
        let request_id = Uuid::new_v4().to_string();
        
        // Register callback
        self.pending_callbacks.insert(request_id.clone(), PendingCallback {
            requester: from.clone(),
            started_at: Utc::now(),
            timeout_ms,
        });
        
        // Route request message
        self.route_message(from, to, request).await?;
        
        // Spawn timeout enforcement
        let router_clone = self.clone();
        let request_id_clone = request_id.clone();
        tokio::spawn(async move {
            sleep(Duration::from_millis(timeout_ms)).await;
            router_clone.enforce_timeout(request_id_clone).await;
        });
        
        Ok(request_id)
    }
    
    pub async fn route_callback(&mut self, request_id: String, result: Result<Vec<u8>, CallbackError>) {
        if let Some(callback) = self.pending_callbacks.remove(&request_id) {
            let requester_actor = self.component_actors.get(&callback.requester).unwrap();
            
            requester_actor.send(ComponentMessage::Callback(CallbackResponse {
                request_id,
                result,
            })).await.ok();
        }
    }
}
```

### Callback Management

**Lifecycle:**

```rust
pub struct CallbackManager {
    pending: HashMap<String, PendingCallback>,
    metrics: CallbackMetrics,
}

struct PendingCallback {
    requester: ComponentId,
    started_at: DateTime<Utc>,
    timeout_ms: u64,
}

impl CallbackManager {
    pub fn register(&mut self, request_id: String, requester: ComponentId, timeout_ms: u64) {
        self.pending.insert(request_id, PendingCallback {
            requester,
            started_at: Utc::now(),
            timeout_ms,
        });
        
        self.metrics.total_registered += 1;
    }
    
    pub fn complete(&mut self, request_id: &str) -> Option<PendingCallback> {
        if let Some(callback) = self.pending.remove(request_id) {
            let duration = (Utc::now() - callback.started_at).num_milliseconds();
            self.metrics.total_completed += 1;
            self.metrics.record_duration(duration as u64);
            Some(callback)
        } else {
            None
        }
    }
    
    pub fn timeout(&mut self, request_id: &str) -> Option<PendingCallback> {
        if let Some(callback) = self.pending.remove(request_id) {
            self.metrics.total_timeouts += 1;
            Some(callback)
        } else {
            None
        }
    }
    
    pub fn cleanup_on_shutdown(&mut self, component_id: &ComponentId) {
        // Cancel all pending callbacks for shutting down component
        let to_cancel: Vec<String> = self.pending.iter()
            .filter(|(_, cb)| &cb.requester == component_id)
            .map(|(id, _)| id.clone())
            .collect();
        
        for request_id in to_cancel {
            self.timeout(&request_id);
        }
    }
    
    pub fn get_metrics(&self) -> &CallbackMetrics {
        &self.metrics
    }
}
```

### Security Enforcement

**Permission Checks:**

```rust
impl MessageRouter {
    pub async fn route_message_with_security(
        &self,
        from: ComponentId,
        to: ComponentId,
        data: Vec<u8>
    ) -> Result<()> {
        // Check if sender has permission to message target
        if !self.security_policy.can_message(&from, &to) {
            self.audit_log.log_violation(SecurityViolation {
                component: from.clone(),
                operation: "send-message",
                target: to.clone(),
                reason: "No messaging permission",
            });
            
            return Err(Error::PermissionDenied(format!(
                "Component '{}' cannot send messages to '{}'",
                from, to
            )));
        }
        
        // Check message size limits
        if data.len() > self.config.max_message_size {
            return Err(Error::MessageTooLarge {
                size: data.len(),
                max: self.config.max_message_size,
            });
        }
        
        // Check rate limiting
        if !self.rate_limiter.check(&from) {
            self.audit_log.log_violation(SecurityViolation {
                component: from.clone(),
                operation: "send-message",
                target: to.clone(),
                reason: "Rate limit exceeded",
            });
            
            return Err(Error::RateLimitExceeded);
        }
        
        // Route message
        self.route_message(from, to, data).await
    }
}
```

**Rate Limiting:**

```rust
pub struct RateLimiter {
    buckets: HashMap<ComponentId, TokenBucket>,
    config: RateLimitConfig,
}

struct TokenBucket {
    tokens: f64,
    last_update: Instant,
}

impl RateLimiter {
    pub fn check(&mut self, component: &ComponentId) -> bool {
        let bucket = self.buckets.entry(component.clone())
            .or_insert_with(|| TokenBucket {
                tokens: self.config.burst_size as f64,
                last_update: Instant::now(),
            });
        
        // Refill tokens
        let now = Instant::now();
        let elapsed = (now - bucket.last_update).as_secs_f64();
        bucket.tokens = (bucket.tokens + elapsed * self.config.rate_per_second)
            .min(self.config.burst_size as f64);
        bucket.last_update = now;
        
        // Check and consume token
        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}
```

## Performance Considerations

### Message Queue Sizing

**Configuration:**

```rust
pub struct MessageQueueConfig {
    /// Maximum messages in component mailbox
    pub max_queue_size: usize,
    
    /// Backpressure strategy when queue full
    pub backpressure_strategy: BackpressureStrategy,
    
    /// Queue monitoring thresholds
    pub warning_threshold: f64,  // 0.0-1.0
    pub critical_threshold: f64,
}

pub enum BackpressureStrategy {
    /// Reject new messages when queue full
    Reject,
    
    /// Block sender until queue has space
    Block,
    
    /// Drop oldest messages to make room
    DropOldest,
    
    /// Drop lowest priority messages first
    DropByPriority,
}
```

**Monitoring:**

```rust
impl ComponentActor {
    fn check_queue_health(&self) -> QueueHealth {
        let usage = self.mailbox.len() as f64 / self.config.max_queue_size as f64;
        
        if usage >= self.config.critical_threshold {
            QueueHealth::Critical { usage, queue_size: self.mailbox.len() }
        } else if usage >= self.config.warning_threshold {
            QueueHealth::Warning { usage, queue_size: self.mailbox.len() }
        } else {
            QueueHealth::Healthy { usage, queue_size: self.mailbox.len() }
        }
    }
}
```

### Latency Targets

**Performance Goals:**

| Operation | Target Latency | Notes |
|-----------|---------------|-------|
| send-message (local) | < 100μs | Host-local routing |
| handle-message invocation | < 500μs | WASM function call overhead |
| send-request registration | < 200μs | Callback tracking |
| handle-callback invocation | < 500μs | WASM function call overhead |
| Message serialization (borsh) | < 50μs | Per 1KB message |
| Message serialization (json) | < 200μs | Per 1KB message |
| Total round-trip (request-response) | < 2ms | Including processing time |

**Measurement:**

```rust
pub struct MessagingMetrics {
    pub message_send_histogram: Histogram,
    pub message_receive_histogram: Histogram,
    pub callback_latency_histogram: Histogram,
    pub queue_depth_gauge: Gauge,
}

impl MessageRouter {
    async fn route_message_with_metrics(&self, from: ComponentId, to: ComponentId, data: Vec<u8>) -> Result<()> {
        let start = Instant::now();
        
        let result = self.route_message(from, to, data).await;
        
        let duration = start.elapsed();
        self.metrics.message_send_histogram.record(duration.as_micros() as f64);
        
        result
    }
}
```

### Bulk Message Optimization

**Batching:**

```rust
// Send multiple messages in single call
pub fn send_message_batch(targets: Vec<(ComponentId, Vec<u8>)>) -> Result<()> {
    for (target, message) in targets {
        send_message(&target, &message)?;
    }
    Ok(())
}

// Component can process batch
#[derive(Serialize, Deserialize)]
struct BatchMessage {
    messages: Vec<IndividualMessage>,
}

fn handle_message_impl(sender: ComponentId, message: &[u8]) -> Result<()> {
    let batch: BatchMessage = decode_borsh(message)?;
    
    for msg in batch.messages {
        process_individual_message(msg)?;
    }
    
    Ok(())
}
```

## Error Handling and Recovery

### Timeout Strategies

**Configurable Timeouts:**

```rust
pub struct TimeoutStrategy {
    /// Default timeout for requests
    pub default_timeout_ms: u64,
    
    /// Timeout per operation type
    pub operation_timeouts: HashMap<String, u64>,
    
    /// Maximum timeout allowed
    pub max_timeout_ms: u64,
}

impl Component {
    fn get_timeout_for_operation(&self, operation: &str) -> u64 {
        self.timeout_strategy.operation_timeouts
            .get(operation)
            .copied()
            .unwrap_or(self.timeout_strategy.default_timeout_ms)
            .min(self.timeout_strategy.max_timeout_ms)
    }
    
    pub fn send_request_with_adaptive_timeout(&mut self, target: &str, request: Vec<u8>) -> Result<String> {
        let operation = identify_operation(&request)?;
        let timeout = self.get_timeout_for_operation(&operation);
        
        let request_id = send_request(&ComponentId::from(target), &request, timeout)?;
        Ok(request_id)
    }
}
```

### Retry Policies

**Exponential Backoff:**

```rust
pub struct RetryPolicy {
    pub max_retries: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub multiplier: f64,
}

impl Component {
    async fn send_with_retry(&mut self, target: &str, request: Vec<u8>) -> Result<Vec<u8>> {
        let mut retry_count = 0;
        let mut delay = self.retry_policy.initial_delay_ms;
        
        loop {
            match self.send_request_sync(target, request.clone()).await {
                Ok(response) => return Ok(response),
                Err(e) if retry_count < self.retry_policy.max_retries => {
                    log_warning(&format!("Request failed (attempt {}): {:?}", retry_count + 1, e));
                    
                    // Exponential backoff
                    sleep_millis(delay);
                    delay = (delay as f64 * self.retry_policy.multiplier) as u64;
                    delay = delay.min(self.retry_policy.max_delay_ms);
                    
                    retry_count += 1;
                }
                Err(e) => {
                    log_error(&format!("Request failed after {} retries: {:?}", retry_count, e));
                    return Err(e);
                }
            }
        }
    }
}
```

### Circuit Breaker

**Fault Isolation:**

```rust
pub struct CircuitBreaker {
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    last_failure_time: Option<Instant>,
    config: CircuitBreakerConfig,
}

pub enum CircuitState {
    Closed,      // Normal operation
    Open,        // Rejecting requests
    HalfOpen,    // Testing recovery
}

pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub timeout_duration: Duration,
}

impl CircuitBreaker {
    pub fn call<F, R>(&mut self, f: F) -> Result<R>
    where
        F: FnOnce() -> Result<R>,
    {
        match self.state {
            CircuitState::Open => {
                // Check if timeout has elapsed
                if let Some(last_failure) = self.last_failure_time {
                    if last_failure.elapsed() > self.config.timeout_duration {
                        self.state = CircuitState::HalfOpen;
                        self.success_count = 0;
                    } else {
                        return Err(Error::CircuitBreakerOpen);
                    }
                }
            }
            _ => {}
        }
        
        match f() {
            Ok(result) => {
                self.on_success();
                Ok(result)
            }
            Err(e) => {
                self.on_failure();
                Err(e)
            }
        }
    }
    
    fn on_success(&mut self) {
        match self.state {
            CircuitState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.config.success_threshold {
                    self.state = CircuitState::Closed;
                    self.failure_count = 0;
                }
            }
            CircuitState::Closed => {
                self.failure_count = 0;
            }
            _ => {}
        }
    }
    
    fn on_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(Instant::now());
        
        if self.failure_count >= self.config.failure_threshold {
            self.state = CircuitState::Open;
        }
    }
}

// Usage
impl Component {
    fn send_with_circuit_breaker(&mut self, target: &str, request: Vec<u8>) -> Result<Vec<u8>> {
        self.circuit_breakers
            .entry(target.to_string())
            .or_insert_with(|| CircuitBreaker::new(CircuitBreakerConfig::default()))
            .call(|| self.send_request_sync(target, request.clone()))
    }
}
```

## Observability and Debugging

### Message Tracing

**Distributed Tracing:**

```rust
use opentelemetry::trace::{Tracer, Span, SpanKind};

impl MessageRouter {
    async fn route_message_with_tracing(
        &self,
        from: ComponentId,
        to: ComponentId,
        data: Vec<u8>,
        trace_context: Option<TraceContext>
    ) -> Result<()> {
        let tracer = opentelemetry::global::tracer("airssys-wasm");
        
        let mut span = tracer
            .span_builder(format!("message {} -> {}", from, to))
            .with_kind(SpanKind::Internal)
            .start(&tracer);
        
        if let Some(ctx) = trace_context {
            span.set_parent_context(ctx);
        }
        
        span.set_attribute(KeyValue::new("from", from.to_string()));
        span.set_attribute(KeyValue::new("to", to.to_string()));
        span.set_attribute(KeyValue::new("message_size", data.len() as i64));
        
        let result = self.route_message(from, to, data).await;
        
        match &result {
            Ok(_) => span.set_status(Status::Ok),
            Err(e) => {
                span.set_status(Status::error(format!("{:?}", e)));
                span.record_error(e);
            }
        }
        
        result
    }
}
```

### Audit Logging

**Comprehensive Message Audit:**

```rust
pub struct MessageAuditLog {
    log: Vec<AuditEntry>,
    config: AuditConfig,
}

pub struct AuditEntry {
    pub timestamp: DateTime<Utc>,
    pub event_type: AuditEventType,
    pub from: ComponentId,
    pub to: Option<ComponentId>,
    pub request_id: Option<String>,
    pub message_size: usize,
    pub result: AuditResult,
}

pub enum AuditEventType {
    MessageSent,
    MessageReceived,
    MessageRejected,
    RequestSent,
    RequestReceived,
    CallbackDelivered,
    RequestTimeout,
    RequestCancelled,
}

pub enum AuditResult {
    Success,
    PermissionDenied,
    RateLimitExceeded,
    ComponentNotFound,
    Timeout,
    Error(String),
}

impl MessageAuditLog {
    pub fn log_message_sent(&mut self, from: ComponentId, to: ComponentId, size: usize) {
        self.log.push(AuditEntry {
            timestamp: Utc::now(),
            event_type: AuditEventType::MessageSent,
            from,
            to: Some(to),
            request_id: None,
            message_size: size,
            result: AuditResult::Success,
        });
        
        self.rotate_if_needed();
    }
    
    pub fn log_permission_denied(&mut self, from: ComponentId, to: ComponentId) {
        self.log.push(AuditEntry {
            timestamp: Utc::now(),
            event_type: AuditEventType::MessageRejected,
            from,
            to: Some(to),
            request_id: None,
            message_size: 0,
            result: AuditResult::PermissionDenied,
        });
    }
    
    pub fn query_by_component(&self, component: &ComponentId) -> Vec<&AuditEntry> {
        self.log.iter()
            .filter(|entry| &entry.from == component || entry.to.as_ref() == Some(component))
            .collect()
    }
    
    pub fn query_by_timerange(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<&AuditEntry> {
        self.log.iter()
            .filter(|entry| entry.timestamp >= start && entry.timestamp <= end)
            .collect()
    }
}
```

### Performance Metrics

**Real-time Metrics:**

```rust
pub struct MessagingMetrics {
    // Counters
    pub total_messages_sent: Counter,
    pub total_messages_received: Counter,
    pub total_requests_sent: Counter,
    pub total_callbacks_delivered: Counter,
    pub total_timeouts: Counter,
    pub total_errors: Counter,
    
    // Histograms
    pub message_send_latency: Histogram,
    pub message_receive_latency: Histogram,
    pub request_response_latency: Histogram,
    pub callback_latency: Histogram,
    pub message_size: Histogram,
    
    // Gauges
    pub active_pending_callbacks: Gauge,
    pub queue_depths: HashMap<ComponentId, Gauge>,
}

impl MessagingMetrics {
    pub fn export_prometheus(&self) -> String {
        // Export metrics in Prometheus format
        format!(
            "# HELP airssys_messages_sent_total Total messages sent\n\
             # TYPE airssys_messages_sent_total counter\n\
             airssys_messages_sent_total {}\n\
             \n\
             # HELP airssys_message_send_latency_seconds Message send latency\n\
             # TYPE airssys_message_send_latency_seconds histogram\n\
             {}\n",
            self.total_messages_sent.get(),
            self.message_send_latency.export_prometheus()
        )
    }
}
```

## Testing Patterns

### Unit Testing Components

**Mock Host Services:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use airssys_wasm_test::MockHostServices;
    
    #[test]
    fn test_send_message() {
        let mut mock_host = MockHostServices::new();
        
        // Configure mock expectations
        mock_host.expect_send_message()
            .with(eq(ComponentId::from("target")), any())
            .times(1)
            .returning(|_, _| Ok(()));
        
        // Test component
        let mut component = Component::new();
        component.send_notification("target", "test message").unwrap();
        
        // Verify
        mock_host.verify();
    }
    
    #[test]
    fn test_handle_message() {
        let sender = ComponentId::from("sender");
        let message = encode_test_message();
        
        let result = handle_message_impl(sender, &message);
        
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_callback_timeout() {
        let mut mock_host = MockHostServices::new();
        
        mock_host.expect_send_request()
            .returning(|_, _, _| Ok("request-123".to_string()));
        
        let mut component = Component::new();
        component.send_request_with_timeout("target", vec![], 1000).unwrap();
        
        // Simulate timeout
        let timeout_error = CallbackError::Timeout {
            request_id: "request-123".to_string(),
            timeout_ms: 1000,
        };
        
        let result = handle_callback_impl("request-123", &encode_borsh(&timeout_error).unwrap(), true);
        
        assert!(result.is_err());
    }
}
```

### Integration Testing

**Multi-Component Test:**

```rust
#[cfg(test)]
mod integration_tests {
    use airssys_wasm_test::TestRuntime;
    
    #[tokio::test]
    async fn test_request_response_flow() {
        let mut runtime = TestRuntime::new();
        
        // Load components
        let requester = runtime.load_component("requester.wasm").await.unwrap();
        let responder = runtime.load_component("responder.wasm").await.unwrap();
        
        // Send request
        let request_data = encode_test_request();
        runtime.execute_component(requester, &request_data).await.unwrap();
        
        // Wait for callback
        runtime.wait_for_callbacks(Duration::from_secs(5)).await;
        
        // Verify callback was delivered
        let callbacks = runtime.get_delivered_callbacks(requester);
        assert_eq!(callbacks.len(), 1);
        assert!(callbacks[0].is_ok());
    }
    
    #[tokio::test]
    async fn test_message_routing() {
        let mut runtime = TestRuntime::new();
        
        let sender = runtime.load_component("sender.wasm").await.unwrap();
        let receiver = runtime.load_component("receiver.wasm").await.unwrap();
        
        // Send message
        let message = encode_test_message();
        runtime.send_message(sender, receiver, message).await.unwrap();
        
        // Verify message delivered
        let received = runtime.get_received_messages(receiver);
        assert_eq!(received.len(), 1);
    }
}
```

## Migration Guide

### From `receive-message` Polling to Push Delivery

**Old Pattern (DEPRECATED):**

```rust
// ❌ OLD - Polling anti-pattern
fn execute(operation: &[u8]) -> Result<Vec<u8>> {
    // Main operation processing
    let result = process_operation(operation)?;
    
    // Check for messages (polling!)
    while let Some(msg) = host::receive_message()? {
        handle_incoming_message(msg)?;
    }
    
    Ok(result)
}
```

**New Pattern (CURRENT):**

```rust
// ✅ NEW - Push delivery
#[export_name = "execute"]
pub extern "C" fn execute(/* ... */) -> i32 {
    // Only handles external RPC
    let result = process_operation(operation);
    // No message polling needed!
}

#[export_name = "handle-message"]
pub extern "C" fn handle_message(/* ... */) -> i32 {
    // Messages delivered here automatically
    let result = handle_incoming_message(sender, message);
}
```

**Migration Steps:**

1. **Add `handle-message` export** to component
2. **Remove polling loops** from `execute` method
3. **Move message handling logic** to `handle-message`
4. **Test with new runtime** that delivers messages via push
5. **Remove `receive-message` calls** completely

**Compatibility Shim (Temporary):**

```rust
// For gradual migration - host can provide compatibility
impl HostRuntime {
    fn provide_compatibility_receive_message(&self, component: &ComponentId) -> Option<Message> {
        // Deprecated - log warning
        log::warn!("Component {} using deprecated receive-message - migrate to handle-message", component);
        
        // Return buffered message if any
        self.component_message_buffers.get(component)
            .and_then(|buffer| buffer.pop_front())
    }
}
```

---

**Document Status:** Complete messaging architecture specification  
**Next Actions:**
- Begin Phase 1 implementation with core messaging runtime
- Implement message routing engine integrated with airssys-rt
- Create testing framework for messaging patterns
- Develop example components demonstrating all patterns

**Cross-References:**
- KNOWLEDGE-WASM-004: WIT Interface Definitions
- Future: KNOWLEDGE-WASM-006: Component Security Model
- Future: KNOWLEDGE-WASM-007: Performance Optimization Guide
