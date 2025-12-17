# Component Composition Guide

This guide shows you how to orchestrate multiple components together to build complex systems. Learn pipeline patterns, parallel execution, and error handling strategies for component composition.

## Overview

Component composition enables building sophisticated systems from simple, reusable components. Components communicate via messages, forming pipelines, parallel processing units, or complex orchestration patterns.

**Key Patterns:**

- **Pipeline**: Sequential processing (A → B → C)
- **Parallel**: Independent concurrent processing
- **Fan-Out/Fan-In**: Distribute work, aggregate results (1 → N → 1)

## Pipeline Patterns

### Sequential Pipeline (A → B → C)

Process data through multiple stages:

```rust
// Layer 1: Standard library
use std::sync::Arc;

// Layer 2: Third-party crates
use tokio::sync::RwLock;

// Layer 3: Internal modules
use airssys_rt::prelude::*;
use airssys_wasm::actor::ComponentActor;

// Stage 1: Ingress (receives raw data)
#[derive(Clone)]
pub struct IngressComponent {
    output_target: Arc<RwLock<Option<ComponentId>>>,
}

#[async_trait::async_trait]
impl Actor for IngressComponent {
    type Message = IngressMessage;
    type Error = ComponentError;
    
    async fn handle_message(
        &mut self,
        message: Self::Message,
        context: &ActorContext,
    ) -> Result<(), Self::Error> {
        match message {
            IngressMessage::RawData(data) => {
                // Process and forward to next stage
                let processed = format!("ingress:{}", data);
                
                let target = self.output_target.read().await;
                if let Some(target_id) = *target {
                    context.send_message(target_id, ProcessorMessage::Process(processed)).await?;
                }
            }
            IngressMessage::SetTarget(target_id) => {
                let mut target = self.output_target.write().await;
                *target = Some(target_id);
            }
        }
        Ok(())
    }
}

// Stage 2: Processor (transforms data)
#[derive(Clone)]
pub struct ProcessorComponent {
    output_target: Arc<RwLock<Option<ComponentId>>>,
}

#[async_trait::async_trait]
impl Actor for ProcessorComponent {
    type Message = ProcessorMessage;
    type Error = ComponentError;
    
    async fn handle_message(
        &mut self,
        message: Self::Message,
        context: &ActorContext,
    ) -> Result<(), Self::Error> {
        match message {
            ProcessorMessage::Process(data) => {
                // Transform and forward to next stage
                let transformed = format!("processor:{}", data);
                
                let target = self.output_target.read().await;
                if let Some(target_id) = *target {
                    context.send_message(target_id, EgressMessage::Finalize(transformed)).await?;
                }
            }
            ProcessorMessage::SetTarget(target_id) => {
                let mut target = self.output_target.write().await;
                *target = Some(target_id);
            }
        }
        Ok(())
    }
}

// Stage 3: Egress (outputs results)
#[derive(Clone)]
pub struct EgressComponent {
    results: Arc<RwLock<Vec<String>>>,
}

#[async_trait::async_trait]
impl Actor for EgressComponent {
    type Message = EgressMessage;
    type Error = ComponentError;
    
    async fn handle_message(
        &mut self,
        message: Self::Message,
        _context: &ActorContext,
    ) -> Result<(), Self::Error> {
        match message {
            EgressMessage::Finalize(data) => {
                let mut results = self.results.write().await;
                results.push(data);
                println!("Pipeline output: {}", results.last().unwrap());
            }
        }
        Ok(())
    }
}
```

### Setting Up Pipeline

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let actor_system = ActorSystem::new("pipeline-system").await?;
    
    // Create components
    let ingress = IngressComponent::new();
    let processor = ProcessorComponent::new();
    let egress = EgressComponent::new();
    
    // Spawn components
    let ingress_id = actor_system.spawn_component(ingress).await?;
    let processor_id = actor_system.spawn_component(processor).await?;
    let egress_id = actor_system.spawn_component(egress).await?;
    
    // Wire pipeline: Ingress → Processor → Egress
    ingress_id.send(IngressMessage::SetTarget(processor_id)).await?;
    processor_id.send(ProcessorMessage::SetTarget(egress_id)).await?;
    
    // Send data through pipeline
    ingress_id.send(IngressMessage::RawData("hello".to_string())).await?;
    // Output: "Pipeline output: processor:ingress:hello"
    
    Ok(())
}
```

**Performance:**

- Message routing: 1.05µs per stage (Task 6.2 messaging_benchmarks.rs)
- 3-stage pipeline: ~3µs total latency
- Throughput: 333k pipelines/sec (1 / 3µs)

## Parallel Execution Patterns

### Independent Parallel Processing

Execute components concurrently:

```rust
use tokio::join;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let actor_system = ActorSystem::new("parallel-system").await?;
    
    // Spawn 3 independent components
    let component_a = ComponentA::new();
    let component_b = ComponentB::new();
    let component_c = ComponentC::new();
    
    let id_a = actor_system.spawn_component(component_a).await?;
    let id_b = actor_system.spawn_component(component_b).await?;
    let id_c = actor_system.spawn_component(component_c).await?;
    
    // Send messages in parallel (concurrent execution)
    let (result_a, result_b, result_c) = join!(
        id_a.send(MessageA::Process("data_a".to_string())),
        id_b.send(MessageB::Process("data_b".to_string())),
        id_c.send(MessageC::Process("data_c".to_string())),
    );
    
    println!("Parallel execution complete");
    println!("  A: {:?}", result_a);
    println!("  B: {:?}", result_b);
    println!("  C: {:?}", result_c);
    
    Ok(())
}
```

**Performance:**

- Concurrent operations validated in Task 6.2 (scalability_benchmarks.rs)
- 100 concurrent operations: 120µs total
- Throughput: 833k concurrent ops/sec

## Fan-Out/Fan-In Pattern

### Fan-Out (1 → N)

Distribute work to multiple workers:

```rust
#[derive(Clone)]
pub struct Coordinator {
    workers: Arc<RwLock<Vec<ComponentId>>>,
}

#[async_trait::async_trait]
impl Actor for Coordinator {
    type Message = CoordinatorMessage;
    type Error = ComponentError;
    
    async fn handle_message(
        &mut self,
        message: Self::Message,
        context: &ActorContext,
    ) -> Result<(), Self::Error> {
        match message {
            CoordinatorMessage::Distribute(data) => {
                // Fan-out: Send to all workers
                let workers = self.workers.read().await;
                for worker_id in workers.iter() {
                    context.send_message(
                        *worker_id,
                        WorkerMessage::Process(data.clone())
                    ).await?;
                }
                println!("Distributed to {} workers", workers.len());
            }
            CoordinatorMessage::AddWorker(worker_id) => {
                let mut workers = self.workers.write().await;
                workers.push(worker_id);
            }
        }
        Ok(())
    }
}
```

**Performance (Task 6.2 messaging_benchmarks.rs):**

- Pub-sub fanout to 100 subscribers: 85.2µs
- Per-subscriber overhead: ~852ns
- Throughput: 11,737 fanouts/sec (100 subscribers each)

### Fan-In (N → 1)

Aggregate results from multiple sources:

```rust
#[derive(Clone)]
pub struct Aggregator {
    results: Arc<RwLock<Vec<WorkerResult>>>,
    expected_count: Arc<RwLock<usize>>,
}

#[async_trait::async_trait]
impl Actor for Aggregator {
    type Message = AggregatorMessage;
    type Error = ComponentError;
    
    async fn handle_message(
        &mut self,
        message: Self::Message,
        _context: &ActorContext,
    ) -> Result<(), Self::Error> {
        match message {
            AggregatorMessage::Result(result) => {
                let mut results = self.results.write().await;
                results.push(result);
                
                // Check if all results received
                let expected = *self.expected_count.read().await;
                if results.len() == expected {
                    println!("All {} results aggregated", expected);
                    // Process aggregated results
                    let sum: i64 = results.iter().map(|r| r.value).sum();
                    println!("Aggregated sum: {}", sum);
                }
            }
            AggregatorMessage::SetExpectedCount(count) => {
                let mut expected = self.expected_count.write().await;
                *expected = count;
            }
        }
        Ok(())
    }
}
```

## Component Dependencies

### Startup Order

Start components in dependency order:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let actor_system = ActorSystem::new("ordered-system").await?;
    
    // 1. Start foundation components (no dependencies)
    let database_id = actor_system.spawn_component(DatabaseComponent::new()).await?;
    let cache_id = actor_system.spawn_component(CacheComponent::new()).await?;
    
    // Wait for foundation components to be ready
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // 2. Start service components (depend on database/cache)
    let service_id = actor_system.spawn_component(
        ServiceComponent::new(database_id, cache_id)
    ).await?;
    
    // 3. Start API gateway (depends on service)
    let api_id = actor_system.spawn_component(
        APIGatewayComponent::new(service_id)
    ).await?;
    
    println!("All components started in order");
    Ok(())
}
```

**Performance:**

- Component spawn: 286ns per component (Task 6.2 actor_lifecycle_benchmarks.rs)
- 10 components: 2.86µs total spawn time
- 100 components: 28.6µs total spawn time

### Shutdown Order

Stop components in reverse dependency order:

```rust
pub async fn graceful_shutdown(
    actor_system: ActorSystem,
    component_ids: Vec<ComponentId>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Reverse order: stop API → Service → Database/Cache
    for component_id in component_ids.iter().rev() {
        println!("Stopping component: {}", component_id);
        actor_system.stop_component(*component_id).await?;
        
        // Wait for component to stop cleanly
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    
    println!("All components stopped");
    Ok(())
}
```

## Error Propagation in Pipelines

### Failure Handling Strategies

**Strategy 1: Stop on First Error**

```rust
#[async_trait::async_trait]
impl Actor for ProcessorComponent {
    type Message = ProcessorMessage;
    type Error = ComponentError;
    
    async fn handle_message(
        &mut self,
        message: Self::Message,
        context: &ActorContext,
    ) -> Result<(), Self::Error> {
        match message {
            ProcessorMessage::Process(data) => {
                // Process or propagate error
                let result = self.process_data(&data).await?;
                
                // Forward only on success
                let target = self.output_target.read().await;
                if let Some(target_id) = *target {
                    context.send_message(target_id, EgressMessage::Finalize(result)).await?;
                }
                
                Ok(())
            }
        }
    }
}
```

**Strategy 2: Continue with Error Logging**

```rust
#[async_trait::async_trait]
impl Actor for ResilientProcessor {
    type Message = ProcessorMessage;
    type Error = ComponentError;
    
    async fn handle_message(
        &mut self,
        message: Self::Message,
        context: &ActorContext,
    ) -> Result<(), Self::Error> {
        match message {
            ProcessorMessage::Process(data) => {
                match self.process_data(&data).await {
                    Ok(result) => {
                        // Success - forward result
                        let target = self.output_target.read().await;
                        if let Some(target_id) = *target {
                            context.send_message(
                                target_id,
                                EgressMessage::Finalize(result)
                            ).await?;
                        }
                    }
                    Err(err) => {
                        // Error - log and continue processing
                        tracing::error!(
                            component_id = %context.component_id,
                            error = %err,
                            "Processing failed, skipping message"
                        );
                    }
                }
                
                Ok(())
            }
        }
    }
}
```

**Strategy 3: Dead Letter Queue**

```rust
pub struct DeadLetterQueue {
    failed_messages: Arc<RwLock<Vec<FailedMessage>>>,
}

#[derive(Debug)]
pub struct FailedMessage {
    pub original_message: String,
    pub error: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub component_id: ComponentId,
}

impl DeadLetterQueue {
    pub async fn send_to_dlq(
        &self,
        message: String,
        error: String,
        component_id: ComponentId,
    ) {
        let mut failed = self.failed_messages.write().await;
        failed.push(FailedMessage {
            original_message: message,
            error,
            timestamp: chrono::Utc::now(),
            component_id,
        });
    }
}

// Usage in component
match self.process_data(&data).await {
    Ok(result) => { /* forward */ }
    Err(err) => {
        // Send to dead letter queue for later retry
        dlq.send_to_dlq(data, err.to_string(), context.component_id).await;
    }
}
```

## State Sharing Between Components

### When to Share State

**Appropriate Use Cases:**

- Configuration data (read-only, infrequent updates)
- Metrics aggregation (write-mostly, periodic reads)
- Shared caches (read-heavy, concurrent access)

**Avoid Sharing State When:**

- Frequent writes from multiple components (high contention)
- Order-dependent operations (use message passing instead)
- Complex synchronization needed (deadlock risk)

### Safe State Sharing Pattern

```rust
use dashmap::DashMap;

// Shared state with concurrent access
pub struct SharedCache {
    cache: Arc<DashMap<String, CachedValue>>,
}

#[derive(Clone)]
pub struct CachedValue {
    pub data: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl SharedCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
        }
    }
    
    /// Read from cache (lock-free for concurrent reads)
    pub fn get(&self, key: &str) -> Option<CachedValue> {
        self.cache.get(key).map(|entry| entry.value().clone())
    }
    
    /// Write to cache (concurrent writes safe)
    pub fn insert(&self, key: String, value: CachedValue) {
        self.cache.insert(key, value);
    }
}

// Component A writes
let cache = SharedCache::new();
cache.insert("key1".to_string(), CachedValue {
    data: "value1".to_string(),
    timestamp: chrono::Utc::now(),
});

// Component B reads (concurrent, no blocking)
if let Some(value) = cache.get("key1") {
    println!("Cached: {}", value.data);
}
```

**Performance (Task 6.2 actor_lifecycle_benchmarks.rs):**

- State access (read): 37ns
- State access (write): 39ns
- Concurrent access validated in scalability tests

## Composition Best Practices

### 1. Keep Components Small and Focused

Each component should have a single responsibility:

```rust
// ✅ GOOD: Focused component
pub struct JSONParserComponent {
    // Only parses JSON
}

// ❌ BAD: Too many responsibilities
pub struct SuperComponent {
    // Parses JSON, validates, transforms, stores, sends emails...
}
```

### 2. Use Message Passing Over Shared State

Prefer messages for coordination:

```rust
// ✅ GOOD: Message-based coordination
component_a.send(MessageA::RequestData(query)).await?;
// Component B responds via callback message

// ❌ BAD: Shared mutable state
let shared_state = Arc::new(RwLock::new(State::new()));
// Multiple components mutate shared_state (contention risk)
```

### 3. Make Components Stateless When Possible

Stateless components are easier to scale and recover:

```rust
// ✅ GOOD: Stateless transformer
pub struct TransformerComponent {
    // No internal state, pure transformation
}

// Message includes all needed data
pub enum TransformerMessage {
    Transform { input: String, output_target: ComponentId },
}
```

### 4. Handle Errors at Boundaries

Don't propagate errors across component boundaries unnecessarily:

```rust
// ✅ GOOD: Handle errors locally
match self.external_api_call().await {
    Ok(result) => { /* forward result */ }
    Err(err) => {
        tracing::error!("API call failed: {}", err);
        // Send error message instead of propagating
        context.send_message(target, ErrorMessage::APIFailure(err.to_string())).await?;
    }
}
```

### 5. Use Supervision for Recovery

Let supervisor handle component crashes:

```rust
// ✅ GOOD: Let supervisor handle crashes
// Component crashes → supervisor restarts → automatic recovery

// ❌ BAD: Try/catch everything
// Component never crashes → errors accumulate → degraded state
```

## Summary

Compose components in under 45 minutes:

1. **Pipeline Pattern**: Chain components for sequential processing (A → B → C)
2. **Parallel Pattern**: Execute components concurrently for independent tasks
3. **Fan-Out/Fan-In**: Distribute work (1 → N) and aggregate results (N → 1)
4. **Manage Dependencies**: Start components in order, stop in reverse order
5. **Error Handling**: Choose strategy (stop, continue, dead letter queue)
6. **State Sharing**: Minimize shared state, prefer message passing

**Performance Characteristics:**

- Message routing: 1.05µs per hop (messaging_benchmarks.rs)
- Pipeline (3 stages): ~3µs total latency
- Fan-out (100 components): 85.2µs (messaging_benchmarks.rs)
- Concurrent operations (100): 120µs (scalability_benchmarks.rs)

## Next Steps

- [Supervision and Recovery](supervision-and-recovery.md) - Handle component failures
- [Production Deployment](production-deployment.md) - Deploy composed systems
- [Best Practices](best-practices.md) - Advanced composition patterns
