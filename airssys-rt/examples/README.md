# AirsSys RT Examples

Comprehensive examples demonstrating the Erlang-Actor runtime system features and patterns.

## Learning Path

Follow this recommended progression to build understanding from fundamentals to advanced patterns:

### 📚 Level 1: Fundamentals

Start here to understand core Actor model concepts and basic usage.

#### **actor_basic.rs** - Actor Trait Fundamentals ⭐ START HERE
- **What You'll Learn**: Actor trait implementation, lifecycle hooks, error handling
- **Key Concepts**: Message handling, pre_start/post_stop, ErrorAction decisions
- **Run**: `cargo run --example actor_basic`
- **Lines**: ~240 lines
- **Topics**: Actor trait, ActorContext, ActorLifecycle, Message trait

#### **actor_lifecycle.rs** - State Machine and Restarts
- **What You'll Learn**: Lifecycle state transitions, restart strategies, terminal states
- **Key Concepts**: State machine (Created → Starting → Running → Stopping → Stopped/Failed)
- **Run**: `cargo run --example actor_lifecycle`
- **Lines**: ~250 lines
- **Topics**: ActorLifecycle, ActorState, restart count tracking, failure recovery

### 📚 Level 2: Supervision Basics

Learn how supervisors manage actor lifecycles and handle failures.

#### **supervisor_basic.rs** - Child Lifecycle Management ⭐ RECOMMENDED
- **What You'll Learn**: Supervisor creation, child specs, restart policies, graceful shutdown
- **Key Concepts**: OneForOne strategy, RestartPolicy (Permanent/Transient/Temporary)
- **Run**: `cargo run --example supervisor_basic`
- **Lines**: ~340 lines
- **Topics**: SupervisorNode, ChildSpec, restart policies, Child trait

#### **supervisor_strategies.rs** - Strategy Comparison
- **What You'll Learn**: OneForOne vs OneForAll vs RestForOne supervision strategies
- **Key Concepts**: Failure isolation, cascading restarts, dependency chains
- **Run**: `cargo run --example supervisor_strategies`
- **Lines**: ~280 lines
- **Topics**: Supervision strategies, failure propagation, restart behavior

### 📚 Level 3: Advanced Supervision

Master fluent builder API and complex supervision patterns.

#### **supervisor_builder_phase1.rs** - Fluent Builder API
- **What You'll Learn**: Builder pattern for supervisor configuration
- **Key Concepts**: Type-safe configuration, method chaining, default values
- **Run**: `cargo run --example supervisor_builder_phase1`
- **Lines**: ~220 lines
- **Topics**: Builder pattern, type safety, ergonomic APIs

#### **supervisor_builder_phase2.rs** - Advanced Builder Patterns
- **What You'll Learn**: Complex builder scenarios, custom strategies, validation
- **Key Concepts**: Validation chains, custom configurations, error handling
- **Run**: `cargo run --example supervisor_builder_phase2`
- **Lines**: ~250 lines
- **Topics**: Advanced builders, validation, custom strategies

### 📚 Level 4: Monitoring and Health

Understand health checks, metrics, and monitoring integration.

#### **monitoring_basic.rs** - Monitoring Integration
- **What You'll Learn**: Health checks, metrics collection, monitoring configuration
- **Key Concepts**: ChildHealth, monitoring events, health check intervals
- **Run**: `cargo run --example monitoring_basic`
- **Lines**: ~200 lines
- **Topics**: Monitoring, health checks, metrics, InMemoryMonitor

#### **monitoring_supervisor.rs** - Supervisor Monitoring
- **What You'll Learn**: Supervisor-level monitoring, aggregated health, tree monitoring
- **Key Concepts**: Supervisor health aggregation, monitoring propagation
- **Run**: `cargo run --example monitoring_supervisor`
- **Lines**: ~230 lines
- **Topics**: Supervisor monitoring, health aggregation, tree traversal

#### **supervisor_automatic_health.rs** - Automatic Health Checks
- **What You'll Learn**: Automated health check scheduling, unhealthy child handling
- **Key Concepts**: Periodic health checks, automatic restart triggers
- **Run**: `cargo run --example supervisor_automatic_health`
- **Lines**: ~210 lines
- **Topics**: Automatic health checks, scheduling, health-based restarts

### 📚 Level 5: Message Passing

Master communication patterns between actors.

#### **message_patterns.rs** - Message Passing Patterns
- **What You'll Learn**: Small messages, Arc<T> sharing, batching, optimization
- **Key Concepts**: Message design, performance patterns, memory management
- **Run**: `cargo run --example message_patterns`
- **Lines**: ~390 lines
- **Topics**: Message patterns, performance, Arc<T>, batching, serialization

#### **actor_patterns.rs** - Actor Communication Patterns
- **What You'll Learn**: Request/reply, pub/sub, actor collaboration patterns
- **Key Concepts**: Synchronous messaging, event broadcasting, actor coordination
- **Run**: `cargo run --example actor_patterns`
- **Lines**: ~310 lines
- **Topics**: Communication patterns, request/reply, pub/sub, coordination

### 📚 Level 6: Real-World Use Cases

Apply learned concepts to practical production scenarios.

#### **worker_pool.rs** - Load-Balanced Worker Pool ⭐ PRODUCTION PATTERN
- **What You'll Learn**: Worker pool pattern, load balancing, request/reply, failure recovery
- **Key Concepts**: OneForOne supervision for isolation, round-robin distribution
- **Run**: `cargo run --example worker_pool`
- **Lines**: ~400 lines
- **Topics**: Worker pools, load balancing, request/reply, parallel processing
- **Use Cases**: Background job processing, API request handling, batch processing

#### **event_pipeline.rs** - Event Processing Pipeline ⭐ PRODUCTION PATTERN
- **What You'll Learn**: Sequential pipeline, RestForOne supervision, backpressure handling
- **Key Concepts**: Pipeline architecture (Ingest → Transform → Output), cascading restarts
- **Run**: `cargo run --example event_pipeline`
- **Lines**: ~600 lines
- **Topics**: Event pipelines, RestForOne, backpressure, ordered processing
- **Use Cases**: ETL pipelines, stream processing, event-driven architectures

#### **getting_started.rs** - Quick Start Template
- **What You'll Learn**: Minimal working example to get started quickly
- **Key Concepts**: Basic actor setup, simple message handling
- **Run**: `cargo run --example getting_started`
- **Lines**: ~120 lines
- **Topics**: Quick start, minimal example, template code

## Topics Index

### By Concept

**Actor Fundamentals**
- `actor_basic.rs` - Core Actor trait implementation
- `actor_lifecycle.rs` - State machine and lifecycle
- `getting_started.rs` - Quick start template

**Supervision**
- `supervisor_basic.rs` - Basic supervisor usage
- `supervisor_strategies.rs` - Strategy comparison
- `supervisor_builder_phase1.rs` - Builder API basics
- `supervisor_builder_phase2.rs` - Advanced builder patterns
- `supervisor_automatic_health.rs` - Health-based supervision

**Monitoring**
- `monitoring_basic.rs` - Basic monitoring
- `monitoring_supervisor.rs` - Supervisor monitoring
- `supervisor_automatic_health.rs` - Automatic health checks

**Messaging**
- `message_patterns.rs` - Message design patterns
- `actor_patterns.rs` - Actor communication patterns
- `worker_pool.rs` - Request/reply in production

**Production Patterns**
- `worker_pool.rs` - Load-balanced worker pool (OneForOne)
- `event_pipeline.rs` - Event processing pipeline (RestForOne)

### By Supervision Strategy

**OneForOne** (Isolated Restarts)
- `supervisor_basic.rs`
- `worker_pool.rs` ⭐

**OneForAll** (Restart All Children)
- `supervisor_strategies.rs`

**RestForOne** (Cascading Restarts)
- `supervisor_strategies.rs`
- `event_pipeline.rs` ⭐

### By Use Case

**Background Job Processing**
- `worker_pool.rs` - Parallel task execution with load balancing

**Event Processing**
- `event_pipeline.rs` - Sequential pipeline with ordered processing

**Service Coordination**
- `supervisor_strategies.rs` - Dependent service management

**Health Monitoring**
- `monitoring_basic.rs` - Basic health checks
- `monitoring_supervisor.rs` - Aggregated monitoring
- `supervisor_automatic_health.rs` - Automated health management

## Running Examples

### Run Individual Examples
```bash
# Basic examples
cargo run --example actor_basic
cargo run --example supervisor_basic

# Advanced examples
cargo run --example worker_pool
cargo run --example event_pipeline

# Run specific example
cargo run --example <example_name>
```

### Build All Examples
```bash
# Build all examples (check for compilation errors)
cargo build --examples

# Build with optimizations (for performance testing)
cargo build --examples --release
```

### Run Examples with Logging
```bash
# Set RUST_LOG for detailed output
RUST_LOG=debug cargo run --example worker_pool

# Specific module logging
RUST_LOG=airssys_rt::supervisor=trace cargo run --example supervisor_basic
```

## Example Conventions

All examples follow these conventions for consistency:

### Structure
- **File-level documentation**: Comprehensive overview with "What You'll Learn" and "Key Concepts"
- **Run instructions**: Explicit `cargo run --example` command
- **Expected output**: Sample output showing typical execution
- **See also**: Cross-references to related examples and documentation

### Code Organization
- **Section headers**: Clear separation with separator lines (`// ============`)
- **Inline comments**: Explain concepts, use cases, and decisions
- **Type annotations**: Clear type information for learning
- **Error handling**: Proper `Result<>` usage with contextual errors

### Output Format
```text
=== Example Name ===

Step 1: Description...
✅ Success message

Step 2: Description...
  • Detail 1
  • Detail 2

=== Example Complete! ===

Key Learnings:
  • Learning point 1
  • Learning point 2
```

## Documentation

For comprehensive guides and API documentation, see:

- **User Guides**: `docs/src/guides/`
  - [Actor Development](../docs/src/guides/actor-development.md)
  - [Supervisor Patterns](../docs/src/guides/supervisor-patterns.md)
  - [Message Passing](../docs/src/guides/message-passing.md)
- **API Reference**: `docs/src/reference/`
- **Generated Docs**: Run `cargo doc --open`

## Testing Examples

Verify all examples work correctly:

```bash
# Compile all examples
cargo build --examples

# Run all examples (bash)
for example in examples/*.rs; do
    name=$(basename "$example" .rs)
    echo "Running $name..."
    cargo run --example "$name" || echo "❌ $name failed"
done

# Run all examples (zsh)
for example in examples/*.rs; do
    name=${example:t:r}
    echo "Running $name..."
    cargo run --example "$name" || echo "❌ $name failed"
done
```

## Contributing Examples

When adding new examples:

1. Follow the documentation structure conventions above
2. Include comprehensive inline comments
3. Provide run instructions and expected output
4. Add cross-references to related examples and docs
5. Update this README.md with proper categorization
6. Ensure zero compiler warnings
7. Test the example runs correctly

## Quick Reference

| Example | Level | Complexity | Topics | Lines |
|---------|-------|------------|--------|-------|
| `getting_started.rs` | 1 | ⭐ | Quick start | ~120 |
| `actor_basic.rs` | 1 | ⭐ | Actor trait | ~240 |
| `actor_lifecycle.rs` | 1 | ⭐⭐ | Lifecycle | ~250 |
| `supervisor_basic.rs` | 2 | ⭐⭐ | Supervision | ~340 |
| `supervisor_strategies.rs` | 2 | ⭐⭐⭐ | Strategies | ~280 |
| `supervisor_builder_phase1.rs` | 3 | ⭐⭐ | Builders | ~220 |
| `supervisor_builder_phase2.rs` | 3 | ⭐⭐⭐ | Advanced builders | ~250 |
| `monitoring_basic.rs` | 4 | ⭐⭐ | Monitoring | ~200 |
| `monitoring_supervisor.rs` | 4 | ⭐⭐⭐ | Supervisor monitoring | ~230 |
| `supervisor_automatic_health.rs` | 4 | ⭐⭐⭐ | Auto health checks | ~210 |
| `message_patterns.rs` | 5 | ⭐⭐⭐ | Messaging | ~390 |
| `actor_patterns.rs` | 5 | ⭐⭐⭐ | Communication | ~310 |
| `worker_pool.rs` | 6 | ⭐⭐⭐⭐ | Worker pool | ~400 |
| `event_pipeline.rs` | 6 | ⭐⭐⭐⭐⭐ | Event processing | ~600 |

Legend: ⭐ = Beginner, ⭐⭐ = Intermediate, ⭐⭐⭐ = Advanced, ⭐⭐⭐⭐ = Production, ⭐⭐⭐⭐⭐ = Complex

## Support

- **Issues**: Report bugs or request new examples at GitHub Issues
- **Discussions**: Ask questions and share use cases in Discussions
- **Documentation**: Full user guides in `docs/src/guides/`

---

**Last Updated**: October 16, 2025  
**Examples**: 15 total (13 existing + 2 new)  
**Documentation Status**: Phase 3 Day 5 Complete
