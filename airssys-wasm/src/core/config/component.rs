//! Component configuration types.

// Layer 1: Standard library imports (none needed)

// Layer 2: Third-party crate imports
use thiserror::Error;

// Layer 3: Internal module imports
use crate::core::component::id::ComponentId;

// =============================================================================
// Constants
// =============================================================================

/// Default maximum memory in bytes (64MB).
pub const DEFAULT_MAX_MEMORY_BYTES: u64 = 64 * 1024 * 1024;

/// Default maximum execution time in milliseconds (30 seconds).
pub const DEFAULT_MAX_EXECUTION_TIME_MS: u64 = 30_000;

// =============================================================================
// Validation Error
// =============================================================================

/// Configuration validation error.
#[derive(Debug, Clone, Error)]
pub enum ConfigValidationError {
    /// Memory limit cannot be zero.
    #[error("max_memory_bytes cannot be zero")]
    MemoryIsZero,

    /// Execution time cannot be zero.
    #[error("max_execution_time_ms cannot be zero")]
    ExecutionTimeIsZero,

    /// Fuel limit cannot be zero (if set).
    #[error("max_fuel cannot be zero when set")]
    FuelIsZero,

    /// Storage namespace is invalid.
    #[error("Storage namespace is invalid: {reason}")]
    InvalidStorageNamespace {
        /// The invalid namespace value.
        value: String,
        /// Reason why it's invalid.
        reason: String,
    },
}

// =============================================================================
// ComponentConfig
// =============================================================================

/// Configuration for component instantiation.
///
/// Fields are private for encapsulation. Use getters to read values
/// and builder methods to construct with custom values.
///
/// # Validation
///
/// Use [`validate()`](Self::validate) to check configuration before use.
#[derive(Debug, Clone)]
pub struct ComponentConfig {
    id: ComponentId,
    max_memory_bytes: u64,
    max_execution_time_ms: u64,
    max_fuel: Option<u64>,
    storage_namespace: Option<String>,
    debug_mode: bool,
}

impl Default for ComponentConfig {
    fn default() -> Self {
        Self {
            id: ComponentId::new("default", "component", "000"),
            max_memory_bytes: DEFAULT_MAX_MEMORY_BYTES,
            max_execution_time_ms: DEFAULT_MAX_EXECUTION_TIME_MS,
            max_fuel: None,
            storage_namespace: None,
            debug_mode: false,
        }
    }
}

impl ComponentConfig {
    // =========================================================================
    // Constructor
    // =========================================================================

    /// Create a new ComponentConfig with the given ComponentId.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the component
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::component::ComponentId;
    /// use airssys_wasm::core::config::ComponentConfig;
    ///
    /// let id = ComponentId::new("system", "database", "prod");
    /// let config = ComponentConfig::new(id);
    /// assert_eq!(config.id().namespace, "system");
    /// ```
    pub fn new(id: ComponentId) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }

    // =========================================================================
    // Builder Methods
    // =========================================================================

    /// Set maximum memory limit in bytes.
    ///
    /// # Arguments
    ///
    /// * `bytes` - Maximum memory allocation in bytes (must be > 0)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::component::ComponentId;
    /// use airssys_wasm::core::config::ComponentConfig;
    ///
    /// let config = ComponentConfig::new(ComponentId::new("a", "b", "c"))
    ///     .with_max_memory(128 * 1024 * 1024); // 128MB
    /// assert_eq!(config.max_memory_bytes(), 128 * 1024 * 1024);
    /// ```
    pub fn with_max_memory(mut self, bytes: u64) -> Self {
        self.max_memory_bytes = bytes;
        self
    }

    /// Set maximum execution time in milliseconds.
    ///
    /// # Arguments
    ///
    /// * `ms` - Maximum execution time in milliseconds (must be > 0)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::component::ComponentId;
    /// use airssys_wasm::core::config::ComponentConfig;
    ///
    /// let config = ComponentConfig::new(ComponentId::new("a", "b", "c"))
    ///     .with_max_execution_time(60_000); // 60 seconds
    /// assert_eq!(config.max_execution_time_ms(), 60_000);
    /// ```
    pub fn with_max_execution_time(mut self, ms: u64) -> Self {
        self.max_execution_time_ms = ms;
        self
    }

    /// Set fuel limit for deterministic execution.
    ///
    /// # Arguments
    ///
    /// * `fuel` - Maximum fuel units (must be > 0 if set)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::component::ComponentId;
    /// use airssys_wasm::core::config::ComponentConfig;
    ///
    /// let config = ComponentConfig::new(ComponentId::new("a", "b", "c"))
    ///     .with_fuel_limit(1_000_000); // 1 million fuel units
    /// assert_eq!(config.max_fuel(), Some(1_000_000));
    /// ```
    pub fn with_fuel_limit(mut self, fuel: u64) -> Self {
        self.max_fuel = Some(fuel);
        self
    }

    /// Set storage namespace for component isolation.
    ///
    /// # Arguments
    ///
    /// * `namespace` - Storage namespace (must not contain `/` or `\`)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::component::ComponentId;
    /// use airssys_wasm::core::config::ComponentConfig;
    ///
    /// let config = ComponentConfig::new(ComponentId::new("a", "b", "c"))
    ///     .with_storage_namespace("my-app-data");
    /// assert_eq!(config.storage_namespace(), Some("my-app-data"));
    /// ```
    pub fn with_storage_namespace(mut self, namespace: impl Into<String>) -> Self {
        self.storage_namespace = Some(namespace.into());
        self
    }

    /// Enable or disable debug mode.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether debug mode is enabled
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::component::ComponentId;
    /// use airssys_wasm::core::config::ComponentConfig;
    ///
    /// let config = ComponentConfig::new(ComponentId::new("a", "b", "c"))
    ///     .with_debug_mode(true);
    /// assert!(config.debug_mode());
    /// ```
    pub fn with_debug_mode(mut self, enabled: bool) -> Self {
        self.debug_mode = enabled;
        self
    }

    // =========================================================================
    // Validation
    // =========================================================================

    /// Validates the configuration.
    ///
    /// # Validation Rules
    ///
    /// - `max_memory_bytes` must be > 0
    /// - `max_execution_time_ms` must be > 0
    /// - `max_fuel` (if set) must be > 0
    /// - `storage_namespace` (if set) must not be empty or contain `/` or `\`
    ///
    /// # Errors
    ///
    /// Returns `ConfigValidationError` if validation fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::component::ComponentId;
    /// use airssys_wasm::core::config::ComponentConfig;
    ///
    /// let config = ComponentConfig::new(ComponentId::new("a", "b", "c"));
    /// assert!(config.validate().is_ok());
    /// ```
    pub fn validate(&self) -> Result<(), ConfigValidationError> {
        if self.max_memory_bytes == 0 {
            return Err(ConfigValidationError::MemoryIsZero);
        }

        if self.max_execution_time_ms == 0 {
            return Err(ConfigValidationError::ExecutionTimeIsZero);
        }

        if let Some(fuel) = self.max_fuel {
            if fuel == 0 {
                return Err(ConfigValidationError::FuelIsZero);
            }
        }

        if let Some(ns) = &self.storage_namespace {
            if ns.is_empty() {
                return Err(ConfigValidationError::InvalidStorageNamespace {
                    value: ns.clone(),
                    reason: "namespace cannot be empty".to_string(),
                });
            }
            if ns.contains('/') || ns.contains('\\') {
                return Err(ConfigValidationError::InvalidStorageNamespace {
                    value: ns.clone(),
                    reason: "namespace cannot contain '/' or '\\'".to_string(),
                });
            }
        }

        Ok(())
    }

    // =========================================================================
    // Getters
    // =========================================================================

    /// Returns the component identifier.
    pub fn id(&self) -> &ComponentId {
        &self.id
    }

    /// Returns the maximum memory limit in bytes.
    pub fn max_memory_bytes(&self) -> u64 {
        self.max_memory_bytes
    }

    /// Returns the maximum execution time in milliseconds.
    pub fn max_execution_time_ms(&self) -> u64 {
        self.max_execution_time_ms
    }

    /// Returns the fuel limit if set.
    pub fn max_fuel(&self) -> Option<u64> {
        self.max_fuel
    }

    /// Returns the storage namespace if set.
    pub fn storage_namespace(&self) -> Option<&str> {
        self.storage_namespace.as_deref()
    }

    /// Returns whether debug mode is enabled.
    pub fn debug_mode(&self) -> bool {
        self.debug_mode
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_default_passes() {
        let config = ComponentConfig::new(ComponentId::new("a", "b", "c"));
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_memory_zero() {
        let config = ComponentConfig::new(ComponentId::new("a", "b", "c")).with_max_memory(0);
        assert!(matches!(
            config.validate(),
            Err(ConfigValidationError::MemoryIsZero)
        ));
    }

    #[test]
    fn test_validate_execution_time_zero() {
        let config =
            ComponentConfig::new(ComponentId::new("a", "b", "c")).with_max_execution_time(0);
        assert!(matches!(
            config.validate(),
            Err(ConfigValidationError::ExecutionTimeIsZero)
        ));
    }

    #[test]
    fn test_validate_fuel_zero() {
        let config = ComponentConfig::new(ComponentId::new("a", "b", "c")).with_fuel_limit(0);
        assert!(matches!(
            config.validate(),
            Err(ConfigValidationError::FuelIsZero)
        ));
    }

    #[test]
    fn test_validate_empty_namespace() {
        let config =
            ComponentConfig::new(ComponentId::new("a", "b", "c")).with_storage_namespace("");
        assert!(matches!(
            config.validate(),
            Err(ConfigValidationError::InvalidStorageNamespace { .. })
        ));
    }

    #[test]
    fn test_validate_namespace_with_slash() {
        let config = ComponentConfig::new(ComponentId::new("a", "b", "c"))
            .with_storage_namespace("invalid/ns");
        assert!(matches!(
            config.validate(),
            Err(ConfigValidationError::InvalidStorageNamespace { .. })
        ));
    }

    #[test]
    fn test_validate_namespace_with_backslash() {
        let config = ComponentConfig::new(ComponentId::new("a", "b", "c"))
            .with_storage_namespace("invalid\\ns");
        assert!(matches!(
            config.validate(),
            Err(ConfigValidationError::InvalidStorageNamespace { .. })
        ));
    }

    #[test]
    fn test_validate_valid_namespace() {
        let config = ComponentConfig::new(ComponentId::new("a", "b", "c"))
            .with_storage_namespace("valid-namespace_123");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_builder_methods_set_correct_values() {
        let config = ComponentConfig::new(ComponentId::new("ns", "name", "inst"))
            .with_max_memory(128 * 1024 * 1024)
            .with_max_execution_time(60_000)
            .with_fuel_limit(1_000_000)
            .with_storage_namespace("my-data")
            .with_debug_mode(true);

        assert_eq!(config.max_memory_bytes(), 128 * 1024 * 1024);
        assert_eq!(config.max_execution_time_ms(), 60_000);
        assert_eq!(config.max_fuel(), Some(1_000_000));
        assert_eq!(config.storage_namespace(), Some("my-data"));
        assert!(config.debug_mode());
    }

    #[test]
    fn test_getters_return_expected_values() {
        let id = ComponentId::new("test-ns", "test-name", "test-inst");
        let config = ComponentConfig::new(id.clone());

        assert_eq!(config.id(), &id);
        assert_eq!(config.max_memory_bytes(), DEFAULT_MAX_MEMORY_BYTES);
        assert_eq!(
            config.max_execution_time_ms(),
            DEFAULT_MAX_EXECUTION_TIME_MS
        );
        assert_eq!(config.max_fuel(), None);
        assert_eq!(config.storage_namespace(), None);
        assert!(!config.debug_mode());
    }

    #[test]
    fn test_default_config_is_valid() {
        let config = ComponentConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_clone_creates_independent_copy() {
        let config1 = ComponentConfig::new(ComponentId::new("a", "b", "c"))
            .with_max_memory(100)
            .with_debug_mode(true);

        let config2 = config1.clone();

        // Verify they are equal initially
        assert_eq!(config1.max_memory_bytes(), config2.max_memory_bytes());
        assert_eq!(config1.debug_mode(), config2.debug_mode());

        // Modify config1 - config2 should remain unchanged
        let config1 = config1.with_max_memory(200);

        assert_eq!(config1.max_memory_bytes(), 200);
        assert_eq!(config2.max_memory_bytes(), 100);
    }
}
