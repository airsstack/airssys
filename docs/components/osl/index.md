# OSL - OS Layer Framework

**Secure, cross-platform abstraction over operating system functionality with comprehensive audit trails and security policy enforcement.**

## Vision

Create a secure foundation for system programming that makes secure OS operations as easy to use as direct system calls, while maintaining comprehensive audit trails and enforcing security policies by default.

## Motivation

The need for `airssys-osl` emerged from real-world challenges building AI-powered tools that require OS access:

### The Problem

When building MCP (Model Context Protocol) servers that give AI models access to local filesystems, several critical challenges emerged:

1. **Security Risks**: Direct OS access (using `std::fs`, `std::process`) provides no security boundaries. A mistake or malicious prompt could read sensitive files, spawn dangerous processes, or access network resources without restriction.

2. **No Audit Trail**: Standard OS operations provide no visibility into what was accessed, when, or by whom. For AI systems making autonomous decisions, this lack of observability is unacceptable.

3. **Repeated Security Logic**: Each tool reimplemented similar security validators to prevent harmful activities (binary file access, path traversal, etc.), leading to inconsistent protection and maintenance overhead.

4. **Limited Control**: OS-level operations lack fine-grained control mechanisms like rate limiting, resource quotas, or conditional access based on context.

### Real-World Example

Consider an MCP tool that allows an AI to read and write files:

**Without OSL (Unsafe)**:
```rust
// Direct access - no security, no logging, no control
std::fs::write("/etc/passwd", malicious_data)?; // 💥 System compromised
```

**With OSL (Secure)**:
```rust
use airssys_osl::helpers::*;

// Security policies enforced, operation logged, controlled access
write_file("/etc/passwd", data, "ai-assistant").await?;
// ❌ Denied: ACL policy blocks system file access
// ✅ Logged: Attempted access recorded with timestamp and principal
```

### The Solution

`airssys-osl` provides a **secure-by-default OS abstraction layer** with three key innovations:

1. **Built-in Security**: Every operation goes through ACL/RBAC policies (deny-by-default)
2. **Comprehensive Logging**: All activities logged with security context for audit trails
3. **Middleware Pipeline**: Extensible architecture for custom security, rate limiting, caching, and more

This makes building secure AI tools, MCP servers, and system utilities **as simple as direct OS calls, but with enterprise-grade security and observability**.

## Key Features

### 🛡️ Security by Default

- **ACL (Access Control Lists)**: Path-based access control with glob pattern matching
- **RBAC (Role-Based Access Control)**: Role hierarchies with permission inheritance
- **Deny-by-Default**: Operations denied unless explicitly allowed by policy
- **Audit Trails**: JSON-formatted security logs for all operations

### 📊 Comprehensive Logging

- **Activity Logging**: Every operation logged with timestamp, principal, operation type
- **Multiple Loggers**: Console, File, Tracing (OpenTelemetry-compatible)
- **Security Events**: Dedicated audit logs for policy denials and security violations
- **Structured Formats**: JSON logs for easy integration with monitoring systems

### 🔧 Flexible Architecture

- **Three API Levels**:
  1. **Helper Functions** (Level 1): Simple, one-line operations with sensible defaults
  2. **Custom Middleware** (Level 2): Advanced middleware composition for custom policies
  3. **Direct Executors** (Level 3): Maximum control for specialized use cases

- **Middleware Pipeline**: Chain multiple middleware for logging, security, rate limiting, caching
- **Custom Executors**: Implement custom executors using procedural macros

### 🌐 Cross-Platform

- **Unified API**: Same interface across Linux, macOS, Windows
- **OS-Specific Optimizations**: Platform-specific implementations where needed
- **Async-First**: Built on Tokio for efficient async operations

### ⚡ Performance

- **Zero-Cost Middleware**: Middleware compiles to direct calls (no dynamic dispatch)
- **Efficient Operations**: Minimal overhead over direct OS calls
- **Configurable Tradeoffs**: Choose security vs. performance based on your needs

## Use Cases

### AI & MCP Servers

Build AI tools that safely access OS resources:

```rust
use airssys_osl::helpers::*;
use airssys_osl::middleware::security::*;

// Create ACL policy for AI assistant
let acl = AccessControlList::new()
    .add_entry(AclEntry::new(
        "ai-assistant".to_string(),
        "/workspace/*".to_string(),       // Only workspace access
        vec!["read".to_string(), "write".to_string()],
        AclPolicy::Allow,
    ));

// AI can only access workspace directory
let data = read_file_with_middleware(
    "/workspace/code.rs",
    "ai-assistant",
    security
).await?; // ✅ Allowed and logged

let sensitive = read_file_with_middleware(
    "/etc/passwd",
    "ai-assistant",
    security
).await?; // ❌ Denied by ACL
```

### Secure Application Development

Build applications requiring system resources with security guarantees:

```rust
use airssys_osl::helpers::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Filesystem operations
    write_file("/tmp/config.json", data, "admin").await?;
    create_directory("/tmp/logs", "admin").await?;
    
    // Process operations  
    let output = spawn_process("docker", vec!["ps".to_string()], "admin").await?;
    
    // Network operations
    let listener = network_listen("127.0.0.1:8080", "admin").await?;
    
    Ok(())
}
```

### Enterprise System Administration

Automate system management with comprehensive audit trails:

```rust
use airssys_osl::prelude::*;
use airssys_osl::middleware::logger::*;

// Configure file-based audit logging
let logger = FileActivityLogger::new("/var/log/system-audit.log").await?;
let middleware = LoggerMiddleware::with_default_config(logger);

// All operations logged to audit file
let executor = FilesystemExecutor::default()
    .with_middleware(middleware);

// Operations executed with full audit trail
executor.execute(operation, context).await?;
// Logged: {"timestamp":"...","principal":"admin","operation":"FileRead",...}
```

### AirsStack Ecosystem Integration

Foundation layer for other AirsSys components:

- **airssys-rt**: Secure process operations for actor lifecycle management
- **airssys-wasm**: Sandboxed file/network access for WASM components via WASI
- **Ecosystem Monitoring**: Unified logging across all components

## Quick Start

### Installation

```toml
[dependencies]
airssys-osl = { version = "0.1", features = ["macros"] }
tokio = { version = "1.0", features = ["full"] }
```

### Hello World

```rust
use airssys_osl::helpers::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Write file with built-in security and logging
    let data = b"Hello, OSL!".to_vec();
    write_file("/tmp/hello.txt", data, "admin").await?;
    
    // Read file back
    let content = read_file("/tmp/hello.txt", "admin").await?;
    println!("{}", String::from_utf8_lossy(&content));
    
    Ok(())
}
```

### Available Helper Functions

**Filesystem (4 functions)**:

- `read_file()` - Read file contents
- `write_file()` - Write data to file
- `create_directory()` - Create directory
- `delete_file()` - Delete file

**Process (3 functions)**:

- `spawn_process()` - Execute command
- `kill_process()` - Terminate process
- `send_signal()` - Send signal (Unix only)

**Network (3 functions)**:

- `network_connect()` - TCP connect
- `network_listen()` - TCP listen
- `create_socket()` - UDP socket

Each function has a `*_with_middleware` variant for custom security policies.

## Architecture

```
┌─────────────────────────────────────────────────┐
│  Application (Your Code)                        │
│  ┌───────────────────────────────────────────┐  │
│  │  Helper Functions API (Level 1)           │  │
│  │  read_file(), write_file(), etc.          │  │
│  └───────────────────────────────────────────┘  │
└─────────────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────┐
│  OSL Framework (Middleware Pipeline)            │
│  ┌────────────┐  ┌──────────┐  ┌─────────────┐ │
│  │ Logger     │→ │ Security │→ │ Rate Limit  │ │
│  │ Middleware │  │ ACL/RBAC │  │ (custom)    │ │
│  └────────────┘  └──────────┘  └─────────────┘ │
└─────────────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────┐
│  OSL Executors (OS Abstraction)                 │
│  ┌──────────┐  ┌─────────┐  ┌───────────────┐  │
│  │Filesystem│  │ Process │  │    Network    │  │
│  │ Executor │  │Executor │  │   Executor    │  │
│  └──────────┘  └─────────┘  └───────────────┘  │
└─────────────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────┐
│  Operating System (Linux, macOS, Windows)       │
└─────────────────────────────────────────────────┘
```

## Documentation

### Guides

- [Middleware Guide](guides/middleware.md) - Learn about middleware pipeline
- [Custom Executors Guide](guides/custom-executors.md) - Build custom executors

### API Reference

- [Core Types](api/core-types.md) - Fundamental types and traits
- [Security Framework](api/security.md) - ACL, RBAC, and audit logging
- [Macros](api/macros.md) - Procedural macros for executors

### Architecture

- [Architecture Overview](architecture/README.md) - System design and patterns

## Current Status

**Version**: 0.1.0  
**Status**: ✅ Production Ready

### What's Complete

- ✅ **10 Helper Functions**: Filesystem, process, and network operations
- ✅ **Security Framework**: ACL and RBAC with audit logging
- ✅ **Logger Middleware**: Console, File, and Tracing loggers
- ✅ **Custom Executors**: Macro-based executor development
- ✅ **Comprehensive Testing**: 419 tests passing
- ✅ **Cross-Platform**: Linux, macOS, Windows support

### What's Next

- 🔄 **OSLFramework API**: High-level builder API with orchestration
- 🔄 **Additional Middleware**: Caching, rate limiting, retry logic
- 🔄 **Advanced Patterns**: Circuit breakers, bulkheads, timeouts

## Examples

See the [`examples/` directory](https://github.com/airsstack/airssys/tree/main/airssys-osl/examples):

- `helper_functions_comprehensive.rs` - All 10 helper functions with real workflows ⭐
- `middleware_pipeline.rs` - Middleware chaining and composition
- `security_middleware_comprehensive.rs` - Security policies and enforcement
- `custom_executor_with_macro.rs` - Creating custom executors

Run examples:
```bash
cargo run --example helper_functions_comprehensive
```

## Resources

- **Repository**: [github.com/airsstack/airssys](https://github.com/airsstack/airssys)
- **Crate**: [crates.io/crates/airssys-osl](https://crates.io/crates/airssys-osl)
- **API Docs**: Run `cargo doc --open` in `airssys-osl/`
- **Issues**: [Report bugs](https://github.com/airsstack/airssys/issues)

## License

Dual-licensed under Apache License 2.0 or MIT License.

---

**Next Steps**: Explore the [Middleware Guide](guides/middleware.md) or jump into [Examples](https://github.com/airsstack/airssys/tree/main/airssys-osl/examples).
