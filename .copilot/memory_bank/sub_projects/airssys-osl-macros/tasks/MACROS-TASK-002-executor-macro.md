# Task: Implement #[executor] Macro

**Task ID:** MACROS-TASK-002  
**Priority:** High  
**Status:** Ready to Start  
**Created:** 2025-10-08  
**Updated:** 2025-10-08 (Development plan complete)  
**Estimated Effort:** 10 days (2 work weeks)  

## Next Steps
After completion, proceed to **MACROS-TASK-003**: Integration with airssys-osl.

## Development Timeline (Updated 2025-10-08)

### Phase 1: Core Parsing & Validation (Days 1-3)
**Goal:** Parse impl blocks and validate method signatures

**Tasks:**
- [ ] Day 1: Implement basic parser in executor.rs
  - Parse ItemImpl with syn::parse2
  - Extract executor type name
  - Find operation methods
  - Return original impl + placeholder trait impls
  
- [ ] Day 2: Implement signature validation
  - Validate async keyword
  - Validate &self receiver (not &mut self, not self)
  - Validate 2 parameters (operation, &ExecutionContext)
  - Validate return type OSResult<ExecutionResult>
  - Add helpful error messages
  
- [ ] Day 3: Unit tests for parsing
  - Test valid impl blocks
  - Test rejection of non-async methods
  - Test rejection of &mut self
  - Test rejection of invalid parameters
  - Test rejection of invalid return types

### Phase 2: Operation Mapping & Code Generation (Days 4-7)
**Goal:** Map method names to operation types and generate trait implementations

**Tasks:**
- [ ] Day 4: Complete operation mapping in utils.rs
  - Create OperationInfo struct
  - Implement get_operation_info() for all 11 operations
  - Add operation_path() method for full qualified paths
  - Add is_operation_method() helper
  
- [ ] Day 5: Implement code generation
  - Generate #[async_trait::async_trait] attribute
  - Generate impl OSExecutor<OperationType> for ExecutorType
  - Generate execute() method delegation
  - Preserve original impl block
  
- [ ] Day 6: Support multiple operations
  - Generate multiple trait impls for multiple methods
  - Detect and error on duplicate methods
  - Test with all operation combinations
  
- [ ] Day 7: Unit tests for generation
  - Test operation mapping table (11 operations)
  - Test single operation code generation
  - Test multiple operations code generation
  - Test generated code compiles

### Phase 3: Testing & Documentation (Days 8-10)
**Goal:** Comprehensive testing and documentation

**Tasks:**
- [ ] Day 8: Integration tests
  - Create tests/integration.rs
  - Test macro with actual airssys-osl types
  - Test all 11 operations compile correctly
  - Test error cases
  
- [ ] Day 9: Documentation
  - Update lib.rs with comprehensive examples
  - Document macro expansion output
  - Create usage guide with before/after code
  - Document all error messages
  
- [ ] Day 10: Final validation
  - Run cargo check --workspace
  - Run cargo clippy --workspace
  - Run cargo test --workspace
  - Update progress.md
  - Git commit

## Quality Checklist

### Before Task Completion
- [ ] All 11 operations mapped correctly
- [ ] Parse impl blocks with syn correctly
- [ ] Validate method signatures with helpful errors
- [ ] Generate correct OSExecutor<O> trait implementations
- [ ] Support multiple operations in one impl block
- [ ] Use async_trait::async_trait correctly
- [ ] 20+ unit tests passing
- [ ] 10+ integration tests passing
- [ ] Comprehensive rustdoc with examples
- [ ] Zero compiler warnings
- [ ] Zero clippy warnings
- [ ] cargo check --workspace passes
- [ ] Memory bank updated
- [ ] Git commit with proper message

## Key Implementation Notes

### Critical Requirements
1. **Must use `#[async_trait::async_trait]`** - OSExecutor trait has async methods
2. **Preserve original impl block** - User's methods must stay unchanged in output
3. **Generate one trait impl per operation method** - Each method gets its own `OSExecutor<SpecificOp>` impl
4. **Type paths must be fully qualified** - Use complete paths like `airssys_osl::operations::filesystem::FileReadOperation`
5. **Error messages must be helpful** - Tell users exactly what's wrong and how to fix it

### Method Signature Requirements
```rust
// VALID signature:
async fn operation_name(
    &self,                              // Must be &self (not &mut self, not self)
    operation: OperationType,           // First param: operation type
    context: &ExecutionContext          // Second param: context reference
) -> OSResult<ExecutionResult>         // Return type must be OSResult<ExecutionResult>

// The macro will generate:
#[async_trait::async_trait]
impl OSExecutor<OperationType> for ExecutorType {
    async fn execute(
        &self,
        operation: OperationType,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        self.operation_name(operation, context).await
    }
}
```

### Operation Mapping Table (Complete)
| Method Name | Operation Type | Module Path |
|-------------|---------------|-------------|
| file_read | FileReadOperation | filesystem |
| file_write | FileWriteOperation | filesystem |
| file_delete | FileDeleteOperation | filesystem |
| directory_create | DirectoryCreateOperation | filesystem |
| directory_list | DirectoryListOperation | filesystem |
| process_spawn | ProcessSpawnOperation | process |
| process_kill | ProcessKillOperation | process |
| process_signal | ProcessSignalOperation | process |
| network_connect | NetworkConnectOperation | network |
| network_listen | NetworkListenOperation | network |
| network_socket | NetworkSocketOperation | network |

### Error Message Standards
- **Non-async method**: "Operation method 'method_name' must be async. Add 'async' keyword."
- **Wrong receiver**: "Use '&self', not '&mut self'. Executors should be immutable."
- **Wrong parameters**: "Operation methods must have signature: async fn name(&self, op: OpType, ctx: &ExecutionContext)"
- **Unknown method**: "Unknown operation method: 'method_name'. Valid operations: file_read, file_write, ..."

## Success Criteria Summary

✅ Task is complete when:
1. All 11 operation types mapped and tested
2. Macro generates valid OSExecutor<O> trait implementations
3. Multiple operations in one impl block supported
4. All validation errors have helpful messages
5. 30+ tests passing (20 unit + 10 integration)
6. Comprehensive documentation with examples
7. Zero warnings (compiler + clippy)
8. Integration with airssys-osl verified
9. Memory bank updated
10. Changes committed to git

## Task Description
Build the core macro logic using syn v2 for parsing and quote for code generation. The macro will detect operation-named methods in impl blocks and generate corresponding trait implementations with full type safety and error handling.

**Key Discovery from airssys-osl Exploration:**
- `OSExecutor<O>` is a generic trait with operation type parameter
- Uses `#[async_trait::async_trait]` for async methods
- Must generate: `impl OSExecutor<OperationType> for ExecutorType`
- 11 concrete operations found across filesystem, process, network modules
- Method signature: `async fn execute(&self, operation: O, context: &ExecutionContext) -> OSResult<ExecutionResult>`

## Dependencies
- **Blocked by:** MACROS-TASK-001 (Foundation Setup) ✅ COMPLETE
- **Blocks:** MACROS-TASK-003 (Integration with airssys-osl)
- **Related:** 
  - OSL-TASK-009 (airssys-osl refactoring)
  - airssys-osl core abstractions (explored and documented)
  
**Core Dependencies Identified:**
- `airssys_osl::core::executor::OSExecutor<O>` trait
- `airssys_osl::core::context::ExecutionContext` type
- `airssys_osl::core::executor::ExecutionResult` type
- `airssys_osl::core::result::{OSResult, OSError}` types
- `async_trait::async_trait` attribute macro
- 11 concrete operation types from airssys-osl/src/operations/

## Acceptance Criteria

### 1. Method Parsing Implementation
- ✅ Parse impl blocks with syn::ItemImpl
- ✅ Extract methods from impl block items
- ✅ Validate method signatures (async, &self, 2 params, correct return type)
- ✅ Detect operation methods by name pattern
- ✅ Comprehensive error messages for invalid signatures

### 2. Operation Name Mapping
- ✅ Complete mapping table for all 11 operations (verified from airssys-osl/src/operations/)
- ✅ Filesystem: file_read, file_write, file_delete, directory_create, directory_list
- ✅ Process: process_spawn, process_kill, process_signal
- ✅ Network: network_connect, network_listen, network_socket
- ✅ Case-sensitive matching
- ✅ Clear error for unknown method names

### 3. Code Generation
- ✅ Generate #[async_trait::async_trait] attribute
- ✅ Generate impl OSExecutor<OperationType> for ExecutorType
- ✅ Generate execute() method delegation to user method
- ✅ Preserve original impl block unchanged
- ✅ Generate multiple trait impls for multiple methods

### 4. Error Handling
- ✅ Clear error messages with span information
- ✅ Validation errors for method signatures
- ✅ Unknown operation name errors
- ✅ Duplicate method detection
- ✅ Helpful suggestions in error messages

### 5. Testing
- ✅ Unit tests for parsing logic (20+ tests)
- ✅ Unit tests for method name mapping (10+ tests)
- ✅ Integration tests with airssys-osl (15+ tests)
- ✅ UI tests for error messages (10+ error cases)
- ✅ 100% coverage of mapping table

### 6. Documentation
- ✅ Comprehensive rustdoc for #[executor] macro
- ✅ Usage examples with expansion output
- ✅ Error message reference
- ✅ Migration guide from manual implementation
- ✅ Best practices documentation

### 7. Quality Gates
- ✅ Zero compiler warnings
- ✅ Zero clippy warnings
- ✅ All tests passing
- ✅ Documentation complete
- ✅ Workspace standards compliance

## Implementation Plan

### ⚠️ CRITICAL: Implementation Based on airssys-osl Core Exploration

**Actual airssys-osl Architecture (Verified 2025-10-08):**

**OSExecutor<O> Trait Signature:**
```rust
#[async_trait]
pub trait OSExecutor<O>: Debug + Send + Sync + 'static
where O: Operation
{
    fn name(&self) -> &str;
    fn supported_operation_types(&self) -> Vec<OperationType>;
    
    async fn execute(
        &self, 
        operation: O, 
        context: &ExecutionContext
    ) -> OSResult<ExecutionResult>;
    
    // Default implementations available:
    async fn can_execute(&self, operation: &O, context: &ExecutionContext) -> OSResult<bool> { ... }
    async fn validate_operation(&self, operation: &O, context: &ExecutionContext) -> OSResult<()> { ... }
    async fn cleanup(&self, context: &ExecutionContext) -> OSResult<()> { ... }
}
```

**Verified Operation Types and Paths:**
- `airssys_osl::operations::filesystem::FileReadOperation`
- `airssys_osl::operations::filesystem::FileWriteOperation`
- `airssys_osl::operations::filesystem::FileDeleteOperation`
- `airssys_osl::operations::filesystem::DirectoryCreateOperation`
- `airssys_osl::operations::filesystem::DirectoryListOperation`
- `airssys_osl::operations::process::ProcessSpawnOperation`
- `airssys_osl::operations::process::ProcessKillOperation`
- `airssys_osl::operations::process::ProcessSignalOperation`
- `airssys_osl::operations::network::NetworkConnectOperation`
- `airssys_osl::operations::network::NetworkListenOperation`
- `airssys_osl::operations::network::NetworkSocketOperation`

**Key Types:**
- Context: `airssys_osl::core::context::ExecutionContext`
- Result: `airssys_osl::core::executor::ExecutionResult`
- Error: `airssys_osl::core::result::{OSResult, OSError}`

---

### Phase 1: Basic Parsing (Days 1-3)
**Goal:** Parse impl blocks and extract method information

#### Step 1.1: Implement Basic Parser
```rust
// In executor.rs
pub fn expand(input: TokenStream) -> Result<TokenStream> {
    let item_impl = syn::parse2::<syn::ItemImpl>(input)?;
    
    // Extract executor type name
    let executor_type = &item_impl.self_ty;
    
    // Find all methods
    let methods = extract_methods(&item_impl)?;
    
    // For now, just return original impl
    Ok(quote! { #item_impl })
}

fn extract_methods(impl_block: &syn::ItemImpl) -> Result<Vec<&syn::ImplItemFn>> {
    let mut methods = Vec::new();
    
    for item in &impl_block.items {
        if let syn::ImplItem::Fn(method) = item {
            methods.push(method);
        }
    }
    
    Ok(methods)
}
```

#### Step 1.2: Add Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    
    #[test]
    fn test_parse_impl_block() {
        let input = quote! {
            impl MyExecutor {
                async fn file_read(&self, op: FileReadOperation, ctx: &ExecutionContext) 
                    -> OSResult<ExecutionResult> 
                {
                    todo!()
                }
            }
        };
        
        let result = expand(input);
        assert!(result.is_ok());
    }
}
```

### Phase 2: Method Validation (Week 1)
**Goal:** Validate method signatures and provide clear errors

#### Step 2.1: Signature Validation
```rust
fn validate_executor_method(method: &syn::ImplItemFn) -> Result<()> {
    let sig = &method.sig;
    
    // Check async
    if sig.asyncness.is_none() {
        return Err(syn::Error::new_spanned(
            sig,
            "executor methods must be async functions"
        ));
    }
    
    // Check receiver (&self)
    validate_receiver(sig)?;
    
    // Check parameters
    validate_parameters(sig)?;
    
    // Check return type
    validate_return_type(sig)?;
    
    Ok(())
}

fn validate_receiver(sig: &syn::Signature) -> Result<()> {
    let receiver = sig.inputs.first()
        .and_then(|arg| match arg {
            syn::FnArg::Receiver(r) => Some(r),
            _ => None,
        })
        .ok_or_else(|| syn::Error::new_spanned(
            sig,
            "executor methods must have &self receiver"
        ))?;
    
    if receiver.mutability.is_some() {
        return Err(syn::Error::new_spanned(
            receiver,
            "executor methods must use &self, not &mut self"
        ));
    }
    
    if receiver.reference.is_none() {
        return Err(syn::Error::new_spanned(
            receiver,
            "executor methods must use &self, not self"
        ));
    }
    
    Ok(())
}
```

#### Step 2.2: Add Validation Tests
```rust
#[test]
fn test_reject_non_async_method() {
    let input = quote! {
        impl MyExecutor {
            fn file_read(&self, op: FileReadOperation, ctx: &ExecutionContext) 
                -> OSResult<ExecutionResult> { todo!() }
        }
    };
    
    let result = expand(input);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("must be async"));
}
```

### Phase 3: Operation Mapping (Week 2)
**Goal:** Complete operation name to type mapping

#### Step 3.1: Operation Mapping Table
```rust
// In utils.rs
#[derive(Debug, Clone)]
pub struct OperationType {
    pub method_name: &'static str,
    pub type_name: &'static str,
    pub module_path: &'static str,
}

impl OperationType {
    pub fn full_path(&self) -> syn::Path {
        syn::parse_str(&format!(
            "airssys_osl::operations::{}::{}",
            self.module_path, self.type_name
        )).unwrap()
    }
}

pub fn map_method_name_to_operation(name: &str) -> Option<OperationType> {
    match name {
        // Filesystem operations
        "file_read" => Some(OperationType {
            method_name: "file_read",
            type_name: "FileReadOperation",
            module_path: "filesystem",
        }),
        "file_write" => Some(OperationType {
            method_name: "file_write",
            type_name: "FileWriteOperation",
            module_path: "filesystem",
        }),
        "file_delete" => Some(OperationType {
            method_name: "file_delete",
            type_name: "FileDeleteOperation",
            module_path: "filesystem",
        }),
        "directory_create" => Some(OperationType {
            method_name: "directory_create",
            type_name: "DirectoryCreateOperation",
            module_path: "filesystem",
        }),
        
        // Process operations
        "process_spawn" => Some(OperationType {
            method_name: "process_spawn",
            type_name: "ProcessSpawnOperation",
            module_path: "process",
        }),
        "process_kill" => Some(OperationType {
            method_name: "process_kill",
            type_name: "ProcessKillOperation",
            module_path: "filesystem",
        }),
        "process_query" => Some(OperationType {
            method_name: "process_query",
            type_name: "ProcessQueryOperation",
            module_path: "process",
        }),
        
        // Network operations
        "tcp_connect" => Some(OperationType {
            method_name: "tcp_connect",
            type_name: "TcpConnectOperation",
            module_path: "network",
        }),
        "tcp_listen" => Some(OperationType {
            method_name: "tcp_listen",
            type_name: "TcpListenOperation",
            module_path: "network",
        }),
        "udp_bind" => Some(OperationType {
            method_name: "udp_bind",
            type_name: "UdpBindOperation",
            module_path: "network",
        }),
        
        _ => None,
    }
}
```

#### Step 3.2: Mapping Tests
```rust
#[test]
fn test_all_operation_mappings() {
    // Filesystem
    assert!(map_method_name_to_operation("file_read").is_some());
    assert!(map_method_name_to_operation("file_write").is_some());
    assert!(map_method_name_to_operation("file_delete").is_some());
    assert!(map_method_name_to_operation("directory_create").is_some());
    
    // Process
    assert!(map_method_name_to_operation("process_spawn").is_some());
    assert!(map_method_name_to_operation("process_kill").is_some());
    assert!(map_method_name_to_operation("process_query").is_some());
    
    // Network
    assert!(map_method_name_to_operation("tcp_connect").is_some());
    assert!(map_method_name_to_operation("tcp_listen").is_some());
    assert!(map_method_name_to_operation("udp_bind").is_some());
    
    // Invalid
    assert!(map_method_name_to_operation("invalid_operation").is_none());
}
```

### Phase 4: Code Generation (Week 2-3)
**Goal:** Generate trait implementations from methods

#### Step 4.1: Trait Implementation Generator
```rust
fn generate_trait_implementations(
    impl_block: &syn::ItemImpl,
    executor_methods: &[ExecutorMethod],
) -> Vec<TokenStream> {
    executor_methods
        .iter()
        .map(|method| generate_single_trait_impl(impl_block, method))
        .collect()
}

fn generate_single_trait_impl(
    impl_block: &syn::ItemImpl,
    method: &ExecutorMethod,
) -> TokenStream {
    let executor_type = &impl_block.self_ty;
    let operation_path = &method.operation_type.full_path();
    let method_name = &method.method_name;
    
    quote! {
        #[async_trait::async_trait]
        impl airssys_osl::core::executor::OSExecutor<#operation_path> for #executor_type {
            async fn execute(
                &self,
                operation: #operation_path,
                context: &airssys_osl::core::context::ExecutionContext,
            ) -> airssys_osl::core::result::OSResult<
                airssys_osl::core::executor::ExecutionResult
            > {
                self.#method_name(operation, context).await
            }
        }
    }
}
```

#### Step 4.2: Complete Expansion Function
```rust
pub fn expand(input: TokenStream) -> Result<TokenStream> {
    let item_impl = syn::parse2::<syn::ItemImpl>(input)?;
    
    // Extract and validate executor methods
    let executor_methods = extract_and_validate_methods(&item_impl)?;
    
    // Generate trait implementations
    let trait_impls = generate_trait_implementations(&item_impl, &executor_methods);
    
    // Return original impl + generated trait impls
    Ok(quote! {
        #item_impl
        #(#trait_impls)*
    })
}
```

### Phase 5: Testing & Documentation (Week 3)
**Goal:** Comprehensive testing and documentation

#### Integration Tests
```rust
// tests/integration.rs
use airssys_osl::prelude::*;
use airssys_osl_macros::executor;

struct CustomExecutor;

#[executor]
impl CustomExecutor {
    async fn file_read(
        &self,
        operation: FileReadOperation,
        _context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        Ok(ExecutionResult::success(b"test data".to_vec()))
    }
}

#[tokio::test]
async fn test_generated_file_read_executor() {
    let executor = CustomExecutor;
    let operation = FileReadOperation::new("/test".into());
    let context = ExecutionContext::new(SecurityContext::new("test".into()));
    
    let result = executor.execute(operation, &context).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().data, b"test data");
}
```

#### UI Tests
```rust
// tests/ui/invalid_receiver.rs
use airssys_osl_macros::executor;

struct MyExecutor;

#[executor]
impl MyExecutor {
    async fn file_read(&mut self, op: FileReadOperation, ctx: &ExecutionContext) 
        -> OSResult<ExecutionResult> 
    {
        todo!()
    }
}
```

## Testing Strategy

### Unit Tests (20+ tests)
- Method parsing and extraction
- Signature validation (async, receiver, parameters, return type)
- Operation name mapping (all 10 operations + invalid names)
- Error message generation
- Duplicate method detection

### Integration Tests (15+ tests)
- Single operation executor generation
- Multiple operations executor generation
- Complex executor with custom fields
- Real airssys-osl operation execution
- Error propagation through generated code

### UI Tests (10+ error cases)
- Invalid method signatures (non-async, wrong receiver, wrong params)
- Unknown operation names
- Invalid return types
- Missing parameters
- Syntax errors in impl blocks

## Documentation Requirements

### Macro Documentation
- Comprehensive rustdoc with examples
- Generated code examples
- Error message reference
- Method naming convention table
- Signature requirements
- Limitations and caveats

### Migration Guide
- Converting manual implementations to macro
- Side-by-side comparison
- Benefits and trade-offs
- Performance implications (none - zero-cost)

## Quality Checklist

- [ ] All 10 operation mappings implemented and tested
- [ ] Comprehensive signature validation with clear errors
- [ ] Code generation produces valid, idiomatic Rust
- [ ] 100% test coverage for mapping table
- [ ] Unit tests pass (20+)
- [ ] Integration tests pass (15+)
- [ ] UI tests pass (10+)
- [ ] Zero compiler warnings
- [ ] Zero clippy warnings
- [ ] Documentation complete
- [ ] Migration guide written
- [ ] Memory bank updated

## Success Metrics
- ~85% code reduction for custom executors
- Clear, actionable error messages
- Zero runtime overhead (compile-time only)
- Seamless integration with airssys-osl

## Next Steps
After completion, proceed to **MACROS-TASK-003**: Integrate with airssys-osl via feature flags.
