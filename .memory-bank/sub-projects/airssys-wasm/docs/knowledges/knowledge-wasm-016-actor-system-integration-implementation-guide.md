# KNOWLEDGE-WASM-016: Actor System Integration Implementation Guide

**Created:** 2025-11-30  
**Updated:** 2025-11-30  
**Status:** Active - Complete Implementation Reference  
**Category:** Implementation Guide  
**Priority:** ⭐ CRITICAL - FOUNDATIONAL  
**Audience:** Developers implementing WASM-TASK-004 (Block 3)

## Overview

This document provides detailed implementation guidance for **WASM-TASK-004: Block 3 - Actor System Integration**. It complements the task definition with:

- **Code-level examples** for all 18 subtasks
- **Specific implementation patterns** with concrete types
- **Per-task effort estimates** (hours)
- **Testing strategies** for each component
- **Performance validation approach**
- **Integration verification checklist**

**Reference**: See `task_004_block_3_actor_system_integration.md` for overall task structure, objectives, and timeline.

---

## Phase 1: ComponentActor Foundation

### Task 1.1: ComponentActor Struct Design (12-16 hours)

#### Core ComponentActor Structure

```rust
// src/actor/component_actor.rs
use crate::core::{WasmRuntime, ComponentId, ComponentMetadata};
use airssys_rt::actor::{Actor, Child};
use tokio::sync::mpsc::UnboundedReceiver;
use chrono::{DateTime, Utc};

/// Central component execution unit: Actor for messaging, Child for WASM lifecycle
pub struct ComponentActor {
    /// Unique component identifier
    component_id: ComponentId,
    
    /// WASM runtime instance (None until Child::start())
    wasm_runtime: Option<WasmRuntime>,
    
    /// Component capabilities and permissions
    capabilities: CapabilitySet,
    
    /// Current actor state
    state: ActorState,
    
    /// Component metadata
    metadata: ComponentMetadata,
    
    /// Mailbox receiver (created by ActorSystem)
    mailbox_rx: Option<UnboundedReceiver<ComponentMessage>>,
    
    /// Timestamps for lifecycle tracking
    created_at: DateTime<Utc>,
    started_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActorState {
    Creating,
    Starting,
    Ready,
    Stopping,
    Terminated,
    Failed(String),
}

#[derive(Debug, Clone)]
pub enum ComponentMessage {
    /// Invoke WASM function with arguments
    Invoke { function: String, args: Vec<u8> },
    
    /// Result of invoke (for request-response)
    InvokeResult { result: Vec<u8>, error: Option<String> },
    
    /// Message from another component
    InterComponent { sender: ComponentId, payload: Vec<u8> },
    
    /// Shutdown signal
    Shutdown,
    
    /// Health check request
    HealthCheck,
    
    /// Health check response
    HealthStatus(HealthStatus),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded { reason: String },
    Unhealthy { reason: String },
}
```

#### Actor Trait Implementation (Stub)

```rust
#[async_trait]
impl Actor for ComponentActor {
    async fn pre_start(&mut self, ctx: &ActorContext) -> Result<()> {
        // 1. Verify WASM loaded
        if self.wasm_runtime.is_none() {
            return Err(WasmError::ComponentNotReady.into());
        }
        
        // 2. Register with component registry
        ctx.registry.register(self.component_id.clone(), self.clone()).await?;
        
        // 3. Start mailbox receiver
        self.mailbox_rx = Some(ctx.mailbox.clone());
        
        self.state = ActorState::Ready;
        Ok(())
    }
    
    async fn handle_message(&mut self, msg: ComponentMessage, ctx: &ActorContext) -> Result<()> {
        // Detailed in Task 1.3
        Ok(())
    }
    
    async fn post_stop(&mut self, ctx: &ActorContext) -> Result<()> {
        // 1. Deregister from component registry
        ctx.registry.unregister(&self.component_id).await.ok();
        
        // 2. Verify WASM cleanup (should be called by Child::stop())
        if self.wasm_runtime.is_some() {
            warn!("ComponentActor stopped but WASM runtime not cleaned up");
        }
        
        self.state = ActorState::Terminated;
        Ok(())
    }
}
```

#### Child Trait Implementation (Stub)

```rust
#[async_trait]
impl Child for ComponentActor {
    async fn start(&mut self, ctx: &ChildContext) -> Result<()> {
        // Detailed in Task 1.2
        Ok(())
    }
    
    async fn stop(&mut self, timeout: Duration) -> Result<()> {
        // Detailed in Task 1.2
        Ok(())
    }
    
    async fn health_check(&mut self) -> Result<HealthStatus> {
        // Detailed in Task 3.3
        Ok(HealthStatus::Healthy)
    }
}
```

#### Implementation Notes

1. **Ownership Model**: Option<WasmRuntime> allows safe handling of unloaded state
2. **State Machine**: ActorState tracks lifecycle transitions
3. **Timestamps**: Track creation and start time for monitoring
4. **Mailbox Integration**: Receiver stored for message processing

#### Testing Strategy (Task 1.1)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_component_actor_creation() {
        let component_id = ComponentId::new();
        let actor = ComponentActor::new(
            component_id.clone(),
            ComponentSpec::default(),
            CapabilitySet::empty(),
        );
        
        assert_eq!(actor.component_id, component_id);
        assert!(actor.wasm_runtime.is_none());
        assert_eq!(actor.state, ActorState::Creating);
    }
    
    #[test]
    fn test_component_actor_implements_actor() {
        // Verify Actor trait implementation compiles
        let _: Box<dyn Actor> = Box::new(ComponentActor::new(...));
    }
    
    #[test]
    fn test_component_actor_implements_child() {
        // Verify Child trait implementation compiles
        let _: Box<dyn Child> = Box::new(ComponentActor::new(...));
    }
}
```

---

### Task 1.2: Child Trait WASM Lifecycle (16-20 hours)

#### Child::start() Implementation

```rust
#[async_trait]
impl Child for ComponentActor {
    async fn start(&mut self, ctx: &ChildContext) -> Result<()> {
        self.state = ActorState::Starting;
        
        // 1. Load WASM bytes from storage (Block 1 integration)
        let wasm_bytes = ctx.storage
            .load_component(&self.component_id)
            .await
            .map_err(|e| WasmError::LoadFailed(self.component_id.clone(), e.to_string()))?;
        
        // 2. Create Wasmtime engine with resource limits
        let mut config = wasmtime::Config::new();
        config.async_support(true);  // Required for component execution
        config.wasm_multi_value(true);
        
        let engine = wasmtime::Engine::new(&config)
            .map_err(|e| WasmError::EngineCreationFailed(e.to_string()))?;
        
        // 3. Compile WASM module
        let module = wasmtime::Module::from_binary(&engine, &wasm_bytes)
            .map_err(|e| WasmError::CompilationFailed(e.to_string()))?;
        
        // 4. Create store with resource limiter
        let limiter = ResourceLimiter::new(
            self.metadata.memory_limit,
            self.metadata.fuel_limit,
        );
        let mut store = wasmtime::Store::new(&engine, limiter);
        
        // 5. Create linker and register host functions
        let linker = self.create_linker(&engine, ctx)?;
        
        // 6. Instantiate with error details
        let instance = linker
            .instantiate_async(&mut store, &module)
            .await
            .map_err(|e| WasmError::InstantiationFailed(
                self.component_id.clone(),
                e.to_string()
            ))?;
        
        // 7. Extract and cache exported functions
        let exports = WasmExports::extract(&instance, &mut store)?;
        
        // 8. Call optional _start export
        if let Some(start_fn) = &exports.start {
            start_fn
                .call_async(&mut store, &[])
                .await
                .map_err(|e| WasmError::StartFunctionFailed(e.to_string()))?;
        }
        
        // 9. Store runtime for later use
        self.wasm_runtime = Some(WasmRuntime::new(engine, store, exports));
        self.started_at = Some(Utc::now());
        self.state = ActorState::Ready;
        
        // 10. Log successful startup
        audit_log::info!(
            event: "component_started",
            component_id: self.component_id,
            timestamp: Utc::now(),
        );
        
        Ok(())
    }
}
```

#### Child::stop() Implementation

```rust
async fn stop(&mut self, timeout: Duration) -> Result<()> {
    self.state = ActorState::Stopping;
    
    if let Some(runtime) = &mut self.wasm_runtime {
        // 1. Call optional _cleanup export
        if let Some(cleanup_fn) = &runtime.exports.cleanup {
            let cleanup_task = cleanup_fn.call_async(&mut runtime.store, &[]);
            
            match tokio::time::timeout(timeout, cleanup_task).await {
                Ok(Ok(())) => {
                    info!("Component cleanup completed successfully");
                }
                Ok(Err(e)) => {
                    warn!("Component cleanup function failed: {}", e);
                }
                Err(_) => {
                    error!("Component cleanup timed out after {:?}", timeout);
                }
            }
        }
    }
    
    // 2. Drop WasmRuntime (frees all linear memory)
    self.wasm_runtime = None;
    self.state = ActorState::Terminated;
    
    // 3. Log shutdown
    audit_log::info!(
        event: "component_stopped",
        component_id: self.component_id,
        uptime: self.started_at.map(|t| Utc::now() - t),
    );
    
    Ok(())
}
```

#### Resource Cleanup Verification

```rust
/// Wrapper for WASM runtime with RAII cleanup
pub struct WasmRuntime {
    engine: wasmtime::Engine,
    store: wasmtime::Store<ResourceLimiter>,
    exports: WasmExports,
}

impl WasmRuntime {
    pub fn new(
        engine: wasmtime::Engine,
        store: wasmtime::Store<ResourceLimiter>,
        exports: WasmExports,
    ) -> Self {
        Self { engine, store, exports }
    }
}

impl Drop for WasmRuntime {
    fn drop(&mut self) {
        // Store drop: frees linear memory
        // Engine drop: frees module cache (if any)
        // Both are automatic - no manual cleanup needed
        debug!("WasmRuntime dropped - all resources freed");
    }
}

/// Resource limits per component instance
pub struct ResourceLimiter {
    max_memory: usize,
    max_fuel: u64,
    current_memory: AtomicUsize,
}

impl wasmtime::ResourceLimiter for ResourceLimiter {
    fn memory_growing(&mut self, current: usize, desired: usize, _maximum: Option<usize>) -> bool {
        let new_total = current + desired;
        if new_total <= self.max_memory {
            self.current_memory.store(new_total, Ordering::Relaxed);
            true
        } else {
            warn!("Memory limit exceeded: {} > {}", new_total, self.max_memory);
            false
        }
    }

    fn table_growing(&mut self, current: u32, desired: u32, _maximum: Option<u32>) -> bool {
        // Table growth allowed by default (tables don't consume linear memory)
        true
    }

    fn fuel_consumed(&mut self, fuel: u64) -> Result<(), String> {
        if self.max_fuel > 0 && fuel > self.max_fuel {
            Err("Fuel limit exceeded".to_string())
        } else {
            Ok(())
        }
    }
}
```

#### Testing Strategy (Task 1.2)

```rust
#[tokio::test]
async fn test_child_start_loads_wasm() {
    let storage = MockStorage::new();
    let component_id = ComponentId::new();
    let wasm_bytes = include_bytes!("../../tests/fixtures/minimal_component.wasm");
    
    storage.set_component(&component_id, wasm_bytes.to_vec()).await;
    
    let mut actor = ComponentActor::new(component_id.clone(), spec, caps);
    let ctx = ChildContext::new(storage);
    
    let result = actor.start(&ctx).await;
    
    assert!(result.is_ok());
    assert!(actor.wasm_runtime.is_some());
    assert_eq!(actor.state, ActorState::Ready);
}

#[tokio::test]
async fn test_child_stop_cleans_up() {
    let mut actor = create_started_component().await;
    
    let result = actor.stop(Duration::from_secs(5)).await;
    
    assert!(result.is_ok());
    assert!(actor.wasm_runtime.is_none());
    assert_eq!(actor.state, ActorState::Terminated);
}

#[tokio::test]
async fn test_child_stop_timeout() {
    let mut actor = create_started_component_with_hanging_cleanup().await;
    
    let result = actor.stop(Duration::from_millis(100)).await;
    
    assert!(result.is_ok());  // Timeout is non-fatal
    assert!(actor.wasm_runtime.is_none());  // Still cleaned up
}

#[tokio::test]
async fn test_no_memory_leaks_on_stop() {
    for _ in 0..100 {
        let mut actor = create_started_component().await;
        let _ = actor.stop(Duration::from_secs(1)).await;
        // Valgrind/LSAN would detect leaks here
    }
}
```

---

### Task 1.3: Actor Trait Message Handling (16-20 hours)

#### Actor::handle_message() Implementation

```rust
#[async_trait]
impl Actor for ComponentActor {
    async fn handle_message(&mut self, msg: ComponentMessage, ctx: &ActorContext) -> Result<()> {
        match msg {
            ComponentMessage::Invoke { function, args } => {
                // 1. Verify WASM loaded
                let runtime = self.wasm_runtime.as_mut()
                    .ok_or(WasmError::ComponentNotReady)?;
                
                // 2. Deserialize args using multicodec (ADR-WASM-001)
                let decoded_args = decode_multicodec(&args)?;
                
                // 3. Call WASM function export
                let result = runtime.call_function(&function, decoded_args).await?;
                
                // 4. Encode result using multicodec
                let encoded_result = encode_multicodec(&result)?;
                
                // 5. Send reply if sender expects one
                ctx.reply(ComponentMessage::InvokeResult {
                    result: encoded_result,
                    error: None,
                }).await?;
                
                Ok(())
            }
            
            ComponentMessage::InterComponent { sender, payload } => {
                // 1. Check capabilities (security validation in Block 4)
                if !self.capabilities.allows_receiving_from(&sender) {
                    return Err(SecurityError::CapabilityViolation.into());
                }
                
                // 2. Route to WASM handle-message export
                let runtime = self.wasm_runtime.as_mut()
                    .ok_or(WasmError::ComponentNotReady)?;
                
                if let Some(handle_fn) = &runtime.exports.handle_message {
                    handle_fn.call_async(&mut runtime.store, &payload).await?;
                }
                
                Ok(())
            }
            
            ComponentMessage::HealthCheck => {
                // 1. Call _health export if available
                let health = if let Some(runtime) = &mut self.wasm_runtime {
                    if let Some(health_fn) = &runtime.exports.health {
                        let result = health_fn.call_async(&mut runtime.store, &[]).await?;
                        HealthStatus::from_wasm_result(result)?
                    } else {
                        HealthStatus::Healthy
                    }
                } else {
                    HealthStatus::Unhealthy {
                        reason: "WASM not loaded".to_string(),
                    }
                };
                
                // 2. Send health status response
                ctx.reply(ComponentMessage::HealthStatus(health)).await?;
                
                Ok(())
            }
            
            ComponentMessage::Shutdown => {
                // Signal ActorSystem to stop this actor
                ctx.stop();
                Ok(())
            }
            
            _ => {
                warn!("Unexpected message type: {:?}", msg);
                Ok(())
            }
        }
    }
}
```

#### Multicodec Deserialization (ADR-WASM-001)

```rust
/// Supported codecs as per ADR-WASM-001
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Codec {
    Borsh = 0x701,      // Rust-native serialization
    CBOR = 0x51,        // CBOR (RFC 7049)
    JSON = 0x0200,      // JSON text
}

impl Codec {
    pub fn from_varint(varint: u32) -> Result<Self> {
        match varint {
            0x701 => Ok(Codec::Borsh),
            0x51 => Ok(Codec::CBOR),
            0x0200 => Ok(Codec::JSON),
            v => Err(CodecError::UnsupportedCodec(v)),
        }
    }
}

/// Decode multicodec-prefixed data
pub fn decode_multicodec(data: &[u8]) -> Result<Vec<u8>> {
    if data.is_empty() {
        return Err(MulticodecError::EmptyData);
    }
    
    // 1. Read varint prefix (1-4 bytes)
    let mut cursor = 0;
    let mut varint = 0u32;
    let mut shift = 0;
    
    loop {
        let byte = data.get(cursor).ok_or(MulticodecError::TruncatedData)?;
        cursor += 1;
        
        varint |= ((byte & 0x7F) as u32) << shift;
        
        if byte & 0x80 == 0 {
            break;
        }
        
        shift += 7;
        if shift > 28 {
            return Err(MulticodecError::InvalidVarint);
        }
    }
    
    // 2. Identify codec
    let codec = Codec::from_varint(varint)?;
    
    // 3. Deserialize based on codec
    let payload = &data[cursor..];
    
    match codec {
        Codec::Borsh => {
            // Borsh deserialization
            Ok(payload.to_vec())  // Already decoded
        }
        Codec::CBOR => {
            // CBOR deserialization
            let value: serde_json::Value = serde_json::from_slice(payload)?;
            serde_json::to_vec(&value)?;
            Ok(payload.to_vec())
        }
        Codec::JSON => {
            // JSON is already text, no further deserialization needed
            Ok(payload.to_vec())
        }
    }
}

/// Encode multicodec-prefixed data
pub fn encode_multicodec(codec: Codec, data: &[u8]) -> Result<Vec<u8>> {
    let mut result = Vec::new();
    
    // 1. Encode varint prefix
    let mut varint = codec as u32;
    while varint >= 0x80 {
        result.push(((varint & 0x7F) as u8) | 0x80);
        varint >>= 7;
    }
    result.push(varint as u8);
    
    // 2. Append payload
    result.extend_from_slice(data);
    
    Ok(result)
}
```

#### Testing Strategy (Task 1.3)

```rust
#[tokio::test]
async fn test_invoke_message_success() {
    let mut actor = create_started_component().await;
    let args = encode_multicodec(Codec::Borsh, b"test_arg")?;
    
    let msg = ComponentMessage::Invoke {
        function: "test_function".to_string(),
        args,
    };
    
    let result = actor.handle_message(msg, &mock_ctx).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_invoke_component_not_ready() {
    let mut actor = ComponentActor::new(...);  // Not started
    
    let msg = ComponentMessage::Invoke {
        function: "test".to_string(),
        args: vec![],
    };
    
    let result = actor.handle_message(msg, &mock_ctx).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_multicodec_decode_borsh() {
    let data = encode_varint(0x701) + b"payload";
    let result = decode_multicodec(&data)?;
    assert_eq!(result, b"payload");
}

#[tokio::test]
async fn test_multicodec_round_trip() {
    for codec in [Codec::Borsh, Codec::CBOR, Codec::JSON] {
        let original = b"test data";
        let encoded = encode_multicodec(codec, original)?;
        let decoded = decode_multicodec(&encoded)?;
        assert_eq!(decoded, original);
    }
}

#[tokio::test]
async fn test_message_throughput() {
    let mut actor = create_started_component().await;
    let msg = ComponentMessage::Invoke {
        function: "noop".to_string(),
        args: vec![],
    };
    
    let start = Instant::now();
    for _ in 0..10_000 {
        actor.handle_message(msg.clone(), &mock_ctx).await.ok();
    }
    let elapsed = start.elapsed();
    
    let throughput = 10_000.0 / elapsed.as_secs_f64();
    println!("Message throughput: {:.0} msg/sec", throughput);
    assert!(throughput > 10_000.0);  // Target >10K msg/sec
}
```

---

## Phase 2: ActorSystem Integration

### Task 2.1: ActorSystem::spawn() Integration (12-16 hours)

#### Component Spawning via ActorSystem

```rust
pub struct ComponentRuntime {
    actor_system: Arc<ActorSystem>,
    component_registry: Arc<ComponentRegistry>,
    supervisor: Arc<SupervisorNode>,
}

impl ComponentRuntime {
    pub async fn spawn_component(&self, spec: ComponentSpec) -> Result<ComponentId> {
        let component_id = ComponentId::new();
        
        // 1. Create ComponentActor (WASM not loaded yet)
        let actor = ComponentActor::new(
            component_id.clone(),
            spec.clone(),
            spec.capabilities.clone(),
        );
        
        // 2. Add to supervisor tree
        // (WASM loads in Child::start(), controlled by supervisor)
        let child_spec = ChildSpec::new(component_id.clone(), actor);
        self.supervisor.start_child(child_spec).await?;
        
        // 3. Spawn actor via ActorSystem (NOT tokio::spawn!)
        // ActorSystem handles mailbox creation, registration, etc.
        let actor_ref = self.actor_system.spawn(actor).await?;
        
        // 4. Wait for actor to fully start
        // (Child trait start() called, WASM loaded)
        tokio::time::timeout(
            Duration::from_secs(10),
            self.wait_for_ready(&component_id),
        ).await??;
        
        // 5. Register in component registry
        self.component_registry.register(
            component_id.clone(),
            ComponentEntry {
                actor_ref,
                spec,
                status: ComponentStatus::Running,
                installed_at: Utc::now(),
                version: VersionInfo::default(),
            },
        ).await?;
        
        Ok(component_id)
    }
    
    async fn wait_for_ready(&self, component_id: &ComponentId) -> Result<()> {
        // Poll health status until Ready
        loop {
            if let Some(entry) = self.component_registry.get(component_id).await {
                if entry.status == ComponentStatus::Running {
                    return Ok(());
                }
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
}
```

#### Testing Strategy (Task 2.1)

```rust
#[tokio::test]
async fn test_spawn_component_success() {
    let runtime = ComponentRuntime::new(
        Arc::new(ActorSystem::new()),
        Arc::new(ComponentRegistry::new()),
        Arc::new(SupervisorNode::new()),
    );
    
    let spec = ComponentSpec::new("test-component", include_bytes!(...));
    let component_id = runtime.spawn_component(spec).await?;
    
    assert!(runtime.component_registry.get(&component_id).await.is_some());
}

#[tokio::test]
async fn test_spawn_time_target() {
    let runtime = ComponentRuntime::new(...);
    let spec = ComponentSpec::new(...);
    
    let start = Instant::now();
    runtime.spawn_component(spec).await?;
    let elapsed = start.elapsed();
    
    println!("Spawn time: {:?}", elapsed);
    assert!(elapsed < Duration::from_millis(5));  // Target <5ms
}

#[tokio::test]
async fn test_spawn_100_concurrent_components() {
    let runtime = Arc::new(ComponentRuntime::new(...));
    let spec = ComponentSpec::new(...);
    
    let mut handles = vec![];
    for i in 0..100 {
        let runtime = runtime.clone();
        let spec = spec.clone();
        
        handles.push(tokio::spawn(async move {
            runtime.spawn_component(spec).await
        }));
    }
    
    let results = futures::future::join_all(handles).await;
    assert!(results.iter().all(|r| r.is_ok()));
}
```

---

### Task 2.2: Component Instance Management (12-16 hours)

#### ComponentRegistry Implementation

```rust
pub struct ComponentRegistry {
    components: DashMap<ComponentId, ComponentEntry>,
    by_name: DashMap<String, ComponentId>,
}

#[derive(Clone)]
pub struct ComponentEntry {
    pub actor_ref: ActorRef<ComponentActor>,
    pub spec: ComponentSpec,
    pub status: ComponentStatus,
    pub installed_at: DateTime<Utc>,
    pub version: VersionInfo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentStatus {
    Installing,
    Running,
    Stopped,
    Failed,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        Self {
            components: DashMap::new(),
            by_name: DashMap::new(),
        }
    }
    
    /// Register a component instance
    pub async fn register(&self, id: ComponentId, entry: ComponentEntry) -> Result<()> {
        // Store by ID
        self.components.insert(id.clone(), entry.clone());
        
        // Store by name for lookup
        self.by_name.insert(entry.spec.name.clone(), id);
        
        debug!("Registered component: {} ({})", entry.spec.name, id);
        Ok(())
    }
    
    /// Unregister a component instance
    pub async fn unregister(&self, id: &ComponentId) -> Result<ComponentEntry> {
        if let Some((_, entry)) = self.components.remove(id) {
            self.by_name.remove(&entry.spec.name);
            debug!("Unregistered component: {}", id);
            Ok(entry)
        } else {
            Err(RegistryError::ComponentNotFound(id.clone()))
        }
    }
    
    /// Get component by ID (O(1))
    pub async fn get(&self, id: &ComponentId) -> Option<ComponentEntry> {
        self.components.get(id).map(|entry| entry.clone())
    }
    
    /// Resolve component by name
    pub async fn resolve_by_name(&self, name: &str) -> Option<ComponentId> {
        self.by_name.get(name).map(|id| id.clone())
    }
    
    /// List all components
    pub async fn list(&self) -> Vec<(ComponentId, ComponentEntry)> {
        self.components
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }
    
    /// Update component status
    pub async fn update_status(&self, id: &ComponentId, status: ComponentStatus) -> Result<()> {
        if let Some(mut entry) = self.components.get_mut(id) {
            entry.status = status;
            Ok(())
        } else {
            Err(RegistryError::ComponentNotFound(id.clone()))
        }
    }
}
```

#### Testing Strategy (Task 2.2)

```rust
#[tokio::test]
async fn test_registry_register_and_get() {
    let registry = ComponentRegistry::new();
    let component_id = ComponentId::new();
    let entry = create_component_entry();
    
    registry.register(component_id.clone(), entry.clone()).await?;
    
    let retrieved = registry.get(&component_id).await;
    assert_eq!(retrieved.map(|e| e.spec.name), Some(entry.spec.name));
}

#[tokio::test]
async fn test_registry_lookup_by_name() {
    let registry = ComponentRegistry::new();
    let component_id = ComponentId::new();
    let mut entry = create_component_entry();
    entry.spec.name = "my-component".to_string();
    
    registry.register(component_id.clone(), entry).await?;
    
    let resolved = registry.resolve_by_name("my-component").await;
    assert_eq!(resolved, Some(component_id));
}

#[tokio::test]
async fn test_registry_lookup_performance() {
    let registry = ComponentRegistry::new();
    
    // Register 10,000 components
    for i in 0..10_000 {
        let component_id = ComponentId::new();
        let entry = create_component_entry();
        registry.register(component_id, entry).await.ok();
    }
    
    // Measure lookup time
    let start = Instant::now();
    for _ in 0..10_000 {
        let id = ComponentId::new();
        registry.get(&id).await;
    }
    let elapsed = start.elapsed();
    
    let avg_latency = elapsed.as_nanos() / 10_000;
    println!("Average lookup latency: {} ns", avg_latency);
    assert!(avg_latency < 1_000);  // Target <1us
}
```

---

### Task 2.3: Actor Address and Routing (8-12 hours)

#### ActorRef Wrapper and Routing

```rust
pub struct ComponentAddress {
    component_id: ComponentId,
    actor_ref: ActorRef<ComponentActor>,
}

impl ComponentAddress {
    pub fn new(component_id: ComponentId, actor_ref: ActorRef<ComponentActor>) -> Self {
        Self { component_id, actor_ref }
    }
    
    /// Send message asynchronously (fire-and-forget)
    pub async fn send(&self, msg: ComponentMessage) -> Result<()> {
        self.actor_ref.send(msg).await
            .map_err(|e| RoutingError::MailboxFull(self.component_id.clone()))
    }
    
    /// Send message and wait for reply (request-response)
    pub async fn ask<R>(&self, msg: ComponentMessage, timeout: Duration) -> Result<R> {
        tokio::time::timeout(
            timeout,
            self.actor_ref.ask(msg),
        )
        .await
        .map_err(|_| RoutingError::Timeout(self.component_id.clone()))?
        .map_err(|e| e.into())
    }
}

/// Routing error types
#[derive(Debug)]
pub enum RoutingError {
    ComponentNotFound(ComponentId),
    ComponentNotRunning(ComponentId),
    MailboxFull(ComponentId),
    Timeout(ComponentId),
}
```

#### Testing Strategy (Task 2.3)

```rust
#[tokio::test]
async fn test_send_message_to_component() {
    let address = create_component_address().await;
    
    let msg = ComponentMessage::Invoke {
        function: "test".to_string(),
        args: vec![],
    };
    
    let result = address.send(msg).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_routing_latency() {
    let address = create_component_address().await;
    
    let start = Instant::now();
    let msg = ComponentMessage::HealthCheck;
    address.send(msg).await?;
    let elapsed = start.elapsed();
    
    println!("Routing latency: {:?}", elapsed);
    assert!(elapsed < Duration::from_millis(1));  // Target <1ms
}
```

---

## Phase 3: SupervisorNode Integration

### Task 3.1: Supervisor Tree Setup (12-16 hours)

```rust
pub struct ComponentSupervisor {
    supervisor: SupervisorNode,
    restart_strategy: SupervisionStrategy,
    max_restarts: u32,
    restart_window: Duration,
}

impl ComponentSupervisor {
    pub fn new(strategy: SupervisionStrategy) -> Self {
        let supervisor = SupervisorNode::builder()
            .strategy(strategy)
            .max_restarts(5)
            .restart_window(Duration::from_secs(60))
            .build();
        
        Self {
            supervisor,
            restart_strategy: strategy,
            max_restarts: 5,
            restart_window: Duration::from_secs(60),
        }
    }
    
    pub async fn add_component(&self, spec: ChildSpec) -> Result<()> {
        self.supervisor.start_child(spec).await
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SupervisionStrategy {
    /// Restart only the failed component
    OneForOne,
    /// Restart all components if one fails
    OneForAll,
    /// Restart failed component and those started after it
    RestForOne,
}
```

---

## Phase 4: MessageBroker Integration

### Task 4.1: MessageBroker Setup (12-16 hours)

```rust
pub struct ComponentMessaging {
    broker: Arc<InMemoryMessageBroker<ComponentMessage>>,
}

impl ComponentMessaging {
    pub async fn new() -> Self {
        let broker = Arc::new(InMemoryMessageBroker::new());
        Self { broker }
    }
    
    pub async fn publish(&self, envelope: MessageEnvelope) -> Result<()> {
        self.broker.publish(envelope).await
    }
    
    pub async fn subscribe(&self) -> Result<MessageStream<ComponentMessage>> {
        self.broker.subscribe().await
    }
}

#[derive(Clone)]
pub struct MessageEnvelope {
    pub from: ComponentId,
    pub to: ComponentId,
    pub payload: Vec<u8>,
    pub correlation_id: Option<String>,
    pub timestamp: DateTime<Utc>,
}
```

---

## Performance Targets & Validation

### Spawn Time Performance (Phase 5, Task 5.1)

**Target:** <5ms average, <10ms P99

**Measurement:**
```rust
let start = Instant::now();
runtime.spawn_component(spec).await?;
let elapsed = start.elapsed();
```

**Optimization Strategies:**
1. Module caching (Block 1 integration)
2. Parallel WASM compilation
3. Pre-allocation of actor resources

### Message Routing Performance (Phase 5, Task 5.2)

**Target:** <1ms end-to-end

**Breakdown:**
- Host function validation: ~50ns
- MessageBroker routing: ~211ns (airssys-rt baseline)
- ActorSystem mailbox delivery: ~40ns
- Total: ~300ns overhead

### Memory Footprint (Phase 5, Task 5.3)

**Target:** <2MB per component instance

**Components:**
- ComponentActor struct: ~1KB
- WASM linear memory: 512KB-4MB (configurable)
- Store/Engine overhead: ~500KB

---

## Integration Verification Checklist

When all phases complete:

- [ ] ComponentActor implements Actor + Child traits
- [ ] Child::start() loads WASM successfully
- [ ] Child::stop() cleans up all resources
- [ ] Actor::handle_message() dispatches to WASM
- [ ] Components spawn via ActorSystem (not tokio::spawn)
- [ ] ComponentRegistry provides O(1) lookup
- [ ] ActorRef routing works correctly
- [ ] SupervisorNode controls component lifecycle
- [ ] Automatic restart on crashes functional
- [ ] Health monitoring via Child::health_check()
- [ ] MessageBroker routes inter-component messages
- [ ] Pub-sub message delivery works
- [ ] ActorSystem as primary subscriber pattern proven
- [ ] Spawn time <5ms average
- [ ] Message routing <1ms end-to-end
- [ ] No memory leaks detected in stress tests
- [ ] 10,000+ concurrent components achievable
- [ ] Integration test suite passing (>90% coverage)
- [ ] Performance benchmarks documented
- [ ] Actor-based testing framework operational

---

## References

**Task Definition:**
- `task_004_block_3_actor_system_integration.md` - Overall structure and objectives

**Related Knowledge:**
- `knowledge_wasm_001_component_framework_architecture.md` - Component framework overview
- `knowledge_wasm_005_inter_component_messaging_architecture.md` - MessageBroker patterns
- `knowledge_rt_013_actor_performance_benchmarking_results.md` - Performance baselines

**ADRs:**
- ADR-WASM-006: Component Isolation and Sandboxing
- ADR-WASM-009: Component Communication Model
- ADR-WASM-010: Implementation Strategy
- ADR-RT-004: Actor and Child Trait Separation

**External:**
- [Erlang OTP Supervision](https://www.erlang.org/doc/design_principles/sup_princ.html)
- [Tokio async patterns](https://tokio.rs/)
- [Actor model principles](https://en.wikipedia.org/wiki/Actor_model)

