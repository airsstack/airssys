# AirS Stack Rust Guidelines

## Overview
This document contains the **AirS Stack Rust Guidelines** for production-quality Rust development, optimized for AI agent collaboration. These standards are mandatory for all AirsSys sub-projects. It is based on the Microsoft Rust Guidelines but tailored for our specific needs.

## AI-Optimized Development (M-DESIGN-FOR-AI) - CRITICAL

**Why this matters for AirsSys:** Our project is developed with AI assistance, making this guideline essential.

**Implementation Requirements:**
- **Create Idiomatic Rust API Patterns**: Follow standard Rust conventions that AI agents understand well
- **Provide Thorough Documentation**: Comprehensive rustdoc for all modules and public items
- **Provide Thorough Examples**: Directly usable examples in documentation and repository
- **Use Strong Types**: Avoid primitive obsession, use newtype patterns for domain concepts
- **Make APIs Testable**: Design APIs for unit testing with mocks, fakes, or cargo features
- **Ensure Test Coverage**: Good observable behavior coverage enables hands-off refactoring

## Dependency Injection Hierarchy (M-DI-HIERARCHY)

**Preference order for abstractions:**
1. **Concrete Types** (when implementation is fixed)
2. **Generics with Constraints** (`impl Trait` or `<T: Trait>`)
3. **`dyn` Traits** (only when generics cause excessive nesting)

```rust
// ✅ PREFERRED: Concrete type for fixed implementation
pub struct FileSystemManager { /* ... */ }

// ✅ ACCEPTABLE: Generic for flexibility
pub trait OSExecutor<O: Operation> {
    async fn execute(&self, operation: O) -> OSResult<ExecutionResult>;
}

// ⚠️ LAST RESORT: dyn only when generics cause problems
trait MiddlewareDispatcher: Debug + Send + Sync {
    async fn dispatch_any(&self, operation: &dyn Operation) -> MiddlewareResult<()>;
}
```

## Structured Error Handling (M-ERRORS-CANONICAL-STRUCTS)

**Implementation Pattern:**
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
    pub fn is_security_violation(&self) -> bool {
        matches!(self, OSError::SecurityViolation { .. })
    }
}
```

## Shared Service Pattern (M-SERVICES-CLONE)

**Implementation:**
```rust
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
    
    pub async fn validate_operation(&self, operation: &dyn Operation) -> PolicyDecision {
        self.inner.validate_operation(operation).await
    }
}
```

## Clean API Design (M-AVOID-WRAPPERS)

**Patterns:**
```rust
// ✅ GOOD: Clean, simple API
pub fn execute_operation(operation: &Operation, context: &ExecutionContext) -> OSResult<()>;
pub fn configure_security(config: SecurityConfig) -> SecurityService;

// ❌ AVOID: Exposing implementation details
pub fn execute_operation(operation: Arc<Mutex<Operation>>) -> Box<dyn Future<Output = Result<(), Box<dyn Error>>>>;
```

## Testable I/O Operations (M-MOCKABLE-SYSCALLS)

**Pattern:**
```rust
pub trait FileSystemExecutor: Debug + Send + Sync {
    async fn create_file(&self, path: &Path) -> Result<FileHandle, FSError>;
    async fn read_file(&self, path: &Path) -> Result<Vec<u8>, FSError>;
}

// Production implementation
pub struct NativeFileSystemExecutor;

impl FileSystemExecutor for NativeFileSystemExecutor {
    async fn create_file(&self, path: &Path) -> Result<FileHandle, FSError> {
        tokio::fs::File::create(path).await.map_err(FSError::from)
    }
}

// Test implementation
#[cfg(feature = "test-util")]
pub struct MockFileSystemExecutor {
    mock_files: Arc<Mutex<HashMap<PathBuf, Vec<u8>>>>,
}
```

## Core Functionality Pattern (M-ESSENTIAL-FN-INHERENT)

**Pattern:**
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
        match self.validate_operation(operation).await {
            PolicyDecision::Allow => Ok(()),
            PolicyDecision::Deny(reason) => Err(MiddlewareError::SecurityViolation(reason)),
        }
    }
}
```

## Universal Standards

### Names are Free of Weasel Words (M-CONCISE-NAMES)
Avoid meaningless terms like `Service`, `Manager`, `Factory` in type names.

### Magic Values are Documented (M-DOCUMENTED-MAGIC)
All hardcoded values must have explanatory comments.

### Panic Means 'Stop the Program' (M-PANIC-IS-STOP)
Panics are for unrecoverable programming errors, not error communication.

### Public Types are Debug (M-PUBLIC-DEBUG)
All public types must implement `Debug`. Sensitive data should use custom implementations.

### Essential Functionality Should be Inherent (M-ESSENTIAL-FN-INHERENT)
Core functionality in inherent methods, trait implementations forward to them.

## Safety Standards

### Unsafe Implies Undefined Behavior (M-UNSAFE-IMPLIES-UB)
Only use `unsafe` if misuse risks undefined behavior.

### Unsafe Needs Reason, Should be Avoided (M-UNSAFE)
Valid reasons: novel abstractions, performance (after benchmarking), FFI.

### All Code Must be Sound (M-UNSOUND)
No exceptions - unsound code is never acceptable.

## Library Standards

### Features are Additive (M-FEATURES-ADDITIVE)
All feature combinations must work. Use `std` feature instead of `no-std`.

### Libraries Work Out of the Box (M-OOBE)
Must compile on Tier 1 platforms without additional prerequisites.

### Don't Leak External Types (M-DONT-LEAK-TYPES)
Prefer `std` types in public APIs over external crate types.

### Types are Send (M-TYPES-SEND)
Public types should be `Send` for runtime compatibility.

### I/O and System Calls Are Mockable (M-MOCKABLE-SYSCALLS)
All I/O operations must be mockable for testing.

## UX Standards

### Avoid Smart Pointers in APIs (M-AVOID-WRAPPERS)
Use simple types like `&T`, `&mut T`, `T` instead of `Arc<T>`, `Box<T>` in public APIs.

### Prefer Types over Generics, Generics over Dyn Traits (M-DI-HIERARCHY)
Follow the hierarchy: concrete types > generics > dyn traits.

### Error are Canonical Structs (M-ERRORS-CANONICAL-STRUCTS)
Use structured error types with `Backtrace` and helper methods.

### Accept `impl AsRef<>` Where Feasible (M-IMPL-ASREF)
Use `impl AsRef<str>`, `impl AsRef<Path>` for flexible function parameters.

### Services are Clone (M-SERVICES-CLONE)
Heavyweight services implement shared-ownership `Clone` via `Arc<Inner>`.

### Abstractions Don't Visibly Nest (M-SIMPLE-ABSTRACTIONS)
Avoid exposing nested type parameters to users. Limit to 1 level deep.

## Quality Gates

### Static Verification (M-STATIC-VERIFICATION)
Use compiler lints, clippy, rustfmt, cargo-audit, miri for quality assurance.

### Documentation Standards
- **M-FIRST-DOC-SENTENCE**: First sentence ≤15 words, one line
- **M-MODULE-DOCS**: All public modules have comprehensive documentation
- **M-CANONICAL-DOCS**: Standard doc sections (Examples, Errors, Panics, Safety)

### Performance Standards
- **M-HOTPATH**: Identify and optimize hot paths early
- **M-THROUGHPUT**: Optimize for throughput, avoid empty cycles
- **M-YIELD-POINTS**: Long-running tasks should yield regularly

## Application-Specific Guidelines

### Applications may use Anyhow (M-APP-ERROR)
Applications can use `anyhow`/`eyre` instead of custom error types.

### Use Mimalloc for Apps (M-MIMALLOC-APPS)
Set `mimalloc` as global allocator for performance.

## Compliance Checklist

### For All AirsSys Components
- [ ] Follow M-DI-HIERARCHY: prefer types > generics > dyn
- [ ] Implement M-ERRORS-CANONICAL-STRUCTS with Backtrace
- [ ] Apply M-SERVICES-CLONE pattern for shared services
- [ ] Ensure M-ESSENTIAL-FN-INHERENT for core functionality
- [ ] Design M-MOCKABLE-SYSCALLS for all I/O operations
- [ ] Avoid M-AVOID-WRAPPERS in public APIs
- [ ] Apply M-DESIGN-FOR-AI principles throughout

### Quality Standards
- [ ] Zero compiler warnings
- [ ] Comprehensive test coverage (>90%)
- [ ] All public items documented with examples
- [ ] Static verification passing (clippy, rustfmt, audit)
- [ ] Performance benchmarks for hot paths

## References
- **Primary**: [Microsoft Rust Guidelines](https://microsoft.github.io/rust-guidelines/)
- **Complete Text**: [AI Agents Reference](https://microsoft.github.io/rust-guidelines/agents/all.txt)
- **Upstream**: [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/checklist.html)
- **AirsSys Integration**: This file serves as authoritative technical standards for all sub-projects

---

*This document contains the AirS Stack Rust Guidelines. All guidelines are mandatory unless explicitly noted otherwise.*
