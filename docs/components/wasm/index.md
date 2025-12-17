# ComponentActor System

**Welcome to the ComponentActor documentation!**

ComponentActor is a production-ready framework for building fault-tolerant, scalable component-based systems in Rust. It combines lifecycle management, message routing, supervision, and state management into a cohesive actor-based architecture.

**Status**: ✅ Production Ready (Phase 6 - Validated)  
**Version**: 0.1.0  
**Repository**: `airssys-wasm/`

## What is ComponentActor?

ComponentActor implements a **dual-trait pattern** that separates component lifecycle from message handling:

- **Child trait**: Manages lifecycle (`pre_start`, `post_start`, `pre_stop`, `post_stop`)
- **Actor trait**: Handles asynchronous message processing

This separation enables:
- Clear lifecycle boundaries
- Independent testing of lifecycle vs messaging
- Flexible composition patterns
- Supervisor integration

## Key Features

### Lifecycle Management
Components have well-defined lifecycle hooks:
```rust
fn pre_start(&mut self);   // Initialize
fn post_start(&mut self);  // Ready
fn pre_stop(&mut self);    // Cleanup
fn post_stop(&mut self);   // Stopped
```

### Supervision & Recovery
Automatic crash recovery with configurable strategies:
- Restart policies (Permanent, Transient, Temporary)
- Exponential backoff
- Health monitoring
- Failure isolation

### Message Routing
Efficient message delivery between components:
- O(1) registry lookup (36ns, Task 6.2 `scalability_benchmarks.rs`)
- Request-response pattern (3.18µs, Task 6.2 `messaging_benchmarks.rs`)
- Pub-sub broadcasting (85.2µs fanout to 100, Task 6.2 `messaging_benchmarks.rs`)

### State Management
Thread-safe state access with Arc<RwLock<T>>:
- Concurrent read access
- Exclusive write access
- 37-39ns access latency (Task 6.2 `actor_lifecycle_benchmarks.rs`)

## Performance Highlights

Measured in Task 6.2 (Phase 6 validation):

| Metric | Value | Source |
|--------|-------|--------|
| Component spawn | 286ns | `actor_lifecycle_benchmarks.rs` |
| Message throughput | 6.12M msg/sec | `messaging_benchmarks.rs` |
| Registry lookup | 36ns O(1) | `scalability_benchmarks.rs` |
| Request-response | 3.18µs | `messaging_benchmarks.rs` |
| Full lifecycle | 1.49µs | `actor_lifecycle_benchmarks.rs` |

**Test conditions**: macOS M1, 100 samples, 95% confidence interval

**Source**: Task 6.2 Completion Report (`.memory-bank/sub-projects/airssys-wasm/tasks/task-004-phase-6-task-6.2-completion-report.md`)

## When to Use ComponentActor

**Ideal for:**

- ✅ Pluggable systems (WASM components, plugins)
- ✅ Multi-component architectures (microservices, actors)
- ✅ Fault-tolerant systems (automatic recovery)
- ✅ High-throughput systems (6M+ msg/sec)

**Consider alternatives for:**

- ❌ Simple single-process applications
- ❌ Systems without isolation requirements
- ❌ Stateless request-response services

## Quick Start

Get started with your first ComponentActor:

1. **Tutorial**: [Your First ComponentActor](./tutorials/your-first-component-actor.md) (1 hour)
2. **Stateful Components**: [Building a Stateful Component](./tutorials/stateful-component-tutorial.md) (1.5 hours)
3. **Communication**: [Request-Response Pattern](./guides/request-response-pattern.md) (30 min)

## Documentation Structure

This documentation follows the [Diátaxis framework](https://diataxis.fr/):

### 📚 Tutorials (Learning-Oriented)
Step-by-step guides to learn by building:
- [Your First ComponentActor](./tutorials/your-first-component-actor.md)
- [Building a Stateful Component](./tutorials/stateful-component-tutorial.md)

### 📖 How-To Guides (Task-Oriented)
Solutions to specific problems:
- [Request-Response Pattern](./guides/request-response-pattern.md)
- [Pub-Sub Broadcasting](./guides/pubsub-broadcasting.md)
- [Supervision and Recovery](./guides/supervision-and-recovery.md)
- [Component Composition](./guides/component-composition.md)
- [Production Deployment](./guides/production-deployment.md)
- [Best Practices](./guides/best-practices.md)
- [Troubleshooting](./guides/troubleshooting.md)

### 📋 Reference (Information-Oriented)
Technical specifications:
- [ComponentActor API](./api/component-actor.md)
- [Lifecycle Hooks](./api/lifecycle-hooks.md)
- [Message Routing](./reference/message-routing.md)
- [Performance Characteristics](./reference/performance-characteristics.md)

### 💡 Explanation (Understanding-Oriented)
Context and rationale:
- [Dual-Trait Design](./explanation/dual-trait-design.md)
- [State Management Patterns](./explanation/state-management-patterns.md)
- [Supervision Architecture](./explanation/supervision-architecture.md)
- [Production Readiness](./explanation/production-readiness.md)

## Architecture Overview

```
┌─────────────────────────────────────────────┐
│         ComponentActor (Your Code)          │
│  ┌──────────────┐      ┌─────────────────┐ │
│  │ Child Trait  │      │  Actor Trait    │ │
│  │ (Lifecycle)  │      │ (Messages)      │ │
│  └──────────────┘      └─────────────────┘ │
└─────────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────┐
│          ActorSystem Integration            │
│  • Spawning     • Supervision               │
│  • Messaging    • Registry                  │
└─────────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────┐
│            airssys-rt Runtime               │
│  • Actor model  • Message broker            │
│  • Supervision  • Scheduling                │
└─────────────────────────────────────────────┘
```

See [Architecture](./architecture.md) for complete details.

## Development Status

### Phase 6: Testing & Validation ✅ COMPLETE
- ✅ Task 6.1: Integration Test Suite (945 tests, 100% pass)
- ✅ Task 6.2: Performance Validation (28 benchmarks, all targets exceeded)
- ✅ Task 6.3: Documentation & Examples (20 docs + 6 examples)

**Quality Score**: 9.5/10 across all dimensions

### Completed Phases
- ✅ Phase 1-3: ComponentActor Foundation
- ✅ Phase 4-5: ActorSystem & Supervisor Integration

## Examples

Working examples demonstrating core patterns:

| Example | Purpose | File |
|---------|---------|------|
| Basic ComponentActor | Minimal lifecycle and messages | `basic_component_actor.rs` |
| Stateful Component | State management patterns | `stateful_component.rs` |
| Request-Response | Correlation-based communication | `request_response_pattern.rs` |
| Pub-Sub Broadcasting | Topic-based messaging | `pubsub_component.rs` |
| Supervised Component | Crash recovery patterns | `supervised_component.rs` |
| Component Composition | Multi-component orchestration | `component_composition.rs` |

See [examples/](../../examples/) directory.

## Integration with AirsSys

ComponentActor integrates with:
- **airssys-rt**: Actor runtime providing supervision, messaging, and scheduling
- **airssys-osl**: Secure system operations for file system, network, and process management

## Next Steps

1. **Learn**: Start with [Your First ComponentActor](./tutorials/your-first-component-actor.md)
2. **Explore**: Try the [examples](../../examples/) (6 working examples)
3. **Deploy**: Read [Production Deployment](./guides/production-deployment.md)
4. **Optimize**: Review [Best Practices](./guides/best-practices.md)

## Getting Help

- **Documentation**: You're reading it!
- **Examples**: See `airssys-wasm/examples/`
- **Tests**: See `airssys-wasm/tests/` for integration patterns
- **Troubleshooting**: See [Troubleshooting Guide](./guides/troubleshooting.md)

## Performance Validation

All performance claims are validated with benchmarks from Task 6.2:

- **28 benchmarks** across 3 categories (lifecycle, messaging, scalability)
- **Criterion framework** with 100 samples per benchmark at 95% confidence
- **Variance < 5%** for 96% of benchmarks
- **All targets exceeded** by 16-26,500x

See [Performance Characteristics](./reference/performance-characteristics.md) for complete data.

## Contributing

See [Contributing Guide](../../contributing.md).

## License

Dual-licensed under MIT or Apache 2.0.

---

**Ready to build your first component?** → [Start the tutorial](./tutorials/your-first-component-actor.md)
