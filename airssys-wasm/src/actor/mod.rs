//! Actor system integration for WebAssembly components.
//!
//! This module implements ComponentActor, which bridges WASM components with the
//! airssys-rt actor system. ComponentActor implements both `Actor` (message handling)
//! and `Child` (lifecycle management) traits, establishing the foundational actor-hosted
//! component architecture.
//!
//! # Architecture (ADR-WASM-006)
//!
//! ComponentActor follows the dual-trait pattern:
//! - **Actor trait**: Handles inter-component messages via mailbox
//! - **Child trait**: Manages WASM runtime lifecycle under supervisor control
//!
//! This separation allows:
//! - Supervision of component lifecycle independent of message passing
//! - Integration with airssys-rt SupervisorNode for automatic restart
//! - Clean separation between communication and lifecycle concerns
//!
//! # Module Organization (§4.3)
//!
//! Per workspace standards, this `mod.rs` contains ONLY module declarations
//! and re-exports. Implementation code resides in individual module files:
//!
//! - `component_actor.rs` - ComponentActor struct, ActorState, ComponentMessage
//! - `actor_impl.rs` - Actor trait implementation (message handling)
//! - `child_impl.rs` - Child trait implementation (WASM lifecycle)
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use airssys_wasm::actor::{ComponentActor, ComponentMessage, ActorState};
//! use airssys_wasm::core::{ComponentId, ComponentMetadata, CapabilitySet};
//! use airssys_rt::supervisor::Child;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create component actor
//!     let component_id = ComponentId::new("my-component");
//!     let metadata = ComponentMetadata { /* ... */ };
//!     let caps = CapabilitySet::new();
//!     
//!     let mut actor = ComponentActor::new(component_id, metadata, caps);
//!     
//!     // Start WASM runtime (Child::start)
//!     actor.start().await?;
//!     
//!     assert_eq!(*actor.state(), ActorState::Ready);
//!     assert!(actor.is_wasm_loaded());
//!     
//!     // Graceful shutdown (Child::stop)
//!     actor.stop(Duration::from_secs(5)).await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! # References
//!
//! - **ADR-WASM-006**: Component Isolation and Sandboxing (Actor-based approach)
//! - **ADR-RT-004**: Actor and Child Trait Separation
//! - **WASM-TASK-004**: Actor System Integration (Block 3)
//! - **KNOWLEDGE-WASM-016**: Actor System Integration Implementation Guide

// Module declarations (§4.3 - mod.rs declaration-only pattern)
pub mod component_actor;
pub mod actor_impl;
pub mod child_impl;
pub mod type_conversion;
pub mod component_spawner;
pub mod component_registry;
pub mod message_router;
pub mod supervisor_config;
pub mod component_supervisor;
pub mod supervisor_bridge;
pub mod supervisor_wrapper;
pub mod health_restart;
pub mod exponential_backoff;
pub mod restart_tracker;
pub mod sliding_window_limiter;
pub mod health_monitor;
pub mod message_broker_bridge;
pub mod message_publisher;
pub mod message_filter;
pub mod subscriber_manager;

// Public re-exports for ergonomic imports
pub use component_actor::{
    ActorState, ComponentActor, ComponentMessage, HealthStatus, WasmRuntime,
};
pub use type_conversion::{prepare_wasm_params, extract_wasm_results};
pub use component_spawner::ComponentSpawner;
pub use component_registry::ComponentRegistry;
pub use message_router::MessageRouter;
pub use supervisor_config::{
    BackoffStrategy, RestartPolicy, SupervisorConfig,
};
pub use component_supervisor::{
    ComponentSupervisor, RestartDecision, SupervisionHandle, SupervisionState,
    SupervisionStatistics, SupervisionTree, SupervisionTreeNode,
};
pub use supervisor_bridge::{ComponentSupervisionState, SupervisorNodeBridge};
pub use supervisor_wrapper::{SupervisorNodeWrapper, RestartStats};
pub use health_restart::HealthRestartConfig;
pub use exponential_backoff::{ExponentialBackoff, ExponentialBackoffConfig};
pub use restart_tracker::{RestartReason, RestartRecord, RestartTracker};
pub use sliding_window_limiter::{SlidingWindowConfig, SlidingWindowLimiter, WindowLimitResult};
pub use health_monitor::{HealthDecision, HealthMonitor, HealthStatus as MonitorHealthStatus};
pub use message_broker_bridge::{MessageBrokerBridge, MessageBrokerWrapper, SubscriptionHandle};
pub use message_publisher::MessagePublisher;
pub use message_filter::TopicFilter;
pub use subscriber_manager::{SubHandle, SubscriberManager};
