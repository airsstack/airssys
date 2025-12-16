//! Lifecycle management for ComponentActor.
//!
//! This module provides lifecycle hooks, event callbacks, and state management
//! capabilities for ComponentActor, enabling extensible component behavior
//! without modifying framework code.
//!
//! # Modules
//!
//! - `hooks`: LifecycleHooks trait for pre/post-start/stop and error handling
//! - `callbacks`: EventCallback trait for monitoring and observability
//! - `executor`: Helper functions for hook execution with timeout and panic protection
//!
//! # Architecture
//!
//! ```text
//! ┌────────────────────────────────────────┐
//! │         ComponentActor<S>              │
//! │  ┌──────────────────────────────────┐  │
//! │  │  LifecycleHooks (extensibility)  │  │
//! │  │  - pre_start / post_start        │  │
//! │  │  - pre_stop / post_stop          │  │
//! │  │  - on_message_received           │  │
//! │  │  - on_error / on_restart         │  │
//! │  └──────────────────────────────────┘  │
//! │  ┌──────────────────────────────────┐  │
//! │  │  Custom State (generic <S>)      │  │
//! │  │  - Arc<RwLock<S>>                │  │
//! │  │  - with_state() / with_state_mut │  │
//! │  └──────────────────────────────────┘  │
//! │  ┌──────────────────────────────────┐  │
//! │  │  EventCallback (monitoring)      │  │
//! │  │  - on_message_received/processed │  │
//! │  │  - on_error_occurred             │  │
//! │  │  - on_restart_triggered          │  │
//! │  │  - on_health_changed             │  │
//! │  └──────────────────────────────────┘  │
//! └────────────────────────────────────────┘
//! ```
//!
//! # References
//!
//! - **WASM-TASK-004 Phase 5 Task 5.2**: Lifecycle Hooks and Custom State Management
//! - **ADR-WASM-018**: Three-Layer Architecture
//! - **ADR-WASM-019**: Runtime Dependency Management

pub mod callbacks;
pub mod executor;
pub mod hooks;

pub use callbacks::{EventCallback, NoOpEventCallback};
pub use executor::{call_hook_with_timeout, catch_unwind_hook};
pub use hooks::{HookResult, LifecycleContext, LifecycleHooks, NoOpHooks, RestartReason};
