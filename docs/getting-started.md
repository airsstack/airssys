# Getting Started with AirsSys

This guide will help you get started with AirsSys components.

## Prerequisites

- **Rust**: 2021 edition or later (install from [rustup.rs](https://rustup.rs))
- **Tokio**: Async runtime (included as dependency)
- **Operating System**: Linux, macOS, or Windows

## Installation

Add AirsSys components to your `Cargo.toml`:

### For OSL (OS Layer)

```toml
[dependencies]
airssys-osl = { version = "0.1", features = ["macros"] }
tokio = { version = "1.0", features = ["full"] }
```

### For RT (Actor Runtime)

```toml
[dependencies]
airssys-rt = "0.1.0"
async-trait = "0.1"
tokio = { version = "1.47", features = ["full"] }
```

### For Both

```toml
[dependencies]
airssys-osl = { version = "0.1", features = ["macros"] }
airssys-rt = "0.1.0"
async-trait = "0.1"
tokio = { version = "1.47", features = ["full"] }
```

## Your First OSL Application

Create a new project and add the following to `src/main.rs`:

```rust
use airssys_osl::helpers::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Write a file with built-in security
    let message = b"Hello from AirsSys OSL!".to_vec();
    write_file("/tmp/airssys_test.txt", message, "admin").await?;
    println!("✓ File written successfully");
    
    // Read the file back
    let content = read_file("/tmp/airssys_test.txt", "admin").await?;
    println!("✓ File content: {}", String::from_utf8_lossy(&content));
    
    // Clean up
    delete_file("/tmp/airssys_test.txt", "admin").await?;
    println!("✓ File deleted successfully");
    
    Ok(())
}
```

Run it:
```bash
cargo run
```

## Your First RT Application

Create a new project and add the following to `src/main.rs`:

```rust
use airssys_rt::prelude::*;
use async_trait::async_trait;

// 1. Define your message type
#[derive(Debug, Clone)]
enum CounterMsg {
    Increment,
    GetCount(tokio::sync::oneshot::Sender<u64>),
}

impl Message for CounterMsg {
    const MESSAGE_TYPE: &'static str = "counter";
}

// 2. Define your actor
struct CounterActor {
    count: u64,
}

// 3. Implement the Actor trait
#[async_trait]
impl Actor for CounterActor {
    type Message = CounterMsg;
    type Error = std::io::Error;
    
    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        msg: Self::Message,
        ctx: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        match msg {
            CounterMsg::Increment => {
                self.count += 1;
                println!("Count incremented to: {}", self.count);
            }
            CounterMsg::GetCount(reply) => {
                let _ = reply.send(self.count);
            }
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create message broker
    let broker = InMemoryMessageBroker::<CounterMsg>::new();
    
    // Create and start actor
    let actor = CounterActor { count: 0 };
    let address = ActorAddress::new("counter-1");
    
    // Spawn actor
    spawn_actor(actor, address.clone(), broker.clone()).await?;
    
    // Send messages
    broker.publish(CounterMsg::Increment, address.clone()).await?;
    broker.publish(CounterMsg::Increment, address.clone()).await?;
    
    // Query count
    let (tx, rx) = tokio::sync::oneshot::channel();
    broker.publish(CounterMsg::GetCount(tx), address).await?;
    
    let count = rx.await?;
    println!("✓ Final count: {}", count);
    
    Ok(())
}
```

Run it:
```bash
cargo run
```

## Integrated Application

Combine OSL and RT for a complete system:

```rust
use airssys_osl::helpers::*;
use airssys_rt::prelude::*;
use airssys_rt::supervisor::OSLSupervisor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create message broker
    let broker = InMemoryMessageBroker::new();
    
    // Create OSL supervisor with RT
    let supervisor = OSLSupervisor::new(broker.clone());
    supervisor.start().await?;
    
    // Now FileSystem, Process, and Network actors are running
    // with fault tolerance from the supervisor
    
    println!("✓ AirsSys integrated system running");
    
    // Use OSL operations with RT supervision
    let content = read_file("/etc/hosts", "admin").await?;
    println!("✓ Read {} bytes with supervision", content.len());
    
    Ok(())
}
```

## Development Workflow

### Project Structure

Recommended project structure:

```
my-airssys-app/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── actors/          # RT actors
│   │   ├── mod.rs
│   │   └── my_actor.rs
│   └── operations/      # OSL operations
│       ├── mod.rs
│       └── my_ops.rs
├── examples/            # Usage examples
└── tests/               # Integration tests
```

### Running Examples

AirsSys includes comprehensive examples:

```bash
# OSL examples
cargo run --example helper_functions_comprehensive
cargo run --example security_middleware_comprehensive
cargo run --example custom_executor_with_macro --features macros

# RT examples
cargo run --example actor_basic
cargo run --example supervisor_basic
cargo run --example osl_integration_example
```

### Testing

```bash
# Run all tests
cargo test

# Run specific component tests
cargo test --package airssys-osl
cargo test --package airssys-rt

# Run with verbose output
cargo test -- --nocapture
```

### Building Documentation

```bash
# Build API documentation
cargo doc --open

# View component examples
ls examples/
```

## Configuration

### OSL Configuration

Configure security policies:

```rust
use airssys_osl::middleware::security::*;

// Create ACL policy
let acl = AccessControlList::new()
    .add_entry(AclEntry::new(
        "alice".to_string(),
        "/data/*".to_string(),
        vec!["read".to_string(), "write".to_string()],
        AclPolicy::Allow,
    ));

// Build security middleware
let security = SecurityMiddlewareBuilder::new()
    .add_policy(Box::new(acl))
    .build()?;
```

### RT Configuration

Configure actor system:

```rust
use airssys_rt::system::{ActorSystemConfig, ActorSystemBuilder};

let config = ActorSystemConfig {
    max_actors: 10000,
    default_mailbox_size: 1000,
    enable_monitoring: true,
};

let system = ActorSystemBuilder::new()
    .with_config(config)
    .build()?;
```

## Next Steps

Now that you have AirsSys running, explore:

1. **[OSL Documentation](../components/osl/index.md)** - Deep dive into secure OS operations
2. **[RT Documentation](../components/rt/index.md)** - Master actor-based concurrency
3. **[Examples](../examples/index.md)** - Learn from real-world patterns
4. **[Integration Guide](../guides/integration.md)** - Combine components effectively

## Troubleshooting

### Common Issues

**Issue**: `error: future cannot be sent between threads safely`

**Solution**: Ensure your actors implement `Send + Sync`:
```rust
#[async_trait]
impl Actor for MyActor {
    // Actor methods must be Send + Sync
}
```

**Issue**: OSL operation denied

**Solution**: Check security policies allow the operation:
```rust
// Operations denied by default
// Explicitly allow in ACL or RBAC policy
```

**Issue**: Actor not receiving messages

**Solution**: Verify broker and address are correctly configured:
```rust
// Ensure same broker instance and correct address
broker.publish(msg, address.clone()).await?;
```

### Getting Help

- **Examples**: Check `/examples` directory for working code
- **API Docs**: Run `cargo doc --open` for detailed API reference
- **Issues**: [Report bugs on GitHub](https://github.com/airsstack/airssys/issues)

## Performance Tips

### OSL Performance

- Use helper functions for simple operations
- Batch operations when possible
- Configure appropriate log levels

### RT Performance

- Tune mailbox sizes for workload
- Use appropriate supervisor strategies
- Monitor actor metrics

See [Performance Guide](../guides/performance.md) for detailed optimization strategies.
