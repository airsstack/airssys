# airssys-wasm Tech Context

## Technology Stack

### Core WASM Runtime
- **wasmtime**: Primary WASM runtime engine (pending ADR decision)
- **WebAssembly Component Model**: Component composition and interface types
- **WASI Preview 2**: System interface for component capabilities
- **wit-bindgen**: Component interface generation and bindings

### Primary Dependencies
```toml
# WASM Runtime
wasmtime = { version = "24.0", features = ["component-model", "async"] }
wasmtime-wasi = { version = "24.0" }
wit-bindgen = { version = "0.30" }

# Component Model Support  
wit-component = { version = "0.200" }
wasm-encoder = { version = "0.200" }
wasm-metadata = { version = "0.200" }

# Async and Concurrency
tokio = { version = "1.47", features = ["full"] }
futures = { version = "0.3" }
async-trait = { version = "0.1" }

# Security and Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = { version = "1.0" }
chrono = { version = "0.4", features = ["serde"] }  # Workspace standard §3.2

# Error Handling
thiserror = { version = "1.0" }
anyhow = { version = "1.0" }
```

### AirsSys Integration
```toml
# Integration with other AirsSys components
airssys-osl = { path = "../airssys-osl" }  # OS layer integration
airssys-rt = { path = "../airssys-rt" }   # Actor system integration
```

## WASM Runtime Architecture

### Component Model Implementation
- **Interface Types**: Strong typing for component boundaries
- **Resource Management**: Automatic resource cleanup and lifetime management
- **Linking**: Dynamic component linking and composition
- **Imports/Exports**: Fine-grained capability exposure

### Security Architecture
```rust
// Capability-based security implementation
#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentCapabilities {
    pub wasi: WasiCapabilities,
    pub airssys: AirsSysCapabilities,
    pub custom: CustomCapabilities,
}

pub struct WasiCapabilities {
    pub filesystem: FilesystemCapabilities,
    pub network: NetworkCapabilities,
    pub clocks: ClocksCapabilities,
    pub random: RandomCapabilities,
}

pub struct AirsSysCapabilities {
    pub osl_operations: OSLCapabilities,
    pub rt_messaging: RTCapabilities,
    pub custom_resources: Vec<CustomResource>,
}
```

### Performance Targets
- **Component Instantiation**: <10ms for typical components
- **Memory Overhead**: <512KB baseline per component
- **Function Call Overhead**: <1μs for simple function calls
- **Component Communication**: <100μs for inter-component messages

## Security Implementation

### Sandbox Architecture
- **Memory Isolation**: Complete memory isolation between components and host
- **Capability Enforcement**: Runtime capability checking for all system access
- **Resource Limits**: CPU time, memory, and I/O bandwidth limits per component
- **Audit Logging**: Comprehensive logging of all component operations

### Threat Model
- **Malicious Components**: Assume components may be adversarial
- **Resource Exhaustion**: Protection against DoS through resource consumption
- **Data Exfiltration**: Prevent unauthorized data access and exfiltration
- **Privilege Escalation**: Prevent components from gaining additional capabilities

## Integration Architecture

### airssys-osl Integration
```rust
// Secure system access through airssys-osl
pub struct WASMOSLBridge {
    osl_context: airssys_osl::SecurityContext,
    component_capabilities: HashMap<ComponentId, CapabilitySet>,
}

impl WASMOSLBridge {
    pub async fn handle_file_operation(
        &self,
        component_id: ComponentId,
        operation: FileOperation,
    ) -> Result<FileResult, BridgeError> {
        // Validate component capability
        let capabilities = self.component_capabilities.get(&component_id)
            .ok_or(BridgeError::UnknownComponent)?;
            
        if !capabilities.can_perform_operation(&operation) {
            return Err(BridgeError::InsufficientCapabilities);
        }
        
        // Delegate to airssys-osl with security context
        self.osl_context.execute_file_operation(operation).await
            .map_err(BridgeError::OSLError)
    }
}
```

### airssys-rt Integration
```rust
// Component hosting through actor system
pub struct ComponentActorHost {
    actor_system: airssys_rt::ActorSystem,
    component_actors: HashMap<ComponentId, ActorId>,
}

impl ComponentActorHost {
    pub async fn spawn_component_actor(
        &mut self,
        component: CompiledComponent,
        capabilities: CapabilitySet,
    ) -> Result<ComponentId, HostError> {
        let component_id = ComponentId::new();
        
        let actor = ComponentActor::new(component, capabilities);
        let actor_id = self.actor_system.spawn_actor(actor).await?;
        
        self.component_actors.insert(component_id, actor_id);
        Ok(component_id)
    }
}
```

## Performance Optimization

### Runtime Optimizations
- **JIT Compilation**: Wasmtime Cranelift JIT for high-performance execution
- **Module Caching**: Compiled module caching for faster instantiation  
- **Memory Pool**: Pre-allocated memory pools for component instances
- **Function Call Optimization**: Direct function call optimization for hot paths

### Component Pool Management
```rust
pub struct ComponentPool<T> {
    template: ComponentTemplate,
    available: VecDeque<PooledComponent<T>>,
    in_use: HashMap<ComponentId, PooledComponent<T>>,
    max_size: usize,
}

impl<T> ComponentPool<T> {
    pub async fn acquire(&mut self) -> Result<ComponentId, PoolError> {
        // Reuse existing instance or create new one
        match self.available.pop_front() {
            Some(component) => {
                let id = ComponentId::new();
                self.in_use.insert(id, component);
                Ok(id)
            }
            None => self.create_new_instance().await,
        }
    }
}
```

## Development and Testing

### Component Development
- **wit-bindgen**: Automatic binding generation from WIT interfaces
- **Component Tooling**: Integration with wasm-tools for component manipulation
- **Language Support**: Rust, C/C++, JavaScript, Go, Python via WASM
- **Hot Reloading**: Component hot-reloading for development workflows

### Testing Strategy
```rust
// Component testing framework
pub struct ComponentTestHarness {
    runtime: ComponentRuntime,
    test_capabilities: CapabilitySet,
}

impl ComponentTestHarness {
    pub async fn test_component(
        &mut self,
        component_bytes: &[u8],
        test_cases: Vec<TestCase>,
    ) -> TestResults {
        let component_id = self.runtime
            .load_component(component_bytes, self.test_capabilities.clone())
            .await?;
            
        let mut results = TestResults::new();
        
        for test_case in test_cases {
            let result = self.execute_test_case(component_id, test_case).await;
            results.add_result(result);
        }
        
        results
    }
}
```

### Security Testing
- **Fuzzing**: Component fuzzing for security vulnerability discovery
- **Capability Testing**: Automated testing of capability enforcement
- **Resource Limit Testing**: Testing of resource limits and enforcement
- **Penetration Testing**: Security testing of component sandbox

## Monitoring and Observability

### Component Metrics
- **Execution Time**: Function execution time and component performance
- **Resource Usage**: Memory, CPU, and I/O usage per component
- **Communication Patterns**: Inter-component communication analysis
- **Security Events**: Capability violations and security incidents

### Integration with AirsSys Monitoring
```rust
// Monitoring integration with airssys-osl logging
pub struct WASMMonitor {
    osl_logger: airssys_osl::ActivityLogger,
    metrics_collector: MetricsCollector,
}

impl WASMMonitor {
    pub async fn log_component_event(
        &self,
        component_id: ComponentId,
        event: ComponentEvent,
    ) -> Result<(), MonitorError> {
        // Log to airssys-osl for unified logging
        self.osl_logger.log_activity(ActivityEvent::ComponentEvent {
            component_id,
            event: event.clone(),
            timestamp: Utc::now(),
        }).await?;
        
        // Collect metrics for performance analysis
        self.metrics_collector.record_component_event(component_id, event).await?;
        
        Ok(())
    }
}
```