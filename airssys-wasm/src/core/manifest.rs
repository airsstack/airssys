//! Component.toml manifest parsing and validation.
//!
//! This module handles parsing and validating Component.toml manifest files
//! which declare component metadata, permissions, and configuration.
//!
//! # Manifest Structure
//!
//! Component.toml follows TOML format with these sections:
//! - `[package]` - Component metadata (name, version, description, etc.)
//! - `[permissions]` - Permission declarations (filesystem, network, storage)
//! - `[build]` - Build configuration (optional)
//! - `[runtime]` - Runtime configuration (memory, timeout, etc.)
//!
//! # Examples
//!
//! ## Complete Component.toml
//!
//! ```toml
//! [package]
//! name = "data-processor"
//! version = "1.0.0"
//! description = "High-performance data processor"
//! authors = ["developer@example.com"]
//!
//! [permissions.filesystem]
//! read = ["/data/**", "/config/*.json"]
//! write = ["/output/**"]
//!
//! [permissions.network]
//! outbound = [
//!     { host = "api.example.com", port = 443 }
//! ]
//!
//! [permissions.storage]
//! namespaces = ["myapp:cache"]
//! max_size_mb = 100
//!
//! [runtime]
//! memory_min_mb = 128
//! memory_max_mb = 512
//! timeout_ms = 60000
//! ```
//!
//! ## Parsing
//!
//! ```rust
//! use airssys_wasm::core::manifest::ComponentManifest;
//!
//! let toml_content = r#"
//! [package]
//! name = "my-component"
//! version = "1.0.0"
//! "#;
//!
//! let manifest = ComponentManifest::from_toml_str(toml_content).unwrap();
//! assert_eq!(manifest.package.name, "my-component");
//! ```
//!
//! # References
//!
//! - **KNOWLEDGE-WASM-009**: Component Installation Architecture
//! - **ADR-WASM-005**: Capability-Based Security Model
//! - **ADR-WASM-003**: Component Lifecycle Management

// Layer 1: Standard library imports
use std::fs;
use std::path::Path;

// Layer 2: Third-party crate imports
use serde::{Deserialize, Serialize};

// Layer 3: Internal module imports
use crate::core::error::{WasmError, WasmResult};
use crate::core::permission::PermissionManifest;

/// Complete Component.toml manifest.
///
/// This is the authoritative source of component metadata and permissions.
/// The host reads this manifest before loading the component WASM binary.
///
/// # Structure
///
/// - `package` - Required: Component identification and metadata
/// - `permissions` - Optional: Permission declarations (default: no permissions)
/// - `runtime` - Optional: Runtime configuration hints
///
/// # Examples
///
/// ```rust
/// use airssys_wasm::core::manifest::ComponentManifest;
///
/// let toml = r#"
/// [package]
/// name = "test-component"
/// version = "1.0.0"
///
/// [permissions.filesystem]
/// read = ["/data/**"]
/// "#;
///
/// let manifest = ComponentManifest::from_toml_str(toml).unwrap();
/// assert_eq!(manifest.package.name, "test-component");
/// assert_eq!(manifest.permissions.filesystem.read.len(), 1);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ComponentManifest {
    /// Package metadata (required)
    pub package: PackageInfo,

    /// Permission declarations (optional, defaults to no permissions)
    #[serde(default)]
    pub permissions: PermissionManifest,

    /// Runtime configuration hints (optional)
    #[serde(default)]
    pub runtime: RuntimeConfig,
}

/// Package metadata section from Component.toml.
///
/// Required fields that identify the component and provide
/// basic metadata for users and administrators.
///
/// # Examples
///
/// ```rust
/// use airssys_wasm::core::manifest::PackageInfo;
///
/// let info = PackageInfo {
///     name: "my-component".to_string(),
///     version: "1.0.0".to_string(),
///     description: Some("A test component".to_string()),
///     authors: vec!["dev@example.com".to_string()],
///     license: Some("MIT".to_string()),
///     repository: None,
///     homepage: None,
/// };
///
/// assert_eq!(info.name, "my-component");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PackageInfo {
    /// Component name (kebab-case recommended)
    ///
    /// Must be unique within the host runtime.
    ///
    /// Examples: `"data-processor"`, `"ai-agent"`, `"api-gateway"`
    pub name: String,

    /// Semantic version (semver 2.0.0)
    ///
    /// Format: `MAJOR.MINOR.PATCH`
    ///
    /// Examples: `"1.0.0"`, `"0.3.5"`, `"2.1.3-beta.1"`
    pub version: String,

    /// Brief component description
    ///
    /// Displayed in component listings and dashboards.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Component authors (email addresses recommended)
    ///
    /// Examples: `["dev@example.com"]`, `["Alice <alice@example.com>"]`
    #[serde(default)]
    pub authors: Vec<String>,

    /// License identifier (SPDX format recommended)
    ///
    /// Examples: `"MIT"`, `"Apache-2.0"`, `"GPL-3.0-or-later"`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,

    /// Source code repository URL
    ///
    /// Examples: `"https://github.com/user/component"`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,

    /// Project homepage URL
    ///
    /// Examples: `"https://example.com/docs"`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
}

/// Runtime configuration section from Component.toml.
///
/// Optional hints about component resource requirements and behavior.
/// Host may override or enforce different limits.
///
/// # Examples
///
/// ```rust
/// use airssys_wasm::core::manifest::RuntimeConfig;
///
/// let config = RuntimeConfig {
///     memory_min_mb: Some(128),
///     memory_max_mb: Some(512),
///     timeout_ms: Some(60000),
///     stateful: Some(true),
/// };
///
/// assert_eq!(config.memory_max_mb, Some(512));
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RuntimeConfig {
    /// Minimum memory in megabytes
    ///
    /// Hint for host resource allocation.
    ///
    /// Example: `128` = 128 MB
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_min_mb: Option<u64>,

    /// Maximum memory in megabytes
    ///
    /// Host should enforce this limit.
    ///
    /// Example: `512` = 512 MB
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_max_mb: Option<u64>,

    /// Execution timeout in milliseconds
    ///
    /// Maximum time for a single execute() call.
    ///
    /// Example: `60000` = 60 seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,

    /// Does component maintain state between calls?
    ///
    /// `true` = Stateful component (needs persistent instance)
    /// `false` = Stateless component (can be pooled/recreated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stateful: Option<bool>,
}

impl ComponentManifest {
    /// Parse manifest from TOML string.
    ///
    /// # Errors
    ///
    /// Returns error if TOML is invalid or missing required fields.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::manifest::ComponentManifest;
    ///
    /// let toml = r#"
    /// [package]
    /// name = "test-component"
    /// version = "1.0.0"
    /// "#;
    ///
    /// let manifest = ComponentManifest::from_toml_str(toml).unwrap();
    /// assert_eq!(manifest.package.name, "test-component");
    /// ```
    pub fn from_toml_str(content: &str) -> WasmResult<Self> {
        toml::from_str(content).map_err(|e| {
            WasmError::component_parse_failed(format!("Failed to parse Component.toml: {e}"))
        })
    }

    /// Parse manifest from file path.
    ///
    /// # Errors
    ///
    /// Returns error if file cannot be read or TOML is invalid.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use airssys_wasm::core::manifest::ComponentManifest;
    /// use std::path::Path;
    ///
    /// let manifest = ComponentManifest::from_file(Path::new("Component.toml")).unwrap();
    /// assert!(!manifest.package.name.is_empty());
    /// ```
    pub fn from_file(path: &Path) -> WasmResult<Self> {
        let content = fs::read_to_string(path).map_err(|e| {
            WasmError::io_error(
                format!("Failed to read manifest file '{}'", path.display()),
                e,
            )
        })?;
        Self::from_toml_str(&content)
    }

    /// Validate manifest fields and permission declarations.
    ///
    /// Checks:
    /// - Package name is valid (non-empty, kebab-case)
    /// - Version is valid semver
    /// - Permission patterns are syntactically valid
    ///
    /// # Errors
    ///
    /// Returns error if validation fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::manifest::{ComponentManifest, PackageInfo};
    /// use airssys_wasm::core::permission::PermissionManifest;
    ///
    /// let manifest = ComponentManifest {
    ///     package: PackageInfo {
    ///         name: "valid-component".to_string(),
    ///         version: "1.0.0".to_string(),
    ///         description: None,
    ///         authors: vec![],
    ///         license: None,
    ///         repository: None,
    ///         homepage: None,
    ///     },
    ///     permissions: PermissionManifest::new(),
    ///     runtime: Default::default(),
    /// };
    ///
    /// assert!(manifest.validate().is_ok());
    /// ```
    pub fn validate(&self) -> WasmResult<()> {
        // Validate package name
        self.validate_package_name()?;

        // Validate version (semver)
        self.validate_version()?;

        // Validate permissions (basic checks, detailed validation in permission_checker)
        self.validate_permissions()?;

        Ok(())
    }

    /// Validate package name format.
    ///
    /// Rules:
    /// - Non-empty
    /// - Lowercase alphanumeric with hyphens
    /// - Must start with letter
    /// - No consecutive hyphens
    fn validate_package_name(&self) -> WasmResult<()> {
        let name = &self.package.name;

        if name.is_empty() {
            return Err(WasmError::component_validation_failed(
                "Package name cannot be empty",
            ));
        }

        if !name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            return Err(WasmError::component_validation_failed(
                "Package name must be lowercase alphanumeric with hyphens",
            ));
        }

        // SAFETY: We've already checked that name is not empty above
        if let Some(first_char) = name.chars().next() {
            if !first_char.is_ascii_lowercase() {
                return Err(WasmError::component_validation_failed(
                    "Package name must start with a letter",
                ));
            }
        }

        if name.contains("--") {
            return Err(WasmError::component_validation_failed(
                "Package name cannot contain consecutive hyphens",
            ));
        }

        Ok(())
    }

    /// Validate version format (basic semver check).
    ///
    /// Format: MAJOR.MINOR.PATCH[-prerelease][+build]
    fn validate_version(&self) -> WasmResult<()> {
        let version = &self.package.version;

        // Basic semver check: at least X.Y.Z format
        let parts: Vec<&str> = version.split(&['.', '-', '+'][..]).collect();
        if parts.len() < 3 {
            return Err(WasmError::component_validation_failed(
                "Version must be valid semver (e.g., 1.0.0)",
            ));
        }

        // Check major.minor.patch are numbers
        for part in parts.iter().take(3) {
            if part.parse::<u64>().is_err() {
                return Err(WasmError::component_validation_failed(
                    "Version major.minor.patch must be numbers",
                ));
            }
        }

        Ok(())
    }

    /// Validate permission declarations (basic syntax checks).
    ///
    /// Detailed pattern validation happens in PermissionChecker.
    fn validate_permissions(&self) -> WasmResult<()> {
        // Check filesystem patterns are non-empty
        for pattern in &self.permissions.filesystem.read {
            if pattern.is_empty() {
                return Err(WasmError::component_validation_failed(
                    "Filesystem read patterns cannot be empty",
                ));
            }
        }

        for pattern in &self.permissions.filesystem.write {
            if pattern.is_empty() {
                return Err(WasmError::component_validation_failed(
                    "Filesystem write patterns cannot be empty",
                ));
            }
        }

        // Check network endpoints are valid
        for endpoint in &self.permissions.network.outbound {
            if endpoint.host.is_empty() {
                return Err(WasmError::component_validation_failed(
                    "Network outbound host cannot be empty",
                ));
            }
            if endpoint.port == 0 {
                return Err(WasmError::component_validation_failed(
                    "Network outbound port cannot be 0",
                ));
            }
        }

        // Check storage namespaces are non-empty
        for namespace in &self.permissions.storage.namespaces {
            if namespace.is_empty() {
                return Err(WasmError::component_validation_failed(
                    "Storage namespaces cannot be empty",
                ));
            }
            if !namespace.contains(':') {
                return Err(WasmError::component_validation_failed(
                    "Storage namespace must have format 'prefix:name'",
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)] // Test code is allowed to use unwrap for clarity
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_manifest() {
        let toml = r#"
[package]
name = "test-component"
version = "1.0.0"
"#;

        let manifest = ComponentManifest::from_toml_str(toml).unwrap();
        assert_eq!(manifest.package.name, "test-component");
        assert_eq!(manifest.package.version, "1.0.0");
        assert!(!manifest.permissions.has_any_permissions());
    }

    #[test]
    fn test_parse_complete_manifest() {
        let toml = r#"
[package]
name = "data-processor"
version = "1.0.0"
description = "Processes data files"
authors = ["dev@example.com"]
license = "MIT"

[permissions.filesystem]
read = ["/data/**", "/config/*.json"]
write = ["/output/**"]

[permissions.network]
outbound = [
    { host = "api.example.com", port = 443 }
]

[permissions.storage]
namespaces = ["myapp:cache"]
max_size_mb = 100

[runtime]
memory_min_mb = 128
memory_max_mb = 512
timeout_ms = 60000
stateful = true
"#;

        let manifest = ComponentManifest::from_toml_str(toml).unwrap();
        assert_eq!(manifest.package.name, "data-processor");
        assert_eq!(manifest.permissions.filesystem.read.len(), 2);
        assert_eq!(manifest.permissions.network.outbound.len(), 1);
        assert_eq!(manifest.runtime.memory_max_mb, Some(512));
    }

    #[test]
    fn test_validate_valid_manifest() {
        let toml = r#"
[package]
name = "valid-component"
version = "1.0.0"
"#;

        let manifest = ComponentManifest::from_toml_str(toml).unwrap();
        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn test_validate_invalid_package_name() {
        let toml = r#"
[package]
name = "Invalid_Component"
version = "1.0.0"
"#;

        let manifest = ComponentManifest::from_toml_str(toml).unwrap();
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_version() {
        let toml = r#"
[package]
name = "test-component"
version = "invalid"
"#;

        let manifest = ComponentManifest::from_toml_str(toml).unwrap();
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_validate_empty_filesystem_pattern() {
        let mut manifest = ComponentManifest {
            package: PackageInfo {
                name: "test-component".to_string(),
                version: "1.0.0".to_string(),
                description: None,
                authors: vec![],
                license: None,
                repository: None,
                homepage: None,
            },
            permissions: PermissionManifest::new(),
            runtime: Default::default(),
        };

        manifest.permissions.filesystem.read.push("".to_string());
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_namespace_format() {
        let mut manifest = ComponentManifest {
            package: PackageInfo {
                name: "test-component".to_string(),
                version: "1.0.0".to_string(),
                description: None,
                authors: vec![],
                license: None,
                repository: None,
                homepage: None,
            },
            permissions: PermissionManifest::new(),
            runtime: Default::default(),
        };

        manifest
            .permissions
            .storage
            .namespaces
            .push("invalid-namespace".to_string());
        assert!(manifest.validate().is_err());
    }
}
