# OSL-TASK-006: Core Builder Implementation

**Task ID**: OSL-TASK-006  
**Created**: 2025-09-29  
**Priority**: High  
**Estimated Duration**: 8-10 hours  
**Dependencies**: OSL-TASK-005 (API Ergonomics Foundation)  
**Status**: Planned  

## Objective

Implement the core OSLFramework builder functionality, including automatic middleware orchestration, executor management, and high-level operation execution. This task delivers the primary developer ergonomics improvements while maintaining full compatibility with explicit primitive APIs.

## Background

Building on the foundation from OSL-TASK-005, this task implements the actual builder pattern functionality that provides:
- **Automatic middleware orchestration**: Eliminates manual pipeline management
- **Unified execution interface**: Simple API for common operations
- **Best practices enforcement**: Framework ensures correct patterns
- **Error handling automation**: Consistent error propagation and handling

## Acceptance Criteria

### 1. OSLFramework Core Implementation
- [ ] Complete `OSLFramework` struct with functional methods
- [ ] Implement automatic middleware pipeline execution
- [ ] Add executor registry with type-based dispatch
- [ ] Create context management and security integration

### 2. Builder Pattern Completion
- [ ] Functional `OSLFrameworkBuilder` with configuration methods
- [ ] Middleware registration and priority sorting
- [ ] Executor registration and validation
- [ ] Configuration validation and error handling

### 3. Automatic Pipeline Orchestration
- [ ] Implement `MiddlewarePipeline` with full lifecycle management
- [ ] Handle middleware errors with appropriate `ErrorAction` processing
- [ ] Support middleware filtering and conditional execution
- [ ] Provide cleanup and resource management

### 4. High-Level Operation Interface
- [ ] Create operation builders for filesystem operations
- [ ] Implement unified `execute()` method
- [ ] Add context creation and management
- [ ] Support operation metadata and timing

## Technical Implementation Details

### OSLFramework Implementation
```rust
// src/framework/mod.rs
impl OSLFramework {
    /// Create a new framework builder
    pub fn builder() -> OSLFrameworkBuilder {
        OSLFrameworkBuilder::new()
    }
    
    /// Execute an operation through the framework pipeline
    pub async fn execute<O: Operation>(&self, operation: O) -> OSResult<ExecutionResult> {
        let exec_context = ExecutionContext::new(self.security_context.clone());
        
        // Execute through pipeline with automatic orchestration
        self.middleware_pipeline
            .execute(operation, exec_context, &self.executors)
            .await
    }
    
    /// Get filesystem operation builder
    pub fn filesystem(&self) -> FilesystemBuilder {
        FilesystemBuilder::new(self)
    }
    
    /// Get process operation builder  
    pub fn process(&self) -> ProcessBuilder {
        ProcessBuilder::new(self)
    }
}
```

### Builder Pattern Implementation
```rust
// src/framework/builder.rs
impl OSLFrameworkBuilder {
    /// Enable security logging with default configuration
    pub fn with_security_logging(mut self, enabled: bool) -> Self {
        if enabled {
            let logging_middleware = LoggingMiddleware::builder()
                .with_security_audit(true)
                .with_level(LogLevel::Info)
                .build();
            self.middlewares.push(Box::new(logging_middleware));
        }
        self
    }
    
    /// Add policy enforcement from configuration file
    pub fn with_policy_enforcement<P: AsRef<Path>>(mut self, policy_path: P) -> Self {
        let security_middleware = SecurityMiddleware::builder()
            .with_policy_file(policy_path)
            .build();
        self.middlewares.push(Box::new(security_middleware));
        self
    }
    
    /// Add custom middleware to the pipeline
    pub fn add_middleware<M: Middleware<dyn Operation> + 'static>(mut self, middleware: M) -> Self {
        self.middlewares.push(Box::new(middleware));
        self
    }
    
    /// Register a custom executor for specific operation types
    pub fn with_executor<E: OSExecutor<dyn Operation> + 'static>(
        mut self,
        operation_types: Vec<OperationType>,
        executor: E,
    ) -> Self {
        let boxed_executor = Box::new(executor);
        for op_type in operation_types {
            self.executors.insert(op_type, boxed_executor.clone());
        }
        self
    }
    
    /// Build the final framework instance
    pub async fn build(mut self) -> OSResult<OSLFramework> {
        // Validate configuration
        self.validate_configuration()?;
        
        // Set up default executors if none provided
        self.setup_default_executors()?;
        
        // Create and validate pipeline
        let mut pipeline = MiddlewarePipeline::new();
        for middleware in self.middlewares {
            pipeline.add_middleware(middleware).await?;
        }
        
        // Initialize all middleware
        pipeline.initialize_all().await?;
        
        // Create executor registry
        let executor_registry = ExecutorRegistry::new(self.executors)?;
        
        // Create security context
        let security_context = self.security_config
            .map(|config| config.create_context())
            .unwrap_or_else(|| SecurityContext::new("default-user".to_string()));
        
        Ok(OSLFramework {
            middleware_pipeline: pipeline,
            executors: executor_registry,
            security_context,
            config: self.build_config()?,
        })
    }
    
    fn validate_configuration(&self) -> OSResult<()> {
        // Validate that we have executors for supported operation types
        let supported_types: HashSet<OperationType> = self.middlewares
            .iter()
            .flat_map(|m| m.supported_operation_types())
            .collect();
        
        for op_type in supported_types {
            if !self.executors.contains_key(&op_type) {
                return Err(OSError::ConfigurationError {
                    reason: format!("No executor registered for operation type: {:?}", op_type),
                });
            }
        }
        
        Ok(())
    }
}
```

### Pipeline Orchestration Implementation  
```rust
// src/framework/pipeline.rs
impl MiddlewarePipeline {
    /// Execute operation through complete middleware pipeline
    pub async fn execute<O: Operation>(
        &self,
        mut operation: O,
        context: ExecutionContext,
        executors: &ExecutorRegistry,
    ) -> OSResult<ExecutionResult> {
        // Before execution phase
        for middleware in &self.middlewares {
            if !middleware.can_process(&operation, &context).await {
                continue;
            }
            
            match middleware.before_execution(operation, &context).await {
                Ok(Some(modified_op)) => {
                    operation = modified_op;
                }
                Ok(None) => {
                    // Middleware handled the operation completely
                    return Ok(ExecutionResult::success(
                        b"Operation handled by middleware".to_vec()
                    ));
                }
                Err(middleware_error) => {
                    let os_error = middleware_error.to_os_error(middleware.name());
                    let action = middleware.handle_error(os_error.clone(), &context).await;
                    
                    match action {
                        ErrorAction::Stop => return Err(os_error),
                        ErrorAction::Continue => continue,
                        ErrorAction::ReplaceError(new_error) => return Err(new_error),
                        ErrorAction::Suppress => continue,
                        ErrorAction::LogAndContinue => {
                            // Log error and continue (implementation depends on logging middleware)
                            continue;
                        }
                        ErrorAction::Retry { max_attempts, delay } => {
                            // Implement retry logic
                            for attempt in 1..=max_attempts {
                                tokio::time::sleep(delay).await;
                                match middleware.before_execution(operation.clone(), &context).await {
                                    Ok(Some(retry_op)) => {
                                        operation = retry_op;
                                        break;
                                    }
                                    Ok(None) => return Ok(ExecutionResult::success(b"Retry handled".to_vec())),
                                    Err(_) if attempt == max_attempts => return Err(os_error),
                                    Err(_) => continue,
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Execute with appropriate executor
        let executor = executors.get_executor(&operation.operation_type())
            .ok_or_else(|| OSError::UnsupportedOperation {
                operation_type: operation.operation_type().as_str().to_string(),
            })?;
        
        let result = executor.execute(operation, &context).await;
        
        // After execution phase (in reverse order)
        for middleware in self.middlewares.iter().rev() {
            if let Err(middleware_error) = middleware.after_execution(&context, &result).await {
                // Log but don't fail the operation for after_execution errors
                let _os_error = middleware_error.to_os_error(middleware.name());
                // TODO: Log the error through logging middleware
            }
        }
        
        result
    }
}
```

### Operation Builders
```rust
// src/framework/operations.rs
/// Builder for filesystem operations
pub struct FilesystemBuilder<'a> {
    framework: &'a OSLFramework,
    operation_id: Option<String>,
    user_context: Option<String>,
    metadata: HashMap<String, String>,
    timeout: Option<Duration>,
}

impl<'a> FilesystemBuilder<'a> {
    pub fn new(framework: &'a OSLFramework) -> Self {
        Self {
            framework,
            operation_id: None,
            user_context: None,
            metadata: HashMap::new(),
            timeout: None,
        }
    }
    
    /// Read file operation
    pub fn read_file<P: AsRef<Path>>(self, path: P) -> FilesystemOperation {
        FilesystemOperation::new_read(
            self.operation_id.unwrap_or_else(|| format!("read-{}", Utc::now().timestamp())),
            path.as_ref().to_path_buf(),
        )
        .with_metadata_map(self.metadata)
    }
    
    /// Write file operation
    pub fn write_file<P: AsRef<Path>>(self, path: P, content: Vec<u8>) -> FilesystemOperation {
        FilesystemOperation::new_write(
            self.operation_id.unwrap_or_else(|| format!("write-{}", Utc::now().timestamp())),
            path.as_ref().to_path_buf(),
            content,
        )
        .with_metadata_map(self.metadata)
    }
    
    /// Set operation ID
    pub fn with_id(mut self, id: String) -> Self {
        self.operation_id = Some(id);
        self
    }
    
    /// Set user context
    pub fn with_user(mut self, user: String) -> Self {
        self.user_context = Some(user);
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

/// Extension trait for direct execution
impl FilesystemOperation {
    /// Execute this operation through the framework
    pub async fn execute_with(self, framework: &OSLFramework) -> OSResult<ExecutionResult> {
        framework.execute(self).await
    }
}
```

## File Structure Changes

```
airssys-osl/
├── src/
│   ├── framework/
│   │   ├── mod.rs              # Updated: complete implementation
│   │   ├── builder.rs          # New: OSLFrameworkBuilder
│   │   ├── pipeline.rs         # Updated: full orchestration
│   │   ├── operations.rs       # New: operation builders
│   │   ├── registry.rs         # New: executor registry
│   │   └── config.rs          # Updated: configuration types
│   ├── operations/             # New: concrete operations
│   │   ├── mod.rs             # New: operation implementations
│   │   ├── filesystem.rs      # New: filesystem operations
│   │   └── process.rs         # New: process operations
│   └── prelude.rs             # Updated: include framework types
```

## Testing Strategy

### Unit Tests
- [ ] OSLFramework builder pattern functionality
- [ ] Middleware pipeline orchestration with various scenarios
- [ ] Error handling for all ErrorAction variants
- [ ] Operation builders create correct operations
- [ ] Executor registry type dispatch

### Integration Tests
- [ ] End-to-end operation execution through framework
- [ ] Multiple middleware interactions and ordering
- [ ] Error propagation through complete pipeline
- [ ] Timeout handling and retry mechanisms
- [ ] Resource cleanup and shutdown procedures

### Performance Tests
- [ ] Framework overhead vs direct primitive usage
- [ ] Pipeline execution performance with multiple middleware
- [ ] Memory usage and allocation patterns
- [ ] Concurrent operation execution

### Example Tests
- [ ] Documentation examples compile and execute correctly
- [ ] Common usage patterns work as expected
- [ ] Migration from explicit to builder API

## Implementation Plan

### Phase 1: Core Framework (3-4 hours)
1. Complete OSLFramework struct implementation
2. Implement basic builder pattern with configuration
3. Create executor registry and type dispatch
4. Add context management integration

### Phase 2: Pipeline Orchestration (3-4 hours)
1. Complete MiddlewarePipeline with full lifecycle
2. Implement all ErrorAction handling scenarios
3. Add middleware filtering and conditional execution
4. Create retry logic and error recovery

### Phase 3: Operation Builders (2-3 hours)
1. Implement FilesystemBuilder with common operations
2. Create operation execution integration
3. Add builder configuration options (timeout, metadata, etc.)
4. Design extension patterns for custom operations

### Phase 4: Testing and Polish (2 hours)
1. Comprehensive test suite for all components
2. Performance benchmarking and optimization
3. Error message improvements and consistency
4. Documentation updates and examples

## Dependencies and Integration

### Required Components
- **Middleware implementations**: Need actual middleware for testing
- **Executor implementations**: Need concrete executors for operations
- **Operation types**: Need FilesystemOperation and ProcessOperation

### Integration Points
- **Core primitives**: Must maintain compatibility
- **Error handling**: Consistent with existing OSError types
- **Security context**: Integration with existing security model
- **Testing framework**: Work with existing test patterns

## Success Metrics

### Functionality
- [ ] Framework builder creates functional instances
- [ ] Operations execute correctly through pipeline
- [ ] All middleware lifecycle methods called appropriately
- [ ] Error handling works for all scenarios

### Developer Experience
- [ ] Simple operations take <10 lines of code
- [ ] Common patterns have builder support
- [ ] Error messages are clear and actionable
- [ ] IDE autocomplete works effectively

### Performance
- [ ] <10% overhead vs direct primitive usage
- [ ] Pipeline scales linearly with middleware count
- [ ] No memory leaks in long-running scenarios
- [ ] Concurrent operations supported efficiently

## Follow-up Tasks

This task enables:
- **OSL-TASK-007**: Extended Operation Builders
- **OSL-TASK-008**: Advanced Framework Features
- **Concrete middleware implementations** (logging, security, etc.)
- **Production deployment patterns**

## Notes

- Prioritize compatibility with existing explicit API
- Use dynamic dispatch thoughtfully (performance vs flexibility)
- Maintain clear separation between framework and core
- Document all architectural decisions and trade-offs