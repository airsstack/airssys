# System Patterns: airssys-wasm-cli

**Sub-Project:** airssys-wasm-cli  
**Last Updated:** 2025-10-18  
**Pattern Categories:** CLI Architecture, Error Handling, Configuration, UX, Testing

---

## §1 CLI Architecture Patterns

### §1.1 Command Structure Pattern

**Pattern:** Consistent command module structure with execute function signature

```rust
// Every command follows this structure
pub async fn execute(_args: &CliArgs) -> Result<()> {
    // 1. Validate inputs
    // 2. Load configuration if needed
    // 3. Perform operation
    // 4. Provide user feedback
    // 5. Return Result
    Ok(())
}
```

**Rationale:**
- Consistent interface across all 14 commands
- Easy to test and mock
- Clear separation of concerns
- Async-ready for I/O operations

**Example (from keygen.rs):**
```rust
pub async fn execute(_args: &KeygenArgs) -> Result<()> {
    // TODO: Implement Ed25519 keypair generation
    // 1. Generate keypair using ed25519-dalek + rand_core
    // 2. Save to ~/.airssys/keys/ directory
    // 3. Display success message with colored output
    Ok(())
}
```

### §1.2 Command Registration Pattern

**Pattern:** Centralized command enum with derive macros

```rust
#[derive(Parser)]
#[command(name = "airssys-wasm")]
#[command(about = "AirsSys WASM Component Management CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Keygen(KeygenArgs),
    Init(InitArgs),
    Build(BuildArgs),
    // ... all 14 commands
}
```

**Benefits:**
- Single source of truth for commands
- Type-safe command routing
- Automatic help generation
- Shell completion support

---

## §2 Error Handling Patterns

### §2.1 Custom Error Type Pattern

**Pattern:** Enum-based errors with thiserror and conversions

```rust
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Component error: {0}")]
    Component(String),
    
    // ... specific error variants
}

pub type Result<T> = std::result::Result<T, CliError>;
```

**Rationale:**
- Clear error categorization
- Automatic conversion from external error types
- Custom error messages for CLI context
- Type alias simplifies function signatures

**Usage Pattern:**
```rust
pub async fn execute(_args: &BuildArgs) -> Result<()> {
    let config = CliConfig::load()?; // auto-converts io::Error
    
    if !config.is_valid() {
        return Err(CliError::Config("Invalid configuration".to_string()));
    }
    
    // ... operation
    Ok(())
}
```

### §2.2 User-Friendly Error Display Pattern

**Pattern:** Colored, contextual error messages with utils

```rust
use crate::utils::{error, warning};

// Instead of returning raw errors to user:
match operation() {
    Ok(_) => success("Operation completed successfully"),
    Err(e) => {
        error(&format!("Failed to complete operation: {}", e));
        std::process::exit(1);
    }
}
```

**Benefits:**
- Consistent error presentation
- Color-coded severity (red for errors, yellow for warnings)
- Clear user guidance
- Professional CLI UX

---

## §3 Configuration Management Patterns

### §3.1 TOML Configuration Pattern

**Pattern:** Struct-based config with serde + default values

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    #[serde(default = "default_registry")]
    pub registry: String,
    
    #[serde(default = "default_install_dir")]
    pub install_dir: PathBuf,
    
    #[serde(default = "default_cache_dir")]
    pub cache_dir: PathBuf,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            registry: default_registry(),
            install_dir: default_install_dir(),
            cache_dir: default_cache_dir(),
        }
    }
}
```

**Location:** `~/.airssys/config.toml`

**Benefits:**
- Human-readable format
- Easy to edit manually
- Strong typing with serde
- Graceful defaults

### §3.2 Configuration Loading Pattern

**Pattern:** Load with fallback to defaults

```rust
impl CliConfig {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path();
        
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: CliConfig = toml::from_str(&content)
                .map_err(|e| CliError::Config(format!("Parse error: {}", e)))?;
            Ok(config)
        } else {
            // Create default config and save
            let config = CliConfig::default();
            config.save()?;
            Ok(config)
        }
    }
}
```

**Rationale:**
- No explicit initialization required
- Safe first-run experience
- Recoverable from missing/corrupt config

---

## §4 UX Patterns

### §4.1 Progress Indicator Pattern

**Pattern:** Use indicatif for long-running operations

```rust
use crate::utils::create_spinner;

pub async fn execute(_args: &BuildArgs) -> Result<()> {
    let spinner = create_spinner("Building component...");
    
    // Perform long operation
    build_component().await?;
    
    spinner.finish_with_message("✓ Build complete");
    success("Component built successfully");
    Ok(())
}
```

**Use Cases:**
- Network operations (install, update)
- Build operations
- File I/O operations
- Git operations

### §4.2 Colored Output Pattern

**Pattern:** Semantic color coding with colored crate

```rust
use crate::utils::{success, error, warning, info};

// Success (green)
success("Component installed successfully");

// Error (red)
error("Failed to verify signature");

// Warning (yellow)
warning("Component not found in cache, downloading...");

// Info (blue)
info("Using registry: https://registry.airsstack.org");
```

**Benefits:**
- Visual hierarchy
- Quick scanning
- Professional appearance
- Accessibility (can be disabled)

### §4.3 Interactive Prompt Pattern

**Pattern:** Use dialoguer for user input

```rust
use dialoguer::{Confirm, Input, Select};

// Confirmation
let confirmed = Confirm::new()
    .with_prompt("Overwrite existing component?")
    .default(false)
    .interact()?;

// Text input
let name: String = Input::new()
    .with_prompt("Component name")
    .interact()?;

// Selection
let options = vec!["local", "git", "registry"];
let selection = Select::new()
    .with_prompt("Installation source")
    .items(&options)
    .interact()?;
```

**Use Cases:**
- Component name in `init`
- Overwrite confirmation in `install`
- Template selection in `init`

---

## §5 Testing Patterns

### §5.1 Command Testing Pattern

**Pattern:** Use assert_cmd for CLI integration testing

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_keygen_command() {
    let mut cmd = Command::cargo_bin("airssys-wasm").unwrap();
    cmd.arg("keygen");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Keypair generated"));
}
```

**Benefits:**
- Full CLI integration testing
- Tests actual binary behavior
- Predicates for flexible assertions
- End-to-end validation

### §5.2 Unit Testing Pattern

**Pattern:** Mock external dependencies, test logic

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_defaults() {
        let config = CliConfig::default();
        assert_eq!(config.registry, "https://registry.airsstack.org");
    }
    
    #[tokio::test]
    async fn test_validate_args() {
        let args = BuildArgs { /* ... */ };
        assert!(validate_build_args(&args).is_ok());
    }
}
```

**Testing Layers:**
1. Unit tests for pure functions
2. Integration tests for command execution
3. CLI tests for end-to-end workflows

---

## §6 Async Patterns

### §6.1 Tokio Runtime Pattern

**Pattern:** Use tokio for all async operations

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI with clap
    let cli = Cli::parse();
    
    // Route to command handler (all async)
    match cli.command {
        Commands::Build(args) => commands::build::execute(&args).await,
        Commands::Install(args) => commands::install::execute(&args).await,
        // ...
    }
}
```

**Benefits:**
- Concurrent I/O operations
- Efficient resource usage
- Non-blocking network/file operations
- Consistent async model

### §6.2 Parallel Operations Pattern

**Pattern:** Use tokio::spawn for parallel tasks

```rust
pub async fn execute(_args: &UpdateArgs) -> Result<()> {
    let components = list_installed_components()?;
    
    let tasks: Vec<_> = components
        .into_iter()
        .map(|comp| tokio::spawn(async move {
            check_for_update(&comp).await
        }))
        .collect();
    
    let results = futures::future::join_all(tasks).await;
    
    // Process results...
    Ok(())
}
```

**Use Cases:**
- Parallel component updates
- Concurrent downloads
- Batch signature verification

---

## §7 Security Patterns

### §7.1 Ed25519 Signing Pattern

**Pattern:** Use ed25519-dalek for component signing

```rust
use ed25519_dalek::{Keypair, PublicKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;

// Generate keypair
let mut csprng = OsRng;
let keypair = Keypair::generate(&mut csprng);

// Sign component
let component_bytes = std::fs::read("component.wasm")?;
let signature = keypair.sign(&component_bytes);

// Verify signature
let public_key: PublicKey = /* load from registry */;
public_key.verify(&component_bytes, &signature)
    .map_err(|_| CliError::Security("Invalid signature".to_string()))?;
```

**Security Requirements:**
- Private keys stored in `~/.airssys/keys/` with 0600 permissions
- Signatures stored in component metadata
- Multi-layer verification (build-time, install-time, runtime)

### §7.2 Secure File Operations Pattern

**Pattern:** Validate paths and permissions

```rust
use std::fs::{self, Permissions};
use std::os::unix::fs::PermissionsExt;

pub fn save_private_key(path: &Path, key: &[u8]) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    // Write key
    fs::write(path, key)?;
    
    // Set restrictive permissions (Unix only)
    #[cfg(unix)]
    fs::set_permissions(path, Permissions::from_mode(0o600))?;
    
    Ok(())
}
```

**Security Considerations:**
- Validate all file paths before operations
- Set proper permissions on sensitive files
- Prevent directory traversal attacks
- Sanitize user input in file paths

---

## §8 Performance Patterns

### §8.1 Caching Pattern

**Pattern:** Local cache for downloads and metadata

```rust
pub struct ComponentCache {
    cache_dir: PathBuf,
}

impl ComponentCache {
    pub fn get(&self, component_id: &str, version: &str) -> Result<Option<PathBuf>> {
        let cache_path = self.cache_dir
            .join(component_id)
            .join(version)
            .join("component.wasm");
        
        if cache_path.exists() {
            Ok(Some(cache_path))
        } else {
            Ok(None)
        }
    }
    
    pub fn store(&self, component_id: &str, version: &str, data: &[u8]) -> Result<PathBuf> {
        let cache_path = self.cache_dir
            .join(component_id)
            .join(version)
            .join("component.wasm");
        
        fs::create_dir_all(cache_path.parent().unwrap())?;
        fs::write(&cache_path, data)?;
        Ok(cache_path)
    }
}
```

**Benefits:**
- Avoid redundant downloads
- Faster repeated operations
- Offline capability for cached components

### §8.2 Lazy Loading Pattern

**Pattern:** Load heavy dependencies only when needed

```rust
// Don't load airssys-wasm until command needs it
pub async fn execute(_args: &BuildArgs) -> Result<()> {
    // Only import/load when actually building
    use airssys_wasm::{Component, ComponentBuilder};
    
    let builder = ComponentBuilder::new();
    // ... build operation
}
```

**Benefits:**
- Faster CLI startup
- Reduced memory for simple commands
- Pay-for-what-you-use model

---

## §9 Logging Patterns

### §9.1 Structured Logging Pattern

**Pattern:** Use tracing for structured logs (future)

```rust
use tracing::{info, debug, warn, error};

pub async fn execute(_args: &InstallArgs) -> Result<()> {
    info!(
        component = %args.component,
        source = %args.source,
        "Starting component installation"
    );
    
    debug!("Checking cache for component");
    
    // ... operation
    
    info!("Component installed successfully");
    Ok(())
}
```

**Levels:**
- ERROR: Operation failures
- WARN: Non-fatal issues
- INFO: Major operation steps
- DEBUG: Detailed operation info
- TRACE: Very detailed debugging

### §9.2 Log Rotation Pattern (for `logs` command)

**Pattern:** Rotate logs by size and time

```rust
pub struct LogManager {
    log_dir: PathBuf,
    max_size: u64,
    max_age_days: u64,
}

impl LogManager {
    pub fn rotate_if_needed(&self) -> Result<()> {
        let current_log = self.log_dir.join("airssys.log");
        
        if let Ok(metadata) = fs::metadata(&current_log) {
            if metadata.len() > self.max_size {
                self.rotate_log()?;
            }
        }
        
        self.cleanup_old_logs()?;
        Ok(())
    }
}
```

---

## §10 Integration Patterns

### §10.1 airssys-wasm Integration Pattern

**Pattern:** Use core library APIs for component operations

```rust
use airssys_wasm::{Component, ComponentRegistry};

pub async fn execute(_args: &InstallArgs) -> Result<()> {
    // Use core library for component operations
    let registry = ComponentRegistry::new()?;
    let component = Component::load(&args.source).await?;
    
    // Verify signature using core library
    component.verify_signature()?;
    
    // Install using core library
    registry.install(component).await?;
    
    Ok(())
}
```

**Key Dependencies:**
- Component loading and parsing
- Signature verification
- Registry operations
- Installation/update logic

**Bounded Context:**
- CLI handles user interaction, progress display
- Core library handles component logic
- Clear separation of concerns

---

## §11 Distribution Patterns

### §11.1 Pre-Built Binary Pattern

**Pattern:** Cross-platform builds with GitHub Actions

```yaml
# .github/workflows/release.yml
jobs:
  build:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
      - run: cargo build --release
      - run: ./scripts/package.sh
```

**Targets:**
- Linux (x86_64-unknown-linux-gnu)
- macOS (x86_64-apple-darwin, aarch64-apple-darwin)
- Windows (x86_64-pc-windows-msvc)

### §11.2 Shell Completion Pattern

**Pattern:** Generate completions for all major shells

```rust
use clap_complete::{generate, Shell};

pub async fn execute(_args: &CompletionsArgs) -> Result<()> {
    let mut cli = Cli::command();
    
    match args.shell {
        Shell::Bash => generate(Shell::Bash, &mut cli, "airssys-wasm", &mut io::stdout()),
        Shell::Zsh => generate(Shell::Zsh, &mut cli, "airssys-wasm", &mut io::stdout()),
        Shell::Fish => generate(Shell::Fish, &mut cli, "airssys-wasm", &mut io::stdout()),
        Shell::PowerShell => generate(Shell::PowerShell, &mut cli, "airssys-wasm", &mut io::stdout()),
    }
    
    Ok(())
}
```

**Installation:**
```bash
# Bash
airssys-wasm completions bash > ~/.airssys/completions/airssys-wasm.bash
source ~/.airssys/completions/airssys-wasm.bash

# Zsh
airssys-wasm completions zsh > ~/.airssys/completions/_airssys-wasm
```

---

## Pattern Evolution

### Planned Patterns (Future Phases)

**Template System:**
- Integrate handlebars or tera for component templates
- Support custom template repositories
- Version-controlled template management

**Build Orchestration:**
- Support multiple build tools (Rust, Go, AssemblyScript)
- Plugin system for custom build steps
- Build caching and incremental compilation

**Progress UI Enhancements:**
- Multi-operation progress bars (parallel installs)
- Real-time log streaming during builds
- Interactive installation wizard

---

## Anti-Patterns to Avoid

### ❌ Synchronous Blocking I/O
```rust
// AVOID: Blocks entire runtime
let content = std::fs::read_to_string("large_file.txt")?;

// PREFER: Use tokio for I/O
let content = tokio::fs::read_to_string("large_file.txt").await?;
```

### ❌ Unwrap/Expect in Library Code
```rust
// AVOID: Panics on error
let config = load_config().expect("Config must exist");

// PREFER: Return Result
let config = load_config()?;
```

### ❌ Hardcoded Paths
```rust
// AVOID: Non-portable
let config_path = "/home/user/.airssys/config.toml";

// PREFER: Use dirs crate
let config_path = dirs::home_dir()
    .unwrap()
    .join(".airssys")
    .join("config.toml");
```

### ❌ Silent Failures
```rust
// AVOID: User doesn't know what happened
if let Err(_) = install_component() {
    return Ok(());
}

// PREFER: Proper error handling
install_component().map_err(|e| {
    error(&format!("Installation failed: {}", e));
    e
})?;
```

---

## Related Documentation

- tech_context.md - Technical architecture decisions
- active_context.md - Current development patterns
- KNOWLEDGE-WASM-010 - CLI command specifications
- workspace/shared_patterns.md - Workspace-wide code standards
