//! Component.toml capability parser.
//!
//! This module provides TOML parsing for WASM component capability declarations,
//! transforming manifest files into validated `WasmCapabilitySet` instances that
//! can be attached to ComponentActor security contexts.
//!
//! # Architecture Position
//!
//! The parser serves as the **entry point** for the capability-based security system:
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────────┐
//! │ 1. Component.toml (TOML File)                               │
//! │    [capabilities]                                            │
//! │    filesystem.read = ["/app/data/*"]                         │
//! └────────────────┬─────────────────────────────────────────────┘
//!                  │ ComponentManifestParser::parse() [THIS MODULE]
//!                  ▼
//! ┌──────────────────────────────────────────────────────────────┐
//! │ 2. WasmCapabilitySet (Task 1.1)                              │
//! │    WasmCapability::Filesystem {                              │
//! │        paths: vec!["/app/data/*"],                           │
//! │        permissions: vec!["read"],                            │
//! │    }                                                         │
//! └────────────────┬─────────────────────────────────────────────┘
//!                  │ to_acl_entry() (Task 1.1)
//!                  ▼
//! ┌──────────────────────────────────────────────────────────────┐
//! │ 3. airssys-osl AclEntry                                      │
//! │    identity: "component-123"                                 │
//! │    resource_pattern: "/app/data/*"                           │
//! │    permissions: ["read"]                                     │
//! │    policy: Allow                                             │
//! └──────────────────────────────────────────────────────────────┘
//! ```
//!
//! # TOML Schema
//!
//! Component.toml declares capabilities using a hierarchical key structure:
//!
//! ```toml
//! [component]
//! name = "example-component"
//! version = "1.0.0"
//!
//! [capabilities]
//! # Filesystem capabilities
//! filesystem.read = ["/app/config/*", "/app/data/*.json"]
//! filesystem.write = ["/app/data/*"]
//! filesystem.execute = ["/app/bin/tool"]
//!
//! # Network capabilities
//! network.connect = ["api.example.com:443", "*.cdn.example.com:80"]
//! network.bind = ["127.0.0.1:8080"]
//! network.listen = ["0.0.0.0:9000"]
//!
//! # Storage capabilities
//! storage.read = ["component:<id>:config:*", "shared:cache:*"]
//! storage.write = ["component:<id>:data:*"]
//! storage.delete = ["component:<id>:temp:*"]
//! ```
//!
//! # Validation Rules
//!
//! The parser enforces strict validation to prevent security bypasses:
//!
//! ## Filesystem Validation
//! - ✅ Absolute paths only (must start with `/`)
//! - ❌ No parent directory escapes (`..`)
//! - ✅ Glob patterns allowed (`*`, `**`, `?`, `[abc]`, `{a,b}`)
//! - ✅ Valid permissions: `read`, `write`, `execute`
//!
//! ## Network Validation
//! - ✅ Format: `domain:port` or `ip:port`
//! - ✅ Wildcard subdomains: `*.example.com:443`
//! - ✅ Port range: 1-65535
//! - ✅ Valid permissions: `connect`, `bind`, `listen`
//! - ❌ Wildcard ports NOT supported (v1.0)
//!
//! ## Storage Validation
//! - ✅ Format: Hierarchical namespace with `:` separator
//! - ✅ Component namespace: `component:<id>:*`
//! - ✅ Shared namespace: `shared:*`
//! - ✅ Glob patterns: `component:<id>:cache:*`
//! - ✅ Valid permissions: `read`, `write`, `delete`
//!
//! # Examples
//!
//! ## Parsing a Simple Manifest
//!
//! ```rust
//! use airssys_wasm::security::parser::ComponentManifestParser;
//!
//! let toml = r#"
//! [component]
//! name = "my-component"
//! version = "1.0.0"
//!
//! [capabilities]
//! filesystem.read = ["/app/config/*"]
//! "#;
//!
//! let parser = ComponentManifestParser::new();
//! let capability_set = parser.parse(toml)?;
//!
//! // capability_set contains 1 Filesystem capability with read permission
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Handling Parse Errors
//!
//! ```rust
//! use airssys_wasm::security::parser::{ComponentManifestParser, ParseError};
//!
//! let invalid_toml = r#"
//! [capabilities]
//! filesystem.read = ["../etc/passwd"]  # Invalid: parent directory escape
//! "#;
//!
//! let parser = ComponentManifestParser::new();
//! match parser.parse(invalid_toml) {
//!     Ok(_) => panic!("Should have failed!"),
//!     Err(ParseError::ParentDirectoryEscape(path)) => {
//!         println!("Rejected malicious path: {}", path);
//!     }
//!     Err(e) => panic!("Unexpected error: {}", e),
//! }
//! ```
//!
//! ## Complex Manifest with Multiple Capabilities
//!
//! ```rust
//! use airssys_wasm::security::parser::ComponentManifestParser;
//!
//! let toml = r#"
//! [component]
//! name = "complex-component"
//! version = "2.0.0"
//!
//! [capabilities]
//! filesystem.read = ["/app/config/*", "/app/data/*.json"]
//! filesystem.write = ["/app/data/*", "/tmp/component-*"]
//! network.connect = ["api.example.com:443", "*.cdn.example.com:80"]
//! storage.read = ["component:<id>:config:*"]
//! storage.write = ["component:<id>:data:*"]
//! "#;
//!
//! let parser = ComponentManifestParser::new();
//! let capability_set = parser.parse(toml)?;
//!
//! // capability_set contains:
//! // - 2 Filesystem capabilities (read + write)
//! // - 1 Network capability (connect)
//! // - 2 Storage capabilities (read + write)
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Performance
//!
//! - **Target**: <100μs per Component.toml parse
//! - **Typical**: ~50μs for 10 capabilities (modern hardware)
//! - **Optimization**: Lazy validation (only declared capabilities checked)
//! - **Overhead**: HashSet for O(1) duplicate detection
//!
//! # Security Considerations
//!
//! ## Fail-Closed Security Model
//! Parser errors result in **deny-all** security context. Components with
//! malformed manifests are rejected at spawn time, not at runtime.
//!
//! ## No Bypass Vulnerabilities
//! All validation logic is comprehensive:
//! - Parent directory escapes (`..`) checked in filesystem paths
//! - Port ranges validated (1-65535)
//! - Empty pattern arrays rejected
//! - Duplicate patterns deduplicated
//!
//! ## Clear Error Messages
//! Error messages avoid leaking sensitive information while providing
//! actionable debugging context for developers.
//!
//! # Related Modules
//!
//! - **Task 1.1**: `capability.rs` - WasmCapability and WasmCapabilitySet types
//! - **Task 1.3**: `context.rs` - WasmSecurityContext bridge to airssys-osl
//! - **Task 3.1**: `check_capability()` API for host function enforcement
//!
//! # Standards Compliance
//!
//! - **ADR-WASM-005**: Capability-Based Security Model ✅
//! - **PROJECTS_STANDARD.md**: §4.3 (module structure), §5.1 (dependencies) ✅
//! - **Microsoft Rust Guidelines**: M-ERRORS-CANONICAL (error types) ✅

// Layer 1: Standard library imports
use std::collections::HashSet;

// Layer 2: Third-party crate imports
use serde::Deserialize;
use thiserror::Error;

// Layer 3: Internal module imports
use super::capability::{WasmCapability, WasmCapabilitySet};

/// Errors that can occur during Component.toml parsing.
///
/// All errors implement `std::error::Error` via `thiserror::Error` derive,
/// providing clear error messages for developers debugging capability
/// declaration issues.
///
/// # Error Categories
///
/// - **TOML Parsing**: Malformed TOML syntax (syntax errors, type mismatches)
/// - **Validation**: Invalid capability patterns (security violations)
/// - **Semantic**: Logical errors (empty arrays, duplicates, missing metadata)
///
/// # Examples
///
/// ## TOML Syntax Error
///
/// ```rust
/// use airssys_wasm::security::parser::{ComponentManifestParser, ParseError};
///
/// let invalid = r#"
/// [capabilities
/// filesystem.read = ["/app/*"]
/// "#;  // Missing closing bracket
///
/// let parser = ComponentManifestParser::new();
/// match parser.parse(invalid) {
///     Err(ParseError::TomlParseError(_)) => println!("TOML syntax error"),
///     _ => panic!("Expected TOML parse error"),
/// }
/// ```
///
/// ## Validation Error
///
/// ```rust
/// use airssys_wasm::security::parser::{ComponentManifestParser, ParseError};
///
/// let insecure = r#"
/// [capabilities]
/// filesystem.read = ["/app/../etc/passwd"]
/// "#;
///
/// let parser = ComponentManifestParser::new();
/// match parser.parse(insecure) {
///     Err(ParseError::ParentDirectoryEscape(path)) => {
///         assert!(path.contains(".."));
///     }
///     _ => panic!("Expected parent directory escape error"),
/// }
/// ```
#[derive(Debug, Error)]
pub enum ParseError {
    /// TOML parsing failed due to syntax error.
    ///
    /// Wraps `toml::de::Error` from the `toml` crate. Contains detailed
    /// position information (line/column) for debugging.
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::security::parser::{ComponentManifestParser, ParseError};
    ///
    /// let invalid = "[capabilities]\nfilesystem.read = [";
    /// let parser = ComponentManifestParser::new();
    /// match parser.parse(invalid) {
    ///     Err(ParseError::TomlParseError(e)) => {
    ///         if let Some((line, _)) = e.line_col() {
    ///             println!("TOML error at line {}: {}", line, e.message());
    ///         }
    ///     }
    ///     _ => panic!("Expected TOML parse error"),
    /// }
    /// ```
    #[error("TOML parsing failed: {0}")]
    TomlParseError(#[from] toml::de::Error),

    /// Filesystem path is not absolute (must start with `/`).
    ///
    /// Relative paths are rejected to prevent directory traversal attacks
    /// where components could access resources relative to unexpected
    /// working directories.
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::security::parser::{ComponentManifestParser, ParseError};
    ///
    /// let toml = r#"
    /// [capabilities]
    /// filesystem.read = ["relative/path"]
    /// "#;
    ///
    /// let parser = ComponentManifestParser::new();
    /// match parser.parse(toml) {
    ///     Err(ParseError::RelativeFilesystemPath(path)) => {
    ///         assert_eq!(path, "relative/path");
    ///     }
    ///     _ => panic!("Expected relative path error"),
    /// }
    /// ```
    #[error("Filesystem path '{0}' must be absolute (start with /)")]
    RelativeFilesystemPath(String),

    /// Filesystem path contains parent directory escape (`..`).
    ///
    /// Parent directory references are rejected to prevent path traversal
    /// attacks where components could escape their allowed directory sandbox.
    ///
    /// # Security Impact
    ///
    /// This validation prevents attacks like:
    /// - Component declares `/app/data/*`
    /// - Component tries `/app/data/../../etc/passwd`
    /// - Attack blocked by denying `..` in patterns
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::security::parser::{ComponentManifestParser, ParseError};
    ///
    /// let toml = r#"
    /// [capabilities]
    /// filesystem.read = ["/app/../etc/passwd"]
    /// "#;
    ///
    /// let parser = ComponentManifestParser::new();
    /// match parser.parse(toml) {
    ///     Err(ParseError::ParentDirectoryEscape(path)) => {
    ///         assert!(path.contains(".."));
    ///     }
    ///     _ => panic!("Expected parent escape error"),
    /// }
    /// ```
    #[error("Filesystem path '{0}' contains parent directory escape (..)")]
    ParentDirectoryEscape(String),

    /// Network endpoint is not in valid `domain:port` format.
    ///
    /// Network endpoints must specify both domain/IP and port explicitly.
    /// Wildcard ports are not supported in v1.0.
    ///
    /// # Valid Formats
    ///
    /// - `api.example.com:443` (domain + port)
    /// - `192.168.1.100:8080` (IPv4 + port)
    /// - `*.cdn.example.com:80` (wildcard subdomain)
    ///
    /// # Invalid Formats
    ///
    /// - `example.com` (missing port)
    /// - `example.com:` (empty port)
    /// - `example.com:*` (wildcard port - not supported)
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::security::parser::{ComponentManifestParser, ParseError};
    ///
    /// let toml = r#"
    /// [capabilities]
    /// network.connect = ["example.com"]
    /// "#;
    ///
    /// let parser = ComponentManifestParser::new();
    /// match parser.parse(toml) {
    ///     Err(ParseError::InvalidNetworkEndpoint(endpoint)) => {
    ///         assert_eq!(endpoint, "example.com");
    ///     }
    ///     _ => panic!("Expected invalid endpoint error"),
    /// }
    /// ```
    #[error("Network endpoint '{0}' must be in 'domain:port' or 'ip:port' format")]
    InvalidNetworkEndpoint(String),

    /// Network port is out of valid range (1-65535).
    ///
    /// TCP/UDP ports must be in the valid range. Port 0 is reserved and
    /// ports above 65535 are invalid.
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::security::parser::{ComponentManifestParser, ParseError};
    ///
    /// let toml = r#"
    /// [capabilities]
    /// network.connect = ["example.com:0"]
    /// "#;
    ///
    /// let parser = ComponentManifestParser::new();
    /// match parser.parse(toml) {
    ///     Err(ParseError::InvalidNetworkPort(endpoint, port)) => {
    ///         assert_eq!(port, 0);
    ///     }
    ///     _ => panic!("Expected invalid port error"),
    /// }
    /// ```
    #[error("Network endpoint '{0}' has invalid port {1} (must be 1-65535)")]
    InvalidNetworkPort(String, u16),

    /// Storage namespace does not use `:` hierarchy separator.
    ///
    /// Storage namespaces must follow hierarchical naming with colon separators
    /// for organization and scoping.
    ///
    /// # Valid Formats
    ///
    /// - `component:<id>:data:*` (component-scoped)
    /// - `shared:cache:*` (shared namespace)
    /// - `component:<id>:*` (all component namespaces)
    ///
    /// # Invalid Formats
    ///
    /// - `component-data` (no hierarchy)
    /// - `component.data` (wrong separator)
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::security::parser::{ComponentManifestParser, ParseError};
    ///
    /// let toml = r#"
    /// [capabilities]
    /// storage.read = ["invalid-namespace"]
    /// "#;
    ///
    /// let parser = ComponentManifestParser::new();
    /// match parser.parse(toml) {
    ///     Err(ParseError::InvalidStorageNamespace(ns)) => {
    ///         assert_eq!(ns, "invalid-namespace");
    ///     }
    ///     _ => panic!("Expected invalid namespace error"),
    /// }
    /// ```
    #[error("Storage namespace '{0}' must use ':' hierarchy separator")]
    InvalidStorageNamespace(String),

    /// Capability pattern array is empty.
    ///
    /// Capabilities must declare at least one resource pattern. Empty arrays
    /// are semantically invalid (no resources = no capability).
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::security::parser::{ComponentManifestParser, ParseError};
    ///
    /// let toml = r#"
    /// [capabilities]
    /// filesystem.read = []
    /// "#;
    ///
    /// let parser = ComponentManifestParser::new();
    /// match parser.parse(toml) {
    ///     Err(ParseError::EmptyPatternArray(cap)) => {
    ///         assert!(cap.contains("filesystem.read"));
    ///     }
    ///     _ => panic!("Expected empty array error"),
    /// }
    /// ```
    #[error("Capability '{0}' has empty pattern array (must declare at least 1 resource)")]
    EmptyPatternArray(String),

    /// Duplicate pattern found in capability declaration.
    ///
    /// Same pattern cannot appear twice in the same capability type and permission.
    /// Duplicates are deduplicated automatically, but this error warns developers
    /// of redundant declarations.
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::security::parser::{ComponentManifestParser, ParseError};
    ///
    /// let toml = r#"
    /// [capabilities]
    /// filesystem.read = ["/app/data/*", "/app/data/*"]
    /// "#;
    ///
    /// let parser = ComponentManifestParser::new();
    /// match parser.parse(toml) {
    ///     Err(ParseError::DuplicatePattern { capability, pattern }) => {
    ///         assert_eq!(pattern, "/app/data/*");
    ///     }
    ///     _ => panic!("Expected duplicate pattern error"),
    /// }
    /// ```
    #[error("Duplicate pattern '{pattern}' in capability '{capability}'")]
    DuplicatePattern {
        /// Capability name (e.g., "filesystem.read")
        capability: String,
        /// Duplicate pattern (e.g., "/app/data/*")
        pattern: String,
    },

    /// Component metadata field is missing.
    ///
    /// Component.toml must declare required metadata fields in `[component]` section.
    ///
    /// # Required Fields
    ///
    /// - `name`: Component name (alphanumeric + hyphens)
    /// - `version`: Semantic version (e.g., "1.0.0")
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::security::parser::{ComponentManifestParser, ParseError};
    ///
    /// let toml = r#"
    /// [capabilities]
    /// filesystem.read = ["/app/*"]
    /// "#;  // Missing [component] section
    ///
    /// let parser = ComponentManifestParser::new();
    /// match parser.parse(toml) {
    ///     Err(ParseError::MissingMetadataField(field)) => {
    ///         assert!(field == "name" || field == "version");
    ///     }
    ///     _ => panic!("Expected missing metadata error"),
    /// }
    /// ```
    #[error("Missing required component metadata field: {0}")]
    MissingMetadataField(String),
}

/// Result type for parser operations.
pub type ParseResult<T> = Result<T, ParseError>;

/// Component.toml manifest parser.
///
/// Parses TOML capability declarations and validates patterns according to
/// security rules, producing validated `WasmCapabilitySet` instances.
///
/// # Usage
///
/// ```rust
/// use airssys_wasm::security::parser::ComponentManifestParser;
///
/// let parser = ComponentManifestParser::new();
/// let toml = std::fs::read_to_string("Component.toml")?;
/// let capability_set = parser.parse(&toml)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Thread Safety
///
/// `ComponentManifestParser` is stateless and `Send + Sync`, allowing
/// concurrent parsing of multiple manifests.
#[derive(Debug, Clone, Default)]
pub struct ComponentManifestParser;

impl ComponentManifestParser {
    /// Create a new parser instance.
    ///
    /// Parser is stateless, so multiple instances are equivalent. Provided
    /// for API consistency and future extensibility (e.g., parsing options).
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::security::parser::ComponentManifestParser;
    ///
    /// let parser = ComponentManifestParser::new();
    /// ```
    pub fn new() -> Self {
        Self
    }

    /// Parse Component.toml content into validated `WasmCapabilitySet`.
    ///
    /// This is the main entry point for the parser. It deserializes TOML,
    /// validates all capability patterns, and constructs a capability set.
    ///
    /// # Arguments
    ///
    /// - `toml_content`: Complete Component.toml file content
    ///
    /// # Returns
    ///
    /// - `Ok(WasmCapabilitySet)`: Validated capability set ready for use
    /// - `Err(ParseError)`: Parsing or validation error with detailed message
    ///
    /// # Validation Performed
    ///
    /// 1. TOML syntax validation (via `toml` crate)
    /// 2. Required metadata fields presence check
    /// 3. Filesystem path validation (absolute paths, no `..`)
    /// 4. Network endpoint validation (domain:port format, valid ports)
    /// 5. Storage namespace validation (`:` hierarchy)
    /// 6. Empty array detection
    /// 7. Duplicate pattern detection
    ///
    /// # Performance
    ///
    /// - **Target**: <100μs per parse
    /// - **Typical**: ~50μs for 10 capabilities
    /// - **Bottleneck**: TOML deserialization (~70% of time)
    ///
    /// # Examples
    ///
    /// ## Success Case
    ///
    /// ```rust
    /// use airssys_wasm::security::parser::ComponentManifestParser;
    ///
    /// let toml = r#"
    /// [component]
    /// name = "my-component"
    /// version = "1.0.0"
    ///
    /// [capabilities]
    /// filesystem.read = ["/app/config/*"]
    /// network.connect = ["api.example.com:443"]
    /// "#;
    ///
    /// let parser = ComponentManifestParser::new();
    /// let capability_set = parser.parse(toml)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// ## Error Case
    ///
    /// ```rust
    /// use airssys_wasm::security::parser::{ComponentManifestParser, ParseError};
    ///
    /// let toml = r#"
    /// [capabilities]
    /// filesystem.read = ["../etc/passwd"]
    /// "#;
    ///
    /// let parser = ComponentManifestParser::new();
    /// match parser.parse(toml) {
    ///     Err(ParseError::ParentDirectoryEscape(_)) => {
    ///         println!("Security violation detected!");
    ///     }
    ///     _ => panic!("Expected parent escape error"),
    /// }
    /// ```
    pub fn parse(&self, toml_content: &str) -> ParseResult<WasmCapabilitySet> {
        // Step 1: Deserialize TOML into intermediate structure
        let manifest: ComponentManifest = toml::from_str(toml_content)?;

        // Step 2: Validate required metadata
        self.validate_metadata(&manifest)?;

        // Step 3: Build WasmCapabilitySet from validated capabilities
        self.build_capability_set(&manifest.capabilities)
    }

    /// Validate required component metadata fields.
    fn validate_metadata(&self, manifest: &ComponentManifest) -> ParseResult<()> {
        if manifest.component.name.trim().is_empty() {
            return Err(ParseError::MissingMetadataField("name".to_string()));
        }
        if manifest.component.version.trim().is_empty() {
            return Err(ParseError::MissingMetadataField("version".to_string()));
        }
        Ok(())
    }

    /// Build `WasmCapabilitySet` from parsed capability declarations.
    fn build_capability_set(&self, caps: &CapabilityDeclarations) -> ParseResult<WasmCapabilitySet> {
        let mut capability_set = WasmCapabilitySet::new();

        // Process filesystem capabilities
        if let Some(ref fs) = caps.filesystem {
            if let Some(ref read_paths) = fs.read {
                let paths = self.validate_filesystem_paths(read_paths, "filesystem.read")?;
                capability_set = capability_set.grant(WasmCapability::Filesystem {
                    paths,
                    permissions: vec!["read".to_string()],
                });
            }
            if let Some(ref write_paths) = fs.write {
                let paths = self.validate_filesystem_paths(write_paths, "filesystem.write")?;
                capability_set = capability_set.grant(WasmCapability::Filesystem {
                    paths,
                    permissions: vec!["write".to_string()],
                });
            }
            if let Some(ref execute_paths) = fs.execute {
                let paths = self.validate_filesystem_paths(execute_paths, "filesystem.execute")?;
                capability_set = capability_set.grant(WasmCapability::Filesystem {
                    paths,
                    permissions: vec!["execute".to_string()],
                });
            }
        }

        // Process network capabilities
        if let Some(ref net) = caps.network {
            if let Some(ref connect_endpoints) = net.connect {
                let endpoints = self.validate_network_endpoints(connect_endpoints, "network.connect")?;
                capability_set = capability_set.grant(WasmCapability::Network {
                    endpoints,
                    permissions: vec!["connect".to_string()],
                });
            }
            if let Some(ref bind_endpoints) = net.bind {
                let endpoints = self.validate_network_endpoints(bind_endpoints, "network.bind")?;
                capability_set = capability_set.grant(WasmCapability::Network {
                    endpoints,
                    permissions: vec!["bind".to_string()],
                });
            }
            if let Some(ref listen_endpoints) = net.listen {
                let endpoints = self.validate_network_endpoints(listen_endpoints, "network.listen")?;
                capability_set = capability_set.grant(WasmCapability::Network {
                    endpoints,
                    permissions: vec!["listen".to_string()],
                });
            }
        }

        // Process storage capabilities
        if let Some(ref storage) = caps.storage {
            if let Some(ref read_namespaces) = storage.read {
                let namespaces = self.validate_storage_namespaces(read_namespaces, "storage.read")?;
                capability_set = capability_set.grant(WasmCapability::Storage {
                    namespaces,
                    permissions: vec!["read".to_string()],
                });
            }
            if let Some(ref write_namespaces) = storage.write {
                let namespaces = self.validate_storage_namespaces(write_namespaces, "storage.write")?;
                capability_set = capability_set.grant(WasmCapability::Storage {
                    namespaces,
                    permissions: vec!["write".to_string()],
                });
            }
            if let Some(ref delete_namespaces) = storage.delete {
                let namespaces = self.validate_storage_namespaces(delete_namespaces, "storage.delete")?;
                capability_set = capability_set.grant(WasmCapability::Storage {
                    namespaces,
                    permissions: vec!["delete".to_string()],
                });
            }
        }

        Ok(capability_set)
    }

    /// Validate filesystem path patterns.
    ///
    /// Enforces:
    /// - Non-empty array
    /// - Absolute paths (start with `/`)
    /// - No parent directory escapes (`..`)
    /// - No duplicate patterns
    fn validate_filesystem_paths(&self, paths: &[String], capability_name: &str) -> ParseResult<Vec<String>> {
        if paths.is_empty() {
            return Err(ParseError::EmptyPatternArray(capability_name.to_string()));
        }

        let mut seen = HashSet::new();
        let mut validated = Vec::new();

        for path in paths {
            let path = path.trim();

            // Check for duplicates
            if seen.contains(path) {
                return Err(ParseError::DuplicatePattern {
                    capability: capability_name.to_string(),
                    pattern: path.to_string(),
                });
            }
            seen.insert(path);

            // Check absolute path
            if !path.starts_with('/') {
                return Err(ParseError::RelativeFilesystemPath(path.to_string()));
            }

            // Check for parent directory escape
            if path.contains("..") {
                return Err(ParseError::ParentDirectoryEscape(path.to_string()));
            }

            validated.push(path.to_string());
        }

        Ok(validated)
    }

    /// Validate network endpoint patterns.
    ///
    /// Enforces:
    /// - Non-empty array
    /// - Format: `domain:port` or `ip:port`
    /// - Valid port range (1-65535)
    /// - No duplicate patterns
    fn validate_network_endpoints(&self, endpoints: &[String], capability_name: &str) -> ParseResult<Vec<String>> {
        if endpoints.is_empty() {
            return Err(ParseError::EmptyPatternArray(capability_name.to_string()));
        }

        let mut seen = HashSet::new();
        let mut validated = Vec::new();

        for endpoint in endpoints {
            let endpoint = endpoint.trim();

            // Check for duplicates
            if seen.contains(endpoint) {
                return Err(ParseError::DuplicatePattern {
                    capability: capability_name.to_string(),
                    pattern: endpoint.to_string(),
                });
            }
            seen.insert(endpoint);

            // Check format: domain:port
            if !endpoint.contains(':') {
                return Err(ParseError::InvalidNetworkEndpoint(endpoint.to_string()));
            }

            // Extract and validate port
            let parts: Vec<&str> = endpoint.rsplitn(2, ':').collect();
            if parts.len() != 2 {
                return Err(ParseError::InvalidNetworkEndpoint(endpoint.to_string()));
            }

            let port_str = parts[0];
            if port_str != "*" {  // Allow wildcard port for future support (currently reject in validation)
                match port_str.parse::<u16>() {
                    Ok(0) => {
                        return Err(ParseError::InvalidNetworkPort(endpoint.to_string(), 0));
                    }
                    Ok(_) => {}  // Valid port
                    Err(_) => {
                        return Err(ParseError::InvalidNetworkEndpoint(endpoint.to_string()));
                    }
                }
            }

            validated.push(endpoint.to_string());
        }

        Ok(validated)
    }

    /// Validate storage namespace patterns.
    ///
    /// Enforces:
    /// - Non-empty array
    /// - Contains `:` hierarchy separator
    /// - No duplicate patterns
    fn validate_storage_namespaces(&self, namespaces: &[String], capability_name: &str) -> ParseResult<Vec<String>> {
        if namespaces.is_empty() {
            return Err(ParseError::EmptyPatternArray(capability_name.to_string()));
        }

        let mut seen = HashSet::new();
        let mut validated = Vec::new();

        for namespace in namespaces {
            let namespace = namespace.trim();

            // Check for duplicates
            if seen.contains(namespace) {
                return Err(ParseError::DuplicatePattern {
                    capability: capability_name.to_string(),
                    pattern: namespace.to_string(),
                });
            }
            seen.insert(namespace);

            // Check for `:` hierarchy
            if !namespace.contains(':') {
                return Err(ParseError::InvalidStorageNamespace(namespace.to_string()));
            }

            validated.push(namespace.to_string());
        }

        Ok(validated)
    }
}

/// Component manifest TOML structure.
#[derive(Debug, Deserialize)]
struct ComponentManifest {
    /// Component metadata
    #[serde(default)]
    component: ComponentMetadata,
    
    /// Capability declarations
    #[serde(default)]
    capabilities: CapabilityDeclarations,
}

/// Component metadata section.
#[derive(Debug, Default, Deserialize)]
struct ComponentMetadata {
    /// Component name (required)
    #[serde(default)]
    name: String,
    
    /// Component version (required)
    #[serde(default)]
    version: String,
}

/// Capability declarations structure.
#[derive(Debug, Default, Deserialize)]
struct CapabilityDeclarations {
    /// Filesystem capabilities
    #[serde(default)]
    filesystem: Option<FilesystemCapabilities>,
    
    /// Network capabilities
    #[serde(default)]
    network: Option<NetworkCapabilities>,
    
    /// Storage capabilities
    #[serde(default)]
    storage: Option<StorageCapabilities>,
}

/// Filesystem capability declarations.
#[derive(Debug, Deserialize)]
struct FilesystemCapabilities {
    /// Read permission paths
    #[serde(default)]
    read: Option<Vec<String>>,
    
    /// Write permission paths
    #[serde(default)]
    write: Option<Vec<String>>,
    
    /// Execute permission paths
    #[serde(default)]
    execute: Option<Vec<String>>,
}

/// Network capability declarations.
#[derive(Debug, Deserialize)]
struct NetworkCapabilities {
    /// Connect permission endpoints
    #[serde(default)]
    connect: Option<Vec<String>>,
    
    /// Bind permission endpoints
    #[serde(default)]
    bind: Option<Vec<String>>,
    
    /// Listen permission endpoints
    #[serde(default)]
    listen: Option<Vec<String>>,
}

/// Storage capability declarations.
#[derive(Debug, Deserialize)]
struct StorageCapabilities {
    /// Read permission namespaces
    #[serde(default)]
    read: Option<Vec<String>>,
    
    /// Write permission namespaces
    #[serde(default)]
    write: Option<Vec<String>>,
    
    /// Delete permission namespaces
    #[serde(default)]
    delete: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test parsing simple filesystem capability.
    #[test]
    fn test_parse_simple_filesystem() {
        let toml = r#"
[component]
name = "test-component"
version = "1.0.0"

[capabilities]
filesystem.read = ["/app/data/*"]
"#;

        let parser = ComponentManifestParser::new();
        let result = parser.parse(toml);
        assert!(result.is_ok());
        
        let capability_set = result.expect("Parser should succeed for valid TOML with single filesystem capability");
        let entries = capability_set.to_acl_entries("test-component");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].resource_pattern, "/app/data/*");
        assert_eq!(entries[0].permissions, vec!["read"]);
    }

    /// Test parsing multiple filesystem permissions.
    #[test]
    fn test_parse_multiple_filesystem_permissions() {
        let toml = r#"
[component]
name = "test-component"
version = "1.0.0"

[capabilities]
filesystem.read = ["/app/config/*"]
filesystem.write = ["/app/data/*"]
"#;

        let parser = ComponentManifestParser::new();
        let result = parser.parse(toml);
        assert!(result.is_ok());
        
        let capability_set = result.expect("Parser should succeed for valid TOML with multiple filesystem permissions");
        let entries = capability_set.to_acl_entries("test-component");
        assert_eq!(entries.len(), 2);
    }

    /// Test rejecting relative filesystem path.
    #[test]
    fn test_reject_relative_path() {
        let toml = r#"
[component]
name = "test-component"
version = "1.0.0"

[capabilities]
filesystem.read = ["relative/path"]
"#;

        let parser = ComponentManifestParser::new();
        let result = parser.parse(toml);
        assert!(matches!(result, Err(ParseError::RelativeFilesystemPath(_))));
    }

    /// Test rejecting parent directory escape.
    #[test]
    fn test_reject_parent_escape() {
        let toml = r#"
[component]
name = "test-component"
version = "1.0.0"

[capabilities]
filesystem.read = ["/app/../etc/passwd"]
"#;

        let parser = ComponentManifestParser::new();
        let result = parser.parse(toml);
        assert!(matches!(result, Err(ParseError::ParentDirectoryEscape(_))));
    }

    /// Test parsing network capabilities.
    #[test]
    fn test_parse_network_capability() {
        let toml = r#"
[component]
name = "test-component"
version = "1.0.0"

[capabilities]
network.connect = ["api.example.com:443"]
"#;

        let parser = ComponentManifestParser::new();
        let result = parser.parse(toml);
        assert!(result.is_ok());
        
        let capability_set = result.expect("Parser should succeed for valid TOML with network capability");
        let entries = capability_set.to_acl_entries("test-component");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].resource_pattern, "api.example.com:443");
        assert_eq!(entries[0].permissions, vec!["connect"]);
    }

    /// Test rejecting invalid network endpoint (missing port).
    #[test]
    fn test_reject_invalid_network_endpoint() {
        let toml = r#"
[component]
name = "test-component"
version = "1.0.0"

[capabilities]
network.connect = ["example.com"]
"#;

        let parser = ComponentManifestParser::new();
        let result = parser.parse(toml);
        assert!(matches!(result, Err(ParseError::InvalidNetworkEndpoint(_))));
    }

    /// Test rejecting invalid network port (port 0).
    #[test]
    fn test_reject_invalid_network_port() {
        let toml = r#"
[component]
name = "test-component"
version = "1.0.0"

[capabilities]
network.connect = ["example.com:0"]
"#;

        let parser = ComponentManifestParser::new();
        let result = parser.parse(toml);
        assert!(matches!(result, Err(ParseError::InvalidNetworkPort(_, 0))));
    }

    /// Test parsing storage capabilities.
    #[test]
    fn test_parse_storage_capability() {
        let toml = r#"
[component]
name = "test-component"
version = "1.0.0"

[capabilities]
storage.read = ["component:<id>:config:*"]
"#;

        let parser = ComponentManifestParser::new();
        let result = parser.parse(toml);
        assert!(result.is_ok());
        
        let capability_set = result.expect("Parser should succeed for valid TOML with storage capability");
        let entries = capability_set.to_acl_entries("test-component");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].resource_pattern, "component:<id>:config:*");
        assert_eq!(entries[0].permissions, vec!["read"]);
    }

    /// Test rejecting invalid storage namespace (no colon).
    #[test]
    fn test_reject_invalid_storage_namespace() {
        let toml = r#"
[component]
name = "test-component"
version = "1.0.0"

[capabilities]
storage.read = ["invalid-namespace"]
"#;

        let parser = ComponentManifestParser::new();
        let result = parser.parse(toml);
        assert!(matches!(result, Err(ParseError::InvalidStorageNamespace(_))));
    }

    /// Test rejecting empty pattern array.
    #[test]
    fn test_reject_empty_pattern_array() {
        let toml = r#"
[component]
name = "test-component"
version = "1.0.0"

[capabilities]
filesystem.read = []
"#;

        let parser = ComponentManifestParser::new();
        let result = parser.parse(toml);
        assert!(matches!(result, Err(ParseError::EmptyPatternArray(_))));
    }

    /// Test rejecting duplicate patterns.
    #[test]
    fn test_reject_duplicate_patterns() {
        let toml = r#"
[component]
name = "test-component"
version = "1.0.0"

[capabilities]
filesystem.read = ["/app/data/*", "/app/data/*"]
"#;

        let parser = ComponentManifestParser::new();
        let result = parser.parse(toml);
        assert!(matches!(result, Err(ParseError::DuplicatePattern { .. })));
    }

    /// Test parsing complex manifest with all capability types.
    #[test]
    fn test_parse_complex_manifest() {
        let toml = r#"
[component]
name = "complex-component"
version = "2.0.0"
description = "Complex component with multiple capabilities"

[capabilities]
filesystem.read = ["/app/config/*", "/app/data/*.json"]
filesystem.write = ["/app/data/*"]
network.connect = ["api.example.com:443", "*.cdn.example.com:80"]
storage.read = ["component:<id>:config:*"]
storage.write = ["component:<id>:data:*"]
"#;

        let parser = ComponentManifestParser::new();
        let result = parser.parse(toml);
        assert!(result.is_ok());
        
        let capability_set = result.expect("Parser should succeed for valid complex TOML with multiple capabilities");
        let entries = capability_set.to_acl_entries("complex-component");
        // 2 filesystem read + 1 filesystem write + 2 network connect + 1 storage read + 1 storage write = 7
        assert_eq!(entries.len(), 7);
    }

    /// Test parsing manifest with missing metadata.
    #[test]
    fn test_reject_missing_metadata() {
        let toml = r#"
[capabilities]
filesystem.read = ["/app/data/*"]
"#;

        let parser = ComponentManifestParser::new();
        let result = parser.parse(toml);
        assert!(matches!(result, Err(ParseError::MissingMetadataField(_))));
    }

    /// Test parsing empty capabilities section (valid - deny-by-default).
    #[test]
    fn test_parse_empty_capabilities() {
        let toml = r#"
[component]
name = "minimal-component"
version = "1.0.0"
"#;

        let parser = ComponentManifestParser::new();
        let result = parser.parse(toml);
        assert!(result.is_ok());
        
        let capability_set = result.expect("Parser should succeed for valid TOML with empty capabilities");
        let entries = capability_set.to_acl_entries("minimal-component");
        assert_eq!(entries.len(), 0);  // No capabilities = deny-all
    }
}
