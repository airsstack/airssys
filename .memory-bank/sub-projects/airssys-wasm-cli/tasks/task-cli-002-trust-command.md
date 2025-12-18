# TASK-CLI-002: Trust Command Implementation

**Task ID:** TASK-CLI-002  
**Status:** 📋 Pending  
**Priority:** High  
**Created:** 2025-12-18  
**Estimated Duration:** 6-8 hours  
**Dependencies:** WASM-TASK-005 Phase 2 Task 2.3 (Trust Configuration System - Core Library)

---

## Executive Summary

**What**: Implement trust management CLI commands (10 commands total) using composable Clap structures. Provides add/remove/list functionality for Git sources, signing keys, and local paths, plus DevMode management and configuration validation.

**Why**: Administrators need intuitive CLI commands to manage trusted component sources without manually editing TOML files. The CLI must provide clear feedback, validation, and safety prompts for sensitive operations.

**How**: Create Clap-based command structures (`TrustArgs`, `TrustCommands`) that compose airssys-wasm core library APIs (`ConfigManager`, `TrustConfig`, `ConfigValidator`). Export from lib.rs for composability by airsstack binary. Implement 10 commands with UX utilities (colored output, confirmations, progress indicators).

**Architecture Position**: This task implements the **CLI layer only** (100% library, zero binary). The CLI structures are exported from `airssys-wasm-cli` and can be composed by any binary application. Core trust management logic resides in `airssys-wasm`.

---

## Dependencies

### External Dependencies
- **WASM-TASK-005 Phase 2 Task 2.3** - Trust Configuration System (Core Library)
  - Status: Must complete first
  - Required APIs:
    - `TrustConfig::from_file()` / `to_toml()`
    - `ConfigManager::load_config()` / `save_config()` / `backup_config()`
    - `ConfigValidator::validate_config()`
    - `TrustSourceConfig` enum variants

### Internal Dependencies
- **TASK-CLI-001** - Foundation Setup ✅ Complete
  - Error handling (`CliError`)
  - UX utilities (`success`, `error`, `warning`, `info`)
  - Configuration management patterns

---

## Implementation Plan

### Subtask 2.1: Create Clap Structures (1.5 hours)

**Deliverables**:
- `TrustArgs` struct with subcommands
- `TrustCommands` enum (10 variants)
- Argument structures for each command

**Implementation**:

```rust
// In src/commands/trust.rs (new file structure)

use clap::{Args, Subcommand};

/// Trust management commands
#[derive(Debug, Args)]
pub struct TrustArgs {
    #[command(subcommand)]
    pub command: TrustCommands,
}

/// Trust subcommands
#[derive(Debug, Subcommand)]
pub enum TrustCommands {
    /// Add trusted Git repository source
    AddGit {
        /// Git URL pattern (supports wildcards)
        #[arg(value_name = "URL")]
        url: String,
        
        /// Optional branch restriction
        #[arg(short, long)]
        branch: Option<String>,
        
        /// Description of this source
        #[arg(short, long)]
        description: String,
    },
    
    /// Add trusted signing key
    AddKey {
        /// Ed25519 public key (format: ed25519:BASE64)
        #[arg(value_name = "PUBLIC_KEY")]
        public_key: String,
        
        /// Signer identity
        #[arg(short, long)]
        signer: String,
        
        /// Description of this key
        #[arg(short, long)]
        description: String,
    },
    
    /// Add trusted local path
    AddLocal {
        /// Local path pattern
        #[arg(value_name = "PATH")]
        path: String,
        
        /// Description of this path
        #[arg(short, long)]
        description: String,
    },
    
    /// List all trusted sources
    List,
    
    /// Remove trusted source by index
    Remove {
        /// Source index (from list command)
        #[arg(value_name = "INDEX")]
        index: usize,
    },
    
    /// Enable development mode (bypasses all security checks)
    EnableDevMode,
    
    /// Disable development mode
    DisableDevMode,
    
    /// Show current DevMode status
    DevModeStatus,
    
    /// Validate current trust configuration
    Validate,
    
    /// Show configuration file path
    ShowPath,
}
```

**Checkpoint**: Clap structures compile, integrate with main.rs command routing.

---

### Subtask 2.2: Implement Execute Function (10 Commands) (3 hours)

**Deliverables**:
- `execute()` function with match on `TrustCommands`
- Implementation for all 10 commands
- Integration with airssys-wasm APIs
- Error handling and user feedback

**Implementation**:

```rust
use crate::{error::Result, utils::{success, error, warning, info}};
use airssys_wasm::security::{ConfigManager, TrustConfig, ConfigValidator};
use colored::Colorize;

/// Execute trust management command
pub async fn execute(args: &TrustArgs) -> Result<()> {
    let config_manager = ConfigManager::new()?;
    
    match &args.command {
        TrustCommands::AddGit { url, branch, description } => {
            add_git_source(&config_manager, url, branch.as_deref(), description).await
        }
        
        TrustCommands::AddKey { public_key, signer, description } => {
            add_signing_key(&config_manager, public_key, signer, description).await
        }
        
        TrustCommands::AddLocal { path, description } => {
            add_local_path(&config_manager, path, description).await
        }
        
        TrustCommands::List => {
            list_sources(&config_manager).await
        }
        
        TrustCommands::Remove { index } => {
            remove_source(&config_manager, *index).await
        }
        
        TrustCommands::EnableDevMode => {
            enable_dev_mode(&config_manager).await
        }
        
        TrustCommands::DisableDevMode => {
            disable_dev_mode(&config_manager).await
        }
        
        TrustCommands::DevModeStatus => {
            show_dev_mode_status(&config_manager).await
        }
        
        TrustCommands::Validate => {
            validate_config(&config_manager).await
        }
        
        TrustCommands::ShowPath => {
            show_config_path(&config_manager).await
        }
    }
}

// Command implementations (examples)

async fn add_git_source(
    manager: &ConfigManager,
    url: &str,
    branch: Option<&str>,
    description: &str,
) -> Result<()> {
    // Load current config
    let mut config = manager.load_config()?;
    
    // Create new source
    let source = TrustSourceConfig::Git {
        url: url.to_string(),
        branch: branch.map(String::from),
        description: description.to_string(),
    };
    
    // Validate before adding
    ConfigValidator::validate_source(&source)?;
    
    // Add to config
    config.trust.sources.push(source);
    
    // Validate full config
    config.validate()?;
    
    // Backup existing config
    let backup_path = manager.backup_config()?;
    
    // Save new config
    manager.save_config(&config)?;
    
    // User feedback
    success("Added trusted Git repository");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("URL Pattern: {}", url.bold());
    if let Some(b) = branch {
        println!("Branch:      {}", b);
    }
    println!("Description: {}", description);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info(&format!("Backup created: {}", backup_path.display()));
    
    Ok(())
}

async fn list_sources(manager: &ConfigManager) -> Result<()> {
    let config = manager.load_config()?;
    
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Trusted Sources ({})", config.trust.sources.len());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    for (i, source) in config.trust.sources.iter().enumerate() {
        println!();
        print!("[{}] ", (i + 1).to_string().bold());
        
        match source {
            TrustSourceConfig::Git { url, branch, description } => {
                println!("Git: {}", url.green());
                if let Some(b) = branch {
                    println!("    Branch: {}", b);
                }
                println!("    Description: {}", description);
            }
            TrustSourceConfig::SigningKey { public_key, signer, description } => {
                let truncated = if public_key.len() > 30 {
                    format!("{}... (truncated)", &public_key[..30])
                } else {
                    public_key.clone()
                };
                println!("SigningKey: {}", truncated.yellow());
                println!("    Signer: {}", signer);
                println!("    Description: {}", description);
            }
            TrustSourceConfig::Local { path_pattern, description } => {
                println!("Local: {}", path_pattern.blue());
                println!("    Description: {}", description);
            }
        }
    }
    
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    Ok(())
}

async fn enable_dev_mode(manager: &ConfigManager) -> Result<()> {
    use dialoguer::Confirm;
    
    // Safety confirmation
    warning("⚠️  ⚠️  ⚠️  ENABLE DEVELOPMENT MODE? ⚠️  ⚠️  ⚠️");
    println!();
    println!("This will BYPASS ALL SECURITY CHECKS!");
    println!();
    println!("Components will have UNRESTRICTED ACCESS to:");
    println!("  • All filesystem paths (read/write/execute)");
    println!("  • All network endpoints (inbound/outbound)");
    println!("  • All storage namespaces (unlimited)");
    println!();
    warning("⚠️  NEVER use DevMode in production!");
    warning("⚠️  Only use for local development and testing!");
    println!();
    
    let confirmed = Confirm::new()
        .with_prompt("Type 'yes' to confirm")
        .default(false)
        .interact()?;
    
    if !confirmed {
        info("DevMode enable cancelled");
        return Ok(());
    }
    
    // Load, modify, save
    let mut config = manager.load_config()?;
    config.trust.dev_mode = true;
    manager.backup_config()?;
    manager.save_config(&config)?;
    
    warning("⚠️  DevMode ENABLED");
    
    Ok(())
}

// ... additional command implementations
```

**Checkpoint**: All 10 commands functional, proper error handling, user-friendly output.

---

### Subtask 2.3: Integration Testing (2 hours)

**Deliverables**:
- Integration tests for all 10 commands
- Test fixtures (sample configs, temp directories)
- Edge case testing

**Implementation**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_add_git_source() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("trust-config.toml");
        
        // Create manager with temp path
        let manager = ConfigManager::with_path(config_path);
        
        // Execute add command
        let result = add_git_source(
            &manager,
            "https://github.com/myorg/*",
            Some("main"),
            "Test repository",
        ).await;
        
        assert!(result.is_ok());
        
        // Verify config file
        let config = manager.load_config().unwrap();
        assert_eq!(config.trust.sources.len(), 1);
        
        match &config.trust.sources[0] {
            TrustSourceConfig::Git { url, branch, description } => {
                assert_eq!(url, "https://github.com/myorg/*");
                assert_eq!(branch.as_deref(), Some("main"));
                assert_eq!(description, "Test repository");
            }
            _ => panic!("Expected Git source"),
        }
    }
    
    #[tokio::test]
    async fn test_list_sources() {
        // Test empty list
        // Test list with multiple sources
        // Test list output formatting
    }
    
    #[tokio::test]
    async fn test_remove_source() {
        // Test remove valid index
        // Test remove invalid index
        // Test remove with backup creation
    }
    
    #[tokio::test]
    async fn test_validate_config() {
        // Test valid config
        // Test invalid config (bad URL, bad key, etc.)
    }
    
    // ... additional tests for all commands
}
```

**Checkpoint**: All tests passing, >90% code coverage, edge cases handled.

---

### Subtask 2.4: Documentation and Export (1.5 hours)

**Deliverables**:
- Rustdoc for all public functions
- Export from lib.rs for composability
- README examples
- Usage documentation

**Implementation**:

```rust
// In src/lib.rs (ensure trust module is exported)

pub mod commands {
    pub mod trust;
    // ... other command modules
}

// Re-export for convenience
pub use commands::trust::{TrustArgs, TrustCommands, execute as execute_trust};
```

**Documentation**:
- Module-level rustdoc explaining composable CLI pattern
- Function-level rustdoc for `execute()` and all helper functions
- Examples in README showing how to compose in a binary

**Checkpoint**: Zero rustdoc warnings, clear examples, ready for composition.

---

## Data Structures

### TrustArgs (Clap Structure)
```rust
#[derive(Debug, Args)]
pub struct TrustArgs {
    #[command(subcommand)]
    pub command: TrustCommands,
}
```

### TrustCommands (Enum)
```rust
#[derive(Debug, Subcommand)]
pub enum TrustCommands {
    AddGit { url: String, branch: Option<String>, description: String },
    AddKey { public_key: String, signer: String, description: String },
    AddLocal { path: String, description: String },
    List,
    Remove { index: usize },
    EnableDevMode,
    DisableDevMode,
    DevModeStatus,
    Validate,
    ShowPath,
}
```

---

## Integration with airssys-wasm

### Core Library APIs Used

```rust
use airssys_wasm::security::{
    ConfigManager,        // File operations
    TrustConfig,          // Configuration structure
    TrustSourceConfig,    // Source variants
    ConfigValidator,      // Validation logic
};
```

### API Usage Pattern

```rust
// 1. Create manager
let manager = ConfigManager::new()?;

// 2. Load config
let mut config = manager.load_config()?;

// 3. Modify config
config.trust.sources.push(new_source);

// 4. Validate
config.validate()?;

// 5. Backup
manager.backup_config()?;

// 6. Save
manager.save_config(&config)?;
```

---

## User Experience Examples

### Example 1: Add Git Source
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
ℹ  Backup created: /etc/airssys/backups/trust-config-2025-12-18T10-30-45.toml
```

### Example 2: List Sources
```bash
$ airssys-wasm trust list

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Trusted Sources (3)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[1] Git: https://github.com/mycompany/*
    Branch: main
    Description: Internal company repositories

[2] SigningKey: ed25519:AAAAC3NzaC1lZDI1NTE... (truncated)
    Signer: engineering@mycompany.com
    Description: Engineering team signing key

[3] Local: /opt/verified-components/*
    Description: System-verified components

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### Example 3: Enable DevMode (with Safety Prompt)
```bash
$ airssys-wasm trust enable-dev-mode

⚠️  ⚠️  ⚠️  ENABLE DEVELOPMENT MODE? ⚠️  ⚠️  ⚠️

This will BYPASS ALL SECURITY CHECKS!

Components will have UNRESTRICTED ACCESS to:
  • All filesystem paths (read/write/execute)
  • All network endpoints (inbound/outbound)
  • All storage namespaces (unlimited)

⚠️  NEVER use DevMode in production!
⚠️  Only use for local development and testing!

Type 'yes' to confirm: yes

⚠️  DevMode ENABLED
```

---

## Success Criteria

### Functional Requirements ✅
- [ ] All 10 commands implemented and functional
- [ ] Proper integration with airssys-wasm ConfigManager
- [ ] Validation errors displayed clearly
- [ ] Backup created before modifications
- [ ] Colored output for all operations
- [ ] Interactive confirmation for dangerous operations (DevMode)

### Quality Requirements ✅
- [ ] Zero compilation warnings
- [ ] Zero clippy warnings
- [ ] All integration tests passing
- [ ] >90% code coverage
- [ ] Zero rustdoc warnings
- [ ] Follows workspace standards (§2.1, §3.2, §4.3, §5.1)

### Composability Requirements ✅
- [ ] `TrustArgs` and `TrustCommands` exported from lib.rs
- [ ] `execute()` function is public and async
- [ ] Can be composed by any binary using Clap's `flatten` or `subcommand`
- [ ] Clear documentation on composition pattern

---

## Testing Strategy

### Unit Tests (10 tests)
- Clap structure parsing
- Command routing logic
- Error handling

### Integration Tests (12 tests)
- Add Git source (valid and invalid)
- Add signing key (valid and invalid)
- Add local path (valid and invalid)
- List sources (empty, single, multiple)
- Remove source (valid index, invalid index)
- DevMode enable/disable
- Validate config (valid, invalid)
- Backup creation

### Edge Case Tests (8 tests)
- Very long URLs
- Unicode in descriptions
- Duplicate sources
- Concurrent config modifications
- Missing config file
- Corrupted config file
- Invalid permissions
- Empty sources list

**Total**: 30+ test cases

---

## Timeline Estimate

| Subtask | Description | Time | Cumulative |
|---------|-------------|------|------------|
| 2.1 | Clap structures | 1.5 hours | 1.5 hours |
| 2.2 | Implement 10 commands | 3 hours | 4.5 hours |
| 2.3 | Integration testing | 2 hours | 6.5 hours |
| 2.4 | Documentation & export | 1.5 hours | **8 hours** |

**Total Duration**: 8 hours ≈ **1 day** (full workday)

**Buffer**: +1-2 hours for unexpected issues

---

## Quality Gates

### Pre-Implementation Checklist
- [x] WASM-TASK-005 Phase 2 Task 2.3 complete (core library)
- [x] Foundation setup complete (TASK-CLI-001)
- [x] Dependencies available (airssys-wasm APIs)

### Implementation Checkpoints
- [ ] Subtask 2.1: Clap structures compile
- [ ] Subtask 2.2: All commands functional
- [ ] Subtask 2.3: All tests passing
- [ ] Subtask 2.4: Zero rustdoc warnings

### Final Verification
```bash
# Compilation
cargo check --package airssys-wasm-cli

# Linting
cargo clippy --package airssys-wasm-cli --all-targets -- -D warnings

# Testing
cargo test --package airssys-wasm-cli

# Documentation
cargo doc --package airssys-wasm-cli --no-deps
```

---

## Standards Compliance

### PROJECTS_STANDARD.md
- §2.1: 3-layer import organization ✅
- §4.3: Module architecture (mod.rs re-exports) ✅
- §5.1: Dependency management (workspace deps) ✅
- §6.1: YAGNI principles (no premature features) ✅

### Microsoft Rust Guidelines
- M-DESIGN-FOR-AI: Clear API, extensive docs ✅
- M-CANONICAL-DOCS: Comprehensive public API docs ✅
- M-EXAMPLES: Examples for all commands ✅
- M-DI-HIERARCHY: Proper dependency injection (ConfigManager) ✅

---

## Related Documentation

### Tasks
- `task-cli-001-foundation-setup.md` - CLI foundation
- `../../airssys-wasm/tasks/task-005-phase-2-task-2.3-plan.md` - Core library implementation

### Knowledge Base
- `../docs/knowledges/knowledge-cli-002-composable-cli-pattern.md` - Composable CLI pattern
- `../docs/knowledges/knowledge_cli_001_implementation_foundation.md` - Foundation knowledge

### Architectural Decisions
- `../docs/adr/adr-cli-001-library-only-architecture.md` - Library-only architecture decision

### Core Files
- `../tech_context.md` - Technical architecture
- `../system_patterns.md` - Implementation patterns
- `../active_context.md` - Current priorities

---

## Risk Assessment

| Risk | Severity | Probability | Mitigation |
|------|----------|-------------|------------|
| airssys-wasm API changes | Medium | Low | Well-defined interface in Task 2.3 plan |
| Complex DevMode UX | Low | Low | Use dialoguer for confirmation |
| Config file corruption | Medium | Low | Backup before all modifications |
| Performance on large configs | Low | Low | ConfigManager optimized in core library |

---

## Approval

**Status**: 📋 Pending (Awaits WASM-TASK-005 Phase 2 Task 2.3 completion)  
**Created By**: Memory Bank Planner  
**Date**: 2025-12-18  
**Ready to Start**: When airssys-wasm ConfigManager APIs are available

This task provides a comprehensive implementation plan for trust management CLI commands with clear integration points, testing strategy, and composability requirements.
