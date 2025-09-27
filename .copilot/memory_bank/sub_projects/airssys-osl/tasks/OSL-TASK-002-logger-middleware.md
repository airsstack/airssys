# Task: Implement Logger Middleware Module

**Task ID:** OSL-TASK-002  
**Priority:** High  
**Status:** Pending  
**Created:** 2025-09-27  
**Estimated Effort:** 1-2 days  

## Task Overview
Implement the logger middleware as a standalone module in `middleware/logger/` providing comprehensive activity logging for all OS operations.

## Task Description
Create the complete logger middleware implementation with structured activity logging, configurable output formats, and integration with the middleware pipeline. This middleware will log all operations before/after execution and provide security audit trails.

## Dependencies
- **Blocked by:** OSL-TASK-001 (Core Module Foundation) - MUST BE COMPLETED FIRST
- **Blocks:** Integration testing and security middleware integration
- **Related:** Security middleware implementation

## Acceptance Criteria

### 1. Module Structure Created
- ✅ `src/middleware/logger/mod.rs` - Clean module exports (§4.3)
- ✅ `src/middleware/logger/activity.rs` - ActivityLog types and ActivityLogger trait
- ✅ `src/middleware/logger/formatter.rs` - Log formatting and output handling
- ✅ `src/middleware/logger/middleware.rs` - LoggerMiddleware implementation

### 2. Technical Standards Compliance
- ✅ All files follow §2.1 3-layer import organization
- ✅ All timestamps use chrono DateTime<Utc> (§3.2)
- ✅ Generic-based implementation, no dyn patterns (§6.2)
- ✅ YAGNI principles - simple, focused implementation (§6.1)
- ✅ Microsoft Rust Guidelines compliance (§6.3)

### 3. Logger Middleware Implementation
- ✅ `LoggerMiddleware<O: Operation>` implementing `Middleware<O>` trait
- ✅ Comprehensive activity logging for all operations
- ✅ Before/after execution logging with duration tracking
- ✅ Error logging with contextual information
- ✅ Configurable log levels and output destinations

### 4. Activity Logging Framework
- ✅ `ActivityLog` struct with structured fields following M-ERRORS-CANONICAL-STRUCTS
- ✅ `ActivityLogger` trait for pluggable log output (file, console, network)
- ✅ JSON serialization support for structured logging
- ✅ Security-relevant operation highlighting

### 5. Quality Gates
- ✅ Zero compiler warnings
- ✅ Comprehensive rustdoc with examples
- ✅ Unit tests with >90% coverage
- ✅ Integration tests with core middleware pipeline
- ✅ Performance tests for high-frequency logging

## Implementation Details

### Module Structure
```
src/middleware/logger/
├── mod.rs              # Logger module exports and public API
├── activity.rs         # ActivityLog types and ActivityLogger trait
├── formatter.rs        # Log formatting and output handling  
├── middleware.rs       # LoggerMiddleware implementation
└── config.rs          # Logger configuration types
```

### Key Types Implementation

#### ActivityLog Structure
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityLog {
    pub timestamp: DateTime<Utc>,       // Following §3.2
    pub operation_id: String,
    pub operation_type: OperationType,
    pub user_context: UserContext,
    pub result: OperationResult,
    pub duration_ms: u64,
    pub metadata: serde_json::Value,
    pub security_relevant: bool,
}

#[async_trait::async_trait]
pub trait ActivityLogger: Debug + Send + Sync + 'static {
    async fn log_activity(&self, log: ActivityLog) -> Result<(), LogError>;
    async fn flush(&self) -> Result<(), LogError>;
}
```

#### Logger Middleware Implementation  
```rust
#[derive(Debug)]
pub struct LoggerMiddleware {
    logger: Arc<dyn ActivityLogger>,    // Exception: dyn needed for pluggable output
    config: LoggerConfig,
}

#[async_trait::async_trait]
impl<O: Operation> Middleware<O> for LoggerMiddleware {
    async fn before_execute(&self, operation: &O, context: &mut ExecutionContext) -> MiddlewareResult<()>;
    async fn after_execute(&self, operation: &O, result: &ExecutionResult, context: &ExecutionContext) -> MiddlewareResult<()>;
    async fn on_error(&self, operation: &O, error: &OSError, context: &ExecutionContext) -> MiddlewareResult<ErrorAction>;
    fn priority(&self) -> u32 { 200 } // Logger runs after security (100)
}
```

### Configuration Support
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggerConfig {
    pub level: LogLevel,
    pub output: LogOutput,
    pub format: LogFormat,
    pub buffer_size: usize,
    pub flush_interval_ms: u64,
}

#[derive(Debug, Clone)]
pub enum LogOutput {
    Console,
    File(PathBuf),
    Network(NetworkConfig),
    Multiple(Vec<LogOutput>),
}
```

## Testing Requirements

### Unit Tests
- ActivityLog serialization/deserialization
- LoggerMiddleware trait implementation
- Error handling and recovery scenarios
- Configuration validation and defaults

### Integration Tests
- Integration with middleware pipeline  
- Multiple output destinations
- High-frequency logging performance
- Error propagation through pipeline

### Performance Tests
- Logging throughput benchmarks
- Memory usage under load
- Buffer management efficiency
- Network output performance

## Documentation Requirements
- Comprehensive rustdoc for all public types
- Usage examples for different output configurations
- Performance characteristics documentation
- Security audit trail usage patterns

## Success Metrics
- Zero performance impact on non-logging operations
- Successful logging of 1000+ operations/second
- Clean integration with middleware pipeline
- Comprehensive audit trail for security review

## Notes
- This middleware should be lightweight and efficient
- Security audit functionality is critical for compliance
- Must work with structured logging systems (JSON output)
- Consider async batching for high-frequency operations

## Cross-References
- Core Architecture: 001-core-architecture-foundations.md
- Workspace Standards: §2.1, §3.2, §4.3, §6.1, §6.2, §6.3  
- Microsoft Guidelines: M-SERVICES-CLONE, M-ERRORS-CANONICAL-STRUCTS
- Related Task: OSL-TASK-001 (Core Module Foundation)
- Related Task: OSL-TASK-003 (Security Middleware Implementation)