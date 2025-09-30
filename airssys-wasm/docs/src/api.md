# API Reference

This section provides comprehensive API documentation for airssys-wasm components, interfaces, and runtime features.

## Core APIs

### Component Interface
The primary interface for creating and managing WASM components within the airssys-wasm framework.

#### Component Trait
Core trait that all components must implement:
```rust
pub trait Component {
    type Input;
    type Output;
    type Error;
    
    fn execute(&self, input: Self::Input) -> Result<Self::Output, Self::Error>;
}
```

### Runtime APIs

#### Hot Deployment API
```rust
pub struct DeploymentManager {
    // Hot deployment management
}

impl DeploymentManager {
    pub fn deploy(&self, component: Component, strategy: DeploymentStrategy) -> Result<(), Error>;
    pub fn rollback(&self, component_id: &str, version: Version) -> Result<(), Error>;
    pub fn get_status(&self, component_id: &str) -> DeploymentStatus;
}
```

#### Capability Management API
```rust
pub struct CapabilityManager {
    // Capability-based security management
}

impl CapabilityManager {
    pub fn grant_capability(&self, component_id: &str, capability: Capability) -> Result<(), Error>;
    pub fn revoke_capability(&self, component_id: &str, capability: Capability) -> Result<(), Error>;
    pub fn check_capability(&self, component_id: &str, capability: &Capability) -> bool;
}
```

## WIT Interfaces

### Standard Component Interface
```wit
package airssys:component;

interface component {
    /// Standard component execution
    execute: func(input: list<u8>) -> result<list<u8>, string>;
    
    /// Component metadata
    get-metadata: func() -> component-metadata;
    
    /// Health check
    health-check: func() -> health-status;
}

record component-metadata {
    name: string,
    version: string,
    capabilities: list<string>,
}

enum health-status {
    healthy,
    degraded,
    unhealthy,
}
```

### Capability Interface
```wit
package airssys:capability;

interface capability {
    /// Request capability
    request-capability: func(capability: string) -> result<capability-handle, string>;
    
    /// Release capability
    release-capability: func(handle: capability-handle);
}

resource capability-handle;
```

## SDK Macros

### Component Derive Macro
```rust
#[derive(Component)]
#[component(
    name = "example-component",
    version = "1.0.0",
    capabilities = ["filesystem", "network"]
)]
pub struct ExampleComponent {
    // Component implementation
}
```

### Interface Binding Macro
```rust
#[interface_binding("airssys:component/component")]
impl ComponentInterface for ExampleComponent {
    // Auto-generated WIT bindings
}
```

## Error Types

### Framework Errors
```rust
#[derive(Debug, Error)]
pub enum FrameworkError {
    #[error("Component not found: {0}")]
    ComponentNotFound(String),
    
    #[error("Deployment failed: {0}")]
    DeploymentFailed(String),
    
    #[error("Capability denied: {0}")]
    CapabilityDenied(String),
    
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
}
```

This API reference provides the essential interfaces for building, deploying, and managing components within the airssys-wasm framework.
