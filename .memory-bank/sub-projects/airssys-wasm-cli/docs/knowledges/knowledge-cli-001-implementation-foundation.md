# KNOWLEDGE-CLI-001: CLI Implementation Foundation

**Knowledge ID:** KNOWLEDGE-CLI-001  
**Created:** 2025-10-18  
**Last Updated:** 2025-10-18  
**Status:** Foundation Complete  
**Category:** Implementation Guide  
**Related:** KNOWLEDGE-WASM-009, KNOWLEDGE-WASM-010

---

## Overview

This document captures the foundational implementation knowledge for airssys-wasm-cli, establishing patterns, decisions, and best practices for the CLI tool development. It serves as the authoritative implementation guide for all 14 commands and their integration with the airssys-wasm core library.

---

## Architecture Foundations

### CLI Framework Selection: clap 4.5

**Decision:** Use clap with derive macros for command-line parsing

**Rationale:**
- Industry-standard Rust CLI framework with mature ecosystem
- Derive macros provide type-safe, declarative command definitions
- Automatic help generation and error handling
- Built-in shell completion support via clap_complete
- Strong integration with serde for configuration

**Implementation Pattern:**
```rust
#[derive(Parser)]
#[command(name = "airssys-wasm")]
#[command(about = "AirsSys WASM Component Management CLI")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate Ed25519 keypair for component signing
    Keygen(KeygenArgs),
    
    /// Initialize a new WASM component project
    Init(InitArgs),
    
    // ... all 14 commands
}
```

**Key Learnings:**
- Use `#[command(name = "binary-name")]` to set binary name (airssys-wasm, not airssys-wasm-cli)
- Doc comments on enum variants become command help text
- Subcommand structs should derive `Args` trait
- Global flags can be added to root `Cli` struct

---

### Module Naming Conflict Resolution

**Problem:** Module naming conflict between `mod config` and `commands::config`

**Original Structure (Broken):**
```rust
// src/lib.rs
pub mod config;           // Configuration management
pub mod commands {
    pub mod config;       // 'config' command
}
// Error: ambiguous module reference
```

**Solution:** Rename core module to `cli_config`

**Final Structure (Working):**
```rust
// src/lib.rs
pub mod cli_config;       // Configuration management (renamed)
pub mod commands {
    pub mod config;       // 'config' command (unchanged)
}
```

**Lessons Learned:**
- Avoid naming conflicts between core modules and command modules
- Use domain-specific prefixes for core modules (`cli_config`, not just `config`)
- Update all imports: `crate::config::CliConfig` → `crate::cli_config::CliConfig`
- Consider this pattern early in project setup to avoid refactoring

**Applied Pattern:**
- Core modules: Prefix with `cli_` if potential conflict exists
- Command modules: Use user-facing command names directly
- Test modules: Always use `#[cfg(test)]` to avoid namespace pollution

---

### Error Handling Architecture

**Pattern:** Enum-based errors with thiserror for user-friendly CLI errors

**Implementation:**
```rust
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Component error: {0}")]
    Component(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Security error: {0}")]
    Security(String),
    
    #[error("Git error: {0}")]
    Git(String),
}

pub type Result<T> = std::result::Result<T, CliError>;
```

**Key Decisions:**
- **String-based variant payloads:** Use `String` not `&str` for owned error messages
- **External error conversions:** Implement `From` traits for io::Error, serde, git2, reqwest
- **User-facing messages:** Error Display impl should be clear and actionable
- **Type alias:** `Result<T>` shorthand improves readability

**Error Display Strategy:**
```rust
// In command execution:
match operation() {
    Ok(_) => success("Operation completed"),
    Err(e) => {
        error(&format!("Operation failed: {}", e));
        std::process::exit(1); // Exit with error code
    }
}
```

**Lessons Learned:**
- Use `#[allow(dead_code)]` on error variants until implemented
- Add explanatory comments: `// TODO: Used when signature verification fails`
- Consider context-rich errors: add file paths, component names to error messages
- Plan for structured error data (for JSON output mode)

---

### UX Utilities Foundation

**Pattern:** Centralized UX utilities for consistent user experience

**Implementation:**
```rust
use colored::*;

pub fn success(msg: &str) {
    println!("{} {}", "✓".green().bold(), msg);
}

pub fn error(msg: &str) {
    eprintln!("{} {}", "✗".red().bold(), msg);
}

pub fn warning(msg: &str) {
    println!("{} {}", "⚠".yellow().bold(), msg);
}

pub fn info(msg: &str) {
    println!("{} {}", "ℹ".blue().bold(), msg);
}

pub fn create_spinner(msg: &str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_message(msg.to_string());
    spinner.enable_steady_tick(Duration::from_millis(100));
    spinner
}

pub fn create_progress_bar(total: u64) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb
}
```

**Key Decisions:**
- **Symbol + Color:** Use both for accessibility (works with color-blind users)
- **Consistent symbols:** ✓ (success), ✗ (error), ⚠ (warning), ℹ (info)
- **stderr for errors:** Use `eprintln!` not `println!` for error messages
- **indicatif integration:** Spinners for indeterminate progress, progress bars for known totals

**Lessons Learned:**
- Add `#[allow(dead_code)]` on utilities until commands use them
- Test unicode symbols work on all target platforms (Windows can be problematic)
- Consider `NO_COLOR` environment variable support for CI/CD environments
- Plan for JSON output mode where colored output is disabled

---

### Configuration Management

**Pattern:** TOML-based configuration with serde + defaults

**File Location:** `~/.airssys/config.toml`

**Structure:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    #[serde(default = "default_registry")]
    pub registry: String,
    
    #[serde(default = "default_install_dir")]
    pub install_dir: PathBuf,
    
    #[serde(default = "default_cache_dir")]
    pub cache_dir: PathBuf,
    
    #[serde(default = "default_keys_dir")]
    pub keys_dir: PathBuf,
}

fn default_registry() -> String {
    "https://registry.airsstack.org".to_string()
}

fn default_install_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".airssys")
        .join("components")
}

// ... other defaults
```

**Key Decisions:**
- **Use dirs crate:** Platform-independent home directory resolution
- **Fallback to defaults:** If config file missing, create with defaults
- **Graceful degradation:** If home directory unavailable, use current directory
- **No mandatory config:** CLI works out-of-box without manual config

**Loading Strategy:**
```rust
impl CliConfig {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path();
        
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            toml::from_str(&content)
                .map_err(|e| CliError::Config(format!("Parse error: {}", e)))
        } else {
            // First run - create default config
            let config = CliConfig::default();
            config.save()?;
            Ok(config)
        }
    }
    
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path();
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let content = toml::to_string_pretty(self)
            .map_err(|e| CliError::Config(format!("Serialize error: {}", e)))?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }
}
```

**Lessons Learned:**
- Create parent directories before writing config file
- Use `to_string_pretty` for human-readable TOML output
- Add validation in `load()` to check directory permissions
- Consider environment variable overrides for CI/CD

---

## Command Implementation Patterns

### Command Stub Pattern

**Current State:** All 14 commands implemented as stubs

**Stub Signature:**
```rust
pub async fn execute(_args: &CommandArgs) -> Result<()> {
    // TODO: Implement command logic
    Ok(())
}
```

**Lessons Learned:**
- Prefix unused args with `_` to suppress warnings
- Use `#[allow(dead_code)]` on arg structs until implemented
- Add comprehensive TODO comments explaining implementation steps
- Keep stub signatures async-ready for future I/O operations

**Implementation Readiness Checklist:**
- [ ] Args struct defined with all required fields
- [ ] Execute function signature matches async pattern
- [ ] Integration points with airssys-wasm identified
- [ ] Error cases documented in TODO comments
- [ ] UX flow documented (progress indication, output format)

---

### Integration with airssys-wasm Core Library

**Dependency Status:** airssys-wasm at 15% completion (architecture phase)

**Integration Strategy:**

**Phase 1 (Current):** Stub implementations
- No actual calls to airssys-wasm yet
- CLI structure and UX layer complete
- Ready for integration when core library APIs available

**Phase 2 (Q1 2026):** Core command implementation
- Requires: `Component`, `ComponentBuilder`, `ComponentRegistry` from airssys-wasm
- Commands: keygen, init, build, sign, install
- Pattern:
  ```rust
  use airssys_wasm::{Component, ComponentBuilder};
  
  pub async fn execute(_args: &BuildArgs) -> Result<()> {
      let builder = ComponentBuilder::new();
      let component = builder.build().await?;
      success("Component built successfully");
      Ok(())
  }
  ```

**Phase 3 (Q2 2026):** Management commands
- Requires: Runtime APIs, health monitoring from airssys-wasm
- Commands: update, uninstall, list, info, status, logs, verify
- Pattern:
  ```rust
  use airssys_wasm::ComponentRegistry;
  
  pub async fn execute(_args: &StatusArgs) -> Result<()> {
      let registry = ComponentRegistry::new()?;
      let status = registry.get_status(&args.component).await?;
      // Display status with colored output
      Ok(())
  }
  ```

**Bounded Context:**
- **CLI Responsibility:** User interaction, progress display, error formatting, configuration
- **Core Library Responsibility:** Component logic, registry operations, WASM runtime
- **Clear Interface:** CLI calls core library functions, doesn't implement component logic

**Lessons Learned:**
- Document integration points in TODO comments
- Reference KNOWLEDGE-WASM-009 and KNOWLEDGE-WASM-010 for API expectations
- Plan for API changes during core library development
- Keep CLI layer thin - logic belongs in core library

---

## Security Implementation

### Ed25519 Signing Integration

**Library:** ed25519-dalek 2.1 with rand_core

**Key Generation Pattern:**
```rust
use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signer, Verifier};
use rand::rngs::OsRng;

pub async fn execute(_args: &KeygenArgs) -> Result<()> {
    let mut csprng = OsRng;
    let keypair = Keypair::generate(&mut csprng);
    
    let keys_dir = CliConfig::load()?.keys_dir;
    std::fs::create_dir_all(&keys_dir)?;
    
    // Save private key (restrictive permissions)
    let private_path = keys_dir.join("private.pem");
    std::fs::write(&private_path, keypair.secret.to_bytes())?;
    #[cfg(unix)]
    std::fs::set_permissions(&private_path, std::fs::Permissions::from_mode(0o600))?;
    
    // Save public key
    let public_path = keys_dir.join("public.pem");
    std::fs::write(&public_path, keypair.public.to_bytes())?;
    
    success(&format!("Keypair generated at {:?}", keys_dir));
    Ok(())
}
```

**Signing Pattern:**
```rust
pub async fn execute(_args: &SignArgs) -> Result<()> {
    let component_bytes = std::fs::read(&args.component)?;
    let keypair = load_keypair(&args.key)?;
    
    let signature = keypair.sign(&component_bytes);
    
    // Store signature in component metadata
    let sig_path = args.component.with_extension("sig");
    std::fs::write(&sig_path, signature.to_bytes())?;
    
    success("Component signed successfully");
    Ok(())
}
```

**Verification Pattern:**
```rust
pub async fn execute(_args: &VerifyArgs) -> Result<()> {
    let component_bytes = std::fs::read(&args.component)?;
    let sig_bytes = std::fs::read(&args.signature)?;
    
    let public_key = load_public_key(&args.public_key)?;
    let signature = Signature::from_bytes(&sig_bytes)?;
    
    public_key.verify(&component_bytes, &signature)
        .map_err(|_| CliError::Security("Invalid signature".to_string()))?;
    
    success("Signature verified successfully");
    Ok(())
}
```

**Security Requirements:**
- Private keys: 0600 permissions (Unix), hidden directory (all platforms)
- Keys stored in `~/.airssys/keys/` by default
- Signature verification required for install/update commands (unless --no-verify)
- Clear warnings when signature verification is disabled

**Lessons Learned:**
- Use OS-level crypto RNG (OsRng) for key generation
- Test key generation on all platforms (Windows lacks some Unix permissions)
- Plan for key rotation and multiple keys
- Consider hardware security module (HSM) integration for production keys

---

## Testing Strategy

### Testing Layers

**1. Unit Tests** (per-module)
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_defaults() {
        let config = CliConfig::default();
        assert_eq!(config.registry, "https://registry.airsstack.org");
        assert!(config.install_dir.ends_with(".airssys/components"));
    }
    
    #[tokio::test]
    async fn test_keygen_args_validation() {
        let args = KeygenArgs { /* ... */ };
        // Test arg validation logic
    }
}
```

**2. Integration Tests** (CLI commands)
```rust
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn test_keygen_command() {
    let temp_dir = TempDir::new().unwrap();
    
    let mut cmd = Command::cargo_bin("airssys-wasm").unwrap();
    cmd.env("HOME", temp_dir.path()); // Override home directory
    cmd.arg("keygen");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Keypair generated"));
    
    // Verify keys were created
    assert!(temp_dir.path().join(".airssys/keys/private.pem").exists());
    assert!(temp_dir.path().join(".airssys/keys/public.pem").exists());
}
```

**3. End-to-End Tests** (complete workflows)
```rust
#[test]
fn test_full_component_workflow() {
    let temp_dir = TempDir::new().unwrap();
    
    // 1. Generate keys
    Command::cargo_bin("airssys-wasm").unwrap()
        .arg("keygen")
        .env("HOME", temp_dir.path())
        .assert()
        .success();
    
    // 2. Initialize component
    Command::cargo_bin("airssys-wasm").unwrap()
        .arg("init")
        .arg("test-component")
        .env("HOME", temp_dir.path())
        .assert()
        .success();
    
    // 3. Build component (when implemented)
    // 4. Sign component
    // 5. Install component
    // 6. Verify installation
}
```

**Testing Checklist:**
- [ ] Unit tests for all utility functions
- [ ] Integration tests for all 14 commands
- [ ] Error path testing (invalid args, missing files, network failures)
- [ ] Platform-specific tests (Unix permissions, Windows paths)
- [ ] Performance benchmarks for critical operations
- [ ] Security tests (key permissions, signature verification)

**Lessons Learned:**
- Use tempfile for isolated test environments
- Mock airssys-wasm interactions until core library ready
- Test both success and failure paths
- Use assert_cmd for CLI testing - simpler than manual subprocess management

---

## Build and Distribution

### Binary Configuration

**Package Name:** airssys-wasm-cli (crate name)  
**Binary Name:** airssys-wasm (user-facing command)

**Cargo.toml Configuration:**
```toml
[package]
name = "airssys-wasm-cli"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "airssys-wasm"
path = "src/main.rs"
```

**Rationale:**
- Crate name follows Rust conventions (descriptive)
- Binary name is user-friendly (short, memorable)
- Users type `airssys-wasm`, not `airssys-wasm-cli`

### Workspace Integration

**Workspace Member:**
```toml
# Root Cargo.toml
[workspace]
members = [
    "airssys-osl",
    "airssys-rt",
    "airssys-wasm",
    "airssys-wasm-component",
    "airssys-wasm-cli",  # Added
]
```

**Dependency Management:**
- All dependencies use `workspace = true` where available
- Layer-based organization: AirsSys crates → Core runtime → External
- Version management centralized in workspace root

**Lessons Learned:**
- Add workspace dependencies before adding to members
- Use layer-based dependency organization for clarity
- Document dependency rationale for future maintainers

---

## Performance Considerations

### Target Metrics

| Operation | Target | Rationale |
|-----------|--------|-----------|
| `keygen` | <100ms | CPU-bound Ed25519 generation |
| `sign` | <50ms | Small file signing is fast |
| `verify` | <200ms | Signature + file hash verification |
| `install` (cached) | <500ms | Local file operations |
| `install` (network) | <5s | Network-dependent, show progress |
| `list` | <100ms | Metadata query, should be instant |
| `status` | <200ms | Runtime health check |

### Optimization Strategies

**Async I/O:**
- Use tokio::fs for file operations (not std::fs)
- Concurrent downloads for multi-component operations
- Non-blocking network requests with reqwest

**Caching:**
- Cache downloaded components in `~/.airssys/cache/`
- Cache metadata queries to avoid repeated registry lookups
- Implement cache cleanup (LRU, size limits, age limits)

**Lazy Loading:**
- Don't load airssys-wasm unless command needs it
- Defer heavy imports until actually required
- Fast startup for simple commands (help, version, config)

**Lessons Learned:**
- Profile before optimizing - measure actual bottlenecks
- User perception matters more than absolute speed (show progress)
- CLI startup time is critical - keep it <100ms for simple commands

---

## Known Limitations and Future Work

### Current Limitations

**Phase 1 (Foundation) Limitations:**
- All commands are stubs - no actual functionality yet
- No tests implemented
- No integration with airssys-wasm core library
- Configuration management not yet tested
- Security operations not implemented

**Blocked on Dependencies:**
- Core commands (build, install) blocked on airssys-wasm Component APIs
- Management commands (status, logs) blocked on airssys-wasm runtime APIs
- Full workflow testing blocked on component execution

### Planned Enhancements (Future Phases)

**Phase 2 (Q1 2026) - Core Commands:**
- Implement keygen, init, build, sign, install
- Integration with airssys-wasm core library
- Basic error handling and recovery
- Integration test suite

**Phase 3 (Q2 2026) - Management Features:**
- Implement update, uninstall, list, info, status, logs, verify
- Health monitoring and log streaming
- Batch operations support
- Comprehensive error messages

**Phase 4 (Q3 2026) - Polish & Distribution:**
- Pre-built binaries for Linux, macOS, Windows
- GitHub Releases automation
- Homebrew formula
- Shell completion testing
- User documentation

**Future Possibilities:**
- Interactive installation wizard
- Component search functionality
- Dependency visualization
- Terminal UI (TUI) dashboard
- Plugin system for custom commands

---

## Decision Log

### ADR-CLI-001: Use clap 4.5 with Derive Macros

**Date:** 2025-10-18  
**Status:** Accepted  
**Context:** Need robust CLI parsing with shell completion support  
**Decision:** Use clap 4.5 with derive macros  
**Consequences:** Type-safe commands, automatic help generation, built-in completion  

### ADR-CLI-002: Rename config Module to cli_config

**Date:** 2025-10-18  
**Status:** Accepted  
**Context:** Module naming conflict between core and commands  
**Decision:** Rename core module to `cli_config`  
**Consequences:** Avoid ambiguity, clear naming, requires import updates  

### ADR-CLI-003: TOML Configuration Format

**Date:** 2025-10-18  
**Status:** Accepted  
**Context:** Need human-editable configuration  
**Decision:** Use TOML at ~/.airssys/config.toml  
**Consequences:** Human-readable, strong typing with serde, industry-standard  

### ADR-CLI-004: Ed25519 for Component Signing

**Date:** 2025-10-18  
**Status:** Accepted  
**Context:** Need fast, secure component signing  
**Decision:** Use ed25519-dalek 2.1  
**Consequences:** Fast signing/verification, small keys, widely supported  

### ADR-CLI-005: Binary Name "airssys-wasm"

**Date:** 2025-10-18  
**Status:** Accepted  
**Context:** Crate name vs. binary name trade-off  
**Decision:** Crate: airssys-wasm-cli, Binary: airssys-wasm  
**Consequences:** User-friendly command name, clear crate identity  

---

## Cross-References

### Related AirsSys Knowledge

**KNOWLEDGE-WASM-009: Installation Architecture**
- CLI install command integrates with installation subsystem
- Multi-source installation (local, Git, registry) patterns
- Installation verification and validation requirements

**KNOWLEDGE-WASM-010: CLI Tool Specification**
- Complete specification for all 14 commands
- Command requirements, arguments, output formats
- UX requirements and error handling

**KNOWLEDGE-WASM-001 through KNOWLEDGE-WASM-008:**
- Component Model architecture
- Registry design
- Security model
- Build system integration

### Memory Bank Files

**project_brief.md:** Project goals, scope, success criteria  
**tech_context.md:** Technical architecture, dependencies, performance targets  
**active_context.md:** Current development status, next steps  
**progress.md:** Development phases, milestones, metrics  
**system_patterns.md:** CLI architecture patterns, UX patterns, testing patterns  
**product_context.md:** User workflows, UX philosophy, documentation strategy  

---

## Maintenance

**Review Schedule:** After each phase completion  
**Update Triggers:**
- Integration with airssys-wasm core library
- Command implementation completion
- Architectural pattern changes
- Security vulnerability discoveries

**Ownership:** airssys-wasm-cli maintainers  
**Approval Required:** For architectural changes, security patterns  

---

**Document Version:** 1.0  
**Last Review:** 2025-10-18  
**Next Review:** When airssys-wasm reaches 50% completion
