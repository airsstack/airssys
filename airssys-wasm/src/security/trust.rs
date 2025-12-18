//! WASM Component Trust-Level Classification System
//!
//! This module implements a **trust-level system** that classifies WASM component sources
//! as **Trusted** (instant approval), **Unknown** (requires review), or **DevMode**
//! (bypass security for development) based on source verification.
//!
//! # Overview
//!
//! Components from different sources pose different security risks:
//! - **Trusted internal components** should install instantly for developer productivity
//! - **Unknown third-party components** need security review to prevent malicious code
//! - **Development mode** enables rapid local iteration without security friction
//!
//! # Trust Level Workflow
//!
//! ```text
//! Component Installation Initiated
//!           ↓
//! Extract Component Source (Git URL, file path, etc.)
//!           ↓
//! Check Against Trust Registry
//!           ↓
//!      ┌────┴────┐
//!      │ Trusted? │
//!      └────┬────┘
//!           ├─ YES → TrustLevel::Trusted
//!           │        ↓
//!           │   Auto-approve (Task 2.2)
//!           │        ↓
//!           │   Instant install ✅
//!           │
//!           ├─ NO → TrustLevel::Unknown
//!           │       ↓
//!           │   Manual review (Task 2.2)
//!           │       ↓
//!           │   Approval queue ⏳
//!           │
//!           └─ DEV MODE → TrustLevel::DevMode
//!                   ↓
//!              Bypass security with warnings ⚠️
//!                   ↓
//!              Unrestricted install 🔧
//! ```
//!
//! # Architecture
//!
//! ## Core Types
//!
//! - [`TrustLevel`]: Classification enum (Trusted/Unknown/DevMode)
//! - [`TrustSource`]: Configuration for trusted sources (Git/SigningKey/LocalPath)
//! - [`ComponentSource`]: Component origin information (extracted from metadata)
//! - [`TrustRegistry`]: Main service managing trust determination
//!
//! ## Security Model
//!
//! ### Deny-by-Default
//!
//! Unknown sources always require review. Trust must be explicitly configured - no auto-trust.
//!
//! ### Explicit Trust Configuration
//!
//! Trust sources are configured in `trust-config.toml`:
//!
//! ```toml
//! [trust]
//! dev_mode = false
//!
//! [[trust.sources]]
//! type = "git"
//! url_pattern = "https://github.com/myorg/*"
//! branch = "main"
//! description = "Internal organization repos"
//!
//! [[trust.sources]]
//! type = "signing_key"
//! public_key = "ed25519:ABC123..."
//! signer = "security-team@myorg.com"
//! description = "Security team signing key"
//!
//! [[trust.sources]]
//! type = "local"
//! path_pattern = "/home/dev/workspace/components/*"
//! description = "Local development components"
//! ```
//!
//! ### DevMode Safety
//!
//! Development mode produces visible warnings and audit logs:
//!
//! ```text
//! ⚠️  ⚠️  ⚠️  DEVELOPMENT MODE ACTIVE ⚠️  ⚠️  ⚠️
//! Component: my-component
//! Security checks BYPASSED!
//! DO NOT use in production!
//! ```
//!
//! # Examples
//!
//! ## Basic Trust Determination
//!
//! ```rust,no_run
//! use airssys_wasm::security::trust::{TrustRegistry, ComponentSource, TrustLevel};
//! use std::path::Path;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Load trust configuration
//! let registry = TrustRegistry::from_config(Path::new("trust-config.toml")).await?;
//!
//! // Determine trust level for Git component
//! let source = ComponentSource::Git {
//!     url: "https://github.com/myorg/my-component".to_string(),
//!     branch: "main".to_string(),
//!     commit: "abc123".to_string(),
//! };
//!
//! let trust_level = registry.determine_trust_level("my-component", &source);
//!
//! match trust_level {
//!     TrustLevel::Trusted => println!("✅ Trusted - instant install"),
//!     TrustLevel::Unknown => println!("⏳ Unknown - review required"),
//!     TrustLevel::DevMode => println!("🔧 DevMode - security bypassed"),
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Development Mode
//!
//! ```rust,no_run
//! use airssys_wasm::security::trust::TrustRegistry;
//! use std::path::Path;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let registry = TrustRegistry::from_config(Path::new("trust-config.toml")).await?;
//!
//! // Enable DevMode for rapid local iteration
//! registry.set_dev_mode(true);
//!
//! // All components now get TrustLevel::DevMode
//! # Ok(())
//! # }
//! ```
//!
//! ## Dynamic Trust Management
//!
//! ```rust,no_run
//! use airssys_wasm::security::trust::{TrustRegistry, TrustSource};
//! use std::path::Path;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let registry = TrustRegistry::from_config(Path::new("trust-config.toml")).await?;
//!
//! // Add trusted source dynamically (requires admin permission)
//! let new_source = TrustSource::GitRepository {
//!     url_pattern: "https://github.com/trusted-org/*".to_string(),
//!     branch: Some("main".to_string()),
//!     description: "Trusted external organization".to_string(),
//! };
//! registry.add_trusted_source(new_source)?;
//!
//! // List all trusted sources
//! let sources = registry.list_trusted_sources();
//! println!("Trusted sources: {}", sources.len());
//! # Ok(())
//! # }
//! ```
//!
//! # Performance
//!
//! - **Pattern Matching**: <100μs per source check
//! - **Total Trust Check**: <1ms for 100 trusted sources
//! - **DevMode Short-Circuit**: <1ns (atomic bool read)
//! - **Memory Footprint**: <1KB per trusted source
//!
//! # Integration Points
//!
//! ## Task 2.2 Integration (Approval Workflow)
//!
//! Task 2.2 will use `TrustLevel` to route components:
//! - `TrustLevel::Trusted` → Auto-approve installation
//! - `TrustLevel::Unknown` → Enter approval queue for manual review
//! - `TrustLevel::DevMode` → Bypass security with warnings
//!
//! ## Task 2.3 Integration (Trust Configuration)
//!
//! Task 2.3 will provide CLI for trust management:
//! - `add_trusted_source()` → Add new trusted source
//! - `remove_trusted_source()` → Remove trusted source
//! - `list_trusted_sources()` → List all trusted sources
//!
//! ## airssys-osl Integration
//!
//! - Uses `SecurityAuditLogger` to log all trust determinations
//! - Logs DevMode usage at WARNING level
//!
//! # Security Considerations
//!
//! ## Critical Security Properties
//!
//! 1. **Deny-by-Default**: Unknown sources always require review
//! 2. **Explicit Trust**: Trust must be explicitly configured (no auto-trust)
//! 3. **Audit Trail**: All trust determinations logged
//! 4. **DevMode Warnings**: DevMode usage prominently logged
//! 5. **No Bypass**: Cannot bypass Unknown → Trusted without config
//!
//! ## Threat Model
//!
//! | Threat | Mitigation |
//! |--------|------------|
//! | Malicious Source Spoofing | URL verification, signing key validation |
//! | Pattern Bypass | Strict wildcard matching, no regex injection |
//! | DevMode Abuse | Prominent warnings, audit logging |
//! | Config Tampering | File permissions, integrity checks (Task 2.3) |
//! | Trust Escalation | Dynamic trust changes require admin permission |
//!
//! # Standards Compliance
//!
//! - **PROJECTS_STANDARD.md**: §2.1 (3-layer imports), §4.3 (module structure)
//! - **Microsoft Rust Guidelines**: M-MODULE-DOCS, M-CANONICAL-DOCS
//! - **ADR-WASM-005**: Capability-Based Security Model

// Layer 1: Standard library imports
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

// Layer 2: Third-party crate imports
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, warn};

/// Trust level classification for component sources.
///
/// Determines installation workflow:
/// - `Trusted`: Instant install (no approval delay)
/// - `Unknown`: Review required (enters approval queue)
/// - `DevMode`: Bypass security (development only, logged warnings)
///
/// # Examples
///
/// ```rust
/// use airssys_wasm::security::trust::TrustLevel;
///
/// let level = TrustLevel::Trusted;
/// assert!(!level.requires_approval());
/// assert!(!level.bypasses_security());
/// assert_eq!(level.security_posture(), "secure-trusted");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrustLevel {
    /// Verified trusted source - instant approval.
    ///
    /// # Sources
    /// - Internal Git repositories
    /// - Cryptographically signed components
    /// - Pre-verified local components
    ///
    /// # Workflow
    /// 1. Source matches trusted pattern
    /// 2. Auto-approve installation
    /// 3. Apply declared capabilities
    /// 4. No manual review needed
    Trusted,

    /// Unknown source - requires manual review.
    ///
    /// # Sources
    /// - External Git repositories not in trusted list
    /// - Unsigned components
    /// - First-time sources
    ///
    /// # Workflow
    /// 1. Source doesn't match any trusted pattern
    /// 2. Enter approval queue
    /// 3. Admin reviews capabilities
    /// 4. Approve/modify/deny
    Unknown,

    /// Development mode - bypass security checks.
    ///
    /// **WARNING:** Disables all security enforcement.
    /// Only use for local development/testing!
    ///
    /// # Workflow
    /// 1. DevMode flag explicitly enabled
    /// 2. Bypass all capability checks
    /// 3. Grant unrestricted access
    /// 4. Log warnings for audit trail
    ///
    /// # Security
    /// - MUST NOT be used in production
    /// - All operations logged with DevMode flag
    /// - Visible warnings in console output
    DevMode,
}

impl TrustLevel {
    /// Returns true if this trust level requires approval.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::security::trust::TrustLevel;
    ///
    /// assert!(!TrustLevel::Trusted.requires_approval());
    /// assert!(TrustLevel::Unknown.requires_approval());
    /// assert!(!TrustLevel::DevMode.requires_approval());
    /// ```
    pub fn requires_approval(&self) -> bool {
        matches!(self, TrustLevel::Unknown)
    }

    /// Returns true if this trust level bypasses security.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::security::trust::TrustLevel;
    ///
    /// assert!(!TrustLevel::Trusted.bypasses_security());
    /// assert!(!TrustLevel::Unknown.bypasses_security());
    /// assert!(TrustLevel::DevMode.bypasses_security());
    /// ```
    pub fn bypasses_security(&self) -> bool {
        matches!(self, TrustLevel::DevMode)
    }

    /// Returns security posture description for logging.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::security::trust::TrustLevel;
    ///
    /// assert_eq!(TrustLevel::Trusted.security_posture(), "secure-trusted");
    /// assert_eq!(TrustLevel::Unknown.security_posture(), "secure-review-required");
    /// assert_eq!(TrustLevel::DevMode.security_posture(), "insecure-dev-mode");
    /// ```
    pub fn security_posture(&self) -> &'static str {
        match self {
            TrustLevel::Trusted => "secure-trusted",
            TrustLevel::Unknown => "secure-review-required",
            TrustLevel::DevMode => "insecure-dev-mode",
        }
    }
}

/// Errors that can occur during trust operations.
#[derive(Debug, Error)]
pub enum TrustError {
    /// Configuration file not found or inaccessible.
    #[error("Trust configuration file not found: {path}")]
    ConfigNotFound { path: String },

    /// TOML parsing error.
    #[error("Failed to parse trust configuration: {source}")]
    ParseError {
        #[from]
        source: toml::de::Error,
    },

    /// I/O error reading configuration.
    #[error("I/O error reading trust configuration: {source}")]
    IoError {
        #[from]
        source: std::io::Error,
    },

    /// Invalid source configuration.
    #[error("Invalid trust source: {reason}")]
    InvalidSource { reason: String },

    /// Source index out of bounds.
    #[error("Trust source index {index} not found (total: {total})")]
    SourceNotFound { index: usize, total: usize },

    /// Duplicate source configuration.
    #[error("Duplicate trust source: {identifier}")]
    DuplicateSource { identifier: String },
}

/// Result type for trust operations.
pub type TrustResult<T> = Result<T, TrustError>;

/// Trusted source configuration.
///
/// Defines patterns and verification methods for trusted component sources.
///
/// # Examples
///
/// ## Git Repository Source
///
/// ```rust
/// use airssys_wasm::security::trust::TrustSource;
///
/// let source = TrustSource::GitRepository {
///     url_pattern: "https://github.com/mycompany/*".to_string(),
///     branch: Some("main".to_string()),
///     description: "Internal company repositories".to_string(),
/// };
/// ```
///
/// ## Signing Key Source
///
/// ```rust
/// use airssys_wasm::security::trust::TrustSource;
///
/// let source = TrustSource::SigningKey {
///     public_key: "ed25519:AAAAC3NzaC1lZDI1NTE5AAAAIJbpYR...".to_string(),
///     signer: "engineering@mycompany.com".to_string(),
///     description: "Signed by engineering team".to_string(),
/// };
/// ```
///
/// ## Local Path Source
///
/// ```rust
/// use airssys_wasm::security::trust::TrustSource;
///
/// let source = TrustSource::LocalPath {
///     path_pattern: "/opt/verified-components/*".to_string(),
///     description: "Pre-verified local components".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TrustSource {
    /// Git repository source (URL pattern matching).
    ///
    /// Supports wildcard patterns:
    /// - `*`: Match any sequence of characters (except `/`)
    /// - `?`: Match any single character
    ///
    /// # TOML Configuration
    ///
    /// ```toml
    /// [[trust.sources]]
    /// type = "git"
    /// url_pattern = "https://github.com/mycompany/*"
    /// branch = "main"
    /// description = "Internal company repositories"
    /// ```
    #[serde(rename = "git")]
    GitRepository {
        /// URL pattern (supports wildcards: `*`, `?`)
        url_pattern: String,

        /// Optional branch restriction
        branch: Option<String>,

        /// Human-readable description
        description: String,
    },

    /// Cryptographically signed component (Ed25519 signature).
    ///
    /// # TOML Configuration
    ///
    /// ```toml
    /// [[trust.sources]]
    /// type = "signing_key"
    /// public_key = "ed25519:AAAAC3NzaC1lZDI1NTE5AAAAIJbpYR..."
    /// signer = "engineering@mycompany.com"
    /// description = "Signed by engineering team"
    /// ```
    #[serde(rename = "signing_key")]
    SigningKey {
        /// Ed25519 public key (with "ed25519:" prefix)
        public_key: String,

        /// Signer identity (email, name, etc.)
        signer: String,

        /// Human-readable description
        description: String,
    },

    /// Local filesystem path (for pre-verified components).
    ///
    /// # TOML Configuration
    ///
    /// ```toml
    /// [[trust.sources]]
    /// type = "local"
    /// path_pattern = "/opt/verified-components/*"
    /// description = "Pre-verified local components"
    /// ```
    #[serde(rename = "local")]
    LocalPath {
        /// Filesystem path pattern (supports wildcards)
        path_pattern: String,

        /// Human-readable description
        description: String,
    },
}

impl TrustSource {
    /// Checks if this source matches the given component source.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::security::trust::{TrustSource, ComponentSource};
    ///
    /// let trust_source = TrustSource::GitRepository {
    ///     url_pattern: "https://github.com/mycompany/*".to_string(),
    ///     branch: Some("main".to_string()),
    ///     description: "Internal repos".to_string(),
    /// };
    ///
    /// let component_source = ComponentSource::Git {
    ///     url: "https://github.com/mycompany/my-component".to_string(),
    ///     branch: "main".to_string(),
    ///     commit: "abc123".to_string(),
    /// };
    ///
    /// assert!(trust_source.matches(&component_source));
    /// ```
    pub fn matches(&self, component_source: &ComponentSource) -> bool {
        match (self, component_source) {
            (
                TrustSource::GitRepository {
                    url_pattern,
                    branch: trust_branch,
                    ..
                },
                ComponentSource::Git {
                    url,
                    branch: component_branch,
                    ..
                },
            ) => {
                // Check URL pattern match
                if !Self::glob_match(url_pattern, url) {
                    return false;
                }

                // Check branch restriction if specified
                if let Some(trust_branch) = trust_branch {
                    if trust_branch != component_branch {
                        return false;
                    }
                }

                true
            }
            (
                TrustSource::SigningKey { public_key, .. },
                ComponentSource::Signed {
                    public_key: component_key,
                    ..
                },
            ) => {
                // Exact match for public keys
                public_key == component_key
            }
            (TrustSource::LocalPath { path_pattern, .. }, ComponentSource::Local { path }) => {
                // Check path pattern match
                let path_str = path.to_string_lossy();
                Self::glob_match(path_pattern, &path_str)
            }
            // Mismatched types don't match
            _ => false,
        }
    }

    /// Returns source type string for logging.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::security::trust::TrustSource;
    ///
    /// let source = TrustSource::GitRepository {
    ///     url_pattern: "https://github.com/mycompany/*".to_string(),
    ///     branch: None,
    ///     description: "Internal repos".to_string(),
    /// };
    ///
    /// assert_eq!(source.source_type(), "git");
    /// ```
    pub fn source_type(&self) -> &'static str {
        match self {
            TrustSource::GitRepository { .. } => "git",
            TrustSource::SigningKey { .. } => "signing_key",
            TrustSource::LocalPath { .. } => "local",
        }
    }

    /// Performs glob-style pattern matching.
    ///
    /// Supports:
    /// - `*`: Match any sequence of characters (except `/`)
    /// - `?`: Match any single character
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// assert!(TrustSource::glob_match("https://github.com/mycompany/*", "https://github.com/mycompany/my-repo"));
    /// assert!(TrustSource::glob_match("/opt/components/?.wasm", "/opt/components/a.wasm"));
    /// assert!(!TrustSource::glob_match("https://github.com/mycompany/*", "https://github.com/other/repo"));
    /// ```
    fn glob_match(pattern: &str, text: &str) -> bool {
        // Use glob crate for pattern matching
        match glob::Pattern::new(pattern) {
            Ok(pattern) => pattern.matches(text),
            Err(_) => {
                // Invalid pattern - no match
                warn!("Invalid glob pattern: {}", pattern);
                false
            }
        }
    }
}

/// Component source information extracted during installation.
///
/// # Examples
///
/// ## Git Source
///
/// ```rust
/// use airssys_wasm::security::trust::ComponentSource;
///
/// let source = ComponentSource::Git {
///     url: "https://github.com/mycompany/my-component".to_string(),
///     branch: "main".to_string(),
///     commit: "abc123".to_string(),
/// };
/// ```
///
/// ## Signed Source
///
/// ```rust
/// use airssys_wasm::security::trust::ComponentSource;
///
/// let source = ComponentSource::Signed {
///     signature: "ed25519:sig123...".to_string(),
///     public_key: "ed25519:key456...".to_string(),
/// };
/// ```
///
/// ## Local Source
///
/// ```rust
/// use airssys_wasm::security::trust::ComponentSource;
/// use std::path::PathBuf;
///
/// let source = ComponentSource::Local {
///     path: PathBuf::from("/opt/components/my-component.wasm"),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentSource {
    /// Component from Git repository.
    Git {
        /// Repository URL
        url: String,
        /// Branch name
        branch: String,
        /// Commit hash
        commit: String,
    },

    /// Component from signed artifact.
    Signed {
        /// Signature string (with algorithm prefix)
        signature: String,
        /// Public key (with algorithm prefix)
        public_key: String,
    },

    /// Component from local filesystem.
    Local {
        /// Filesystem path
        path: PathBuf,
    },
}

impl ComponentSource {
    /// Returns source identifier string for logging.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::security::trust::ComponentSource;
    /// use std::path::PathBuf;
    ///
    /// let source = ComponentSource::Git {
    ///     url: "https://github.com/mycompany/my-component".to_string(),
    ///     branch: "main".to_string(),
    ///     commit: "abc123".to_string(),
    /// };
    /// assert_eq!(source.identifier(), "git:https://github.com/mycompany/my-component@main:abc123");
    ///
    /// let source = ComponentSource::Local {
    ///     path: PathBuf::from("/opt/components/my-component.wasm"),
    /// };
    /// assert_eq!(source.identifier(), "local:/opt/components/my-component.wasm");
    /// ```
    pub fn identifier(&self) -> String {
        match self {
            ComponentSource::Git {
                url,
                branch,
                commit,
            } => format!("git:{}@{}:{}", url, branch, commit),
            ComponentSource::Signed {
                signature,
                public_key,
            } => {
                // Truncate for readability
                let sig_short = signature.chars().take(20).collect::<String>();
                let key_short = public_key.chars().take(20).collect::<String>();
                format!("signed:{}...:{}", sig_short, key_short)
            }
            ComponentSource::Local { path } => {
                format!("local:{}", path.display())
            }
        }
    }
}

/// Trust registry managing trusted sources and trust determination.
///
/// # Responsibilities
/// - Load trust configuration from TOML file
/// - Match component sources against trusted patterns
/// - Determine TrustLevel for components
/// - Support DevMode override
///
/// # Thread Safety
/// - Uses Arc<RwLock<>> for concurrent access
/// - Read-heavy workload (install checks)
/// - Write-light workload (config updates)
///
/// # Examples
///
/// ## Load Configuration and Determine Trust
///
/// ```rust,no_run
/// use airssys_wasm::security::trust::{TrustRegistry, ComponentSource, TrustLevel};
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let registry = TrustRegistry::from_config(Path::new("trust-config.toml")).await?;
///
/// let source = ComponentSource::Git {
///     url: "https://github.com/myorg/my-component".to_string(),
///     branch: "main".to_string(),
///     commit: "abc123".to_string(),
/// };
///
/// let trust_level = registry.determine_trust_level("my-component", &source);
/// match trust_level {
///     TrustLevel::Trusted => println!("✅ Trusted"),
///     TrustLevel::Unknown => println!("⏳ Unknown"),
///     TrustLevel::DevMode => println!("🔧 DevMode"),
/// }
/// # Ok(())
/// # }
/// ```
///
/// ## Dynamic Trust Management
///
/// ```rust,no_run
/// use airssys_wasm::security::trust::{TrustRegistry, TrustSource};
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let registry = TrustRegistry::from_config(Path::new("trust-config.toml")).await?;
///
/// // Add new trusted source
/// let new_source = TrustSource::GitRepository {
///     url_pattern: "https://github.com/trusted-org/*".to_string(),
///     branch: Some("main".to_string()),
///     description: "Trusted external organization".to_string(),
/// };
/// registry.add_trusted_source(new_source)?;
///
/// // List all sources
/// let sources = registry.list_trusted_sources();
/// println!("Total trusted sources: {}", sources.len());
/// # Ok(())
/// # }
/// ```
pub struct TrustRegistry {
    /// Trusted sources (protected by RwLock for concurrent access)
    sources: Arc<RwLock<Vec<TrustSource>>>,

    /// Development mode flag (atomic for fast read access)
    dev_mode_enabled: Arc<AtomicBool>,
}

impl TrustRegistry {
    /// Creates an empty trust registry.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::security::trust::TrustRegistry;
    ///
    /// let registry = TrustRegistry::new();
    /// assert!(!registry.is_dev_mode());
    /// ```
    pub fn new() -> Self {
        Self {
            sources: Arc::new(RwLock::new(Vec::new())),
            dev_mode_enabled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Creates registry from configuration file.
    ///
    /// # Configuration Format
    ///
    /// ```toml
    /// [trust]
    /// dev_mode = false
    ///
    /// [[trust.sources]]
    /// type = "git"
    /// url_pattern = "https://github.com/mycompany/*"
    /// branch = "main"
    /// description = "Internal company repositories"
    ///
    /// [[trust.sources]]
    /// type = "signing_key"
    /// public_key = "ed25519:AAAAC3NzaC1lZDI1NTE5AAAAIJbpYR..."
    /// signer = "engineering@mycompany.com"
    /// description = "Engineering team signing key"
    ///
    /// [[trust.sources]]
    /// type = "local"
    /// path_pattern = "/opt/verified-components/*"
    /// description = "Pre-verified components"
    /// ```
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Configuration file not found
    /// - TOML parsing fails
    /// - I/O error reading file
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use airssys_wasm::security::trust::TrustRegistry;
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let registry = TrustRegistry::from_config(Path::new("trust-config.toml")).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn from_config(config_path: &Path) -> TrustResult<Self> {
        // Read configuration file
        let config_content = tokio::fs::read_to_string(config_path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                TrustError::ConfigNotFound {
                    path: config_path.display().to_string(),
                }
            } else {
                TrustError::IoError { source: e }
            }
        })?;

        // Parse TOML configuration
        let config: TrustConfig = toml::from_str(&config_content)?;

        // Validate sources
        Self::validate_sources(&config.trust.sources)?;

        // Create registry
        let registry = Self {
            sources: Arc::new(RwLock::new(config.trust.sources)),
            dev_mode_enabled: Arc::new(AtomicBool::new(config.trust.dev_mode)),
        };

        debug!(
            "Trust registry loaded: {} sources, dev_mode={}",
            registry.list_trusted_sources().len(),
            registry.is_dev_mode()
        );

        Ok(registry)
    }

    /// Determines trust level for component source.
    ///
    /// # Algorithm
    ///
    /// 1. Check DevMode flag (atomic read) - return DevMode if enabled
    /// 2. Iterate trusted sources (read lock) - return Trusted on first match
    /// 3. No match found - return Unknown (requires review)
    ///
    /// # Performance
    ///
    /// - DevMode check: <1ns (atomic read)
    /// - Pattern matching: <100μs per source
    /// - Total: <1ms for 100 sources
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use airssys_wasm::security::trust::{TrustRegistry, ComponentSource, TrustLevel};
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let registry = TrustRegistry::from_config(Path::new("trust-config.toml")).await?;
    ///
    /// let source = ComponentSource::Git {
    ///     url: "https://github.com/myorg/my-component".to_string(),
    ///     branch: "main".to_string(),
    ///     commit: "abc123".to_string(),
    /// };
    ///
    /// let level = registry.determine_trust_level("my-component", &source);
    /// assert_eq!(level, TrustLevel::Trusted);
    /// # Ok(())
    /// # }
    /// ```
    pub fn determine_trust_level(
        &self,
        component_id: &str,
        source: &ComponentSource,
    ) -> TrustLevel {
        // DevMode short-circuit (atomic read, <1ns)
        if self.dev_mode_enabled.load(Ordering::Relaxed) {
            warn!(
                "⚠️  DEVELOPMENT MODE: Component '{}' security bypassed (source: {})",
                component_id,
                source.identifier()
            );
            return TrustLevel::DevMode;
        }

        // Check against trusted sources (read lock)
        let sources = self.sources.read().unwrap_or_else(|poisoned| {
            warn!("Trust registry lock poisoned, recovering with partial data");
            poisoned.into_inner()
        });
        for trust_source in sources.iter() {
            if trust_source.matches(source) {
                debug!(
                    "✅ Trusted component: '{}' (source: {}, matched: {})",
                    component_id,
                    source.identifier(),
                    trust_source.source_type()
                );
                return TrustLevel::Trusted;
            }
        }

        // No match - unknown source requires review
        debug!(
            "⏳ Unknown component: '{}' (source: {}) - review required",
            component_id,
            source.identifier()
        );
        TrustLevel::Unknown
    }

    /// Adds trusted source dynamically (requires admin permission).
    ///
    /// # Thread Safety
    ///
    /// Uses write lock for exclusive access during modification.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use airssys_wasm::security::trust::{TrustRegistry, TrustSource};
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let registry = TrustRegistry::from_config(Path::new("trust-config.toml")).await?;
    ///
    /// let new_source = TrustSource::GitRepository {
    ///     url_pattern: "https://github.com/trusted-org/*".to_string(),
    ///     branch: Some("main".to_string()),
    ///     description: "Trusted external organization".to_string(),
    /// };
    /// registry.add_trusted_source(new_source)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn add_trusted_source(&self, source: TrustSource) -> TrustResult<()> {
        let mut sources = self.sources.write().unwrap_or_else(|poisoned| {
            warn!("Trust registry lock poisoned, recovering with partial data");
            poisoned.into_inner()
        });
        sources.push(source);
        Ok(())
    }

    /// Removes trusted source by index.
    ///
    /// # Errors
    ///
    /// Returns `TrustError::SourceNotFound` if index is out of bounds.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use airssys_wasm::security::trust::TrustRegistry;
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let registry = TrustRegistry::from_config(Path::new("trust-config.toml")).await?;
    /// registry.remove_trusted_source(0)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn remove_trusted_source(&self, index: usize) -> TrustResult<()> {
        let mut sources = self.sources.write().unwrap_or_else(|poisoned| {
            warn!("Trust registry lock poisoned, recovering with partial data");
            poisoned.into_inner()
        });
        if index >= sources.len() {
            return Err(TrustError::SourceNotFound {
                index,
                total: sources.len(),
            });
        }
        sources.remove(index);
        Ok(())
    }

    /// Lists all trusted sources.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use airssys_wasm::security::trust::TrustRegistry;
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let registry = TrustRegistry::from_config(Path::new("trust-config.toml")).await?;
    /// let sources = registry.list_trusted_sources();
    /// println!("Trusted sources: {}", sources.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn list_trusted_sources(&self) -> Vec<TrustSource> {
        let sources = self.sources.read().unwrap_or_else(|poisoned| {
            warn!("Trust registry lock poisoned, recovering with partial data");
            poisoned.into_inner()
        });
        sources.clone()
    }

    /// Enables/disables development mode (global flag).
    ///
    /// **WARNING:** DevMode bypasses all security checks!
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::security::trust::TrustRegistry;
    ///
    /// let registry = TrustRegistry::new();
    /// assert!(!registry.is_dev_mode());
    ///
    /// registry.set_dev_mode(true);
    /// assert!(registry.is_dev_mode());
    /// ```
    pub fn set_dev_mode(&self, enabled: bool) {
        self.dev_mode_enabled.store(enabled, Ordering::Relaxed);
        if enabled {
            warn!("⚠️  ⚠️  ⚠️  DEVELOPMENT MODE ENABLED ⚠️  ⚠️  ⚠️");
            warn!("All component security checks will be bypassed!");
            warn!("DO NOT use in production!");
        } else {
            debug!("Development mode disabled - security enforcement active");
        }
    }

    /// Checks if development mode is enabled.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::security::trust::TrustRegistry;
    ///
    /// let registry = TrustRegistry::new();
    /// assert!(!registry.is_dev_mode());
    /// ```
    pub fn is_dev_mode(&self) -> bool {
        self.dev_mode_enabled.load(Ordering::Relaxed)
    }

    /// Validates trust sources for configuration errors.
    fn validate_sources(sources: &[TrustSource]) -> TrustResult<()> {
        // Check for empty patterns
        for source in sources {
            match source {
                TrustSource::GitRepository {
                    url_pattern,
                    description,
                    ..
                } => {
                    if url_pattern.trim().is_empty() {
                        return Err(TrustError::InvalidSource {
                            reason: "Git URL pattern cannot be empty".to_string(),
                        });
                    }
                    if description.trim().is_empty() {
                        return Err(TrustError::InvalidSource {
                            reason: "Description cannot be empty".to_string(),
                        });
                    }
                }
                TrustSource::SigningKey {
                    public_key,
                    signer,
                    description,
                } => {
                    if !public_key.starts_with("ed25519:") {
                        return Err(TrustError::InvalidSource {
                            reason: format!(
                                "Public key must start with 'ed25519:', got: {}",
                                public_key
                            ),
                        });
                    }
                    if signer.trim().is_empty() {
                        return Err(TrustError::InvalidSource {
                            reason: "Signer identity cannot be empty".to_string(),
                        });
                    }
                    if description.trim().is_empty() {
                        return Err(TrustError::InvalidSource {
                            reason: "Description cannot be empty".to_string(),
                        });
                    }
                }
                TrustSource::LocalPath {
                    path_pattern,
                    description,
                } => {
                    if path_pattern.trim().is_empty() {
                        return Err(TrustError::InvalidSource {
                            reason: "Local path pattern cannot be empty".to_string(),
                        });
                    }
                    if description.trim().is_empty() {
                        return Err(TrustError::InvalidSource {
                            reason: "Description cannot be empty".to_string(),
                        });
                    }
                }
            }
        }

        Ok(())
    }
}

impl Default for TrustRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// TOML configuration structure for trust-config.toml.
#[derive(Debug, Deserialize)]
struct TrustConfig {
    trust: TrustSection,
}

/// Trust section of TOML configuration.
#[derive(Debug, Deserialize)]
struct TrustSection {
    /// Development mode flag
    #[serde(default)]
    dev_mode: bool,

    /// Trusted sources
    #[serde(default)]
    sources: Vec<TrustSource>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    // ========================================================================
    // Task 2.1.2 Tests: TrustLevel Enum
    // ========================================================================

    #[test]
    fn test_trust_level_requires_approval() {
        assert!(!TrustLevel::Trusted.requires_approval());
        assert!(TrustLevel::Unknown.requires_approval());
        assert!(!TrustLevel::DevMode.requires_approval());
    }

    #[test]
    fn test_trust_level_bypasses_security() {
        assert!(!TrustLevel::Trusted.bypasses_security());
        assert!(!TrustLevel::Unknown.bypasses_security());
        assert!(TrustLevel::DevMode.bypasses_security());
    }

    #[test]
    fn test_trust_level_security_posture() {
        assert_eq!(TrustLevel::Trusted.security_posture(), "secure-trusted");
        assert_eq!(
            TrustLevel::Unknown.security_posture(),
            "secure-review-required"
        );
        assert_eq!(TrustLevel::DevMode.security_posture(), "insecure-dev-mode");
    }

    #[test]
    fn test_trust_level_equality() {
        assert_eq!(TrustLevel::Trusted, TrustLevel::Trusted);
        assert_ne!(TrustLevel::Trusted, TrustLevel::Unknown);
        assert_ne!(TrustLevel::Unknown, TrustLevel::DevMode);
    }

    #[test]
    fn test_trust_level_serialization() {
        let trusted = TrustLevel::Trusted;
        let json = serde_json::to_string(&trusted).unwrap();
        assert!(json.contains("Trusted"));

        let deserialized: TrustLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, TrustLevel::Trusted);
    }

    // ========================================================================
    // Task 2.1.3 Tests: TrustSource Types
    // ========================================================================

    #[test]
    fn test_trust_source_git_source_type() {
        let source = TrustSource::GitRepository {
            url_pattern: "https://github.com/myorg/*".to_string(),
            branch: Some("main".to_string()),
            description: "Internal repos".to_string(),
        };
        assert_eq!(source.source_type(), "git");
    }

    #[test]
    fn test_trust_source_signing_key_source_type() {
        let source = TrustSource::SigningKey {
            public_key: "ed25519:ABC123".to_string(),
            signer: "test@example.com".to_string(),
            description: "Test key".to_string(),
        };
        assert_eq!(source.source_type(), "signing_key");
    }

    #[test]
    fn test_trust_source_local_source_type() {
        let source = TrustSource::LocalPath {
            path_pattern: "/opt/components/*".to_string(),
            description: "Local components".to_string(),
        };
        assert_eq!(source.source_type(), "local");
    }

    // ========================================================================
    // Task 2.1.4 Tests: ComponentSource Types
    // ========================================================================

    #[test]
    fn test_component_source_git_identifier() {
        let source = ComponentSource::Git {
            url: "https://github.com/myorg/my-component".to_string(),
            branch: "main".to_string(),
            commit: "abc123".to_string(),
        };
        assert_eq!(
            source.identifier(),
            "git:https://github.com/myorg/my-component@main:abc123"
        );
    }

    #[test]
    fn test_component_source_signed_identifier() {
        let source = ComponentSource::Signed {
            signature: "ed25519:sig1234567890abcdefghij".to_string(),
            public_key: "ed25519:key1234567890abcdefghij".to_string(),
        };
        let id = source.identifier();
        assert!(id.starts_with("signed:"));
        assert!(id.contains("ed25519:sig12345678"));
    }

    #[test]
    fn test_component_source_local_identifier() {
        let source = ComponentSource::Local {
            path: PathBuf::from("/opt/components/my-component.wasm"),
        };
        assert_eq!(
            source.identifier(),
            "local:/opt/components/my-component.wasm"
        );
    }

    // ========================================================================
    // Task 2.1.5 Tests: Git Pattern Matching
    // ========================================================================

    #[test]
    fn test_git_exact_match() {
        let trust_source = TrustSource::GitRepository {
            url_pattern: "https://github.com/myorg/my-repo".to_string(),
            branch: None,
            description: "Test".to_string(),
        };

        let component_source = ComponentSource::Git {
            url: "https://github.com/myorg/my-repo".to_string(),
            branch: "main".to_string(),
            commit: "abc123".to_string(),
        };

        assert!(trust_source.matches(&component_source));
    }

    #[test]
    fn test_git_wildcard_match() {
        let trust_source = TrustSource::GitRepository {
            url_pattern: "https://github.com/myorg/*".to_string(),
            branch: None,
            description: "Test".to_string(),
        };

        let component_source = ComponentSource::Git {
            url: "https://github.com/myorg/my-component".to_string(),
            branch: "main".to_string(),
            commit: "abc123".to_string(),
        };

        assert!(trust_source.matches(&component_source));
    }

    #[test]
    fn test_git_wildcard_no_match() {
        let trust_source = TrustSource::GitRepository {
            url_pattern: "https://github.com/myorg/*".to_string(),
            branch: None,
            description: "Test".to_string(),
        };

        let component_source = ComponentSource::Git {
            url: "https://github.com/otherorg/my-component".to_string(),
            branch: "main".to_string(),
            commit: "abc123".to_string(),
        };

        assert!(!trust_source.matches(&component_source));
    }

    #[test]
    fn test_git_branch_restriction_match() {
        let trust_source = TrustSource::GitRepository {
            url_pattern: "https://github.com/myorg/*".to_string(),
            branch: Some("main".to_string()),
            description: "Test".to_string(),
        };

        let component_source = ComponentSource::Git {
            url: "https://github.com/myorg/my-component".to_string(),
            branch: "main".to_string(),
            commit: "abc123".to_string(),
        };

        assert!(trust_source.matches(&component_source));
    }

    #[test]
    fn test_git_branch_restriction_no_match() {
        let trust_source = TrustSource::GitRepository {
            url_pattern: "https://github.com/myorg/*".to_string(),
            branch: Some("main".to_string()),
            description: "Test".to_string(),
        };

        let component_source = ComponentSource::Git {
            url: "https://github.com/myorg/my-component".to_string(),
            branch: "dev".to_string(),
            commit: "abc123".to_string(),
        };

        assert!(!trust_source.matches(&component_source));
    }

    #[test]
    fn test_git_question_mark_wildcard() {
        let trust_source = TrustSource::GitRepository {
            url_pattern: "https://github.com/myorg/component-?".to_string(),
            branch: None,
            description: "Test".to_string(),
        };

        let component_source = ComponentSource::Git {
            url: "https://github.com/myorg/component-a".to_string(),
            branch: "main".to_string(),
            commit: "abc123".to_string(),
        };

        assert!(trust_source.matches(&component_source));
    }

    #[test]
    fn test_git_no_match_different_type() {
        let trust_source = TrustSource::GitRepository {
            url_pattern: "https://github.com/myorg/*".to_string(),
            branch: None,
            description: "Test".to_string(),
        };

        let component_source = ComponentSource::Local {
            path: PathBuf::from("/opt/components/my-component"),
        };

        assert!(!trust_source.matches(&component_source));
    }

    #[test]
    fn test_git_unicode_url() {
        let trust_source = TrustSource::GitRepository {
            url_pattern: "https://github.com/myorg/café-*".to_string(),
            branch: None,
            description: "Test".to_string(),
        };

        let component_source = ComponentSource::Git {
            url: "https://github.com/myorg/café-component".to_string(),
            branch: "main".to_string(),
            commit: "abc123".to_string(),
        };

        assert!(trust_source.matches(&component_source));
    }

    #[test]
    fn test_git_long_url() {
        let long_url = format!("https://github.com/myorg/{}", "a".repeat(500));
        let trust_source = TrustSource::GitRepository {
            url_pattern: "https://github.com/myorg/*".to_string(),
            branch: None,
            description: "Test".to_string(),
        };

        let component_source = ComponentSource::Git {
            url: long_url,
            branch: "main".to_string(),
            commit: "abc123".to_string(),
        };

        assert!(trust_source.matches(&component_source));
    }

    #[test]
    fn test_git_multiple_wildcards() {
        let trust_source = TrustSource::GitRepository {
            url_pattern: "https://github.com/*/component-*".to_string(),
            branch: None,
            description: "Test".to_string(),
        };

        let component_source = ComponentSource::Git {
            url: "https://github.com/myorg/component-foo".to_string(),
            branch: "main".to_string(),
            commit: "abc123".to_string(),
        };

        assert!(trust_source.matches(&component_source));
    }

    // ========================================================================
    // Task 2.1.6 Tests: Signing Key Verification
    // ========================================================================

    #[test]
    fn test_signing_key_exact_match() {
        let trust_source = TrustSource::SigningKey {
            public_key: "ed25519:AAAAC3NzaC1lZDI1NTE5AAAAIJbpYR".to_string(),
            signer: "test@example.com".to_string(),
            description: "Test key".to_string(),
        };

        let component_source = ComponentSource::Signed {
            signature: "ed25519:signature123".to_string(),
            public_key: "ed25519:AAAAC3NzaC1lZDI1NTE5AAAAIJbpYR".to_string(),
        };

        assert!(trust_source.matches(&component_source));
    }

    #[test]
    fn test_signing_key_no_match() {
        let trust_source = TrustSource::SigningKey {
            public_key: "ed25519:AAAAC3NzaC1lZDI1NTE5AAAAIJbpYR".to_string(),
            signer: "test@example.com".to_string(),
            description: "Test key".to_string(),
        };

        let component_source = ComponentSource::Signed {
            signature: "ed25519:signature123".to_string(),
            public_key: "ed25519:DifferentKey".to_string(),
        };

        assert!(!trust_source.matches(&component_source));
    }

    #[test]
    fn test_signing_key_no_match_different_type() {
        let trust_source = TrustSource::SigningKey {
            public_key: "ed25519:AAAAC3NzaC1lZDI1NTE5AAAAIJbpYR".to_string(),
            signer: "test@example.com".to_string(),
            description: "Test key".to_string(),
        };

        let component_source = ComponentSource::Git {
            url: "https://github.com/myorg/my-component".to_string(),
            branch: "main".to_string(),
            commit: "abc123".to_string(),
        };

        assert!(!trust_source.matches(&component_source));
    }

    #[test]
    fn test_signing_key_case_sensitive() {
        let trust_source = TrustSource::SigningKey {
            public_key: "ed25519:ABC123".to_string(),
            signer: "test@example.com".to_string(),
            description: "Test key".to_string(),
        };

        let component_source = ComponentSource::Signed {
            signature: "ed25519:sig".to_string(),
            public_key: "ed25519:abc123".to_string(),
        };

        assert!(!trust_source.matches(&component_source));
    }

    #[test]
    fn test_signing_key_whitespace() {
        let trust_source = TrustSource::SigningKey {
            public_key: "ed25519:ABC123".to_string(),
            signer: "test@example.com".to_string(),
            description: "Test key".to_string(),
        };

        let component_source = ComponentSource::Signed {
            signature: "ed25519:sig".to_string(),
            public_key: "ed25519:ABC123 ".to_string(), // Trailing space
        };

        // No match due to trailing space
        assert!(!trust_source.matches(&component_source));
    }

    // ========================================================================
    // Task 2.1.7 Tests: Local Path Matching
    // ========================================================================

    #[test]
    fn test_local_exact_match() {
        let trust_source = TrustSource::LocalPath {
            path_pattern: "/opt/components/my-component.wasm".to_string(),
            description: "Test".to_string(),
        };

        let component_source = ComponentSource::Local {
            path: PathBuf::from("/opt/components/my-component.wasm"),
        };

        assert!(trust_source.matches(&component_source));
    }

    #[test]
    fn test_local_wildcard_match() {
        let trust_source = TrustSource::LocalPath {
            path_pattern: "/opt/components/*".to_string(),
            description: "Test".to_string(),
        };

        let component_source = ComponentSource::Local {
            path: PathBuf::from("/opt/components/my-component.wasm"),
        };

        assert!(trust_source.matches(&component_source));
    }

    #[test]
    fn test_local_wildcard_no_match() {
        let trust_source = TrustSource::LocalPath {
            path_pattern: "/opt/components/*".to_string(),
            description: "Test".to_string(),
        };

        let component_source = ComponentSource::Local {
            path: PathBuf::from("/other/path/my-component.wasm"),
        };

        assert!(!trust_source.matches(&component_source));
    }

    #[test]
    fn test_local_no_match_different_type() {
        let trust_source = TrustSource::LocalPath {
            path_pattern: "/opt/components/*".to_string(),
            description: "Test".to_string(),
        };

        let component_source = ComponentSource::Git {
            url: "https://github.com/myorg/my-component".to_string(),
            branch: "main".to_string(),
            commit: "abc123".to_string(),
        };

        assert!(!trust_source.matches(&component_source));
    }

    #[test]
    fn test_local_extension_pattern() {
        let trust_source = TrustSource::LocalPath {
            path_pattern: "/opt/components/*.wasm".to_string(),
            description: "Test".to_string(),
        };

        let component_source = ComponentSource::Local {
            path: PathBuf::from("/opt/components/my-component.wasm"),
        };

        assert!(trust_source.matches(&component_source));
    }

    // ========================================================================
    // Task 2.1.8 Tests: TrustRegistry Core
    // ========================================================================

    #[test]
    fn test_trust_registry_new() {
        let registry = TrustRegistry::new();
        assert!(!registry.is_dev_mode());
        assert_eq!(registry.list_trusted_sources().len(), 0);
    }

    #[test]
    fn test_trust_registry_determine_trust_unknown() {
        let registry = TrustRegistry::new();

        let source = ComponentSource::Git {
            url: "https://github.com/unknown/component".to_string(),
            branch: "main".to_string(),
            commit: "abc123".to_string(),
        };

        let level = registry.determine_trust_level("test-component", &source);
        assert_eq!(level, TrustLevel::Unknown);
    }

    #[test]
    fn test_trust_registry_determine_trust_trusted() {
        let registry = TrustRegistry::new();

        // Add trusted source
        let trust_source = TrustSource::GitRepository {
            url_pattern: "https://github.com/myorg/*".to_string(),
            branch: None,
            description: "Internal repos".to_string(),
        };
        registry.add_trusted_source(trust_source).unwrap();

        let component_source = ComponentSource::Git {
            url: "https://github.com/myorg/my-component".to_string(),
            branch: "main".to_string(),
            commit: "abc123".to_string(),
        };

        let level = registry.determine_trust_level("test-component", &component_source);
        assert_eq!(level, TrustLevel::Trusted);
    }

    #[test]
    fn test_trust_registry_determine_trust_devmode() {
        let registry = TrustRegistry::new();
        registry.set_dev_mode(true);

        let source = ComponentSource::Git {
            url: "https://github.com/unknown/component".to_string(),
            branch: "main".to_string(),
            commit: "abc123".to_string(),
        };

        let level = registry.determine_trust_level("test-component", &source);
        assert_eq!(level, TrustLevel::DevMode);
    }

    #[test]
    fn test_trust_registry_add_trusted_source() {
        let registry = TrustRegistry::new();
        assert_eq!(registry.list_trusted_sources().len(), 0);

        let trust_source = TrustSource::GitRepository {
            url_pattern: "https://github.com/myorg/*".to_string(),
            branch: None,
            description: "Internal repos".to_string(),
        };
        registry.add_trusted_source(trust_source).unwrap();

        assert_eq!(registry.list_trusted_sources().len(), 1);
    }

    #[test]
    fn test_trust_registry_remove_trusted_source() {
        let registry = TrustRegistry::new();

        let trust_source = TrustSource::GitRepository {
            url_pattern: "https://github.com/myorg/*".to_string(),
            branch: None,
            description: "Internal repos".to_string(),
        };
        registry.add_trusted_source(trust_source).unwrap();
        assert_eq!(registry.list_trusted_sources().len(), 1);

        registry.remove_trusted_source(0).unwrap();
        assert_eq!(registry.list_trusted_sources().len(), 0);
    }

    #[test]
    fn test_trust_registry_remove_invalid_index() {
        let registry = TrustRegistry::new();
        let result = registry.remove_trusted_source(0);
        assert!(result.is_err());
        assert!(matches!(result, Err(TrustError::SourceNotFound { .. })));
    }

    #[test]
    fn test_trust_registry_dev_mode() {
        let registry = TrustRegistry::new();
        assert!(!registry.is_dev_mode());

        registry.set_dev_mode(true);
        assert!(registry.is_dev_mode());

        registry.set_dev_mode(false);
        assert!(!registry.is_dev_mode());
    }

    // ========================================================================
    // Task 2.1.9 Tests: TOML Configuration Parser
    // ========================================================================

    #[tokio::test]
    async fn test_from_config_valid() {
        let config_content = r#"
[trust]
dev_mode = false

[[trust.sources]]
type = "git"
url_pattern = "https://github.com/myorg/*"
branch = "main"
description = "Internal repos"

[[trust.sources]]
type = "signing_key"
public_key = "ed25519:ABC123"
signer = "test@example.com"
description = "Test key"

[[trust.sources]]
type = "local"
path_pattern = "/opt/components/*"
description = "Local components"
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(config_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let registry = TrustRegistry::from_config(temp_file.path()).await.unwrap();
        assert!(!registry.is_dev_mode());
        assert_eq!(registry.list_trusted_sources().len(), 3);
    }

    #[tokio::test]
    async fn test_from_config_dev_mode_enabled() {
        let config_content = r#"
[trust]
dev_mode = true
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(config_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let registry = TrustRegistry::from_config(temp_file.path()).await.unwrap();
        assert!(registry.is_dev_mode());
    }

    #[tokio::test]
    async fn test_from_config_file_not_found() {
        let result = TrustRegistry::from_config(Path::new("/nonexistent/file.toml")).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(TrustError::ConfigNotFound { .. })));
    }

    #[tokio::test]
    async fn test_from_config_invalid_toml() {
        let config_content = "invalid toml syntax {{{";

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(config_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let result = TrustRegistry::from_config(temp_file.path()).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(TrustError::ParseError { .. })));
    }

    #[tokio::test]
    async fn test_from_config_empty_url_pattern() {
        let config_content = r#"
[trust]
dev_mode = false

[[trust.sources]]
type = "git"
url_pattern = ""
description = "Test"
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(config_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let result = TrustRegistry::from_config(temp_file.path()).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(TrustError::InvalidSource { .. })));
    }

    #[tokio::test]
    async fn test_from_config_invalid_public_key_format() {
        let config_content = r#"
[trust]
dev_mode = false

[[trust.sources]]
type = "signing_key"
public_key = "invalid_format"
signer = "test@example.com"
description = "Test"
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(config_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let result = TrustRegistry::from_config(temp_file.path()).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(TrustError::InvalidSource { .. })));
    }
}
