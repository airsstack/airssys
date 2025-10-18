use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// CLI configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    /// Default storage backend (sled or rocksdb)
    pub storage_backend: String,

    /// Component storage directory
    pub storage_path: PathBuf,

    /// Registry cache directory
    pub cache_dir: PathBuf,

    /// Ed25519 keypair path
    pub keypair_path: Option<PathBuf>,

    /// Default Git branch for installations
    pub default_branch: String,

    /// Enable automatic signature verification
    pub auto_verify: bool,

    /// Default output format (text, json, yaml)
    pub output_format: String,
}

impl Default for CliConfig {
    fn default() -> Self {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let airssys_dir = home_dir.join(".airssys");

        Self {
            storage_backend: "sled".to_string(),
            storage_path: airssys_dir.join("components"),
            cache_dir: airssys_dir.join("cache"),
            keypair_path: None,
            default_branch: "main".to_string(),
            auto_verify: true,
            output_format: "text".to_string(),
        }
    }
}

impl CliConfig {
    /// Load configuration from file
    pub fn load() -> crate::error::Result<Self> {
        let config_path = Self::config_path();

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: CliConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> crate::error::Result<()> {
        let config_path = Self::config_path();

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| crate::error::CliError::Serialization(e.to_string()))?;

        std::fs::write(&config_path, content)?;
        Ok(())
    }

    /// Get configuration file path
    pub fn config_path() -> PathBuf {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home_dir.join(".airssys").join("config.toml")
    }
}
