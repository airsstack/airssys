# Reference

**Technical specifications and authoritative information about the AirsSys-RT system.**

Reference documentation provides neutral, objective descriptions of the system's machinery. Each reference page:
- Describes and only describes (no instruction or explanation)
- Adopts standard patterns for consistency
- Respects the structure of the code itself
- States facts, lists features, provides warnings
- Serves as authoritative technical documentation

## API Reference

Detailed specifications for all public APIs, organized by module:

- **[Core Types](./api/core-types.md)** - Fundamental types (ActorId, ActorRef, ActorContext, ActorSystem)
- **[Actor Traits](./api/actor-traits.md)** - Actor trait and lifecycle methods
- **[Message Types](./api/message-types.md)** - Message trait, Envelope, MessageMetadata
- **[Supervisor API](./api/supervisor-api.md)** - Supervisor trait, configuration, builders

## Performance Reference

Performance characteristics and benchmarking data:

- **Baseline Performance**: Actor spawn (625ns), Message latency (737ns), Throughput (4.7M msg/sec)
- **Scaling Characteristics**: Linear scaling with 6% overhead (1→50 actors)
- **Resource Usage**: Memory footprint, CPU utilization patterns

> **Note**: Full performance guide will be available at `reference/performance.md`

## Troubleshooting

Common errors, debugging techniques, and problem resolution:

- Compilation errors and solutions
- Runtime error diagnosis
- Performance issue investigation  
- Common pitfalls and how to avoid them

> **Note**: Full troubleshooting guide will be available at `reference/troubleshooting.md`

## How to Use Reference Documentation

### Looking for specific API information?
→ Navigate to the relevant API reference page
→ Use browser search (Ctrl+F / Cmd+F) to find specific types or methods

### Need to understand how something works?
→ Reference tells you **what** it is
→ For **why** and **how to use**, see [Explanation](./explanation.md) and [How-To Guides](./guides.md)

### Comparing options or features?
→ Reference provides neutral descriptions
→ For recommendations and tradeoffs, see [Explanation](./explanation.md)

## Rustdoc API Documentation

For automatically generated API documentation, run:

```bash
cargo doc --open --no-deps
```

This provides:
- Complete type signatures
- Method documentation  
- Implementation details
- Source code links

## Related Resources

- **How-To Guides**: Task-oriented instructions at [Guides](./guides.md)
- **Tutorials**: Learning-oriented exercises at [Tutorials](./tutorials.md)
- **Explanation**: Understanding-oriented content at [Explanation](./explanation.md)
- **Examples**: Working code in `examples/` directory with `examples/README.md` catalog

## Diátaxis Framework

This section follows the **Reference** category of the [Diátaxis framework](https://diataxis.fr/):
- **Purpose**: Technical descriptions of the machinery and how to operate it
- **User Need**: "I need accurate, authoritative information about this"
- **Focus**: Information-oriented, describes the product
- **Approach**: Austere, uncompromising, wholly authoritative
