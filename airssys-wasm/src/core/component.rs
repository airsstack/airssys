//! Component abstractions for airssys-wasm framework.
//!
//! This module defines the core types and traits for WASM components:
//!
//! - **Component types**: ComponentId, ComponentMetadata, ResourceLimits
//! - **Input/Output**: ComponentInput, ComponentOutput (multicodec-encoded)
//! - **Configuration**: ComponentConfig, InstallationSource, ComponentState
//! - **Component trait**: Core behavior contract for all components
//!
//! # Design Principles
//!
//! - **Type Safety**: ComponentId uses newtype pattern to prevent string confusion
//! - **Mandatory Limits**: ResourceLimits enforces ADR-WASM-002 requirements
//! - **Multicodec**: Input/Output support multiple encodings (ADR-WASM-001)
//! - **Minimalism**: Component trait has only essential methods (init, execute, shutdown, metadata)
//!
//! # References
//!
//! - **ADR-WASM-001**: Multicodec Compatibility Strategy
//! - **ADR-WASM-002**: WASM Runtime Engine Selection
//! - **ADR-WASM-003**: Component Lifecycle Management

// Layer 1: Standard library imports
use std::collections::HashMap;
use std::path::PathBuf;

// Layer 2: Third-party crate imports
use serde::{Deserialize, Serialize};

// Layer 3: Internal module imports
use crate::core::capability::Capability;

// TODO(PHASE-4): Replace with comprehensive WasmError from core/error.rs
// Temporary placeholder for Component trait signatures
pub type WasmError = String;

/// Unique identifier for a component instance.
///
/// Uses newtype pattern to prevent accidental string misuse and provide
/// type safety at compile time.
///
/// # Examples
///
/// ```
/// use airssys_wasm::core::component::ComponentId;
///
/// let id = ComponentId::new("image-processor-v1");
/// assert_eq!(id.as_str(), "image-processor-v1");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComponentId(String);

impl ComponentId {
    /// Create a new component ID from a string.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the ID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Resource limits for component execution.
///
/// All limits are **mandatory** per ADR-WASM-002 to prevent resource exhaustion.
/// Components exceeding these limits will be terminated.
///
/// # Examples
///
/// ```
/// use airssys_wasm::core::component::ResourceLimits;
///
/// let limits = ResourceLimits {
///     max_memory_bytes: 64 * 1024 * 1024,  // 64MB
///     max_fuel: 1_000_000,                  // 1M fuel units
///     max_execution_ms: 5000,               // 5 seconds
///     max_storage_bytes: 10 * 1024 * 1024, // 10MB
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory in bytes (REQUIRED from Component.toml)
    pub max_memory_bytes: u64,

    /// Maximum fuel per execution (CPU limiting)
    pub max_fuel: u64,

    /// Maximum execution time in milliseconds (wall-clock timeout)
    pub max_execution_ms: u64,

    /// Maximum storage quota in bytes
    pub max_storage_bytes: u64,
}

/// Component metadata describing the component's identity and requirements.
///
/// # Examples
///
/// ```
/// use airssys_wasm::core::component::{ComponentMetadata, ResourceLimits};
///
/// let metadata = ComponentMetadata {
///     name: "image-processor".to_string(),
///     version: "1.0.0".to_string(),
///     author: "Acme Corp".to_string(),
///     description: Some("Processes images with filters".to_string()),
///     required_capabilities: vec![],
///     resource_limits: ResourceLimits {
///         max_memory_bytes: 64 * 1024 * 1024,
///         max_fuel: 1_000_000,
///         max_execution_ms: 5000,
///         max_storage_bytes: 10 * 1024 * 1024,
///     },
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMetadata {
    /// Component name (e.g., "image-processor")
    pub name: String,

    /// Semantic version (e.g., "1.2.3")
    pub version: String,

    /// Component author/publisher
    pub author: String,

    /// Optional description
    pub description: Option<String>,

    /// Required capabilities
    pub required_capabilities: Vec<Capability>,

    /// Resource limits from Component.toml
    pub resource_limits: ResourceLimits,
}

/// Component input for execution.
///
/// Input data is multicodec-encoded to support multiple data formats
/// (JSON, CBOR, MessagePack, Protobuf, etc.) per ADR-WASM-001.
///
/// # Multicodec Format
///
/// The `codec` field contains the multicodec prefix identifying the encoding:
/// - `0x0200` - JSON
/// - `0x51` - CBOR
/// - `0x0201` - MessagePack
///
/// # Examples
///
/// ```
/// use airssys_wasm::core::component::ComponentInput;
/// use std::collections::HashMap;
///
/// let input = ComponentInput {
///     data: vec![/* JSON bytes */],
///     codec: 0x0200,  // JSON multicodec
///     metadata: HashMap::new(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentInput {
    /// Input data (multicodec-encoded)
    pub data: Vec<u8>,

    /// Multicodec prefix identifying format
    pub codec: u64,

    /// Optional metadata
    pub metadata: HashMap<String, String>,
}

/// Component output from execution.
///
/// Output data is multicodec-encoded to support multiple data formats
/// (JSON, CBOR, MessagePack, Protobuf, etc.) per ADR-WASM-001.
///
/// # Examples
///
/// ```
/// use airssys_wasm::core::component::ComponentOutput;
/// use std::collections::HashMap;
///
/// let output = ComponentOutput {
///     data: vec![/* JSON bytes */],
///     codec: 0x0200,  // JSON multicodec
///     metadata: HashMap::new(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentOutput {
    /// Output data (multicodec-encoded)
    pub data: Vec<u8>,

    /// Multicodec prefix identifying format
    pub codec: u64,

    /// Optional metadata
    pub metadata: HashMap<String, String>,
}

/// Installation source for components.
///
/// Supports three installation methods per ADR-WASM-003:
/// 1. Git repository (reproducible builds from source)
/// 2. Local file path (fast development iteration)
/// 3. Remote URL (pre-built, offline-capable)
///
/// # Examples
///
/// ```
/// use airssys_wasm::core::component::InstallationSource;
///
/// // Install from Git (recommended for production)
/// let git_source = InstallationSource::Git {
///     url: "https://github.com/user/component.git".to_string(),
///     commit: "abc123".to_string(),
/// };
///
/// // Install from local file (development)
/// let file_source = InstallationSource::File {
///     path: "/path/to/component.wasm".into(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstallationSource {
    /// Git repository (reproducible builds)
    Git {
        /// Repository URL
        url: String,
        /// Commit hash
        commit: String,
    },
    /// Local file path (fast development)
    File {
        /// Path to WASM file
        path: PathBuf,
    },
    /// Remote URL (pre-built, offline-capable)
    Url {
        /// URL to WASM file
        url: String,
    },
}

/// Component lifecycle state.
///
/// Simple 2-state model per ADR-WASM-003:
/// - **Installed**: Component is available for execution
/// - **Uninstalled**: Component is removed or awaiting cleanup
///
/// # Examples
///
/// ```
/// use airssys_wasm::core::component::ComponentState;
///
/// let state = ComponentState::Installed;
/// assert_eq!(state, ComponentState::Installed);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComponentState {
    /// Component is installed and can be executed
    Installed,
    /// Component is uninstalled (awaiting cleanup or already removed)
    Uninstalled,
}

/// Component configuration.
///
/// # Examples
///
/// ```
/// use airssys_wasm::core::component::{ComponentConfig, ComponentId, ComponentMetadata, InstallationSource, ComponentState, ResourceLimits};
///
/// let config = ComponentConfig {
///     id: ComponentId::new("my-component"),
///     metadata: ComponentMetadata {
///         name: "my-component".to_string(),
///         version: "1.0.0".to_string(),
///         author: "Me".to_string(),
///         description: None,
///         required_capabilities: vec![],
///         resource_limits: ResourceLimits {
///             max_memory_bytes: 1024 * 1024,
///             max_fuel: 1000,
///             max_execution_ms: 5000,
///             max_storage_bytes: 512,
///         },
///     },
///     source: InstallationSource::File { path: "/tmp/component.wasm".into() },
///     state: ComponentState::Installed,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentConfig {
    /// Component ID
    pub id: ComponentId,

    /// Component metadata
    pub metadata: ComponentMetadata,

    /// Installation source (Git, File, URL)
    pub source: InstallationSource,

    /// Component state (Installed, Uninstalled)
    pub state: ComponentState,
}

/// Core component trait defining component behavior.
///
/// All WASM components implement this trait through generated WIT bindings.
/// The host runtime calls these methods during component lifecycle:
///
/// 1. **init()** - Called once when component is first loaded
/// 2. **execute()** - Main entry point for component logic
/// 3. **shutdown()** - Called when component is being unloaded
/// 4. **metadata()** - Provides component metadata to runtime
///
/// # Lifecycle
///
/// ```text
/// [Load] → init() → [Ready] → execute()* → shutdown() → [Unloaded]
///                             ↑__________|
///                           (multiple executions)
/// ```
///
/// # Examples
///
/// ```rust
/// use airssys_wasm::core::component::{Component, ComponentConfig, ComponentInput, ComponentOutput, ComponentMetadata, ResourceLimits};
///
/// struct MyComponent {
///     metadata: ComponentMetadata,
/// }
///
/// impl Component for MyComponent {
///     fn init(&mut self, _config: ComponentConfig) -> Result<(), String> {
///         // Initialize component state
///         Ok(())
///     }
///
///     fn execute(&self, _input: ComponentInput) -> Result<ComponentOutput, String> {
///         // Process input and produce output
///         Ok(ComponentOutput {
///             data: vec![],
///             codec: 0x0200,
///             metadata: Default::default(),
///         })
///     }
///
///     fn shutdown(&mut self) -> Result<(), String> {
///         // Clean up resources
///         Ok(())
///     }
///
///     fn metadata(&self) -> &ComponentMetadata {
///         &self.metadata
///     }
/// }
/// ```
pub trait Component {
    /// Initialize component with configuration.
    ///
    /// Called once when component is first loaded into the runtime.
    /// Use this for one-time setup, resource allocation, and state initialization.
    ///
    /// # Parameters
    /// - `config` - Component configuration including metadata and resource limits
    ///
    /// # Errors
    /// Returns error if initialization fails (e.g., invalid config, resource allocation failure)
    fn init(&mut self, config: ComponentConfig) -> Result<(), WasmError>;

    /// Execute component with input, producing output.
    ///
    /// This is the main entry point for component logic. Called multiple times
    /// during component lifetime. Must be idempotent and stateless where possible.
    ///
    /// # Parameters
    /// - `input` - Component input with multicodec-encoded data
    ///
    /// # Returns
    /// Component output with multicodec-encoded result data
    ///
    /// # Errors
    /// Returns error if execution fails (e.g., invalid input, resource limit exceeded)
    fn execute(&self, input: ComponentInput) -> Result<ComponentOutput, WasmError>;

    /// Shutdown component gracefully.
    ///
    /// Called when component is being unloaded or system is shutting down.
    /// Use this for cleanup, resource deallocation, and state persistence.
    ///
    /// # Errors
    /// Returns error if shutdown fails (non-fatal, logged by runtime)
    fn shutdown(&mut self) -> Result<(), WasmError>;

    /// Get component metadata.
    ///
    /// Called by runtime to retrieve component information for monitoring,
    /// logging, and capability verification.
    fn metadata(&self) -> &ComponentMetadata;
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // ComponentId Tests
    // ============================================================================

    #[test]
    fn test_component_id_creation() {
        let id = ComponentId::new("test-component");
        assert_eq!(id.as_str(), "test-component");
    }

    #[test]
    fn test_component_id_equality() {
        let id1 = ComponentId::new("comp1");
        let id2 = ComponentId::new("comp1");
        let id3 = ComponentId::new("comp2");

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_component_id_hash() {
        let mut map = HashMap::new();
        let id = ComponentId::new("test");
        map.insert(id.clone(), "value");

        assert_eq!(map.get(&id), Some(&"value"));
    }

    // ============================================================================
    // ResourceLimits Tests
    // ============================================================================

    #[test]
    fn test_resource_limits_creation() {
        let limits = ResourceLimits {
            max_memory_bytes: 1024,
            max_fuel: 1000,
            max_execution_ms: 5000,
            max_storage_bytes: 512,
        };

        assert_eq!(limits.max_memory_bytes, 1024);
        assert_eq!(limits.max_fuel, 1000);
        assert_eq!(limits.max_execution_ms, 5000);
        assert_eq!(limits.max_storage_bytes, 512);
    }

    #[test]
    fn test_resource_limits_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let limits = ResourceLimits {
            max_memory_bytes: 1024,
            max_fuel: 1000,
            max_execution_ms: 5000,
            max_storage_bytes: 512,
        };

        let json = serde_json::to_string(&limits)?;
        let deserialized: ResourceLimits = serde_json::from_str(&json)?;

        assert_eq!(limits.max_memory_bytes, deserialized.max_memory_bytes);
        assert_eq!(limits.max_fuel, deserialized.max_fuel);
        Ok(())
    }

    // ============================================================================
    // ComponentInput/Output Tests
    // ============================================================================

    #[test]
    fn test_component_input_multicodec() {
        let input = ComponentInput {
            data: vec![1, 2, 3],
            codec: 0x0200, // JSON
            metadata: HashMap::new(),
        };

        assert_eq!(input.codec, 0x0200);
        assert_eq!(input.data, vec![1, 2, 3]);
    }

    #[test]
    fn test_component_input_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let mut metadata = HashMap::new();
        metadata.insert("key".to_string(), "value".to_string());

        let input = ComponentInput {
            data: vec![1, 2, 3],
            codec: 0x0200,
            metadata,
        };

        let json = serde_json::to_string(&input)?;
        let deserialized: ComponentInput = serde_json::from_str(&json)?;

        assert_eq!(input.codec, deserialized.codec);
        assert_eq!(input.metadata.get("key"), Some(&"value".to_string()));
        Ok(())
    }

    #[test]
    fn test_component_output_creation() {
        let output = ComponentOutput {
            data: vec![4, 5, 6],
            codec: 0x51, // CBOR
            metadata: HashMap::new(),
        };

        assert_eq!(output.codec, 0x51);
        assert_eq!(output.data, vec![4, 5, 6]);
    }

    // ============================================================================
    // InstallationSource Tests
    // ============================================================================

    #[test]
    fn test_installation_source_git() {
        let source = InstallationSource::Git {
            url: "https://github.com/user/repo.git".to_string(),
            commit: "abc123".to_string(),
        };

        match source {
            InstallationSource::Git { url, commit } => {
                assert_eq!(url, "https://github.com/user/repo.git");
                assert_eq!(commit, "abc123");
            }
            _ => unreachable!("Expected Git variant"),
        }
    }

    #[test]
    fn test_installation_source_file() {
        let source = InstallationSource::File {
            path: PathBuf::from("/tmp/component.wasm"),
        };

        match source {
            InstallationSource::File { path } => {
                assert_eq!(path, PathBuf::from("/tmp/component.wasm"));
            }
            _ => unreachable!("Expected File variant"),
        }
    }

    #[test]
    fn test_installation_source_url() {
        let source = InstallationSource::Url {
            url: "https://example.com/component.wasm".to_string(),
        };

        match source {
            InstallationSource::Url { url } => {
                assert_eq!(url, "https://example.com/component.wasm");
            }
            _ => unreachable!("Expected Url variant"),
        }
    }

    #[test]
    fn test_installation_source_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let source = InstallationSource::File {
            path: PathBuf::from("/tmp/component.wasm"),
        };

        let json = serde_json::to_string(&source)?;
        let deserialized: InstallationSource = serde_json::from_str(&json)?;

        match deserialized {
            InstallationSource::File { path } => {
                assert_eq!(path, PathBuf::from("/tmp/component.wasm"));
            }
            _ => unreachable!("Expected File variant"),
        }
        Ok(())
    }

    // ============================================================================
    // ComponentState Tests
    // ============================================================================

    #[test]
    fn test_component_state_equality() {
        assert_eq!(ComponentState::Installed, ComponentState::Installed);
        assert_ne!(ComponentState::Installed, ComponentState::Uninstalled);
    }

    #[test]
    fn test_component_state_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let state = ComponentState::Installed;
        let json = serde_json::to_string(&state)?;
        let deserialized: ComponentState = serde_json::from_str(&json)?;

        assert_eq!(state, deserialized);
        Ok(())
    }

    // ============================================================================
    // ComponentMetadata Tests
    // ============================================================================

    #[test]
    fn test_component_metadata_creation() {
        let metadata = ComponentMetadata {
            name: "test-component".to_string(),
            version: "1.0.0".to_string(),
            author: "Test Author".to_string(),
            description: Some("A test component".to_string()),
            required_capabilities: vec![],
            resource_limits: ResourceLimits {
                max_memory_bytes: 1024,
                max_fuel: 1000,
                max_execution_ms: 5000,
                max_storage_bytes: 512,
            },
        };

        assert_eq!(metadata.name, "test-component");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.description, Some("A test component".to_string()));
    }

    // ============================================================================
    // ComponentConfig Tests
    // ============================================================================

    #[test]
    fn test_component_config_creation() {
        let config = ComponentConfig {
            id: ComponentId::new("my-component"),
            metadata: ComponentMetadata {
                name: "my-component".to_string(),
                version: "1.0.0".to_string(),
                author: "Me".to_string(),
                description: None,
                required_capabilities: vec![],
                resource_limits: ResourceLimits {
                    max_memory_bytes: 1024,
                    max_fuel: 1000,
                    max_execution_ms: 5000,
                    max_storage_bytes: 512,
                },
            },
            source: InstallationSource::File {
                path: PathBuf::from("/tmp/component.wasm"),
            },
            state: ComponentState::Installed,
        };

        assert_eq!(config.id.as_str(), "my-component");
        assert_eq!(config.state, ComponentState::Installed);
    }

    // ============================================================================
    // Component Trait Tests (using a mock implementation)
    // ============================================================================

    struct MockComponent {
        metadata: ComponentMetadata,
    }

    impl Component for MockComponent {
        fn init(&mut self, _config: ComponentConfig) -> Result<(), WasmError> {
            Ok(())
        }

        fn execute(&self, _input: ComponentInput) -> Result<ComponentOutput, WasmError> {
            Ok(ComponentOutput {
                data: vec![],
                codec: 0x0200,
                metadata: HashMap::new(),
            })
        }

        fn shutdown(&mut self) -> Result<(), WasmError> {
            Ok(())
        }

        fn metadata(&self) -> &ComponentMetadata {
            &self.metadata
        }
    }

    #[test]
    fn test_component_trait_implementation() {
        let mut component = MockComponent {
            metadata: ComponentMetadata {
                name: "mock".to_string(),
                version: "1.0.0".to_string(),
                author: "Test".to_string(),
                description: None,
                required_capabilities: vec![],
                resource_limits: ResourceLimits {
                    max_memory_bytes: 1024,
                    max_fuel: 1000,
                    max_execution_ms: 5000,
                    max_storage_bytes: 512,
                },
            },
        };

        // Test init
        let config = ComponentConfig {
            id: ComponentId::new("mock"),
            metadata: component.metadata.clone(),
            source: InstallationSource::File {
                path: PathBuf::from("/tmp/mock.wasm"),
            },
            state: ComponentState::Installed,
        };
        assert!(component.init(config).is_ok());

        // Test execute
        let input = ComponentInput {
            data: vec![1, 2, 3],
            codec: 0x0200,
            metadata: HashMap::new(),
        };
        assert!(component.execute(input).is_ok());

        // Test shutdown
        assert!(component.shutdown().is_ok());

        // Test metadata
        assert_eq!(component.metadata().name, "mock");
    }
}
