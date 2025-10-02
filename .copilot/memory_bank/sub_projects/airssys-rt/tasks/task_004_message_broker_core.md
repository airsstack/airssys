# [RT-TASK-004] - Message Broker Core

**Status:** pending  
**Added:** 2025-10-02  
**Updated:** 2025-10-02

## Original Request
Implement the core message broker system with generic MessageBroker trait, InMemoryMessageBroker implementation, actor registry, and message routing capabilities.

## Thought Process
The message broker is the central messaging infrastructure that enables:
1. Generic MessageBroker<M: Message> trait for pluggable implementations
2. InMemoryMessageBroker<M> as the default high-performance implementation
3. Actor registry for address resolution and routing
4. Request-reply pattern support with message ID tracking
5. Message delivery with error handling and retries
6. Integration with mailbox system for reliable delivery

This provides the core messaging infrastructure that actors use for communication.

## Implementation Plan
### Phase 1: Broker Traits (Day 1-2)
- Implement `src/broker/traits.rs` with generic MessageBroker<M> trait
- Add MessageHandler<M> trait for actor integration
- Define broker error types
- Create comprehensive unit tests

### Phase 2: Actor Registry (Day 3-4)
- Implement `src/broker/registry.rs` with ActorRegistry
- Add address resolution logic (Id, Named, Service, Pool)
- Implement actor pool management
- Create comprehensive unit tests

### Phase 3: InMemory Broker (Day 5-6)
- Implement `src/broker/in_memory.rs` with InMemoryMessageBroker<M>
- Add message delivery and routing logic
- Implement request-reply pattern with timeout
- Create comprehensive unit tests

### Phase 4: Message Delivery (Day 7-8)
- Implement `src/broker/delivery.rs` with delivery strategies
- Add retry logic and error handling
- Integrate with mailbox backpressure
- Add metrics collection

## Progress Tracking

**Overall Status:** not_started - 0%

### Subtasks
| ID | Description | Status | Updated | Notes |
|----|-------------|--------|---------|-------|
| 4.1 | MessageBroker trait definition | not_started | 2025-10-02 | Generic broker interface |
| 4.2 | MessageHandler trait | not_started | 2025-10-02 | Actor integration interface |
| 4.3 | Broker error types | not_started | 2025-10-02 | Comprehensive error handling |
| 4.4 | ActorRegistry implementation | not_started | 2025-10-02 | Address resolution system |
| 4.5 | Actor pool management | not_started | 2025-10-02 | Load balancing and routing |
| 4.6 | InMemoryMessageBroker core | not_started | 2025-10-02 | Default broker implementation |
| 4.7 | Request-reply pattern | not_started | 2025-10-02 | Message ID tracking and timeout |
| 4.8 | Message delivery system | not_started | 2025-10-02 | Reliable delivery with retries |
| 4.9 | Metrics collection | not_started | 2025-10-02 | Broker performance monitoring |
| 4.10 | Unit test coverage | not_started | 2025-10-02 | Comprehensive tests in each module |

## Progress Log
### 2025-10-02
- Task created with detailed implementation plan
- Depends on RT-TASK-001 Message System and RT-TASK-003 Mailbox System
- Architecture design finalized with generic constraints
- Estimated duration: 7-8 days

## Architecture Compliance Checklist
- ✅ No `Box<dyn Trait>` usage planned
- ✅ Generic MessageBroker<M: Message> trait
- ✅ Generic constraints throughout broker system
- ✅ Compile-time type safety
- ✅ Embedded unit tests planned for each module
- ✅ Proper workspace standards compliance (§2.1-§6.3)

## Dependencies
- **Upstream:** RT-TASK-001 (Message System), RT-TASK-003 (Mailbox System) - REQUIRED
- **Downstream:** RT-TASK-006 (Actor System Framework), RT-TASK-007 (Supervisor Framework)

## Definition of Done
- [ ] Generic MessageBroker<M> trait implemented
- [ ] MessageHandler<M> trait implemented
- [ ] Comprehensive broker error types
- [ ] ActorRegistry with address resolution
- [ ] Actor pool management with load balancing
- [ ] InMemoryMessageBroker<M> implementation
- [ ] Request-reply pattern with timeout
- [ ] Message delivery with retry logic
- [ ] Metrics collection system
- [ ] All unit tests passing with >95% coverage
- [ ] Clean compilation with zero warnings
- [ ] Proper module exports and public API
- [ ] Documentation with usage examples
- [ ] Architecture compliance verified