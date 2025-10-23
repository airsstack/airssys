//! Component loading and validation implementation.
//!
//! This module provides utilities for loading WebAssembly components from
//! various sources (files, bytes) with validation and error handling.
//!
//! # Architecture
//!
//! ```text
//! ComponentLoader
//!     ├── load_from_file() → Read → Validate → Component
//!     ├── load_from_bytes() → Validate → Component
//!     └── validate() → Format check + Semantic validation
//! ```
//!
//! # Design Decisions
//!
//! - **Validation-first**: All components validated before loading
//! - **Error Context**: Rich error messages for debugging
//! - **Async I/O**: Non-blocking file operations with tokio
//! - **Format Support**: Component Model binary format (.wasm)
//!
//! # Example
//!
//! ```rust,ignore
//! use airssys_wasm::runtime::ComponentLoader;
//! use airssys_wasm::runtime::WasmEngine;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let engine = WasmEngine::new()?;
//!     let loader = ComponentLoader::new(&engine);
//!     
//!     // Load from file
//!     let bytes = loader.load_from_file("component.wasm").await?;
//!     println!("Loaded {} bytes", bytes.len());
//!     
//!     // Validate component
//!     loader.validate(&bytes)?;
//!     println!("Component validated successfully");
//!     
//!     Ok(())
//! }
//! ```

// Layer 1: Standard library imports (§2.1 - 3-layer import organization)
use std::path::Path;

// Layer 2: External crate imports
use tokio::fs;
use wasmtime::component::Component;

// Layer 3: Internal module imports
use crate::core::error::{WasmError, WasmResult};
use crate::runtime::WasmEngine;

/// Component loader for WebAssembly components.
///
/// Provides utilities to load and validate WebAssembly components from
/// files and byte arrays. Works with `WasmEngine` to ensure components
/// are properly formatted and compatible with the runtime.
///
/// # Design Pattern
///
/// Uses reference to `WasmEngine` rather than ownership to allow:
/// - Multiple loaders sharing same engine
/// - Flexible composition with other runtime components
/// - Clear separation between loading and execution concerns
///
/// # Example
///
/// ```rust,ignore
/// use airssys_wasm::runtime::{WasmEngine, ComponentLoader};
///
/// let engine = WasmEngine::new()?;
/// let loader = ComponentLoader::new(&engine);
///
/// // Load and validate component
/// let bytes = loader.load_from_file("my_component.wasm").await?;
/// loader.validate(&bytes)?;
/// ```
pub struct ComponentLoader<'a> {
    #[allow(dead_code)]
    engine: &'a WasmEngine,
}

impl<'a> ComponentLoader<'a> {
    /// Create a new component loader.
    ///
    /// # Parameters
    ///
    /// - `engine`: Reference to WasmEngine for validation context
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use airssys_wasm::runtime::{WasmEngine, ComponentLoader};
    ///
    /// let engine = WasmEngine::new()?;
    /// let loader = ComponentLoader::new(&engine);
    /// ```
    pub fn new(engine: &'a WasmEngine) -> Self {
        Self { engine }
    }
    
    /// Load component from file path.
    ///
    /// Reads component bytes from filesystem with validation.
    /// Uses async I/O for non-blocking operation.
    ///
    /// # Parameters
    ///
    /// - `path`: Path to .wasm component file
    ///
    /// # Returns
    ///
    /// - `Ok(Vec<u8>)`: Component bytes loaded successfully
    /// - `Err(WasmError)`: File read failed or invalid path
    ///
    /// # Errors
    ///
    /// - `WasmError::IoError`: File not found or permission denied
    /// - `WasmError::ComponentLoadFailed`: Invalid file format
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let bytes = loader.load_from_file("component.wasm").await?;
    /// println!("Loaded {} bytes", bytes.len());
    /// ```
    pub async fn load_from_file<P: AsRef<Path>>(&self, path: P) -> WasmResult<Vec<u8>> {
        let path = path.as_ref();
        
        // Read component bytes from filesystem
        let bytes = fs::read(path).await.map_err(|e| {
            WasmError::component_load_failed(
                path.display().to_string(),
                format!("Failed to read file: {e}"),
            )
        })?;
        
        // Validate component format
        self.validate(&bytes)?;
        
        Ok(bytes)
    }
    
    /// Load component from byte array.
    ///
    /// Validates and prepares component bytes for instantiation.
    /// Useful for in-memory components or network-loaded components.
    ///
    /// # Parameters
    ///
    /// - `bytes`: Component binary data
    ///
    /// # Returns
    ///
    /// - `Ok(Vec<u8>)`: Validated component bytes (may be same as input)
    /// - `Err(WasmError)`: Component validation failed
    ///
    /// # Errors
    ///
    /// - `WasmError::ComponentParseFailed`: Invalid WASM format
    /// - `WasmError::ComponentValidationFailed`: Semantic validation failed
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let bytes = include_bytes!("component.wasm");
    /// let validated = loader.load_from_bytes(bytes).await?;
    /// ```
    pub async fn load_from_bytes(&self, bytes: &[u8]) -> WasmResult<Vec<u8>> {
        // Validate component format before returning
        self.validate(bytes)?;
        Ok(bytes.to_vec())
    }
    
    /// Validate component format and semantics.
    ///
    /// Performs comprehensive validation including:
    /// - WASM magic number and version check
    /// - Component Model format validation
    /// - Semantic validation (imports, exports, types)
    ///
    /// # Parameters
    ///
    /// - `bytes`: Component binary data
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Component is valid
    /// - `Err(WasmError)`: Component validation failed
    ///
    /// # Errors
    ///
    /// - `WasmError::ComponentParseFailed`: Invalid WASM format
    /// - `WasmError::ComponentValidationFailed`: Semantic validation failed
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let bytes = std::fs::read("component.wasm")?;
    /// loader.validate(&bytes)?;
    /// println!("Component is valid");
    /// ```
    pub fn validate(&self, bytes: &[u8]) -> WasmResult<()> {
        // Basic magic number check (WASM binary format)
        if bytes.len() < 4 {
            return Err(WasmError::component_parse_failed(
                "Component too small (< 4 bytes)",
            ));
        }
        
        // Check WASM magic number: 0x00 0x61 0x73 0x6D ("\0asm")
        if &bytes[0..4] != b"\0asm" {
            return Err(WasmError::component_parse_failed(
                "Invalid WASM magic number (expected \\0asm)",
            ));
        }
        
        // Validate with Wasmtime Component Model parser
        Component::from_binary(self.engine.engine(), bytes).map_err(|e| {
            WasmError::component_validation_failed(format!(
                "Component validation failed: {e}"
            ))
        })?;
        
        Ok(())
    }
}

impl std::fmt::Debug for ComponentLoader<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComponentLoader")
            .field("engine", &"WasmEngine")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    #![allow(clippy::expect_used)]
    
    use super::*;
    
    fn create_test_engine() -> WasmEngine {
        WasmEngine::new().unwrap()
    }
    
    #[test]
    fn test_loader_creation() {
        let engine = create_test_engine();
        let loader = ComponentLoader::new(&engine);
        assert!(format!("{loader:?}").contains("ComponentLoader"));
    }
    
    #[test]
    fn test_validate_empty_bytes() {
        let engine = create_test_engine();
        let loader = ComponentLoader::new(&engine);
        
        let result = loader.validate(&[]);
        assert!(result.is_err());
        
        if let Err(e) = result {
            assert!(e.to_string().contains("too small"));
        }
    }
    
    #[test]
    fn test_validate_invalid_magic() {
        let engine = create_test_engine();
        let loader = ComponentLoader::new(&engine);
        
        let invalid_bytes = b"INVALID_WASM_FORMAT";
        let result = loader.validate(invalid_bytes);
        assert!(result.is_err());
        
        if let Err(e) = result {
            assert!(e.to_string().contains("magic number"));
        }
    }
    
    #[test]
    fn test_validate_valid_magic() {
        let engine = create_test_engine();
        let loader = ComponentLoader::new(&engine);
        
        // Minimal valid WASM header: magic (4 bytes) + version (4 bytes)
        let valid_header = b"\0asm\x01\x00\x00\x00";
        let result = loader.validate(valid_header);
        
        // This is not a valid Component Model component, so validation should fail
        // (magic number check passes, but Component Model validation fails)
        assert!(result.is_err());
        
        if let Err(e) = result {
            assert!(e.to_string().contains("validation failed"));
        }
    }
    
    #[tokio::test]
    async fn test_load_from_bytes_validation() {
        let engine = create_test_engine();
        let loader = ComponentLoader::new(&engine);
        
        // Test with invalid bytes
        let invalid_bytes = b"INVALID";
        let result = loader.load_from_bytes(invalid_bytes).await;
        assert!(result.is_err());
    }
}
