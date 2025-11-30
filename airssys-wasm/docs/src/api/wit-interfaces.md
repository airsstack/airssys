# WIT Interfaces Reference

**Document Type**: REFERENCE - API Documentation

This document provides the API reference for airssys-wasm's WebAssembly Interface Types (WIT) system. All interfaces are language-agnostic and generate bindings for Rust, JavaScript, Python, and other WASM-capable languages.

## Overview

The airssys-wasm framework provides **115 operations** across **16 WIT interface files**, organized into two tiers:

1. **Core Package** (`airssys:core@1.0.0`): 15 operations - foundation types, lifecycle, capabilities
2. **Extension Packages** (`airssys:ext-*@1.0.0`): 100 operations - filesystem, network, process

**Complete system documentation**: See [WIT System Architecture](../reference/wit-system-architecture.md) for detailed architectural specifications (1,066 lines).

## Quick Reference

### Core Interfaces

| Interface | Purpose | Operations | Package |
|-----------|---------|------------|---------|
| [types](#types-interface) | Foundation types and error enums | 13 types | `airssys:core` |
| [capabilities](#capabilities-interface) | Permission and capability grants | 10 types | `airssys:core` |
| [component-lifecycle](#component-lifecycle-interface) | Component init, execute, shutdown | 7 functions | `airssys:core` |
| [host-services](#host-services-interface) | Logging, monitoring, configuration | 8 functions | `airssys:core` |

### Extension Interfaces

| Domain | Interfaces | Operations | Package |
|--------|------------|------------|---------|
| [Filesystem](#filesystem-interfaces) | types, filesystem, metadata | 36 ops | `airssys:ext-filesystem` |
| [Network](#network-interfaces) | types, socket, connection | 32 ops | `airssys:ext-network` |
| [Process](#process-interfaces) | types, lifecycle, signals | 32 ops | `airssys:ext-process` |

## Core Package: `airssys:core@1.0.0`

The core package provides fundamental types and services required by all components.

### Types Interface

**Package**: `airssys:core@1.0.0`  
**File**: `wit/core/types.wit` (112 lines)  
**Purpose**: Foundation types - single source of truth for core data structures

#### Identity Types

```wit
// Unique component identifier
type component-id = string;

// Request tracking identifier
type request-id = string;
```

**Usage**: Component identification and request correlation.

#### Timestamp Type

```wit
// UTC timestamp with nanosecond precision
record timestamp {
    seconds: u64,      // Seconds since Unix epoch
    nanoseconds: u32,  // Nanosecond component (0-999,999,999)
}
```

**Example** (Rust):
```rust
use airssys::core::types::Timestamp;

let now = Timestamp {
    seconds: 1730000000,
    nanoseconds: 123456789,
};
```

#### Error Types

**Component Errors**:
```wit
enum component-error {
    invalid-config,        // Configuration parsing failed
    invalid-input,         // Input validation failed
    permission-denied,     // Capability check failed
    resource-not-found,    // Required resource missing
    execution-failed,      // Operation execution error
    timeout,              // Operation exceeded time limit
    unknown,              // Unspecified error
}
```

**Execution Errors**:
```wit
enum execution-error {
    panic,                // Component panicked
    trap,                 // WebAssembly trap
    out-of-memory,        // Memory allocation failed
    resource-exhausted,   // Resource limit exceeded
    invalid-operation,    // Operation not allowed in current state
}
```

**File Errors**:
```wit
enum file-error {
    not-found,           // File does not exist
    permission-denied,   // Insufficient permissions
    already-exists,      // File already exists (create exclusive)
    is-directory,        // Expected file, found directory
    not-directory,       // Expected directory, found file
    io-error,           // Generic I/O error
    invalid-path,       // Malformed path
}
```

**Network Errors**:
```wit
enum network-error {
    connection-refused,  // Remote host refused connection
    connection-reset,    // Connection reset by peer
    connection-aborted,  // Connection aborted
    not-connected,       // Socket not connected
    already-connected,   // Socket already connected
    timeout,            // Operation timed out
    dns-failed,         // DNS resolution failed
    invalid-address,    // Invalid address format
}
```

**Process Errors**:
```wit
enum process-error {
    not-found,          // Executable not found
    permission-denied,  // Insufficient permissions to spawn
    invalid-args,       // Invalid arguments
    spawn-failed,       // Process spawn failed
    already-finished,   // Process already terminated
    io-error,          // I/O operation failed
}
```

#### Status Types

```wit
// Component health status
enum health-status {
    healthy,            // Operating normally
    degraded,           // Operating with reduced capacity
    unhealthy,          // Not operating correctly
    unknown,            // Status cannot be determined
}

// Log severity levels
enum log-level {
    trace,              // Detailed trace information
    debug,              // Debug information
    info,               // Informational messages
    warn,               // Warning messages
    error,              // Error messages
    fatal,              // Fatal error messages
}

// Component execution status
enum execution-status {
    initialized,        // Component initialized
    running,           // Currently executing
    completed,         // Execution completed successfully
    failed,            // Execution failed
    terminated,        // Execution terminated
}
```

### Capabilities Interface

**Package**: `airssys:core@1.0.0`  
**File**: `wit/core/capabilities.wit` (89 lines)  
**Purpose**: Permission model and capability grants

```wit
use types.{component-id, error};

interface capabilities {
    // Capability grant representation
    record capability {
        id: string,
        component: component-id,
        permission-type: permission-type,
        resource: string,
        granted-at: timestamp,
    }
    
    // Permission type enum
    enum permission-type {
        filesystem-read,
        filesystem-write,
        filesystem-delete,
        network-connect,
        network-bind,
        process-spawn,
        process-signal,
    }
    
    // Grant capability to component
    grant-capability: func(
        component: component-id,
        capability: capability
    ) -> result<_, error>;
    
    // Revoke capability from component
    revoke-capability: func(
        component: component-id,
        capability-id: string
    ) -> result<_, error>;
    
    // Check if component has capability
    has-capability: func(
        component: component-id,
        permission: permission-type,
        resource: string
    ) -> bool;
}
```

**Usage Pattern** (Rust):
```rust
use airssys::core::capabilities::{Guest, Capability, PermissionType};

impl Guest for MyComponent {
    fn grant_capability(
        component: ComponentId,
        capability: Capability
    ) -> Result<(), Error> {
        // Framework implementation - validates and grants capability
        Ok(())
    }
    
    fn has_capability(
        component: ComponentId,
        permission: PermissionType,
        resource: String
    ) -> bool {
        // Check Component.toml declarations
        true
    }
}
```

### Component Lifecycle Interface

**Package**: `airssys:core@1.0.0`  
**File**: `wit/core/component-lifecycle.wit` (105 lines)  
**Purpose**: Component initialization, execution, and shutdown

```wit
use types.{component-error, execution-error};

interface component-lifecycle {
    // Component configuration
    record component-config {
        data: list<u8>,      // Configuration data (TOML/JSON)
        metadata: list<u8>,  // Optional metadata
    }
    
    // Component input
    record component-input {
        data: list<u8>,      // Input data
        metadata: list<u8>,  // Optional metadata
    }
    
    // Component output
    record component-output {
        data: list<u8>,      // Output data
        metadata: list<u8>,  // Optional metadata
    }
    
    // Initialize component with configuration
    init: func(config: component-config) -> result<_, component-error>;
    
    // Execute component with input data
    execute: func(input: component-input) -> result<component-output, execution-error>;
    
    // Shutdown component and cleanup resources
    shutdown: func() -> result<_, component-error>;
}
```

**Required Implementation** (Rust):
```rust
use airssys::core::component_lifecycle::{Guest, ComponentConfig, ComponentInput, ComponentOutput};
use airssys::core::types::{ComponentError, ExecutionError};

struct MyComponent;

impl Guest for MyComponent {
    fn init(config: ComponentConfig) -> Result<(), ComponentError> {
        // 1. Parse configuration
        // 2. Initialize resources
        // 3. Validate permissions
        Ok(())
    }
    
    fn execute(input: ComponentInput) -> Result<ComponentOutput, ExecutionError> {
        // 1. Validate input
        // 2. Process data
        // 3. Return output
        Ok(ComponentOutput {
            data: vec![],
            metadata: vec![],
        })
    }
    
    fn shutdown() -> Result<(), ComponentError> {
        // 1. Close resources
        // 2. Flush buffers
        // 3. Release locks
        Ok(())
    }
}
```

### Host Services Interface

**Package**: `airssys:core@1.0.0`  
**File**: `wit/core/host-services.wit` (88 lines)  
**Purpose**: Host-provided services (logging, monitoring, configuration)

```wit
use types.{component-id, timestamp, log-level, health-status};

interface host-services {
    // Log message from component
    log: func(level: log-level, message: string);
    
    // Report component health
    report-health: func(status: health-status, message: string);
    
    // Get configuration value
    get-config: func(key: string) -> option<string>;
    
    // Set configuration value
    set-config: func(key: string, value: string) -> result<_, error>;
    
    // Get current timestamp
    now: func() -> timestamp;
    
    // Sleep for duration (milliseconds)
    sleep: func(duration-ms: u64);
    
    // Get component ID
    component-id: func() -> component-id;
    
    // Emit metric
    emit-metric: func(name: string, value: f64, tags: list<tuple<string, string>>);
}
```

**Usage Example** (Rust):
```rust
use airssys::core::host_services::Guest as HostGuest;
use airssys::core::types::LogLevel;

// Log from component
HostGuest::log(LogLevel::Info, "Component initialized".to_string());

// Report health
HostGuest::report_health(
    HealthStatus::Healthy,
    "All systems operational".to_string()
);

// Get configuration
if let Some(api_key) = HostGuest::get_config("api_key".to_string()) {
    // Use API key
}

// Emit metric
HostGuest::emit_metric(
    "requests_processed".to_string(),
    42.0,
    vec![("status".to_string(), "success".to_string())]
);
```

## Extension Package: Filesystem

**Package**: `airssys:ext-filesystem@1.0.0`  
**Files**: 3 interfaces, 371 lines  
**Operations**: 36 functions

### Filesystem Types

```wit
// File descriptor handle
type file-descriptor = u32;

// File open flags
flags open-flags {
    read,           // Open for reading
    write,          // Open for writing
    create,         // Create if doesn't exist
    exclusive,      // Fail if exists (with create)
    truncate,       // Truncate to zero length
    append,         // Append mode
}

// File permissions
record file-mode {
    user-read: bool,
    user-write: bool,
    user-execute: bool,
    group-read: bool,
    group-write: bool,
    group-execute: bool,
    other-read: bool,
    other-write: bool,
    other-execute: bool,
}

// File metadata
record file-metadata {
    size: u64,
    is-file: bool,
    is-directory: bool,
    is-symlink: bool,
    created-at: timestamp,
    modified-at: timestamp,
    accessed-at: timestamp,
    permissions: file-mode,
}
```

### Filesystem Operations

**Core operations** (22 functions):
- `open(path, flags, mode)` - Open/create file
- `close(fd)` - Close file descriptor
- `read(fd, buffer, count)` - Read bytes
- `write(fd, data)` - Write bytes
- `seek(fd, offset, whence)` - Seek to position
- `delete(path)` - Delete file
- `rename(old-path, new-path)` - Rename/move file
- `copy(src, dest)` - Copy file
- `exists(path)` - Check file existence
- `canonicalize(path)` - Resolve absolute path

**Metadata operations** (14 functions):
- `metadata(path)` - Get file metadata
- `set-permissions(path, mode)` - Set file permissions
- `create-dir(path)` - Create directory
- `remove-dir(path)` - Remove empty directory
- `remove-dir-all(path)` - Remove directory recursively
- `read-dir(path)` - List directory contents
- `read-link(path)` - Read symbolic link target
- `create-symlink(target, link)` - Create symbolic link

Complete API documentation: See [WIT System Architecture - Filesystem](../reference/wit-system-architecture.md#filesystem-extension).

## Extension Package: Network

**Package**: `airssys:ext-network@1.0.0`  
**Files**: 3 interfaces, 422 lines  
**Operations**: 32 functions

### Network Types

```wit
// Socket handle
type socket = u32;

// Socket types
enum socket-type {
    stream,         // TCP socket
    datagram,       // UDP socket
}

// Socket address
record socket-address {
    host: string,   // Hostname or IP address
    port: u16,      // Port number
}

// IP address
variant ip-address {
    ipv4(tuple<u8, u8, u8, u8>),
    ipv6(tuple<u16, u16, u16, u16, u16, u16, u16, u16>),
}
```

### Network Operations

**Socket operations** (20 functions):
- `create-socket(type)` - Create socket
- `close(socket)` - Close socket
- `bind(socket, address)` - Bind to address
- `listen(socket, backlog)` - Listen for connections
- `accept(socket)` - Accept connection
- `send(socket, data, flags)` - Send data
- `receive(socket, buffer, flags)` - Receive data
- `shutdown(socket, how)` - Shutdown connection

**Connection operations** (12 functions):
- `connect(socket, address)` - Connect to remote host
- `disconnect(socket)` - Disconnect socket
- `resolve(hostname)` - DNS resolution
- `get-peer-address(socket)` - Get remote address
- `get-local-address(socket)` - Get local address
- `set-socket-option(socket, option, value)` - Configure socket

Complete API documentation: See [WIT System Architecture - Network](../reference/wit-system-architecture.md#network-extension).

## Extension Package: Process

**Package**: `airssys:ext-process@1.0.0`  
**Files**: 3 interfaces, 440 lines  
**Operations**: 32 functions

### Process Types

```wit
// Process handle
type process = u32;

// Process exit status
variant exit-status {
    success,
    failure(i32),
    signal(string),
}

// Standard I/O configuration
enum stdio-config {
    inherit,        // Inherit from parent
    null,          // Discard
    pipe,          // Create pipe
}

// Process spawn configuration
record spawn-config {
    executable: string,
    args: list<string>,
    env: list<tuple<string, string>>,
    stdin: stdio-config,
    stdout: stdio-config,
    stderr: stdio-config,
}
```

### Process Operations

**Lifecycle operations** (18 functions):
- `spawn(config)` - Spawn new process
- `wait(process)` - Wait for process completion
- `try-wait(process)` - Non-blocking wait
- `kill(process)` - Terminate process
- `is-running(process)` - Check if process is running
- `get-exit-status(process)` - Get exit code
- `read-stdout(process)` - Read standard output
- `read-stderr(process)` - Read standard error
- `write-stdin(process, data)` - Write to standard input

**Signal operations** (14 functions):
- `send-signal(process, signal)` - Send signal to process
- `wait-for-signal(signals)` - Wait for signal
- `block-signals(signals)` - Block signals
- `unblock-signals(signals)` - Unblock signals

Complete API documentation: See [WIT System Architecture - Process](../reference/wit-system-architecture.md#process-extension).

## Binding Generation

### Rust Bindings

Automatically generated via `wit-bindgen` in `build.rs`:

```rust
// build.rs
wit_bindgen::generate!({
    path: "path/to/wit",
    world: "component",
    exports: { world: true },
});

// src/lib.rs - Use generated bindings
wit_bindgen::generate!("component");

use crate::exports::airssys::core::component_lifecycle::Guest;
use crate::airssys::core::types::*;
use crate::airssys::ext::filesystem::*;
```

### JavaScript Bindings

Generated via `jco` (JavaScript Component Tools):

```javascript
import { init, execute, shutdown } from './my-component.js';

// Component lifecycle
await init({ data: new Uint8Array(), metadata: new Uint8Array() });
const output = await execute({ data: inputData, metadata: new Uint8Array() });
await shutdown();
```

### Python Bindings

Generated via `componentize-py`:

```python
from airssys_core import ComponentLifecycle
from airssys_core.types import ComponentConfig, ComponentInput

class MyComponent(ComponentLifecycle):
    def init(self, config: ComponentConfig) -> None:
        # Initialize component
        pass
    
    def execute(self, input: ComponentInput) -> ComponentOutput:
        # Process data
        return ComponentOutput(data=result, metadata=[])
    
    def shutdown(self) -> None:
        # Cleanup
        pass
```

## Validation

### WIT Validation

Validate WIT interfaces with `wasm-tools`:

```bash
# Validate all packages
cd wit && wasm-tools component wit .

# Validate specific package
wasm-tools component wit wit/core

# Validate component binary
wasm-tools validate my-component.wasm --features component-model
```

### Interface Inspection

Inspect component exports:

```bash
# View exported interfaces
wasm-tools component wit my-component.wasm

# View complete component structure
wasm-tools print my-component.wasm
```

## Related Documentation

- **[WIT System Architecture](../reference/wit-system-architecture.md)**: Complete 1,066-line architectural reference
- **[Getting Started](../implementation/getting-started.md)**: Tutorial with WIT interface usage
- **[Component Development](../implementation/component-development.md)**: Patterns for using interfaces
- **[Component.toml Specification](../reference/component-toml-spec.md)**: Permission declarations
- **[WebAssembly Component Model](https://component-model.bytecodealliance.org/)**: Official specification
- **[WIT Format Specification](https://component-model.bytecodealliance.org/design/wit.html)**: WIT language reference

## Version Information

- **Core Package**: `airssys:core@1.0.0`
- **Extension Packages**: `airssys:ext-*@1.0.0`
- **Total Interfaces**: 16 files
- **Total Operations**: 115 functions
- **WIT Lines**: 2,214 lines
- **Last Updated**: 2025-11-29
