# TODO: OSL Integration Tests

## Status: PENDING - Task 2.3

**Removed:** `osl_actors_tests.rs` (December 2025)  
**Reason:** Tests incompatible with broker injection architecture (ADR-RT-009)

## Why Tests Were Removed

The original `osl_actors_tests.rs` integration tests were written for the **pre-refactoring architecture** where actors didn't require broker injection. After refactoring FileSystemActor and ProcessActor to generic `Actor<M, B>` patterns, these tests had fundamental type mismatches:

```rust
// Old test pattern (incompatible):
let broker = InMemoryMessageBroker::<TestOSLMessage>::new();
let actor = FileSystemActor::new(broker.clone());  // Actor<TestOSLMessage, Broker>
let context = ActorContext::new(addr, broker);      // Context<TestOSLMessage, Broker>

// But handle_message expects:
actor.handle_message(request, &mut context);        // ❌ Context<FileSystemRequest, Broker>
```

The `Actor` trait's `handle_message()` method expects `ActorContext<Self::Message, B>` where `Self::Message` is the **request type** (FileSystemRequest), but actors are now generic over a **unified message type** (OSLMessage or TestOSLMessage) that includes both requests and responses.

## What Needs to Be Recreated (Task 2.3)

After completing OSL integration (Task 2.1.2 - making OSLSupervisor generic), we need to create **new broker-based integration tests** that properly test the message flow architecture.

### Required Test Coverage

Create `tests/supervisor_hierarchy_tests.rs` with:

#### 1. Supervisor Creation Tests (3 tests)
- ✅ OSLSupervisor creation with shared broker
- ✅ Actor registration and startup
- ✅ Actor address configuration

#### 2. Cross-Supervisor Communication Tests (4 tests)
- ✅ FileSystem → Process actor communication via broker
- ✅ Process → Network actor communication via broker
- ✅ Message correlation with request_id
- ✅ Response routing to correct reply_to address

#### 3. Fault Isolation Tests (5 tests)
- ✅ FileSystemActor restart doesn't affect other actors
- ✅ ProcessActor crash triggers RestForOne strategy
- ✅ NetworkActor failure recovery
- ✅ Supervisor health check during failures
- ✅ Message loss prevention during restarts

#### 4. Lifecycle Management Tests (3 tests)
- ✅ Graceful shutdown sequence (Network → Process → FileSystem)
- ✅ Resource cleanup verification
- ✅ Idempotent start/stop operations

### New Test Architecture Pattern

```rust
// Correct broker-based integration test pattern:

#[tokio::test]
async fn test_filesystem_operation_via_broker() {
    // 1. Create shared broker with OSLMessage type
    let broker = InMemoryMessageBroker::<OSLMessage>::new();
    
    // 2. Create OSLSupervisor with broker
    let supervisor = OSLSupervisor::new(broker.clone());
    supervisor.start().await.unwrap();
    
    // 3. Subscribe to responses
    let mut response_rx = broker.subscribe(ActorAddress::named("test-client")).await;
    
    // 4. Publish request
    let request = OSLMessage::FileSystemReq(FileSystemRequest {
        operation: FileSystemOperation::ReadFile { path: "/test".into() },
        reply_to: ActorAddress::named("test-client"),
        request_id: MessageId::new(),
    });
    
    broker.publish(request, supervisor.filesystem_addr().clone()).await.unwrap();
    
    // 5. Await response via broker
    let response = tokio::time::timeout(
        Duration::from_secs(1),
        response_rx.recv()
    ).await.unwrap().unwrap();
    
    // 6. Verify response
    match response {
        OSLMessage::FileSystemResp(resp) => {
            assert_eq!(resp.request_id, request.request_id);
            // Verify response data
        }
        _ => panic!("Expected FileSystemResponse"),
    }
    
    // 7. Cleanup
    supervisor.shutdown(Duration::from_secs(1)).await.unwrap();
}
```

### Key Differences from Old Tests

| Aspect | Old Tests (Removed) | New Tests (Required) |
|--------|---------------------|---------------------|
| **Actor Creation** | Direct `Actor::new()` | Via `OSLSupervisor::start()` |
| **Message Type** | Request-specific context | Unified OSLMessage enum |
| **Communication** | Direct `handle_message()` | Broker publish/subscribe |
| **Response Verification** | N/A (void returns) | Subscribe and receive via broker |
| **Supervision** | No supervision | Full supervision tree testing |
| **Isolation** | Individual actor tests | Cross-actor communication tests |

## Test Files to Create

1. **tests/supervisor_hierarchy_tests.rs** - Main integration tests (15 tests)
2. **tests/osl_broker_communication_tests.rs** - Broker message flow tests (8 tests)
3. **tests/osl_fault_tolerance_tests.rs** - Failure and recovery scenarios (10 tests)

## Dependencies

- ✅ Task 2.1.1: All three actors refactored (FileSystem, Process, Network)
- ✅ Task 2.1.2: OSLSupervisor made generic with shared broker
- ⏳ Task 2.2: Example application (reference implementation)

## Estimated Effort

**4-5 hours** - Comprehensive broker-based integration test suite

## References

- **ADR-RT-009**: OSL Broker Dependency Injection architecture
- **Removed commit**: 811d966 (ProcessActor refactoring + test cleanup)
- **Memory Bank**: `.copilot/memory_bank/sub_projects/airssys-rt/`

---

**Note**: Do NOT attempt to recreate these tests until Task 2.1.2 (OSLSupervisor generic refactoring) is complete. The architecture must be stable first.
