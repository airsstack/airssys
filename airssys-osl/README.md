# airssys-osl (OS Abstraction Layer)

This component will handle all low-level OS system programming, enhanced with activity logs and robust security policies

This component will handle these important and common system activities:
- Filesystem management
- Process management
- Network management
- Utils management
    - Something like calling other programs such as: `docker`, or `gh (github cli)`

## Motivation

The reason why I think I need this component is inspired by `airs-mcpserver-fs` project. This MCP tool provides access for an AI model so they can access the OS local environment to manipulate some filesystem (i/o), such as reading or writing a file. This MCP tool actually already provides a good enough security validator that tries to prevent any harmful activities, including avoiding any access to binary files, through its custom configurations.

I'm thinking of continuing to create other MCP tools that may need direct access to OS environments, such as running or stopping Docker, but I think I need more robust security, and also for its low-level system programming management. Actually, I can just use direct OS `Command` or `std::fs` from `Rust`, but I'm thinking to provides more controllable environments, such as:

- Monitoring commands, processes or activities
- More robust security policies, like `ACL` or `RBAC`

Based on these needs, I'm thinking of providing a high-level `OS Abstraction Layer (OSL)` that try to abstracting all of possible solutions and also provides `OS Middleware Layer`:

- Activity logs
- Robust security framework

## Architecture

![airssys-osl-arch](./assets/image.png)

### Building Blocks

#### Airssys OSL API

Provides high-level API methods or functions used by the caller to access OS activities, such as creating a new file or executing some OS processes

#### Airssys OSL Middleware

The `Middleware` component is a layer in the middle of the process between high-level APIs and their low-level OS executor. Our `OSL Framework` will pass through all requested activities to all available middleware , if there is an error on some middleware's processes, it will stop the request and throw the error.

Provides default three middlewares:

- `Logger`
- `Security`

Before each of the requested actions/activities is executed, it must go through this layer to log the activity and check for the security allowances. If it passed security check successfully, it will be forwarding to the *executor*

#### Airssys OSL Executor

Once a request passes security validations and has already been registered as a new `Runtime Process`, it will be *forwarded* to its specific executor based on activity or action type. On this layer, it will execute directly to the OS low-level executor through specific Rust OS executors. 

The main purpose of the executor must be modular, meaning that we can customize executors in the future, such as:

- Filesystem Executor
- Process Executor
- Network Executor
- Utils Executor

And all of those executors must implement the same `Executor` trait, so the `OSL Framework` can call them in the same way.

## Quick Start

Add `airssys-osl` to your `Cargo.toml`:

```toml
[dependencies]
airssys-osl = { version = "0.1", features = ["macros"] }
tokio = { version = "1.0", features = ["full"] }
```

### Basic Usage with Helper Functions (Level 1 API)

The simplest way to use `airssys-osl` is through helper functions with default security:

```rust
use airssys_osl::helpers::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Filesystem operations
    let data = b"Hello, World!".to_vec();
    write_file("/tmp/test.txt", data, "admin").await?;
    let content = read_file("/tmp/test.txt", "admin").await?;
    println!("Read: {}", String::from_utf8_lossy(&content));
    
    // Process operations
    let output = spawn_process("echo", vec!["Hello!".to_string()], "admin").await?;
    println!("Output: {}", String::from_utf8_lossy(&output));
    
    // Network operations
    let listener = network_listen("127.0.0.1:0", "admin").await?;
    println!("Listening on: {:?}", listener.local_addr()?);
    
    Ok(())
}
```

**Security:** All helper functions enforce default ACL and RBAC policies with audit logging.

### Advanced Usage with Custom Middleware (Level 2 API)

For custom security policies or middleware, use the `*_with_middleware` variants:

```rust
use airssys_osl::helpers::*;
use airssys_osl::middleware::security::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create custom ACL policy
    let acl = AccessControlList::new()
        .add_entry(AclEntry::new(
            "alice".to_string(),
            "/data/*".to_string(),
            vec!["read".to_string(), "write".to_string()],
            AclPolicy::Allow,
        ));
    
    // Build security middleware
    let security = SecurityMiddlewareBuilder::new()
        .add_policy(Box::new(acl))
        .build()?;
    
    // Use with custom middleware
    let data = read_file_with_middleware("/data/file.txt", "alice", security).await?;
    println!("Read {} bytes", data.len());
    
    Ok(())
}
```

### Advanced Usage with Direct Executors

For more control, you can use executors directly:

```rust
use airssys_osl::prelude::*;

#[tokio::main]
async fn main() -> Result<(), OSError> {
    // Create operation and context
    let operation = FileReadOperation::new("/tmp/test.txt".into());
    let context = ExecutionContext::default();
    
    // Execute with executor
    let executor = FilesystemExecutor::default();
    let result = executor.execute(operation, context).await?;
    
    println!("Read {} bytes", result.output.len());
    Ok(())
}
```

### Using Middleware with Extension Trait

Add middleware capabilities to any executor using the extension trait:

```rust
use airssys_osl::prelude::*;
use airssys_osl::middleware::ExecutorExt;

#[tokio::main]
async fn main() -> Result<(), OSError> {
    // Create executor with logging middleware
    let logger = ConsoleActivityLogger::default();
    let middleware = LoggerMiddleware::with_default_config(logger);
    
    let executor = FilesystemExecutor::default()
        .with_middleware(middleware);
    
    // Execute operation - automatically logs activity
    let operation = FileReadOperation::new("/tmp/test.txt".to_string());
    let context = ExecutionContext::new(SecurityContext::new("user".to_string()));
    let result = executor.execute(operation, &context).await?;
    
    Ok(())
}
```

### Chaining Multiple Middleware

You can chain multiple middleware together to create a processing pipeline:

```rust
use airssys_osl::prelude::*;
use airssys_osl::middleware::logger::{ConsoleActivityLogger, FileActivityLogger, LoggerMiddleware};
use airssys_osl::middleware::ExecutorExt;

#[tokio::main]
async fn main() -> Result<(), OSError> {
    // Create multiple middleware instances
    let console_logger = ConsoleActivityLogger::default();
    let console_middleware = LoggerMiddleware::with_default_config(console_logger);
    
    let file_logger = FileActivityLogger::new("/tmp/ops.log").await?;
    let file_middleware = LoggerMiddleware::with_default_config(file_logger);
    
    // Chain middleware together
    let executor = FilesystemExecutor::default()
        .with_middleware(console_middleware)  // Logs to console
        .with_middleware(file_middleware);     // Also logs to file
    
    // Execute operation - both middleware will process it
    let operation = FileReadOperation::new("/tmp/test.txt".to_string());
    let context = ExecutionContext::new(SecurityContext::new("user".to_string()));
    let result = executor.execute(operation, &context).await?;
    
    Ok(())
}
```

### Creating Custom Executors with Macros

The `#[executor]` macro simplifies creating custom executors by generating boilerplate code:

```rust
use airssys_osl::prelude::*;

#[executor(operations = [Filesystem])]
struct MyCustomExecutor;

impl MyCustomExecutor {
    async fn execute_file_read(
        &self,
        operation: FileReadOperation,
        _context: ExecutionContext,
    ) -> Result<ExecutionResult, OSError> {
        println!("Reading file: {}", operation.path());
        
        // Custom implementation
        Ok(ExecutionResult::success(vec![]))
    }
    
    async fn execute_file_write(
        &self,
        operation: FileWriteOperation,
        _context: ExecutionContext,
    ) -> Result<ExecutionResult, OSError> {
        println!("Writing {} bytes to: {}", operation.content().len(), operation.path());
        
        // Custom implementation
        Ok(ExecutionResult::success(vec![]))
    }
}
```

For more details, see:
- [Helper Functions Guide](docs/src/guides/helper-functions.md)
- [Middleware Guide](docs/src/guides/middleware.md)
- [Custom Executor Guide](docs/src/guides/custom-executors.md)
- [Macros API Reference](docs/src/api/macros.md)
- [Examples](examples/)

## Helper Functions

AirsSys OSL provides **10 high-level helper functions** for common OS operations with built-in security and audit logging:

### Filesystem Operations (4 helpers)

```rust
// Read file contents
let data = read_file("/path/to/file.txt", "user").await?;

// Write data to file
write_file("/path/to/file.txt", data, "user").await?;

// Create directory
create_directory("/path/to/dir", "user").await?;

// Delete file
delete_file("/path/to/file.txt", "user").await?;
```

### Process Operations (3 helpers)

```rust
// Spawn a process
let output = spawn_process("echo", vec!["Hello!".to_string()], "user").await?;

// Kill a process
kill_process(1234, "user").await?;

// Send signal to process (Unix only)
#[cfg(unix)]
send_signal(1234, nix::sys::signal::Signal::SIGTERM, "user").await?;
```

### Network Operations (3 helpers)

```rust
// TCP connect
let stream = network_connect("127.0.0.1:8080", "user").await?;

// TCP listen
let listener = network_listen("127.0.0.1:8080", "user").await?;

// UDP socket
let socket = create_socket("127.0.0.1:8080", "user").await?;
```

### Advanced Middleware Variants

All helpers have `*_with_middleware` variants for custom security policies:

```rust
use airssys_osl::middleware::security::*;

// Custom rate limiting middleware
let rate_limiter = RateLimitMiddleware::new(100); // 100 ops/sec

let data = read_file_with_middleware(
    "/path/to/file.txt",
    "user",
    rate_limiter
).await?;
```

**Complete Example:** See [`examples/helper_functions_comprehensive.rs`](examples/helper_functions_comprehensive.rs) for a full demonstration of all 10 helpers with error handling and real-world workflows.

## Features

### Core Features
- **Helper Functions API** - 10 high-level functions for filesystem, process, and network operations with built-in security
- **Three API Levels** - Simple helpers, custom middleware, and trait composition (planned) for different use cases
- **Cross-platform OS abstraction** - Unified interface for filesystem, process, and network operations
- **Type-safe operations** - Strongly-typed operation definitions with compile-time guarantees
- **Async/await support** - Built on Tokio for efficient async operations
- **Middleware pipeline** - Extensible middleware for logging, security, rate limiting, caching, and custom logic
- **Security framework** - Built-in ACL and RBAC policies with deny-by-default security model
- **Comprehensive audit logging** - All operations logged with security context for compliance

### Security Features
- **ACL (Access Control List)** - Path-based access control with glob pattern matching
- **RBAC (Role-Based Access Control)** - Role hierarchies with permission inheritance
- **Deny-by-default** - Operations denied unless explicitly allowed by policy
- **Security middleware** - Automatic policy enforcement in all helper functions
- **Audit trails** - JSON-formatted security audit logs for all operations

### Optional Features
- `macros` - Procedural macros for simplified custom executor development

## Examples

The `examples/` directory contains comprehensive examples:

- **`helper_functions_comprehensive.rs`** - Complete demonstration of all 10 helper functions with real-world workflows ⭐
- `basic_usage.rs` - Basic operation execution
- `middleware_pipeline.rs` - Middleware chaining and composition
- `custom_middleware.rs` - Creating custom middleware (rate limiting example)
- `logger_comprehensive.rs` - Advanced logging configuration
- `security_middleware_comprehensive.rs` - Security policies and enforcement
- `custom_executor_with_macro.rs` - Creating custom executors with macros

Run examples with:

```bash
# Run comprehensive helper functions example
cargo run --example helper_functions_comprehensive

# Run custom middleware example
cargo run --example custom_middleware

# Run with features
cargo run --example custom_executor_with_macro --features macros
```

## Documentation

Full documentation is available in the `docs/` directory and can be built with mdBook:

```bash
# Install mdBook (one-time)
cargo install mdbook

# Serve documentation locally
mdbook serve airssys-osl/docs

# Build documentation
mdbook build airssys-osl/docs
```

## Development

### Running Tests

```bash
# Run all tests
cargo test --workspace

# Run OSL tests only
cargo test --package airssys-osl

# Run with features
cargo test --features macros
```

### Code Quality

```bash
# Check code
cargo check --workspace

# Run clippy
cargo clippy --workspace --all-targets --all-features

# Format code
cargo fmt --all
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE))
- MIT License ([LICENSE-MIT](../LICENSE-MIT))

at your option.