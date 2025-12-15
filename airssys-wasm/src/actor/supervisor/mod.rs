//! Generic supervision infrastructure for component restart management.
//!
//! This module provides reusable supervision building blocks including
//! restart policies, backoff strategies, rate limiting, and bridges to
//! the airssys-rt supervisor system.
//!
//! # Architecture
//!
//! The supervisor infrastructure is designed to be:
//! - **Reusable**: Generic patterns applicable beyond components
//! - **Composable**: Building blocks that work together
//! - **Configurable**: Flexible policies and strategies
//!
//! # Module Organization
//!
//! - `supervisor_config` - Supervision policies and configuration
//! - `supervisor_bridge` - Bridge to airssys-rt SupervisorNode
//! - `supervisor_wrapper` - Wrapper around SupervisorNode
//! - `restart_tracker` - Restart history tracking
//! - `exponential_backoff` - Exponential backoff strategy
//! - `sliding_window_limiter` - Rate limiting for restarts

// Module declarations
pub mod supervisor_config;
pub mod supervisor_bridge;
pub mod supervisor_wrapper;
pub mod restart_tracker;
pub mod exponential_backoff;
pub mod sliding_window_limiter;

// Public re-exports
#[doc(inline)]
pub use supervisor_config::{
    BackoffStrategy, RestartPolicy, SupervisorConfig,
};
#[doc(inline)]
pub use supervisor_bridge::{ComponentSupervisionState, SupervisorNodeBridge};
#[doc(inline)]
pub use supervisor_wrapper::{SupervisorNodeWrapper, RestartStats};
#[doc(inline)]
pub use restart_tracker::{RestartReason, RestartRecord, RestartTracker};
#[doc(inline)]
pub use exponential_backoff::{ExponentialBackoff, ExponentialBackoffConfig};
#[doc(inline)]
pub use sliding_window_limiter::{SlidingWindowConfig, SlidingWindowLimiter, WindowLimitResult};
