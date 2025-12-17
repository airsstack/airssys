# WASM-TASK-005 Phase 2 Task 2.3: Trust Configuration System - IMPLEMENTATION PLAN

**Task:** Trust Configuration System  
**Status:** 📋 PLANNED (Ready to Start)  
**Date Created:** 2025-12-17  
**Estimated Duration:** 2-3 days (16 hours)  
**Prerequisites:** ✅ Phase 1 complete (Tasks 1.1-1.3), Tasks 2.1-2.2 (Trust + Approval) in progress

---

## Executive Summary

**What**: Create a comprehensive trust configuration management system including: **trust-config.toml** file parser, configuration validation, CLI tools for managing trusted sources (add/remove/list), DevMode enable/disable controls, and configuration integrity verification.

**Why**: Administrators need intuitive tools to manage which component sources are trusted. The system must be easy to configure initially, maintainable over time, and secure against tampering. Clear CLI commands enable rapid trust management without editing raw TOML files.

**How**: Implement a TOML configuration parser with strict validation, CLI commands for trust management (`trust add-git`, `trust add-key`, `trust list`, `trust remove`), DevMode toggle with safety checks, configuration file integrity verification (checksums), and comprehensive error messages for troubleshooting.

**Architecture Position**: This module provides the user-facing interface for trust management, wrapping the trust registry (Task 2.1) with CLI commands and configuration persistence.

---

## Implementation Strategy

### Core Design Principles

1. **User-Friendly CLI**: Intuitive commands with helpful prompts
2. **Validation First**: Reject invalid configurations before saving
3. **Fail-Safe**: Backup config before modifications
4. **Clear Errors**: Actionable error messages with examples
5. **Audit Trail**: Log all configuration changes

### Trust Configuration Workflow

```text
User Action (CLI/File Edit)
          ↓
Parse trust-config.toml
          ↓
Validate Configuration
     ├─ Valid? ─→ Apply Changes
     │             ↓
     │        Update TrustRegistry
     │             ↓
     │        Save to File
     │             ↓
     │        Audit Log
     │
     └─ Invalid? ─→ Show Errors
                    ↓
                Return Suggestions
```

---

## Data Structure Specifications

### 1. TrustConfig (Main Configuration)

```rust
/// Complete trust configuration.
/// 
/// Deserialized from `trust-config.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustConfig {
    /// Trust settings
    pub trust: TrustSettings,
}

/// Trust configuration settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustSettings {
    /// Development mode flag (DANGEROUS in production)
    #[serde(default)]
    pub dev_mode: bool,
    
    /// List of trusted sources
    #[serde(default)]
    pub sources: Vec<TrustSourceConfig>,
}

impl TrustConfig {
    /// Parses configuration from TOML string.
    pub fn from_toml(toml: &str) -> Result<Self, ConfigError>;
    
    /// Parses configuration from file.
    pub fn from_file(path: &Path) -> Result<Self, ConfigError>;
    
    /// Serializes configuration to TOML string.
    pub fn to_toml(&self) -> Result<String, ConfigError>;
    
    /// Saves configuration to file.
    pub fn save_to_file(&self, path: &Path) -> Result<(), ConfigError>;
    
    /// Validates configuration.
    pub fn validate(&self) -> Result<(), ConfigError>;
}
```

### 2. TrustSourceConfig (TOML Representation)

```rust
/// Trust source configuration from TOML.
/// 
/// Corresponds to `[[trust.sources]]` entries in trust-config.toml.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TrustSourceConfig {
    /// Git repository source.
    Git {
        /// URL pattern
        url: String,
        
        /// Optional branch restriction
        #[serde(skip_serializing_if = "Option::is_none")]
        branch: Option<String>,
        
        /// Description
        description: String,
    },
    
    /// Signing key source.
    SigningKey {
        /// Public key (Ed25519)
        public_key: String,
        
        /// Signer identity
        signer: String,
        
        /// Description
        description: String,
    },
    
    /// Local filesystem path source.
    Local {
        /// Path pattern
        path_pattern: String,
        
        /// Description
        description: String,
    },
}

impl TrustSourceConfig {
    /// Converts to runtime TrustSource (Task 2.1).
    pub fn to_trust_source(&self) -> TrustSource;
    
    /// Validates this source configuration.
    pub fn validate(&self) -> Result<(), ConfigError>;
}
```

### 3. ConfigValidator (Validation Engine)

```rust
/// Trust configuration validator.
/// 
/// Validates trust-config.toml for correctness and security.
pub struct ConfigValidator;

impl ConfigValidator {
    /// Validates complete configuration.
    pub fn validate_config(config: &TrustConfig) -> Result<(), ConfigError>;
    
    /// Validates trust source.
    pub fn validate_source(source: &TrustSourceConfig) -> Result<(), ConfigError>;
    
    /// Validates Git URL pattern.
    pub fn validate_git_url(url: &str) -> Result<(), ConfigError>;
    
    /// Validates signing key format.
    pub fn validate_signing_key(key: &str) -> Result<(), ConfigError>;
    
    /// Validates local path pattern.
    pub fn validate_local_path(path: &str) -> Result<(), ConfigError>;
    
    /// Checks for duplicate sources.
    pub fn check_duplicates(sources: &[TrustSourceConfig]) -> Result<(), ConfigError>;
    
    /// Validates DevMode usage (warn if enabled).
    pub fn validate_dev_mode(enabled: bool) -> Result<(), ConfigError>;
}
```

### 4. ConfigManager (File Operations)

```rust
/// Trust configuration file manager.
/// 
/// Handles loading, saving, backup, and integrity verification.
pub struct ConfigManager {
    /// Config file path
    config_path: PathBuf,
    
    /// Backup directory
    backup_dir: PathBuf,
    
    /// Audit logger
    audit_logger: Arc<SecurityAuditLogger>,
}

impl ConfigManager {
    /// Creates manager for specified config file.
    pub fn new(
        config_path: PathBuf,
        backup_dir: PathBuf,
        audit_logger: Arc<SecurityAuditLogger>,
    ) -> Self;
    
    /// Loads configuration from file.
    pub fn load_config(&self) -> Result<TrustConfig, ConfigError>;
    
    /// Saves configuration to file (with backup).
    pub fn save_config(&self, config: &TrustConfig) -> Result<(), ConfigError>;
    
    /// Creates backup of current config.
    pub fn backup_config(&self) -> Result<PathBuf, ConfigError>;
    
    /// Restores config from backup.
    pub fn restore_backup(&self, backup_path: &Path) -> Result<(), ConfigError>;
    
    /// Lists available backups.
    pub fn list_backups(&self) -> Result<Vec<PathBuf>, ConfigError>;
    
    /// Verifies config file integrity (checksum).
    pub fn verify_integrity(&self) -> Result<bool, ConfigError>;
    
    /// Computes config checksum (SHA-256).
    pub fn compute_checksum(&self, config: &TrustConfig) -> Result<String, ConfigError>;
}
```

### 5. TrustCli (CLI Commands)

```rust
/// Trust configuration CLI.
/// 
/// Provides user-facing commands for trust management.
pub struct TrustCli {
    /// Config manager
    config_manager: Arc<ConfigManager>,
    
    /// Trust registry (Task 2.1)
    trust_registry: Arc<TrustRegistry>,
}

impl TrustCli {
    /// Creates CLI with dependencies.
    pub fn new(
        config_manager: Arc<ConfigManager>,
        trust_registry: Arc<TrustRegistry>,
    ) -> Self;
    
    // ===== Git Repository Commands =====
    
    /// Adds trusted Git repository.
    pub fn add_git_source(
        &self,
        url_pattern: String,
        branch: Option<String>,
        description: String,
    ) -> Result<(), CliError>;
    
    // ===== Signing Key Commands =====
    
    /// Adds trusted signing key.
    pub fn add_signing_key(
        &self,
        public_key: String,
        signer: String,
        description: String,
    ) -> Result<(), CliError>;
    
    // ===== Local Path Commands =====
    
    /// Adds trusted local path.
    pub fn add_local_path(
        &self,
        path_pattern: String,
        description: String,
    ) -> Result<(), CliError>;
    
    // ===== List/Remove Commands =====
    
    /// Lists all trusted sources.
    pub fn list_sources(&self) -> Result<(), CliError>;
    
    /// Removes trusted source by index.
    pub fn remove_source(&self, index: usize) -> Result<(), CliError>;
    
    // ===== DevMode Commands =====
    
    /// Enables development mode (with confirmation).
    pub fn enable_dev_mode(&self) -> Result<(), CliError>;
    
    /// Disables development mode.
    pub fn disable_dev_mode(&self) -> Result<(), CliError>;
    
    /// Shows current DevMode status.
    pub fn show_dev_mode_status(&self) -> Result<(), CliError>;
    
    // ===== Validation Commands =====
    
    /// Validates current configuration.
    pub fn validate_config(&self) -> Result<(), CliError>;
    
    /// Shows configuration file path.
    pub fn show_config_path(&self) -> Result<(), CliError>;
}
```

---

## Configuration File Specifications

### Complete trust-config.toml Schema

```toml
# Trust Configuration for airssys-wasm
# Version: 1.0.0
# Generated: 2025-12-17T10:30:00Z

[trust]
# Development Mode: BYPASS ALL SECURITY CHECKS
# ⚠️  WARNING: NEVER enable in production! ⚠️
# Only use for local development and testing.
dev_mode = false

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Trusted Git Repositories
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[[trust.sources]]
type = "git"
url = "https://github.com/mycompany/*"
branch = "main"
description = "Internal company repositories (auto-approved)"

[[trust.sources]]
type = "git"
url = "https://github.com/verified-org/wasm-*"
branch = "stable"
description = "Verified external organization components"

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Trusted Signing Keys (Ed25519)
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[[trust.sources]]
type = "signing_key"
public_key = "ed25519:AAAAC3NzaC1lZDI1NTE5AAAAIJbpYR+abc123..."
signer = "engineering@mycompany.com"
description = "Engineering team release signing key"

[[trust.sources]]
type = "signing_key"
public_key = "ed25519:AAAAC3NzaC1lZDI1NTE5AAAAIJbpYR+xyz789..."
signer = "security@mycompany.com"
description = "Security team audit signing key"

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Trusted Local Paths (Pre-Verified Components)
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[[trust.sources]]
type = "local"
path_pattern = "/opt/verified-components/*"
description = "System-verified components in /opt"

[[trust.sources]]
type = "local"
path_pattern = "/usr/local/airssys/components/*"
description = "Official AirsSys distribution components"
```

### Validation Rules

| Rule | Description | Error Message | Suggestion |
|------|-------------|---------------|------------|
| **Valid TOML** | File must parse as valid TOML | "Invalid TOML syntax at line {line}: {error}" | "Check TOML syntax with online validator" |
| **Required Section** | `[trust]` section required | "Missing [trust] section" | "Add [trust] section to config file" |
| **Valid Type** | `type` must be "git", "signing_key", or "local" | "Invalid source type: '{type}'" | "Use 'git', 'signing_key', or 'local'" |
| **Non-Empty URL** | Git URL pattern must not be empty | "Empty Git URL pattern" | "Provide valid Git URL pattern" |
| **Valid URL Format** | Git URL must be valid HTTP(S) or Git URL | "Invalid Git URL format: '{url}'" | "Use https://github.com/org/repo format" |
| **Valid Branch** | Branch name must be valid | "Invalid branch name: '{branch}'" | "Use alphanumeric branch names" |
| **Valid Key Format** | Public key must start with "ed25519:" | "Invalid public key format: must start with 'ed25519:'" | "Use format: ed25519:BASE64_KEY" |
| **Valid Key Length** | Public key must be correct length (32 bytes Ed25519) | "Invalid public key length: expected 44 chars" | "Verify Ed25519 public key encoding" |
| **Non-Empty Path** | Local path pattern must not be empty | "Empty local path pattern" | "Provide valid filesystem path pattern" |
| **Absolute Path** | Local path must be absolute | "Local path must be absolute: '{path}'" | "Use absolute path starting with /" |
| **No Duplicates** | Same source cannot appear twice | "Duplicate source: {identifier}" | "Remove duplicate entry" |
| **Description Required** | Description must not be empty | "Empty description for source" | "Provide descriptive text" |
| **DevMode Warning** | Warn if dev_mode = true | "⚠️  DevMode enabled - DO NOT use in production!" | "Set dev_mode = false for production" |

---

## Implementation Steps (18 Steps, ~16 hours)

### Step 1: Create Config Module Structure (30 min)
- Create `airssys-wasm/src/security/config.rs`
- Add module declaration to `security/mod.rs`
- Add 3-layer imports (§2.1)
- Define module-level rustdoc
- **Checkpoint**: `cargo check` passes

### Step 2: Implement TrustConfig Types (1 hour)
- `TrustConfig` and `TrustSettings` structs
- `TrustSourceConfig` enum
- Serde derives
- 3 unit tests
- **Checkpoint**: Types deserialize correctly

### Step 3: Implement TOML Parsing (1.5 hours)
- `from_toml()` method
- `from_file()` method
- `to_toml()` method
- `save_to_file()` method
- 8 unit tests (valid, invalid, edge cases)
- **Checkpoint**: TOML parsing tests pass

### Step 4: Implement Git URL Validation (1 hour)
- `validate_git_url()` method
- URL format checking (https://, git://, git@)
- Wildcard pattern validation
- 6 unit tests
- **Checkpoint**: Git URL validation works

### Step 5: Implement Signing Key Validation (1.5 hours)
- `validate_signing_key()` method
- Ed25519 format checking (prefix + length)
- Base64 decoding validation
- 8 unit tests
- **Checkpoint**: Key validation tests pass

### Step 6: Implement Local Path Validation (1 hour)
- `validate_local_path()` method
- Absolute path check
- Path pattern validation
- 5 unit tests
- **Checkpoint**: Path validation works

### Step 7: Implement Duplicate Detection (1 hour)
- `check_duplicates()` method
- Source identifier comparison
- 5 unit tests
- **Checkpoint**: Duplicate detection works

### Step 8: Implement ConfigValidator (1.5 hours)
- `ConfigValidator` struct
- `validate_config()` orchestrator
- `validate_source()` method
- 10 unit tests
- **Checkpoint**: Full validation works

### Step 9: Implement ConfigManager Core (2 hours)
- `ConfigManager` struct
- `load_config()` method
- `save_config()` method
- File I/O error handling
- 8 unit tests
- **Checkpoint**: File operations work

### Step 10: Implement Backup System (1.5 hours)
- `backup_config()` method
- `restore_backup()` method
- `list_backups()` method
- Timestamp-based backup naming
- 6 unit tests
- **Checkpoint**: Backup tests pass

### Step 11: Implement Integrity Verification (1 hour)
- `verify_integrity()` method
- `compute_checksum()` SHA-256 hashing
- 4 unit tests
- **Checkpoint**: Integrity checks work

### Step 12: Implement TrustCli Core (2 hours)
- `TrustCli` struct
- `add_git_source()`, `add_signing_key()`, `add_local_path()`
- Config mutation logic
- 10 unit tests
- **Checkpoint**: Add commands work

### Step 13: Implement List/Remove Commands (1 hour)
- `list_sources()` pretty printing
- `remove_source()` by index
- 5 unit tests
- **Checkpoint**: List/remove tests pass

### Step 14: Implement DevMode Commands (1.5 hours)
- `enable_dev_mode()` with confirmation prompt
- `disable_dev_mode()`
- `show_dev_mode_status()`
- Safety warnings
- 5 unit tests
- **Checkpoint**: DevMode commands work

### Step 15: Implement Validation Commands (1 hour)
- `validate_config()` CLI command
- `show_config_path()` command
- Pretty error formatting
- 4 unit tests
- **Checkpoint**: Validation commands work

### Step 16: Integration with TrustRegistry (1.5 hours)
- Apply config changes to TrustRegistry (Task 2.1)
- Live reload functionality
- 5 integration tests
- **Checkpoint**: Registry integration works

### Step 17: Comprehensive Documentation (1.5 hours)
- Module-level rustdoc
- CLI command documentation
- Configuration file examples
- Troubleshooting guide
- **Checkpoint**: Zero rustdoc warnings

### Step 18: Final Quality Gates (30 min)
- `cargo clippy --all-targets` (zero warnings)
- `cargo test --all-targets` (all pass)
- `cargo doc --no-deps` (zero warnings)
- **Checkpoint**: All quality gates pass

---

## Test Plan (40+ Test Scenarios)

### Positive Tests (12 tests)

| Test ID | Scenario | Expected Output |
|---------|----------|-----------------|
| V01 | Parse valid trust-config.toml | TrustConfig returned |
| V02 | Add Git source | Source added, config saved |
| V03 | Add signing key | Key added, config saved |
| V04 | Add local path | Path added, config saved |
| V05 | List all sources | Sources displayed |
| V06 | Remove source by index | Source removed |
| V07 | Enable DevMode | DevMode enabled with warning |
| V08 | Disable DevMode | DevMode disabled |
| V09 | Backup config | Backup file created |
| V10 | Restore backup | Config restored |
| V11 | Validate valid config | Validation success |
| V12 | Compute checksum | SHA-256 hash returned |

### Negative Tests (15 tests)

| Test ID | Scenario | Expected Output |
|---------|----------|----------------|
| E01 | Invalid TOML syntax | ParseError with line number |
| E02 | Missing [trust] section | ValidationError |
| E03 | Invalid source type | ValidationError |
| E04 | Empty Git URL | ValidationError |
| E05 | Invalid Git URL format | ValidationError |
| E06 | Invalid signing key format | ValidationError |
| E07 | Invalid signing key length | ValidationError |
| E08 | Empty local path | ValidationError |
| E09 | Relative local path | ValidationError |
| E10 | Duplicate Git source | ValidationError |
| E11 | Empty description | ValidationError |
| E12 | Remove non-existent index | NotFoundError |
| E13 | Load non-existent config | FileNotFoundError |
| E14 | Save to read-only directory | IoError |
| E15 | Restore non-existent backup | NotFoundError |

### Edge Case Tests (13 tests)

| Test ID | Scenario | Expected Behavior |
|---------|----------|-------------------|
| EC01 | Very long Git URL (1000 chars) | Parse correctly |
| EC02 | Unicode in Git URL | Handle correctly |
| EC03 | Whitespace in patterns | Trim and parse |
| EC04 | Comments in TOML | Ignore and parse |
| EC05 | Empty sources array | Valid config |
| EC06 | DevMode = true in file | Warn prominently |
| EC07 | 100 trusted sources | Parse and list all |
| EC08 | Concurrent config saves | Last write wins |
| EC09 | Backup with same timestamp | Append counter |
| EC10 | Config file permissions | Check writable |
| EC11 | Symlink in path pattern | Resolve and validate |
| EC12 | Checksum mismatch | Integrity failure |
| EC13 | Malformed backup file | Restore error |

---

## CLI Command Examples

### Add Trusted Git Repository

```bash
$ airssys-wasm trust add-git "https://github.com/mycompany/*" \
    --branch main \
    --description "Internal company repositories"

✅ Added trusted Git repository
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
URL Pattern: https://github.com/mycompany/*
Branch:      main
Description: Internal company repositories
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Configuration saved to: /etc/airssys/trust-config.toml
Backup created: /etc/airssys/backups/trust-config-2025-12-17T10-30-45.toml
```

### Add Trusted Signing Key

```bash
$ airssys-wasm trust add-key "ed25519:AAAAC3NzaC1lZDI1NTE5AAAAIJbpYR..." \
    --signer "engineering@mycompany.com" \
    --description "Engineering team signing key"

✅ Added trusted signing key
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Public Key: ed25519:AAAAC3NzaC1lZDI1NTE5AAAAIJbpYR... (truncated)
Signer:     engineering@mycompany.com
Description: Engineering team signing key
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Configuration saved to: /etc/airssys/trust-config.toml
```

### List Trusted Sources

```bash
$ airssys-wasm trust list

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Trusted Sources (5)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[1] Git: https://github.com/mycompany/*
    Branch: main
    Description: Internal company repositories

[2] Git: https://github.com/verified-org/wasm-*
    Branch: stable
    Description: Verified external organization

[3] SigningKey: ed25519:AAAAC3Nz... (truncated)
    Signer: engineering@mycompany.com
    Description: Engineering team signing key

[4] SigningKey: ed25519:AAAAC3Nz... (truncated)
    Signer: security@mycompany.com
    Description: Security team audit key

[5] Local: /opt/verified-components/*
    Description: System-verified components

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### Remove Trusted Source

```bash
$ airssys-wasm trust remove 3

⚠️  Remove trusted source?
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
[3] SigningKey: ed25519:AAAAC3Nz... (truncated)
    Signer: engineering@mycompany.com
    Description: Engineering team signing key
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Confirm removal [y/N]: y

✅ Removed trusted source #3

Configuration saved to: /etc/airssys/trust-config.toml
Backup created: /etc/airssys/backups/trust-config-2025-12-17T10-35-12.toml
```

### Enable DevMode (with Safety Confirmation)

```bash
$ airssys-wasm trust dev-mode enable

⚠️  ⚠️  ⚠️  ENABLE DEVELOPMENT MODE? ⚠️  ⚠️  ⚠️

This will BYPASS ALL SECURITY CHECKS!

Components will have UNRESTRICTED ACCESS to:
  • All filesystem paths (read/write/execute)
  • All network endpoints (inbound/outbound)
  • All storage namespaces (unlimited)

⚠️  NEVER use DevMode in production!
⚠️  Only use for local development and testing!

Type "I UNDERSTAND THE RISKS" to confirm: I UNDERSTAND THE RISKS

⚠️  DevMode ENABLED

Configuration saved to: /etc/airssys/trust-config.toml
```

### Disable DevMode

```bash
$ airssys-wasm trust dev-mode disable

✅ DevMode DISABLED

Security checks will be enforced for all components.

Configuration saved to: /etc/airssys/trust-config.toml
```

### Validate Configuration

```bash
$ airssys-wasm trust validate

✅ Configuration is valid
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Config file:   /etc/airssys/trust-config.toml
DevMode:       Disabled ✅
Sources:       5 trusted sources
Checksum:      SHA-256:abc123... ✅
Last modified: 2025-12-17 10:30:45 UTC
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### Validate with Errors

```bash
$ airssys-wasm trust validate

❌ Configuration validation failed
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
[ERROR] Invalid Git URL format: 'github.com/mycompany/*'
        Expected: https:// or git:// prefix
        Suggestion: Use https://github.com/mycompany/*

[ERROR] Duplicate source: git:https://github.com/verified-org/*
        Found at: line 15 and line 23
        Suggestion: Remove duplicate entry

[WARNING] DevMode enabled
          DO NOT use in production!
          Suggestion: Set dev_mode = false
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Fix errors and run: airssys-wasm trust validate
```

---

## Performance Targets

### Configuration Operations
- **Load Config**: <10ms (parse + validate)
- **Save Config**: <20ms (serialize + write + backup)
- **Validate Config**: <5ms (all validation rules)
- **List Sources**: <1ms (format and print)
- **Backup Creation**: <15ms (copy file)

---

## Integration Points

### Task 2.1 Integration (TrustRegistry)

```rust
// Apply configuration changes to TrustRegistry
let config = config_manager.load_config()?;

// Clear existing sources
trust_registry.clear_sources();

// Add sources from config
for source_config in config.trust.sources {
    let source = source_config.to_trust_source();
    trust_registry.add_trusted_source(source)?;
}

// Set DevMode
trust_registry.set_dev_mode(config.trust.dev_mode);
```

### Task 2.2 Integration (CLI Approval Commands)

```bash
# List approval requests (from Task 2.2)
$ airssys-wasm approval list

# If component needs approval, add source to trusted list
$ airssys-wasm trust add-git "https://github.com/new-org/*"
✅ Added trusted Git repository

# Future installations from this source auto-approved
```

---

## Quality Gates

### Cargo Clippy Requirements
- **Command**: `cargo clippy --all-targets --all-features -- -D warnings`
- **Target**: Zero warnings (deny warnings)
- **Enforced Lints**: `unwrap_used`, `expect_used`, `panic` (deny)

### Rustdoc Requirements
- **Command**: `cargo doc --no-deps --document-private-items`
- **Target**: Zero rustdoc warnings
- **Standards**: Microsoft Rust Guidelines (M-MODULE-DOCS, M-CANONICAL-DOCS)

### Test Coverage Targets
- **Unit Test Coverage**: >90% (all config logic)
- **Integration Test Coverage**: 10+ integration tests
- **Edge Case Coverage**: 13+ edge case tests
- **Total Tests**: 40+ test cases

---

## Timeline Estimate

| Step | Description | Time | Cumulative |
|------|-------------|------|------------|
| 1 | Config module structure | 30 min | 30 min |
| 2 | TrustConfig types | 1 hour | 1.5 hours |
| 3 | TOML parsing | 1.5 hours | 3 hours |
| 4 | Git URL validation | 1 hour | 4 hours |
| 5 | Signing key validation | 1.5 hours | 5.5 hours |
| 6 | Local path validation | 1 hour | 6.5 hours |
| 7 | Duplicate detection | 1 hour | 7.5 hours |
| 8 | ConfigValidator | 1.5 hours | 9 hours |
| 9 | ConfigManager core | 2 hours | 11 hours |
| 10 | Backup system | 1.5 hours | 12.5 hours |
| 11 | Integrity verification | 1 hour | 13.5 hours |
| 12 | TrustCli core | 2 hours | 15.5 hours |
| 13 | List/remove commands | 1 hour | 16.5 hours |
| 14 | DevMode commands | 1.5 hours | 18 hours |
| 15 | Validation commands | 1 hour | 19 hours |
| 16 | Registry integration | 1.5 hours | 20.5 hours |
| 17 | Documentation | 1.5 hours | 22 hours |
| 18 | Final quality gates | 30 min | **16 hours** |

**Total Duration**: 16 hours ≈ **2-3 days** (6-8 hour workdays)

**Breakdown by Activity**:
- Core implementation: 11 hours (69%)
- Testing: 2.5 hours (16%)
- Documentation: 1.5 hours (9%)
- Quality assurance: 1 hour (6%)

---

## Risk Assessment

### Technical Risks

| Risk | Severity | Probability | Mitigation |
|------|----------|-------------|------------|
| **Config File Corruption** | High | Low | Atomic writes, backups |
| **Invalid TOML Parsing** | Medium | Medium | Comprehensive validation |
| **DevMode Misuse** | High | Low | Prominent warnings, confirmation |
| **File Permission Issues** | Medium | Medium | Clear error messages |

---

## Standards Compliance

### PROJECTS_STANDARD.md
- §2.1: 3-layer import organization ✅
- §4.3: Module architecture (mod.rs only re-exports) ✅
- §5.1: Dependency management ✅
- §6.1: YAGNI principles ✅

### Microsoft Rust Guidelines
- M-DESIGN-FOR-AI: Clear API, extensive docs ✅
- M-CANONICAL-DOCS: Comprehensive public API docs ✅
- M-EXAMPLES: Examples for all commands ✅

### ADR Compliance
- ADR-WASM-005: Capability-Based Security Model ✅
- ADR-WASM-010: Implementation Strategy ✅

---

## Approval Status

**Planner**: Memory Bank Planner  
**Date**: 2025-12-17  
**Status**: ✅ **APPROVED** - Ready for implementation

This plan provides a comprehensive blueprint for implementing the trust configuration system with user-friendly CLI, robust validation, and production-ready documentation.

**Ready to Start:** Task 2.3 implementation can begin after Tasks 2.1-2.2 completion.
