# Actor Model Design

The actor model in `airssys-rt` is designed to provide a clean, type-safe, and performant implementation of the actor pattern while remaining true to the principles established by Erlang/OTP.

## Actor Trait Architecture

### Core Actor Trait
The fundamental trait that all actors must implement:

```rust
#[async_trait]
pub trait Actor: Send + 'static {
    /// The message type this actor can handle
    type Message: Send + 'static;
    
    /// Handle an incoming message
    async fn handle(&mut self, msg: Self::Message) -> ActorResult<()>;
    
    /// Called when the actor is started
    async fn started(&mut self) -> ActorResult<()> {
        Ok(())
    }
    
    /// Called when the actor is stopping
    async fn stopping(&mut self) -> ActorResult<()> {
        Ok(())
    }
    
    /// Handle system errors and supervision events
    async fn handle_error(&mut self, error: ActorError) -> ErrorAction {
        ErrorAction::Stop
    }
}
```

### Actor State Management
Actors encapsulate their state completely:

```rust
struct DatabaseActor {
    // Private state - not accessible externally
    connection_pool: DatabasePool,
    active_queries: HashMap<QueryId, Query>,
    stats: QueryStats,
    config: DatabaseConfig,
}

impl Actor for DatabaseActor {
    type Message = DatabaseMessage;
    
    async fn handle(&mut self, msg: DatabaseMessage) -> ActorResult<()> {
        match msg {
            DatabaseMessage::Execute { query, response_channel } => {
                // State is modified only through message handling
                let query_id = self.generate_query_id();
                self.active_queries.insert(query_id, query.clone());
                
                let result = self.connection_pool.execute(query).await?;
                response_channel.send(result).map_err(|_| ActorError::ChannelClosed)?;
                
                self.active_queries.remove(&query_id);
                self.stats.increment_completed_queries();
                
                Ok(())
            }
            DatabaseMessage::GetStats { response_channel } => {
                response_channel.send(self.stats.clone())
                    .map_err(|_| ActorError::ChannelClosed)?;
                Ok(())
            }
        }
    }
}
```

## Message Design Patterns

### Strongly Typed Messages
Messages are defined as enums with explicit variants:

```rust
#[derive(Debug, Clone)]
pub enum UserServiceMessage {
    CreateUser {
        name: String,
        email: String,
        response: oneshot::Sender<Result<UserId, UserError>>,
    },
    GetUser {
        id: UserId,
        response: oneshot::Sender<Result<User, UserError>>,
    },
    UpdateUser {
        id: UserId,
        updates: UserUpdates,
        response: oneshot::Sender<Result<(), UserError>>,
    },
    DeleteUser {
        id: UserId,
        response: oneshot::Sender<Result<(), UserError>>,
    },
    // Events (no response required)
    UserCreated {
        user: User,
        timestamp: DateTime<Utc>,
    },
    UserDeleted {
        user_id: UserId,
        timestamp: DateTime<Utc>,
    },
}
```

### Message Categories

#### Command Messages
Request the actor to perform an action:
```rust
UserServiceMessage::CreateUser {
    name: "Alice".to_string(),
    email: "alice@example.com".to_string(),
    response: tx,
}
```

#### Query Messages  
Request information from the actor:
```rust
UserServiceMessage::GetUser {
    id: user_id,
    response: tx,
}
```

#### Event Messages
Notify the actor of something that happened:
```rust
UserServiceMessage::UserCreated {
    user: new_user,
    timestamp: Utc::now(),
}
```

## Actor Lifecycle Management

### Spawning Actors
Actors are created and registered in the system:

```rust
use airssys_rt::{ActorSystem, ActorRef};

// Create the actor system
let system = ActorSystem::new().await?;

// Spawn an actor
let user_service = UserService::new(database_pool);
let user_actor_ref = system.spawn(user_service).await?;

// Spawn with supervision
let supervised_actor = system
    .spawn_supervised(user_service, supervisor_ref)
    .await?;
```

### Actor References
Actors are accessed through type-safe references:

```rust
#[derive(Debug, Clone)]
pub struct ActorRef<M> {
    pid: ProcessId,
    sender: mpsc::UnboundedSender<M>,
    _phantom: PhantomData<M>,
}

impl<M: Send + 'static> ActorRef<M> {
    /// Send a message without waiting for response
    pub async fn tell(&self, msg: M) -> Result<(), SendError> {
        self.sender.send(msg).map_err(|_| SendError::ActorNotFound)
    }
    
    /// Send a message and wait for response
    pub async fn ask<R>(&self, msg: M) -> Result<R, AskError>
    where
        M: HasResponse<Response = R>,
    {
        let (tx, rx) = oneshot::channel();
        let msg_with_response = msg.with_response(tx);
        self.tell(msg_with_response).await?;
        rx.await.map_err(|_| AskError::ResponseTimeout)
    }
}
```

### Actor Addressing
Actors can be addressed in multiple ways:

```rust
// By typed reference (preferred)
let user_ref: ActorRef<UserServiceMessage> = system.get_actor("user_service")?;

// By process ID
let pid = ProcessId::new();
let actor_ref = system.get_actor_by_pid(pid)?;

// By name registration
system.register_actor("user_service", user_actor_ref).await?;
let user_ref = system.resolve_actor("user_service").await?;
```

## Actor Communication Patterns

### Request-Response Pattern
Synchronous-style communication with response channels:

```rust
async fn create_user_example() -> Result<UserId, UserError> {
    let (response_tx, response_rx) = oneshot::channel();
    
    user_actor.tell(UserServiceMessage::CreateUser {
        name: "Bob".to_string(),
        email: "bob@example.com".to_string(),
        response: response_tx,
    }).await?;
    
    response_rx.await?
}
```

### Fire-and-Forget Pattern
Asynchronous messaging without response:

```rust
async fn log_user_activity() -> Result<(), ActorError> {
    logger_actor.tell(LogMessage {
        level: LogLevel::Info,
        message: "User logged in".to_string(),
        timestamp: Utc::now(),
    }).await?;
    
    Ok(())
}
```

### Event Broadcasting
One-to-many communication:

```rust
struct EventBus {
    subscribers: HashMap<TypeId, Vec<ActorRef<dyn Any>>>,
}

impl EventBus {
    async fn publish<E: Event>(&self, event: E) -> Result<(), BroadcastError> {
        let type_id = TypeId::of::<E>();
        if let Some(subscribers) = self.subscribers.get(&type_id) {
            for subscriber in subscribers {
                subscriber.tell(event.clone()).await?;
            }
        }
        Ok(())
    }
}
```

## Actor State Patterns

### Stateful Actors
Actors that maintain persistent state:

```rust
struct CounterActor {
    count: i64,
    max_count: i64,
    name: String,
}

impl Actor for CounterActor {
    type Message = CounterMessage;
    
    async fn handle(&mut self, msg: CounterMessage) -> ActorResult<()> {
        match msg {
            CounterMessage::Increment => {
                if self.count < self.max_count {
                    self.count += 1;
                    Ok(())
                } else {
                    Err(ActorError::InvalidState("Counter at maximum".to_string()))
                }
            }
            CounterMessage::Reset => {
                self.count = 0;
                Ok(())
            }
        }
    }
}
```

### Stateless Actors
Actors that process messages without maintaining state:

```rust
struct HashingActor;

impl Actor for HashingActor {
    type Message = HashRequest;
    
    async fn handle(&mut self, msg: HashRequest) -> ActorResult<()> {
        match msg {
            HashRequest::Sha256 { data, response } => {
                let hash = sha256::digest(data);
                response.send(hash).map_err(|_| ActorError::ChannelClosed)?;
                Ok(())
            }
        }
    }
}
```

### State Machine Actors
Actors that implement explicit state machines:

```rust
#[derive(Debug, Clone)]
enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
}

struct ConnectionActor {
    state: ConnectionState,
    connection: Option<Connection>,
    retry_count: u32,
    max_retries: u32,
}

impl Actor for ConnectionActor {
    type Message = ConnectionMessage;
    
    async fn handle(&mut self, msg: ConnectionMessage) -> ActorResult<()> {
        match (&self.state, msg) {
            (ConnectionState::Disconnected, ConnectionMessage::Connect) => {
                self.state = ConnectionState::Connecting;
                self.start_connection().await?;
                Ok(())
            }
            (ConnectionState::Connected, ConnectionMessage::Disconnect) => {
                self.state = ConnectionState::Disconnected;
                self.close_connection().await?;
                Ok(())
            }
            (state, msg) => {
                Err(ActorError::InvalidTransition {
                    from: format!("{:?}", state),
                    message: format!("{:?}", msg),
                })
            }
        }
    }
}
```

## Performance Optimizations

### Zero-Copy Message Passing
Leverage Rust's ownership system for efficient message transfer:

```rust
// Messages are moved, not copied
pub enum FileMessage {
    ProcessData(Vec<u8>),  // Data is moved to actor
    ProcessFile(PathBuf),  // Only path is moved, not file content
}

// Large data can be shared using Arc
pub enum SharedDataMessage {
    ProcessShared(Arc<LargeDataStructure>),
}
```

### Message Batching
Process multiple messages together for efficiency:

```rust
impl Actor for BatchProcessorActor {
    type Message = BatchMessage;
    
    async fn handle(&mut self, msg: BatchMessage) -> ActorResult<()> {
        match msg {
            BatchMessage::AddItem(item) => {
                self.buffer.push(item);
                
                // Process when buffer is full
                if self.buffer.len() >= self.batch_size {
                    self.process_batch().await?;
                }
                Ok(())
            }
            BatchMessage::Flush => {
                self.process_batch().await
            }
        }
    }
}
```

### Actor Pooling
Reuse actor instances for high-throughput scenarios:

```rust
struct WorkerPool {
    workers: Vec<ActorRef<WorkMessage>>,
    next_worker: AtomicUsize,
}

impl WorkerPool {
    async fn submit_work(&self, work: WorkMessage) -> Result<(), PoolError> {
        let worker_index = self.next_worker.fetch_add(1, Ordering::Relaxed) % self.workers.len();
        let worker = &self.workers[worker_index];
        worker.tell(work).await?;
        Ok(())
    }
}
```

## Error Handling in Actors

### Actor-Level Error Handling
Actors can handle their own errors:

```rust
impl Actor for ResilientActor {
    type Message = WorkMessage;
    
    async fn handle(&mut self, msg: WorkMessage) -> ActorResult<()> {
        self.process_message(msg).await
            .or_else(|e| self.handle_processing_error(e))
    }
    
    async fn handle_error(&mut self, error: ActorError) -> ErrorAction {
        match error {
            ActorError::TemporaryFailure(_) => {
                // Retry or continue running
                ErrorAction::Continue
            }
            ActorError::PermanentFailure(_) => {
                // Let supervisor handle restart
                ErrorAction::Stop
            }
        }
    }
}
```

### Error Propagation to Supervisors
Unhandled errors are escalated to supervisors:

```rust
pub enum ErrorAction {
    Continue,    // Actor continues running
    Stop,        // Actor stops, supervisor decides restart
    Escalate,    // Error is escalated to parent supervisor
}
```

The actor model design in `airssys-rt` provides a solid foundation for building concurrent, fault-tolerant systems while leveraging Rust's type system and performance characteristics.