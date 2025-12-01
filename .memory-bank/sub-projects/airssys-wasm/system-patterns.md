# airssys-wasm System Patterns

## Component Architecture

### WASM Component System
```
┌─────────────────────────────────────────┐
│         Component Registry              │ ← Component discovery
├─────────────────────────────────────────┤
│         Component Host                  │ ← Component lifecycle
├─────────────────────────────────────────┤
│         Security Sandbox                │ ← Capability enforcement
├─────────────────────────────────────────┤
│         WASM Runtime                    │ ← WASM execution
├─────────────────────────────────────────┤
│         WASI Implementation             │ ← System interface
├─────────────────────────────────────────┤
│         AirsSys Integration             │ ← OS/RT integration
└─────────────────────────────────────────┘
```

## Core Implementation Patterns

### Component Host Pattern
```rust
use std::collections::HashMap;
use wasmtime::{Engine, Module, Instance, Store};
use chrono::{DateTime, Utc};

use crate::security::CapabilitySet;
use crate::component::ComponentId;

pub struct ComponentHost {
    engine: Engine,
    components: HashMap<ComponentId, ComponentInstance>,
    security_enforcer: SecurityEnforcer,
}

pub struct ComponentInstance {
    instance: Instance,
    store: Store<ComponentContext>,
    capabilities: CapabilitySet,
    created_at: DateTime<Utc>,
}

impl ComponentHost {
    pub async fn load_component(
        &mut self,
        component_bytes: &[u8],
        capabilities: CapabilitySet,
    ) -> Result<ComponentId, WasmError> {
        // Security validation
        self.security_enforcer.validate_component(component_bytes)?;
        
        // Module compilation
        let module = Module::from_binary(&self.engine, component_bytes)?;
        
        // Context creation with capabilities
        let context = ComponentContext::new(capabilities.clone());
        let mut store = Store::new(&self.engine, context);
        
        // Instance creation with capability enforcement
        let instance = Instance::new(&mut store, &module, &[])?;
        
        let component_id = ComponentId::new();
        let component_instance = ComponentInstance {
            instance,
            store,
            capabilities,
            created_at: Utc::now(),
        };
        
        self.components.insert(component_id, component_instance);
        Ok(component_id)
    }
}
```

### Capability-Based Security Pattern
```rust
use std::collections::HashSet;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitySet {
    pub file_system: FileSystemCapabilities,
    pub network: NetworkCapabilities,
    pub process: ProcessCapabilities,
    pub custom: HashSet<CustomCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSystemCapabilities {
    pub readable_paths: Vec<PathPattern>,
    pub writable_paths: Vec<PathPattern>,
    pub executable_paths: Vec<PathPattern>,
}

pub struct SecurityEnforcer {
    policy_engine: PolicyEngine,
}

impl SecurityEnforcer {
    pub fn check_file_access(
        &self,
        component_id: ComponentId,
        path: &Path,
        access_type: AccessType,
    ) -> Result<(), SecurityError> {
        let capabilities = self.get_component_capabilities(component_id)?;
        
        match access_type {
            AccessType::Read => {
                if !capabilities.file_system.can_read(path) {
                    return Err(SecurityError::AccessDenied {
                        component_id,
                        resource: path.to_string(),
                        access_type,
                    });
                }
            }
            AccessType::Write => {
                if !capabilities.file_system.can_write(path) {
                    return Err(SecurityError::AccessDenied {
                        component_id,
                        resource: path.to_string(),
                        access_type,
                    });
                }
            }
        }
        Ok(())
    }
}
```

### Component Communication Pattern
```rust
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMessage {
    pub from: ComponentId,
    pub to: ComponentId,
    pub payload: MessagePayload,
    pub timestamp: DateTime<Utc>,
}

pub struct ComponentBridge {
    message_router: MessageRouter,
    component_registry: ComponentRegistry,
}

impl ComponentBridge {
    pub async fn send_message(
        &self,
        from: ComponentId,
        to: ComponentId,
        payload: MessagePayload,
    ) -> Result<(), CommunicationError> {
        // Security check - can 'from' component communicate with 'to'?
        self.check_communication_permission(from, to)?;
        
        // Message serialization and routing
        let message = ComponentMessage {
            from,
            to,
            payload,
            timestamp: Utc::now(),
        };
        
        self.message_router.route_message(message).await?;
        Ok(())
    }
}

// Zero-copy communication for large data
#[derive(Debug, Clone)]
pub struct SharedMemoryRegion {
    data: Arc<[u8]>,
    permissions: MemoryPermissions,
    component_access: HashSet<ComponentId>,
}

impl SharedMemoryRegion {
    pub fn grant_access(
        &mut self,
        component_id: ComponentId,
        permissions: MemoryPermissions,
    ) -> Result<(), SecurityError> {
        // Validate component has permission to access shared memory
        self.validate_access_grant(component_id, permissions)?;
        self.component_access.insert(component_id);
        Ok(())
    }
}
```

## Integration Patterns

### airssys-osl Integration
```rust
use airssys_osl::security::SecurityContext;
use airssys_osl::fs::FileSystemManager;

pub struct OSLIntegration {
    osl_security: SecurityContext,
    file_manager: FileSystemManager,
}

impl OSLIntegration {
    pub async fn handle_file_operation(
        &self,
        component_id: ComponentId,
        operation: FileOperation,
    ) -> Result<FileResult, WasmError> {
        // Component capability check
        self.check_component_capabilities(component_id, &operation)?;
        
        // Delegate to airssys-osl with component context
        let secure_operation = self.create_secure_operation(component_id, operation);
        let result = self.file_manager.execute_operation(secure_operation).await?;
        
        Ok(result)
    }
}
```

### airssys-rt Integration  
```rust
use airssys_rt::{Actor, ActorContext};

pub struct ComponentActor {
    component_host: ComponentHost,
    component_id: ComponentId,
}

#[async_trait::async_trait]
impl Actor for ComponentActor {
    type Message = ComponentActorMessage;
    type State = ComponentState;
    
    async fn handle_message(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext,
    ) -> Result<(), ActorError> {
        match message {
            ComponentActorMessage::ExecuteFunction { function_name, args } => {
                let result = self.component_host
                    .call_component_function(self.component_id, function_name, args)
                    .await?;
                context.reply(ComponentActorMessage::FunctionResult(result)).await?;
            }
            ComponentActorMessage::ComponentMessage(msg) => {
                self.component_host
                    .deliver_message(self.component_id, msg)
                    .await?;
            }
        }
        Ok(())
    }
}
```

## Performance Optimization Patterns

### Component Pool Pattern
```rust
pub struct ComponentPool {
    available: Vec<ComponentInstance>,
    in_use: HashMap<ComponentId, ComponentInstance>,
    template: ComponentTemplate,
}

impl ComponentPool {
    pub async fn acquire_component(&mut self) -> Result<ComponentId, PoolError> {
        match self.available.pop() {
            Some(instance) => {
                let component_id = ComponentId::new();
                self.in_use.insert(component_id, instance);
                Ok(component_id)
            }
            None => {
                // Create new instance from template
                let instance = self.template.instantiate().await?;
                let component_id = ComponentId::new();
                self.in_use.insert(component_id, instance);
                Ok(component_id)
            }
        }
    }
    
    pub fn release_component(&mut self, component_id: ComponentId) -> Result<(), PoolError> {
        if let Some(mut instance) = self.in_use.remove(&component_id) {
            // Reset instance state
            instance.reset().await?;
            self.available.push(instance);
        }
        Ok(())
    }
}
```

### Streaming Data Pattern
```rust
use tokio::io::{AsyncRead, AsyncWrite};

pub struct StreamingInterface {
    input_stream: Box<dyn AsyncRead + Send + Unpin>,
    output_stream: Box<dyn AsyncWrite + Send + Unpin>,
}

impl StreamingInterface {
    pub async fn process_stream(
        &mut self,
        component_id: ComponentId,
        processor_function: &str,
    ) -> Result<(), StreamError> {
        // Stream data through WASM component without loading entire dataset
        let mut buffer = [0u8; 8192];
        
        loop {
            let bytes_read = self.input_stream.read(&mut buffer).await?;
            if bytes_read == 0 {
                break;
            }
            
            let processed = self.component_host
                .call_streaming_function(component_id, processor_function, &buffer[..bytes_read])
                .await?;
                
            self.output_stream.write_all(&processed).await?;
        }
        
        self.output_stream.flush().await?;
        Ok(())
    }
}
```