//! Configuration types for airssys-wasm runtime and components.
//!
//! This module defines configuration patterns for runtime execution, security
//! enforcement, storage backends, and Component.toml parsing. All configuration
//! types provide sensible defaults and support serialization to TOML/JSON for
//! file-based configuration.
//!
//! # Configuration Categories
//!
//! - **ComponentConfigToml**: Component.toml parsing with resource limit validation
//! - **RuntimeConfig**: WASM engine execution settings
//! - **SecurityConfig**: Capability enforcement and audit settings
//! - **StorageConfig**: Storage backend and quota settings
//!
//! # Component.toml Parsing
//!
//! Components must declare resource limits in `Component.toml` following ADR-WASM-002.
//! The `[resources.memory]` section is MANDATORY with limits between 512KB-4MB.
//!
//! ```
//! use airssys_wasm::core::config::ComponentConfigToml;
//!
//! let toml_content = r#"
//! [component]
//! name = "my-component"
//! version = "1.0.0"
//!
//! [resources.memory]
//! max_bytes = 1048576  # 1MB (MANDATORY: 512KB-4MB range)
//!
//! [resources.cpu]
//! max_fuel = 10000  # MANDATORY (1,000-100,000,000 range)
//! timeout_seconds = 60  # MANDATORY (1-300 seconds range)
//! "#;
//!
//! let config = ComponentConfigToml::from_str(toml_content).unwrap();
//! let limits = config.to_resource_limits().unwrap();
//! assert_eq!(limits.max_memory_bytes(), 1048576);
//! ```
//!
//! # Default Constants
//!
//! All default configuration values are exposed as public constants, allowing
//! users to reference standard values and build custom configurations:
//!
//! - **Runtime**: `DEFAULT_MAX_FUEL`, `DEFAULT_EXECUTION_TIMEOUT_MS`, etc.
//! - **Security**: `DEFAULT_SECURITY_MODE`, `DEFAULT_AUDIT_LOGGING`, etc.
//! - **Storage**: `DEFAULT_STORAGE_BACKEND`, `DEFAULT_STORAGE_PATH`, etc.
//!
//! # Examples
//!
//! ```
//! use airssys_wasm::core::config::{
//!     RuntimeConfig, SecurityConfig, StorageConfig,
//!     DEFAULT_MAX_FUEL, DEFAULT_SECURITY_MODE,
//! };
//!
//! // Use default configurations
//! let runtime_config = RuntimeConfig::default();
//! let security_config = SecurityConfig::default();
//! let storage_config = StorageConfig::default();
//!
//! // Customize using constants
//! let custom_runtime = RuntimeConfig {
//!     default_max_fuel: DEFAULT_MAX_FUEL * 2, // Double the default fuel
//!     default_execution_timeout_ms: 500,
//!     ..Default::default()
//! };
//! ```

// Layer 1: Standard library imports
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

// Layer 2: Third-party crate imports
use serde::{Deserialize, Serialize};
use thiserror::Error;

// Layer 3: Internal module imports
use crate::core::error::WasmError;
use crate::runtime::limits::{CpuConfig, MemoryConfig, ResourceConfig, ResourceLimits};

// ============================================================================
// Component.toml Parsing Errors
// ============================================================================

/// Errors that can occur during Component.toml parsing and validation.
///
/// These errors provide user-friendly messages with references to ADR-WASM-002
/// for context and guidance on correct configuration.
///
/// # Examples
///
/// ```
/// use airssys_wasm::core::config::ConfigError;
///
/// let error = ConfigError::MissingMemoryConfig;
/// println!("{}", error);
/// ```
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Missing [resources.memory] section in Component.toml.
    ///
    /// Memory limits are MANDATORY per ADR-WASM-002. Components must explicitly
    /// declare memory requirements.
    #[error(
        "Missing [resources.memory] section in Component.toml\n\
         \n\
         Memory limits are MANDATORY per ADR-WASM-002.\n\
         \n\
         Add the following section to your Component.toml:\n\
         \n\
         [resources.memory]\n\
         max_bytes = 1048576  # 1MB (range: 512KB-4MB)\n\
         \n\
         See ADR-WASM-002 Section 2.4.2 for rationale."
    )]
    MissingMemoryConfig,

    /// Missing [resources.cpu] section in Component.toml.
    ///
    /// CPU limits are MANDATORY per ADR-WASM-002. Components must explicitly
    /// declare CPU resource requirements (fuel and timeout).
    #[error(
        "Missing [resources.cpu] section in Component.toml\n\
         \n\
         CPU limits are MANDATORY per ADR-WASM-002.\n\
         \n\
         Add the following section to your Component.toml:\n\
         \n\
         [resources.cpu]\n\
         max_fuel = 1000000       # 1M fuel units (MANDATORY)\n\
         timeout_seconds = 30     # 30s timeout (MANDATORY)\n\
         \n\
         See ADR-WASM-002 Section 2.4.3 for rationale."
    )]
    MissingCpuConfig,

    /// Invalid configuration detected.
    ///
    /// This error indicates that a configuration value is missing or invalid,
    /// with a custom message describing the specific problem.
    #[error("Invalid configuration: {message}")]
    InvalidConfiguration {
        /// Descriptive message explaining what is invalid.
        message: String,
    },

    /// Invalid resource limits detected during validation.
    ///
    /// This error wraps WasmError from the runtime module,
    /// providing context about which specific limit validation failed.
    #[error("Invalid resource limits: {0}")]
    InvalidResourceLimits(#[from] WasmError),

    /// Failed to parse Component.toml file.
    ///
    /// The TOML file has syntax errors or invalid structure.
    #[error("Failed to parse Component.toml: {0}")]
    TomlParseError(#[from] toml::de::Error),

    /// Failed to read Component.toml file.
    ///
    /// File may not exist or lacks read permissions.
    #[error("Failed to read Component.toml: {0}")]
    IoError(#[from] std::io::Error),
}

// ============================================================================
// Component.toml Configuration Structures
// ============================================================================

/// Component metadata from [component] section.
///
/// Basic identification and versioning information for the component.
///
/// # Examples
///
/// ```toml
/// [component]
/// name = "my-component"
/// version = "0.1.0"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComponentMetadataToml {
    /// Component name.
    pub name: String,

    /// Component version (semver format recommended).
    pub version: String,
}

/// Memory configuration from [resources.memory] section.
///
/// **MANDATORY** per ADR-WASM-002. No defaults - explicit declaration required.
///
/// # Valid Range
///
/// - Minimum: 512KB (524,288 bytes)
/// - Maximum: 4MB (4,194,304 bytes)
///
/// # Examples
///
/// ```toml
/// [resources.memory]
/// max_bytes = 1048576  # 1MB
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryConfigToml {
    /// Maximum memory in bytes (MANDATORY).
    ///
    /// Range: 512KB-4MB (524,288-4,194,304 bytes).
    pub max_bytes: usize,
}

/// CPU configuration from [resources.cpu] section.
///
/// Contains fuel metering and execution timeout configuration.
/// Both fields are MANDATORY following ADR-WASM-002 explicit security philosophy.
///
/// # Examples
///
/// ```toml
/// [resources.cpu]
/// max_fuel = 1000000        # MANDATORY - 1M fuel units
/// timeout_seconds = 30      # MANDATORY - 30s timeout
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CpuConfigToml {
    /// Maximum fuel units (MANDATORY).
    ///
    /// Fuel represents deterministic CPU usage. Each WASM instruction
    /// consumes fuel units. When fuel is exhausted, execution stops.
    ///
    /// Range: 1,000 - 100,000,000 fuel units
    pub max_fuel: Option<u64>,

    /// Execution timeout in seconds (MANDATORY).
    ///
    /// Wall-clock timeout that stops execution even if fuel remains.
    /// Provides fail-safe protection against infinite loops or
    /// misconfigured fuel limits.
    ///
    /// Range: 1 - 300 seconds
    pub timeout_seconds: Option<u64>,
}

/// Resource configuration from [resources] section.
///
/// Contains memory configuration (MANDATORY) and CPU configuration (MANDATORY).
/// Following ADR-WASM-002, both memory and CPU limits must be explicitly declared.
///
/// # Examples
///
/// ```toml
/// [resources.memory]
/// max_bytes = 1048576
///
/// [resources.cpu]
/// max_fuel = 1000000
/// timeout_seconds = 30
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourcesConfigToml {
    /// Memory configuration (MANDATORY).
    pub memory: Option<MemoryConfigToml>,

    /// CPU configuration (MANDATORY).
    #[serde(default)]
    pub cpu: Option<CpuConfigToml>,
}

/// Top-level Component.toml configuration.
///
/// Represents the complete Component.toml file structure with component
/// metadata and resource configuration.
///
/// # Validation Rules (ADR-WASM-002)
///
/// - `[component]` section is required
/// - `[resources.memory]` section is MANDATORY
///   - Memory limits must be in range 512KB-4MB
/// - `[resources.cpu]` section is MANDATORY
///   - Fuel limit must be in range 1,000-100,000,000
///   - Timeout must be in range 1-300 seconds
///
/// # Examples
///
/// ```
/// use airssys_wasm::core::config::ComponentConfigToml;
///
/// let toml_content = r#"
/// [component]
/// name = "my-component"
/// version = "0.1.0"
///
/// [resources.memory]
/// max_bytes = 1048576
///
/// [resources.cpu]
/// max_fuel = 10000
/// timeout_seconds = 60
/// "#;
///
/// let config = ComponentConfigToml::from_str(toml_content).unwrap();
/// assert_eq!(config.component.name, "my-component");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComponentConfigToml {
    /// Component metadata.
    pub component: ComponentMetadataToml,

    /// Resource configuration (optional in TOML, but memory is validated as MANDATORY).
    #[serde(default)]
    pub resources: Option<ResourcesConfigToml>,
}

impl ComponentConfigToml {
    /// Parse Component.toml from file path.
    ///
    /// Reads and parses the TOML file, then validates resource limits.
    ///
    /// # Errors
    ///
    /// - `ConfigError::IoError` - File read failure
    /// - `ConfigError::TomlParseError` - TOML parsing failure
    /// - `ConfigError::MissingMemoryConfig` - Missing [resources.memory] section
    /// - `ConfigError::InvalidResourceLimits` - Memory limits out of range
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use airssys_wasm::core::config::ComponentConfigToml;
    ///
    /// let config = ComponentConfigToml::from_file("Component.toml").unwrap();
    /// println!("Component: {} v{}", config.component.name, config.component.version);
    /// ```
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)?;
        Self::from_str(&content)
    }

    /// Parse Component.toml from string.
    ///
    /// Parses TOML content and validates resource limits.
    /// This is a convenience method that delegates to the `FromStr` trait implementation.
    ///
    /// # Errors
    ///
    /// - `ConfigError::TomlParseError` - TOML parsing failure
    /// - `ConfigError::MissingMemoryConfig` - Missing [resources.memory] section
    /// - `ConfigError::InvalidResourceLimits` - Memory limits out of range
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::config::ComponentConfigToml;
    ///
    /// let toml_content = r#"
    /// [component]
    /// name = "test"
    /// version = "1.0.0"
    ///
    /// [resources.memory]
    /// max_bytes = 1048576
    ///
    /// [resources.cpu]
    /// max_fuel = 10000
    /// timeout_seconds = 60
    /// "#;
    ///
    /// let config = ComponentConfigToml::from_str(toml_content).unwrap();
    /// assert_eq!(config.component.name, "test");
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(content: &str) -> Result<Self, ConfigError> {
        content.parse()
    }

    /// Validate Component.toml configuration.
    ///
    /// Ensures [resources.memory] and [resources.cpu] sections exist and
    /// all limits are valid (ADR-WASM-002).
    ///
    /// # Errors
    ///
    /// - `ConfigError::MissingMemoryConfig` - Missing [resources.memory] section
    /// - `ConfigError::MissingCpuConfig` - Missing [resources.cpu] section
    /// - `ConfigError::InvalidResourceLimits` - Resource limits out of range
    fn validate(&self) -> Result<(), ConfigError> {
        let resources = self
            .resources
            .as_ref()
            .ok_or(ConfigError::MissingMemoryConfig)?;

        // Validate memory configuration (MANDATORY)
        let memory_config = resources
            .memory
            .as_ref()
            .ok_or(ConfigError::MissingMemoryConfig)?;

        let memory = MemoryConfig {
            max_memory_bytes: memory_config.max_bytes as u64,
        };

        memory.validate()?;

        // Validate CPU configuration (MANDATORY)
        let cpu_config = resources
            .cpu
            .as_ref()
            .ok_or(ConfigError::MissingCpuConfig)?;

        let max_fuel = cpu_config
            .max_fuel
            .ok_or_else(|| ConfigError::InvalidConfiguration {
                message: "max_fuel is MANDATORY and must be explicitly set (ADR-WASM-002)"
                    .to_string(),
            })?;

        let timeout_seconds = cpu_config
            .timeout_seconds
            .ok_or_else(|| ConfigError::InvalidConfiguration {
                message:
                    "timeout_seconds is MANDATORY and must be explicitly set (ADR-WASM-002)"
                        .to_string(),
            })?;

        let cpu = CpuConfig {
            max_fuel,
            timeout_seconds,
        };

        cpu.validate()?;

        Ok(())
    }

    /// Convert to ResourceLimits for runtime usage.
    ///
    /// Extracts resource limits from Component.toml configuration and converts
    /// to ResourceLimits structure used by the runtime.
    ///
    /// # Errors
    ///
    /// - `ConfigError::MissingMemoryConfig` - Missing [resources.memory] section
    /// - `ConfigError::InvalidResourceLimits` - Invalid resource configuration
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::config::ComponentConfigToml;
    ///
    /// let toml_content = r#"
    /// [component]
    /// name = "test"
    /// version = "1.0.0"
    ///
    /// [resources.memory]
    /// max_bytes = 1048576
    ///
    /// [resources.cpu]
    /// max_fuel = 10000
    /// timeout_seconds = 60
    /// "#;
    ///
    /// let config = ComponentConfigToml::from_str(toml_content).unwrap();
    /// let limits = config.to_resource_limits().unwrap();
    /// assert_eq!(limits.max_memory_bytes(), 1048576);
    /// ```
    pub fn to_resource_limits(&self) -> Result<ResourceLimits, ConfigError> {
        let resources = self
            .resources
            .as_ref()
            .ok_or(ConfigError::MissingMemoryConfig)?;

        // Extract and validate memory config
        let memory_config_toml = resources
            .memory
            .as_ref()
            .ok_or(ConfigError::MissingMemoryConfig)?;

        let memory = MemoryConfig {
            max_memory_bytes: memory_config_toml.max_bytes as u64,
        };

        // Extract and validate CPU config
        let cpu_config_toml = resources
            .cpu
            .as_ref()
            .ok_or(ConfigError::MissingCpuConfig)?;

        let max_fuel = cpu_config_toml.max_fuel.ok_or_else(|| {
            ConfigError::InvalidConfiguration {
                message: "max_fuel is MANDATORY and must be explicitly set (ADR-WASM-002)"
                    .to_string(),
            }
        })?;

        let timeout_seconds = cpu_config_toml.timeout_seconds.ok_or_else(|| {
            ConfigError::InvalidConfiguration {
                message: "timeout_seconds is MANDATORY and must be explicitly set (ADR-WASM-002)"
                    .to_string(),
            }
        })?;

        let cpu = CpuConfig {
            max_fuel,
            timeout_seconds,
        };

        // Build ResourceConfig and convert to ResourceLimits
        let resource_config = ResourceConfig { memory, cpu };
        let limits = ResourceLimits::try_from(resource_config)?;

        Ok(limits)
    }
}

impl FromStr for ComponentConfigToml {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let config: ComponentConfigToml = toml::from_str(s)?;
        config.validate()?;
        Ok(config)
    }
}

// ============================================================================
// RuntimeConfig Default Constants
// ============================================================================

/// Default value for async execution (enabled).
pub const DEFAULT_ASYNC_ENABLED: bool = true;

/// Default value for fuel metering (enabled for resource protection).
pub const DEFAULT_FUEL_METERING_ENABLED: bool = true;

/// Default maximum fuel limit (1 million fuel units).
///
/// Approximately 1-10ms of execution depending on operations performed.
pub const DEFAULT_MAX_FUEL: u64 = 1_000_000;

/// Default execution timeout in milliseconds (100ms).
///
/// Wall-clock timeout for component execution.
pub const DEFAULT_EXECUTION_TIMEOUT_MS: u64 = 100;

/// Default module caching setting (enabled for performance).
pub const DEFAULT_MODULE_CACHING_ENABLED: bool = true;

/// Default maximum number of cached modules (100).
///
/// LRU eviction when cache is full.
pub const DEFAULT_MAX_CACHED_MODULES: usize = 100;

// ============================================================================
// SecurityConfig Default Constants
// ============================================================================

/// Default security mode (Strict for production security).
pub const DEFAULT_SECURITY_MODE: SecurityMode = SecurityMode::Strict;

/// Default audit logging setting (enabled for compliance).
pub const DEFAULT_AUDIT_LOGGING: bool = true;

/// Default capability check timeout in microseconds (5μs target).
pub const DEFAULT_CAPABILITY_CHECK_TIMEOUT_US: u64 = 5;

// ============================================================================
// StorageConfig Default Constants
// ============================================================================

/// Default storage backend (Sled - pure Rust, no C dependencies).
pub const DEFAULT_STORAGE_BACKEND: StorageBackend = StorageBackend::Sled;

/// Default storage directory path.
pub const DEFAULT_STORAGE_PATH: &str = "./airssys_wasm_storage";

/// Default storage quotas setting (enabled for resource protection).
pub const DEFAULT_QUOTAS_ENABLED: bool = true;

// ============================================================================
// Configuration Types
// ============================================================================

/// Runtime configuration for WASM engine.
///
/// Controls execution behavior including async execution, fuel metering,
/// timeouts, and module caching. All settings have sensible defaults.
///
/// # Examples
///
/// ```
/// use airssys_wasm::core::config::RuntimeConfig;
///
/// // Use defaults
/// let config = RuntimeConfig::default();
/// assert_eq!(config.default_max_fuel, 1_000_000);
/// assert_eq!(config.default_execution_timeout_ms, 100);
///
/// // Customize specific settings
/// let custom = RuntimeConfig {
///     default_max_fuel: 5_000_000,
///     default_execution_timeout_ms: 1000,
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RuntimeConfig {
    /// Enable async execution (default: true).
    ///
    /// When enabled, component execution uses async/await patterns for
    /// concurrent execution and efficient resource utilization.
    pub async_enabled: bool,

    /// Enable fuel metering (default: true).
    ///
    /// Fuel metering tracks computational cost and prevents infinite loops
    /// or excessive CPU usage. Recommended for production.
    pub fuel_metering_enabled: bool,

    /// Default fuel limit (can be overridden per component).
    ///
    /// Default: 1,000,000 fuel units (approximately 1-10ms of execution
    /// depending on operations performed).
    pub default_max_fuel: u64,

    /// Default execution timeout in milliseconds.
    ///
    /// Default: 100ms. Wall-clock timeout for component execution.
    /// Components exceeding this timeout will be terminated.
    pub default_execution_timeout_ms: u64,

    /// Enable module caching for faster instantiation.
    ///
    /// When enabled, compiled WASM modules are cached in memory for
    /// faster instantiation on subsequent loads.
    pub module_caching_enabled: bool,

    /// Maximum cached modules (LRU eviction).
    ///
    /// When module cache is full, least recently used modules are evicted.
    /// Default: 100 modules.
    pub max_cached_modules: usize,
}

impl Default for RuntimeConfig {
    /// Create RuntimeConfig with production-ready defaults.
    ///
    /// Uses constants defined in this module for all default values.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::config::{RuntimeConfig, DEFAULT_MAX_FUEL, DEFAULT_EXECUTION_TIMEOUT_MS};
    ///
    /// let config = RuntimeConfig::default();
    /// assert!(config.async_enabled);
    /// assert!(config.fuel_metering_enabled);
    /// assert_eq!(config.default_max_fuel, DEFAULT_MAX_FUEL);
    /// assert_eq!(config.default_execution_timeout_ms, DEFAULT_EXECUTION_TIMEOUT_MS);
    /// assert!(config.module_caching_enabled);
    /// assert_eq!(config.max_cached_modules, 100);
    /// ```
    fn default() -> Self {
        Self {
            async_enabled: DEFAULT_ASYNC_ENABLED,
            fuel_metering_enabled: DEFAULT_FUEL_METERING_ENABLED,
            default_max_fuel: DEFAULT_MAX_FUEL,
            default_execution_timeout_ms: DEFAULT_EXECUTION_TIMEOUT_MS,
            module_caching_enabled: DEFAULT_MODULE_CACHING_ENABLED,
            max_cached_modules: DEFAULT_MAX_CACHED_MODULES,
        }
    }
}

/// Security configuration for capability enforcement.
///
/// Controls security mode, audit logging, and capability check performance.
/// Default configuration uses strict mode with audit logging enabled.
///
/// # Examples
///
/// ```
/// use airssys_wasm::core::config::{SecurityConfig, SecurityMode};
///
/// // Use strict defaults
/// let config = SecurityConfig::default();
/// assert_eq!(config.mode, SecurityMode::Strict);
/// assert!(config.audit_logging);
///
/// // Development mode (disable checks for testing)
/// let dev_config = SecurityConfig {
///     mode: SecurityMode::Development,
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SecurityConfig {
    /// Security mode (Strict, Permissive, Development).
    ///
    /// - **Strict**: All capabilities must be explicitly granted (production default)
    /// - **Permissive**: Some auto-approval for trusted sources
    /// - **Development**: Bypass capability checks (DEV/TEST ONLY)
    pub mode: SecurityMode,

    /// Audit logging enabled.
    ///
    /// When enabled, all capability checks and security events are logged
    /// for security auditing and compliance.
    pub audit_logging: bool,

    /// Capability check timeout (microseconds).
    ///
    /// Maximum time allowed for capability verification. Target: 5μs.
    pub capability_check_timeout_us: u64,
}

impl Default for SecurityConfig {
    /// Create SecurityConfig with secure defaults.
    ///
    /// Uses constants defined in this module for all default values.
    /// Uses Strict mode with audit logging enabled for production security.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::config::{SecurityConfig, SecurityMode, DEFAULT_SECURITY_MODE};
    ///
    /// let config = SecurityConfig::default();
    /// assert_eq!(config.mode, DEFAULT_SECURITY_MODE);
    /// assert!(config.audit_logging);
    /// assert_eq!(config.capability_check_timeout_us, 5);
    /// ```
    fn default() -> Self {
        Self {
            mode: DEFAULT_SECURITY_MODE,
            audit_logging: DEFAULT_AUDIT_LOGGING,
            capability_check_timeout_us: DEFAULT_CAPABILITY_CHECK_TIMEOUT_US,
        }
    }
}

/// Security enforcement mode.
///
/// Determines how strictly capability requirements are enforced.
/// Production systems should use Strict mode.
///
/// # Examples
///
/// ```
/// use airssys_wasm::core::config::SecurityMode;
///
/// // Production: strict enforcement
/// let prod_mode = SecurityMode::Strict;
///
/// // Development: bypass checks for testing
/// let dev_mode = SecurityMode::Development;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityMode {
    /// Strict mode: All capabilities must be explicitly granted.
    ///
    /// Components must declare all required capabilities in their manifest.
    /// Any capability not explicitly granted will be denied.
    /// **Recommended for production**.
    Strict,

    /// Permissive mode: Allows some auto-approval for trusted sources.
    ///
    /// Components from trusted sources may be auto-approved for common
    /// capabilities. Use with caution in production.
    Permissive,

    /// Development mode: Bypass capability checks.
    ///
    /// **⚠️ DEVELOPMENT/TESTING ONLY**. All capability checks are skipped.
    /// Never use in production.
    Development,
}

/// Storage configuration.
///
/// Controls storage backend selection, storage directory, and quota enforcement.
/// Default configuration uses Sled (pure Rust) backend with quotas enabled.
///
/// # Examples
///
/// ```
/// use airssys_wasm::core::config::{StorageConfig, StorageBackend};
/// use std::path::PathBuf;
///
/// // Use defaults (Sled backend)
/// let config = StorageConfig::default();
/// assert_eq!(config.backend, StorageBackend::Sled);
/// assert!(config.quotas_enabled);
///
/// // Use RocksDB for production
/// let prod_config = StorageConfig {
///     backend: StorageBackend::RocksDB,
///     storage_path: PathBuf::from("/var/lib/airssys_wasm"),
///     quotas_enabled: true,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StorageConfig {
    /// Storage backend (Sled, RocksDB).
    ///
    /// - **Sled**: Pure Rust, embedded database (default)
    /// - **RocksDB**: Production-proven, higher performance (optional)
    pub backend: StorageBackend,

    /// Storage directory path.
    ///
    /// Directory where component storage data is persisted.
    /// Default: `./airssys_wasm_storage`
    pub storage_path: PathBuf,

    /// Enable storage quotas.
    ///
    /// When enabled, each component has a storage quota limit enforced
    /// by the runtime. Recommended for multi-tenant deployments.
    pub quotas_enabled: bool,
}

impl Default for StorageConfig {
    /// Create StorageConfig with reasonable defaults.
    ///
    /// Uses constants defined in this module for all default values.
    /// Uses Sled backend (pure Rust) with local storage directory
    /// and quotas enabled.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::config::{StorageConfig, StorageBackend, DEFAULT_STORAGE_BACKEND};
    ///
    /// let config = StorageConfig::default();
    /// assert_eq!(config.backend, DEFAULT_STORAGE_BACKEND);
    /// assert_eq!(config.storage_path.to_str().unwrap(), "./airssys_wasm_storage");
    /// assert!(config.quotas_enabled);
    /// ```
    fn default() -> Self {
        Self {
            backend: DEFAULT_STORAGE_BACKEND,
            storage_path: PathBuf::from(DEFAULT_STORAGE_PATH),
            quotas_enabled: DEFAULT_QUOTAS_ENABLED,
        }
    }
}

/// Storage backend selection.
///
/// Determines which embedded database is used for component storage.
/// Both backends provide ACID guarantees and crash recovery.
///
/// # Examples
///
/// ```
/// use airssys_wasm::core::config::StorageBackend;
///
/// // Default: Sled (pure Rust, no C dependencies)
/// let default_backend = StorageBackend::Sled;
///
/// // Production: RocksDB (battle-tested, higher performance)
/// let prod_backend = StorageBackend::RocksDB;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageBackend {
    /// Sled (pure Rust, default).
    ///
    /// Pure Rust embedded database. No C dependencies, easier to build.
    /// Good performance for most use cases. **Default choice**.
    Sled,

    /// RocksDB (production-proven, optional).
    ///
    /// Production-proven embedded database from Facebook. Higher performance
    /// and maturity. Requires C++ dependencies. **Recommended for production**.
    RocksDB,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    #![allow(clippy::expect_used)]
    #![allow(clippy::panic)]

    use super::*;

    // ============================================================================
    // RuntimeConfig Tests
    // ============================================================================

    #[test]
    fn test_runtime_config_default() {
        let config = RuntimeConfig::default();
        assert!(config.async_enabled);
        assert!(config.fuel_metering_enabled);
        assert_eq!(config.default_max_fuel, 1_000_000);
        assert_eq!(config.default_execution_timeout_ms, 100);
        assert!(config.module_caching_enabled);
        assert_eq!(config.max_cached_modules, 100);
    }

    #[test]
    fn test_runtime_config_custom() {
        let config = RuntimeConfig {
            default_max_fuel: 5_000_000,
            default_execution_timeout_ms: 500,
            ..Default::default()
        };
        assert_eq!(config.default_max_fuel, 5_000_000);
        assert_eq!(config.default_execution_timeout_ms, 500);
        assert!(config.async_enabled); // From default
    }

    #[test]
    fn test_runtime_config_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let config = RuntimeConfig::default();

        // Serialize to JSON
        let json = serde_json::to_string(&config)?;
        assert!(json.contains("async_enabled"));

        // Deserialize from JSON
        let deserialized: RuntimeConfig = serde_json::from_str(&json)?;
        assert_eq!(config, deserialized);

        Ok(())
    }

    // ============================================================================
    // SecurityConfig Tests
    // ============================================================================

    #[test]
    fn test_security_config_default() {
        let config = SecurityConfig::default();
        assert_eq!(config.mode, SecurityMode::Strict);
        assert!(config.audit_logging);
        assert_eq!(config.capability_check_timeout_us, 5);
    }

    #[test]
    fn test_security_config_modes() {
        let strict = SecurityConfig {
            mode: SecurityMode::Strict,
            ..Default::default()
        };
        assert_eq!(strict.mode, SecurityMode::Strict);

        let permissive = SecurityConfig {
            mode: SecurityMode::Permissive,
            ..Default::default()
        };
        assert_eq!(permissive.mode, SecurityMode::Permissive);

        let development = SecurityConfig {
            mode: SecurityMode::Development,
            ..Default::default()
        };
        assert_eq!(development.mode, SecurityMode::Development);
    }

    #[test]
    fn test_security_mode_equality() {
        assert_eq!(SecurityMode::Strict, SecurityMode::Strict);
        assert_ne!(SecurityMode::Strict, SecurityMode::Permissive);
        assert_ne!(SecurityMode::Permissive, SecurityMode::Development);
    }

    #[test]
    fn test_security_config_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let config = SecurityConfig::default();

        // Serialize to JSON
        let json = serde_json::to_string(&config)?;
        assert!(json.contains("mode"));
        assert!(json.contains("Strict"));

        // Deserialize from JSON
        let deserialized: SecurityConfig = serde_json::from_str(&json)?;
        assert_eq!(config, deserialized);

        Ok(())
    }

    // ============================================================================
    // StorageConfig Tests
    // ============================================================================

    #[test]
    fn test_storage_config_default() {
        let config = StorageConfig::default();
        assert_eq!(config.backend, StorageBackend::Sled);
        assert_eq!(config.storage_path, PathBuf::from("./airssys_wasm_storage"));
        assert!(config.quotas_enabled);
    }

    #[test]
    fn test_storage_config_custom() {
        let config = StorageConfig {
            backend: StorageBackend::RocksDB,
            storage_path: PathBuf::from("/var/lib/airssys_wasm"),
            quotas_enabled: false,
        };
        assert_eq!(config.backend, StorageBackend::RocksDB);
        assert_eq!(config.storage_path, PathBuf::from("/var/lib/airssys_wasm"));
        assert!(!config.quotas_enabled);
    }

    #[test]
    fn test_storage_backend_equality() {
        assert_eq!(StorageBackend::Sled, StorageBackend::Sled);
        assert_ne!(StorageBackend::Sled, StorageBackend::RocksDB);
    }

    #[test]
    fn test_storage_config_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let config = StorageConfig::default();

        // Serialize to JSON
        let json = serde_json::to_string(&config)?;
        assert!(json.contains("backend"));
        assert!(json.contains("Sled"));

        // Deserialize from JSON
        let deserialized: StorageConfig = serde_json::from_str(&json)?;
        assert_eq!(config, deserialized);

        Ok(())
    }

    // ============================================================================
    // Integration Tests
    // ============================================================================

    #[test]
    fn test_constants_match_defaults() {
        // Verify RuntimeConfig constants match Default impl
        let runtime = RuntimeConfig::default();
        assert_eq!(runtime.async_enabled, DEFAULT_ASYNC_ENABLED);
        assert_eq!(runtime.fuel_metering_enabled, DEFAULT_FUEL_METERING_ENABLED);
        assert_eq!(runtime.default_max_fuel, DEFAULT_MAX_FUEL);
        assert_eq!(
            runtime.default_execution_timeout_ms,
            DEFAULT_EXECUTION_TIMEOUT_MS
        );
        assert_eq!(
            runtime.module_caching_enabled,
            DEFAULT_MODULE_CACHING_ENABLED
        );
        assert_eq!(runtime.max_cached_modules, DEFAULT_MAX_CACHED_MODULES);

        // Verify SecurityConfig constants match Default impl
        let security = SecurityConfig::default();
        assert_eq!(security.mode, DEFAULT_SECURITY_MODE);
        assert_eq!(security.audit_logging, DEFAULT_AUDIT_LOGGING);
        assert_eq!(
            security.capability_check_timeout_us,
            DEFAULT_CAPABILITY_CHECK_TIMEOUT_US
        );

        // Verify StorageConfig constants match Default impl
        let storage = StorageConfig::default();
        assert_eq!(storage.backend, DEFAULT_STORAGE_BACKEND);
        assert_eq!(storage.storage_path, PathBuf::from(DEFAULT_STORAGE_PATH));
        assert_eq!(storage.quotas_enabled, DEFAULT_QUOTAS_ENABLED);
    }

    #[test]
    fn test_all_configs_together() {
        let runtime = RuntimeConfig::default();
        let security = SecurityConfig::default();
        let storage = StorageConfig::default();

        // All defaults should be consistent
        assert!(runtime.async_enabled);
        assert_eq!(security.mode, SecurityMode::Strict);
        assert_eq!(storage.backend, StorageBackend::Sled);
    }

    #[test]
    fn test_config_cloning() {
        let config = RuntimeConfig::default();
        let cloned = config.clone();
        assert_eq!(config, cloned);

        let security = SecurityConfig::default();
        let cloned_security = security.clone();
        assert_eq!(security, cloned_security);
    }

    #[test]
    fn test_config_debug_format() {
        let config = RuntimeConfig::default();
        let debug_str = format!("{config:?}");
        assert!(debug_str.contains("RuntimeConfig"));
        assert!(debug_str.contains("async_enabled"));

        let security = SecurityConfig::default();
        let debug_str = format!("{security:?}");
        assert!(debug_str.contains("SecurityConfig"));
        assert!(debug_str.contains("Strict"));
    }

    // ============================================================================
    // Component.toml Parsing Tests
    // ============================================================================

    #[test]
    fn test_component_config_valid() {
        let toml_content = r#"
[component]
name = "test-component"
version = "1.0.0"

[resources.memory]
max_bytes = 1048576

[resources.cpu]
max_fuel = 10000
timeout_seconds = 30
        "#;

        let config = ComponentConfigToml::from_str(toml_content).unwrap();
        assert_eq!(config.component.name, "test-component");
        assert_eq!(config.component.version, "1.0.0");
        assert_eq!(
            config.resources.as_ref().unwrap().memory.as_ref().unwrap().max_bytes,
            1048576
        );
    }

    #[test]
    fn test_component_config_missing_memory_section() {
        let toml_content = r#"
[component]
name = "test-component"
version = "1.0.0"
        "#;

        let result = ComponentConfigToml::from_str(toml_content);
        assert!(result.is_err());
        let err = result.unwrap_err();
        eprintln!("Error: {err:?}");
        assert!(matches!(err, ConfigError::MissingMemoryConfig));
    }

    #[test]
    fn test_component_config_memory_below_minimum() {
        let toml_content = r#"
[component]
name = "test-component"
version = "1.0.0"

[resources.memory]
max_bytes = 262144
        "#;

        let result = ComponentConfigToml::from_str(toml_content);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConfigError::InvalidResourceLimits(_)
        ));
    }

    #[test]
    fn test_component_config_memory_above_maximum() {
        let toml_content = r#"
[component]
name = "test-component"
version = "1.0.0"

[resources.memory]
max_bytes = 5242880
        "#;

        let result = ComponentConfigToml::from_str(toml_content);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConfigError::InvalidResourceLimits(_)
        ));
    }

    #[test]
    fn test_component_config_minimum_valid_memory() {
        let toml_content = r#"
[component]
name = "test-component"
version = "1.0.0"

[resources.memory]
max_bytes = 524288

[resources.cpu]
max_fuel = 10000
timeout_seconds = 30
        "#;

        let config = ComponentConfigToml::from_str(toml_content).unwrap();
        assert_eq!(
            config.resources.as_ref().unwrap().memory.as_ref().unwrap().max_bytes,
            524288
        );
    }

    #[test]
    fn test_component_config_maximum_valid_memory() {
        let toml_content = r#"
[component]
name = "test-component"
version = "1.0.0"

[resources.memory]
max_bytes = 4194304

[resources.cpu]
max_fuel = 10000
timeout_seconds = 30
        "#;

        let config = ComponentConfigToml::from_str(toml_content).unwrap();
        assert_eq!(
            config.resources.as_ref().unwrap().memory.as_ref().unwrap().max_bytes,
            4194304
        );
    }

    #[test]
    fn test_component_config_to_resource_limits() {
        let toml_content = r#"
[component]
name = "test-component"
version = "1.0.0"

[resources.memory]
max_bytes = 2097152

[resources.cpu]
max_fuel = 10000
timeout_seconds = 30
        "#;

        let config = ComponentConfigToml::from_str(toml_content).unwrap();
        let limits = config.to_resource_limits().unwrap();
        assert_eq!(limits.max_memory_bytes(), 2097152);
        assert_eq!(limits.max_fuel(), 10000);
        assert_eq!(limits.timeout_seconds(), 30);
    }

    #[test]
    fn test_component_config_with_cpu() {
        let toml_content = r#"
[component]
name = "test-component"
version = "1.0.0"

[resources.memory]
max_bytes = 1048576

[resources.cpu]
max_fuel = 50000
timeout_seconds = 60
        "#;

        let config = ComponentConfigToml::from_str(toml_content).unwrap();
        assert_eq!(config.component.name, "test-component");
        assert_eq!(
            config.resources.as_ref().unwrap().memory.as_ref().unwrap().max_bytes,
            1048576
        );
        assert_eq!(
            config.resources.as_ref().unwrap().cpu.as_ref().unwrap().max_fuel,
            Some(50000)
        );
        assert_eq!(
            config.resources.as_ref().unwrap().cpu.as_ref().unwrap().timeout_seconds,
            Some(60)
        );
    }
}
