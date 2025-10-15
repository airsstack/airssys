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
