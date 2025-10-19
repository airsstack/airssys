# [WASM-TASK-006] - Block 5: Inter-Component Communication

**Status:** not-started  
**Added:** 2025-10-20  
**Updated:** 2025-10-20  
**Priority:** Critical Path - Core Services Layer  
**Layer:** 2 - Core Services  
**Block:** 5 of 11  
**Estimated Effort:** 5-6 weeks  

## Overview

Implement the actor-based inter-component messaging system that enables secure, high-performance communication between WASM components through MessageBroker integration, supporting fire-and-forget and request-response patterns with multicodec self-describing serialization, capability-based security, and push-based event delivery achieving ~260ns messaging overhead.

## Context

**Current State:**
- Architecture complete: KNOWLEDGE-WASM-005 (Inter-Component Messaging Architecture)
- Foundation ready: Block 3 provides MessageBroker (~211ns routing proven)
- Security ready: Block 4 provides capability-based permission checks
- Integration: airssys-rt MessageBroker, actor mailboxes, push-based delivery

**Problem Statement:**
Components need to communicate with each other for:
1. **Event Notifications** - Fire-and-forget one-way messages (like gen_server:cast)
2. **RPC Requests** - Request-response with automatic callbacks (like gen_server:call)
3. **Pub-Sub Patterns** - Topic-based message filtering and delivery
4. **Security** - Capability checks prevent unauthorized messaging
5. **Performance** - Low-latency, high-throughput messaging (>10,000 msg/sec)

**Why This Block Matters:**
Without inter-component communication:
- Components are isolated islands (no cooperation)
- No event-driven architectures possible
- No microservices-style composition
- Framework limited to single-component use cases

This block enables the "composable components" value proposition.

**Critical Dependencies:**
- **REQUIRES Block 3**: MessageBroker routing, ComponentActor mailboxes
- **REQUIRES Block 4**: Capability-based message security
- **Performance Baseline**: airssys-rt MessageBroker proven at ~211ns routing

## Objectives

### Primary Objective
Implement actor-based inter-component messaging with MessageBroker integration, fire-and-forget and request-response patterns, multicodec serialization, capability enforcement, and push-based delivery achieving ~260ns total messaging overhead (~211ns routing + ~49ns validation/serialization).

### Secondary Objectives
- Achieve 4.7M messages/sec throughput (MessageBroker proven capacity)
- Implement automatic request-response correlation and timeouts
- Support multicodec self-describing message format
- Create comprehensive message security and quota system
- Establish message tracing and audit logging

## Scope

### In Scope
1. **MessageBroker Integration** - airssys-rt MessageBroker for routing
2. **Fire-and-Forget Pattern** - One-way async messages (~280ns)
3. **Request-Response Pattern** - RPC with callbacks (~560ns round-trip)
4. **Multicodec Serialization** - Self-describing message format
5. **Push-Based Delivery** - handle-message export invocation
6. **Message Security** - Capability checks and quotas
7. **Topic Management** - Subscription and filtering
8. **Request Correlation** - Automatic request_id management
9. **Timeout Handling** - Request timeout enforcement

### Out of Scope
- Message persistence/durability (Phase 2)
- Cross-host messaging (Phase 2)
- Message ordering guarantees beyond actor mailbox FIFO
- Distributed transactions (not aligned with actor model)
- Synchronous blocking calls (async-only architecture)

## Implementation Plan

### Phase 1: MessageBroker Integration Foundation (Week 1-2)

#### Task 1.1: MessageBroker Setup for Components
**Deliverables:**
- MessageBroker instance initialization
- Component subscription management
- Topic routing configuration
- ActorSystem as primary subscriber pattern
- MessageBroker performance validation

**Success Criteria:**
- MessageBroker routes component messages
- Components can subscribe to topics
- Routing performance: ~211ns (airssys-rt baseline)
- ActorSystem subscribes to all component topics
- Performance validated with benchmarks

#### Task 1.2: ComponentActor Message Reception
**Deliverables:**
- Actor mailbox integration
- Message queue management per component
- Backpressure handling (mailbox overflow)
- Message delivery to WASM handle-message export
- Message reception tests

**Success Criteria:**
- Messages delivered to ComponentActor mailbox
- WASM handle-message invoked with push delivery
- Backpressure prevents mailbox overflow
- Failed delivery handled gracefully
- Comprehensive test coverage

#### Task 1.3: Topic Subscription System
**Deliverables:**
- Topic subscription API
- Dynamic subscription management
- Topic pattern matching (wildcards)
- Subscription persistence
- Subscription documentation

**Success Criteria:**
- Components subscribe to topics at runtime
- Wildcards work (`events.*`, `logs.error.*`)
- Subscriptions persist across restarts
- Clear subscription management API
- Examples demonstrate patterns

---

### Phase 2: Fire-and-Forget Messaging (Week 2-3)

#### Task 2.1: send-message Host Function
**Deliverables:**
- `send-message` WIT interface implementation
- Message serialization (multicodec)
- Target component resolution
- MessageBroker publish integration
- Error handling (component not found, serialization failure)

**Success Criteria:**
- Components can send fire-and-forget messages
- Multicodec format works correctly
- Target resolution by component ID
- Errors return clear status codes
- Unit tests comprehensive

#### Task 2.2: handle-message Component Export
**Deliverables:**
- `handle-message` WIT interface specification
- Push-based message delivery to WASM
- Sender metadata (component ID, timestamp)
- Message deserialization
- Error propagation from component

**Success Criteria:**
- WASM components receive messages via export
- Sender information available
- Deserialization errors handled
- Component errors logged and supervised
- Examples demonstrate usage

#### Task 2.3: Fire-and-Forget Performance
**Deliverables:**
- End-to-end latency measurement
- Throughput benchmarks
- Overhead breakdown (routing, validation, serialization)
- Performance optimization
- Performance documentation

**Success Criteria:**
- Total latency: ~280ns (211ns routing + 49ns overhead + 20ns WASM call)
- Throughput: >10,000 msg/sec per component
- Overhead breakdown documented
- No performance regressions
- Benchmarks reproducible

---

### Phase 3: Request-Response Pattern (Week 3-4)

#### Task 3.1: send-request Host Function
**Deliverables:**
- `send-request` WIT interface implementation
- Request ID generation (UUID v4)
- Callback registration system
- Timeout management (tokio::time::timeout)
- Request tracking data structure

**Success Criteria:**
- Components can send requests
- Unique request IDs generated
- Callbacks registered with timeouts
- Request tracking efficient (O(1) lookup)
- Clear API documentation

#### Task 3.2: Response Routing and Callbacks
**Deliverables:**
- Response correlation by request ID
- Callback invocation (handle-callback export)
- Success and error response handling
- Callback cleanup after invocation
- Response routing tests

**Success Criteria:**
- Responses route to correct requesters
- handle-callback invoked with response data
- Success and error responses distinguished
- Callbacks cleaned up (no memory leaks)
- Round-trip latency ~560ns

#### Task 3.3: Timeout and Cancellation
**Deliverables:**
- Request timeout enforcement
- Timeout error delivery to callback
- Cancel-request API
- Timeout cleanup (remove stale callbacks)
- Edge case handling

**Success Criteria:**
- Timeouts trigger after configured duration
- Timeout errors delivered to handle-callback
- Cancellation works correctly
- No callback leaks on timeout
- Edge cases tested (component crash mid-request)

---

### Phase 4: Multicodec Serialization (Week 4)

#### Task 4.1: Multicodec Message Format
**Deliverables:**
- Multicodec message envelope structure
- Codec type field (e.g., 0x700 for cbor, 0x701 for borsh)
- Message payload encoding/decoding
- Format validation
- Format documentation

**Success Criteria:**
- Messages self-describe codec type
- Multiple codecs supported (cbor, borsh, bincode)
- Format validation catches invalid messages
- Clear format specification
- Examples for each codec

#### Task 4.2: Multi-Language Serialization Support
**Deliverables:**
- Rust serialization examples (serde)
- Codec selection in Component.toml
- Language-specific serialization patterns
- Cross-language messaging tests
- Serialization guide

**Success Criteria:**
- Rust components can use multiple codecs
- Component.toml declares preferred codec
- Cross-codec messaging works (with compatibility)
- Clear patterns for each language
- Documentation comprehensive

#### Task 4.3: Codec Compatibility Validation
**Deliverables:**
- Codec compatibility matrix
- Runtime codec validation (ADR-WASM-001)
- Fail-fast for incompatible codecs
- Clear error messages
- Compatibility documentation

**Success Criteria:**
- Incompatible codecs rejected at send time
- Clear error messages for mismatches
- Compatibility matrix documented
- No silent failures
- Validation tested thoroughly

---

### Phase 5: Message Security and Quotas (Week 5)

#### Task 5.1: Capability-Based Message Permissions
**Deliverables:**
- Message capability type (MessagingCapability)
- Permission checks at send-message/send-request
- Target component permission patterns
- Topic permission patterns
- Security integration tests

**Success Criteria:**
- Components declare messaging capabilities in Component.toml
- Unauthorized messaging denied
- Topic-based permissions work
- Clear permission errors
- Security validated thoroughly

#### Task 5.2: Message Rate Limiting
**Deliverables:**
- Per-component message quotas
- Rate limit enforcement (token bucket algorithm)
- Quota configuration in Component.toml
- Quota exceeded error handling
- Quota monitoring

**Success Criteria:**
- Message rate limits enforced per component
- Quota violations return clear errors
- Quotas configurable (messages/sec, burst)
- Quota usage queryable
- Prevents message spam

#### Task 5.3: Security Audit Logging
**Deliverables:**
- Message send/receive logging
- Security event logging (denied messaging)
- Audit log format (structured JSON)
- Log performance optimization
- Audit log documentation

**Success Criteria:**
- All messaging logged with context
- Security denials logged separately
- Structured format for analysis
- Logging overhead minimal (<5%)
- Clear audit trail

---

### Phase 6: Advanced Features and Testing (Week 5-6)

#### Task 6.1: Message Tracing
**Deliverables:**
- Trace ID propagation across messages
- Distributed tracing integration points
- Trace context in WIT interface
- Tracing performance overhead
- Tracing documentation

**Success Criteria:**
- Trace IDs propagate through message chains
- Integration with tracing systems (OpenTelemetry)
- Overhead acceptable (<2%)
- Clear tracing patterns
- Examples demonstrate usage

#### Task 6.2: Error Handling and Resilience
**Deliverables:**
- Component crash mid-message handling
- Dead letter queue for failed messages (Phase 2)
- Retry strategies (immediate delivery only in Phase 1)
- Error categorization
- Resilience tests

**Success Criteria:**
- Component crashes don't lose messages
- Failed messages logged clearly
- No cascading failures
- Error recovery documented
- Resilience validated

#### Task 6.3: Comprehensive Testing and Documentation
**Deliverables:**
- End-to-end messaging tests
- Performance benchmarks (latency, throughput)
- Security test suite
- Multi-component integration tests
- Complete messaging guide

**Success Criteria:**
- Test coverage >95%
- All messaging patterns tested
- Performance targets met
- Security validated
- Documentation comprehensive

---

## Success Criteria

### Definition of Done
This task is complete when:

1. ✅ **MessageBroker Integration Working**
   - airssys-rt MessageBroker routes component messages
   - Topic subscription functional
   - Routing performance: ~211ns baseline maintained
   - ActorSystem as primary subscriber pattern

2. ✅ **Fire-and-Forget Messaging Operational**
   - send-message host function implemented
   - handle-message push delivery working
   - Total latency: ~280ns average
   - Throughput: >10,000 msg/sec per component

3. ✅ **Request-Response Pattern Working**
   - send-request with automatic request ID generation
   - Response correlation and callback invocation
   - Timeout enforcement functional
   - Round-trip latency: ~560ns average

4. ✅ **Multicodec Serialization Implemented**
   - Self-describing message format
   - Multiple codec support (cbor, borsh, bincode)
   - Codec compatibility validation
   - Cross-language messaging working

5. ✅ **Message Security Enforced**
   - Capability-based messaging permissions
   - Message rate limiting functional
   - Security audit logging operational
   - No security bypass vulnerabilities

6. ✅ **Advanced Features Complete**
   - Message tracing with trace ID propagation
   - Error handling and resilience validated
   - Component crash recovery working
   - Clear error messages

7. ✅ **Testing & Documentation Complete**
   - Test coverage >95%
   - Performance benchmarks meet targets
   - Security tests comprehensive
   - Complete messaging guide with examples

## Dependencies

### Upstream Dependencies
- ✅ WASM-TASK-004: Actor System Integration (Block 3) - **REQUIRED** for MessageBroker
- ✅ WASM-TASK-005: Security & Isolation (Block 4) - **REQUIRED** for capability checks
- ✅ WASM-TASK-003: WIT Interface System (Block 2) - **REQUIRED** for messaging interfaces
- ✅ ADR-WASM-009: Component Communication Model - **COMPLETE**
- ✅ ADR-WASM-001: Multicodec Compatibility Strategy - **COMPLETE**
- ✅ KNOWLEDGE-WASM-005: Inter-Component Messaging Architecture - **COMPLETE**

### Downstream Dependencies (Blocks This Task)
- WASM-TASK-008: Component Lifecycle (Block 7) - needs messaging for coordination
- WASM-TASK-010: Monitoring & Observability (Block 9) - needs messaging metrics
- WASM-TASK-011: Component SDK (Block 10) - needs messaging API wrappers

### External Dependencies
- airssys-rt MessageBroker (InMemoryMessageBroker)
- airssys-rt Actor mailboxes
- Multicodec specification (Protocol Labs)
- Serde for serialization (Rust)

## Risks and Mitigations

### Risk 1: Performance Not Meeting Targets
**Impact:** High - Slow messaging could make framework unusable  
**Probability:** Low - airssys-rt MessageBroker proven at 211ns  
**Mitigation:**
- Build on proven airssys-rt MessageBroker performance
- Profile messaging path extensively
- Optimize serialization (zero-copy where possible)
- Benchmark continuously during development

### Risk 2: Request-Response Complexity
**Impact:** Medium - Correlation bugs could lose responses  
**Probability:** Medium - Callback management is complex  
**Mitigation:**
- Comprehensive correlation testing
- Request tracking with explicit lifecycle
- Timeout enforcement prevents leaks
- Extensive edge case testing

### Risk 3: Multicodec Compatibility Issues
**Impact:** Medium - Codec mismatches could break messaging  
**Probability:** Medium - Cross-language compatibility is hard  
**Mitigation:**
- Follow ADR-WASM-001 fail-fast strategy
- Comprehensive codec validation
- Clear error messages for mismatches
- Test cross-codec scenarios extensively

### Risk 4: Message Security Bypass
**Impact:** Critical - Security bypass defeats isolation  
**Probability:** Low - Building on Block 4 capability system  
**Mitigation:**
- Security review by experts
- Penetration testing with malicious components
- No message delivery without capability check
- Comprehensive security test suite

### Risk 5: Backpressure and Overload
**Impact:** High - Message storms could DoS components  
**Probability:** Medium - Distributed systems are prone to cascades  
**Mitigation:**
- Mailbox size limits with backpressure
- Message rate limiting per component
- Fast failure rather than queuing
- Monitor message queue depths

## Progress Tracking

**Overall Status:** not-started - 0%

### Phase Breakdown
| Phase | Description | Status | Estimated Duration | Notes |
|-------|-------------|--------|-------------------|-------|
| 1 | MessageBroker Integration Foundation | not-started | Week 1-2 | Routing foundation |
| 2 | Fire-and-Forget Messaging | not-started | Week 2-3 | Core pattern |
| 3 | Request-Response Pattern | not-started | Week 3-4 | RPC pattern |
| 4 | Multicodec Serialization | not-started | Week 4 | Language-agnostic |
| 5 | Message Security and Quotas | not-started | Week 5 | Security layer |
| 6 | Advanced Features and Testing | not-started | Week 5-6 | Production readiness |

### Subtasks
| ID | Description | Status | Updated | Notes |
|----|-------------|--------|---------|-------|
| 1.1 | MessageBroker Setup for Components | not-started | - | Foundation |
| 1.2 | ComponentActor Message Reception | not-started | - | Mailbox integration |
| 1.3 | Topic Subscription System | not-started | - | Pub-sub |
| 2.1 | send-message Host Function | not-started | - | Fire-and-forget |
| 2.2 | handle-message Component Export | not-started | - | Push delivery |
| 2.3 | Fire-and-Forget Performance | not-started | - | Performance target |
| 3.1 | send-request Host Function | not-started | - | RPC foundation |
| 3.2 | Response Routing and Callbacks | not-started | - | Correlation |
| 3.3 | Timeout and Cancellation | not-started | - | Resilience |
| 4.1 | Multicodec Message Format | not-started | - | Self-describing |
| 4.2 | Multi-Language Serialization Support | not-started | - | Language-agnostic |
| 4.3 | Codec Compatibility Validation | not-started | - | Fail-fast |
| 5.1 | Capability-Based Message Permissions | not-started | - | Security |
| 5.2 | Message Rate Limiting | not-started | - | Abuse prevention |
| 5.3 | Security Audit Logging | not-started | - | Audit trail |
| 6.1 | Message Tracing | not-started | - | Observability |
| 6.2 | Error Handling and Resilience | not-started | - | Production readiness |
| 6.3 | Comprehensive Testing and Documentation | not-started | - | Quality assurance |

## Progress Log

*No progress yet - task just created*

## Related Documentation

### ADRs
- **ADR-WASM-009: Component Communication Model** - Primary messaging architecture reference
- **ADR-WASM-001: Multicodec Compatibility Strategy** - Serialization strategy
- **ADR-WASM-006: Component Isolation and Sandboxing** - Actor-based architecture
- **ADR-WASM-005: Capability-Based Security Model** - Message permissions

### Knowledge Documentation
- **KNOWLEDGE-WASM-005: Inter-Component Messaging Architecture** - Complete messaging specification
- **KNOWLEDGE-WASM-006: Multiformat Strategy** - Multicodec integration
- **KNOWLEDGE-WASM-004: WIT Management Architecture** - Messaging WIT interfaces
- **KNOWLEDGE-RT-013: Actor Performance Benchmarking** - MessageBroker performance baseline

### airssys-rt References
- **RT-TASK-004: PubSub System Foundation** - MessageBroker implementation
- MessageBroker API documentation
- Actor mailbox patterns
- InMemoryMessageBroker performance characteristics

### External References
- [Multiformats Specification](https://github.com/multiformats/multiformats)
- [Multicodec Table](https://github.com/multiformats/multicodec)
- [Actor Model (Erlang OTP)](https://www.erlang.org/doc/design_principles/gen_server_concepts.html)
- [WebAssembly Component Model](https://github.com/WebAssembly/component-model)

## Notes

**Performance Baseline from airssys-rt:**
- MessageBroker routing: ~211ns (proven)
- Actor mailbox delivery: ~100ns (proven)
- Target total overhead: ~260ns (routing + validation + serialization)
- Fire-and-forget: ~280ns (211ns + 49ns overhead + 20ns WASM call)
- Request-response: ~560ns round-trip (2x fire-and-forget)

**Critical Dependencies:**
This block HEAVILY depends on Block 3 (Actor System Integration):
- MessageBroker for routing
- ComponentActor mailboxes for delivery
- ActorSystem as primary subscriber pattern
- SupervisorNode for fault tolerance

**Push-Based Delivery:**
NO polling required. Components export handle-message which host invokes directly.
This is NOT HTTP-style request-pull. It's Erlang-style message-push.

**Multicodec Self-Describing:**
Messages carry codec information (ADR-WASM-001). Host does NOT translate between codecs.
Components are responsible for codec compatibility. Fail-fast on mismatch.

**Actor Model Alignment:**
- Fire-and-forget: Like gen_server:cast in Erlang
- Request-response: Like gen_server:call in Erlang
- Components are gen_server-like actors

**Security Integration:**
Every send-message/send-request MUST check capabilities (Block 4).
No message delivery without permission. Rate limiting prevents abuse.

**Request Correlation:**
Host runtime manages request_id generation and callback correlation automatically.
Components don't need to implement correlation logic.

**Timeout Enforcement:**
Host runtime enforces timeouts using tokio::time::timeout.
Timeout errors delivered to handle-callback with error status.

**Topic Pattern Matching:**
Wildcard support: `events.*` matches `events.user.created`, `events.order.shipped`
Follows pub-sub best practices from message queuing systems.

**Phase 2 Enhancements:**
- Message persistence and durability
- Cross-host messaging (distributed components)
- Dead letter queues
- Advanced retry strategies
- Message replay capabilities
