# ADR-027: Builder Pattern Architecture Implementation

**Status**: Accepted  
**Date**: 2025-10-03  
**Deciders**: Development Team  
**Category**: Architecture Patterns  

## Context

OSLFramework requires a builder pattern that provides:
1. **Ergonomic Configuration**: Simple fluent interface for common setups
2. **Middleware Management**: Easy registration and configuration of middleware
3. **Executor Registration**: Automatic executor selection and management
4. **Validation**: Configuration validation before framework construction

## Decision

Implement a **Multi-Level Builder Architecture** with automatic orchestration and progressive complexity support.

### Core Architecture Components

#### 1. OSLFramework - Main Framework Entry Point
```rust
pub struct OSLFramework {
    middleware_pipeline: MiddlewarePipeline,
    executors: ExecutorRegistry,
    security_context: SecurityContext,
    config: OSLConfig,
}

impl OSLFramework {
    pub fn builder() -> OSLFrameworkBuilder {
        OSLFrameworkBuilder::new()
    }
    
    // High-level operation builders
    pub fn filesystem(&self) -> FilesystemBuilder<'_> { ... }
    pub fn process(&self) -> ProcessBuilder<'_> { ... }
    pub fn network(&self) -> NetworkBuilder<'_> { ... }
    
    // Direct execution method
    pub async fn execute<O: Operation>(&self, operation: O) -> OSResult<ExecutionResult> { ... }
}
```

#### 2. OSLFrameworkBuilder - Configuration Builder
```rust
pub struct OSLFrameworkBuilder {
    middlewares: Vec<Box<dyn Middleware<dyn Operation>>>,
    executors: HashMap<OperationType, Box<dyn OSExecutor<dyn Operation>>>,
    security_config: SecurityConfig,
    config: OSLConfigBuilder,
}

impl OSLFrameworkBuilder {
    // Convenience methods for common setups
    pub fn with_default_security(mut self) -> Self { ... }
    pub fn with_security_logging(mut self, enabled: bool) -> Self { ... }
    pub fn with_policy_file<P: AsRef<Path>>(mut self, path: P) -> Self { ... }
    
    // Advanced configuration
    pub fn add_middleware<M: Middleware<dyn Operation> + 'static>(mut self, middleware: M) -> Self { ... }
    pub fn add_executor<E: OSExecutor<dyn Operation> + 'static>(mut self, op_type: OperationType, executor: E) -> Self { ... }
    
    // Construction
    pub async fn build(self) -> OSResult<OSLFramework> { ... }
}
```

#### 3. Operation Builders - Fluent Operation Construction
```rust
pub struct FilesystemBuilder<'a> {
    framework: &'a OSLFramework,
    operation: FilesystemOperation,
}

impl<'a> FilesystemBuilder<'a> {
    pub fn read_file<P: AsRef<Path>>(mut self, path: P) -> Self { ... }
    pub fn write_file<P: AsRef<Path>>(mut self, path: P, content: impl Into<Vec<u8>>) -> Self { ... }
    pub fn with_user(mut self, user: impl Into<String>) -> Self { ... }
    pub fn with_permissions(mut self, permissions: Permission) -> Self { ... }
    
    pub async fn execute(self) -> OSResult<ExecutionResult> { ... }
}
```

#### 4. MiddlewarePipeline - Automatic Orchestration
```rust
pub(crate) struct MiddlewarePipeline {
    middlewares: Vec<Box<dyn Middleware<dyn Operation>>>,
    sorted: bool,
}

impl MiddlewarePipeline {
    pub async fn execute<O: Operation>(
        &self,
        operation: O,
        context: ExecutionContext,
        executors: &ExecutorRegistry,
    ) -> OSResult<ExecutionResult> {
        // 1. Sort middleware by priority if needed
        // 2. Execute before_execute() for each middleware
        // 3. Handle middleware errors with ErrorAction
        // 4. Execute operation with appropriate executor
        // 5. Execute after_execute() in reverse order
        // 6. Handle cleanup and error propagation
    }
}
```

#### 5. ExecutorRegistry - Automatic Executor Selection
```rust
pub(crate) struct ExecutorRegistry {
    executors: HashMap<OperationType, Box<dyn OSExecutor<dyn Operation>>>,
    default_executors: HashMap<OperationType, fn() -> Box<dyn OSExecutor<dyn Operation>>>,
}

impl ExecutorRegistry {
    pub fn get_executor(&self, op_type: OperationType) -> OSResult<&dyn OSExecutor<dyn Operation>> {
        // 1. Check for user-registered executor
        // 2. Fall back to default executor for operation type
        // 3. Return error if no executor available
    }
}
```

## Builder Pattern Levels

### Level 1: Simple Builder (80% of use cases)
```rust
let osl = OSLFramework::builder()
    .with_default_security()
    .with_security_logging(true)
    .build().await?;
```

### Level 2: Custom Configuration (15% of use cases)
```rust
let osl = OSLFramework::builder()
    .with_policy_file("/etc/osl/custom-policy.toml")
    .add_middleware(CustomAuditMiddleware::new())
    .with_security_logging(true)
    .build().await?;
```

### Level 3: Advanced Composition (5% of use cases)
```rust
let osl = OSLFramework::builder()
    .add_middleware(LoggerMiddleware::builder()
        .with_level(LogLevel::Debug)
        .with_format(LogFormat::Json)
        .build())
    .add_middleware(SecurityMiddleware::builder()
        .with_rbac_policy(rbac_config)
        .build())
    .add_executor(OperationType::Filesystem, CustomFilesystemExecutor::new())
    .build().await?;
```

## Internal Architecture Patterns

### 1. Automatic Default Population
```rust
impl OSLFrameworkBuilder {
    fn new() -> Self {
        let mut builder = Self::default();
        
        // Auto-populate with sensible defaults
        builder.add_default_executors();
        builder.security_config = SecurityConfig::default();
        
        builder
    }
    
    fn add_default_executors(&mut self) {
        self.executors.insert(OperationType::Filesystem, Box::new(DefaultFilesystemExecutor::new()));
        self.executors.insert(OperationType::Process, Box::new(DefaultProcessExecutor::new()));
        self.executors.insert(OperationType::Network, Box::new(DefaultNetworkExecutor::new()));
    }
}
```

### 2. Configuration Validation
```rust
impl OSLFrameworkBuilder {
    pub async fn build(self) -> OSResult<OSLFramework> {
        // 1. Validate configuration consistency
        self.validate_security_config()?;
        self.validate_middleware_compatibility()?;
        self.validate_executor_coverage()?;
        
        // 2. Sort middleware by priority
        let mut middlewares = self.middlewares;
        middlewares.sort_by_key(|m| m.priority());
        
        // 3. Initialize components
        let pipeline = MiddlewarePipeline::new(middlewares);
        let executors = ExecutorRegistry::new(self.executors);
        
        // 4. Create framework instance
        Ok(OSLFramework {
            middleware_pipeline: pipeline,
            executors,
            security_context: self.security_config.build_context()?,
            config: self.config.build()?,
        })
    }
}
```

### 3. Error Handling Strategy
```rust
#[derive(Error, Debug)]
pub enum FrameworkBuildError {
    #[error("Missing executor for operation type: {op_type:?}")]
    MissingExecutor { op_type: OperationType },
    
    #[error("Middleware configuration conflict: {reason}")]
    MiddlewareConflict { reason: String },
    
    #[error("Security policy validation failed: {reason}")]
    SecurityValidation { reason: String },
    
    #[error("Configuration error: {source}")]
    Configuration { #[from] source: ConfigError },
}
```

## Implementation Requirements

### 1. Backward Compatibility
- All existing core APIs remain unchanged
- Framework layer is additive, not replacing
- Migration path from primitives to framework

### 2. Performance Characteristics
- Framework overhead < 10% compared to direct primitive usage
- Lazy initialization where possible
- Efficient middleware pipeline execution

### 3. Testing Strategy
- Unit tests for each builder component
- Integration tests for complete builder workflows
- Performance tests comparing framework vs primitive APIs
- Error path testing for all validation scenarios

## Consequences

### Positive
- ✅ Dramatically improved developer experience
- ✅ Automatic best practices enforcement
- ✅ Reduced boilerplate code (80% reduction expected)
- ✅ Clear progression from simple to advanced usage
- ✅ Comprehensive error handling and validation

### Negative
- ❌ Additional complexity in framework layer
- ❌ Some runtime overhead compared to direct primitives
- ❌ Larger binary size due to default executors
- ❌ More complex testing requirements

### Mitigation
- **Complexity**: Clear internal architecture and comprehensive documentation
- **Performance**: Profile and optimize hot paths, provide primitive fallback
- **Size**: Make default executors optional via feature flags if needed
- **Testing**: Automated testing across all builder patterns and error scenarios

## Implementation Timeline

### Phase 1: Foundation (OSL-TASK-005) - 4-6 hours
- Basic OSLFramework and OSLFrameworkBuilder structures
- Prelude module with framework exports
- Simple builder methods (with_default_security, etc.)

### Phase 2: Core Implementation (OSL-TASK-006) - 8-10 hours
- Complete MiddlewarePipeline automatic orchestration
- ExecutorRegistry with default executor population
- Operation builders (FilesystemBuilder, etc.)
- Comprehensive validation and error handling

### Phase 3: Advanced Features (Future)
- Advanced middleware configuration patterns
- Custom executor registration
- Performance optimization
- Plugin architecture support

## References
- ADR-025: Framework dyn Pattern Exception
- ADR-026: Framework as Primary API Strategy
- Knowledge Doc 004: API Ergonomics Architecture Analysis
- OSL-TASK-005: API Ergonomics Foundation
- OSL-TASK-006: Core Builder Implementation