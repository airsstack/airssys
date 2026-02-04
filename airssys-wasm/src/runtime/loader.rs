//! Component loading implementations.
//!
//! This module provides concrete implementations of the [`ComponentLoader`] trait
//! for loading WASM component binaries from various sources:
//!
//! - [`FileComponentLoader`] - Loads components from filesystem
//! - [`InMemoryComponentLoader`] - In-memory loader for testing (cfg(test))
//!
//! # Architecture
//!
//! These loaders implement the [`ComponentLoader`] trait defined in
//! `core/runtime/traits.rs`. They follow the dependency inversion principle:
//!
//! - Trait is defined in `core/` (Layer 1)
//! - Implementations are in `runtime/` (Layer 2B)
//! - Used by higher-level components through dependency injection
//!
//! # WASM Validation
//!
//! All loaders perform basic WASM binary validation by checking the magic
//! number (0x00 0x61 0x73 0x6D, i.e., `\0asm`). This ensures loaded files
//! are valid WASM binaries before attempting execution.

// Layer 1: Standard library imports (per PROJECTS_STANDARD.md §2.1)
// None needed for production code

// Layer 2: Third-party crate imports (per PROJECTS_STANDARD.md §2.1)
// None needed

// Layer 3: Internal module imports (per PROJECTS_STANDARD.md §2.1)
use crate::core::component::id::ComponentId;
use crate::core::runtime::errors::WasmError;
use crate::core::runtime::traits::ComponentLoader;

/// File-based component loader.
///
/// `FileComponentLoader` loads WASM component binaries from filesystem,
/// organizing them in a hierarchical namespace structure:
///
/// ```text
/// {base_path}/{namespace}/{name}/{instance}.wasm
/// ```
///
/// # Examples
///
/// ```rust,no_run
/// use airssys_wasm::runtime::loader::FileComponentLoader;
/// use airssys_wasm::core::component::id::ComponentId;
/// use airssys_wasm::core::runtime::traits::ComponentLoader;
///
/// let loader = FileComponentLoader::new("/wasm/components");
/// let id = ComponentId::new("system", "database", "prod");
///
/// // Loads: /wasm/components/system/database/prod.wasm
/// let bytes = loader.load_bytes(&id)?;
/// # Ok::<(), airssys_wasm::core::runtime::errors::WasmError>(())
/// ```
pub struct FileComponentLoader {
    base_path: String,
}

impl FileComponentLoader {
    /// Creates a new file-based component loader.
    ///
    /// # Arguments
    ///
    /// * `base_path` - Base directory path for WASM components
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::runtime::loader::FileComponentLoader;
    ///
    /// let loader = FileComponentLoader::new("/wasm/components");
    /// ```
    pub fn new(base_path: impl Into<String>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }

    /// Constructs the filesystem path for a component.
    ///
    /// Components are stored in a hierarchical structure:
    /// `{base_path}/{namespace}/{name}/{instance}.wasm`
    ///
    /// # Arguments
    ///
    /// * `id` - Component identifier
    ///
    /// # Returns
    ///
    /// Full filesystem path to the component WASM file.
    fn component_path(&self, id: &ComponentId) -> String {
        format!(
            "{}/{}/{}/{}.wasm",
            self.base_path, id.namespace, id.name, id.instance
        )
    }
}

impl ComponentLoader for FileComponentLoader {
    /// Loads component bytes from filesystem.
    ///
    /// This method reads WASM binary file from filesystem path
    /// constructed from the component's namespace, name, and instance.
    ///
    /// # Errors
    ///
    /// - `WasmError::ComponentNotFound` - File cannot be read (not found, permission denied, etc.)
    fn load_bytes(&self, id: &ComponentId) -> Result<Vec<u8>, WasmError> {
        let path = self.component_path(id);

        std::fs::read(&path)
            .map_err(|e| WasmError::ComponentNotFound(format!("Failed to load {}: {}", path, e)))
    }

    /// Validates WASM binary bytes.
    ///
    /// Performs basic validation by checking the WASM magic number:
    /// - Magic: 0x00 0x61 0x73 0x6D (i.e., `\0asm`)
    /// - Minimum size: 4 bytes
    ///
    /// # Errors
    ///
    /// - `WasmError::InvalidComponent` - Bytes too small or invalid magic number
    fn validate(&self, bytes: &[u8]) -> Result<(), WasmError> {
        // Check minimum size (need at least 4 bytes for magic number)
        if bytes.len() < 4 {
            return Err(WasmError::InvalidComponent("File too small".to_string()));
        }

        // Validate WASM magic number: 0x00 0x61 0x73 0x6D (\0asm)
        if &bytes[0..4] != b"\0asm" {
            return Err(WasmError::InvalidComponent(
                "Invalid WASM magic number".to_string(),
            ));
        }

        Ok(())
    }
}

/// In-memory component loader for testing.
///
/// `InMemoryComponentLoader` stores WASM component binaries in memory,
/// avoiding filesystem access during tests. Components are stored in a
/// HashMap keyed by their string-formatted component ID.
///
/// # Availability
///
/// This type is only available when the `test` configuration is enabled:
///
/// ```text
/// #[cfg(test)]
/// ```
///
/// # Examples
///
/// ```rust,ignore
/// use airssys_wasm::runtime::loader::InMemoryComponentLoader;
/// use airssys_wasm::core::component::id::ComponentId;
/// use airssys_wasm::core::runtime::traits::ComponentLoader;
///
/// let mut loader = InMemoryComponentLoader::new();
/// let id = ComponentId::new("test", "component", "1");
/// let wasm_bytes = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
///
/// loader.add_component(&id, wasm_bytes.clone());
///
/// let loaded = loader.load_bytes(&id)?;
/// assert_eq!(loaded, wasm_bytes);
/// # Ok::<(), airssys_wasm::core::runtime::errors::WasmError>(())
/// ```
#[cfg(test)]
pub struct InMemoryComponentLoader {
    components: std::collections::HashMap<String, Vec<u8>>,
}

#[cfg(test)]
impl InMemoryComponentLoader {
    /// Creates a new empty in-memory component loader.
    pub fn new() -> Self {
        Self {
            components: std::collections::HashMap::new(),
        }
    }

    /// Adds a component to the in-memory store.
    ///
    /// # Arguments
    ///
    /// * `id` - Component identifier
    /// * `bytes` - WASM binary bytes
    pub fn add_component(&mut self, id: &ComponentId, bytes: Vec<u8>) {
        self.components.insert(id.to_string_id(), bytes);
    }
}

#[cfg(test)]
impl Default for InMemoryComponentLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl ComponentLoader for InMemoryComponentLoader {
    /// Loads component bytes from in-memory store.
    ///
    /// # Errors
    ///
    /// - `WasmError::ComponentNotFound` - Component ID not in store
    fn load_bytes(&self, id: &ComponentId) -> Result<Vec<u8>, WasmError> {
        self.components
            .get(&id.to_string_id())
            .cloned()
            .ok_or_else(|| WasmError::ComponentNotFound(id.to_string_id()))
    }

    /// Validates WASM binary bytes.
    ///
    /// Performs the same validation as `FileComponentLoader`:
    /// - Magic: 0x00 0x61 0x73 0x6D (i.e., `\0asm`)
    /// - Minimum size: 4 bytes
    ///
    /// # Errors
    ///
    /// - `WasmError::InvalidComponent` - Bytes too small or invalid magic number
    fn validate(&self, bytes: &[u8]) -> Result<(), WasmError> {
        if bytes.len() < 4 || &bytes[0..4] != b"\0asm" {
            return Err(WasmError::InvalidComponent("Invalid WASM".to_string()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_loader_path_construction() {
        let loader = FileComponentLoader::new("/wasm");
        let id = ComponentId::new("ns", "comp", "0");
        let path = loader.component_path(&id);
        assert_eq!(path, "/wasm/ns/comp/0.wasm");
    }

    #[test]
    fn test_validate_valid_wasm_magic() {
        let loader = FileComponentLoader::new("/wasm");
        let bytes = b"\0asm\x01\x00\x00\x00";
        assert!(loader.validate(bytes).is_ok());
    }

    #[test]
    fn test_validate_invalid_magic() {
        let loader = FileComponentLoader::new("/wasm");
        let bytes = b"notw\x01\x00\x00\x00";
        assert!(loader.validate(bytes).is_err());
    }

    #[test]
    fn test_validate_too_small() {
        let loader = FileComponentLoader::new("/wasm");
        let bytes = b"\0as";
        assert!(loader.validate(bytes).is_err());
    }

    #[test]
    fn test_in_memory_loader() {
        let mut loader = InMemoryComponentLoader::new();
        let id = ComponentId::new("test", "comp", "0");
        let bytes = b"\0asm\x01\x00\x00\x00".to_vec();

        loader.add_component(&id, bytes.clone());

        let loaded = loader.load_bytes(&id).unwrap();
        assert_eq!(loaded, bytes);
    }

    #[test]
    fn test_in_memory_loader_not_found() {
        let loader = InMemoryComponentLoader::new();
        let id = ComponentId::new("test", "comp", "0");

        let result = loader.load_bytes(&id);
        assert!(matches!(result, Err(WasmError::ComponentNotFound(_))));
    }
}
