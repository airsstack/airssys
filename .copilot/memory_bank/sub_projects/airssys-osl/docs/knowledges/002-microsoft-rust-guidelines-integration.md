# Microsoft Rust Guidelines Integration - Knowledge Document

## Overview
This document captures the integration of Microsoft Rust Guidelines into the AirsSys project, providing comprehensive standards for production-quality Rust development optimized for AI agent collaboration.

## Key Guidelines Integration

### M-DESIGN-FOR-AI: AI-Optimized Development (CRITICAL)
**Why this matters for AirsSys:** Our project is developed with AI assistance, making this guideline essential.

**Implementation Requirements:**
- **Create Idiomatic Rust API Patterns**: Follow standard Rust conventions that AI agents understand well
- **Provide Thorough Documentation**: Comprehensive rustdoc for all modules and public items
- **Provide Thorough Examples**: Directly usable examples in documentation and repository
- **Use Strong Types**: Avoid primitive obsession, use newtype patterns for domain concepts
- **Make APIs Testable**: Design APIs for unit testing with mocks, fakes, or cargo features
- **Ensure Test Coverage**: Good observable behavior coverage enables hands-off refactoring

### M-DI-HIERARCHY: Dependency Injection Hierarchy
**Preference order for abstractions:**
1. **Concrete Types** (when implementation is fixed)
2. **Generics with Constraints** (`impl Trait` or `<T: Trait>`)
3. **`dyn` Traits** (only when generics cause excessive nesting)

**AirsSys Application:**
```rust
// ✅ PREFERRED: Concrete type for fixed implementation
pub struct FileSystemManager { /* ... */ }

// ✅ ACCEPTABLE: Generic for flexibility
pub trait OSExecutor<O: Operation> {
    async fn execute(&self, operation: O) -> OSResult<ExecutionResult>;
}

// ⚠️ LAST RESORT: dyn only when generics cause problems
// (Used minimally in pipeline for type erasure)
trait MiddlewareDispatcher: Debug + Send + Sync {
    async fn dispatch_any(&self, operation: &dyn Operation) -> MiddlewareResult<()>;
}
```

### M-ERRORS-CANONICAL-STRUCTS: Structured Error Handling
**AirsSys Error Pattern:**
```rust
use std::backtrace::Backtrace;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OSError {
    #[error("Security policy violation: {reason}")]
    SecurityViolation { 
        reason: String,
        backtrace: Backtrace,
    },
    
    #[error("Filesystem operation failed: {operation} on {path}: {source}")]
    FilesystemError {
        operation: String,
        path: PathBuf,
        #[source]
        source: std::io::Error,
        backtrace: Backtrace,
    },
}

impl OSError {
    // Helper methods for error categorization
    pub fn is_security_violation(&self) -> bool {
        matches!(self, OSError::SecurityViolation { .. })
    }
    
    pub fn is_filesystem_error(&self) -> bool {
        matches!(self, OSError::FilesystemError { .. })
    }
}
```

### M-SERVICES-CLONE: Shared Service Pattern
**AirsSys Service Implementation:**
```rust
// Service with cheap Clone via Arc<Inner>
#[derive(Debug, Clone)]
pub struct SecurityService {
    inner: Arc<SecurityServiceInner>,
}

struct SecurityServiceInner {
    policies: Vec<SecurityPolicy>,
    audit_logger: AuditLogger,
}

impl SecurityService {
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            inner: Arc::new(SecurityServiceInner::new(config))
        }
    }
    
    // Methods forward to inner implementation
    pub async fn validate_operation(&self, operation: &dyn Operation) -> PolicyDecision {
        self.inner.validate_operation(operation).await
    }
}
```

### M-AVOID-WRAPPERS: Clean API Design
**AirsSys API Patterns:**
```rust
// ✅ GOOD: Clean, simple API
pub fn execute_operation(operation: &Operation, context: &ExecutionContext) -> OSResult<()>;
pub fn configure_security(config: SecurityConfig) -> SecurityService;

// ❌ AVOID: Exposing implementation details
pub fn execute_operation(operation: Arc<Mutex<Operation>>) -> Box<dyn Future<Output = Result<(), Box<dyn Error>>>>;
```

### M-MOCKABLE-SYSCALLS: Testable I/O Operations
**AirsSys Testing Pattern:**
```rust
// Core trait for OS operations
pub trait FileSystemExecutor: Debug + Send + Sync {
    async fn create_file(&self, path: &Path) -> Result<FileHandle, FSError>;
    async fn read_file(&self, path: &Path) -> Result<Vec<u8>, FSError>;
}

// Production implementation
pub struct NativeFileSystemExecutor;

impl FileSystemExecutor for NativeFileSystemExecutor {
    async fn create_file(&self, path: &Path) -> Result<FileHandle, FSError> {
        // Real filesystem operations
        tokio::fs::File::create(path).await.map_err(FSError::from)
    }
}

// Test implementation
#[cfg(feature = "test-util")]
pub struct MockFileSystemExecutor {
    mock_files: Arc<Mutex<HashMap<PathBuf, Vec<u8>>>>,
}

#[cfg(feature = "test-util")]
impl FileSystemExecutor for MockFileSystemExecutor {
    async fn create_file(&self, path: &Path) -> Result<FileHandle, FSError> {
        // Mock implementation for testing
        self.mock_files.lock().unwrap().insert(path.to_path_buf(), Vec::new());
        Ok(MockFileHandle::new(path))
    }
}
```

### M-ESSENTIAL-FN-INHERENT: Core Functionality Pattern
**AirsSys Implementation:**
```rust
pub struct SecurityMiddleware {
    policies: Vec<SecurityPolicy>,
}

impl SecurityMiddleware {
    // ✅ Core functionality as inherent methods
    pub async fn validate_operation(&self, operation: &dyn Operation) -> PolicyDecision {
        // Core validation logic
    }
    
    pub async fn audit_security_event(&self, event: SecurityEvent) {
        // Core audit logging
    }
}

// Trait implementations forward to inherent methods
impl<O: Operation> Middleware<O> for SecurityMiddleware {
    async fn before_execute(&self, operation: &O, context: &mut ExecutionContext) -> MiddlewareResult<()> {
        // Forward to inherent implementation
        match self.validate_operation(operation).await {
            PolicyDecision::Allow => Ok(()),
            PolicyDecision::Deny(reason) => Err(MiddlewareError::SecurityViolation(reason)),
        }
    }
}
```

## Quality Gates and Compliance

### Static Verification Integration
Following M-STATIC-VERIFICATION, all AirsSys code must pass:
- **Compiler lints**: Comprehensive lint configuration in Cargo.toml
- **Clippy lints**: All major categories enabled with restriction group additions
- **Rustfmt**: Consistent source formatting
- **Cargo-audit**: Security vulnerability scanning
- **Miri**: Unsafe code validation (where applicable)

### Documentation Standards
Following M-DESIGN-FOR-AI documentation requirements:
- **M-FIRST-DOC-SENTENCE**: First sentence ≤15 words, one line
- **M-MODULE-DOCS**: All public modules have comprehensive documentation
- **M-CANONICAL-DOCS**: Standard doc sections (Examples, Errors, Panics, Safety)

### Testing Standards
Following M-DESIGN-FOR-AI testing requirements:
- **Observable behavior coverage**: >90% coverage of user-visible functionality
- **Mockable dependencies**: All I/O and system calls mockable
- **Test utility feature gating**: Testing functionality behind `test-util` feature
- **Property-based testing**: Use `proptest` for complex algorithms

## Implementation Checklist

### For All AirsSys Components
- [ ] Follow M-DI-HIERARCHY: prefer types > generics > dyn
- [ ] Implement M-ERRORS-CANONICAL-STRUCTS with Backtrace
- [ ] Apply M-SERVICES-CLONE pattern for shared services
- [ ] Ensure M-ESSENTIAL-FN-INHERENT for core functionality
- [ ] Design M-MOCKABLE-SYSCALLS for all I/O operations
- [ ] Avoid M-AVOID-WRAPPERS in public APIs
- [ ] Apply M-DESIGN-FOR-AI principles throughout

### For Core Module (OSL-TASK-001)
- [ ] Generic-based trait definitions (no dyn except where necessary)
- [ ] Comprehensive error types with helper methods
- [ ] Strong type definitions avoiding primitive obsession
- [ ] Complete rustdoc with examples for all public items
- [ ] Test framework preparation for mockable operations

### For Middleware Components
- [ ] Service clone pattern implementation
- [ ] Inherent method core functionality
- [ ] Mockable I/O operations with test-util feature
- [ ] Comprehensive error handling with categorization
- [ ] AI-friendly API design with clear examples

## Migration from Previous Standards

### What Changed
- **Enhanced AI optimization**: Added M-DESIGN-FOR-AI as critical guideline
- **Comprehensive error handling**: Detailed M-ERRORS-CANONICAL-STRUCTS implementation
- **Expanded testing requirements**: M-MOCKABLE-SYSCALLS and comprehensive mocking
- **Additional quality standards**: M-UNSAFE, M-UNSOUND, M-ESSENTIAL-FN-INHERENT

### Implementation Priority
1. **Immediate**: Apply to OSL-TASK-001 (Core Module Foundation)
2. **Phase 1**: Integrate into middleware implementations
3. **Ongoing**: Maintain compliance in all future development

## References
- **Primary**: [Microsoft Rust Guidelines](https://microsoft.github.io/rust-guidelines/)
- **Complete Text**: [AI Agents Reference](https://microsoft.github.io/rust-guidelines/agents/all.txt)
- **Upstream Standards**: [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/checklist.html)
- **Workspace Integration**: `shared_patterns.md` §6.3
- **Project Integration**: `AGENTS.md` technical standards section