# KNOWLEDGE-WASM-013: Core WIT Package Structure and Host-Component Contract

**Created:** 2025-11-24  
**Status:** Active  
**Category:** Architecture / WIT Interface Design  
**Related ADRs:** ADR-WASM-005 (Capability-Based Security)  
**Related Tasks:** WASM-TASK-003 (Block 2: WIT Interface System)

---

## Overview

This document explains the structure and purpose of the `airssys:core@1.0.0` WIT package, which serves as the foundational contract between the WASM host runtime and component implementations. It clarifies the bidirectional nature of the interface definitions and the responsibilities of each party.

---

## Package Structure

The `airssys:core@1.0.0` package is organized into **4 layered interfaces**, each serving a distinct architectural purpose:

```
airssys-wasm/wit/core/
├── types.wit                    # Layer 0: Foundation types
├── capabilities.wit             # Layer 1: Security model
├── component-lifecycle.wit      # Layer 2: Component contract
└── host-services.wit           # Layer 3: Host services
```

### Layer Dependencies

```
types.wit (Layer 0 - Foundation)
    ↓
capabilities.wit (Layer 1 - Security)
    ↓
component-lifecycle.wit (Layer 2 - Component Management)
    ↓
host-services.wit (Layer 3 - Host Integration)
```

Each layer imports types from previous layers using WIT `use` statements, creating a clean dependency hierarchy with no circular dependencies.

---

## Layer 0: `types.wit` - The Shared Vocabulary

### Purpose
Defines the common data structures and types used throughout the framework. This is the **single source of truth** for all foundation types.

### Key Types

#### Identity Types
- **`component-id`**: Unique identifier with namespace, name, and version
- **`request-id`**: Correlation identifier for async operations
- **`timestamp`**: High-precision time representation (seconds + nanoseconds)

#### Error Types
- **`component-error`**: Lifecycle and initialization errors
- **`execution-error`**: Operation execution errors
- **`file-error`**: Filesystem operation errors
- **`network-error`**: Network operation errors
- **`process-error`**: Process operation errors

#### Status Types
- **`health-status`**: Component health enumeration (healthy/degraded/unhealthy/unknown)
- **`log-level`**: Logging severity levels (trace/debug/info/warn/error)
- **`execution-status`**: Execution result tracking (success/failed/timeout/cancelled)

### Usage
All other interfaces import types from this interface using:
```wit
use types.{component-id, request-id, ...};
```

---

## Layer 1: `capabilities.wit` - The Security Model

### Purpose
Defines the capability-based security model that governs what operations components are permitted to perform.

### Permission Types

#### Filesystem Permissions
- **`filesystem-permission`**: Action + path pattern
- **`filesystem-action`**: read, write, delete, list-dir

#### Network Permissions
- **`network-permission`**: Action + host pattern + optional port
- **`network-action`**: outbound, inbound

#### Process Permissions
- **`process-permission`**: Action + command pattern
- **`process-action`**: spawn, kill, signal

### Permission Aggregation
- **`requested-permissions`**: Complete set of permissions a component requests
- **`permission-result`**: Grant/deny result from host validation

### Usage by Both Parties
- **Components**: Declare required permissions in their manifest
- **Host**: Validates permission requests against security policies

---

## Layer 2: `component-lifecycle.wit` - The Component Contract

### Purpose
Defines the **interface that components must implement** to participate in the framework. This is the contract that allows the host to manage component lifecycles uniformly.

### Implementation Direction
**Components implement → Host calls**

### Lifecycle Functions

#### Initialization
```wit
init: func(config: component-config) -> result<_, component-error>;
```
Called by host to initialize the component with configuration and resource limits.

#### Execution
```wit
execute: func(
    operation: list<u8>,
    context: execution-context
) -> result<list<u8>, execution-error>;
```
Called by host to execute RPC operations on the component.

#### Messaging
```wit
handle-message: func(
    sender: component-id,
    message: list<u8>
) -> result<_, component-error>;
```
Called by host when another component sends a message to this component.

#### Callbacks
```wit
handle-callback: func(
    request-id: request-id,
    callback-result: result<list<u8>, string>
) -> result<_, component-error>;
```
Called by host to deliver async callback results from previous requests.

#### Introspection
```wit
metadata: func() -> component-metadata;
health: func() -> health-status;
```
Called by host to query component metadata and health status.

#### Shutdown
```wit
shutdown: func() -> result<_, component-error>;
```
Called by host to gracefully shutdown the component.

### Key Insight
The component is **passive** - it exposes these functions and waits for the host to call them. The component cannot initiate actions on its own; it can only respond to host requests.

---

## Layer 3: `host-services.wit` - The Host Services

### Purpose
Defines the **interface that the host implements** to provide services to components. This is the "toolbox" that sandboxed components use to interact with the outside world.

### Implementation Direction
**Host implements → Components call**

### Service Categories

#### Logging
```wit
log: func(
    level: log-level,
    message: string,
    context: option<list<tuple<string, string>>>
);
```
Allows components to emit structured log messages to the host's logging system.

#### Inter-Component Messaging
```wit
send-message: func(
    target: component-id,
    message: list<u8>
) -> result<_, messaging-error>;

send-request: func(
    target: component-id,
    request: list<u8>,
    timeout-ms: u64
) -> result<request-id, messaging-error>;

cancel-request: func(
    request-id: request-id
) -> result<_, messaging-error>;
```
Enables fire-and-forget messaging and request-response patterns between components.

#### Timing Services
```wit
current-time-millis: func() -> u64;
sleep-millis: func(duration-ms: u64);
```
Provides time access and sleep functionality to components.

#### Component Introspection
```wit
list-components: func() -> list<component-id>;
get-component-metadata: func(id: component-id) -> result<component-metadata, component-error>;
```
Allows components to discover and query other loaded components.

### Key Insight
Components are **sandboxed** and cannot directly access system resources. All external interactions must go through these host-provided services, which enforce security policies and resource limits.

---

## The Bidirectional Contract

### Relationship Diagram

```
┌─────────────────────────────────────────────────┐
│                    HOST                         │
│                                                 │
│  IMPLEMENTS:                                    │
│    • host-services.wit (provides services)      │
│                                                 │
│  CALLS:                                         │
│    • component-lifecycle.wit (manages lifecycle)│
│                                                 │
│  VALIDATES:                                     │
│    • capabilities.wit (enforces security)       │
│                                                 │
└─────────────────────────────────────────────────┘
                      ↕
          (Uses shared types.wit)
                      ↕
┌─────────────────────────────────────────────────┐
│                 COMPONENT                       │
│                                                 │
│  IMPLEMENTS:                                    │
│    • component-lifecycle.wit (exposes API)      │
│                                                 │
│  CALLS:                                         │
│    • host-services.wit (uses services)          │
│                                                 │
│  DECLARES:                                      │
│    • capabilities.wit (requests permissions)    │
│                                                 │
└─────────────────────────────────────────────────┘
```

### Interaction Flow

1. **Component Installation**
   - Component declares required permissions in manifest (capabilities.wit)
   - Host validates permissions against security policies

2. **Component Initialization**
   - Host calls `component-lifecycle.init()` with configuration
   - Component initializes internal state and returns success/error

3. **Component Execution**
   - Host calls `component-lifecycle.execute()` with operation data
   - Component processes operation, may call `host-services.*` functions
   - Component returns result to host

4. **Inter-Component Communication**
   - Component A calls `host-services.send-message()` to send to Component B
   - Host validates permissions and routes message
   - Host calls `component-lifecycle.handle-message()` on Component B
   - Component B processes message

5. **Component Shutdown**
   - Host calls `component-lifecycle.shutdown()`
   - Component performs cleanup and returns

---

## Type Reuse Pattern

### Component Model v0.1 Constraint
Cross-package type imports are **not supported** in Component Model v0.1. However, cross-interface type reuse **within the same package** is fully supported via `use` statements.

### Implementation
All interfaces in `airssys:core@1.0.0` share types through explicit imports:

**Example from `capabilities.wit`:**
```wit
interface capabilities {
    use types.{component-id};
    
    record filesystem-permission {
        action: filesystem-action,
        path-pattern: string,
    }
    // ...
}
```

**Example from `host-services.wit`:**
```wit
interface host-services {
    use types.{component-id, request-id, component-error, log-level, timestamp};
    
    log: func(
        level: log-level,
        message: string,
        context: option<list<tuple<string, string>>>
    );
    // ...
}
```

### Benefits
- **Zero duplication**: Each type defined once in `types.wit`
- **Clear dependencies**: `use` statements document type dependencies
- **Single source of truth**: Changes to types propagate automatically
- **Interface isolation**: Each interface remains focused and cohesive

---

## Design Principles

### 1. Separation of Concerns
Each layer has a single, well-defined responsibility:
- Layer 0: Common vocabulary
- Layer 1: Security model
- Layer 2: Component behavior contract
- Layer 3: Host service contract

### 2. Bidirectional Contract
The package defines interfaces for both host and component, creating a complete contract that governs their interaction.

### 3. Security by Default
All component operations require explicit permission declarations (capabilities.wit), enforcing a deny-by-default security model.

### 4. Language Agnostic
WIT interfaces are language-neutral, enabling components written in Rust, AssemblyScript, TinyGo, or any WASM-compatible language to participate in the framework.

### 5. Versioned Evolution
The package version (`@1.0.0`) allows for controlled evolution of the interface contract over time.

---

## Usage Examples

### Component Implementation (Rust)
```rust
use airssys_wasm_bindings::component_lifecycle::*;

struct MyComponent {
    state: ComponentState,
}

impl ComponentLifecycle for MyComponent {
    fn init(&mut self, config: ComponentConfig) -> Result<(), ComponentError> {
        // Initialize component
        self.state = ComponentState::from_config(config)?;
        Ok(())
    }
    
    fn execute(&mut self, operation: Vec<u8>, context: ExecutionContext) 
        -> Result<Vec<u8>, ExecutionError> {
        // Process operation
        let result = self.state.process(operation)?;
        
        // Call host service
        host_services::log(LogLevel::Info, "Operation completed", None);
        
        Ok(result)
    }
    
    // ... other lifecycle methods
}
```

### Host Implementation (Rust)
```rust
use airssys_wasm::runtime::ComponentInstance;
use airssys_wasm_bindings::host_services::*;

struct WasmHost {
    components: HashMap<ComponentId, ComponentInstance>,
}

impl HostServices for WasmHost {
    fn log(&self, level: LogLevel, message: String, context: Option<Vec<(String, String)>>) {
        // Implement logging
        tracing::log!(level, "{}", message);
    }
    
    fn send_message(&self, target: ComponentId, message: Vec<u8>) 
        -> Result<(), MessagingError> {
        // Route message to target component
        let component = self.components.get(&target)
            .ok_or(MessagingError::ComponentNotFound)?;
        
        component.handle_message(self.current_component_id(), message)?;
        Ok(())
    }
    
    // ... other host services
}
```

---

## Related Documentation

### ADRs
- **ADR-WASM-005**: Capability-Based Security Model - Defines the security architecture
- **ADR-WASM-015**: WIT Package Structure - Original 7-package design (superseded by single-package approach)

### Knowledge Documents
- **KNOWLEDGE-WASM-004**: WIT Management Architecture - Comprehensive WIT design patterns
- **KNOWLEDGE-WASM-001**: Component Framework Architecture - Overall framework design

### Tasks
- **WASM-TASK-003**: Block 2: WIT Interface System - Implementation of this package structure

### Technical Debt
- **DEBT-WASM-003**: Component Model v0.1 Type Import Limitation - Documents cross-package import constraints

---

## Future Evolution

### Component Model v0.2
When Component Model v0.2 is released with cross-package import support, the architecture may be refactored into separate packages:
- `airssys:types@1.0.0` - Foundation types
- `airssys:capabilities@1.0.0` - Security model
- `airssys:component@1.0.0` - Component lifecycle
- `airssys:host@1.0.0` - Host services

This would provide even cleaner separation and allow independent versioning of each concern.

### Extension Packages
The `airssys:ext@1.0.0` package follows the same single-package pattern and provides:
- Filesystem operations interface
- Network operations interface
- Process operations interface

These extend the core package with domain-specific capabilities.

---

## Summary

The `airssys:core@1.0.0` package is the **foundational contract** of the WASM framework, defining:

1. **Shared vocabulary** (types.wit) - Common data structures
2. **Security model** (capabilities.wit) - Permission system
3. **Component contract** (component-lifecycle.wit) - What components must implement
4. **Host services** (host-services.wit) - What the host provides

It establishes a **bidirectional contract** where:
- Components implement lifecycle functions and call host services
- Host implements services and calls component lifecycle functions
- Both parties use shared types and respect the capability-based security model

This design enables **language-agnostic**, **secure**, and **manageable** component-based systems with clear separation of concerns and well-defined interaction patterns.
