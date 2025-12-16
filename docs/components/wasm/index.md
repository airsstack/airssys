# WASM Component Framework

WebAssembly Component Framework for pluggable systems with actor-based runtime integration.

**Status**: ⏳ In Development (Phase 6 - Testing & Validation)  
**Version**: 0.1.0  
**Repository**: `airssys-wasm/`

## Overview

The WASM Component Framework provides a runtime deployment model for WebAssembly components, inspired by smart contract patterns. It enables hot deployment, zero-downtime updates, and secure component isolation with capability-based security.

## Key Features

- **ComponentActor Pattern**: Dual-trait pattern combining WASM lifecycle with actor concurrency
- **Hot Deployment**: Zero-downtime component updates
- **O(1) Registry**: 36ns component lookup (measured)
- **High Throughput**: 6.12 million messages/sec
- **Supervisor Integration**: Automatic restart and recovery
- **Performance Validated**: 945 tests + 28 benchmarks

## Architecture

- **ComponentActor**: Dual-trait pattern (Child + Actor)
- **ActorSystem**: Component spawning and lifecycle
- **Registry**: O(1) component lookup
- **MessageRouter**: Low-latency message routing (~1µs)
- **SupervisorNode**: Fault tolerance with exponential backoff

## Quick Start

See [Your First ComponentActor](tutorials/your-first-component-actor.md) tutorial.

## Documentation Sections

### [API Reference](api/)
- [ComponentActor API](api/component-actor.md)
- [Lifecycle Hooks](api/lifecycle-hooks.md)

### [Tutorials](tutorials/)
- [Your First ComponentActor](tutorials/your-first-component-actor.md)

### Guides (Coming in Checkpoint 2)
- Request-Response Patterns
- Pub-Sub Broadcasting
- Supervision and Recovery

### Reference (Coming in Checkpoint 2)
- Message Routing
- Performance Characteristics

### Explanation (Coming in Checkpoint 2)
- State Management Patterns
- Dual-Trait Design

## Performance Characteristics

Based on Task 6.2 benchmarks (28 benchmarks, criterion framework):

| Operation | Performance | Source |
|-----------|-------------|--------|
| Component construction | 286ns | actor_lifecycle_benchmarks |
| Full lifecycle | 1.49µs | actor_lifecycle_benchmarks |
| Registry lookup | 36ns O(1) | scalability_benchmarks |
| Message routing | 1.05µs | messaging_benchmarks |
| Request-response | 3.18µs | messaging_benchmarks |
| Message throughput | 6.12M msg/sec | messaging_benchmarks |

## Development Status

### Phase 6: Testing & Validation ⏳
- ✅ Task 6.1: Integration Test Suite (945 tests)
- ✅ Task 6.2: Performance Validation (28 benchmarks)
- ⏳ Task 6.3: Documentation & Examples (Checkpoint 1 complete)

### Completed Phases
- ✅ Phase 1-3: ComponentActor Foundation
- ✅ Phase 4-5: ActorSystem & Supervisor Integration

## Examples

See [examples/](../../examples/) directory:
- `basic_component_actor.rs` - Minimal ComponentActor
- `stateful_component.rs` - State management patterns

More examples coming in Checkpoint 2.

## Integration with AirsSys

WASM components integrate with:
- **airssys-rt**: Actor runtime for component hosting
- **airssys-osl**: Secure system operations (via WASI)

## Contributing

See [Contributing Guide](../../contributing.md).

## License

Dual-licensed under MIT or Apache 2.0.
