# KNOWLEDGE-RT-011: Actor System Framework Implementation Guide

**Status:** active  
**Created:** 2025-10-06  
**Task:** RT-TASK-006  
**Category:** Implementation Guide

## Overview

Complete implementation guide for RT-TASK-006: Actor System Framework. This guide provides the main entry point and user-facing API that integrates all foundation components (RT-TASK-001 through RT-TASK-005) into a cohesive, production-ready actor runtime system.

## Strategic Context

### Purpose
The Actor System Framework provides:
1. **High-level API**: Simple, ergonomic interface for actor lifecycle management
2. **Component Integration**: Seamless integration of message broker, mailboxes, and actors
3. **Configuration Management**: System-wide configuration with sensible defaults
4. **Lifecycle Management**: Actor spawning, supervision, and graceful shutdown
5. **Error Handling**: Comprehensive error types and recovery strategies

### Dependencies (All Satisfied ✅)
- ✅ RT-TASK-001: Message System (MessageEnvelope, Message trait)
- ✅ RT-TASK-002: Actor System Core (Actor trait, ActorContext)
- ✅ RT-TASK-003: Mailbox System (BoundedMailbox, UnboundedMailbox)
- ✅ RT-TASK-004: Message Broker (MessageBroker, InMemoryMessageBroker, ActorRegistry)
- ✅ RT-TASK-005: Actor Addressing (ActorAddress, PoolStrategy)

## Architecture Design

### Core Components

```
┌─────────────────────────────────────────────────────────┐
│                    ActorSystem<B>                       │
│  ┌───────────────────────────────────────────────────┐  │
│  │                  SystemConfig                     │  │
│  │  - default_mailbox_capacity                       │  │
│  │  - spawn_timeout                                  │  │
│  │  - shutdown_timeout                               │  │
│  │  - max_actors                                     │  │
│  └───────────────────────────────────────────────────┘  │
│                                                         │
│  ┌───────────────────────────────────────────────────┐  │
│  │              MessageBroker<M>                     │  │
│  │  - ActorRegistry (routing)                        │  │
│  │  - Message delivery                               │  │
│  └───────────────────────────────────────────────────┘  │
│                                                         │
│  ┌───────────────────────────────────────────────────┐  │
│  │        Actor Metadata Storage                     │  │
│  │  HashMap<ActorId, ActorMetadata>                  │  │
│  │  - spawned_at, mailbox_sender                     │  │
│  └───────────────────────────────────────────────────┘  │
│                                                         │
│  ┌───────────────────────────────────────────────────┐  │
│  │         ActorSpawnBuilder<A, B>                   │  │
│  │  - Fluent API for actor spawning                  │  │
│  │  - Configuration (name, capacity, backpressure)   │  │
│  └───────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

### Type Safety Architecture

**Generic Constraints** (§6.2, §6.3 M-DI-HIERARCHY):
```rust
// ActorSystem is generic over MessageBroker
pub struct ActorSystem<B: MessageBroker> { /* ... */ }

// ActorSpawnBuilder is generic over Actor and Broker
pub struct ActorSpawnBuilder<A, B> 
where
    A: Actor,
    B: MessageBroker,
{ /* ... */ }
```

**Key Design Principles**:
- ✅ No `dyn` trait objects (static dispatch only)
- ✅ Compile-time type safety
- ✅ Zero-cost abstractions
- ✅ Cheap Clone via Arc<Inner> (§6.3 M-SERVICES-CLONE)

## Implementation Plan

### Phase 1: Error Types & Configuration (Day 1, ~4-5 hours)

#### Module Structure
```
src/system/
├── mod.rs           # Module declarations only (§4.3)
├── errors.rs        # SystemError types (~200 lines)
└── config.rs        # SystemConfig (~250 lines)
```

#### File 1.1: `src/system/mod.rs`
```rust
//! Actor system framework with lifecycle management.
//!
//! Provides the main entry point for the actor runtime system.

pub mod config;
pub mod errors;
pub mod actor_system;
pub mod builder;

// Re-exports
pub use config::SystemConfig;
pub use errors::SystemError;
pub use actor_system::ActorSystem;
pub use builder::ActorSpawnBuilder;
```

**Compliance**: §4.3 - mod.rs contains ONLY declarations and re-exports

#### File 1.2: `src/system/errors.rs` (~200 lines)
```rust
//! System-level error types.

// Layer 1: Standard library
use std::fmt;

// Layer 2: Third-party
use thiserror::Error;

// Layer 3: Internal
use crate::broker::BrokerError;
use crate::util::ActorId;

/// System-level errors for actor runtime operations.
///
/// Follows §6.3 M-ERRORS-CANONICAL-STRUCTS pattern with
/// structured error types and helper methods.
#[derive(Error, Debug)]
pub enum SystemError {
    /// Actor with given ID not found in registry
    #[error("Actor not found: {0}")]
    ActorNotFound(ActorId),
    
    /// Failed to spawn actor
    #[error("Failed to spawn actor: {0}")]
    SpawnFailed(String),
    
    /// System is shutting down, cannot accept new operations
    #[error("System shutdown in progress")]
    ShuttingDown,
    
    /// Actor mailbox is full (bounded mailbox with backpressure)
    #[error("Actor mailbox full: {0}")]
    MailboxFull(ActorId),
    
    /// Message broker error
    #[error("Broker error: {0}")]
    BrokerError(#[from] BrokerError),
    
    /// Configuration validation error
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    /// Actor limit exceeded
    #[error("Actor limit exceeded: current {current}, max {max}")]
    ActorLimitExceeded { current: usize, max: usize },
    
    /// Shutdown timeout exceeded
    #[error("Shutdown timeout exceeded after {0:?}")]
    ShutdownTimeout(std::time::Duration),
}

impl SystemError {
    /// Check if error is transient (can retry)
    pub fn is_transient(&self) -> bool {
        matches!(self, SystemError::MailboxFull(_))
    }
    
    /// Check if error is fatal (system must stop)
    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            SystemError::ShuttingDown | SystemError::ShutdownTimeout(_)
        )
    }
    
    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        !self.is_fatal()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_actor_not_found_display() {
        let id = ActorId::new();
        let err = SystemError::ActorNotFound(id);
        assert!(err.to_string().contains("Actor not found"));
    }
    
    #[test]
    fn test_spawn_failed_display() {
        let err = SystemError::SpawnFailed("initialization error".to_string());
        assert!(err.to_string().contains("Failed to spawn"));
    }
    
    #[test]
    fn test_error_categorization() {
        let mailbox_err = SystemError::MailboxFull(ActorId::new());
        assert!(mailbox_err.is_transient());
        assert!(!mailbox_err.is_fatal());
        assert!(mailbox_err.is_recoverable());
        
        let shutdown_err = SystemError::ShuttingDown;
        assert!(!shutdown_err.is_transient());
        assert!(shutdown_err.is_fatal());
        assert!(!shutdown_err.is_recoverable());
    }
    
    #[test]
    fn test_broker_error_conversion() {
        let broker_err = BrokerError::ActorNotRegistered(ActorId::new());
        let system_err: SystemError = broker_err.into();
        assert!(matches!(system_err, SystemError::BrokerError(_)));
    }
    
    #[test]
    fn test_actor_limit_exceeded() {
        let err = SystemError::ActorLimitExceeded {
            current: 100,
            max: 50,
        };
        let msg = err.to_string();
        assert!(msg.contains("100"));
        assert!(msg.contains("50"));
    }
}
```

**Tests**: 8-10 unit tests for error creation, Display, categorization, conversions

#### File 1.3: `src/system/config.rs` (~250 lines)
```rust
//! System configuration with sensible defaults.

// Layer 1: Standard library
use std::time::Duration;

// Layer 2: Third-party
use serde::{Deserialize, Serialize};

// Layer 3: Internal
// (none initially)

/// System-wide configuration for actor runtime.
///
/// Provides sensible defaults following §6.1 YAGNI principles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    /// Default mailbox capacity for bounded mailboxes
    pub default_mailbox_capacity: usize,
    
    /// Timeout for actor spawn operations
    pub spawn_timeout: Duration,
    
    /// Timeout for graceful system shutdown
    pub shutdown_timeout: Duration,
    
    /// Maximum concurrent actors (0 = unlimited)
    pub max_actors: usize,
    
    /// Enable system metrics collection (disabled by default - YAGNI)
    pub enable_metrics: bool,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            default_mailbox_capacity: 1000,
            spawn_timeout: Duration::from_secs(5),
            shutdown_timeout: Duration::from_secs(30),
            max_actors: 0, // unlimited
            enable_metrics: false, // YAGNI §6.1
        }
    }
}

impl SystemConfig {
    /// Create a new configuration builder
    pub fn builder() -> SystemConfigBuilder {
        SystemConfigBuilder::default()
    }
    
    /// Validate configuration values
    pub fn validate(&self) -> Result<(), String> {
        if self.default_mailbox_capacity == 0 {
            return Err("default_mailbox_capacity must be > 0".to_string());
        }
        
        if self.spawn_timeout.as_secs() == 0 {
            return Err("spawn_timeout must be > 0".to_string());
        }
        
        if self.shutdown_timeout.as_secs() == 0 {
            return Err("shutdown_timeout must be > 0".to_string());
        }
        
        Ok(())
    }
}

/// Builder for SystemConfig with fluent API
#[derive(Debug, Default)]
pub struct SystemConfigBuilder {
    config: SystemConfig,
}

impl SystemConfigBuilder {
    /// Set default mailbox capacity
    pub fn with_mailbox_capacity(mut self, capacity: usize) -> Self {
        self.config.default_mailbox_capacity = capacity;
        self
    }
    
    /// Set spawn timeout
    pub fn with_spawn_timeout(mut self, timeout: Duration) -> Self {
        self.config.spawn_timeout = timeout;
        self
    }
    
    /// Set shutdown timeout
    pub fn with_shutdown_timeout(mut self, timeout: Duration) -> Self {
        self.config.shutdown_timeout = timeout;
        self
    }
    
    /// Set maximum actors (0 = unlimited)
    pub fn with_max_actors(mut self, max: usize) -> Self {
        self.config.max_actors = max;
        self
    }
    
    /// Enable metrics collection
    pub fn with_metrics(mut self, enabled: bool) -> Self {
        self.config.enable_metrics = enabled;
        self
    }
    
    /// Build and validate configuration
    pub fn build(self) -> Result<SystemConfig, String> {
        self.config.validate()?;
        Ok(self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = SystemConfig::default();
        assert_eq!(config.default_mailbox_capacity, 1000);
        assert_eq!(config.spawn_timeout, Duration::from_secs(5));
        assert_eq!(config.shutdown_timeout, Duration::from_secs(30));
        assert_eq!(config.max_actors, 0);
        assert!(!config.enable_metrics);
    }
    
    #[test]
    fn test_config_validation() {
        let config = SystemConfig::default();
        assert!(config.validate().is_ok());
        
        let invalid = SystemConfig {
            default_mailbox_capacity: 0,
            ..Default::default()
        };
        assert!(invalid.validate().is_err());
    }
    
    #[test]
    fn test_builder_pattern() {
        let config = SystemConfig::builder()
            .with_mailbox_capacity(500)
            .with_spawn_timeout(Duration::from_secs(10))
            .with_max_actors(100)
            .build()
            .unwrap();
        
        assert_eq!(config.default_mailbox_capacity, 500);
        assert_eq!(config.spawn_timeout, Duration::from_secs(10));
        assert_eq!(config.max_actors, 100);
    }
    
    #[test]
    fn test_serialization() {
        let config = SystemConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: SystemConfig = serde_json::from_str(&json).unwrap();
        
        assert_eq!(config.default_mailbox_capacity, deserialized.default_mailbox_capacity);
    }
    
    #[test]
    fn test_builder_validation() {
        let invalid = SystemConfig::builder()
            .with_mailbox_capacity(0)
            .build();
        
        assert!(invalid.is_err());
    }
}
```

**Tests**: 10-12 unit tests for defaults, builder, validation, serialization

**Phase 1 Deliverables**:
- ✅ 3 files created (~500 lines total)
- ✅ 18-22 unit tests passing
- ✅ Zero warnings
- ✅ Update `src/lib.rs` to expose system module

### Phase 2: Actor System Core (Day 2-3, ~10-12 hours)

#### File 2.1: `src/system/actor_system.rs` (~600-700 lines)

**Key Implementation Components**:

```rust
//! Main actor system implementation.

// Layer 1: Standard library
use std::collections::HashMap;
use std::sync::Arc;

// Layer 2: Third-party
use parking_lot::RwLock;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use chrono::{DateTime, Utc}; // §3.2 MANDATORY

// Layer 3: Internal
use crate::actor::{Actor, ActorContext, ActorLifecycle, ActorState, ErrorAction};
use crate::broker::MessageBroker;
use crate::mailbox::{BoundedMailbox, BackpressureStrategy, MailboxReceiver, MailboxSender};
use crate::message::{Message, MessageEnvelope};
use crate::util::{ActorAddress, ActorId};
use super::{SystemConfig, SystemError, ActorSpawnBuilder};

/// System state enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SystemState {
    /// System is running normally
    Running,
    /// Graceful shutdown in progress
    ShuttingDown,
    /// System has stopped
    Stopped,
}

/// Internal actor metadata
struct ActorMetadata {
    id: ActorId,
    name: Option<String>,
    spawned_at: DateTime<Utc>,
    // Type-erased mailbox sender (each actor may have different message type)
    mailbox_sender: Box<dyn std::any::Any + Send + Sync>,
    // Join handle for actor task
    task_handle: JoinHandle<()>,
}

/// Main actor system managing actor lifecycle.
///
/// Generic over MessageBroker implementation for pluggability.
/// Implements cheap Clone via Arc<Inner> pattern (§6.3 M-SERVICES-CLONE).
///
/// # Examples
///
/// ```rust
/// use airssys_rt::*;
///
/// #[tokio::main]
/// async fn main() {
///     let config = SystemConfig::default();
///     let broker = InMemoryMessageBroker::new();
///     let system = ActorSystem::new(config, broker);
///     
///     // Spawn actors using builder pattern
///     // let addr = system.spawn().with_name("worker").spawn(actor).await?;
///     
///     // Graceful shutdown
///     // system.shutdown().await?;
/// }
/// ```
pub struct ActorSystem<B: MessageBroker> {
    inner: Arc<ActorSystemInner<B>>,
}

struct ActorSystemInner<B: MessageBroker> {
    config: SystemConfig,
    broker: B,
    actors: RwLock<HashMap<ActorId, ActorMetadata>>,
    state: RwLock<SystemState>,
    shutdown_signal: RwLock<Option<oneshot::Sender<()>>>,
}

impl<B: MessageBroker> ActorSystem<B> {
    /// Create a new actor system with given configuration and broker.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::*;
    ///
    /// let config = SystemConfig::default();
    /// let broker = InMemoryMessageBroker::new();
    /// let system = ActorSystem::new(config, broker);
    /// ```
    pub fn new(config: SystemConfig, broker: B) -> Self {
        Self {
            inner: Arc::new(ActorSystemInner {
                config,
                broker,
                actors: RwLock::new(HashMap::new()),
                state: RwLock::new(SystemState::Running),
                shutdown_signal: RwLock::new(None),
            }),
        }
    }
    
    /// Spawn a new actor with builder pattern.
    ///
    /// Returns ActorSpawnBuilder for fluent configuration.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let address = system.spawn()
    ///     .with_name("my-actor")
    ///     .with_mailbox_capacity(500)
    ///     .spawn(actor)
    ///     .await?;
    /// ```
    pub fn spawn<A>(&self) -> ActorSpawnBuilder<A, B>
    where
        A: Actor,
    {
        ActorSpawnBuilder::new(self.clone())
    }
    
    /// Get current number of active actors.
    pub fn actor_count(&self) -> usize {
        self.inner.actors.read().len()
    }
    
    /// Check if system is running.
    pub fn is_running(&self) -> bool {
        *self.inner.state.read() == SystemState::Running
    }
    
    /// Check if system is shutting down.
    pub fn is_shutting_down(&self) -> bool {
        *self.inner.state.read() == SystemState::ShuttingDown
    }
    
    /// Initiate graceful shutdown with timeout.
    ///
    /// Waits for all actors to finish processing current messages.
    /// Returns error if shutdown timeout is exceeded.
    pub async fn shutdown(&self) -> Result<(), SystemError> {
        // Set state to shutting down
        {
            let mut state = self.inner.state.write();
            if *state != SystemState::Running {
                return Err(SystemError::ShuttingDown);
            }
            *state = SystemState::ShuttingDown;
        }
        
        // Wait for all actors to finish (with timeout)
        let timeout = self.inner.config.shutdown_timeout;
        let result = tokio::time::timeout(timeout, self.wait_for_actors()).await;
        
        match result {
            Ok(()) => {
                *self.inner.state.write() = SystemState::Stopped;
                Ok(())
            }
            Err(_) => Err(SystemError::ShutdownTimeout(timeout)),
        }
    }
    
    /// Force immediate shutdown without waiting.
    pub async fn force_shutdown(&self) {
        *self.inner.state.write() = SystemState::Stopped;
        
        // Abort all actor tasks
        let mut actors = self.inner.actors.write();
        for metadata in actors.values() {
            metadata.task_handle.abort();
        }
        actors.clear();
    }
    
    /// Internal: Spawn actor with full configuration
    pub(crate) async fn spawn_actor_internal<A>(
        &self,
        actor: A,
        name: Option<String>,
        mailbox_capacity: usize,
        backpressure: BackpressureStrategy,
    ) -> Result<ActorId, SystemError>
    where
        A: Actor + Send + 'static,
        A::Message: 'static,
    {
        // Check if shutting down
        if self.is_shutting_down() {
            return Err(SystemError::ShuttingDown);
        }
        
        // Check actor limit
        if self.inner.config.max_actors > 0 {
            let current = self.actor_count();
            if current >= self.inner.config.max_actors {
                return Err(SystemError::ActorLimitExceeded {
                    current,
                    max: self.inner.config.max_actors,
                });
            }
        }
        
        // Create actor ID and address
        let actor_id = ActorId::new();
        let address = if let Some(ref n) = name {
            ActorAddress::named(n)
        } else {
            ActorAddress::anonymous(actor_id)
        };
        
        // Create mailbox
        let (sender, receiver) = BoundedMailbox::<A::Message, _>::new(
            mailbox_capacity,
            backpressure,
        );
        
        // Create actor context
        let context = ActorContext::<A::Message>::new(address.clone());
        
        // Register with broker
        self.inner.broker.register_actor(address.clone(), sender.clone()).await
            .map_err(|e| SystemError::SpawnFailed(e.to_string()))?;
        
        // Spawn actor task
        let task_handle = self.spawn_actor_task(actor, receiver, context);
        
        // Store metadata
        let metadata = ActorMetadata {
            id: actor_id,
            name,
            spawned_at: Utc::now(), // §3.2
            mailbox_sender: Box::new(sender),
            task_handle,
        };
        
        self.inner.actors.write().insert(actor_id, metadata);
        
        Ok(actor_id)
    }
    
    /// Spawn tokio task for actor message loop
    fn spawn_actor_task<A>(
        &self,
        mut actor: A,
        mut receiver: impl MailboxReceiver<A::Message> + Send + 'static,
        mut context: ActorContext<A::Message>,
    ) -> JoinHandle<()>
    where
        A: Actor + Send + 'static,
        A::Message: 'static,
    {
        tokio::spawn(async move {
            // Call pre_start hook
            if let Err(_e) = actor.pre_start(&mut context).await {
                // Log error and stop
                return;
            }
            
            // Message processing loop
            while let Ok(envelope) = receiver.receive().await {
                let result = actor.handle_message(envelope.payload, &mut context).await;
                
                match result {
                    Ok(()) => {
                        // Continue processing
                    }
                    Err(err) => {
                        // Call error handler
                        let action = actor.on_error(err, &mut context).await;
                        
                        match action {
                            ErrorAction::Stop => break,
                            ErrorAction::Resume => continue,
                            ErrorAction::Restart => {
                                // Simple restart: continue loop
                                // TODO: RT-TASK-007 will add supervisor restart
                                continue;
                            }
                            ErrorAction::Escalate => {
                                // TODO: RT-TASK-007 will escalate to supervisor
                                break;
                            }
                        }
                    }
                }
            }
            
            // Call post_stop hook
            let _ = actor.post_stop(&mut context).await;
        })
    }
    
    /// Wait for all actors to finish
    async fn wait_for_actors(&self) {
        loop {
            let count = self.actor_count();
            if count == 0 {
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }
}

// Cheap clone implementation (§6.3 M-SERVICES-CLONE)
impl<B: MessageBroker> Clone for ActorSystem<B> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::broker::InMemoryMessageBroker;
    
    #[test]
    fn test_new_system() {
        let config = SystemConfig::default();
        let broker = InMemoryMessageBroker::new();
        let system = ActorSystem::new(config, broker);
        
        assert!(system.is_running());
        assert_eq!(system.actor_count(), 0);
    }
    
    #[test]
    fn test_clone_cheap() {
        let config = SystemConfig::default();
        let broker = InMemoryMessageBroker::new();
        let system1 = ActorSystem::new(config, broker);
        let system2 = system1.clone();
        
        assert_eq!(system1.actor_count(), system2.actor_count());
    }
    
    // More tests for spawn, shutdown, etc.
}
```

**Tests**: 20-25 unit tests covering creation, spawn, shutdown, state management

### Phase 3: Actor Spawn Builder (Day 3-4, ~6-8 hours)

#### File 3.1: `src/system/builder.rs` (~400-500 lines)

```rust
//! Actor spawn builder with fluent API.

// Layer 1: Standard library
use std::marker::PhantomData;

// Layer 2: Third-party
// (minimal)

// Layer 3: Internal
use crate::actor::Actor;
use crate::broker::MessageBroker;
use crate::mailbox::BackpressureStrategy;
use crate::util::ActorAddress;
use super::{ActorSystem, SystemError};

/// Fluent builder for spawning actors.
///
/// Provides ergonomic API for configuring and spawning actors with
/// compile-time type safety.
///
/// # Examples
///
/// ```rust,ignore
/// let address = system.spawn()
///     .with_name("worker")
///     .with_mailbox_capacity(500)
///     .with_backpressure(BackpressureStrategy::Drop)
///     .spawn(MyActor::new())
///     .await?;
/// ```
pub struct ActorSpawnBuilder<A, B>
where
    A: Actor,
    B: MessageBroker,
{
    system: ActorSystem<B>,
    name: Option<String>,
    mailbox_capacity: Option<usize>,
    backpressure: Option<BackpressureStrategy>,
    supervisor: Option<ActorAddress>, // For RT-TASK-007
    _marker: PhantomData<A>,
}

impl<A, B> ActorSpawnBuilder<A, B>
where
    A: Actor,
    B: MessageBroker,
{
    /// Create new builder (internal, called by ActorSystem)
    pub(crate) fn new(system: ActorSystem<B>) -> Self {
        Self {
            system,
            name: None,
            mailbox_capacity: None,
            backpressure: None,
            supervisor: None,
            _marker: PhantomData,
        }
    }
    
    /// Set actor name for identification.
    ///
    /// Named actors can be looked up by name in the registry.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
    
    /// Set mailbox capacity (creates bounded mailbox).
    ///
    /// Defaults to system config if not specified.
    pub fn with_mailbox_capacity(mut self, capacity: usize) -> Self {
        self.mailbox_capacity = Some(capacity);
        self
    }
    
    /// Set backpressure strategy for bounded mailbox.
    ///
    /// Defaults to Block strategy if not specified.
    pub fn with_backpressure(mut self, strategy: BackpressureStrategy) -> Self {
        self.backpressure = Some(strategy);
        self
    }
    
    /// Set supervisor for fault tolerance.
    ///
    /// NOTE: Supervision will be implemented in RT-TASK-007.
    pub fn under_supervisor(mut self, supervisor: ActorAddress) -> Self {
        self.supervisor = Some(supervisor);
        self
    }
    
    /// Spawn the actor with configured settings.
    ///
    /// Returns ActorAddress for sending messages to the actor.
    pub async fn spawn(self, actor: A) -> Result<ActorAddress, SystemError>
    where
        A: Send + 'static,
        A::Message: 'static,
    {
        // Use builder config or system defaults
        let capacity = self.mailbox_capacity
            .unwrap_or(self.system.inner.config.default_mailbox_capacity);
        
        let backpressure = self.backpressure
            .unwrap_or(BackpressureStrategy::Block);
        
        // Call system's internal spawn method
        let id = self.system.spawn_actor_internal(
            actor,
            self.name.clone(),
            capacity,
            backpressure,
        ).await?;
        
        // Return address
        let address = if let Some(name) = self.name {
            ActorAddress::named(name)
        } else {
            ActorAddress::anonymous(id)
        };
        
        Ok(address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::broker::InMemoryMessageBroker;
    use crate::message::Message;
    use async_trait::async_trait;
    
    #[derive(Debug, Clone)]
    struct TestMessage;
    
    impl Message for TestMessage {
        const MESSAGE_TYPE: &'static str = "test";
    }
    
    struct TestActor;
    
    #[async_trait]
    impl Actor for TestActor {
        type Message = TestMessage;
        type Error = std::io::Error;
        
        async fn handle_message(
            &mut self,
            _msg: Self::Message,
            _ctx: &mut crate::actor::ActorContext<Self::Message>,
        ) -> Result<(), Self::Error> {
            Ok(())
        }
    }
    
    #[tokio::test]
    async fn test_builder_with_name() {
        let config = super::super::SystemConfig::default();
        let broker = InMemoryMessageBroker::new();
        let system = ActorSystem::new(config, broker);
        
        let address = system.spawn()
            .with_name("test-actor")
            .spawn(TestActor)
            .await
            .unwrap();
        
        assert_eq!(address.name(), Some("test-actor"));
    }
    
    // More builder tests...
}
```

**Tests**: 12-15 unit tests for builder pattern, spawn integration

### Phase 4: Integration & Examples (Day 5, ~6-8 hours)

#### Example Files

**File 4.1: `examples/actor_system_basic.rs`**
- Basic ActorSystem creation
- Simple actor spawning
- Message sending
- Graceful shutdown

**File 4.2: `examples/actor_system_advanced.rs`**
- Multiple actors
- Inter-actor communication
- Error handling
- Configuration examples

**File 4.3: Integration Tests** (`tests/actor_system_integration.rs`)
- Full end-to-end system tests
- Multiple actor scenarios
- Shutdown behavior tests
- Error recovery tests

### Phase 5: Documentation & Polish (Day 6, ~4-5 hours)

#### Tasks
1. Comprehensive rustdoc for all public items
2. Module-level documentation
3. Update airssys-rt/README.md
4. Update memory bank progress
5. Create completion summary
6. Quality gates verification

## Success Metrics

### Quantitative
- **Lines of Code**: ~2,000-2,500
- **Unit Tests**: 50-60 tests
- **Integration Tests**: 10-15 tests
- **Examples**: 2 comprehensive
- **Test Coverage**: >95%
- **Warnings**: 0

### Qualitative
- ✅ Ergonomic builder pattern API
- ✅ Type-safe actor spawning
- ✅ Graceful shutdown
- ✅ Comprehensive error handling
- ✅ Production-ready quality

## Workspace Standards Compliance

### §2.1 Import Organization
- ✅ 3-layer import structure in all files

### §3.2 Time Handling
- ✅ Use `chrono::DateTime<Utc>` for timestamps

### §4.3 Module Architecture
- ✅ mod.rs contains ONLY declarations and re-exports

### §6.1 YAGNI Principles
- ✅ Metrics disabled by default
- ✅ Build only needed features

### §6.2 Avoid dyn Patterns
- ✅ Generic constraints instead of trait objects
- ✅ Static dispatch throughout

### §6.3 Microsoft Rust Guidelines
- ✅ M-SERVICES-CLONE: Cheap Clone via Arc
- ✅ M-ERRORS-CANONICAL-STRUCTS: Structured errors
- ✅ M-DI-HIERARCHY: Prefer generics over dyn
- ✅ M-DESIGN-FOR-AI: Comprehensive docs and examples

## Related Knowledge

- **KNOWLEDGE-RT-001**: Zero-cost abstractions
- **KNOWLEDGE-RT-005**: Actor system core patterns
- **KNOWLEDGE-RT-009**: Message broker architecture
- **KNOWLEDGE-RT-010**: Actor messaging patterns

## Next Steps

After RT-TASK-006 completion:
- **RT-TASK-007**: Supervisor Framework (10-12 days)
- **RT-TASK-008**: Performance Features (3-5 days)
- **RT-TASK-009**: OSL Integration (10-14 days)

---

**Status**: Ready for implementation  
**Last Updated**: 2025-10-06
