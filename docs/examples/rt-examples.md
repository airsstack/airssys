# RT Examples

Examples demonstrating the Actor Runtime for high-concurrency fault-tolerant applications.

## Basic Actor Examples

### Simple Counter Actor

```rust
use airssys_rt::prelude::*;
use async_trait::async_trait;

#[derive(Debug, Clone)]
enum CounterMsg {
    Increment,
    Decrement,
    GetCount(tokio::sync::oneshot::Sender<i64>),
}

impl Message for CounterMsg {
    const MESSAGE_TYPE: &'static str = "counter";
}

struct CounterActor {
    count: i64,
}

#[async_trait]
impl Actor for CounterActor {
    type Message = CounterMsg;
    type Error = std::io::Error;
    
    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        msg: Self::Message,
        _ctx: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        match msg {
            CounterMsg::Increment => {
                self.count += 1;
                println!("Count: {}", self.count);
            }
            CounterMsg::Decrement => {
                self.count -= 1;
                println!("Count: {}", self.count);
            }
            CounterMsg::GetCount(reply) => {
                let _ = reply.send(self.count);
            }
        }
        Ok(())
    }
}
```

### Actor Lifecycle

```rust
use airssys_rt::prelude::*;

#[async_trait]
impl Actor for MyActor {
    type Message = MyMsg;
    type Error = std::io::Error;
    
    async fn pre_start<B: MessageBroker<Self::Message>>(
        &mut self,
        _ctx: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        println!("Actor starting...");
        // Initialize resources
        Ok(())
    }
    
    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        msg: Self::Message,
        _ctx: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        // Process message
        Ok(())
    }
    
    async fn post_stop<B: MessageBroker<Self::Message>>(
        &mut self,
        _ctx: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        println!("Actor stopping...");
        // Cleanup resources
        Ok(())
    }
}
```

## Supervision Examples

### Basic Supervisor

```rust
use airssys_rt::supervisor::*;
use airssys_rt::broker::InMemoryMessageBroker;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create message broker
    let broker = InMemoryMessageBroker::new();
    
    // Create supervisor with OneForOne strategy
    let supervisor = SupervisorBuilder::new()
        .with_strategy(RestartStrategy::OneForOne)
        .with_max_restarts(3, Duration::from_secs(60))
        .build();
    
    // Add child actors
    supervisor.add_child(
        "worker-1",
        Box::new(WorkerActor::new()),
        RestartPolicy::Permanent,
    ).await?;
    
    // Start supervision
    supervisor.start().await?;
    
    Ok(())
}
```

### Supervision Strategies

```rust
// OneForOne: Restart only failed child
let supervisor = SupervisorBuilder::new()
    .with_strategy(RestartStrategy::OneForOne)
    .build();

// OneForAll: Restart all children when one fails
let supervisor = SupervisorBuilder::new()
    .with_strategy(RestartStrategy::OneForAll)
    .build();

// RestForOne: Restart failed child and those started after it
let supervisor = SupervisorBuilder::new()
    .with_strategy(RestartStrategy::RestForOne)
    .build();
```

### Restart Policies

```rust
// Permanent: Always restart
supervisor.add_child(
    "critical-worker",
    Box::new(CriticalActor::new()),
    RestartPolicy::Permanent,
).await?;

// Transient: Restart only if abnormal termination
supervisor.add_child(
    "task-worker",
    Box::new(TaskActor::new()),
    RestartPolicy::Transient,
).await?;

// Temporary: Never restart
supervisor.add_child(
    "one-shot",
    Box::new(OneShotActor::new()),
    RestartPolicy::Temporary,
).await?;
```

## Message Passing Examples

### Request-Reply Pattern

```rust
#[derive(Debug, Clone)]
enum QueryMsg {
    GetData(tokio::sync::oneshot::Sender<Data>),
}

impl Message for QueryMsg {
    const MESSAGE_TYPE: &'static str = "query";
}

// Send request
let (tx, rx) = tokio::sync::oneshot::channel();
broker.publish(QueryMsg::GetData(tx), address).await?;

// Wait for reply
let data = rx.await?;
```

### Pub-Sub Pattern

```rust
// Subscriber 1
broker.subscribe("events", subscriber1_address).await?;

// Subscriber 2
broker.subscribe("events", subscriber2_address).await?;

// Publisher
broker.publish_to_topic("events", Event::new()).await?;

// Both subscribers receive the message
```

## Integration Examples

### OSL Integration

```rust
use airssys_rt::supervisor::OSLSupervisor;
use airssys_rt::broker::InMemoryMessageBroker;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create broker
    let broker = InMemoryMessageBroker::new();
    
    // Create OSL supervisor
    let supervisor = OSLSupervisor::new(broker.clone());
    supervisor.start().await?;
    
    // Now FileSystem, Process, Network actors are running
    // with full fault tolerance
    
    println!("OSL actors under supervision");
    
    // Keep running
    tokio::time::sleep(Duration::from_secs(3600)).await;
    
    Ok(())
}
```

## Performance Examples

### High-Throughput Message Processing

```rust
use airssys_rt::prelude::*;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let broker = InMemoryMessageBroker::new();
    
    // Spawn actor
    let actor = HighThroughputActor::new();
    let address = ActorAddress::new("throughput-test");
    spawn_actor(actor, address.clone(), broker.clone()).await?;
    
    // Send 1M messages
    let start = Instant::now();
    for _ in 0..1_000_000 {
        broker.publish(ProcessMsg::new(), address.clone()).await?;
    }
    
    let duration = start.elapsed();
    let msgs_per_sec = 1_000_000.0 / duration.as_secs_f64();
    
    println!("Throughput: {:.2} msgs/sec", msgs_per_sec);
    
    Ok(())
}
```

## Complete Examples

For complete, runnable examples, see the repository:

```bash
# Basic actor implementation
cargo run --example actor_basic

# Actor lifecycle hooks
cargo run --example actor_lifecycle

# Actor patterns (request-reply, pub-sub)
cargo run --example actor_patterns

# Basic supervision
cargo run --example supervisor_basic

# Supervision strategies
cargo run --example supervisor_strategies

# Advanced supervision with health checks
cargo run --example supervisor_advanced

# OSL integration
cargo run --example osl_integration_example

# Worker pool pattern
cargo run --example worker_pool

# Message passing patterns
cargo run --example message_patterns
```

## Next Steps

- [RT API Reference](../components/rt/api/index.md)
- [RT Architecture](../components/rt/architecture/core-concepts.md)
- [OSL Examples](osl-examples.md)
- [Integration Guide](../guides/integration.md)
