# WASM-TASK-005 Phase 2 Task 2.3: Trust Configuration System - IMPLEMENTATION PLAN

**Task:** Trust Configuration System  
**Status:** ✅ **COMPLETE**  
**Date Created:** 2025-12-17  
**Date Completed:** 2025-12-19  
**Estimated Duration:** 2 days (13.5 hours)  
**Prerequisites:** ✅ Phase 1 complete (Tasks 1.1-1.3), ✅ Tasks 2.1-2.2 (Trust + Approval) COMPLETE

---

## Executive Summary

**What**: Create a comprehensive trust configuration management system including: **trust-config.toml** file parser, configuration validation, configuration file management (load/save/backup), DevMode enable/disable controls, and configuration integrity verification. **This task focuses ONLY on core library logic** - CLI commands are implemented separately in airssys-wasm-cli.

**Why**: Administrators need programmatic APIs to manage which component sources are trusted. The system must be easy to configure, maintainable over time, and secure against tampering. The core library provides the foundation for CLI tools and other management interfaces.

**How**: Implement a TOML configuration parser with strict validation (`TrustConfig`, `TrustSourceConfig`), configuration file management (`ConfigManager`), validation engine (`ConfigValidator`), DevMode toggle with safety checks, configuration file integrity verification (checksums), and comprehensive error messages for troubleshooting.

**Architecture Position**: This module provides the **core library API** for trust management, wrapping the trust registry (Task 2.1) with configuration persistence and validation. CLI commands are implemented in `airssys-wasm-cli` (see separate task documentation).

---

## Scope: airssys-wasm Deliverables ONLY

**This task implements the core library in `airssys-wasm` package:**

### ✅ What WILL Be Implemented (airssys-wasm)

| Component | File | Responsibility | Lines (Est.) |
|-----------|------|----------------|--------------|
| **TrustConfig** | `src/security/config.rs` | TOML configuration structs | ~200 |
| **TrustSourceConfig** | `src/security/config.rs` | Source type enum (Git/Key/Local) | ~150 |
| **ConfigValidator** | `src/security/config.rs` | Validation engine (13 rules) | ~300 |
| **ConfigManager** | `src/security/config.rs` | File operations, backup, integrity | ~400 |
| **Tests** | `src/security/config.rs` | 40+ unit tests | ~600 |
| **Integration** | `src/security/mod.rs` | Module exports | ~20 |

**Total Code:** ~1,670 lines in `airssys-wasm/src/security/config.rs`

**Public APIs Exposed:**
```rust
// These APIs will be consumed by airssys-wasm-cli
pub struct TrustConfig { /* ... */ }
pub struct ConfigManager { /* ... */ }
pub struct ConfigValidator { /* ... */ }

impl TrustConfig {
    pub fn from_file(path: &Path) -> Result<Self>;
    pub fn from_toml(toml: &str) -> Result<Self>;
    pub fn to_toml(&self) -> Result<String>;
    pub fn validate(&self) -> Result<()>;
}

impl ConfigManager {
    pub fn load_config(&self) -> Result<TrustConfig>;
    pub fn save_config(&self, config: &TrustConfig) -> Result<()>;
    pub fn backup_config(&self) -> Result<PathBuf>;
    pub fn verify_integrity(&self) -> Result<bool>;
}
```

### ❌ What Will NOT Be Implemented (Separate Package)

**CLI Commands** → Implemented in `airssys-wasm-cli` package:
- ❌ Clap command structures (`TrustArgs`, `TrustCommands`)
- ❌ User-facing command implementations (add-git, list, remove, etc.)
- ❌ Terminal output formatting (colored output, progress bars)
- ❌ Interactive prompts (confirmations, safety warnings)
- ❌ Shell completion scripts

**Reference:** `.memory-bank/sub-projects/airssys-wasm-cli/tasks/task-cli-002-trust-command.md`

---

## Implementation Strategy

### Core Design Principles (airssys-wasm Library)

1. **Library-First Design**: Clean, testable APIs that can be consumed by any client
2. **Validation First**: Reject invalid configurations before saving
3. **Fail-Safe**: Backup config before modifications
4. **Clear Errors**: Actionable error messages with context
5. **Audit Trail**: Log all configuration changes via airssys-osl audit logger

### Trust Configuration Workflow

```text
Library Consumer (CLI/API/Service)
          ↓
     ConfigManager::load_config()
          ↓
     TrustConfig::validate()
          ↓
     ├─ Valid? ─→ Apply Changes
     │             ↓
     │        TrustRegistry::update()
     │             ↓
     │        ConfigManager::save_config()
     │             ↓
     │        Audit Log (airssys-osl)
     │
     └─ Invalid? ─→ Return ConfigError
                    ↓
                 (Consumer handles display)
```

**Note**: This task implements ONLY the library layer. CLI commands that consume these APIs are in `airssys-wasm-cli` (task-cli-002).

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

### 5. TrustCli (CLI Commands) - See airssys-wasm-cli

**Note**: CLI commands for trust management are implemented in the `airssys-wasm-cli` package (library-only, 100% composable).

**See**: `.memory-bank/sub-projects/airssys-wasm-cli/tasks/task-cli-002-trust-command.md`

**APIs Exposed for CLI Integration:**
```rust
// airssys-wasm exposes these APIs for CLI usage:
impl TrustConfig {
    pub fn from_file(path: &Path) -> Result<Self, ConfigError>;
    pub fn to_toml(&self) -> Result<String, ConfigError>;
    pub fn validate(&self) -> Result<(), ConfigError>;
}

impl ConfigManager {
    pub fn load_config(&self) -> Result<TrustConfig, ConfigError>;
    pub fn save_config(&self, config: &TrustConfig) -> Result<(), ConfigError>;
    pub fn backup_config(&self) -> Result<PathBuf, ConfigError>;
}

impl ConfigValidator {
    pub fn validate_config(config: &TrustConfig) -> Result<(), ConfigError>;
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

## File Structure: What Will Be Created/Modified

### Files to Create

```
airssys-wasm/
└── src/
    └── security/
        └── config.rs           ← NEW FILE (~1,670 lines)
            ├── TrustConfig struct
            ├── TrustSettings struct  
            ├── TrustSourceConfig enum
            ├── ConfigValidator struct
            ├── ConfigManager struct
            ├── ConfigError enum
            └── 40+ unit tests (inline)
```

### Files to Modify

```
airssys-wasm/
└── src/
    └── security/
        └── mod.rs              ← UPDATE (add config module)
            // Add this line:
            pub mod config;
            
            // Re-export public types:
            pub use config::{
                TrustConfig,
                TrustSettings,
                TrustSourceConfig,
                ConfigManager,
                ConfigValidator,
                ConfigError,
            };
```

### Existing Files (No Changes)

```
airssys-wasm/
└── src/
    └── security/
        ├── trust.rs            ← EXISTING (Task 2.1 - TrustRegistry)
        ├── approval.rs         ← EXISTING (Task 2.2 - ApprovalWorkflow)
        ├── capability.rs       ← EXISTING (Phase 1 - Capabilities)
        └── parser.rs           ← EXISTING (Phase 1 - Component.toml parser)
```

### Integration Points

**config.rs will use:**
- `trust.rs` → `TrustRegistry` for applying configuration
- `trust.rs` → `TrustSource` enum for conversion
- `airssys-osl` → `SecurityAuditLogger` for audit trail

**config.rs will NOT use:**
- ❌ `approval.rs` (approval workflow - CLI will coordinate if needed)
- ❌ `capability.rs` (capability enforcement - different concern)

---

## Implementation Steps (11 Steps, ~13.5 hours)

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

### Steps 12-15: CLI Commands - See airssys-wasm-cli
**Note**: CLI command implementation has been moved to `airssys-wasm-cli` package.
- **Package**: airssys-wasm-cli (library-only, 100% composable)
- **Task**: `.memory-bank/sub-projects/airssys-wasm-cli/tasks/task-cli-002-trust-command.md`
- **Commands**: add-git, add-key, add-local, list, remove, dev-mode, validate
- **Implementation**: Clap structures that compose library APIs

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

## CLI Integration

**Note**: CLI commands for trust management are implemented in a separate package:
- **Package**: `airssys-wasm-cli` (library-only, 100% reusable)
- **Task**: See `.memory-bank/sub-projects/airssys-wasm-cli/tasks/task-cli-002-trust-command.md`
- **Architecture**: airssys-wasm-cli exports Clap structures that can be composed by any binary

**APIs Exposed for CLI:**
- `TrustConfig::from_file()` - Load configuration
- `TrustConfig::to_toml()` - Serialize configuration
- `ConfigManager::load_config()` - Load with validation
- `ConfigManager::save_config()` - Save with backup
- `ConfigManager::backup_config()` - Create backup
- `ConfigValidator::validate_config()` - Validate configuration

**CLI Usage Pattern:**
```rust
use airssys_wasm::security::{TrustConfig, ConfigManager};

// CLI calls library functions
let config_manager = ConfigManager::new()?;
let mut config = config_manager.load_config()?;
config.add_git_source(&url, branch, &description)?;
config.validate()?;
config_manager.backup_config()?;
config_manager.save_config(&config)?;
```

---

## CLI Command Examples

**Note**: These CLI commands are implemented in `airssys-wasm-cli` package. This section provides reference examples of how the library APIs are used by CLI commands.

**See**: `.memory-bank/sub-projects/airssys-wasm-cli/tasks/task-cli-002-trust-command.md` for complete CLI implementation details.

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
| 12-15 | (See airssys-wasm-cli) | - | - |

**Total Duration**: 13.5 hours ≈ **2 days** (6-8 hour workdays)

**Breakdown by Activity**:
- Core implementation: 11 hours (81%)
- Testing: 2.5 hours (19%)

**Note**: CLI commands (Steps 12-15 from original plan) are implemented in airssys-wasm-cli package (estimated 6.5 additional hours for CLI-specific work).

---

## Performance Targets

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

## Task Summary: airssys-wasm Deliverables

### What This Task Delivers (airssys-wasm Package)

**Single New File:** `airssys-wasm/src/security/config.rs` (~1,670 lines)

**Contents:**
1. ✅ `TrustConfig` - TOML configuration struct
2. ✅ `TrustSettings` - Settings container (dev_mode + sources)
3. ✅ `TrustSourceConfig` - Source type enum (Git/SigningKey/Local)
4. ✅ `ConfigValidator` - Validation engine (13 rules)
5. ✅ `ConfigManager` - File operations (load/save/backup/verify)
6. ✅ `ConfigError` - Error types
7. ✅ 40+ comprehensive unit tests
8. ✅ Complete rustdoc documentation

**Module Export:** Update `src/security/mod.rs` (add `pub mod config;`)

**External Dependencies:** None (uses existing airssys-osl for audit logging)

**Timeline:** 13.5 hours ≈ 2 days

### What This Task Does NOT Deliver

**CLI Commands** → Separate package: `airssys-wasm-cli`
- ❌ Clap structures (TrustArgs, TrustCommands)
- ❌ User-facing commands (add-git, list, remove, etc.)
- ❌ Terminal output formatting
- ❌ Interactive prompts

**Reference:** `.memory-bank/sub-projects/airssys-wasm-cli/tasks/task-cli-002-trust-command.md`

### Public APIs for Consumers

**airssys-wasm-cli will consume these APIs:**

```rust
// Data structures
pub struct TrustConfig { /* ... */ }
pub struct TrustSettings { /* ... */ }
pub enum TrustSourceConfig { /* ... */ }

// Core operations
impl TrustConfig {
    pub fn from_file(path: &Path) -> Result<Self>;
    pub fn from_toml(toml: &str) -> Result<Self>;
    pub fn to_toml(&self) -> Result<String>;
    pub fn validate(&self) -> Result<()>;
    pub fn add_git_source(&mut self, url: &str, branch: Option<&str>, desc: &str);
    pub fn add_signing_key(&mut self, key: &str, signer: &str, desc: &str);
    pub fn add_local_path(&mut self, path: &str, desc: &str);
}

// File management
impl ConfigManager {
    pub fn new(config_path: PathBuf, backup_dir: PathBuf) -> Self;
    pub fn load_config(&self) -> Result<TrustConfig>;
    pub fn save_config(&self, config: &TrustConfig) -> Result<()>;
    pub fn backup_config(&self) -> Result<PathBuf>;
    pub fn verify_integrity(&self) -> Result<bool>;
}

// Validation
impl ConfigValidator {
    pub fn validate_config(config: &TrustConfig) -> Result<()>;
}
```

---

## Approval Status

**Planner**: Memory Bank Planner  
**Date**: 2025-12-18 (Updated)  
**Status**: ✅ **APPROVED** - Ready for implementation

**Scope Clarification (2025-12-18):**
- This plan now focuses EXCLUSIVELY on `airssys-wasm` core library
- CLI commands have been properly moved to `airssys-wasm-cli` (separate task)
- Clear file structure and deliverables defined
- Public APIs documented for future CLI consumption

This plan provides a comprehensive blueprint for implementing the trust configuration **core library** with robust validation, safe file operations, and production-ready documentation. CLI integration will be handled separately in the airssys-wasm-cli package.

**Ready to Start:** Task 2.3 implementation can begin immediately (Tasks 2.1-2.2 COMPLETE).



---

## Completion Summary

**Date Completed:** 2025-12-19  
**Auditor:** Memory Bank Auditor  
**Completion Report:** `task-005-phase-2-task-2.3-completion.md`

### Deliverables

✅ **All 11 Implementation Steps Complete:**
1. ✅ Config module structure (config.rs created, 3-layer imports)
2. ✅ TrustConfig types (TrustConfig, TrustSettings, TrustSourceConfig)
3. ✅ TOML parsing (from_toml, from_file, to_toml, save_to_file)
4. ✅ Git URL validation (5 tests passing)
5. ✅ Signing key validation (8 tests passing)
6. ✅ Local path validation (5 tests passing)
7. ✅ Duplicate detection (6 tests passing)
8. ✅ ConfigValidator (13 validation rules implemented)
9. ✅ ConfigManager core (load/save with tests)
10. ✅ Backup system (create/restore/list with cleanup)
11. ✅ Integrity verification (SHA-256 checksums)

### Quality Gates

✅ **All 5 Gates Passed:**
- Gate 1: Zero compiler warnings ✅
- Gate 2: Zero clippy warnings ✅
- Gate 3: All tests passing (770/770) ✅
- Gate 4: Docs build without warnings ✅
- Gate 5: Test coverage 64 tests (160% of target) ✅

### Metrics

- **File Size:** 2,437 lines (46% above estimate)
- **Test Count:** 64 tests (60% above plan target of 40)
- **Test Pass Rate:** 100% (770/770 total suite)
- **Documentation:** 145 lines module docs + comprehensive API docs
- **Integration:** Full Task 2.1 TrustRegistry integration verified

### Verification

- All checkboxes completed: ✅ (11/11 steps)
- All requirements met: ✅ (13 validation rules, TOML parsing, file ops, backup, integrity)
- Implementation verified: ✅ (config.rs 2,437 lines, 64 tests passing, integration working)
- Tests passing: 770/770 tests (100% pass rate)
- Code quality: 0 compiler warnings, 0 clippy warnings

### Production Readiness

✅ **APPROVED FOR PRODUCTION:**
- ✅ All planned features implemented
- ✅ Zero warnings (compiler + clippy + rustdoc)
- ✅ 100% test pass rate (770/770)
- ✅ Task 2.1 integration verified
- ✅ APIs ready for airssys-wasm-cli consumption
- ✅ Standards compliant (PROJECTS_STANDARD.md, Microsoft Guidelines, ADRs)
- ✅ Documentation complete (145 lines + API docs + 8 examples)

**Score:** 50/50 (100%)

**Status:** ✅ **COMPLETE** - Ready for Phase 3 (Capability Enforcement)

**Next:** Task 3.1 (Capability Check API) ready to start
