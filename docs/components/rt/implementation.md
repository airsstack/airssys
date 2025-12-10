# Implementation Guide

This guide provides practical, step-by-step instructions for building applications with `airssys-rt`. All examples are based on the actual implementation and working code from the `examples/` directory.

## Quick Start

### Add Dependency

Add `airssys-rt` to your `Cargo.toml`:

```toml
[dependencies]
airssys-rt = { path = "../airssys-rt" }  # or version from crates.io when published
async-trait = "0.1"
tokio = { version = "1.47", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
```

### Your First Actor

Create a simple counter actor (from `examples/actor_basic.rs`):

```rust
use airssys_rt::{Actor, ActorContext, Message};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// 1. Define your message type
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CounterMessage {
    delta: i32,
}

impl Message for CounterMessage {
    const MESSAGE_TYPE: &'static str = "counter";
}

// 2. Define your actor
struct CounterActor {
    value: i32,
    max_value: i32,
}

// 3. Define error type
#[derive(Debug)]
struct CounterError {
    message: String,
}

impl std::fmt::Display for CounterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CounterError: {}", self.message)
    }
}

impl std::error::Error for CounterError {}

// 4. Implement the Actor trait
#[async_trait]
impl Actor for CounterActor {
    type Message = CounterMessage;
    type Error = CounterError;

    async fn handle_message<B: airssys_rt::broker::MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        self.value += message.delta;

        if self.value > self.max_value {
            return Err(CounterError {
                message: format!("Value {} exceeds maximum {}", 
                    self.value, self.max_value),
            });
        }

        context.record_message();
        Ok(())
    }
}
```

Run the complete example:
```bash
cargo run --example actor_basic
```

## Actor Lifecycle Hooks

Actors can override lifecycle hooks for initialization and cleanup (from `examples/actor_lifecycle.rs`):

```rust
#[async_trait]
impl Actor for CounterActor {
    type Message = CounterMessage;
    type Error = CounterError;

    // Called before actor starts processing messages
    async fn pre_start<B: MessageBroker<Self::Message>>(
        &mut self,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        println!(
            "[Actor {}] Starting with initial value: {}",
            context.address().name().unwrap_or("anonymous"),
            self.value
        );
        // Initialize resources here (e.g., database connections)
        Ok(())
    }

    // Called when actor stops
    async fn post_stop<B: MessageBroker<Self::Message>>(
        &mut self,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        println!(
            "[Actor {}] Stopping with final value: {} (processed {} messages)",
            context.address().name().unwrap_or("anonymous"),
            self.value,
            context.message_count()
        );
        // Cleanup resources here (e.g., close connections)
        Ok(())
    }

    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        // Message handling logic
        self.value += message.delta;
        context.record_message();
        Ok(())
    }
}
```

Run the lifecycle example:
```bash
cargo run --example actor_lifecycle
```

## Error Handling and Supervision

### ErrorAction for Fault Tolerance

Actors return `ErrorAction` from `on_error` to control supervision behavior:

```rust
use airssys_rt::ErrorAction;

#[async_trait]
impl Actor for CounterActor {
    type Message = CounterMessage;
    type Error = CounterError;

    async fn on_error<B: MessageBroker<Self::Message>>(
        &mut self,
        error: Self::Error,
        context: &mut ActorContext<Self::Message, B>,
    ) -> ErrorAction {
        eprintln!("[Actor {}] Error: {}", 
            context.address().name().unwrap_or("anonymous"), 
            error);
        
        // Supervisor will restart this actor
        ErrorAction::Restart
    }

    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        self.value += message.delta;

        if self.value > self.max_value {
            return Err(CounterError {
                message: format!("Value {} exceeds maximum", self.value),
            });
        }

        Ok(())
    }
}
```

### Building Supervisor Trees

Create supervisors to manage child actors (from `examples/supervisor_basic.rs`):

```rust
use airssys_rt::supervisor::{
    Child, ChildHealth, ChildSpec, RestartPolicy, 
    ShutdownPolicy, SupervisorNode, OneForOne
};
use async_trait::async_trait;

// 1. Define a worker that implements Child
struct SimpleWorker {
    id: String,
}

#[async_trait]
impl Child for SimpleWorker {
    async fn start(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        println!("Worker {} started", self.id);
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        println!("Worker {} stopped", self.id);
        Ok(())
    }

    async fn health_check(&self) -> ChildHealth {
        ChildHealth::Healthy
    }
}

// 2. Create supervisor and add children
async fn create_supervisor() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut supervisor = SupervisorNode::new(
        SupervisorId::new(),
        OneForOne::new(),  // Restart strategy
    );

    // Add a child worker
    supervisor.add_child(
        ChildSpec {
            id: ChildId::new(),
            restart_policy: RestartPolicy::Permanent,
            shutdown_policy: ShutdownPolicy::default(),
            significant: true,
        },
        Box::new(SimpleWorker {
            id: "worker-1".to_string(),
        }),
    ).await?;

    // Start all children
    supervisor.start_all_children().await?;

    Ok(())
}
```

Run the supervisor example:
```bash
cargo run --example supervisor_basic
```

### Restart Strategies

Choose the appropriate strategy for your use case (from `examples/supervisor_strategies.rs`):

**OneForOne** - Restart only the failed child:
```rust
use airssys_rt::supervisor::OneForOne;

let supervisor = SupervisorNode::new(
    SupervisorId::new(),
    OneForOne::new(),
);
```

**OneForAll** - Restart all children when one fails:
```rust
use airssys_rt::supervisor::OneForAll;

let supervisor = SupervisorNode::new(
    SupervisorId::new(),
    OneForAll::new(),
);
```

**RestForOne** - Restart failed child and those started after it:
```rust
use airssys_rt::supervisor::RestForOne;

let supervisor = SupervisorNode::new(
    SupervisorId::new(),
    RestForOne::new(),
);
```

Run the strategies example:
```bash
cargo run --example supervisor_strategies
```

## Actor Monitoring

### Health Checks

Implement health checking for supervised actors (from `examples/supervisor_automatic_health.rs`):

```rust
use airssys_rt::supervisor::{Child, ChildHealth, HealthConfig};

struct MonitoredWorker {
    is_healthy: bool,
}

#[async_trait]
impl Child for MonitoredWorker {
    async fn start(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.is_healthy = true;
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.is_healthy = false;
        Ok(())
    }

    async fn health_check(&self) -> ChildHealth {
        if self.is_healthy {
            ChildHealth::Healthy
        } else {
            ChildHealth::Unhealthy("Worker is unhealthy".to_string())
        }
    }
}

// Configure automatic health monitoring
let health_config = HealthConfig {
    check_interval: Duration::from_secs(5),
    unhealthy_threshold: 3,
    restart_on_unhealthy: true,
};
```

Run the health monitoring example:
```bash
cargo run --example supervisor_automatic_health
```

### Monitoring System

Use the monitoring system to track actor metrics (from `examples/monitoring_basic.rs` and `examples/monitoring_supervisor.rs`):

```rust
use airssys_rt::monitoring::{ActorMonitor, MonitoringConfig};

// Monitor individual actors
let monitor = ActorMonitor::new(MonitoringConfig::default());

// Monitor supervisors
cargo run --example monitoring_basic
cargo run --example monitoring_supervisor
```

## Integration with AirsSys OSL

Integrate with `airssys-osl` for secure system operations (from `examples/osl_integration_example.rs`):

```rust
use airssys_osl::prelude::*;
use airssys_rt::{Actor, ActorContext, Message};

#[derive(Debug, Clone, Serialize, Deserialize)]
enum FileMessage {
    ReadFile { path: String },
    WriteFile { path: String, content: String },
}

impl Message for FileMessage {
    const MESSAGE_TYPE: &'static str = "file_operation";
}

struct FileActor {
    executor: OslExecutor,
}

#[async_trait]
impl Actor for FileActor {
    type Message = FileMessage;
    type Error = FileError;

    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        match message {
            FileMessage::ReadFile { path } => {
                // Use OSL for secure file operations
                let content = self.executor.read_file(&path)?;
                println!("Read {} bytes from {}", content.len(), path);
                Ok(())
            }
            FileMessage::WriteFile { path, content } => {
                self.executor.write_file(&path, content.as_bytes())?;
                println!("Wrote to {}", path);
                Ok(())
            }
        }
    }
}
```

Run the OSL integration example:
```bash
cargo run --example osl_integration_example
```

## Complete Examples Reference

All working examples are in the `examples/` directory:

| Example | Description | Run Command |
|---------|-------------|-------------|
| `actor_basic.rs` | Basic actor implementation | `cargo run --example actor_basic` |
| `actor_lifecycle.rs` | Lifecycle hooks (pre_start, post_stop) | `cargo run --example actor_lifecycle` |
| `supervisor_basic.rs` | Basic supervision patterns | `cargo run --example supervisor_basic` |
| `supervisor_strategies.rs` | Restart strategies comparison | `cargo run --example supervisor_strategies` |
| `supervisor_automatic_health.rs` | Automatic health monitoring | `cargo run --example supervisor_automatic_health` |
| `monitoring_basic.rs` | Actor monitoring basics | `cargo run --example monitoring_basic` |
| `monitoring_supervisor.rs` | Supervisor monitoring | `cargo run --example monitoring_supervisor` |
| `osl_integration_example.rs` | OSL integration for file operations | `cargo run --example osl_integration_example` |

## Next Steps

1. **Explore Examples**: Run each example to see the runtime in action
2. **Build Your Actor**: Start with `actor_basic.rs` as a template
3. **Add Supervision**: Use `supervisor_basic.rs` for fault tolerance
4. **Monitor Health**: Implement health checks for production readiness
5. **Integrate OSL**: Use `osl_integration_example.rs` for system operations

All examples demonstrate real, production-ready patterns using the actual `airssys-rt` implementation.