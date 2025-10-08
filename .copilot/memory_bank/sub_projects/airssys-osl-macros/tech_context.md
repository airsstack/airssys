# airssys-osl-macros Technical Context

## Technical Architecture

### System Overview
**airssys-osl-macros** is a procedural macro crate that generates boilerplate trait implementations for airssys-osl core abstractions at compile time. It uses syn v2 for parsing Rust syntax and quote for code generation, producing zero-cost abstractions with full type safety.

### Core Components

#### 1. Macro Entry Points (lib.rs)
```rust
#[proc_macro_attribute]
pub fn executor(attr: TokenStream, item: TokenStream) -> TokenStream;

#[proc_macro_derive(Operation, attributes(operation))]
pub fn derive_operation(input: TokenStream) -> TokenStream;

// Future: #[middleware] macro
```

#### 2. Parsing Logic (executor.rs, operation.rs)
- **syn::ItemImpl** parsing for attribute macros
- **syn::DeriveInput** parsing for derive macros
- Method signature extraction and validation
- Attribute parsing for macro configuration

#### 3. Code Generation (codegen.rs)
- **quote!** macro for generating Rust code
- Trait implementation generation
- Error handling code injection
- Timing and metadata code generation

#### 4. Utilities (utils.rs)
- Operation name to type mapping
- Method signature validation
- Error message formatting
- Span handling for diagnostics

### Technical Patterns

#### Proc-Macro Architecture Pattern
```
User Code (impl block)
    ↓ [TokenStream]
syn Parser
    ↓ [syn::ItemImpl]
Validation & Analysis
    ↓ [ParsedData]
Code Generator
    ↓ [proc_macro2::TokenStream]
quote! macro
    ↓ [TokenStream]
Generated Code (trait impls)
```

#### Method Name Mapping Pattern
```rust
// User writes:
async fn file_read(...)

// Macro detects:
- Method name: "file_read"
- Maps to: FileReadOperation
- Full path: airssys_osl::operations::filesystem::FileReadOperation
- Generates: impl OSExecutor<FileReadOperation>
```

#### Token Reference Pattern
```rust
// Macros use full paths, not imports
quote! {
    #[async_trait::async_trait]
    impl airssys_osl::core::executor::OSExecutor<
        airssys_osl::operations::filesystem::FileReadOperation
    > for #executor_name {
        async fn execute(
            &self,
            operation: airssys_osl::operations::filesystem::FileReadOperation,
            context: &airssys_osl::core::context::ExecutionContext,
        ) -> airssys_osl::core::result::OSResult<
            airssys_osl::core::executor::ExecutionResult
        > {
            self.file_read(operation, context).await
        }
    }
}
```

## Implementation Details

### Operation Name Mapping Table

#### Filesystem Operations
| Method Name        | Operation Type              | Module Path                           |
|--------------------|----------------------------|---------------------------------------|
| `file_read`        | FileReadOperation          | airssys_osl::operations::filesystem   |
| `file_write`       | FileWriteOperation         | airssys_osl::operations::filesystem   |
| `file_delete`      | FileDeleteOperation        | airssys_osl::operations::filesystem   |
| `directory_create` | DirectoryCreateOperation   | airssys_osl::operations::filesystem   |

#### Process Operations
| Method Name      | Operation Type          | Module Path                        |
|------------------|------------------------|------------------------------------|
| `process_spawn`  | ProcessSpawnOperation  | airssys_osl::operations::process   |
| `process_kill`   | ProcessKillOperation   | airssys_osl::operations::process   |
| `process_query`  | ProcessQueryOperation  | airssys_osl::operations::process   |

#### Network Operations
| Method Name   | Operation Type         | Module Path                       |
|---------------|------------------------|-----------------------------------|
| `tcp_connect` | TcpConnectOperation    | airssys_osl::operations::network  |
| `tcp_listen`  | TcpListenOperation     | airssys_osl::operations::network  |
| `udp_bind`    | UdpBindOperation       | airssys_osl::operations::network  |

### Method Signature Requirements

#### Valid Executor Method Signature
```rust
async fn <operation_name>(
    &self,                                          // Required: &self receiver
    operation: <OperationType>,                     // Required: Exact operation type
    context: &ExecutionContext,                     // Required: Execution context
) -> OSResult<ExecutionResult>                      // Required: Return type
```

#### Validation Rules
1. **Method must be async**: `async fn`
2. **Receiver must be &self**: No `&mut self`, `self`, or `Self`
3. **Exactly 2 parameters**: operation and context
4. **Operation parameter**: Must match expected type for method name
5. **Context parameter**: Must be `&ExecutionContext`
6. **Return type**: Must be `OSResult<ExecutionResult>`

### Error Handling Strategy

#### Compile-Time Errors
```rust
// Invalid method name
async fn invalid_operation(...) -> OSResult<ExecutionResult> { }
// Error: Unknown operation name 'invalid_operation'
//        Supported: file_read, file_write, process_spawn, ...

// Invalid signature
async fn file_read(&mut self, ...) -> OSResult<ExecutionResult> { }
// Error: Executor methods must use &self receiver, found &mut self

// Wrong parameter count
async fn file_read(&self, operation: FileReadOperation) -> ... { }
// Error: Expected 2 parameters (operation, context), found 1
```

#### Error Message Quality
- **Clear error type**: What went wrong
- **Context**: Where it happened (method name, line number)
- **Expected vs Found**: What was expected, what was provided
- **Suggestions**: How to fix the issue
- **Examples**: Valid usage patterns

## Performance Characteristics

### Compile-Time Performance
- **Parsing overhead**: ~10-50ms per impl block (syn v2 parsing)
- **Generation overhead**: ~5-20ms per trait impl (quote expansion)
- **Total impact**: <2% additional compilation time for typical usage
- **Caching**: Incremental compilation caches macro expansions

### Runtime Performance
- **Zero cost**: Macros generate identical code to manual implementations
- **Static dispatch**: All generated code uses generic constraints, no dyn
- **Inlining**: Generated code is fully inlinable by compiler
- **No allocations**: Code generation produces stack-allocated code

### Generated Code Size
- **Per operation impl**: ~30-50 lines of generated code
- **Total overhead**: ~10KB per executor with 10 operations
- **Binary size impact**: Zero (code identical to manual implementation)

## Testing Strategy

### Unit Tests (tests/unit/)
- Parsing logic tests
- Method name mapping tests
- Code generation tests
- Error message tests

### Integration Tests (tests/integration.rs)
- Full macro expansion tests
- Generated trait implementation tests
- Multi-operation executor tests
- Error case validation

### UI Tests (tests/ui/)
- Compile-time error message validation (trybuild)
- Invalid usage pattern detection
- Error message quality verification
- Suggestion accuracy tests

### Example Pattern
```rust
// tests/ui/invalid_signature.rs
use airssys_osl_macros::executor;

struct MyExecutor;

#[executor]
impl MyExecutor {
    async fn file_read(&mut self, op: FileReadOperation) { }
    //                 ^^^^^^^^^ ERROR: expected &self
}
```

## Dependencies and Constraints

### Dependency Version Matrix
```toml
[dependencies]
syn = { version = "2.0", features = ["full", "visit", "visit-mut"] }
quote = "1.0"
proc-macro2 = "1.0"

[dev-dependencies]
airssys-osl = { path = "../airssys-osl" }
async-trait = "0.1"
tokio = { version = "1.47", features = ["full"] }
trybuild = "1.0"
```

### Feature Requirements
- **syn features**: "full" (complete syntax tree), "visit" (AST traversal)
- **No runtime dependencies**: Macro crate cannot have runtime deps
- **Dev dependencies only**: airssys-osl for testing generated code

### Compilation Target
- **Proc-macro crate**: Compiles for host platform (build machine)
- **Different from target**: Not compiled for target platform
- **Separate compilation unit**: Cannot share types with main crate at runtime

## Security Considerations

### Code Injection Prevention
- **Validated inputs**: All user-provided identifiers validated
- **No arbitrary code execution**: Macros generate only known patterns
- **Type safety**: Generated code is fully type-checked by compiler
- **No eval**: No runtime code generation or evaluation

### Error Information Disclosure
- **Span preservation**: Error messages show exact source location
- **Limited information**: No sensitive data in error messages
- **Source code safety**: User source code not logged or transmitted

## Integration Points

### With airssys-osl
```rust
// In airssys-osl/Cargo.toml
[features]
default = ["macros"]
macros = ["airssys-osl-macros"]

[dependencies]
airssys-osl-macros = { path = "../airssys-osl-macros", optional = true }

// In airssys-osl/src/lib.rs
#[cfg(feature = "macros")]
pub use airssys_osl_macros::executor;

// In airssys-osl/src/prelude.rs
#[cfg(feature = "macros")]
pub use crate::executor;
```

### User Usage Pattern
```rust
// User code
use airssys_osl::prelude::*;  // Imports macro when feature enabled

#[executor]
impl MyExecutor {
    async fn file_read(&self, op: FileReadOperation, ctx: &ExecutionContext) 
        -> OSResult<ExecutionResult> 
    {
        // Custom implementation
        todo!()
    }
}
```

## Future Enhancements

### Phase 2: #[operation] Derive Macro
- Automatic Operation trait implementation
- Permission inference from struct fields
- Operation ID generation strategies
- Custom operation types support

### Phase 3: Advanced Error Handling
- Better error recovery
- Partial implementation support
- Default method generation
- Validation warnings (non-fatal)

### Phase 4: Developer Tools
- Macro expansion debugging aids
- IDE integration improvements
- Documentation generation
- Code completion hints

## Documentation Requirements

### Rustdoc Coverage
- 100% public items documented
- Macro usage examples with expansion
- Error cases with solutions
- Migration guide from manual implementation

### Example Documentation Structure
```rust
/// Generates `OSExecutor<O>` trait implementations from method names.
///
/// # Usage
///
/// ```rust
/// #[executor]
/// impl MyExecutor {
///     async fn file_read(&self, op: FileReadOperation, ctx: &ExecutionContext)
///         -> OSResult<ExecutionResult>
///     {
///         // Implementation
///     }
/// }
/// ```
///
/// # Generated Code
///
/// ```rust
/// #[async_trait::async_trait]
/// impl OSExecutor<FileReadOperation> for MyExecutor {
///     async fn execute(&self, operation: FileReadOperation, 
///         context: &ExecutionContext) -> OSResult<ExecutionResult>
///     {
///         self.file_read(operation, context).await
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn executor(attr: TokenStream, item: TokenStream) -> TokenStream;
```

## Workspace Standards Compliance

### §2.1 3-Layer Import Organization
- Standard library (proc_macro, std)
- Third-party (syn, quote, proc-macro2)
- Internal (crate modules)

### §4.3 Module Architecture
- mod.rs: Only declarations and re-exports
- executor.rs, operation.rs: Implementation modules
- utils.rs: Shared utilities

### §5.1 Dependency Management
- Minimal dependencies (syn, quote, proc-macro2 only)
- Dev dependencies for testing only
- No runtime dependencies (proc-macro constraint)

### §6.1 YAGNI Principles
- Build only #[executor] macro first
- Other macros only when explicitly needed
- No speculative features

### §6.2 Avoid `dyn` Patterns
- Generated code uses static dispatch only
- Generic constraints, no trait objects
- Zero-cost abstractions

### §6.3 Microsoft Rust Guidelines
- M-DESIGN-FOR-AI: Clear, documented, idiomatic generated code
- M-ERRORS-CANONICAL-STRUCTS: Structured error messages with context
- M-DI-HIERARCHY: Generated code uses concrete types and generics
