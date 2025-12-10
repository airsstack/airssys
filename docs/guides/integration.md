# Integration Guide

This guide demonstrates how to integrate AirsSys components effectively.

## Overview

AirsSys components can be used independently or combined for powerful system programming solutions:

- **OSL alone**: Secure OS operations
- **RT alone**: Actor-based concurrency
- **OSL + RT**: Supervised secure operations

## Pattern 1: OSL Operations in RT Actors

Wrap OSL helper functions in RT actors:

```rust
use airssys_rt::prelude::*;
use airssys_osl::helpers::*;
use async_trait::async_trait;

#[derive(Debug, Clone)]
enum FileServiceMsg {
    ReadFile(String, tokio::sync::oneshot::Sender<Vec<u8>>),
    WriteFile(String, Vec<u8>),
}

impl Message for FileServiceMsg {
    const MESSAGE_TYPE: &'static str = "file-service";
}

struct FileServiceActor {
    principal: String,
}

#[async_trait]
impl Actor for FileServiceActor {
    type Message = FileServiceMsg;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    
    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        msg: Self::Message,
        _ctx: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        match msg {
            FileServiceMsg::ReadFile(path, reply) => {
                // Use OSL with security
                let content = read_file(&path, &self.principal).await?;
                let _ = reply.send(content);
            }
            FileServiceMsg::WriteFile(path, data) => {
                write_file(&path, data, &self.principal).await?;
            }
        }
        Ok(())
    }
}
```

**Benefits**:
- OSL security policies automatically enforced
- RT fault tolerance for file operations
- Actor isolation prevents shared state issues

## Pattern 2: Using OSLSupervisor

The built-in `OSLSupervisor` manages filesystem, process, and network actors:

```rust
use airssys_rt::supervisor::OSLSupervisor;
use airssys_rt::broker::InMemoryMessageBroker;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create shared broker
    let broker = InMemoryMessageBroker::new();
    
    // Create OSL supervisor
    let supervisor = OSLSupervisor::new(broker.clone());
    
    // Start all OSL actors with supervision
    supervisor.start().await?;
    
    // Actors are now running:
    // - FileSystemActor at "osl-filesystem"
    // - ProcessActor at "osl-process"
    // - NetworkActor at "osl-network"
    
    // They automatically restart on failure (RestForOne strategy)
    
    println!("OSL system running with supervision");
    
    // Keep running
    tokio::signal::ctrl_c().await?;
    
    Ok(())
}
```

**Architecture**:
```
OSLSupervisor (RestForOne)
     ├─→ FileSystemActor
     ├─→ ProcessActor
     └─→ NetworkActor
```

## Pattern 3: Custom Security in RT Actors

Use OSL middleware directly in RT actors:

```rust
use airssys_rt::prelude::*;
use airssys_osl::middleware::security::*;

struct SecureActor {
    security: SecurityMiddleware,
}

#[async_trait]
impl Actor for SecureActor {
    type Message = SecureMsg;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    
    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        msg: Self::Message,
        _ctx: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        // Check security before processing
        self.security.check_access(&msg.resource, &msg.principal)?;
        
        // Process message
        // ...
        
        Ok(())
    }
}
```

## Pattern 4: Hierarchical Supervision with OSL

Build supervision trees with OSL operations at leaves:

```rust
use airssys_rt::supervisor::*;

// Root supervisor
let root = SupervisorBuilder::new()
    .with_strategy(RestartStrategy::OneForOne)
    .build();

// Add OSL supervisor as child
let osl_supervisor = OSLSupervisor::new(broker.clone());
root.add_child("osl", Box::new(osl_supervisor), RestartPolicy::Permanent).await?;

// Add application supervisor as child
let app_supervisor = AppSupervisor::new(broker.clone());
root.add_child("app", Box::new(app_supervisor), RestartPolicy::Permanent).await?;

// Start entire tree
root.start().await?;
```

**Architecture**:
```
RootSupervisor
  ├─→ OSLSupervisor
  │     ├─→ FileSystemActor
  │     ├─→ ProcessActor
  │     └─→ NetworkActor
  └─→ AppSupervisor
        ├─→ BusinessActor1
        └─→ BusinessActor2
```

## Pattern 5: Message-Based OS Operations

Define operation messages for OSL actors:

```rust
#[derive(Debug, Clone)]
enum OSLMessage {
    FileRead { path: String, reply: Sender<Vec<u8>> },
    FileWrite { path: String, data: Vec<u8> },
    ProcessSpawn { cmd: String, args: Vec<String> },
}

impl Message for OSLMessage {
    const MESSAGE_TYPE: &'static str = "osl";
}

// Send message to OSL actor
broker.publish(
    OSLMessage::FileRead {
        path: "/tmp/test.txt".to_string(),
        reply: tx,
    },
    ActorAddress::new("osl-filesystem"),
).await?;
```

## Integration Checklist

### For OSL + RT Integration

- [ ] Create shared message broker
- [ ] Define message types for OS operations
- [ ] Create OSL actors or use OSLSupervisor
- [ ] Configure security policies
- [ ] Set up supervision strategies
- [ ] Add monitoring/logging
- [ ] Test failure recovery

### Security Considerations

- [ ] OSL security policies configured
- [ ] Principals assigned to actors
- [ ] Audit logging enabled
- [ ] Actor isolation verified
- [ ] No shared mutable state

### Performance Considerations

- [ ] Mailbox sizes tuned
- [ ] Backpressure strategy chosen
- [ ] Message batching where appropriate
- [ ] Monitoring metrics collected

## Common Patterns

### Request-Reply with OSL

```rust
// Actor receives request
async fn handle_message(&mut self, msg: Msg) {
    match msg {
        Msg::GetFile(path, reply) => {
            // Use OSL
            let content = read_file(&path, &self.principal).await?;
            // Send reply
            let _ = reply.send(content);
        }
    }
}
```

### Fire-and-Forget with OSL

```rust
async fn handle_message(&mut self, msg: Msg) {
    match msg {
        Msg::LogEvent(event) => {
            // Use OSL asynchronously
            write_file(&log_path, event.as_bytes(), "logger").await?;
            // No reply needed
        }
    }
}
```

### Streaming with OSL

```rust
async fn handle_message(&mut self, msg: Msg) {
    match msg {
        Msg::StreamFile(path) => {
            // Read in chunks
            let mut offset = 0;
            loop {
                let chunk = read_file_chunk(&path, offset, 4096).await?;
                if chunk.is_empty() {
                    break;
                }
                // Process chunk
                offset += chunk.len();
            }
        }
    }
}
```

## Testing Integration

### Unit Testing

Test actors independently:

```rust
#[tokio::test]
async fn test_file_service_actor() {
    let broker = InMemoryMessageBroker::new();
    let actor = FileServiceActor::new("test-user");
    // Test actor behavior
}
```

### Integration Testing

Test complete system:

```rust
#[tokio::test]
async fn test_osl_supervision() {
    let broker = InMemoryMessageBroker::new();
    let supervisor = OSLSupervisor::new(broker.clone());
    supervisor.start().await.unwrap();
    
    // Test operations work
    // Test failure recovery
}
```

## Best Practices

1. **Use OSLSupervisor** for standard OS operations
2. **Define clear message types** for operations
3. **Configure security policies** before starting actors
4. **Enable monitoring** for production systems
5. **Test failure scenarios** to verify recovery
6. **Document supervision trees** for maintenance

## Next Steps

- [Security Guide](security.md)
- [Performance Guide](performance.md)
- [Examples](../examples/index.md)
