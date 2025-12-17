# Understanding the Builder Pattern

This document explains the builder pattern usage in AirsSys RT, the rationale for adopting it, and how it improves API ergonomics and type safety.

## Table of Contents

- [What is the Builder Pattern?](#what-is-the-builder-pattern)
- [Why Builder Pattern in AirsSys RT?](#why-builder-pattern-in-airssys-rt)
- [Builder Pattern Implementation](#builder-pattern-implementation)
- [Design Decisions](#design-decisions)
- [Migration from Direct Construction](#migration-from-direct-construction)
- [Best Practices](#best-practices)

---

## What is the Builder Pattern?

The **Builder Pattern** is a creational design pattern that provides a fluent API for constructing complex objects step-by-step. Instead of using constructors with many parameters, builders allow setting each property individually with method chaining.

### Traditional Construction (Without Builder)

```rust
// Constructor with many parameters
let supervisor = Supervisor::new(
    RestartStrategy::OneForOne,
    10,  // max_restarts
    Duration::from_secs(60),  // restart_window
    vec![
        ChildSpec::new("worker-1", ...),
        ChildSpec::new("worker-2", ...),
    ],
);

// Problems:
// 1. Parameter order easily confused
// 2. No compile-time validation of required fields
// 3. Difficult to add optional parameters
// 4. Hard to read (what do 10 and 60 mean?)
```

### Builder Pattern Construction

```rust
// Fluent builder API
let supervisor = SupervisorBuilder::new()
    .with_strategy(RestartStrategy::OneForOne)  // Clear intent
    .with_max_restarts(10)                       // Named parameter
    .with_restart_window(Duration::from_secs(60)) // Self-documenting
    .with_child(
        ChildSpec::new("worker-1")
            .with_actor::<WorkerActor>()
    )
    .with_child(
        ChildSpec::new("worker-2")
            .with_actor::<WorkerActor>()
    )
    .build()  // Validates and constructs
    .await?;

// Benefits:
// 1. Parameter names clear (self-documenting)
// 2. Compile-time required field checking
// 3. Easy to add optional parameters
// 4. Readable, fluent API
```

---

## Why Builder Pattern in AirsSys RT?

### Problem: Complex Configuration

AirsSys RT components (supervisors, actors, mailboxes) have many configuration options:

**Supervisor Configuration:**

- Restart strategy (OneForOne, OneForAll, RestForOne)
- Max restarts and restart window
- Child specifications (multiple children)
- Health monitoring configuration
- Shutdown policies

**Actor Configuration:**

- Mailbox type (Bounded, Unbounded)
- Mailbox capacity (if bounded)
- Backpressure strategy (Block, Drop, Error)
- Initial state
- Lifecycle hooks

**Challenge:** How to provide flexible, ergonomic configuration without sacrificing type safety?

### Solution: Builder Pattern

**Decision:** Migrate all complex constructors to builder pattern.

**Rationale:**

1. **Ergonomics:** Method chaining provides fluent, readable API
2. **Type Safety:** Compile-time checking of required fields
3. **Flexibility:** Easy to add optional configuration without breaking changes
4. **Discoverability:** IDEs autocomplete available configuration methods
5. **Validation:** `build()` method validates configuration before construction

**Implementation Status:**

- ✅ **Completed:** Supervisor builder pattern
- ✅ **Completed:** Batch supervisor operations
- ✅ **Completed:** Migration guide and examples
- 🔄 **Future:** Actor builder, ChildSpec builder, System builder

---

## Builder Pattern Implementation

### Supervisor Builder

**API Design:**

```rust
pub struct SupervisorBuilder {
    strategy: Option<RestartStrategy>,
    max_restarts: Option<u32>,
    restart_window: Option<Duration>,
    children: Vec<ChildSpec>,
    health_monitoring: bool,
}

impl SupervisorBuilder {
    pub fn new() -> Self {
        Self {
            strategy: None,
            max_restarts: Some(10),  // Sensible default
            restart_window: Some(Duration::from_secs(60)),  // Sensible default
            children: Vec::new(),
            health_monitoring: false,
        }
    }
    
    pub fn with_strategy(mut self, strategy: RestartStrategy) -> Self {
        self.strategy = Some(strategy);
        self  // Return self for method chaining
    }
    
    pub fn with_max_restarts(mut self, max: u32) -> Self {
        self.max_restarts = Some(max);
        self
    }
    
    pub fn with_restart_window(mut self, window: Duration) -> Self {
        self.restart_window = Some(window);
        self
    }
    
    pub fn with_child(mut self, child: ChildSpec) -> Self {
        self.children.push(child);
        self
    }
    
    pub async fn build(self) -> Result<Supervisor, BuildError> {
        // Validate required fields
        let strategy = self.strategy
            .ok_or(BuildError::MissingStrategy)?;
        
        // Validate configuration
        if self.children.is_empty() {
            return Err(BuildError::NoChildren);
        }
        
        // Construct supervisor
        Ok(Supervisor {
            strategy,
            max_restarts: self.max_restarts.unwrap(),
            restart_window: self.restart_window.unwrap(),
            children: self.children,
            health_monitoring: self.health_monitoring,
        })
    }
}
```

**Key Design Elements:**

1. **Builder struct with `Option<T>` fields** - Track which fields are set
2. **`new()` constructor** - Initialize with defaults
3. **`with_*()` methods** - Set individual fields, return `self` for chaining
4. **`build()` method** - Validate and construct final object

### ChildSpec Builder (Future)

**Planned API:**

```rust
let child = ChildSpec::builder("worker-1")
    .with_actor::<WorkerActor>()
    .with_restart_policy(RestartPolicy::Permanent)
    .with_shutdown_timeout(Duration::from_secs(5))
    .build()?;
```

**Current API (Direct Construction):**

```rust
let child = ChildSpec::new("worker-1")
    .with_actor::<WorkerActor>()
    .with_restart_policy(RestartPolicy::Permanent);
```

**Migration:** Current API already uses method chaining, formal builder provides validation.

---

## Design Decisions

### Decision: Separate Builder and Product

**Choice:** Builder (`SupervisorBuilder`) and product (`Supervisor`) are separate types.

**Alternative:** Mutable methods on `Supervisor` itself.

```rust
// Alternative: In-place mutation (NOT chosen)
let mut supervisor = Supervisor::new();
supervisor.set_strategy(RestartStrategy::OneForOne);
supervisor.add_child(child);
```

**Rationale for Separation:**

1. **Immutability:** Built object is immutable (configuration frozen after `build()`)
2. **Validation:** `build()` validates before construction (invalid config never creates object)
3. **Type State Pattern (Future):** Builder can use type state to enforce required fields at compile time

**Tradeoff:** More code (two types instead of one) vs. Better encapsulation and validation.

### Decision: Sensible Defaults

**Choice:** Provide sensible defaults for optional configuration.

```rust
impl SupervisorBuilder {
    pub fn new() -> Self {
        Self {
            max_restarts: Some(10),  // Default: 10 restarts
            restart_window: Some(Duration::from_secs(60)),  // Default: 60 seconds
            strategy: None,  // Required field (no default)
            children: Vec::new(),  // Required field (no default)
            health_monitoring: false,  // Default: disabled
        }
    }
}
```

**Rationale:**

- **Required fields:** No default (force user to specify)
- **Optional fields:** Sensible defaults (minimize boilerplate)

**Example:**

```rust
// Minimal configuration (uses defaults for optional fields)
let supervisor = SupervisorBuilder::new()
    .with_strategy(RestartStrategy::OneForOne)  // Required
    .with_child(child)  // Required
    .build()
    .await?;
// max_restarts, restart_window use defaults
```

**Tradeoff:** Hidden defaults may surprise users vs. Reduced boilerplate.

### Decision: Async `build()` Method

**Choice:** `build()` is `async fn` returning `Future`.

```rust
pub async fn build(self) -> Result<Supervisor, BuildError> { ... }
```

**Rationale:**

- Supervisor construction may involve async operations (spawning actors, initialization)
- Consistent with Rust async ecosystem (Tokio, async-trait)

**Alternative:** Synchronous `build()` + separate `start()` method.

```rust
// Alternative: Two-phase construction (NOT chosen)
let supervisor = builder.build()?;  // Sync
supervisor.start().await?;  // Async
```

**Tradeoff:** Simpler API (one method) vs. Two-phase initialization flexibility.

### Decision: Compile-Time vs. Runtime Validation

**Current:** Runtime validation in `build()` method.

```rust
pub async fn build(self) -> Result<Supervisor, BuildError> {
    // Runtime check
    let strategy = self.strategy.ok_or(BuildError::MissingStrategy)?;
    
    if self.children.is_empty() {
        return Err(BuildError::NoChildren);  // Runtime error
    }
    
    Ok(...)
}
```

**Future:** Type-state pattern for compile-time validation.

```rust
// Type-state pattern (future enhancement)
struct SupervisorBuilder<State> {
    strategy: RestartStrategy,  // Always set in this state
    ...
}

impl SupervisorBuilder<NoStrategy> {
    pub fn with_strategy(self, strategy: RestartStrategy) -> SupervisorBuilder<HasStrategy> {
        SupervisorBuilder { strategy, ... }  // Transition to HasStrategy state
    }
}

impl SupervisorBuilder<HasStrategy> {
    pub async fn build(self) -> Supervisor {  // No Result, always succeeds
        Supervisor { strategy: self.strategy, ... }
    }
}

// Usage
let supervisor = SupervisorBuilder::new()
    .with_strategy(RestartStrategy::OneForOne)  // Type changes to HasStrategy
    .build()  // Compiles only if strategy is set
    .await;
```

**Tradeoff:** Type-state is complex but provides compile-time guarantees.

**Decision:** Start with runtime validation, migrate to type-state if proven beneficial.

---

## Migration from Direct Construction

### Migration Guide

**Old API (Direct Construction):**

```rust
// Old: Direct constructor with positional arguments
let supervisor = Supervisor::new(
    RestartStrategy::OneForOne,
    10,
    Duration::from_secs(60),
    vec![child1, child2],
);
```

**New API (Builder Pattern):**

```rust
// New: Fluent builder API
let supervisor = SupervisorBuilder::new()
    .with_strategy(RestartStrategy::OneForOne)
    .with_max_restarts(10)
    .with_restart_window(Duration::from_secs(60))
    .with_child(child1)
    .with_child(child2)
    .build()
    .await?;
```

### Migration Strategy

**Phase 1:** Introduce builder alongside existing constructor (both APIs coexist)

```rust
// Old API still works (deprecated)
#[deprecated(note = "Use SupervisorBuilder instead")]
impl Supervisor {
    pub fn new(...) -> Self { ... }
}

// New API available
impl SupervisorBuilder { ... }
```

**Phase 2:** Update examples and documentation to use builder

**Phase 3:** Remove old constructor in next major version (breaking change)

### Backward Compatibility

**Approach:** Deprecate old API but keep functional for one major version.

```rust
#[deprecated(since = "0.2.0", note = "Use SupervisorBuilder instead")]
pub fn new(...) -> Self { ... }
```

**Timeline:**

- v0.1.x: Old API primary, builder experimental
- v0.2.x: Builder primary, old API deprecated
- v0.3.x: Old API removed (breaking change)

---

## Best Practices

### 1. Use Builders for Complex Configuration

**When to Use Builder:**

- ✅ 3+ configuration parameters
- ✅ Optional parameters common
- ✅ Configuration order not obvious
- ✅ Validation required before construction

**When to Use Direct Constructor:**

- ✅ Simple objects (1-2 parameters)
- ✅ All parameters required
- ✅ No validation needed
- ✅ Construction is straightforward

**Example:**

```rust
// Simple object: Direct constructor ok
let actor_ref = ActorRef::new(actor_id);

// Complex object: Builder preferred
let supervisor = SupervisorBuilder::new()
    .with_strategy(RestartStrategy::OneForOne)
    .with_max_restarts(10)
    .with_child(child)
    .build()
    .await?;
```

### 2. Provide Sensible Defaults

**Principle:** Required fields have no default, optional fields have sensible defaults.

```rust
impl SupervisorBuilder {
    pub fn new() -> Self {
        Self {
            // Required (no default) - user must specify
            strategy: None,
            children: Vec::new(),
            
            // Optional (sensible defaults)
            max_restarts: Some(10),
            restart_window: Some(Duration::from_secs(60)),
            health_monitoring: false,
        }
    }
}
```

**Guideline:** Choose defaults that work for 80% of use cases.

### 3. Validate in `build()` Method

**Principle:** Validate configuration before constructing object.

```rust
pub async fn build(self) -> Result<Supervisor, BuildError> {
    // Validate required fields
    let strategy = self.strategy.ok_or(BuildError::MissingStrategy)?;
    
    // Validate constraints
    if self.children.is_empty() {
        return Err(BuildError::NoChildren);
    }
    
    if self.max_restarts.unwrap() == 0 {
        return Err(BuildError::InvalidMaxRestarts);
    }
    
    // All valid, construct
    Ok(Supervisor { ... })
}
```

**Benefits:**

- Invalid configuration detected early (at `build()`, not later)
- Clear error messages (explain what's missing/wrong)
- Prevents invalid objects from being constructed

### 4. Document Builder Usage

**Provide Examples:**

```rust
/// # Examples
///
/// Basic usage:
/// ```
/// # use airssys_rt::supervisor::*;
/// # async fn example() -> Result<(), BuildError> {
/// let supervisor = SupervisorBuilder::new()
///     .with_strategy(RestartStrategy::OneForOne)
///     .with_child(ChildSpec::new("worker").with_actor::<Worker>())
///     .build()
///     .await?;
/// # Ok(())
/// # }
/// ```
impl SupervisorBuilder { ... }
```

**Rationale:** Examples help users discover builder API and understand usage patterns.

---

## Builder Pattern Benefits

### 1. Self-Documenting Code

**Without Builder:**
```rust
// What do these numbers mean?
Supervisor::new(RestartStrategy::OneForOne, 10, 60, children);
```

**With Builder:**
```rust
// Crystal clear intent
SupervisorBuilder::new()
    .with_strategy(RestartStrategy::OneForOne)
    .with_max_restarts(10)  // 10 restarts
    .with_restart_window(Duration::from_secs(60))  // in 60 seconds
    .with_children(children)
    .build()
    .await?;
```

### 2. Easy API Evolution

**Adding Optional Parameter (Without Builder):**

```rust
// Old constructor
pub fn new(strategy: RestartStrategy, children: Vec<ChildSpec>) -> Self { ... }

// New constructor (BREAKING CHANGE!)
pub fn new(
    strategy: RestartStrategy,
    children: Vec<ChildSpec>,
    health_monitoring: bool,  // New parameter breaks all existing code!
) -> Self { ... }
```

**Adding Optional Parameter (With Builder):**

```rust
// Old builder
impl SupervisorBuilder {
    pub fn with_strategy(...) -> Self { ... }
    pub fn with_children(...) -> Self { ... }
    pub fn build(...) -> Result<Supervisor, BuildError> { ... }
}

// New builder (NON-BREAKING!)
impl SupervisorBuilder {
    pub fn with_strategy(...) -> Self { ... }
    pub fn with_children(...) -> Self { ... }
    pub fn with_health_monitoring(mut self, enabled: bool) -> Self {  // New method, old code unaffected!
        self.health_monitoring = enabled;
        self
    }
    pub fn build(...) -> Result<Supervisor, BuildError> { ... }
}
```

### 3. Compile-Time Safety (Future: Type-State)

**Type-state pattern** can enforce required fields at compile time:

```rust
// Won't compile: missing strategy
let supervisor = SupervisorBuilder::new()
    .with_child(child)
    .build()  // ERROR: cannot build without strategy
    .await?;
```

**Currently:** Runtime validation, future enhancement for compile-time validation.

---

## Further Reading

### AirsSys RT Documentation

- [Supervisor Patterns Guide](../guides/supervisor-patterns.md) - Builder usage examples
- [Architecture Overview](../reference/architecture/system-overview.md) - System design

### External Resources

- **Rust API Guidelines:** Builder pattern recommendations
- **Effective Rust:** Builder pattern chapter
- **Type-State Pattern in Rust:** Advanced compile-time validation

---

**Last Updated:** 2025-01-18 (RT-TASK-011 Phase 4 Day 7)
