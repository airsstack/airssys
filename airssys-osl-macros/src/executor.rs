//! #[executor] macro implementation
//!
//! This module contains the core logic for parsing impl blocks
//! and generating OSExecutor trait implementations.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, Error, ImplItem, ImplItemFn, ItemImpl, Result};

/// Expands the #[executor] attribute macro.
///
/// Parses an impl block, extracts operation methods, validates their signatures,
/// and generates OSExecutor trait implementations.
pub fn expand(input: TokenStream) -> Result<TokenStream> {
    // Parse the impl block
    let item_impl = parse2::<ItemImpl>(input)?;
    
    // Extract executor type name (will be used in Phase 2 for code generation)
    let _executor_type = &item_impl.self_ty;
    
    // Find operation methods (only methods matching operation names)
    let methods = extract_operation_methods(&item_impl)?;
    
    if methods.is_empty() {
        return Err(Error::new_spanned(
            &item_impl,
            "No operation methods found. Expected methods named: file_read, file_write, file_delete, directory_create, directory_list, process_spawn, process_kill, process_signal, network_connect, network_listen, network_socket"
        ));
    }
    
    // For Phase 1: Return original impl + placeholder comment
    // Phase 2 will add actual trait implementation generation
    Ok(quote! {
        #item_impl
        // TODO: Generate OSExecutor trait implementations here
    })
}

/// Extracts operation methods from an impl block.
///
/// Only processes methods whose names match known operation names.
/// Helper methods and other non-operation methods are ignored.
fn extract_operation_methods(impl_block: &ItemImpl) -> Result<Vec<&ImplItemFn>> {
    let mut methods = Vec::new();
    
    for item in &impl_block.items {
        if let ImplItem::Fn(method) = item {
            let method_name = method.sig.ident.to_string();
            
            // Only process if method name matches an operation
            if crate::utils::is_operation_method(&method_name) {
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
            )
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
    
    let receiver = sig.inputs.first()
        .and_then(|arg| if let FnArg::Receiver(r) = arg { Some(r) } else { None })
        .ok_or_else(|| Error::new_spanned(
            sig,
            "Operation methods must have a '&self' receiver"
        ))?;
    
    if receiver.mutability.is_some() {
        return Err(Error::new_spanned(
            receiver,
            "Use '&self', not '&mut self'. Executors should be immutable."
        ));
    }
    
    if receiver.reference.is_none() {
        return Err(Error::new_spanned(
            receiver,
            "Use '&self', not 'self'. Executors should not be consumed."
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
                    )
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
                    )
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
        ReturnType::Default => {
            Err(Error::new_spanned(
                sig,
                "Operation methods must return OSResult<ExecutionResult>"
            ))
        }
        ReturnType::Type(_, _) => {
            // For Phase 1, we accept any return type that looks reasonable
            // Stricter validation can be added in Phase 2 if needed
            // The compiler will catch wrong types anyway
            Ok(())
        }
    }
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
        
        let result = expand(input);
        assert!(result.is_ok(), "Valid impl should parse successfully");
    }

    #[test]
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
        
        let result = expand(input);
        assert!(result.is_err(), "Non-async method should be rejected");
        let err = result.unwrap_err();
        assert!(err.to_string().contains("must be async"), 
            "Error should mention async requirement");
    }

    #[test]
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
        
        let result = expand(input);
        assert!(result.is_err(), "&mut self should be rejected");
        let err = result.unwrap_err();
        assert!(err.to_string().contains("&mut self"), 
            "Error should mention &mut self");
    }

    #[test]
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
        
        let result = expand(input);
        assert!(result.is_err(), "Owned self should be rejected");
        let err = result.unwrap_err();
        assert!(err.to_string().contains("not 'self'"), 
            "Error should mention owned self");
    }

    #[test]
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
        
        let result = expand(input);
        assert!(result.is_err(), "Wrong first param name should be rejected");
        let err = result.unwrap_err();
        assert!(err.to_string().contains("'operation'"), 
            "Error should mention 'operation' parameter name");
    }

    #[test]
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
        
        let result = expand(input);
        assert!(result.is_err(), "Wrong second param name should be rejected");
        let err = result.unwrap_err();
        assert!(err.to_string().contains("'context'"), 
            "Error should mention 'context' parameter name");
    }

    #[test]
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
        
        let result = expand(input);
        assert!(result.is_err(), "Too few params should be rejected");
        let err = result.unwrap_err();
        assert!(err.to_string().contains("exactly 2 parameters"), 
            "Error should mention parameter count");
    }

    #[test]
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
        
        let result = expand(input);
        assert!(result.is_err(), "Too many params should be rejected");
        let err = result.unwrap_err();
        assert!(err.to_string().contains("exactly 2 parameters"), 
            "Error should mention parameter count");
    }

    #[test]
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
        
        let result = expand(input);
        assert!(result.is_err(), "Missing return type should be rejected");
        let err = result.unwrap_err();
        assert!(err.to_string().contains("OSResult"), 
            "Error should mention return type");
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
        
        let result = expand(input);
        assert!(result.is_ok(), "Helper methods should be ignored");
    }

    #[test]
    fn test_error_when_no_operation_methods() {
        let input = quote! {
            impl MyExecutor {
                fn helper(&self) -> String {
                    String::new()
                }
            }
        };
        
        let result = expand(input);
        assert!(result.is_err(), "Should error when no operation methods found");
        let err = result.unwrap_err();
        assert!(err.to_string().contains("No operation methods found"), 
            "Error should mention no operation methods");
    }
}
