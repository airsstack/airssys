# airssys-osl-macros System Patterns

## Macro Architecture Patterns

### Token Stream Processing Pattern
```rust
// Entry point pattern
#[proc_macro_attribute]
pub fn executor(_attr: TokenStream, item: TokenStream) -> TokenStream {
    match executor::expand(item.into()) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

// Expansion pattern (in executor.rs)
pub fn expand(input: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let item_impl = syn::parse2::<syn::ItemImpl>(input)?;
    
    // Validate and extract methods
    let executor_methods = extract_executor_methods(&item_impl)?;
    
    // Generate trait implementations
    let trait_impls = generate_trait_implementations(&item_impl, &executor_methods);
    
    Ok(quote! {
        #item_impl
        #(#trait_impls)*
    })
}
```

### Method Parsing Pattern
```rust
pub struct ExecutorMethod {
    pub method_name: syn::Ident,
    pub operation_type: OperationType,
    pub operation_path: syn::Path,
    pub signature: syn::Signature,
}

fn extract_executor_methods(impl_block: &syn::ItemImpl) -> syn::Result<Vec<ExecutorMethod>> {
    let mut methods = Vec::new();
    
    for item in &impl_block.items {
        if let syn::ImplItem::Fn(method) = item {
            // Validate method signature
            validate_executor_method_signature(&method.sig)?;
            
            // Map method name to operation type
            if let Some(op_type) = map_method_name_to_operation(&method.sig.ident) {
                methods.push(ExecutorMethod {
                    method_name: method.sig.ident.clone(),
                    operation_type: op_type.clone(),
                    operation_path: op_type.full_path(),
                    signature: method.sig.clone(),
                });
            }
        }
    }
    
    Ok(methods)
}
```

### Operation Mapping Pattern
```rust
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

pub fn map_method_name_to_operation(name: &syn::Ident) -> Option<OperationType> {
    match name.to_string().as_str() {
        "file_read" => Some(OperationType {
            method_name: "file_read",
            type_name: "FileReadOperation",
            module_path: "filesystem",
        }),
        "process_spawn" => Some(OperationType {
            method_name: "process_spawn",
            type_name: "ProcessSpawnOperation",
            module_path: "process",
        }),
        // ... other mappings
        _ => None,
    }
}
```

### Code Generation Pattern
```rust
fn generate_trait_implementation(
    impl_block: &syn::ItemImpl,
    method: &ExecutorMethod,
) -> proc_macro2::TokenStream {
    let executor_type = &impl_block.self_ty;
    let operation_path = &method.operation_path;
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

### Error Handling Pattern
```rust
// Validation with detailed error messages
fn validate_executor_method_signature(sig: &syn::Signature) -> syn::Result<()> {
    // Check async
    if sig.asyncness.is_none() {
        return Err(syn::Error::new_spanned(
            sig,
            "executor methods must be async functions"
        ));
    }
    
    // Check receiver
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
    
    // Check parameter count
    if sig.inputs.len() != 3 {
        return Err(syn::Error::new_spanned(
            &sig.inputs,
            format!(
                "executor methods must have exactly 2 parameters (operation, context), found {}",
                sig.inputs.len() - 1
            )
        ));
    }
    
    Ok(())
}
```

## Testing Patterns

### Unit Test Pattern
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    
    #[test]
    fn test_method_name_mapping() {
        let ident = syn::parse_str::<syn::Ident>("file_read").unwrap();
        let op_type = map_method_name_to_operation(&ident).unwrap();
        
        assert_eq!(op_type.type_name, "FileReadOperation");
        assert_eq!(op_type.module_path, "filesystem");
    }
    
    #[test]
    fn test_invalid_method_name() {
        let ident = syn::parse_str::<syn::Ident>("invalid_operation").unwrap();
        assert!(map_method_name_to_operation(&ident).is_none());
    }
}
```

### UI Test Pattern (trybuild)
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

// Expected error in tests/ui/invalid_receiver.stderr:
// error: executor methods must use &self, not &mut self
//   --> tests/ui/invalid_receiver.rs:7:28
//    |
// 7  |     async fn file_read(&mut self, op: FileReadOperation, ctx: &ExecutionContext)
//    |                        ^^^^^^^^^
```

### Integration Test Pattern
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
        Ok(ExecutionResult::success(vec![1, 2, 3]))
    }
}

#[tokio::test]
async fn test_generated_executor() {
    let executor = CustomExecutor;
    let operation = FileReadOperation::new("/test".into());
    let context = ExecutionContext::new(SecurityContext::new("test".into()));
    
    let result = executor.execute(operation, &context).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().data, vec![1, 2, 3]);
}
```

## Documentation Patterns

### Macro Documentation Pattern
```rust
/// Generates `OSExecutor<O>` trait implementations for custom executors.
///
/// This attribute macro reduces boilerplate by automatically generating trait
/// implementations from method names. Each method name maps to a specific
/// operation type, and the macro generates the corresponding `OSExecutor<O>` impl.
///
/// # Method Naming Convention
///
/// Methods must follow this naming pattern to be recognized:
///
/// - `file_read` → `OSExecutor<FileReadOperation>`
/// - `file_write` → `OSExecutor<FileWriteOperation>`
/// - `process_spawn` → `OSExecutor<ProcessSpawnOperation>`
/// - `tcp_connect` → `OSExecutor<TcpConnectOperation>`
/// - ... (see full list in documentation)
///
/// # Method Signature Requirements
///
/// Each executor method must follow this exact signature:
///
/// ```rust,ignore
/// async fn <operation_name>(
///     &self,                                    // Required: immutable reference
///     operation: <OperationType>,               // Required: specific operation type
///     context: &ExecutionContext,               // Required: execution context
/// ) -> OSResult<ExecutionResult>                // Required: result type
/// ```
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use airssys_osl::prelude::*;
///
/// struct MyExecutor {
///     base_path: PathBuf,
/// }
///
/// #[executor]
/// impl MyExecutor {
///     async fn file_read(
///         &self,
///         operation: FileReadOperation,
///         context: &ExecutionContext,
///     ) -> OSResult<ExecutionResult> {
///         // Custom implementation
///         let full_path = self.base_path.join(&operation.path);
///         let data = tokio::fs::read(full_path).await?;
///         Ok(ExecutionResult::success(data))
///     }
/// }
/// ```
///
/// ## Generated Code
///
/// The macro generates this trait implementation:
///
/// ```rust,ignore
/// #[async_trait::async_trait]
/// impl OSExecutor<FileReadOperation> for MyExecutor {
///     async fn execute(
///         &self,
///         operation: FileReadOperation,
///         context: &ExecutionContext,
///     ) -> OSResult<ExecutionResult> {
///         self.file_read(operation, context).await
///     }
/// }
/// ```
///
/// # Error Messages
///
/// The macro provides detailed error messages for common mistakes:
///
/// ```rust,compile_fail
/// # use airssys_osl::prelude::*;
/// # struct MyExecutor;
/// #[executor]
/// impl MyExecutor {
///     // Error: executor methods must be async functions
///     fn file_read(&self, op: FileReadOperation, ctx: &ExecutionContext) 
///         -> OSResult<ExecutionResult> { todo!() }
/// }
/// ```
///
/// ```rust,compile_fail
/// # use airssys_osl::prelude::*;
/// # struct MyExecutor;
/// #[executor]
/// impl MyExecutor {
///     // Error: executor methods must use &self, not &mut self
///     async fn file_read(&mut self, op: FileReadOperation, ctx: &ExecutionContext) 
///         -> OSResult<ExecutionResult> { todo!() }
/// }
/// ```
///
/// # Limitations
///
/// - Method names must exactly match supported operation names
/// - Signature must follow the exact pattern (no variations)
/// - Cannot mix manual and macro-generated implementations for same type
/// - Macro expansion happens at compile time (no runtime cost)
#[proc_macro_attribute]
pub fn executor(_attr: TokenStream, item: TokenStream) -> TokenStream;
```

### Migration Guide Pattern
```markdown
## Migrating from Manual to Macro Implementation

### Before (Manual Implementation)
```rust
use async_trait::async_trait;
use airssys_osl::prelude::*;

struct MyExecutor;

#[async_trait]
impl OSExecutor<FileReadOperation> for MyExecutor {
    async fn execute(
        &self,
        operation: FileReadOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        // Implementation
        todo!()
    }
}

#[async_trait]
impl OSExecutor<FileWriteOperation> for MyExecutor {
    async fn execute(
        &self,
        operation: FileWriteOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        // Implementation
        todo!()
    }
}
```

### After (Macro Implementation)
```rust
use airssys_osl::prelude::*;

struct MyExecutor;

#[executor]
impl MyExecutor {
    async fn file_read(
        &self,
        operation: FileReadOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        // Same implementation
        todo!()
    }
    
    async fn file_write(
        &self,
        operation: FileWriteOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        // Same implementation
        todo!()
    }
}
```

### Benefits
- ~85% less boilerplate code
- No need for `#[async_trait]` on impl blocks
- Clearer method naming (operation-focused)
- Easier to scan and understand
- Same performance (zero-cost abstraction)
```

## Performance Patterns

### Compile-Time Optimization
```rust
// Lazy static for operation mappings (computed once)
use once_cell::sync::Lazy;

static OPERATION_MAP: Lazy<HashMap<&'static str, OperationType>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert("file_read", OperationType { /* ... */ });
    map.insert("file_write", OperationType { /* ... */ });
    // ... all mappings
    map
});

pub fn map_method_name_to_operation(name: &syn::Ident) -> Option<&'static OperationType> {
    OPERATION_MAP.get(name.to_string().as_str())
}
```

### Minimal Token Generation
```rust
// Generate only necessary code, avoid redundancy
fn generate_minimal_impl(method: &ExecutorMethod) -> proc_macro2::TokenStream {
    let op = &method.operation_path;
    let name = &method.method_name;
    
    // Use type aliases for brevity
    quote! {
        #[async_trait::async_trait]
        impl OSExecutor<#op> for Self {
            async fn execute(&self, operation: #op, context: &ExecutionContext) 
                -> OSResult<ExecutionResult> 
            {
                self.#name(operation, context).await
            }
        }
    }
}
```

## Workspace Standards Compliance

### §2.1 Import Organization
```rust
// Standard library
use std::collections::HashMap;
use proc_macro::TokenStream;

// Third-party
use quote::quote;
use syn::{parse_macro_input, ItemImpl};

// Internal
use crate::utils::validate_signature;
```

### §4.3 Module Architecture
```rust
// mod.rs - ONLY declarations and re-exports
pub mod executor;
pub mod operation;
pub mod utils;

pub use executor::expand as expand_executor;
```

### §6.1 YAGNI Principles
- Implement only #[executor] macro initially
- Add #[operation] macro only when needed
- No speculative features or abstractions
- Build exactly what's required
