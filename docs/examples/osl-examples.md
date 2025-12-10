# OSL Examples

Examples demonstrating the OS Layer Framework for secure system operations.

## Helper Functions Examples

### Basic File Operations

```rust
use airssys_osl::helpers::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Write file
    let data = b"Hello, AirsSys!".to_vec();
    write_file("/tmp/test.txt", data, "admin").await?;
    
    // Read file
    let content = read_file("/tmp/test.txt", "admin").await?;
    println!("Content: {}", String::from_utf8_lossy(&content));
    
    // Delete file
    delete_file("/tmp/test.txt", "admin").await?;
    
    Ok(())
}
```

### Process Management

```rust
use airssys_osl::helpers::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Spawn process
    let output = spawn_process(
        "echo",
        vec!["Hello from process!".to_string()],
        "admin"
    ).await?;
    
    println!("Output: {}", String::from_utf8_lossy(&output));
    
    Ok(())
}
```

### Network Operations

```rust
use airssys_osl::helpers::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Start TCP listener
    let listener = network_listen("127.0.0.1:0", "admin").await?;
    println!("Listening on: {:?}", listener.local_addr()?);
    
    // Connect to server
    let stream = network_connect("127.0.0.1:8080", "admin").await?;
    
    Ok(())
}
```

## Security Examples

### ACL Configuration

```rust
use airssys_osl::middleware::security::*;
use airssys_osl::helpers::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create ACL policy
    let acl = AccessControlList::new()
        .add_entry(AclEntry::new(
            "alice".to_string(),
            "/data/*".to_string(),
            vec!["read".to_string(), "write".to_string()],
            AclPolicy::Allow,
        ))
        .add_entry(AclEntry::new(
            "bob".to_string(),
            "/data/*".to_string(),
            vec!["read".to_string()],
            AclPolicy::Allow,
        ));
    
    // Build security middleware
    let security = SecurityMiddlewareBuilder::new()
        .add_policy(Box::new(acl))
        .build()?;
    
    // Use with helper function
    let content = read_file_with_middleware(
        "/data/file.txt",
        "alice",
        security
    ).await?;
    
    Ok(())
}
```

### RBAC Configuration

```rust
use airssys_osl::middleware::security::*;

// Define roles and permissions
let rbac = RoleBasedAccessControl::new()
    .add_role("admin", vec!["read", "write", "delete"])
    .add_role("user", vec!["read"])
    .add_role_hierarchy("admin", "user"); // admin inherits user permissions

// Use in operations
let security = SecurityMiddlewareBuilder::new()
    .add_policy(Box::new(rbac))
    .build()?;
```

## Middleware Examples

### Logging Configuration

```rust
use airssys_osl::middleware::logger::*;
use airssys_osl::middleware::ExecutorExt;
use airssys_osl::executors::filesystem::FilesystemExecutor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Console logger
    let console_logger = ConsoleActivityLogger::default();
    let console_middleware = LoggerMiddleware::with_default_config(console_logger);
    
    // File logger
    let file_logger = FileActivityLogger::new("/tmp/ops.log").await?;
    let file_middleware = LoggerMiddleware::with_default_config(file_logger);
    
    // Chain middleware
    let executor = FilesystemExecutor::default()
        .with_middleware(console_middleware)
        .with_middleware(file_middleware);
    
    Ok(())
}
```

### Custom Middleware

```rust
use airssys_osl::core::middleware::Middleware;
use airssys_osl::core::operation::Operation;
use airssys_osl::core::context::ExecutionContext;
use async_trait::async_trait;

struct RateLimitMiddleware {
    max_ops_per_sec: u32,
}

#[async_trait]
impl Middleware for RateLimitMiddleware {
    async fn process(
        &self,
        operation: &dyn Operation,
        context: &ExecutionContext,
    ) -> Result<(), OSError> {
        // Check rate limit
        // Implementation...
        Ok(())
    }
}
```

## Custom Executor Examples

### With Macros

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
        println!("Custom file read: {}", operation.path());
        // Implementation...
        Ok(ExecutionResult::success(vec![]))
    }
}
```

## Complete Examples

For complete, runnable examples, see the repository:

```bash
# Comprehensive helper functions demo
cargo run --example helper_functions_comprehensive

# Security middleware configuration
cargo run --example security_middleware_comprehensive

# Custom middleware (rate limiting)
cargo run --example custom_middleware

# Logger configuration
cargo run --example logger_comprehensive

# Custom executor with macros
cargo run --example custom_executor_with_macro --features macros
```

## Next Steps

- [OSL API Reference](../components/osl/api/index.md)
- [Security Best Practices](../components/osl/reference/security-practices.md)
- [RT Examples](rt-examples.md)
