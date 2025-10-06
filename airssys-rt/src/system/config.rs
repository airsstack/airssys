//! System configuration with sensible defaults.

// Layer 1: Standard library
use std::time::Duration;

// Layer 2: Third-party
use serde::{Deserialize, Serialize};

// Layer 3: Internal
// (none initially)

/// Default mailbox capacity for bounded mailboxes
pub const DEFAULT_MAILBOX_CAPACITY: usize = 1000;

/// Default timeout for actor spawn operations (5 seconds)
pub const DEFAULT_SPAWN_TIMEOUT: Duration = Duration::from_secs(5);

/// Default timeout for graceful system shutdown (30 seconds)
pub const DEFAULT_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(30);

/// Default maximum concurrent actors (0 = unlimited)
pub const DEFAULT_MAX_ACTORS: usize = 0;

/// Default metrics collection setting (disabled following YAGNI §6.1)
pub const DEFAULT_ENABLE_METRICS: bool = false;

/// System-wide configuration for actor runtime.
///
/// Provides sensible defaults following §6.1 YAGNI principles.
///
/// # Examples
///
/// ```rust
/// use airssys_rt::system::{SystemConfig, DEFAULT_MAILBOX_CAPACITY};
/// use std::time::Duration;
///
/// // Use default configuration
/// let config = SystemConfig::default();
/// assert_eq!(config.default_mailbox_capacity, DEFAULT_MAILBOX_CAPACITY);
///
/// // Use builder for custom configuration
/// let config = SystemConfig::builder()
///     .with_mailbox_capacity(500)
///     .with_spawn_timeout(Duration::from_secs(10))
///     .build()
///     .unwrap();
/// assert_eq!(config.default_mailbox_capacity, 500);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    /// Default mailbox capacity for bounded mailboxes
    pub default_mailbox_capacity: usize,

    /// Timeout for actor spawn operations
    pub spawn_timeout: Duration,

    /// Timeout for graceful system shutdown
    pub shutdown_timeout: Duration,

    /// Maximum concurrent actors (0 = unlimited)
    pub max_actors: usize,

    /// Enable system metrics collection (disabled by default - YAGNI)
    pub enable_metrics: bool,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            default_mailbox_capacity: DEFAULT_MAILBOX_CAPACITY,
            spawn_timeout: DEFAULT_SPAWN_TIMEOUT,
            shutdown_timeout: DEFAULT_SHUTDOWN_TIMEOUT,
            max_actors: DEFAULT_MAX_ACTORS,
            enable_metrics: DEFAULT_ENABLE_METRICS,
        }
    }
}

impl SystemConfig {
    /// Create a new configuration builder.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::system::SystemConfig;
    /// use std::time::Duration;
    ///
    /// let config = SystemConfig::builder()
    ///     .with_mailbox_capacity(500)
    ///     .with_max_actors(100)
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn builder() -> SystemConfigBuilder {
        SystemConfigBuilder::default()
    }

    /// Validate configuration values.
    ///
    /// Returns `Err` if any configuration value is invalid.
    pub fn validate(&self) -> Result<(), String> {
        if self.default_mailbox_capacity == 0 {
            return Err("default_mailbox_capacity must be > 0".to_string());
        }

        if self.spawn_timeout.as_secs() == 0 && self.spawn_timeout.as_millis() == 0 {
            return Err("spawn_timeout must be > 0".to_string());
        }

        if self.shutdown_timeout.as_secs() == 0 && self.shutdown_timeout.as_millis() == 0 {
            return Err("shutdown_timeout must be > 0".to_string());
        }

        Ok(())
    }
}

/// Builder for SystemConfig with fluent API.
///
/// Follows Builder Pattern for ergonomic configuration.
#[derive(Debug, Default)]
pub struct SystemConfigBuilder {
    config: SystemConfig,
}

impl SystemConfigBuilder {
    /// Set default mailbox capacity for bounded mailboxes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::system::SystemConfig;
    ///
    /// let config = SystemConfig::builder()
    ///     .with_mailbox_capacity(500)
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(config.default_mailbox_capacity, 500);
    /// ```
    pub fn with_mailbox_capacity(mut self, capacity: usize) -> Self {
        self.config.default_mailbox_capacity = capacity;
        self
    }

    /// Set timeout for actor spawn operations.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::system::SystemConfig;
    /// use std::time::Duration;
    ///
    /// let config = SystemConfig::builder()
    ///     .with_spawn_timeout(Duration::from_secs(10))
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(config.spawn_timeout, Duration::from_secs(10));
    /// ```
    pub fn with_spawn_timeout(mut self, timeout: Duration) -> Self {
        self.config.spawn_timeout = timeout;
        self
    }

    /// Set timeout for graceful system shutdown.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::system::SystemConfig;
    /// use std::time::Duration;
    ///
    /// let config = SystemConfig::builder()
    ///     .with_shutdown_timeout(Duration::from_secs(60))
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(config.shutdown_timeout, Duration::from_secs(60));
    /// ```
    pub fn with_shutdown_timeout(mut self, timeout: Duration) -> Self {
        self.config.shutdown_timeout = timeout;
        self
    }

    /// Set maximum number of concurrent actors (0 = unlimited).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::system::SystemConfig;
    ///
    /// let config = SystemConfig::builder()
    ///     .with_max_actors(100)
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(config.max_actors, 100);
    /// ```
    pub fn with_max_actors(mut self, max: usize) -> Self {
        self.config.max_actors = max;
        self
    }

    /// Enable or disable system metrics collection.
    ///
    /// Disabled by default following YAGNI principle.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::system::SystemConfig;
    ///
    /// let config = SystemConfig::builder()
    ///     .with_metrics(true)
    ///     .build()
    ///     .unwrap();
    /// assert!(config.enable_metrics);
    /// ```
    pub fn with_metrics(mut self, enabled: bool) -> Self {
        self.config.enable_metrics = enabled;
        self
    }

    /// Build and validate the configuration.
    ///
    /// Returns `Err` if configuration is invalid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::system::SystemConfig;
    ///
    /// // Valid configuration
    /// let config = SystemConfig::builder()
    ///     .with_mailbox_capacity(500)
    ///     .build();
    /// assert!(config.is_ok());
    ///
    /// // Invalid configuration
    /// let invalid = SystemConfig::builder()
    ///     .with_mailbox_capacity(0)
    ///     .build();
    /// assert!(invalid.is_err());
    /// ```
    pub fn build(self) -> Result<SystemConfig, String> {
        self.config.validate()?;
        Ok(self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SystemConfig::default();
        assert_eq!(config.default_mailbox_capacity, 1000);
        assert_eq!(config.spawn_timeout, Duration::from_secs(5));
        assert_eq!(config.shutdown_timeout, Duration::from_secs(30));
        assert_eq!(config.max_actors, 0);
        assert!(!config.enable_metrics);
    }

    #[test]
    fn test_config_validation_success() {
        let config = SystemConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_zero_capacity() {
        let invalid = SystemConfig {
            default_mailbox_capacity: 0,
            ..Default::default()
        };
        let result = invalid.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("mailbox_capacity"));
    }

    #[test]
    fn test_config_validation_zero_spawn_timeout() {
        let invalid = SystemConfig {
            spawn_timeout: Duration::from_secs(0),
            ..Default::default()
        };
        let result = invalid.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("spawn_timeout"));
    }

    #[test]
    fn test_config_validation_zero_shutdown_timeout() {
        let invalid = SystemConfig {
            shutdown_timeout: Duration::from_secs(0),
            ..Default::default()
        };
        let result = invalid.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("shutdown_timeout"));
    }

    #[test]
    fn test_builder_default() {
        let config = SystemConfig::builder().build().unwrap();
        assert_eq!(config.default_mailbox_capacity, 1000);
    }

    #[test]
    fn test_builder_with_capacity() {
        let config = SystemConfig::builder()
            .with_mailbox_capacity(500)
            .build()
            .unwrap();
        assert_eq!(config.default_mailbox_capacity, 500);
    }

    #[test]
    fn test_builder_with_spawn_timeout() {
        let config = SystemConfig::builder()
            .with_spawn_timeout(Duration::from_secs(10))
            .build()
            .unwrap();
        assert_eq!(config.spawn_timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_builder_with_shutdown_timeout() {
        let config = SystemConfig::builder()
            .with_shutdown_timeout(Duration::from_secs(60))
            .build()
            .unwrap();
        assert_eq!(config.shutdown_timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_builder_with_max_actors() {
        let config = SystemConfig::builder()
            .with_max_actors(100)
            .build()
            .unwrap();
        assert_eq!(config.max_actors, 100);
    }

    #[test]
    fn test_builder_with_metrics() {
        let config = SystemConfig::builder().with_metrics(true).build().unwrap();
        assert!(config.enable_metrics);
    }

    #[test]
    fn test_builder_multiple_options() {
        let config = SystemConfig::builder()
            .with_mailbox_capacity(500)
            .with_spawn_timeout(Duration::from_secs(10))
            .with_shutdown_timeout(Duration::from_secs(60))
            .with_max_actors(100)
            .with_metrics(true)
            .build()
            .unwrap();

        assert_eq!(config.default_mailbox_capacity, 500);
        assert_eq!(config.spawn_timeout, Duration::from_secs(10));
        assert_eq!(config.shutdown_timeout, Duration::from_secs(60));
        assert_eq!(config.max_actors, 100);
        assert!(config.enable_metrics);
    }

    #[test]
    fn test_builder_validation_failure() {
        let result = SystemConfig::builder().with_mailbox_capacity(0).build();
        assert!(result.is_err());
    }

    #[test]
    fn test_serialization_json() {
        let config = SystemConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: SystemConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(
            config.default_mailbox_capacity,
            deserialized.default_mailbox_capacity
        );
        assert_eq!(config.max_actors, deserialized.max_actors);
        assert_eq!(config.enable_metrics, deserialized.enable_metrics);
    }

    #[test]
    fn test_config_clone() {
        let config1 = SystemConfig::default();
        let config2 = config1.clone();

        assert_eq!(
            config1.default_mailbox_capacity,
            config2.default_mailbox_capacity
        );
    }

    #[test]
    fn test_config_debug() {
        let config = SystemConfig::default();
        let debug = format!("{:?}", config);
        assert!(debug.contains("SystemConfig"));
    }
}
