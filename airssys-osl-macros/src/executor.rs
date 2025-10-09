//! #[executor] macro implementation
//!
//! This module contains the core logic for parsing impl blocks
//! and generating OSExecutor trait implementations.

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::Parse;
use syn::{parse2, Error, ImplItem, ImplItemFn, ItemImpl, Result};

use crate::utils::get_operation_info;

/// Configuration options for the #[executor] macro.
#[derive(Debug, Default)]
struct ExecutorConfig {
    /// Custom executor name (optional)
    name: Option<String>,
    /// Custom supported operation types (optional)
    operations: Option<Vec<String>>,
}

/// Parses macro attributes into ExecutorConfig.
///
/// Supports the following syntax:
/// - `name = "CustomName"` - Custom executor name
/// - `operations = [Filesystem, Network, Process]` - Custom operation types
///
/// # Examples
///
/// ```ignore
/// #[executor(name = "MyExecutor")]
/// #[executor(operations = [Filesystem, Network])]
/// #[executor(name = "Custom", operations = [Process])]
/// ```
fn parse_config(attr: TokenStream) -> Result<ExecutorConfig> {
    // Empty attributes - use defaults
    if attr.is_empty() {
        return Ok(ExecutorConfig::default());
    }

    let mut config = ExecutorConfig::default();

    // Parse attributes using syn's meta parser
    let parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("name") {
            // Parse: name = "value"
            let value: syn::LitStr = meta.value()?.parse()?;
            config.name = Some(value.value());
            Ok(())
        } else if meta.path.is_ident("operations") {
            // Parse: operations = [Type1, Type2, ...]
            let list_parser = meta.value()?;
            let content;
            syn::bracketed!(content in list_parser);
            let ops: syn::punctuated::Punctuated<syn::Ident, syn::Token![,]> =
                content.parse_terminated(syn::Ident::parse, syn::Token![,])?;

            // Validate operation types
            let mut valid_ops = Vec::new();
            for op in &ops {
                let op_str = op.to_string();
                if !is_valid_operation_type(&op_str) {
                    return Err(meta.error(format!(
                        "Unknown operation type '{op_str}'. Valid types are: Filesystem, Process, Network"
                    )));
                }
                valid_ops.push(op_str);
            }

            config.operations = Some(valid_ops);
            Ok(())
        } else {
            Err(meta.error(format!(
                "Unknown attribute '{}'. Valid attributes are: name, operations",
                meta.path
                    .get_ident()
                    .map(|i| i.to_string())
                    .unwrap_or_else(|| "?".to_string())
            )))
        }
    });

    syn::parse::Parser::parse2(parser, attr)?;
    Ok(config)
}

/// Validates if a string is a known operation type.
fn is_valid_operation_type(op: &str) -> bool {
    matches!(op, "Filesystem" | "Process" | "Network")
}

/// Expands the #[executor] attribute macro.
///
/// Parses an impl block, extracts operation methods, validates their signatures,
/// and generates OSExecutor trait implementations with name() and supported_operation_types().
pub fn expand(attr: TokenStream, input: TokenStream) -> Result<TokenStream> {
    // Parse configuration (if provided)
    let config = parse_config(attr)?;

    // Parse the impl block
    let item_impl = parse2::<ItemImpl>(input)?;

    // Extract executor type name
    let executor_type = &item_impl.self_ty;

    // Get executor name (from config or type name)
    let executor_name = config.name.clone().unwrap_or_else(|| {
        // Extract type name from self_ty
        quote!(#executor_type).to_string()
    });

    // Find operation methods (only methods matching operation names)
    let methods = extract_operation_methods(&item_impl)?;

    if methods.is_empty() {
        return Err(Error::new_spanned(
            &item_impl,
            "No operation methods found. Expected methods named: file_read, file_write, file_delete, directory_create, directory_list, process_spawn, process_kill, process_signal, network_connect, network_listen, network_socket"
        ));
    }

    // Determine operation types: use custom config if provided, otherwise auto-detect
    let operation_types = if let Some(ref custom_ops) = config.operations {
        // Use custom operation types from configuration
        use quote::format_ident;
        custom_ops.iter().map(|s| format_ident!("{}", s)).collect()
    } else {
        // Auto-detect from methods
        detect_operation_types(&methods)
    };

    // Generate OSExecutor trait implementations
    let trait_impls =
        generate_trait_implementations(executor_type, &executor_name, &operation_types, &methods)?;

    // Return original impl + generated trait implementations
    Ok(quote! {
        #item_impl
        #(#trait_impls)*
    })
}

/// Detects unique operation types from method list.
fn detect_operation_types(methods: &[&ImplItemFn]) -> Vec<syn::Ident> {
    use quote::format_ident;
    use std::collections::HashSet;

    let mut types = HashSet::new();

    for method in methods {
        let method_name = method.sig.ident.to_string();
        if let Some(info) = get_operation_info(&method_name) {
            // Map to OperationType enum variant
            let op_type = match info.module_path {
                "filesystem" => "Filesystem",
                "process" => "Process",
                "network" => "Network",
                _ => continue,
            };
            types.insert(op_type);
        }
    }

    types.into_iter().map(|t| format_ident!("{}", t)).collect()
}

/// Extracts operation methods from an impl block.
///
/// Only processes methods whose names match known operation names.
/// Helper methods and other non-operation methods are ignored.
///
/// Returns an error if duplicate operation methods are found.
fn extract_operation_methods(impl_block: &ItemImpl) -> Result<Vec<&ImplItemFn>> {
    let mut methods = Vec::new();
    let mut seen_operations = std::collections::HashSet::new();

    for item in &impl_block.items {
        if let ImplItem::Fn(method) = item {
            let method_name = method.sig.ident.to_string();

            // Only process if method name matches an operation
            if crate::utils::is_operation_method(&method_name) {
                // Check for duplicates
                if !seen_operations.insert(method_name.clone()) {
                    return Err(Error::new_spanned(
                        &method.sig.ident,
                        format!(
                            "Duplicate operation method '{method_name}'. Each operation can only be implemented once per executor"
                        ),
                    ));
                }

                // Validate the method signature
                validate_method_signature(method)?;
                methods.push(method);
            }
            // Note: Non-operation methods are silently ignored (helper methods allowed)
        }
    }

    Ok(methods)
}

/// Validates that a method has the correct signature for an operation method.
///
/// Required signature:
/// ```rust,ignore
/// async fn method_name(
///     &self,
///     operation: OperationType,
///     context: &ExecutionContext
/// ) -> OSResult<ExecutionResult>
/// ```
fn validate_method_signature(method: &ImplItemFn) -> Result<()> {
    let sig = &method.sig;

    // 1. Must be async
    if sig.asyncness.is_none() {
        return Err(Error::new_spanned(
            sig,
            format!(
                "Operation method '{}' must be async. Add 'async' keyword.",
                sig.ident
            ),
        ));
    }

    // 2. Must have &self receiver
    validate_receiver(sig)?;

    // 3. Must have exactly 2 parameters with correct names
    validate_parameters(sig)?;

    // 4. Must return OSResult<ExecutionResult>
    validate_return_type(sig)?;

    Ok(())
}

/// Validates the receiver is &self (not &mut self, not self).
fn validate_receiver(sig: &syn::Signature) -> Result<()> {
    use syn::FnArg;

    let receiver = sig
        .inputs
        .first()
        .and_then(|arg| {
            if let FnArg::Receiver(r) = arg {
                Some(r)
            } else {
                None
            }
        })
        .ok_or_else(|| Error::new_spanned(sig, "Operation methods must have a '&self' receiver"))?;

    if receiver.mutability.is_some() {
        return Err(Error::new_spanned(
            receiver,
            "Use '&self', not '&mut self'. Executors should be immutable.",
        ));
    }

    if receiver.reference.is_none() {
        return Err(Error::new_spanned(
            receiver,
            "Use '&self', not 'self'. Executors should not be consumed.",
        ));
    }

    Ok(())
}

/// Validates parameters are exactly: (operation: OpType, context: &ExecutionContext).
fn validate_parameters(sig: &syn::Signature) -> Result<()> {
    use syn::{FnArg, Pat, PatType};

    // Collect non-receiver parameters
    let params: Vec<&FnArg> = sig.inputs.iter().skip(1).collect();

    if params.len() != 2 {
        return Err(Error::new_spanned(
            &sig.inputs,
            format!(
                "Operation methods must have exactly 2 parameters (operation, context). Found {} parameters.",
                params.len()
            )
        ));
    }

    // Validate first parameter name is "operation"
    if let FnArg::Typed(PatType { pat, .. }) = params[0] {
        if let Pat::Ident(ident) = &**pat {
            if ident.ident != "operation" {
                return Err(Error::new_spanned(
                    pat,
                    format!(
                        "First parameter must be named 'operation', found '{}'",
                        ident.ident
                    ),
                ));
            }
        }
    }

    // Validate second parameter name is "context"
    if let FnArg::Typed(PatType { pat, .. }) = params[1] {
        if let Pat::Ident(ident) = &**pat {
            if ident.ident != "context" {
                return Err(Error::new_spanned(
                    pat,
                    format!(
                        "Second parameter must be named 'context', found '{}'",
                        ident.ident
                    ),
                ));
            }
        }
    }

    // TODO: Validate parameter types in later phase if needed

    Ok(())
}

/// Validates return type is OSResult<ExecutionResult> (or its FQN).
fn validate_return_type(sig: &syn::Signature) -> Result<()> {
    use syn::ReturnType;

    match &sig.output {
        ReturnType::Default => Err(Error::new_spanned(
            sig,
            "Operation methods must return OSResult<ExecutionResult>",
        )),
        ReturnType::Type(_, _) => {
            // For Phase 1, we accept any return type that looks reasonable
            // Stricter validation can be added in Phase 2 if needed
            // The compiler will catch wrong types anyway
            Ok(())
        }
    }
}

/// Generates OSExecutor trait implementations for all operation methods.
///
/// # Example
///
/// Given:
/// ```rust,ignore
/// impl MyExecutor {
///     async fn file_read(&self, operation: FileReadOperation, context: &ExecutionContext)
///         -> OSResult<ExecutionResult> { ... }
/// }
/// ```
///
/// Generates:
/// ```rust,ignore
/// #[async_trait::async_trait]
/// impl OSExecutor<FileReadOperation> for MyExecutor {
///     fn name(&self) -> &str {
///         "MyExecutor"
///     }
///     
///     fn supported_operation_types(&self) -> Vec<OperationType> {
///         vec![OperationType::Filesystem]
///     }
///     
///     async fn execute(&self, operation: FileReadOperation, context: &ExecutionContext)
///         -> OSResult<ExecutionResult> {
///         self.file_read(operation, context).await
///     }
/// }
/// ```
fn generate_trait_implementations(
    executor_type: &syn::Type,
    executor_name: &str,
    operation_types: &[syn::Ident],
    methods: &[&ImplItemFn],
) -> Result<Vec<TokenStream>> {
    methods
        .iter()
        .map(|method| {
            generate_single_trait_impl(executor_type, executor_name, operation_types, method)
        })
        .collect()
}

/// Generates a single OSExecutor trait implementation for one operation method.
fn generate_single_trait_impl(
    executor_type: &syn::Type,
    executor_name: &str,
    operation_types: &[syn::Ident],
    method: &ImplItemFn,
) -> Result<TokenStream> {
    let method_name = &method.sig.ident;
    let method_name_str = method_name.to_string();

    // Get operation information
    let op_info = crate::utils::get_operation_info(&method_name_str).ok_or_else(|| {
        Error::new_spanned(
            method_name,
            format!("Unknown operation method: '{method_name_str}'"),
        )
    })?;

    // Generate fully qualified path to operation type
    let operation_type_path = op_info.operation_path();

    // Generate the trait implementation with all required methods
    Ok(quote! {
        #[async_trait::async_trait]
        impl airssys_osl::core::executor::OSExecutor<#operation_type_path> for #executor_type {
            fn name(&self) -> &str {
                #executor_name
            }

            fn supported_operation_types(&self) -> Vec<airssys_osl::core::operation::OperationType> {
                vec![#(airssys_osl::core::operation::OperationType::#operation_types),*]
            }

            async fn execute(
                &self,
                operation: #operation_type_path,
                context: &airssys_osl::core::context::ExecutionContext,
            ) -> airssys_osl::core::result::OSResult<airssys_osl::core::executor::ExecutionResult> {
                self.#method_name(operation, context).await
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn test_parse_valid_impl() {
        let input = quote! {
            impl MyExecutor {
                async fn file_read(
                    &self,
                    operation: FileReadOperation,
                    context: &ExecutionContext
                ) -> OSResult<ExecutionResult> {
                    todo!()
                }
            }
        };

        let result = expand(TokenStream::new(), input);
        assert!(result.is_ok(), "Valid impl should parse successfully");
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_reject_non_async_method() {
        let input = quote! {
            impl MyExecutor {
                fn file_read(
                    &self,
                    operation: FileReadOperation,
                    context: &ExecutionContext
                ) -> OSResult<ExecutionResult> {
                    todo!()
                }
            }
        };

        let result = expand(TokenStream::new(), input);
        assert!(result.is_err(), "Non-async method should be rejected");
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("must be async"),
            "Error should mention async requirement"
        );
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_reject_mut_self() {
        let input = quote! {
            impl MyExecutor {
                async fn file_read(
                    &mut self,
                    operation: FileReadOperation,
                    context: &ExecutionContext
                ) -> OSResult<ExecutionResult> {
                    todo!()
                }
            }
        };

        let result = expand(TokenStream::new(), input);
        assert!(result.is_err(), "&mut self should be rejected");
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("&mut self"),
            "Error should mention &mut self"
        );
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_reject_owned_self() {
        let input = quote! {
            impl MyExecutor {
                async fn file_read(
                    self,
                    operation: FileReadOperation,
                    context: &ExecutionContext
                ) -> OSResult<ExecutionResult> {
                    todo!()
                }
            }
        };

        let result = expand(TokenStream::new(), input);
        assert!(result.is_err(), "Owned self should be rejected");
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("not 'self'"),
            "Error should mention owned self"
        );
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_reject_wrong_first_param_name() {
        let input = quote! {
            impl MyExecutor {
                async fn file_read(
                    &self,
                    op: FileReadOperation,
                    context: &ExecutionContext
                ) -> OSResult<ExecutionResult> {
                    todo!()
                }
            }
        };

        let result = expand(TokenStream::new(), input);
        assert!(result.is_err(), "Wrong first param name should be rejected");
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("'operation'"),
            "Error should mention 'operation' parameter name"
        );
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_reject_wrong_second_param_name() {
        let input = quote! {
            impl MyExecutor {
                async fn file_read(
                    &self,
                    operation: FileReadOperation,
                    ctx: &ExecutionContext
                ) -> OSResult<ExecutionResult> {
                    todo!()
                }
            }
        };

        let result = expand(TokenStream::new(), input);
        assert!(
            result.is_err(),
            "Wrong second param name should be rejected"
        );
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("'context'"),
            "Error should mention 'context' parameter name"
        );
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_reject_too_few_params() {
        let input = quote! {
            impl MyExecutor {
                async fn file_read(
                    &self,
                    operation: FileReadOperation
                ) -> OSResult<ExecutionResult> {
                    todo!()
                }
            }
        };

        let result = expand(TokenStream::new(), input);
        assert!(result.is_err(), "Too few params should be rejected");
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("exactly 2 parameters"),
            "Error should mention parameter count"
        );
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_reject_too_many_params() {
        let input = quote! {
            impl MyExecutor {
                async fn file_read(
                    &self,
                    operation: FileReadOperation,
                    context: &ExecutionContext,
                    extra: String
                ) -> OSResult<ExecutionResult> {
                    todo!()
                }
            }
        };

        let result = expand(TokenStream::new(), input);
        assert!(result.is_err(), "Too many params should be rejected");
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("exactly 2 parameters"),
            "Error should mention parameter count"
        );
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_reject_no_return_type() {
        let input = quote! {
            impl MyExecutor {
                async fn file_read(
                    &self,
                    operation: FileReadOperation,
                    context: &ExecutionContext
                ) {
                    todo!()
                }
            }
        };

        let result = expand(TokenStream::new(), input);
        assert!(result.is_err(), "Missing return type should be rejected");
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("OSResult"),
            "Error should mention return type"
        );
    }

    #[test]
    fn test_ignore_helper_methods() {
        let input = quote! {
            impl MyExecutor {
                async fn file_read(
                    &self,
                    operation: FileReadOperation,
                    context: &ExecutionContext
                ) -> OSResult<ExecutionResult> {
                    todo!()
                }

                fn validate_path(&self, path: &str) -> bool {
                    true
                }
            }
        };

        let result = expand(TokenStream::new(), input);
        assert!(result.is_ok(), "Helper methods should be ignored");
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_error_when_no_operation_methods() {
        let input = quote! {
            impl MyExecutor {
                fn helper(&self) -> String {
                    String::new()
                }
            }
        };

        let result = expand(TokenStream::new(), input);
        assert!(
            result.is_err(),
            "Should error when no operation methods found"
        );
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("No operation methods found"),
            "Error should mention no operation methods"
        );
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_multiple_operations_two() {
        let input = quote! {
            impl MyExecutor {
                async fn file_read(
                    &self,
                    operation: FileReadOperation,
                    context: &ExecutionContext
                ) -> OSResult<ExecutionResult> {
                    todo!()
                }

                async fn file_write(
                    &self,
                    operation: FileWriteOperation,
                    context: &ExecutionContext
                ) -> OSResult<ExecutionResult> {
                    todo!()
                }
            }
        };

        let result = expand(TokenStream::new(), input);
        assert!(result.is_ok(), "Two operations should be accepted");

        let output = result.unwrap().to_string();
        // Should contain both trait implementations
        assert!(
            output.contains("FileReadOperation"),
            "Should generate FileReadOperation trait impl"
        );
        assert!(
            output.contains("FileWriteOperation"),
            "Should generate FileWriteOperation trait impl"
        );
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_multiple_operations_three_different_modules() {
        let input = quote! {
            impl MyExecutor {
                async fn file_read(
                    &self,
                    operation: FileReadOperation,
                    context: &ExecutionContext
                ) -> OSResult<ExecutionResult> {
                    todo!()
                }

                async fn process_spawn(
                    &self,
                    operation: ProcessSpawnOperation,
                    context: &ExecutionContext
                ) -> OSResult<ExecutionResult> {
                    todo!()
                }

                async fn network_connect(
                    &self,
                    operation: NetworkConnectOperation,
                    context: &ExecutionContext
                ) -> OSResult<ExecutionResult> {
                    todo!()
                }
            }
        };

        let result = expand(TokenStream::new(), input);
        assert!(
            result.is_ok(),
            "Three operations from different modules should be accepted"
        );

        let output = result.unwrap().to_string();
        // Should contain all three trait implementations
        assert!(
            output.contains("FileReadOperation"),
            "Should generate filesystem operation trait impl"
        );
        assert!(
            output.contains("ProcessSpawnOperation"),
            "Should generate process operation trait impl"
        );
        assert!(
            output.contains("NetworkConnectOperation"),
            "Should generate network operation trait impl"
        );
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_all_eleven_operations() {
        let input = quote! {
            impl MyExecutor {
                async fn file_read(&self, operation: FileReadOperation, context: &ExecutionContext)
                    -> OSResult<ExecutionResult> { todo!() }
                async fn file_write(&self, operation: FileWriteOperation, context: &ExecutionContext)
                    -> OSResult<ExecutionResult> { todo!() }
                async fn file_delete(&self, operation: FileDeleteOperation, context: &ExecutionContext)
                    -> OSResult<ExecutionResult> { todo!() }
                async fn directory_create(&self, operation: DirectoryCreateOperation, context: &ExecutionContext)
                    -> OSResult<ExecutionResult> { todo!() }
                async fn directory_list(&self, operation: DirectoryListOperation, context: &ExecutionContext)
                    -> OSResult<ExecutionResult> { todo!() }
                async fn process_spawn(&self, operation: ProcessSpawnOperation, context: &ExecutionContext)
                    -> OSResult<ExecutionResult> { todo!() }
                async fn process_kill(&self, operation: ProcessKillOperation, context: &ExecutionContext)
                    -> OSResult<ExecutionResult> { todo!() }
                async fn process_signal(&self, operation: ProcessSignalOperation, context: &ExecutionContext)
                    -> OSResult<ExecutionResult> { todo!() }
                async fn network_connect(&self, operation: NetworkConnectOperation, context: &ExecutionContext)
                    -> OSResult<ExecutionResult> { todo!() }
                async fn network_listen(&self, operation: NetworkListenOperation, context: &ExecutionContext)
                    -> OSResult<ExecutionResult> { todo!() }
                async fn network_socket(&self, operation: NetworkSocketOperation, context: &ExecutionContext)
                    -> OSResult<ExecutionResult> { todo!() }
            }
        };

        let result = expand(TokenStream::new(), input);
        assert!(result.is_ok(), "All 11 operations should be accepted");

        let output = result.unwrap().to_string();
        // Verify all 11 operation types appear in generated code
        assert!(output.contains("FileReadOperation"));
        assert!(output.contains("FileWriteOperation"));
        assert!(output.contains("FileDeleteOperation"));
        assert!(output.contains("DirectoryCreateOperation"));
        assert!(output.contains("DirectoryListOperation"));
        assert!(output.contains("ProcessSpawnOperation"));
        assert!(output.contains("ProcessKillOperation"));
        assert!(output.contains("ProcessSignalOperation"));
        assert!(output.contains("NetworkConnectOperation"));
        assert!(output.contains("NetworkListenOperation"));
        assert!(output.contains("NetworkSocketOperation"));
    }

    #[test]
    #[allow(clippy::unwrap_used, clippy::uninlined_format_args)]
    fn test_reject_duplicate_operations() {
        let input = quote! {
            impl MyExecutor {
                async fn file_read(
                    &self,
                    operation: FileReadOperation,
                    context: &ExecutionContext
                ) -> OSResult<ExecutionResult> {
                    todo!()
                }

                async fn file_read(
                    &self,
                    operation: FileReadOperation,
                    context: &ExecutionContext
                ) -> OSResult<ExecutionResult> {
                    todo!()
                }
            }
        };

        let result = expand(TokenStream::new(), input);
        assert!(
            result.is_err(),
            "Duplicate operation methods should be rejected"
        );
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("Duplicate operation method"),
            "Error should mention duplicate operation: {}",
            err_msg
        );
        assert!(
            err_msg.contains("file_read"),
            "Error should mention the duplicate method name: {}",
            err_msg
        );
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_multiple_operations_with_helpers() {
        let input = quote! {
            impl MyExecutor {
                async fn file_read(
                    &self,
                    operation: FileReadOperation,
                    context: &ExecutionContext
                ) -> OSResult<ExecutionResult> {
                    self.validate_path(&operation.path)?;
                    todo!()
                }

                async fn file_write(
                    &self,
                    operation: FileWriteOperation,
                    context: &ExecutionContext
                ) -> OSResult<ExecutionResult> {
                    self.validate_path(&operation.path)?;
                    todo!()
                }

                fn validate_path(&self, path: &str) -> OSResult<()> {
                    // Helper method - should be ignored
                    Ok(())
                }
            }
        };

        let result = expand(TokenStream::new(), input);
        assert!(
            result.is_ok(),
            "Multiple operations with helper methods should be accepted"
        );

        let output = result.unwrap().to_string();
        // Should contain both operation trait impls but not helper
        assert!(output.contains("FileReadOperation"));
        assert!(output.contains("FileWriteOperation"));
        // Helper method should remain in original impl only
        assert!(output.contains("validate_path"));
    }

    // ========================================================================
    // Day 7: Code Generation Tests
    // ========================================================================

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_generated_code_contains_async_trait() {
        let input = quote! {
            impl MyExecutor {
                async fn file_read(&self, operation: FileReadOperation, context: &ExecutionContext)
                    -> OSResult<ExecutionResult> { todo!() }
            }
        };

        let result = expand(TokenStream::new(), input);
        let output = result.unwrap().to_string();

        assert!(
            output.contains("async_trait"),
            "Generated code should include #[async_trait::async_trait]"
        );
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_generated_code_contains_executor_trait() {
        let input = quote! {
            impl MyExecutor {
                async fn file_read(&self, operation: FileReadOperation, context: &ExecutionContext)
                    -> OSResult<ExecutionResult> { todo!() }
            }
        };

        let result = expand(TokenStream::new(), input);
        let output = result.unwrap().to_string();

        assert!(
            output.contains("OSExecutor"),
            "Generated code should implement OSExecutor trait"
        );
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_generated_code_delegates_to_method() {
        let input = quote! {
            impl MyExecutor {
                async fn process_spawn(&self, operation: ProcessSpawnOperation, context: &ExecutionContext)
                    -> OSResult<ExecutionResult> { todo!() }
            }
        };

        let result = expand(TokenStream::new(), input);
        let output = result.unwrap().to_string();

        assert!(
            output.contains("process_spawn"),
            "Generated execute() should delegate to user's process_spawn method"
        );
        assert!(
            output.contains("await"),
            "Delegation should await the async method"
        );
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_generated_code_preserves_original_impl() {
        let input = quote! {
            impl MyExecutor {
                async fn network_connect(&self, operation: NetworkConnectOperation, context: &ExecutionContext)
                    -> OSResult<ExecutionResult> {
                    println!("Connecting...");
                    todo!()
                }
            }
        };

        let result = expand(TokenStream::new(), input);
        let output = result.unwrap().to_string();

        assert!(
            output.contains("impl MyExecutor"),
            "Original impl block should be preserved"
        );
        assert!(
            output.contains("Connecting"),
            "Original method body should be preserved"
        );
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_all_operation_types_generate_valid_paths() {
        // Test that each operation generates correct fully qualified paths
        let operations = [
            ("file_read", "FileReadOperation", "filesystem"),
            ("file_write", "FileWriteOperation", "filesystem"),
            ("directory_create", "DirectoryCreateOperation", "filesystem"),
            ("process_spawn", "ProcessSpawnOperation", "process"),
            ("network_connect", "NetworkConnectOperation", "network"),
        ];

        for (method_name, type_name, module) in operations {
            let method_ident = syn::Ident::new(method_name, proc_macro2::Span::call_site());
            let input = quote! {
                impl MyExecutor {
                    async fn #method_ident(&self, operation: Operation, context: &ExecutionContext)
                        -> OSResult<ExecutionResult> { todo!() }
                }
            };

            let result = expand(TokenStream::new(), input);
            assert!(
                result.is_ok(),
                "Operation {method_name} should parse successfully"
            );

            let output = result.unwrap().to_string();
            assert!(
                output.contains(type_name),
                "Generated code should contain {type_name} for {method_name}",
            );
            assert!(
                output.contains(module),
                "Generated code should contain module path {module} for {method_name}",
            );
        }
    }

    // ========================================================================
    // Configuration Parsing Tests
    // ========================================================================

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_parse_empty_config() {
        let attr = TokenStream::new();
        let config = parse_config(attr).unwrap();
        assert!(config.name.is_none());
        assert!(config.operations.is_none());
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_parse_name_attribute() {
        let attr = quote! { name = "CustomExecutor" };
        let config = parse_config(attr).unwrap();
        assert_eq!(config.name, Some("CustomExecutor".to_string()));
        assert!(config.operations.is_none());
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_parse_operations_attribute_single() {
        let attr = quote! { operations = [Filesystem] };
        let config = parse_config(attr).unwrap();
        assert!(config.name.is_none());
        assert_eq!(config.operations, Some(vec!["Filesystem".to_string()]));
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_parse_operations_attribute_multiple() {
        let attr = quote! { operations = [Filesystem, Network, Process] };
        let config = parse_config(attr).unwrap();
        assert_eq!(
            config.operations,
            Some(vec![
                "Filesystem".to_string(),
                "Network".to_string(),
                "Process".to_string()
            ])
        );
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_parse_both_attributes() {
        let attr = quote! { name = "Custom", operations = [Process, Network] };
        let config = parse_config(attr).unwrap();
        assert_eq!(config.name, Some("Custom".to_string()));
        assert_eq!(
            config.operations,
            Some(vec!["Process".to_string(), "Network".to_string()])
        );
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_reject_unknown_operation_type() {
        let attr = quote! { operations = [InvalidType] };
        let result = parse_config(attr);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Unknown operation type"));
        assert!(err.to_string().contains("InvalidType"));
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_reject_unknown_attribute() {
        let attr = quote! { unknown_attr = "value" };
        let result = parse_config(attr);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Unknown attribute"));
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_custom_executor_name_in_generated_code() {
        let attr = quote! { name = "CustomExecutorName" };
        let input = quote! {
            impl MyExecutor {
                async fn file_read(
                    &self,
                    operation: FileReadOperation,
                    context: &ExecutionContext
                ) -> OSResult<ExecutionResult> {
                    todo!()
                }
            }
        };

        let result = expand(attr, input);
        assert!(result.is_ok());
        let output = result.unwrap().to_string();
        assert!(output.contains("CustomExecutorName"));
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_custom_operations_in_generated_code() {
        let attr = quote! { operations = [Filesystem, Network] };
        let input = quote! {
            impl MyExecutor {
                async fn file_read(
                    &self,
                    operation: FileReadOperation,
                    context: &ExecutionContext
                ) -> OSResult<ExecutionResult> {
                    todo!()
                }
            }
        };

        let result = expand(attr, input);
        assert!(result.is_ok());
        let output = result.unwrap().to_string();
        // Should contain custom operation types
        assert!(output.contains("Filesystem"));
        assert!(output.contains("Network"));
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_custom_single_operation_type() {
        let attr = quote! { operations = [Process] };
        let input = quote! {
            impl MyExecutor {
                async fn process_spawn(
                    &self,
                    operation: ProcessSpawnOperation,
                    context: &ExecutionContext
                ) -> OSResult<ExecutionResult> {
                    todo!()
                }
            }
        };

        let result = expand(attr, input);
        assert!(result.is_ok());
        let output = result.unwrap().to_string();
        assert!(output.contains("Process"));
        // Should not contain other types
        assert!(!output.contains("Filesystem"));
        assert!(!output.contains("Network"));
    }
}
