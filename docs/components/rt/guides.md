# How-To Guides

**Task-oriented guides that show you how to accomplish specific goals.**

How-To Guides provide practical directions for solving real-world problems. Each guide:
- Addresses a specific task or problem
- Provides executable instructions
- Assumes you know what you want to achieve
- Adapts to real-world complexity and use-cases
- Focuses on practical usability over completeness

## Available Guides

### Actor Development

#### [Actor Development Guide](./guides/actor-development.md)
Comprehensive guide to developing actors with best practices, lifecycle management, error handling, and testing strategies.

**What you'll learn:**

- Actor trait implementation patterns
- Lifecycle hook usage (pre_start, post_stop, on_error)
- Message handling best practices
- Error handling and recovery
- Testing actors effectively

### Supervision

#### [Supervisor Patterns Guide](./guides/supervisor-patterns.md)
Learn how to design and implement fault-tolerant supervision trees using different supervision strategies.

**What you'll learn:**

- Supervision strategy selection (OneForOne, OneForAll, RestForOne)
- Restart policy configuration (Permanent, Transient, Temporary)
- Supervisor tree design patterns
- Child specification setup
- Monitoring integration

### Messaging

#### [Message Passing Guide](./guides/message-passing.md)
Master message design, performance optimization, and communication patterns between actors.

**What you'll learn:**

- Message type design patterns
- Performance optimization (small messages, Arc<T>, batching)
- Request/reply patterns
- Pub/sub messaging via MessageBroker
- Backpressure handling

## How to Use These Guides

### If you want to...

**Build a new actor:**
→ Start with [Actor Development Guide](./guides/actor-development.md)

**Add supervision to your system:**
→ See [Supervisor Patterns Guide](./guides/supervisor-patterns.md)

**Optimize message passing:**
→ Read [Message Passing Guide](./guides/message-passing.md)

**Implement a specific pattern:**
→ Check the examples in `examples/` directory:
- `worker_pool.rs` - Load-balanced worker pool
- `event_pipeline.rs` - Event processing pipeline
- `examples/README.md` - Complete catalog

## Related Resources

- **Tutorials**: If you need to learn fundamentals first, see [Tutorials](./tutorials.md)
- **Examples**: Working code for common patterns in `examples/` directory
- **API Reference**: Detailed API specifications in [Reference](./api.md)
- **Explanation**: Understand the "why" behind patterns in [Explanation](./explanation.md)

## Diátaxis Framework

This section follows the **How-To Guides** category of the [Diátaxis framework](https://diataxis.fr/):
- **Purpose**: Directions to guide readers through problems to achieve specific goals
- **User Need**: "I need to accomplish this specific task"
- **Focus**: Solving real-world problems for competent users
- **Approach**: Practical, executable instructions adaptable to use-cases
