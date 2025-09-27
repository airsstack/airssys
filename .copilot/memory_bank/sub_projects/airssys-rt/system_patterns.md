# airssys-rt System Patterns

## Actor Model Architecture

### Core Actor System
```
┌─────────────────────────────────────────┐
│            Actor System                 │
├─────────────────────────────────────────┤
│         Supervisor Tree                 │ ← Fault tolerance
├─────────────────────────────────────────┤
│         Message Router                  │ ← Message routing
├─────────────────────────────────────────┤
│         Actor Registry                  │ ← Actor addressing
├─────────────────────────────────────────┤
│         Actor Runtime                   │ ← Actor execution
├─────────────────────────────────────────┤
│         airssys-osl Integration         │ ← OS operations
└─────────────────────────────────────────┘
```

## Core Implementation Patterns

### Actor Trait Pattern
```rust
use std::collections::VecDeque;
use tokio::sync::mpsc;
use chrono::{DateTime, Utc};

use crate::message::{Message, MessageId};
use crate::context::ActorContext;

#[async_trait::async_trait]
pub trait Actor: Send + Sync + 'static {
    type Message: Message;
    type State: Send + Sync;
    
    async fn handle_message(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext,
    ) -> Result<(), ActorError>;
    
    async fn pre_start(&mut self, context: &mut ActorContext) -> Result<(), ActorError> {
        Ok(())
    }
    
    async fn post_stop(&mut self, context: &ActorContext) -> Result<(), ActorError> {
        Ok(())
    }
}

pub struct ActorHandle<T: Actor> {
    sender: mpsc::UnboundedSender<T::Message>,
    actor_id: ActorId,
    created_at: DateTime<Utc>,
}
```

### Message Passing Pattern
```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub trait Message: Send + Sync + Clone + 'static {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope<T: Message> {
    pub message_id: MessageId,
    pub sender: Option<ActorId>,
    pub recipient: ActorId,
    pub message: T,
    pub timestamp: DateTime<Utc>,
}

pub struct Mailbox<T: Message> {
    queue: VecDeque<Envelope<T>>,
    capacity: usize,
}

impl<T: Message> Mailbox<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            queue: VecDeque::with_capacity(capacity),
            capacity,
        }
    }
    
    pub fn push(&mut self, envelope: Envelope<T>) -> Result<(), MailboxError> {
        if self.queue.len() >= self.capacity {
            return Err(MailboxError::Full);
        }
        self.queue.push_back(envelope);
        Ok(())
    }
}
```

### Supervisor Pattern
```rust
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum SupervisionStrategy {
    OneForOne,
    OneForAll,
    RestForOne,
}

#[derive(Debug, Clone)]
pub enum RestartStrategy {
    Permanent,
    Temporary,
    Transient,
}

pub struct SupervisorSpec {
    pub strategy: SupervisionStrategy,
    pub max_restarts: u32,
    pub max_restart_period: Duration,
    pub children: Vec<ChildSpec>,
}

pub struct Supervisor {
    spec: SupervisorSpec,
    children: HashMap<ActorId, ChildState>,
    restart_count: u32,
    restart_window_start: DateTime<Utc>,
}

impl Supervisor {
    pub async fn handle_child_failure(
        &mut self,
        failed_child: ActorId,
        error: ActorError,
    ) -> Result<(), SupervisorError> {
        match self.spec.strategy {
            SupervisionStrategy::OneForOne => {
                self.restart_child(failed_child).await?;
            }
            SupervisionStrategy::OneForAll => {
                self.restart_all_children().await?;
            }
            SupervisionStrategy::RestForOne => {
                self.restart_child_and_after(failed_child).await?;
            }
        }
        Ok(())
    }
}
```

## Integration Patterns

### airssys-osl Integration
```rust
use airssys_osl::process::ProcessManager;
use airssys_osl::security::SecurityContext;

pub struct OSLIntegratedActor {
    process_mgr: ProcessManager,
    security_context: SecurityContext,
}

#[async_trait::async_trait]
impl Actor for OSLIntegratedActor {
    type Message = OSLMessage;
    type State = OSLState;
    
    async fn handle_message(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext,
    ) -> Result<(), ActorError> {
        match message {
            OSLMessage::SpawnProcess(cmd) => {
                let handle = self.process_mgr.spawn_secure(cmd).await?;
                context.reply(OSLMessage::ProcessSpawned(handle)).await?;
            }
            OSLMessage::FileOperation(op) => {
                let result = self.process_mgr.execute_file_operation(op).await?;
                context.reply(OSLMessage::FileResult(result)).await?;
            }
        }
        Ok(())
    }
}
```

### Performance Optimization Patterns
```rust
use std::sync::Arc;
use parking_lot::RwLock;

pub struct ActorPool<T: Actor> {
    actors: Vec<ActorHandle<T>>,
    round_robin_counter: Arc<RwLock<usize>>,
}

impl<T: Actor> ActorPool<T> {
    pub fn route_message(&self, message: T::Message) -> Result<(), ActorError> {
        let mut counter = self.round_robin_counter.write();
        let index = *counter % self.actors.len();
        *counter = (*counter + 1) % self.actors.len();
        
        self.actors[index].send(message)?;
        Ok(())
    }
}

// Zero-copy message passing for large data
pub struct ZeroCopyMessage {
    data: Arc<[u8]>,
    metadata: MessageMetadata,
}

impl Message for ZeroCopyMessage {}

impl Clone for ZeroCopyMessage {
    fn clone(&self) -> Self {
        Self {
            data: Arc::clone(&self.data), // Zero-copy clone
            metadata: self.metadata.clone(),
        }
    }
}
```

## Fault Tolerance Patterns

### Circuit Breaker Pattern
```rust
use std::sync::atomic::{AtomicU32, AtomicBool, Ordering};

pub struct CircuitBreaker {
    failure_threshold: u32,
    failure_count: AtomicU32,
    is_open: AtomicBool,
    last_failure_time: Arc<RwLock<DateTime<Utc>>>,
    timeout: Duration,
}

impl CircuitBreaker {
    pub async fn call<F, T, E>(&self, f: F) -> Result<T, CircuitBreakerError>
    where
        F: Future<Output = Result<T, E>>,
    {
        if self.is_open() {
            return Err(CircuitBreakerError::Open);
        }
        
        match f.await {
            Ok(result) => {
                self.record_success();
                Ok(result)
            }
            Err(_) => {
                self.record_failure();
                Err(CircuitBreakerError::Failure)
            }
        }
    }
}
```

### Health Monitoring Pattern
```rust
pub struct ActorSystemHealth {
    actor_count: AtomicUsize,
    message_throughput: Arc<RwLock<MessageThroughput>>,
    supervisor_stats: Arc<RwLock<SupervisorStats>>,
}

impl ActorSystemHealth {
    pub async fn health_check(&self) -> HealthStatus {
        let actor_count = self.actor_count.load(Ordering::Relaxed);
        let throughput = self.message_throughput.read().clone();
        let supervisor_stats = self.supervisor_stats.read().clone();
        
        HealthStatus {
            actor_count,
            throughput,
            supervisor_stats,
            timestamp: Utc::now(),
        }
    }
}