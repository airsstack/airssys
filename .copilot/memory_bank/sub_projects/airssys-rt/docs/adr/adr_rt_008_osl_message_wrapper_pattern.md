# ADR-RT-008: OSL Message Wrapper Pattern for Cloneable Messages

**ADR ID:** ADR-RT-008  
**Created:** 2025-10-14  
**Updated:** 2025-10-14  
**Status:** Accepted  
**Deciders:** Architecture team, RT-TASK-009 implementation team  
**Supersedes:** Initial OSL actor design with oneshot channels

---

## Title
OSL Message Wrapper Pattern to Enable Clone-Compatible Actor Message Passing

---

## Context

### Problem Statement

During RT-TASK-009 Phase 1 implementation, we encountered a fundamental incompatibility between the OSL integration actor design and the airssys-rt Actor trait requirements:

**The Conflict:**
```rust
// airssys-rt Message trait REQUIRES Clone
pub trait Message: Send + Sync + Clone + Debug + 'static {
    const MESSAGE_TYPE: &'static str;
}

// BUT our OSL messages contain oneshot::Sender (NOT Clone)
pub enum FileSystemMessage {
    ReadFile {
        path: PathBuf,
        respond_to: oneshot::Sender<FileSystemResponse>, // ❌ Not Clone!
    },
}

// RESULT: Compilation error
impl Message for FileSystemMessage { ... } // ❌ Clone trait not satisfied
```

**Compilation Errors Encountered:**
```
error[E0277]: the trait bound `FileSystemMessage: Clone` is not satisfied
   --> airssys-rt/src/osl/actors/messages.rs:98:18
    |
98  | impl Message for FileSystemMessage {
    |                  ^^^^^^^^^^^^^^^^^ the trait `Clone` is not implemented
```

### Business Requirements

1. **Message Passing is Essential**: Application actors must communicate with OSL actors via message passing (not direct method calls)
2. **Supervised Service Architecture**: OSL actors must be supervised children (implement `Child` trait)
3. **Standard Actor Integration**: OSL actors should use standard `Actor` trait and `ActorSystem` infrastructure
4. **Request-Response Pattern**: OSL operations need response delivery mechanism
5. **Type Safety**: Compile-time verification of message types and operations

### Technical Context

**Current State:**
- airssys-rt Actor system requires `Clone` for all messages (for broker routing, retry, etc.)
- OSL operations are naturally request-response (need reply channel)
- Initial design used `oneshot::Sender` for simple, direct responses
- Supervisor framework expects both `Actor` + `Child` trait implementations

**Constraints:**
- Cannot modify core `Message` trait (breaking change for entire actor system)
- Cannot use `oneshot::Sender` in messages (not `Clone`)
- Must maintain type safety and performance
- Must integrate with existing `MessageBroker` and `ActorSystem`

**Related ADRs:**
- ADR-RT-007: Hierarchical Supervisor Architecture for OSL Integration
- ADR-RT-001: Zero-Cost Abstractions (performance requirements)
- ADR-006: Pub-Sub MessageBroker Architecture

---

## Decision

### Summary

**ACCEPTED: OSL Message Wrapper Pattern with Broker-Based Response Routing**

Redesign OSL messages to be fully cloneable by:
1. Separating request operations (cloneable) from response channels
2. Using `ActorAddress` or `MessageId` for reply routing
3. Sending responses through the `MessageBroker` instead of direct channels
4. Implementing standard `Actor` trait for full integration

### Architecture

```rust
// ============================================================================
// Layer 1: Cloneable Operation Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileSystemOperation {
    ReadFile { path: PathBuf },
    WriteFile { path: PathBuf, content: Vec<u8> },
    DeleteFile { path: PathBuf },
    ListDirectory { path: PathBuf },
    CreateDirectory { path: PathBuf },
    DeleteDirectory { path: PathBuf, recursive: bool },
}

// ============================================================================
// Layer 2: Request Message (Cloneable)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSystemRequest {
    /// The file system operation to perform
    pub operation: FileSystemOperation,
    
    /// Actor address to send response to
    pub reply_to: ActorAddress,
    
    /// Unique request identifier for correlation
    pub request_id: MessageId,
}

impl Message for FileSystemRequest {
    const MESSAGE_TYPE: &'static str = "osl::filesystem::request";
}

// ============================================================================
// Layer 3: Response Message (Cloneable)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSystemResponse {
    /// Original request ID for correlation
    pub request_id: MessageId,
    
    /// Operation result
    pub result: FileSystemResult,
}

impl Message for FileSystemResponse {
    const MESSAGE_TYPE: &'static str = "osl::filesystem::response";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileSystemResult {
    ReadSuccess { content: Vec<u8> },
    WriteSuccess,
    DeleteSuccess,
    ListSuccess { entries: Vec<DirEntry> },
    CreateSuccess,
    DeleteDirectorySuccess,
    Error { error: FileSystemError },
}

// ============================================================================
// Layer 4: Actor Implementation
// ============================================================================

impl Actor for FileSystemActor {
    type Message = FileSystemRequest;
    type Error = FileSystemError;
    
    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        // Perform operation
        let result = match message.operation {
            FileSystemOperation::ReadFile { path } => {
                self.read_file_internal(path).await
            }
            FileSystemOperation::WriteFile { path, content } => {
                self.write_file_internal(path, content).await
            }
            // ... other operations
        };
        
        // Create response
        let response = FileSystemResponse {
            request_id: message.request_id,
            result,
        };
        
        // Send response back via broker
        context.broker()
            .publish(message.reply_to, response)
            .await?;
        
        Ok(())
    }
}

impl Child for FileSystemActor {
    type Error = FileSystemError;
    // ... lifecycle methods
}
```

### Usage Pattern

```rust
// Application actor sends request to FileSystemActor
pub struct AppActor {
    filesystem_addr: ActorAddress,
    pending_requests: HashMap<MessageId, PendingRequest>,
}

impl Actor for AppActor {
    type Message = AppMessage;
    type Error = AppError;
    
    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        match message {
            AppMessage::ProcessFile { path } => {
                // Create request
                let request_id = MessageId::new();
                let request = FileSystemRequest {
                    operation: FileSystemOperation::ReadFile { path },
                    reply_to: context.address().clone(),
                    request_id,
                };
                
                // Track pending request
                self.pending_requests.insert(request_id, PendingRequest::new());
                
                // Send to FileSystemActor via broker
                context.broker()
                    .publish(self.filesystem_addr.clone(), request)
                    .await?;
            }
            
            AppMessage::FileSystemResponse(response) => {
                // Correlate with pending request
                if let Some(pending) = self.pending_requests.remove(&response.request_id) {
                    // Process response
                    self.handle_filesystem_response(response, pending).await?;
                }
            }
        }
        Ok(())
    }
}
```

### Key Design Decisions

1. **Cloneable Messages**: All OSL message types implement `Clone` by removing `oneshot::Sender`
2. **Request-Response Correlation**: Use `MessageId` to correlate requests with responses
3. **Broker-Based Routing**: Responses sent via `MessageBroker.publish()` to `reply_to` address
4. **Standard Actor Trait**: OSL actors implement full `Actor` trait for complete integration
5. **Dual Trait Implementation**: OSL actors implement both `Actor` (for messages) and `Child` (for supervision)

---

## Rationale

### Why Option 3 (Wrapper Pattern) Over Alternatives

**Rejected Alternative 1: Child-Only Implementation**
```rust
// Only implement Child trait
impl Child for FileSystemActor { ... }

// Direct method calls (no messages)
let response = fs_actor.read_file(path).await?;
```

❌ **Rejected because:**
- No message passing capability
- Tight coupling between application actors and OSL actors
- Defeats the purpose of actor-based OSL integration
- Not aligned with ADR-RT-007's service-oriented actor model

**Rejected Alternative 2: Keep oneshot::Sender, Custom OslActor Trait**
```rust
pub trait OslMessage: Send + Sync + Debug + 'static {
    const MESSAGE_TYPE: &'static str;
    // No Clone requirement
}

pub trait OslActor: Child {
    type Message: OslMessage;
    async fn handle(&mut self, msg: Self::Message) -> Result<(), Self::Error>;
}
```

❌ **Rejected because:**
- Cannot use existing `ActorSystem` infrastructure
- Cannot integrate with `MessageBroker`
- Need custom spawning and management logic
- Fragments actor system into two incompatible subsystems
- Higher maintenance burden (two actor systems to maintain)

**Accepted Option 3: Wrapper Pattern with Broker Routing**

✅ **Accepted because:**

1. **Full Standard Integration** ⭐⭐⭐⭐⭐
   - Uses standard `Actor` trait (no custom trait needed)
   - Works with existing `ActorSystem` spawning
   - Integrates with `MessageBroker` pub-sub
   - Supervised via standard `Child` trait

2. **Message Passing Preserved** ⭐⭐⭐⭐⭐
   - Application actors communicate via messages
   - Loose coupling (actors only know addresses)
   - Asynchronous request-response pattern
   - Aligns with actor model principles

3. **Type Safety** ⭐⭐⭐⭐
   - Compile-time message type checking
   - Request-response correlation via `MessageId`
   - Strongly typed operations and results

4. **Performance** ⭐⭐⭐⭐
   - Clone overhead minimal (no heap allocations for addresses/IDs)
   - Broker routing is already optimized
   - No additional abstraction layers

5. **Testability** ⭐⭐⭐⭐⭐
   - Mock OSL actors with same message interface
   - Test request-response flows
   - Verify broker routing
   - Standard actor testing patterns apply

6. **BEAM/OTP Alignment** ⭐⭐⭐⭐
   - Similar to Erlang's `gen_server:call/2` pattern
   - Request-response via message passing
   - Supervised service architecture

### Trade-offs

**Benefits:**
- ✅ Full integration with actor system infrastructure
- ✅ Message passing between all actors
- ✅ Type-safe request-response pattern
- ✅ Standard supervision works
- ✅ No custom actor traits needed
- ✅ Testable with mock actors

**Costs:**
- ⚠️ Request-response requires two message passes (request + response)
- ⚠️ Manual correlation via `MessageId` (more code than oneshot)
- ⚠️ Application actors must handle response messages
- ⚠️ Slightly more complex than direct channel pattern

**Mitigations:**
- Performance cost is minimal (message passing is already async)
- Helper functions can simplify request-response correlation
- Pattern is well-understood in actor systems
- Benefits outweigh complexity cost

---

## Consequences

### Positive

1. **OSL actors fully integrated** with actor runtime (spawn, supervise, message passing)
2. **Standard patterns** apply (no special-case code needed)
3. **Loose coupling** between application actors and OSL service actors
4. **Testability** via mock actors with same message interface
5. **Scalability** - can add more OSL actors without changing infrastructure
6. **Future extensibility** - easy to add new OSL operations as new messages

### Negative

1. **Increased message volume** - two messages per request-response (vs. one with oneshot)
2. **Manual correlation** - application actors must track pending requests
3. **Response handling** - application actors need response message handlers
4. **Code complexity** - slightly more complex than direct channel pattern

### Neutral

1. **Learning curve** - developers must understand request-response pattern
2. **Documentation** - need examples showing request-response flow
3. **Helper utilities** - may want helper functions for common patterns

---

## Implementation Plan

### Phase 1: Refactor Message Types (Immediate)

1. **Redesign OSL messages**:
   - Create `*Operation` enums (cloneable)
   - Create `*Request` structs with `reply_to` and `request_id`
   - Create `*Response` structs with `request_id` and `result`
   - Implement `Message` trait for all

2. **Update Actor implementations**:
   - Remove `impl Actor` with old message types
   - Add proper `Actor` implementation with new request messages
   - Handle operation, send response via broker
   - Keep `impl Child` for supervision

3. **Fix compilation errors**:
   - Remove unused imports
   - Fix trait method signatures
   - Add `type Error` associated types

### Phase 2: Helper Utilities (Future)

```rust
// Helper for request-response pattern
pub struct RequestResponseHelper<Req, Resp> {
    pending: HashMap<MessageId, oneshot::Sender<Resp>>,
}

impl<Req, Resp> RequestResponseHelper<Req, Resp> {
    pub async fn send_request<B>(
        &mut self,
        request: Req,
        target: ActorAddress,
        context: &mut ActorContext<_, B>,
    ) -> Result<oneshot::Receiver<Resp>, Error>
    where
        B: MessageBroker<Req>,
        Req: Message + HasRequestId,
    {
        let (tx, rx) = oneshot::channel();
        self.pending.insert(request.request_id(), tx);
        context.broker().publish(target, request).await?;
        Ok(rx)
    }
    
    pub fn handle_response(&mut self, response: Resp) -> Result<(), Error>
    where
        Resp: HasRequestId,
    {
        if let Some(tx) = self.pending.remove(&response.request_id()) {
            let _ = tx.send(response);
        }
        Ok(())
    }
}
```

### Phase 3: Documentation & Examples (Future)

- Create example showing application actor → OSL actor request-response
- Document request-response pattern in mdBook
- Add integration tests for OSL actor message passing

---

## Validation

### Acceptance Criteria

- [ ] All OSL message types implement `Clone`
- [ ] All OSL actors implement `Actor` + `Child` traits
- [ ] Compilation succeeds with zero warnings
- [ ] Request-response pattern works via broker
- [ ] Response correlation via `MessageId` works
- [ ] Unit tests pass with >95% coverage
- [ ] Integration tests validate message passing flow

### Testing Strategy

1. **Unit Tests**: Each actor handles requests and sends responses
2. **Integration Tests**: Full request-response flow through broker
3. **Mock Tests**: Application actors with mock OSL actors
4. **Supervisor Tests**: OSL actors supervised and restarted correctly

---

## Related Documentation

- **ADR-RT-007**: Hierarchical Supervisor Architecture for OSL Integration
- **KNOWLEDGE-RT-017**: OSL Integration Actors Pattern (needs update)
- **Task**: RT-TASK-009 Phase 1 - OSL Integration Actors

---

## Notes

**Decision Date:** 2025-10-14  
**Implementation Status:** Approved, ready for implementation  
**Next Steps:** Refactor message types and actor implementations per this ADR

**Key Insight:** The `Clone` requirement on `Message` trait is fundamental to the actor system's broker routing and retry mechanisms. Rather than fight this requirement, we embrace it by designing messages that are naturally cloneable. The request-response pattern via broker is a well-established pattern in actor systems (Erlang's `gen_server:call`, Akka's `ask` pattern) and provides better decoupling than direct channels.
