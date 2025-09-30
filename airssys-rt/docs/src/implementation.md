# Implementation Overview

This section provides practical guidance for using `airssys-rt` in your applications, covering everything from basic setup to advanced patterns for building robust actor-based systems.

## Getting Started with airssys-rt

The implementation guide is designed to take you from initial setup through advanced usage patterns, providing practical examples and best practices for each step of the development process.

### Prerequisites
- Rust 2021 Edition or later
- Basic understanding of async/await programming
- Familiarity with concurrent programming concepts
- Knowledge of actor model principles (helpful but not required)

### Development Workflow
1. **Setup**: Install dependencies and configure your project
2. **Basic Actors**: Create your first actors and message handlers
3. **Message Design**: Design effective message protocols
4. **Supervision**: Implement fault tolerance through supervisor trees
5. **Optimization**: Performance tuning and monitoring

## Implementation Sections

The implementation guide is organized into practical, step-by-step sections:

### [Getting Started](./implementation/getting-started.md)
Complete setup and initial project configuration:
- Project setup and dependency management
- Basic actor system creation and configuration
- Your first actor implementation
- Simple message passing examples
- Running and testing your actor system

### [Actor Creation](./implementation/actor-creation.md)
Comprehensive guide to creating and configuring actors:
- Implementing the Actor trait
- Actor state design and management
- Actor lifecycle hooks and callbacks
- Registration and addressing patterns
- Actor configuration and customization

### [Message Handling](./implementation/message-handling.md)
Best practices for designing and handling messages:
- Message type design and organization
- Request-response patterns
- Event broadcasting and subscription
- Error handling in message processing
- Message serialization and persistence

### [Supervision Setup](./implementation/supervision-setup.md)
Building robust supervision hierarchies:
- Supervisor creation and configuration
- Restart strategies and policies
- Error handling and escalation
- Monitoring and health checks
- Testing supervision behavior

## Common Implementation Patterns

### Basic Actor Pattern
```rust
use airssys_rt::{Actor, ActorResult, ActorSystem};

struct CounterActor {
    count: i64,
}

#[derive(Debug)]
enum CounterMessage {
    Increment,
    Decrement,
    GetCount(oneshot::Sender<i64>),
}

impl Actor for CounterActor {
    type Message = CounterMessage;
    
    async fn handle(&mut self, msg: CounterMessage) -> ActorResult<()> {
        match msg {
            CounterMessage::Increment => {
                self.count += 1;
                Ok(())
            }
            CounterMessage::Decrement => {
                self.count -= 1;
                Ok(())
            }
            CounterMessage::GetCount(sender) => {
                sender.send(self.count).map_err(|_| {
                    ActorError::ChannelClosed
                })?;
                Ok(())
            }
        }
    }
}
```

### Supervised Actor Pattern
```rust
use airssys_rt::{Supervisor, RestartStrategy, RestartPolicy};

async fn create_supervised_system() -> Result<(), Box<dyn std::error::Error>> {
    let system = ActorSystem::new().await?;
    
    let supervisor = Supervisor::new()
        .strategy(RestartStrategy::OneForOne)
        .policy(RestartPolicy::Permanent)
        .child("counter", CounterActor { count: 0 })
        .child("logger", LoggerActor::new())
        .start(&system).await?;
    
    Ok(())
}
```

### Request-Response Pattern
```rust
use airssys_rt::{ActorRef, ask};

async fn use_counter(counter: ActorRef<CounterMessage>) -> Result<i64, ActorError> {
    // Increment the counter
    counter.tell(CounterMessage::Increment).await?;
    
    // Get current count
    let (sender, receiver) = oneshot::channel();
    counter.tell(CounterMessage::GetCount(sender)).await?;
    let count = receiver.await?;
    
    Ok(count)
}
```

## Development Best Practices

### Actor Design Guidelines

#### 1. Single Responsibility
Design actors with a clear, focused purpose:
```rust
// Good: Focused on user management
struct UserManagerActor {
    users: HashMap<UserId, User>,
    database: DatabasePool,
}

// Avoid: Too many responsibilities
struct SystemActor {
    users: HashMap<UserId, User>,
    orders: Vec<Order>,
    payments: PaymentProcessor,
    notifications: NotificationService,
}
```

#### 2. Immutable Messages
Design messages as immutable data structures:
```rust
// Good: Immutable message with owned data
#[derive(Debug, Clone)]
enum UserMessage {
    CreateUser { name: String, email: String },
    UpdateUser { id: UserId, changes: UserUpdate },
}

// Avoid: Mutable references in messages
enum BadUserMessage {
    ProcessUser(&mut User),  // Don't do this!
}
```

#### 3. Clear Error Handling
Handle errors explicitly and appropriately:
```rust
impl Actor for UserActor {
    async fn handle(&mut self, msg: UserMessage) -> ActorResult<()> {
        match msg {
            UserMessage::CreateUser { name, email } => {
                self.create_user(name, email).await
                    .map_err(|e| ActorError::BusinessLogic(e.to_string()))
            }
        }
    }
}
```

### Message Design Patterns

#### Command-Query Separation
Separate commands (actions) from queries (data requests):
```rust
#[derive(Debug)]
enum UserCommand {
    CreateUser { name: String, email: String },
    UpdateUser { id: UserId, changes: UserUpdate },
    DeleteUser { id: UserId },
}

#[derive(Debug)]
enum UserQuery {
    GetUser { id: UserId, response: oneshot::Sender<Option<User>> },
    ListUsers { response: oneshot::Sender<Vec<User>> },
}
```

#### Event Sourcing Pattern
Use events to represent state changes:
```rust
#[derive(Debug, Clone)]
enum UserEvent {
    UserCreated { id: UserId, name: String, email: String },
    UserUpdated { id: UserId, changes: UserUpdate },
    UserDeleted { id: UserId },
}
```

### Supervision Strategies

#### Choose Appropriate Restart Policies
```rust
// Permanent: Critical services that should always be running
let database_supervisor = Supervisor::new()
    .policy(RestartPolicy::Permanent)
    .child("db_pool", DatabasePoolActor::new());

// Temporary: One-time tasks that shouldn't restart
let task_supervisor = Supervisor::new()
    .policy(RestartPolicy::Temporary)
    .child("backup_task", BackupTaskActor::new());

// Transient: Services that should restart only on abnormal termination
let cache_supervisor = Supervisor::new()
    .policy(RestartPolicy::Transient)
    .child("cache", CacheActor::new());
```

## Testing Strategies

### Unit Testing Actors
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use airssys_rt::testing::ActorTestHarness;
    
    #[tokio::test]
    async fn test_counter_increment() {
        let mut harness = ActorTestHarness::new(CounterActor { count: 0 });
        
        harness.send(CounterMessage::Increment).await;
        
        let (sender, receiver) = oneshot::channel();
        harness.send(CounterMessage::GetCount(sender)).await;
        let count = receiver.await.unwrap();
        
        assert_eq!(count, 1);
    }
}
```

### Integration Testing
```rust
#[tokio::test]
async fn test_user_system_integration() {
    let system = ActorSystem::new().await.unwrap();
    
    let user_manager = system.spawn(UserManagerActor::new()).await.unwrap();
    let notification_service = system.spawn(NotificationActor::new()).await.unwrap();
    
    // Test user creation with notification
    user_manager.tell(UserMessage::CreateUser {
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    }).await.unwrap();
    
    // Verify notification was sent
    // ... test implementation
}
```

## Performance Considerations

### Memory Management
- Keep actor state minimal and efficient
- Use `Arc` for sharing large immutable data
- Implement proper cleanup in actor stop hooks
- Monitor memory usage in production

### Message Throughput
- Batch related operations when possible
- Use zero-copy techniques for large payloads
- Consider message prioritization for critical operations
- Profile message processing hot paths

### Scalability Planning
- Design for horizontal scaling from the start
- Use actor pools for high-throughput scenarios
- Implement backpressure for flow control
- Monitor system metrics and actor health

The implementation guide provides everything you need to build robust, scalable applications using the `airssys-rt` actor runtime.