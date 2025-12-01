# KNOWLEDGE-RT-015: Supervisor Builder Pattern Design & Implementation Guide

**Created:** 2025-10-08  
**Updated:** 2025-10-08  
**Status:** Active  
**Related Tasks:** RT-TASK-013  
**Related ADRs:** ADR-RT-004 (Child Trait Separation)

---

## Overview

This knowledge document provides comprehensive design rationale, implementation guidance, and best practices for the Supervisor Builder Pattern system in airssys-rt. The builder pattern reduces boilerplate and cognitive load while maintaining full backward compatibility with manual `ChildSpec` construction.

---

## Problem Statement

### Current Pain Points

**1. Verbose Child Specification**
```rust
// Current approach - repetitive and verbose
let worker1_start_count = Arc::new(AtomicU32::new(0));
let worker1_stop_count = Arc::new(AtomicU32::new(0));
let worker1_should_fail = Arc::new(AtomicBool::new(false));

let spec1 = ChildSpec {
    id: "worker-1".into(),
    factory: {
        let start_count = Arc::clone(&worker1_start_count);
        let stop_count = Arc::clone(&worker1_stop_count);
        let should_fail = Arc::clone(&worker1_should_fail);
        move || SimpleWorker {
            id: "worker-1".to_string(),
            start_count: Arc::clone(&start_count),
            stop_count: Arc::clone(&stop_count),
            should_fail: Arc::clone(&should_fail),
        }
    },
    restart_policy: RestartPolicy::Permanent,
    shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
    start_timeout: Duration::from_secs(10),
    shutdown_timeout: Duration::from_secs(10),
};

let child1_id = supervisor.start_child(spec1).await?;
```

**2. Repetitive Configuration**
- Same policies repeated across multiple children
- No sensible defaults for common cases
- Manual Arc cloning ceremony for factories
- Multi-step process (create spec → start child)

**3. Cognitive Load**
- Must remember all required fields
- Must specify timeouts even when defaults are fine
- Complex factory closures obscure intent

### Quantified Impact
- **10 lines** minimum for simple child (just start/stop)
- **40+ lines** for 3 similar workers with shared config
- **High cognitive load** - must remember all ChildSpec fields

---

## Solution Architecture

### Three-Layer Design Philosophy

```
Layer 1: Manual ChildSpec (Current - Preserved)
↓ For maximum control and complex scenarios

Layer 2: Single Child Builder (NEW)
↓ For individual children with optional customization

Layer 3: Batch Operations Builder (NEW)
↓ For multiple children with shared configuration
```

**Key Principles:**
1. **Progressive Disclosure** - Simple cases are simple, complex cases possible
2. **Backward Compatibility** - Zero breaking changes
3. **Type Safety** - All compile-time validated
4. **Zero Overhead** - No runtime cost
5. **YAGNI Compliance** - Only essential features

---

## Module Structure

### Directory Layout

```
src/supervisor/builder/
├── mod.rs          (~50 lines)   # Re-exports and module docs
├── constants.rs    (~40 lines)   # Default configuration values
├── single.rs       (~350 lines)  # SingleChildBuilder + tests
├── batch.rs        (~450 lines)  # ChildrenBatchBuilder + tests
└── customizer.rs   (~200 lines)  # BatchChildCustomizer + tests

Total: ~1,090 lines across 5 focused files
```

### Rationale for Modular Structure

**Benefits:**
- ✅ **Separation of Concerns** - Each file has single responsibility
- ✅ **Maintainability** - Changes isolated to specific builders
- ✅ **Discoverability** - Clear file names indicate purpose
- ✅ **Testability** - Unit tests colocated with implementation
- ✅ **Scalability** - Easy to add new builders in future

**Alternative Rejected:**
- ❌ Single `builder.rs` file (~1,000+ lines) - too large, hard to navigate

---

## API Design

### 1. Entry Points (SupervisorNode)

**Naming Convention:**
- Short, clear method names
- Obvious builder pattern (return builder, not spawned child)

```rust
impl<S, C, M> SupervisorNode<S, C, M>
where
    S: SupervisionStrategy,
    C: Child,
    M: Monitor<SupervisionEvent>,
{
    /// Create a builder for a single child.
    pub fn child(&mut self, id: impl Into<ChildId>) -> SingleChildBuilder<'_, S, C, M> {
        SingleChildBuilder {
            supervisor: self,
            id: id.into(),
            factory: None,
            restart_policy: None,
            shutdown_policy: None,
            start_timeout: None,
            shutdown_timeout: None,
        }
    }
    
    /// Create a builder for multiple children with shared defaults.
    pub fn children(&mut self) -> ChildrenBatchBuilder<'_, S, C, M> {
        ChildrenBatchBuilder {
            supervisor: self,
            default_restart: DEFAULT_RESTART_POLICY,
            default_shutdown: DEFAULT_SHUTDOWN_POLICY,
            default_start_timeout: DEFAULT_START_TIMEOUT,
            default_shutdown_timeout: DEFAULT_SHUTDOWN_TIMEOUT,
            children: Vec::new(),
        }
    }
}
```

**Design Decisions:**
- ✅ `child()` over `child_builder()` - shorter, clear intent
- ✅ `children()` over `children_builder()` - consistent, concise
- ✅ Mutable borrow required - ensures exclusive access during build

---

### 2. Default Configuration Values

**File:** `src/supervisor/builder/constants.rs`

```rust
/// Default restart policy: Always restart on failure.
///
/// This is the safest default for fault-tolerant systems as it ensures
/// children are automatically restarted after crashes.
pub const DEFAULT_RESTART_POLICY: RestartPolicy = RestartPolicy::Permanent;

/// Default shutdown policy: Graceful shutdown with 5 second timeout.
///
/// Gives children reasonable time to cleanup resources without
/// blocking supervisor shutdown indefinitely.
pub const DEFAULT_SHUTDOWN_POLICY: ShutdownPolicy = 
    ShutdownPolicy::Graceful(Duration::from_secs(5));

/// Default timeout for child start operations: 30 seconds.
///
/// Allows time for initialization tasks like database connections,
/// configuration loading, and resource acquisition.
pub const DEFAULT_START_TIMEOUT: Duration = Duration::from_secs(30);

/// Default timeout for child shutdown operations: 10 seconds.
///
/// Provides sufficient cleanup time for most children without
/// excessive blocking during supervisor shutdown.
pub const DEFAULT_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(10);
```

**Rationale:**
- **Permanent restart**: Primary supervisor purpose is fault tolerance
- **Graceful 5s shutdown**: Balance between cleanup time and responsiveness
- **30s start timeout**: Accommodates slow initialization (DB connections)
- **10s shutdown timeout**: Reasonable cleanup without long blocking

---

### 3. SingleChildBuilder

**Purpose:** Fluent API for individual child configuration with defaults.

**Type Signature:**
```rust
pub struct SingleChildBuilder<'a, S, C, M>
where
    S: SupervisionStrategy,
    C: Child,
    M: Monitor<SupervisionEvent>,
{
    supervisor: &'a mut SupervisorNode<S, C, M>,
    id: ChildId,
    factory: Option<Box<dyn FnMut() -> C>>,
    restart_policy: Option<RestartPolicy>,
    shutdown_policy: Option<ShutdownPolicy>,
    start_timeout: Option<Duration>,
    shutdown_timeout: Option<Duration>,
}
```

**Key Methods:**

```rust
// Factory configuration
pub fn factory(mut self, f: impl FnMut() -> C + 'static) -> Self;
pub fn factory_default<T: Default + Into<C> + 'static>(self) -> Self;

// Restart policy shortcuts
pub fn restart_permanent(mut self) -> Self;
pub fn restart_transient(mut self) -> Self;
pub fn restart_temporary(mut self) -> Self;
pub fn restart_policy(mut self, policy: RestartPolicy) -> Self;

// Shutdown policy shortcuts
pub fn shutdown_graceful(mut self, timeout: Duration) -> Self;
pub fn shutdown_immediate(mut self) -> Self;
pub fn shutdown_infinity(mut self) -> Self;
pub fn shutdown_policy(mut self, policy: ShutdownPolicy) -> Self;

// Timeout configuration
pub fn start_timeout(mut self, timeout: Duration) -> Self;
pub fn shutdown_timeout(mut self, timeout: Duration) -> Self;

// Execution
pub async fn spawn(self) -> Result<ChildId, SupervisorError<C::Error>>;
pub fn build(self) -> Result<ChildSpec<C>, SupervisorError<C::Error>>;
```

**Usage Examples:**

```rust
// Minimal (uses all defaults)
let id = supervisor
    .child("worker")
    .factory(|| Worker::new())
    .spawn()
    .await?;

// Custom configuration
let id = supervisor
    .child("critical")
    .factory(|| CriticalWorker::new())
    .restart_transient()
    .shutdown_graceful(Duration::from_secs(15))
    .start_timeout(Duration::from_secs(60))
    .spawn()
    .await?;

// With Default implementation
let id = supervisor
    .child("worker")
    .factory_default::<Worker>()
    .spawn()
    .await?;
```

---

### 4. ChildrenBatchBuilder

**Purpose:** Configure multiple children with shared defaults and per-child overrides.

**Type Signature:**
```rust
pub struct ChildrenBatchBuilder<'a, S, C, M>
where
    S: SupervisionStrategy,
    C: Child,
    M: Monitor<SupervisionEvent>,
{
    supervisor: &'a mut SupervisorNode<S, C, M>,
    default_restart: RestartPolicy,
    default_shutdown: ShutdownPolicy,
    default_start_timeout: Duration,
    default_shutdown_timeout: Duration,
    children: Vec<BatchChildSpec<C>>,
}

struct BatchChildSpec<C: Child> {
    id: ChildId,
    factory: Box<dyn FnMut() -> C>,
    restart_policy: Option<RestartPolicy>,      // None = use default
    shutdown_policy: Option<ShutdownPolicy>,
    start_timeout: Option<Duration>,
    shutdown_timeout: Option<Duration>,
}
```

**Key Methods:**

```rust
// Shared defaults
pub fn default_restart_permanent(mut self) -> Self;
pub fn default_restart_transient(mut self) -> Self;
pub fn default_restart_temporary(mut self) -> Self;
pub fn default_shutdown_graceful(mut self, timeout: Duration) -> Self;
pub fn default_shutdown_immediate(mut self) -> Self;
pub fn default_start_timeout(mut self, timeout: Duration) -> Self;
pub fn default_shutdown_timeout(mut self, timeout: Duration) -> Self;

// Add children
pub fn child(mut self, id: impl Into<ChildId>, factory: impl FnMut() -> C + 'static) -> Self;
pub fn child_with(self, id: impl Into<ChildId>, factory: impl FnMut() -> C + 'static) 
    -> BatchChildCustomizer<'a, S, C, M>;

// Execution
pub async fn spawn_all(mut self) -> Result<Vec<ChildId>, SupervisorError<C::Error>>;
pub async fn spawn_all_map(mut self) -> Result<HashMap<String, ChildId>, SupervisorError<C::Error>>;
```

**Configuration Precedence:**
```
Individual Override > Batch Default > Global Default
```

**Usage Examples:**

```rust
// All children with shared defaults
let ids = supervisor
    .children()
    .default_restart_permanent()
    .default_shutdown_graceful(Duration::from_secs(5))
    .child("worker-1", || Worker::new(1))
    .child("worker-2", || Worker::new(2))
    .child("worker-3", || Worker::new(3))
    .spawn_all()
    .await?;

// Shared defaults with per-child overrides
let ids = supervisor
    .children()
    .default_restart_permanent()
    .child("worker-1", || Worker::new(1))
    .child_with("special", || SpecialWorker::new())
        .restart_transient()                // Override
        .shutdown_timeout(Duration::from_secs(20))  // Override
        .done()                             // Return to batch
    .child("worker-2", || Worker::new(2))
    .spawn_all()
    .await?;

// Return as HashMap for name-based lookup
let ids = supervisor
    .children()
    .default_restart_permanent()
    .child("api", || Api::new())
    .child("db", || Db::new())
    .child("cache", || Cache::new())
    .spawn_all_map()
    .await?;

let api_id = ids.get("api").unwrap();
supervisor.restart_child(api_id).await?;
```

---

### 5. BatchChildCustomizer

**Purpose:** Per-child configuration within batch operations.

**Type Signature:**
```rust
pub struct BatchChildCustomizer<'a, S, C, M>
where
    S: SupervisionStrategy,
    C: Child,
    M: Monitor<SupervisionEvent>,
{
    batch_builder: ChildrenBatchBuilder<'a, S, C, M>,
    child_spec: BatchChildSpec<C>,
}
```

**Key Methods:**

```rust
// Override restart policy
pub fn restart_permanent(mut self) -> Self;
pub fn restart_transient(mut self) -> Self;
pub fn restart_temporary(mut self) -> Self;

// Override shutdown policy
pub fn shutdown_graceful(mut self, timeout: Duration) -> Self;
pub fn shutdown_immediate(mut self) -> Self;
pub fn shutdown_infinity(mut self) -> Self;

// Override timeouts
pub fn start_timeout(mut self, timeout: Duration) -> Self;
pub fn shutdown_timeout(mut self, timeout: Duration) -> Self;

// Return to batch builder
pub fn done(mut self) -> ChildrenBatchBuilder<'a, S, C, M>;
```

---

## Return Types: Vec vs HashMap

### Option 1: `spawn_all() → Vec<ChildId>`

**Pros:**
- ✅ Simple and predictable
- ✅ Preserves spawn order
- ✅ Lightweight (no HashMap overhead)
- ✅ Easy iteration

**Cons:**
- ❌ No name-based lookup
- ❌ Positional coupling

**Use Cases:**
- Uniform processing of all children
- Order-dependent operations
- Dynamic child lists

**Example:**
```rust
let ids = supervisor
    .children()
    .child("w1", || Worker::new(1))
    .child("w2", || Worker::new(2))
    .spawn_all()
    .await?;

for id in &ids {
    supervisor.restart_child(id).await?;
}
```

---

### Option 2: `spawn_all_map() → HashMap<String, ChildId>`

**Pros:**
- ✅ Name-based lookup
- ✅ Self-documenting
- ✅ Flexible access

**Cons:**
- ❌ HashMap overhead
- ❌ Unordered (though Rust HashMap maintains insertion order in practice)

**Use Cases:**
- Selective child access by name
- Configuration-driven systems
- Service orchestration

**Example:**
```rust
let ids = supervisor
    .children()
    .child("api", || Api::new())
    .child("db", || Db::new())
    .spawn_all_map()
    .await?;

let api_id = ids.get("api").unwrap();
supervisor.configure(api_id, config).await?;
```

---

### Design Decision: Provide Both

**Rationale:**
- ✅ User chooses based on use case
- ✅ Explicit method names show intent
- ✅ Common Rust pattern (like Iterator methods)
- ✅ No ambiguity or type inference issues

---

## Implementation Guidelines

### 1. Error Handling

**Missing Factory:**
```rust
pub fn build(self) -> Result<ChildSpec<C>, SupervisorError<C::Error>> {
    let factory = self.factory.ok_or_else(|| {
        SupervisorError::InvalidChildSpec {
            id: self.id.clone(),
            reason: "Factory function is required".to_string(),
        }
    })?;
    
    Ok(ChildSpec {
        id: self.id,
        factory,
        restart_policy: self.restart_policy.unwrap_or(DEFAULT_RESTART_POLICY),
        shutdown_policy: self.shutdown_policy.unwrap_or(DEFAULT_SHUTDOWN_POLICY),
        start_timeout: self.start_timeout.unwrap_or(DEFAULT_START_TIMEOUT),
        shutdown_timeout: self.shutdown_timeout.unwrap_or(DEFAULT_SHUTDOWN_TIMEOUT),
    })
}
```

**Batch Spawn Errors:**
- Early return on first error
- Already-started children remain running
- User responsible for cleanup if needed

---

### 2. Default Application Logic

**Batch Builder:**
```rust
fn build_child_spec(&self, batch_spec: BatchChildSpec<C>) -> ChildSpec<C> {
    ChildSpec {
        id: batch_spec.id,
        factory: batch_spec.factory,
        // Apply defaults where individual overrides not present
        restart_policy: batch_spec.restart_policy.unwrap_or(self.default_restart),
        shutdown_policy: batch_spec.shutdown_policy.unwrap_or(self.default_shutdown.clone()),
        start_timeout: batch_spec.start_timeout.unwrap_or(self.default_start_timeout),
        shutdown_timeout: batch_spec.shutdown_timeout.unwrap_or(self.default_shutdown_timeout),
    }
}
```

---

### 3. Testing Strategy

**Unit Tests (~45 tests):**

File: `src/supervisor/builder/single.rs`
- Factory configuration (with/without Default)
- All restart policy shortcuts
- All shutdown policy shortcuts
- Timeout overrides
- Default value application
- Missing factory error
- Build vs spawn

File: `src/supervisor/builder/batch.rs`
- Shared default propagation
- Per-child overrides
- Mixed default + override scenarios
- Empty batch handling
- spawn_all vs spawn_all_map

File: `src/supervisor/builder/customizer.rs`
- All override methods
- done() returns correct builder
- Customizer chain validation

**Integration Tests (~15 tests):**

File: `tests/supervisor_builder_tests.rs`
- Builder children start successfully
- Builder children restart correctly
- Supervision strategies work with builders
- Batch operations respect spawn order
- spawn_all_map returns correct mapping
- Builder + manual ChildSpec coexistence
- Real supervisor lifecycle with builders

---

### 4. Documentation Standards

**Module-Level Documentation:**
```rust
//! Builder patterns for ergonomic child process management.
//!
//! This module provides three ways to add children to a supervisor:
//!
//! 1. **Manual ChildSpec** - Maximum control, explicit configuration
//! 2. **Single Child Builder** - Fluent API with sensible defaults
//! 3. **Batch Operations** - Shared configuration for multiple children
//!
//! # Examples
//!
//! ## Manual ChildSpec (Layer 1)
//! [...]
//!
//! ## Single Child Builder (Layer 2)
//! [...]
//!
//! ## Batch Operations (Layer 3)
//! [...]
```

**Method Documentation:**
- Clear description of what method does
- When to use this method
- Example code
- Links to related methods
- Error conditions

---

## Performance Considerations

### Zero-Overhead Principle

**Builder consumption:**
- Builder types consumed by `spawn()` / `spawn_all()`
- No allocations beyond manual ChildSpec approach
- Same execution path after spec construction
- Generic monomorphization eliminates abstraction cost

**Validation:**
```rust
// Benchmark comparison (expected results)
manual_childspec:     1,234 ns  // Baseline
single_builder:       1,234 ns  // Same (zero overhead)
batch_builder_3:      3,702 ns  // 3x manual (3 children)
```

---

## Migration Guide

### From Manual ChildSpec to Builder

**Before:**
```rust
let spec = ChildSpec {
    id: "worker".into(),
    factory: || Worker::new(),
    restart_policy: RestartPolicy::Permanent,
    shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
    start_timeout: Duration::from_secs(30),
    shutdown_timeout: Duration::from_secs(10),
};
let id = supervisor.start_child(spec).await?;
```

**After:**
```rust
let id = supervisor
    .child("worker")
    .factory(|| Worker::new())
    .spawn()
    .await?;
```

**Code Reduction:** 10 lines → 4 lines (60%)

---

### Multiple Children with Shared Config

**Before:**
```rust
let spec1 = ChildSpec { /* ... full config ... */ };
let spec2 = ChildSpec { /* ... full config ... */ };
let spec3 = ChildSpec { /* ... full config ... */ };
let id1 = supervisor.start_child(spec1).await?;
let id2 = supervisor.start_child(spec2).await?;
let id3 = supervisor.start_child(spec3).await?;
```

**After:**
```rust
let ids = supervisor
    .children()
    .default_restart_permanent()
    .default_shutdown_graceful(Duration::from_secs(5))
    .child("worker-1", || Worker::new(1))
    .child("worker-2", || Worker::new(2))
    .child("worker-3", || Worker::new(3))
    .spawn_all()
    .await?;
```

**Code Reduction:** 40+ lines → 10 lines (75%)

---

## Common Patterns

### Pattern 1: Worker Pool
```rust
// Identical workers with shared configuration
let worker_ids = supervisor
    .children()
    .default_restart_permanent()
    .default_shutdown_graceful(Duration::from_secs(5))
    .child("worker-1", || Worker::new(1))
    .child("worker-2", || Worker::new(2))
    .child("worker-3", || Worker::new(3))
    .child("worker-4", || Worker::new(4))
    .spawn_all()
    .await?;
```

---

### Pattern 2: Microservice Architecture
```rust
// Different service types with category-specific configs
let service_ids = supervisor
    .children()
    .default_restart_permanent()
    .default_shutdown_graceful(Duration::from_secs(10))
    
    // Fast services
    .child_with("metrics", || MetricsService::new())
        .shutdown_graceful(Duration::from_secs(1))
        .done()
    
    // Slow initialization
    .child_with("ml-model", || MLModel::load())
        .start_timeout(Duration::from_secs(300))
        .done()
    
    // Critical services
    .child_with("health", || HealthMonitor::new())
        .restart_transient()
        .done()
    
    .spawn_all_map()
    .await?;

// Access by name
let health_id = service_ids.get("health").unwrap();
```

---

### Pattern 3: Dependency Chain
```rust
// Services with startup dependencies
let service_ids = supervisor
    .children()
    .default_restart_permanent()
    .child("config", || ConfigLoader::new())
    .child("database", || Database::new())
    .child_with("cache", || Cache::new())
        .restart_temporary()  // Cache doesn't need restart
        .done()
    .child("api-server", || ApiServer::new())
    .spawn_all()  // Spawned in order
    .await?;
```

---

## Troubleshooting

### Issue: "Factory function is required" error

**Cause:** Forgot to call `.factory()` before `.spawn()`

**Solution:**
```rust
// ❌ Missing factory
let id = supervisor
    .child("worker")
    .restart_permanent()
    .spawn()  // ERROR!
    .await?;

// ✅ With factory
let id = supervisor
    .child("worker")
    .factory(|| Worker::new())  // Required!
    .spawn()
    .await?;
```

---

### Issue: Type inference failures with batch builder

**Cause:** Compiler can't infer child type from empty batch

**Solution:**
```rust
// ❌ Can't infer type
let ids = supervisor.children().spawn_all().await?;

// ✅ Add at least one child for type inference
let ids = supervisor
    .children()
    .child("worker", || Worker::new())
    .spawn_all()
    .await?;
```

---

## Future Enhancements (Not in Scope)

### Potential Future Features:
1. **Async factory functions** - If factories need async initialization
2. **Batch error strategies** - Collect all errors vs fail-fast
3. **Configuration presets** - Named policy bundles
4. **Builder validation hooks** - Custom validation before spawn

**Note:** These are YAGNI violations until proven necessary by real usage.

---

## References

### Related Documentation
- **RT-TASK-013**: Supervisor Builder Pattern & Batch Operations (implementation task)
- **RT-TASK-007**: Supervisor Framework (dependency)
- **KNOWLEDGE-RT-003**: Supervisor Tree Implementation Strategies
- **ADR-RT-004**: Child Trait Separation

### External References
- Rust Builder Pattern: https://rust-lang.github.io/api-guidelines/type-safety.html#builders-enable-construction-of-complex-values-c-builder
- Microsoft Rust Guidelines: M-DESIGN-FOR-AI (fluent APIs)
- Workspace Standards: §6.1 (YAGNI), §6.2 (Avoid dyn), §4.3 (Module Architecture)

---

## Revision History

| Date | Author | Changes |
|------|--------|---------|
| 2025-10-08 | AI Agent | Initial creation with complete design specification |
