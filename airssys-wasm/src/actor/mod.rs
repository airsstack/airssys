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
//! # Module Organization
//!
//! The actor module is organized into four subdomains:
//!
//! ## Component (`component/`)
//! Core component actor implementation including:
//! - ComponentActor struct and lifecycle management
//! - Component registry and spawning
//! - Component-specific supervision
//!
//! ## Supervisor (`supervisor/`)
//! Generic supervision infrastructure including:
//! - Supervision policies and configuration
//! - Restart tracking and backoff strategies
//! - Rate limiting and supervision bridges
//!
//! ## Health (`health/`)
//! Health monitoring system including:
//! - Health check evaluation
//! - Health-triggered restart decisions
//!
//! ## Message (`message/`)
//! Message routing and pub/sub system including:
//! - Inter-component message routing
//! - Topic filtering and subscription management
//! - Message broker integration
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

// Subdomain module declarations
pub mod component;
pub mod health;
pub mod lifecycle;
pub mod message;
pub mod supervisor;

// Public re-exports for backward compatibility and ergonomic imports

// Component subdomain re-exports
#[doc(inline)]
pub use component::{
    ActorState, ComponentActor, ComponentMessage, ComponentRegistry,
    ComponentSpawner, ComponentSupervisor, HealthStatus, RestartDecision,
    SupervisionHandle, SupervisionState, SupervisionStatistics,
    SupervisionTree, SupervisionTreeNode, WasmRuntime,
    extract_wasm_results, prepare_wasm_params,
};

// Supervisor subdomain re-exports
#[doc(inline)]
pub use supervisor::{
    BackoffStrategy, ComponentSupervisionState, ExponentialBackoff,
    ExponentialBackoffConfig, RestartPolicy, RestartReason, RestartRecord,
    RestartStats, RestartTracker, SlidingWindowConfig, SlidingWindowLimiter,
    SupervisorConfig, SupervisorNodeBridge, SupervisorNodeWrapper,
    WindowLimitResult,
};

// Health subdomain re-exports
#[doc(inline)]
pub use health::{
    HealthDecision, HealthMonitor, HealthRestartConfig,
    MonitorHealthStatus,
};

// Lifecycle subdomain re-exports
#[doc(inline)]
pub use lifecycle::{
    EventCallback, HookResult, LifecycleContext, LifecycleHooks,
    NoOpEventCallback, NoOpHooks, RestartReason as LifecycleRestartReason,
    call_hook_with_timeout, catch_unwind_hook,
};

// Message subdomain re-exports
#[doc(inline)]
pub use message::{
    ActorSystemSubscriber, CorrelationId, CorrelationTracker,
    MessageBrokerBridge, MessageBrokerWrapper, MessagePublisher,
    MessageRouter, PendingRequest, RequestError, RequestMessage,
    ResponseMessage, RoutingStats, SubHandle, SubscriberManager,
    SubscriptionHandle, TimeoutHandler, TopicFilter, UnifiedRouter,
};
