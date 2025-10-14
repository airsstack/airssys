# RT-TASK-009 Phase 1: ADR-RT-008 Implementation Summary

**Created:** 2025-10-14  
**Status:** Ready for Implementation  
**ADR Reference:** ADR-RT-008 - OSL Message Wrapper Pattern for Cloneable Messages

---

## Decision Summary

**Problem**: OSL messages with `oneshot::Sender` can't implement `Clone`, but `Message` trait requires `Clone`.

**Solution**: Wrapper pattern with cloneable messages + broker-based response routing.

**Pattern**:
```
Request (Clone) → OSL Actor → Response (Clone) → via Broker → Application Actor
```

---

## Implementation Checklist

### Phase 1A: Refactor Message Types ✅ DOCUMENTED

- [x] Create ADR-RT-008 with complete architecture
- [x] Update ADR index
- [x] Update RT-TASK-009 with ADR reference
- [x] Document wrapper pattern design

### Phase 1B: Implement Message Types (NEXT)

- [ ] Create `FileSystemOperation` enum (cloneable)
- [ ] Create `FileSystemRequest` struct with `reply_to` and `request_id`
- [ ] Create `FileSystemResponse` struct with `request_id` and `result`
- [ ] Implement `Message` trait for Request/Response
- [ ] Repeat for `Process*` and `Network*` types
- [ ] Remove old message types with oneshot channels

### Phase 1C: Fix Actor Implementations

- [ ] Add `type Error` to all OSL actors
- [ ] Implement `Actor::handle_message()` (not `handle()`)
- [ ] Send responses via `context.broker().publish()`
- [ ] Remove `ActorLifecycle` impl (it's a struct, not trait)
- [ ] Fix `Child` trait methods (no context params, add timeout)
- [ ] Remove unused imports

### Phase 1D: Compilation & Testing

- [ ] Run `cargo check` - should compile with zero warnings
- [ ] Run `cargo test` - embedded tests should pass
- [ ] Create unit tests for message serialization
- [ ] Create integration tests for request-response flow

---

## Key Code Patterns

### Message Structure

```rust
// 1. Operation (cloneable enum)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileSystemOperation {
    ReadFile { path: PathBuf },
    WriteFile { path: PathBuf, content: Vec<u8> },
    // ...
}

// 2. Request (cloneable struct)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSystemRequest {
    pub operation: FileSystemOperation,
    pub reply_to: ActorAddress,
    pub request_id: MessageId,
}

impl Message for FileSystemRequest {
    const MESSAGE_TYPE: &'static str = "osl::filesystem::request";
}

// 3. Response (cloneable struct)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSystemResponse {
    pub request_id: MessageId,
    pub result: FileSystemResult,
}

impl Message for FileSystemResponse {
    const MESSAGE_TYPE: &'static str = "osl::filesystem::response";
}
```

### Actor Implementation

```rust
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
            // ...
        };
        
        // Create response
        let response = FileSystemResponse {
            request_id: message.request_id,
            result,
        };
        
        // Send via broker
        context.broker()
            .publish(message.reply_to, response)
            .await?;
        
        Ok(())
    }
}

impl Child for FileSystemActor {
    type Error = FileSystemError;
    
    async fn start(&mut self) -> Result<(), Self::Error> {
        // No context parameter
        Ok(())
    }
    
    async fn stop(&mut self, timeout: Duration) -> Result<(), Self::Error> {
        // Timeout parameter, no context
        Ok(())
    }
    
    async fn health_check(&self) -> ChildHealth {
        // No context parameter
        ChildHealth::Healthy
    }
}
```

---

## Files to Modify

### Core Message Files
- `airssys-rt/src/osl/actors/messages.rs` - Complete rewrite with wrapper pattern

### Actor Implementation Files
- `airssys-rt/src/osl/actors/filesystem.rs` - Fix Actor + Child implementations
- `airssys-rt/src/osl/actors/process.rs` - Fix Actor + Child implementations
- `airssys-rt/src/osl/actors/network.rs` - Fix Actor + Child implementations

### Test Files (To Create)
- `airssys-rt/tests/osl_actors_tests.rs` - Integration tests
- Embedded unit tests in each actor file

---

## Expected Compilation Fixes

### Before (33 errors):
```
error[E0277]: the trait bound `FileSystemMessage: Clone` is not satisfied
error[E0432]: unresolved import `crate::supervisor::ChildContext`
error[E0407]: method `handle` is not a member of trait `Actor`
error[E0404]: expected trait, found struct `ActorLifecycle`
error[E0046]: not all trait items implemented, missing: `Error`, `handle_message`
```

### After (0 errors):
```
Compiling airssys-rt v0.1.0
Finished dev [unoptimized + debuginfo] target(s) in X.XXs
```

---

## Testing Strategy

### Unit Tests (Embedded)
- Message serialization/deserialization
- Actor state management
- Health check logic
- Error handling

### Integration Tests
- Request-response flow through broker
- Message correlation via MessageId
- Supervisor lifecycle management
- Failure isolation between supervisors

### Mock Testing
- Application actors with mock OSL actors
- Response verification
- Timeout handling

---

## Success Criteria

- [ ] Zero compilation errors
- [ ] Zero warnings
- [ ] All messages implement `Clone + Message`
- [ ] All actors implement `Actor + Child`
- [ ] Request-response works via broker
- [ ] Unit tests pass (>95% coverage)
- [ ] Integration tests demonstrate full flow
- [ ] Documentation examples work

---

## Related Documentation

- **ADR-RT-008**: Complete architectural decision and rationale
- **ADR-RT-007**: Hierarchical supervisor architecture context
- **RT-TASK-009**: Task specification and requirements
- **KNOWLEDGE-RT-017**: Implementation patterns (needs update)

---

## Timeline

**Estimated Time**: 3-4 hours for Phase 1B-1D

- **1 hour**: Refactor message types
- **1.5 hours**: Fix actor implementations
- **0.5 hour**: Compilation fixes and cleanup
- **1 hour**: Unit tests and validation

**Ready to begin implementation!**
