//! Builder patterns for ergonomic supervisor child management.
//!
//! This module provides three layers of child configuration to reduce boilerplate
//! while maintaining full backward compatibility and customization capabilities:
//!
//! 1. **Manual ChildSpec** (existing) - Maximum control for complex scenarios
//! 2. **SingleChildBuilder** (Phase 1) - Fluent API with sensible defaults
//! 3. **ChildrenBatchBuilder** (Phase 2) - Batch operations with shared configuration
//!
//! # Design Philosophy
//!
//! The builder pattern follows the principle of **progressive disclosure**:
//! - Simple cases are simple (minimal configuration)
//! - Complex cases are possible (full customization available)
//! - Zero breaking changes (100% backward compatible)
//! - Zero runtime overhead (compile-time validated)
//!
//! # Examples
//!
//! ## Single Child Builder (Phase 1)
//!
//! ```rust,no_run
//! use airssys_rt::supervisor::*;
//! use std::time::Duration;
//!
//! # async fn example() -> Result<(), SupervisorError<std::io::Error>> {
//! let mut supervisor = SupervisorNode::new(OneForOne::default());
//!
//! // Minimal configuration (uses defaults)
//! let id = supervisor
//!     .child("worker")
//!     .factory(|| my_worker())
//!     .spawn()
//!     .await?;
//!
//! // Full customization
//! let id = supervisor
//!     .child("critical")
//!     .factory(|| my_critical_worker())
//!     .restart_transient()
//!     .shutdown_graceful(Duration::from_secs(15))
//!     .start_timeout(Duration::from_secs(60))
//!     .spawn()
//!     .await?;
//! # Ok(())
//! # }
//! # fn my_worker() -> impl airssys_rt::supervisor::Child { unimplemented!() }
//! # fn my_critical_worker() -> impl airssys_rt::supervisor::Child { unimplemented!() }
//! ```
//!
//! # Migration Guide
//!
//! This section helps you migrate from manual `ChildSpec` construction to builder patterns.
//!
//! ## Why Migrate?
//!
//! Builder patterns offer several advantages:
//! - **60-75% less boilerplate** for common cases
//! - **Better IDE autocomplete** and discoverability
//! - **Type-safe fluent API** prevents configuration errors
//! - **Same runtime performance** - zero overhead abstraction
//! - **Fully backward compatible** - can mix approaches
//!
//! ## When to Use Which Approach
//!
//! ### Use Builder Pattern When:
//! - ✅ Spawning standard workers with typical policies
//! - ✅ Adding multiple similar children (batch operations)
//! - ✅ Working on new code or refactoring
//! - ✅ Want reduced boilerplate and better readability
//!
//! ### Use Manual ChildSpec When:
//! - ⚠️ Reusable child specifications across multiple supervisors
//! - ⚠️ Dynamic child spec construction based on runtime data
//! - ⚠️ Legacy code that works fine (don't fix what isn't broken)
//! - ⚠️ Complex custom configurations beyond builder support
//!
//! ## Migration Examples
//!
//! ### Example 1: Simple Worker Migration
//!
//! **Before (Manual ChildSpec):**
//! ```rust,no_run
//! # use airssys_rt::supervisor::*;
//! # use std::time::Duration;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let mut supervisor: SupervisorNode<OneForOne, _, _> = unimplemented!();
//! // OLD WAY - 10 lines of boilerplate
//! let child_id = supervisor.add_child(
//!     ChildSpec {
//!         id: "worker-1".to_string(),
//!         factory: Box::new(|| Box::new(MyWorker::new())),
//!         restart_policy: RestartPolicy::Permanent,
//!         shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
//!         start_timeout: Duration::from_secs(30),
//!         shutdown_timeout: Duration::from_secs(10),
//!     }
//! ).await?;
//! # Ok(())
//! # }
//! # struct MyWorker;
//! # impl MyWorker { fn new() -> Self { Self } }
//! # impl airssys_rt::supervisor::Child for MyWorker {
//! #     type Error = std::io::Error;
//! #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
//! #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
//! # }
//! ```
//!
//! **After (Builder Pattern):**
//! ```rust,no_run
//! # use airssys_rt::supervisor::*;
//! # use std::time::Duration;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let mut supervisor: SupervisorNode<OneForOne, _, _> = unimplemented!();
//! // NEW WAY - 4 lines, 60% less code
//! let child_id = supervisor
//!     .child("worker-1")
//!     .factory(|| MyWorker::new())
//!     .spawn()
//!     .await?;
//! # Ok(())
//! # }
//! # struct MyWorker;
//! # impl MyWorker { fn new() -> Self { Self } }
//! # impl airssys_rt::supervisor::Child for MyWorker {
//! #     type Error = std::io::Error;
//! #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
//! #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
//! # }
//! ```
//!
//! ### Example 2: Custom Policies Migration
//!
//! **Before:**
//! ```rust,no_run
//! # use airssys_rt::supervisor::*;
//! # use std::time::Duration;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let mut supervisor: SupervisorNode<OneForOne, _, _> = unimplemented!();
//! let child_id = supervisor.add_child(
//!     ChildSpec {
//!         id: "critical-service".to_string(),
//!         factory: Box::new(|| Box::new(CriticalService::new())),
//!         restart_policy: RestartPolicy::Transient,
//!         shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(15)),
//!         start_timeout: Duration::from_secs(60),
//!         shutdown_timeout: Duration::from_secs(20),
//!     }
//! ).await?;
//! # Ok(())
//! # }
//! # struct CriticalService;
//! # impl CriticalService { fn new() -> Self { Self } }
//! # impl airssys_rt::supervisor::Child for CriticalService {
//! #     type Error = std::io::Error;
//! #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
//! #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
//! # }
//! ```
//!
//! **After:**
//! ```rust,no_run
//! # use airssys_rt::supervisor::*;
//! # use std::time::Duration;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let mut supervisor: SupervisorNode<OneForOne, _, _> = unimplemented!();
//! let child_id = supervisor
//!     .child("critical-service")
//!     .factory(|| CriticalService::new())
//!     .restart_transient()
//!     .shutdown_graceful(Duration::from_secs(15))
//!     .start_timeout(Duration::from_secs(60))
//!     .shutdown_timeout(Duration::from_secs(20))
//!     .spawn()
//!     .await?;
//! # Ok(())
//! # }
//! # struct CriticalService;
//! # impl CriticalService { fn new() -> Self { Self } }
//! # impl airssys_rt::supervisor::Child for CriticalService {
//! #     type Error = std::io::Error;
//! #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
//! #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
//! # }
//! ```
//!
//! ### Example 3: Batch Operations Migration
//!
//! **Before (Multiple Manual ChildSpecs):**
//! ```rust,no_run
//! # use airssys_rt::supervisor::*;
//! # use std::time::Duration;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let mut supervisor: SupervisorNode<OneForOne, _, _> = unimplemented!();
//! // OLD WAY - 40+ lines of repetitive code
//! let id1 = supervisor.add_child(
//!     ChildSpec {
//!         id: "worker-1".to_string(),
//!         factory: Box::new(|| Box::new(Worker::new(1))),
//!         restart_policy: RestartPolicy::Permanent,
//!         shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
//!         start_timeout: Duration::from_secs(30),
//!         shutdown_timeout: Duration::from_secs(10),
//!     }
//! ).await?;
//!
//! let id2 = supervisor.add_child(
//!     ChildSpec {
//!         id: "worker-2".to_string(),
//!         factory: Box::new(|| Box::new(Worker::new(2))),
//!         restart_policy: RestartPolicy::Permanent,
//!         shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
//!         start_timeout: Duration::from_secs(30),
//!         shutdown_timeout: Duration::from_secs(10),
//!     }
//! ).await?;
//!
//! let id3 = supervisor.add_child(
//!     ChildSpec {
//!         id: "worker-3".to_string(),
//!         factory: Box::new(|| Box::new(Worker::new(3))),
//!         restart_policy: RestartPolicy::Permanent,
//!         shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
//!         start_timeout: Duration::from_secs(30),
//!         shutdown_timeout: Duration::from_secs(10),
//!     }
//! ).await?;
//! # Ok(())
//! # }
//! # struct Worker;
//! # impl Worker { fn new(_: u32) -> Self { Self } }
//! # impl airssys_rt::supervisor::Child for Worker {
//! #     type Error = std::io::Error;
//! #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
//! #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
//! # }
//! ```
//!
//! **After (Batch Builder):**
//! ```rust,no_run
//! # use airssys_rt::supervisor::*;
//! # use std::time::Duration;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let mut supervisor: SupervisorNode<OneForOne, _, _> = unimplemented!();
//! // NEW WAY - 10 lines, 75% less code
//! let child_ids = supervisor
//!     .children()
//!     .restart_policy(RestartPolicy::Permanent)
//!     .shutdown_policy(ShutdownPolicy::Graceful(Duration::from_secs(5)))
//!     .child("worker-1", || Worker::new(1))
//!     .child("worker-2", || Worker::new(2))
//!     .child("worker-3", || Worker::new(3))
//!     .spawn_all()
//!     .await?;
//! # Ok(())
//! # }
//! # struct Worker;
//! # impl Worker { fn new(_: u32) -> Self { Self } }
//! # impl airssys_rt::supervisor::Child for Worker {
//! #     type Error = std::io::Error;
//! #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
//! #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
//! # }
//! ```
//!
//! ### Example 4: Per-Child Customization in Batch
//!
//! **When you need shared defaults BUT one child needs different policies:**
//!
//! ```rust,no_run
//! # use airssys_rt::supervisor::*;
//! # use std::time::Duration;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let mut supervisor: SupervisorNode<OneForOne, _, _> = unimplemented!();
//! let child_ids = supervisor
//!     .children()
//!     // Shared defaults for most workers
//!     .restart_policy(RestartPolicy::Permanent)
//!     .shutdown_policy(ShutdownPolicy::Graceful(Duration::from_secs(5)))
//!     // Regular workers use defaults
//!     .child("worker-1", || Worker::new(1))
//!     .child("worker-2", || Worker::new(2))
//!     // Special worker with custom policies
//!     .child_with("special-worker", || SpecialWorker::new())
//!         .restart_policy(RestartPolicy::Transient)
//!         .shutdown_policy(ShutdownPolicy::Immediate)
//!         .start_timeout(Duration::from_secs(60))
//!         .done()
//!     // Back to regular worker with defaults
//!     .child("worker-3", || Worker::new(3))
//!     .spawn_all()
//!     .await?;
//! # Ok(())
//! # }
//! # struct Worker;
//! # impl Worker { fn new(_: u32) -> Self { Self } }
//! # impl airssys_rt::supervisor::Child for Worker {
//! #     type Error = std::io::Error;
//! #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
//! #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
//! # }
//! # struct SpecialWorker;
//! # impl SpecialWorker { fn new() -> Self { Self } }
//! # impl airssys_rt::supervisor::Child for SpecialWorker {
//! #     type Error = std::io::Error;
//! #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
//! #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
//! # }
//! ```
//!
//! ## Migration Strategy
//!
//! ### Incremental Migration (Recommended)
//!
//! You don't need to migrate everything at once. Both approaches work together:
//!
//! ```rust,no_run
//! # use airssys_rt::supervisor::*;
//! # use std::time::Duration;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let mut supervisor: SupervisorNode<OneForOne, _, _> = unimplemented!();
//! // Old code - keep it if it works
//! let old_child = supervisor.add_child(
//!     ChildSpec { /* ... */ # id: String::new(), factory: Box::new(|| Box::new(OldWorker)), restart_policy: RestartPolicy::Permanent, shutdown_policy: ShutdownPolicy::Immediate, start_timeout: Duration::from_secs(30), shutdown_timeout: Duration::from_secs(10) }
//! ).await?;
//!
//! // New code - use builder for new workers
//! let new_child = supervisor
//!     .child("new-worker")
//!     .factory(|| NewWorker::new())
//!     .spawn()
//!     .await?;
//! # Ok(())
//! # }
//! # struct OldWorker;
//! # impl airssys_rt::supervisor::Child for OldWorker {
//! #     type Error = std::io::Error;
//! #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
//! #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
//! # }
//! # struct NewWorker;
//! # impl NewWorker { fn new() -> Self { Self } }
//! # impl airssys_rt::supervisor::Child for NewWorker {
//! #     type Error = std::io::Error;
//! #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
//! #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
//! # }
//! ```
//!
//! ### Migration Checklist
//!
//! - [ ] Identify repetitive `ChildSpec` construction code
//! - [ ] Start with new features or code you're already modifying
//! - [ ] For batches of similar children, use `ChildrenBatchBuilder`
//! - [ ] For individual children, use `SingleChildBuilder`
//! - [ ] Test thoroughly - behavior should be identical
//! - [ ] Gradually migrate old code as you touch it
//! - [ ] Keep manual `ChildSpec` for truly complex cases
//!
//! ## Common Patterns
//!
//! ### Pattern 1: Pool of Identical Workers
//!
//! ```rust,no_run
//! # use airssys_rt::supervisor::*;
//! # use std::time::Duration;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let mut supervisor: SupervisorNode<OneForOne, _, _> = unimplemented!();
//! # let pool_size = 10;
//! let child_ids = supervisor
//!     .children()
//!     .restart_policy(RestartPolicy::Permanent)
//!     .shutdown_policy(ShutdownPolicy::Graceful(Duration::from_secs(5)))
//!     .start_timeout(Duration::from_secs(30))
//!     .shutdown_timeout(Duration::from_secs(10))
//!     .child(format!("worker-{}", 0), || Worker::new(0))
//!     # ;
//! # for i in 1..pool_size {
//! #     let child_ids = child_ids
//!     .child(format!("worker-{}", i), || Worker::new(i))
//! # ;
//! # }
//! # let child_ids = child_ids
//!     .spawn_all()
//!     .await?;
//! # Ok(())
//! # }
//! # struct Worker;
//! # impl Worker { fn new(_: u32) -> Self { Self } }
//! # impl airssys_rt::supervisor::Child for Worker {
//! #     type Error = std::io::Error;
//! #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
//! #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
//! # }
//! ```
//!
//! ### Pattern 2: Name-Based Child Lookup
//!
//! ```rust,no_run
//! # use airssys_rt::supervisor::*;
//! # use std::time::Duration;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let mut supervisor: SupervisorNode<OneForOne, _, _> = unimplemented!();
//! // Use spawn_all_map() for name-based lookups
//! let child_map = supervisor
//!     .children()
//!     .restart_policy(RestartPolicy::Permanent)
//!     .child("database-pool", || DatabasePool::new())
//!     .child("cache-manager", || CacheManager::new())
//!     .child("metrics-collector", || MetricsCollector::new())
//!     .spawn_all_map()
//!     .await?;
//!
//! // Access children by name
//! let db_id = child_map.get("database-pool").unwrap();
//! let cache_id = child_map.get("cache-manager").unwrap();
//! # Ok(())
//! # }
//! # struct DatabasePool;
//! # impl DatabasePool { fn new() -> Self { Self } }
//! # impl airssys_rt::supervisor::Child for DatabasePool {
//! #     type Error = std::io::Error;
//! #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
//! #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
//! # }
//! # struct CacheManager;
//! # impl CacheManager { fn new() -> Self { Self } }
//! # impl airssys_rt::supervisor::Child for CacheManager {
//! #     type Error = std::io::Error;
//! #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
//! #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
//! # }
//! # struct MetricsCollector;
//! # impl MetricsCollector { fn new() -> Self { Self } }
//! # impl airssys_rt::supervisor::Child for MetricsCollector {
//! #     type Error = std::io::Error;
//! #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
//! #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
//! # }
//! ```
//!
//! ## Performance Notes
//!
//! - **Zero Runtime Overhead**: Builders compile to the same code as manual `ChildSpec`
//! - **No Allocations**: Builder is consumed and generates `ChildSpec` directly
//! - **Type Safety**: All validation happens at compile time
//! - **Inlining**: Builder methods inline away completely in release builds
//!
//! ## Further Reading
//!
//! - [`SingleChildBuilder`] - Single child builder API reference
//! - [`ChildrenBatchBuilder`] - Batch builder API reference
//! - [`BatchChildCustomizer`] - Per-child customization API reference
//! - [`constants`] - Default values and configuration rationale
//! - `examples/supervisor_builder_phase1.rs` - Comprehensive Phase 1 examples
//! - `examples/supervisor_builder_phase2.rs` - Comprehensive Phase 2 examples
//!
//! # Architecture
//!
//! The builder system maintains strict compliance with AirsSys standards:
//! - **§6.2 Avoid dyn**: Generic constraints instead of trait objects (factory storage excepted)
//! - **§6.1 YAGNI**: Only essential features, no speculative complexity
//! - **M-DESIGN-FOR-AI**: Fluent APIs for excellent discoverability
//! - **M-ESSENTIAL-FN-INHERENT**: Core functionality in inherent methods
//!
//! # See Also
//!
//! - [`SingleChildBuilder`] - Fluent builder for individual children
//! - [`constants`] - Default configuration values and rationale

pub mod batch;
pub mod constants;
pub mod customizer;
pub mod single;

// Re-exports for convenient access
pub use batch::ChildrenBatchBuilder;
pub use constants::{
    DEFAULT_RESTART_POLICY, DEFAULT_SHUTDOWN_POLICY, DEFAULT_SHUTDOWN_TIMEOUT,
    DEFAULT_START_TIMEOUT,
};
pub use customizer::BatchChildCustomizer;
pub use single::SingleChildBuilder;
