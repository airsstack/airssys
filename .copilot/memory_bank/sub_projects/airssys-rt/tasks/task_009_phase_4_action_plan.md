# RT-TASK-009 Phase 4: Documentation and Examples - Action Plan

**Created:** 2025-10-14  
**Status:** Ready for Implementation  
**Phase:** Phase 4 of 4 (OSL Integration - Final Phase)  
**Prerequisites:** Phase 3 Complete (Security & Audit)  
**Duration Estimate:** 2 days (16 hours)

---

## Executive Summary

**Objective:** Provide comprehensive documentation, working examples, migration guides, and performance validation to enable developers to effectively use OSL integration actors in production systems.

**Key Deliverables:**
1. Four comprehensive usage examples demonstrating all patterns
2. Migration guide from direct OSL helpers to actor-based approach
3. Performance benchmarks validating <1% overhead target
4. Complete mdBook chapter on OSL integration

**Success Criteria:**
- ✅ 4+ working examples covering all use cases
- ✅ Migration guide enables smooth transition
- ✅ Performance benchmarks validate overhead targets
- ✅ mdBook documentation comprehensive and accurate

---

## Phase 4 Overview

### Goals
- Demonstrate OSL integration patterns through working examples
- Enable migration from direct OSL helpers to actor-based architecture
- Validate and document performance characteristics
- Provide complete technical documentation in mdBook

### Success Criteria
- ✅ All examples compile and run successfully
- ✅ Migration guide tested with real code transitions
- ✅ Performance overhead <1% vs direct OSL calls
- ✅ mdBook chapter explains all concepts clearly
- ✅ Code examples follow workspace standards (§2.1-§6.3)

---

## Task Breakdown

### Task 4.1: Comprehensive Examples (Day 9, ~8 hours)

#### Example 1: FileSystem Actor Usage (2 hours)
**File:** `examples/filesystem_actor_usage.rs` (~150-200 lines)

**Purpose:** Demonstrate all FileSystemActor capabilities and patterns

**Content:**

```rust
//! FileSystem Actor Usage Example
//! 
//! Demonstrates:
//! - FileSystem actor setup and configuration
//! - All file operations (read, write, list, delete)
//! - Security context integration
//! - Error handling patterns
//! - Async/await with broker messaging

use airssys_rt::prelude::*;
use airssys_rt::osl::{
    FileSystemActor, FileSystemOperation, FileSystemRequest, FileSystemResponse,
    OSLMessage, SecurityContext, Permission,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== FileSystem Actor Usage Example ===\n");

    // 1. Setup: Create broker and actor
    let broker = Arc::new(InMemoryMessageBroker::<OSLMessage>::new());
    let mut fs_actor = FileSystemActor::new(Arc::new(ConsoleAuditLogger));
    
    // Subscribe actor to broker
    broker.subscribe("osl-filesystem", Arc::new(fs_actor)).await;
    
    // 2. Create security context
    let security_ctx = SecurityContext::new(Principal {
        user_id: "user123".to_string(),
        user_name: "Alice".to_string(),
        roles: vec!["developer".to_string()],
    })
    .with_permission(Permission::new("file:read", "/tmp/*"))
    .with_permission(Permission::new("file:write", "/tmp/*"));
    
    // 3. Example: Read file
    println!("--- Example 1: Read File ---");
    let read_request = FileSystemRequest {
        operation: FileSystemOperation::ReadFile {
            path: PathBuf::from("/tmp/example.txt"),
        },
        request_id: MessageId::new(),
        security_context: Some(security_ctx.clone()),
    };
    
    broker.publish(OSLMessage::FileSystem(read_request)).await?;
    
    // Wait for response (simplified - real code would use proper response handling)
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // 4. Example: Write file
    println!("\n--- Example 2: Write File ---");
    let write_request = FileSystemRequest {
        operation: FileSystemOperation::WriteFile {
            path: PathBuf::from("/tmp/output.txt"),
            content: b"Hello from FileSystemActor!".to_vec(),
        },
        request_id: MessageId::new(),
        security_context: Some(security_ctx.clone()),
    };
    
    broker.publish(OSLMessage::FileSystem(write_request)).await?;
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // 5. Example: List directory
    println!("\n--- Example 3: List Directory ---");
    let list_request = FileSystemRequest {
        operation: FileSystemOperation::ListDirectory {
            path: PathBuf::from("/tmp"),
        },
        request_id: MessageId::new(),
        security_context: Some(security_ctx.clone()),
    };
    
    broker.publish(OSLMessage::FileSystem(list_request)).await?;
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // 6. Example: Error handling - Permission denied
    println!("\n--- Example 4: Permission Denied ---");
    let denied_request = FileSystemRequest {
        operation: FileSystemOperation::ReadFile {
            path: PathBuf::from("/etc/passwd"),  // Outside allowed paths
        },
        request_id: MessageId::new(),
        security_context: Some(security_ctx),
    };
    
    broker.publish(OSLMessage::FileSystem(denied_request)).await?;
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    println!("\n=== Example Complete ===");
    Ok(())
}
```

**Acceptance Criteria:**
- ✅ Demonstrates all FileSystemOperation variants
- ✅ Shows security context usage
- ✅ Includes error handling examples
- ✅ Complete with comments explaining patterns
- ✅ Compiles and runs successfully

#### Example 2: Process Actor Usage (2 hours)
**File:** `examples/process_actor_usage.rs` (~150-200 lines)

**Purpose:** Demonstrate process spawning and management patterns

**Content:**

```rust
//! Process Actor Usage Example
//! 
//! Demonstrates:
//! - Process spawning with arguments
//! - Output capture and streaming
//! - Process lifecycle tracking
//! - Security context for process execution
//! - Process termination and cleanup

use airssys_rt::prelude::*;
use airssys_rt::osl::{
    ProcessActor, ProcessOperation, ProcessRequest, ProcessResponse,
    OSLMessage, SecurityContext, Permission,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Process Actor Usage Example ===\n");

    // 1. Setup
    let broker = Arc::new(InMemoryMessageBroker::<OSLMessage>::new());
    let mut process_actor = ProcessActor::new(Arc::new(ConsoleAuditLogger));
    
    broker.subscribe("osl-process", Arc::new(process_actor)).await;
    
    // 2. Create security context with process:spawn permission
    let security_ctx = SecurityContext::new(Principal {
        user_id: "admin".to_string(),
        user_name: "Bob".to_string(),
        roles: vec!["admin".to_string()],
    })
    .with_permission(Permission::new("process:spawn", "/usr/bin/*"));
    
    // 3. Example: Spawn simple process
    println!("--- Example 1: Spawn Process ---");
    let spawn_request = ProcessRequest {
        operation: ProcessOperation::Spawn {
            command: "/usr/bin/echo".to_string(),
            args: vec!["Hello", "from", "ProcessActor"].iter().map(|s| s.to_string()).collect(),
            env: HashMap::new(),
            working_dir: None,
        },
        request_id: MessageId::new(),
        security_context: Some(security_ctx.clone()),
    };
    
    broker.publish(OSLMessage::Process(spawn_request)).await?;
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    // 4. Example: Spawn with output capture
    println!("\n--- Example 2: Capture Output ---");
    let capture_request = ProcessRequest {
        operation: ProcessOperation::SpawnWithOutput {
            command: "/usr/bin/ls".to_string(),
            args: vec!["-la".to_string()],
            env: HashMap::new(),
        },
        request_id: MessageId::new(),
        security_context: Some(security_ctx.clone()),
    };
    
    broker.publish(OSLMessage::Process(capture_request)).await?;
    tokio::time::sleep(Duration::from_millis(300)).await;
    
    // 5. Example: Long-running process with termination
    println!("\n--- Example 3: Process Lifecycle ---");
    let long_running = ProcessRequest {
        operation: ProcessOperation::Spawn {
            command: "/usr/bin/sleep".to_string(),
            args: vec!["30".to_string()],
            env: HashMap::new(),
            working_dir: None,
        },
        request_id: MessageId::new(),
        security_context: Some(security_ctx.clone()),
    };
    
    broker.publish(OSLMessage::Process(long_running)).await?;
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Terminate the process
    let terminate_request = ProcessRequest {
        operation: ProcessOperation::Terminate {
            pid: 12345,  // Would be actual PID from spawn response
        },
        request_id: MessageId::new(),
        security_context: Some(security_ctx),
    };
    
    broker.publish(OSLMessage::Process(terminate_request)).await?;
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    println!("\n=== Example Complete ===");
    Ok(())
}
```

**Acceptance Criteria:**
- ✅ Demonstrates process spawning patterns
- ✅ Shows output capture
- ✅ Includes process termination
- ✅ Security context for process operations
- ✅ Compiles and runs successfully

#### Example 3: Network Actor Usage (2 hours)
**File:** `examples/network_actor_usage.rs` (~150-200 lines)

**Purpose:** Demonstrate network operations and connection pooling

**Content:**

```rust
//! Network Actor Usage Example
//! 
//! Demonstrates:
//! - TCP connection establishment
//! - HTTP request patterns
//! - Connection pooling usage
//! - Network operation timeouts
//! - Security context for network operations
//! - Error handling and retries

use airssys_rt::prelude::*;
use airssys_rt::osl::{
    NetworkActor, NetworkOperation, NetworkRequest, NetworkResponse,
    OSLMessage, SecurityContext, Permission,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Network Actor Usage Example ===\n");

    // 1. Setup
    let broker = Arc::new(InMemoryMessageBroker::<OSLMessage>::new());
    let mut network_actor = NetworkActor::new(Arc::new(ConsoleAuditLogger));
    
    broker.subscribe("osl-network", Arc::new(network_actor)).await;
    
    // 2. Create security context
    let security_ctx = SecurityContext::new(Principal {
        user_id: "service".to_string(),
        user_name: "APIService".to_string(),
        roles: vec!["service".to_string()],
    })
    .with_permission(Permission::new("network:connect", "httpbin.org:443"))
    .with_permission(Permission::new("network:connect", "localhost:*"));
    
    // 3. Example: TCP connection
    println!("--- Example 1: TCP Connect ---");
    let tcp_request = NetworkRequest {
        operation: NetworkOperation::TcpConnect {
            host: "localhost".to_string(),
            port: 8080,
            timeout_ms: Some(5000),
        },
        request_id: MessageId::new(),
        security_context: Some(security_ctx.clone()),
    };
    
    broker.publish(OSLMessage::Network(tcp_request)).await?;
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // 4. Example: HTTP request
    println!("\n--- Example 2: HTTP Request ---");
    let http_request = NetworkRequest {
        operation: NetworkOperation::HttpRequest {
            url: "https://httpbin.org/get".to_string(),
            method: "GET".to_string(),
            headers: HashMap::new(),
            body: None,
        },
        request_id: MessageId::new(),
        security_context: Some(security_ctx.clone()),
    };
    
    broker.publish(OSLMessage::Network(http_request)).await?;
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // 5. Example: Connection with timeout
    println!("\n--- Example 3: Timeout Handling ---");
    let timeout_request = NetworkRequest {
        operation: NetworkOperation::TcpConnect {
            host: "10.255.255.1".to_string(),  // Non-routable IP
            port: 9999,
            timeout_ms: Some(100),  // Short timeout
        },
        request_id: MessageId::new(),
        security_context: Some(security_ctx),
    };
    
    broker.publish(OSLMessage::Network(timeout_request)).await?;
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    println!("\n=== Example Complete ===");
    Ok(())
}
```

**Acceptance Criteria:**
- ✅ Demonstrates network operations
- ✅ Shows timeout handling
- ✅ Includes HTTP request patterns
- ✅ Security context for network ops
- ✅ Compiles and runs successfully

#### Example 4: Complete Supervisor Hierarchy (2 hours)
**File:** `examples/supervisor_hierarchy_complete.rs` (~250-300 lines)

**Purpose:** Demonstrate full production-ready supervisor setup

**Content:**

```rust
//! Complete Supervisor Hierarchy Example
//! 
//! Demonstrates:
//! - Full hierarchical supervisor setup
//! - RootSupervisor → OSLSupervisor + ApplicationSupervisor
//! - Cross-supervisor communication via broker
//! - Fault isolation and restart strategies
//! - Real-world application structure
//! - Security context flow across boundaries

use airssys_rt::prelude::*;
use airssys_rt::osl::*;

// Application actor example
struct WorkerActor {
    id: String,
}

#[async_trait]
impl<M, B> Actor<M, B> for WorkerActor
where
    M: Message,
    B: MessageBroker<M>,
{
    async fn handle_message(&mut self, message: M, context: &mut ActorContext<M, B>) {
        println!("[WorkerActor-{}] Received message", self.id);
        
        // Worker wants to read a file - send to FileSystemActor
        let security_ctx = SecurityContext::new(/* ... */);
        
        let fs_request = FileSystemRequest {
            operation: FileSystemOperation::ReadFile {
                path: PathBuf::from("/tmp/worker_data.txt"),
            },
            request_id: MessageId::new(),
            security_context: Some(security_ctx),
        };
        
        context.broker().publish(OSLMessage::FileSystem(fs_request)).await.ok();
    }
}

#[async_trait]
impl Child for WorkerActor {
    async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("[WorkerActor-{}] Started", self.id);
        Ok(())
    }
    
    async fn stop(&mut self, timeout: Duration) -> Result<(), Box<dyn std::error::Error>> {
        println!("[WorkerActor-{}] Stopped", self.id);
        Ok(())
    }
    
    async fn health_check(&self) -> HealthStatus {
        HealthStatus::Healthy
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Complete Supervisor Hierarchy Example ===\n");

    // 1. Create shared broker
    let broker = Arc::new(InMemoryMessageBroker::<OSLMessage>::new());
    
    // 2. Create OSL Supervisor with actors
    println!("--- Setting up OSL Supervisor ---");
    let osl_supervisor = OSLSupervisor::new(broker.clone())
        .with_audit_logger(Arc::new(ConsoleAuditLogger))
        .with_strategy(RestartStrategy::RestForOne);
    
    // 3. Create Application Supervisor
    println!("--- Setting up Application Supervisor ---");
    let mut app_supervisor = Supervisor::new(RestartStrategy::OneForOne);
    
    // Add worker actors
    for i in 0..3 {
        let worker = WorkerActor { id: format!("worker-{}", i) };
        app_supervisor.add_child(
            format!("app-worker-{}", i),
            Arc::new(worker),
            RestartStrategy::Permanent,
        ).await?;
    }
    
    // 4. Create Root Supervisor managing both
    println!("--- Setting up Root Supervisor ---");
    let mut root_supervisor = Supervisor::new(RestartStrategy::OneForAll);
    
    root_supervisor.add_child(
        "osl-supervisor",
        Arc::new(osl_supervisor),
        RestartStrategy::Permanent,
    ).await?;
    
    root_supervisor.add_child(
        "app-supervisor",
        Arc::new(app_supervisor),
        RestartStrategy::Permanent,
    ).await?;
    
    // 5. Start the entire hierarchy
    println!("\n--- Starting Supervisor Hierarchy ---");
    root_supervisor.start().await?;
    
    println!("\n✅ Supervisor hierarchy running!");
    println!("   RootSupervisor");
    println!("   ├── OSLSupervisor (RestForOne)");
    println!("   │   ├── FileSystemActor");
    println!("   │   ├── ProcessActor");
    println!("   │   └── NetworkActor");
    println!("   └── ApplicationSupervisor (OneForOne)");
    println!("       ├── WorkerActor-0");
    println!("       ├── WorkerActor-1");
    println!("       └── WorkerActor-2");
    
    // 6. Demonstrate cross-supervisor communication
    println!("\n--- Cross-Supervisor Communication ---");
    
    let security_ctx = SecurityContext::new(Principal {
        user_id: "system".to_string(),
        user_name: "System".to_string(),
        roles: vec!["system".to_string()],
    })
    .with_permission(Permission::new("file:read", "/tmp/*"));
    
    let request = FileSystemRequest {
        operation: FileSystemOperation::ReadFile {
            path: PathBuf::from("/tmp/test.txt"),
        },
        request_id: MessageId::new(),
        security_context: Some(security_ctx),
    };
    
    broker.publish(OSLMessage::FileSystem(request)).await?;
    println!("✅ Message sent from app context to OSL actor via broker");
    
    // 7. Let it run for a bit
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // 8. Demonstrate fault isolation
    println!("\n--- Fault Isolation Test ---");
    println!("Simulating OSL actor failure...");
    // (Would trigger actor failure here in real scenario)
    println!("✅ OSL actors restart, app actors unaffected (separate supervisors)");
    
    // 9. Graceful shutdown
    println!("\n--- Graceful Shutdown ---");
    root_supervisor.stop(Duration::from_secs(5)).await?;
    println!("✅ All actors stopped gracefully");
    
    println!("\n=== Example Complete ===");
    Ok(())
}
```

**Acceptance Criteria:**
- ✅ Complete hierarchy setup demonstrated
- ✅ Shows cross-supervisor communication
- ✅ Fault isolation explained
- ✅ Graceful shutdown pattern
- ✅ Production-ready structure
- ✅ Compiles and runs successfully

---

### Task 4.2: Migration Guide (Day 9, ~4 hours)

**File:** `MIGRATION.md` (~300-400 lines)

**Structure:**

```markdown
# Migration Guide: Direct OSL Helpers → OSL Integration Actors

## Overview

This guide helps you migrate from direct airssys-osl helper function usage to the actor-based OSL integration architecture.

## Why Migrate?

**Benefits of Actor-Based Approach:**
- ✅ **Fault Tolerance**: Supervisor trees provide automatic recovery
- ✅ **Testability**: Mock actors for unit testing
- ✅ **Isolation**: OS failures don't crash application
- ✅ **Centralized Management**: Single point of control for OS operations
- ✅ **Audit Trail**: Automatic logging of all operations
- ✅ **Security**: Integrated security context enforcement

## Before and After

### Before: Direct OSL Helpers
```rust
use airssys_osl::helpers;

async fn read_config() -> Result<String, std::io::Error> {
    helpers::read_file("/etc/config.toml").await
}
```

### After: OSL Integration Actors
```rust
use airssys_rt::osl::*;

async fn read_config(broker: &Arc<InMemoryMessageBroker<OSLMessage>>) 
    -> Result<String, Box<dyn std::error::Error>> 
{
    let request = FileSystemRequest {
        operation: FileSystemOperation::ReadFile {
            path: PathBuf::from("/etc/config.toml"),
        },
        request_id: MessageId::new(),
        security_context: Some(create_security_context()),
    };
    
    broker.publish(OSLMessage::FileSystem(request)).await?;
    
    // Handle response via subscription or correlation
    Ok(/* ... */)
}
```

## Step-by-Step Migration

### Step 1: Set Up Supervisor Hierarchy

### Step 2: Replace Direct Calls with Messages

### Step 3: Handle Responses

### Step 4: Add Security Context

### Step 5: Configure Audit Logging

### Step 6: Update Tests

## Common Patterns

### Pattern 1: Request-Response
### Pattern 2: Fire-and-Forget
### Pattern 3: Batch Operations
### Pattern 4: Error Handling

## Troubleshooting

### Issue: Response Not Received
### Issue: Permission Denied
### Issue: Actor Not Started

## Performance Considerations

## FAQ

```

**Acceptance Criteria:**
- ✅ Clear before/after comparisons
- ✅ Step-by-step instructions
- ✅ Common patterns documented
- ✅ Troubleshooting guide included
- ✅ Tested with actual migration

---

### Task 4.3: Performance Benchmarks (Day 10, ~4 hours)

**File:** `benches/osl_actor_benchmarks.rs` (~200-300 lines)

**Benchmark Categories:**

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use airssys_rt::osl::*;

fn benchmark_filesystem_operations(c: &mut Criterion) {
    // Benchmark 1: Direct OSL helper (baseline)
    c.bench_function("fs_direct_read_file", |b| {
        b.iter(|| {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                airssys_osl::helpers::read_file(black_box("/tmp/bench.txt")).await
            })
        });
    });
    
    // Benchmark 2: Actor message passing
    c.bench_function("fs_actor_read_file", |b| {
        b.iter(|| {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let broker = Arc::new(InMemoryMessageBroker::new());
                let actor = FileSystemActor::new(Arc::new(NullAuditLogger));
                
                let request = FileSystemRequest {
                    operation: FileSystemOperation::ReadFile {
                        path: PathBuf::from("/tmp/bench.txt"),
                    },
                    request_id: MessageId::new(),
                    security_context: None,
                };
                
                broker.publish(OSLMessage::FileSystem(request)).await
            })
        });
    });
}

fn benchmark_process_operations(c: &mut Criterion) {
    // Similar benchmarks for process spawning
}

fn benchmark_network_operations(c: &mut Criterion) {
    // Similar benchmarks for network connections
}

fn benchmark_message_overhead(c: &mut Criterion) {
    // Pure message serialization/deserialization overhead
}

criterion_group!(
    benches,
    benchmark_filesystem_operations,
    benchmark_process_operations,
    benchmark_network_operations,
    benchmark_message_overhead
);
criterion_main!(benches);
```

**Analysis and Reporting:**
- Compare direct vs actor-based approach
- Calculate percentage overhead
- Identify bottlenecks
- Document results in README

**Acceptance Criteria:**
- ✅ Comprehensive benchmarks for all operations
- ✅ Baseline comparison with direct OSL
- ✅ Overhead <1% for typical operations
- ✅ Results documented in README
- ✅ Performance report generated

---

### Task 4.4: mdBook Documentation (Day 10, ~4 hours)

**File:** `airssys-rt/docs/src/osl_integration.md` (~500-800 lines)

**Chapter Structure:**

```markdown
# OSL Integration

## Introduction

Overview of OSL integration architecture and benefits.

## Architecture

### Hierarchical Supervisor Pattern
- Diagram of RootSupervisor → OSLSupervisor + ApplicationSupervisor
- Explanation of fault isolation
- Restart strategy configuration

### OSL Integration Actors
- FileSystemActor responsibilities
- ProcessActor responsibilities
- NetworkActor responsibilities

### Message Protocol
- OSLMessage enum structure
- Request-response correlation
- Security context in messages

### Broker-Based Communication
- Pub-sub pattern explanation
- Cross-supervisor messaging
- Message isolation

## Security Integration

### Security Context Flow
- Application actors → OSL actors → OS operations
- Permission model
- Context validation

### Permission Configuration
- Defining permissions
- Role-based access
- Constraint specification

### Audit Logging
- Audit event structure
- Configuring audit backends
- Querying audit logs

## Usage Patterns

### Basic Usage

#### Setting Up OSL Supervisor
```rust
// Code example
```

#### Sending Messages to OSL Actors
```rust
// Code example
```

#### Handling Responses
```rust
// Code example
```

### Advanced Patterns

#### Cross-Supervisor Communication
#### Batch Operations
#### Error Recovery

## Performance

### Benchmark Results
- Table of overhead measurements
- Comparison with direct OSL calls

### Optimization Guidelines
- When to use actors vs direct helpers
- Message pooling strategies
- Broker configuration tuning

### Performance Characteristics
- Latency profiles
- Throughput measurements
- Resource usage

## Migration Guide

### From Direct OSL Helpers
- Step-by-step process
- Common patterns
- Code examples

### Integration with Existing Code
- Gradual migration strategy
- Compatibility considerations

## Examples

### Complete Examples
- Link to filesystem_actor_usage.rs
- Link to process_actor_usage.rs
- Link to network_actor_usage.rs
- Link to supervisor_hierarchy_complete.rs

### Common Use Cases
- Configuration file management
- Background job execution
- External service integration

## Troubleshooting

### Common Issues

#### Actor Not Responding
**Problem:** Messages sent but no response received
**Solution:** Check actor subscription and broker configuration

#### Permission Denied
**Problem:** Operations fail with permission error
**Solution:** Verify security context and permissions

#### Performance Issues
**Problem:** High latency in OSL operations
**Solution:** Check audit logging overhead, consider batching

### Debug Strategies
- Enable debug logging
- Audit trail analysis
- Message tracing

## API Reference

### Core Types
- SecurityContext
- AuditEvent
- OSLMessage enum

### Actors
- FileSystemActor
- ProcessActor
- NetworkActor

### Supervisors
- OSLSupervisor

## Best Practices

### Security
- Always provide security context
- Use least privilege principle
- Validate permissions before operations

### Performance
- Batch operations when possible
- Configure audit logging appropriately
- Use async patterns effectively

### Testing
- Use mock actors for unit tests
- Test security boundaries
- Validate audit logging

## FAQ

### Q: When should I use OSL actors vs direct helpers?
A: Use actors for long-running applications needing fault tolerance...

### Q: What is the performance overhead?
A: Benchmarks show <1% overhead for typical operations...

### Q: How do I handle actor failures?
A: Supervisor trees automatically restart failed actors...

## References
- ADR-RT-007: Hierarchical Supervisor Architecture
- ADR-RT-008: OSL Message Wrapper Pattern
- ADR-RT-009: OSL Broker Injection
- KNOWLEDGE-RT-017: OSL Integration Actors Pattern
```

**Acceptance Criteria:**
- ✅ Complete chapter covering all topics
- ✅ Clear explanations with diagrams
- ✅ Working code examples
- ✅ Troubleshooting guide
- ✅ API reference
- ✅ Best practices documented

---

## Quality Checklist

### Examples
- [ ] All 4 examples compile without errors
- [ ] All examples run successfully
- [ ] Examples follow workspace standards (§2.1-§6.3)
- [ ] Code comments explain patterns clearly
- [ ] Examples tested on target platforms

### Migration Guide
- [ ] Before/after comparisons accurate
- [ ] Step-by-step instructions clear
- [ ] Common patterns documented
- [ ] Tested with real code migration
- [ ] Troubleshooting guide complete

### Performance Benchmarks
- [ ] Benchmarks comprehensive
- [ ] Baseline comparison valid
- [ ] Overhead within targets (<1%)
- [ ] Results documented
- [ ] Performance report generated

### mdBook Documentation
- [ ] All sections complete
- [ ] Code examples working
- [ ] Diagrams clear and accurate
- [ ] Cross-references correct
- [ ] Table of contents organized

### Overall Quality
- [ ] Zero compilation errors
- [ ] Zero warnings
- [ ] All examples tested
- [ ] Documentation reviewed for accuracy
- [ ] Workspace standards compliance

---

## Integration Points

### airssys-rt Documentation
- Add OSL integration chapter to mdBook
- Update SUMMARY.md with new chapter
- Cross-reference from other chapters

### Examples Directory
- Organize examples in logical order
- Add README.md in examples/ explaining each example
- Ensure examples are discoverable

### README Updates
- Add "OSL Integration" section
- Link to examples
- Link to migration guide
- Include performance summary

---

## Success Metrics

**Phase 4 Complete When:**
- ✅ 4+ working examples demonstrating all patterns
- ✅ Migration guide enables smooth transition from direct OSL
- ✅ Performance benchmarks validate <1% overhead target
- ✅ mdBook chapter comprehensive and accurate
- ✅ All examples compile and run successfully
- ✅ Documentation reviewed and approved
- ✅ Code examples follow workspace standards

**Deliverables:**
- ✅ `examples/filesystem_actor_usage.rs` (~150-200 lines)
- ✅ `examples/process_actor_usage.rs` (~150-200 lines)
- ✅ `examples/network_actor_usage.rs` (~150-200 lines)
- ✅ `examples/supervisor_hierarchy_complete.rs` (~250-300 lines)
- ✅ `benches/osl_actor_benchmarks.rs` (~200-300 lines)
- ✅ `MIGRATION.md` (~300-400 lines)
- ✅ `airssys-rt/docs/src/osl_integration.md` (~500-800 lines)
- ✅ README.md updates

---

## RT-TASK-009 Completion

**After Phase 4:**
- ✅ All 4 phases complete (100%)
- ✅ OSL integration fully functional
- ✅ Security and audit integrated
- ✅ Comprehensive documentation
- ✅ Production-ready examples
- ✅ Performance validated

**Total Deliverables Across All Phases:**
- **Phase 1:** OSL actors implementation (~1,500 lines + 26 tests)
- **Phase 2:** Supervisor hierarchy (~800 lines + 9 tests)
- **Phase 3:** Security & audit (~800 lines + 15 tests)
- **Phase 4:** Documentation & examples (~2,400 lines)
- **Total:** ~5,500 lines of code and documentation

**Quality Achievements:**
- ✅ 450+ tests passing (all phases)
- ✅ Zero warnings
- ✅ >95% test coverage
- ✅ Performance targets met
- ✅ Security validated
- ✅ Complete documentation

---

**This action plan provides complete guidance for implementing documentation, examples, migration support, and performance validation in Phase 4, completing the OSL integration task.**
