# Technical Context: airssys-wasm-cli

**Sub-Project:** airssys-wasm-cli  
**Last Updated:** 2025-10-18  
**Phase:** Foundation Setup (10% complete)

---

## Technical Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────┐
│ airssys-wasm CLI                                            │
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │   Commands   │  │  CLI Config  │  │   Utilities  │     │
│  │  (14 cmds)   │  │  Management  │  │  (UX, Crypto)│     │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘     │
│         │                  │                  │             │
│         └──────────────────┴──────────────────┘             │
│                            │                                │
│                     ┌──────▼───────┐                        │
│                     │ Error Handler│                        │
│                     └──────┬───────┘                        │
└────────────────────────────┼────────────────────────────────┘
                             │
                    ┌────────▼────────┐
                    │  airssys-wasm   │ (Core Library)
                    │  Component APIs │
                    └─────────────────┘
```

### Module Organization

```
src/
├── main.rs              # CLI entry point, command routing
├── cli_config.rs        # Configuration management
├── error.rs             # Error types and conversions
├── utils.rs             # Shared utilities (UX, formatting)
└── commands/
    ├── mod.rs           # Command module exports
    ├── keygen.rs        # Ed25519 keypair generation
    ├── init.rs          # Component project initialization
    ├── build.rs         # WASM component building
    ├── sign.rs          # Component signing
    ├── install.rs       # Multi-source installation
    ├── update.rs        # Component updates
    ├── uninstall.rs     # Component removal
    ├── list.rs          # List installed components
    ├── info.rs          # Component details
    ├── logs.rs          # Log viewing/streaming
    ├── status.rs        # Health checking
    ├── verify.rs        # Signature verification
    ├── config.rs        # Configuration commands
    └── completions.rs   # Shell completions
```

---

## Technology Stack

### Core Dependencies

**CLI Framework:**
- `clap` 4.5 - Modern CLI parsing with derive macros
- `clap_complete` 4.5 - Shell completion generation

**Runtime:**
- `tokio` 1.47 - Async runtime for concurrent operations
- `env_logger` 0.11 - Logging infrastructure

**User Experience:**
- `colored` 3.0 - Colored terminal output
- `indicatif` 0.17 - Progress bars and spinners
- `console` 0.15 - Terminal utilities
- `dialoguer` 0.11 - Interactive prompts

**Cryptography:**
- `ed25519-dalek` 2.1 - Ed25519 digital signatures
- `rand` 0.8 - Cryptographic randomness
- `base64` 0.22 - Key encoding

**Data Handling:**
- `serde` 1.0 - Serialization framework
- `serde_json` 1.0 - JSON support
- `toml` 0.9.7 - TOML config files

**External Operations:**
- `git2` 0.18 - Git repository operations
- `reqwest` 0.12 - HTTP client
- `walkdir` 2.5 - Directory traversal

**File System:**
- `tempfile` 3.20 - Temporary file management
- `dirs` 6.0 - Standard directories

**Error Handling:**
- `thiserror` 2.0 - Structured error types
- `anyhow` 1.0 - Error context

### Development Dependencies

- `assert_cmd` 2.0 - CLI testing
- `predicates` 3.1 - Assertion helpers
- `tempfile` 3.20 - Test fixtures

---

## Performance Targets

### Command Execution

| Command | Target Latency | Notes |
|---------|---------------|-------|
| `keygen` | < 100ms | Ed25519 key generation |
| `init` | < 200ms | Project scaffolding |
| `build` | Variable | Depends on component size |
| `sign` | < 50ms | Signature generation |
| `install` (local) | < 500ms | File-based installation |
| `install` (Git) | Variable | Network dependent |
| `list` | < 100ms | Registry query |
| `info` | < 50ms | Component metadata |
| `verify` | < 100ms | Signature verification |

### Resource Constraints

- **Memory**: < 50MB for typical operations
- **Disk**: Minimal temp space for downloads
- **Network**: Efficient streaming for large components

---

## Data Models

### Configuration Structure

```toml
# ~/.airssys/config.toml

# Storage backend (sled or rocksdb)
storage_backend = "sled"

# Component storage directory
storage_path = "/Users/user/.airssys/components"

# Registry cache directory
cache_dir = "/Users/user/.airssys/cache"

# Ed25519 keypair path (optional)
keypair_path = "/Users/user/.airssys/keypair.json"

# Default Git branch for installations
default_branch = "main"

# Enable automatic signature verification
auto_verify = true

# Default output format (text, json, yaml)
output_format = "text"
```

### Keypair Format

```json
{
  "public_key": "base64-encoded-public-key",
  "private_key": "base64-encoded-private-key",
  "algorithm": "ed25519",
  "created_at": "2025-10-18T10:30:00Z"
}
```

### Component Manifest (Component.toml)

Documented in KNOWLEDGE-WASM-009. Key sections:
- `[component]` - Name, version, description
- `[component.metadata]` - License, repository, tags
- `[build]` - Language, target, build commands
- `[permissions]` - Filesystem, network, environment
- `[signature]` - Public key, algorithm

---

## Security Architecture

### Cryptographic Security

**Ed25519 Digital Signatures:**
- Public-key cryptography for component ownership
- 256-bit security level
- Fast signature generation and verification
- Compact signatures (64 bytes)

**Key Management:**
- Keys stored in `~/.airssys/keypair.json`
- File permissions: 0600 (user read/write only)
- Never transmit private keys
- Public keys embedded in signed components

### Installation Security

**Multi-Layer Verification:**
1. **Manifest Validation** - TOML syntax and required fields
2. **WASM Validation** - Valid WebAssembly binary
3. **Signature Verification** - Ed25519 signature check
4. **Capability Validation** - Permission requirements
5. **Host Policy** - Final authorization decision

**Git Security:**
- HTTPS or SSH for repository access
- Commit hash verification
- No arbitrary code execution during clone

**URL Downloads:**
- HTTPS required for remote URLs
- TLS certificate validation
- Content-type verification

---

## Integration Points

### airssys-wasm Core Library

```rust
// CLI uses core library for component operations
use airssys_wasm::{
    Component,
    ComponentRegistry,
    ComponentMetadata,
    Verifier,
    InstallOptions,
};

// Example: Install command integration
async fn install_component(source: &str) -> Result<()> {
    let registry = ComponentRegistry::new()?;
    let options = InstallOptions::from_source(source)?;
    let metadata = registry.install(options).await?;
    Ok(())
}
```

### Language Toolchain Integration

**Rust Components:**
```bash
cargo build --target wasm32-wasi --release
```

**Go Components:**
```bash
GOOS=wasip1 GOARCH=wasm go build -o component.wasm
```

**Component Model Conversion:**
```bash
wasm-tools component new component.wasm -o component.component.wasm
```

### CI/CD Integration

**JSON Output:**
```bash
airssys-wasm list --output json | jq '.[] | select(.status=="running")'
```

**Exit Codes:**
- 0: Success
- 1: General error
- 2: Invalid arguments
- 3: Component not found
- 4: Signature verification failed
- 5: Permission denied

---

## Error Handling Strategy

### Error Type Hierarchy

```rust
pub enum CliError {
    Config(String),              // Configuration errors
    ComponentNotFound(String),   // Missing components
    InstallationFailed(String),  // Installation errors
    SignatureVerificationFailed(String), // Crypto errors
    BuildFailed(String),         // Build errors
    GitError(String),            // Git operation errors
    Io(io::Error),              // File system errors
    Serialization(String),       // Data format errors
    InvalidArgument(String),     // CLI argument errors
    PermissionDenied(String),    // Authorization errors
    Network(String),             // HTTP/network errors
    Other(anyhow::Error),       // Catch-all
}
```

### Error Conversion

Automatic conversion from external error types:
- `std::io::Error` → `CliError::Io`
- `serde_json::Error` → `CliError::Serialization`
- `toml::de::Error` → `CliError::Serialization`
- `git2::Error` → `CliError::GitError`
- `reqwest::Error` → `CliError::Network`

### User-Facing Error Messages

- Actionable error messages with suggestions
- Context about what operation failed
- Hints for common resolution steps
- Links to documentation when appropriate

---

## Configuration Management

### Configuration File Location

**Priority Order:**
1. `--config` CLI flag (highest priority)
2. `AIRSSYS_CONFIG` environment variable
3. `~/.airssys/config.toml` (default)
4. Built-in defaults (fallback)

### Environment Variables

- `AIRSSYS_CONFIG` - Custom config file path
- `AIRSSYS_STORAGE_BACKEND` - Override storage backend
- `AIRSSYS_STORAGE_PATH` - Override storage path
- `AIRSSYS_KEYPAIR_PATH` - Override keypair location
- `AIRSSYS_LOG` - Logging level (error, warn, info, debug, trace)

### Runtime Configuration

```rust
pub struct CliConfig {
    pub storage_backend: String,
    pub storage_path: PathBuf,
    pub cache_dir: PathBuf,
    pub keypair_path: Option<PathBuf>,
    pub default_branch: String,
    pub auto_verify: bool,
    pub output_format: String,
}
```

---

## Testing Strategy

### Test Categories

**Unit Tests:**
- Error conversion logic
- Configuration parsing
- Utility functions (format_bytes, etc.)

**Integration Tests:**
- Command execution end-to-end
- Configuration management
- Error handling flows

**CLI Tests:**
```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_keygen_command() {
    let mut cmd = Command::cargo_bin("airssys-wasm").unwrap();
    cmd.arg("keygen")
        .arg("--output")
        .arg("test_keypair.json")
        .assert()
        .success()
        .stdout(predicate::str::contains("Keypair generated"));
}
```

### Test Fixtures

- Temporary directories for component installation
- Mock Git repositories
- Sample Component.toml files
- Test keypairs (not for production use)

---

## Build and Distribution

### Build Configuration

```toml
[[bin]]
name = "airssys-wasm"
path = "src/main.rs"
```

### Release Artifacts

**crates.io:**
```bash
cargo publish --package airssys-wasm-cli
```

**Pre-built Binaries:**
- Linux (x86_64, aarch64)
- macOS (x86_64, aarch64)
- Windows (x86_64)

**Homebrew Formula:**
```ruby
class AirssysWasm < Formula
  desc "CLI for airssys-wasm component management"
  homepage "https://github.com/airsstack/airssys"
  url "https://github.com/airsstack/airssys/releases/download/v0.1.0/airssys-wasm-cli-0.1.0.tar.gz"
  sha256 "..."
  
  def install
    bin.install "airssys-wasm"
  end
end
```

---

## Development Workflow

### Local Development

```bash
# Build CLI
cargo build --package airssys-wasm-cli

# Run CLI
cargo run --package airssys-wasm-cli -- keygen

# Test CLI
cargo test --package airssys-wasm-cli

# Check code quality
cargo clippy --package airssys-wasm-cli --all-targets --all-features -- -D warnings
```

### Release Process

1. Update version in Cargo.toml
2. Run full test suite
3. Generate changelog
4. Tag release
5. Publish to crates.io
6. Build release binaries
7. Create GitHub Release
8. Update Homebrew formula

---

## Performance Optimization

### Future Optimizations

**Caching:**
- Cache Git repository clones
- Cache downloaded components
- Cache verification results

**Parallelization:**
- Parallel component installations
- Concurrent signature verification
- Parallel component list queries

**Binary Size:**
- Link-time optimization (LTO)
- Strip debug symbols in release
- Minimize dependency tree

---

## Monitoring and Observability

### Logging

```rust
// Structured logging with tracing
tracing::info!(
    component = %name,
    version = %version,
    "Installing component"
);
```

### Telemetry (Future)

- Optional usage analytics
- Error reporting (opt-in)
- Performance metrics

---

## Related Documentation

**Knowledge Base:**
- KNOWLEDGE-WASM-009: Component Installation Architecture
- KNOWLEDGE-WASM-010: CLI Tool Specification
- KNOWLEDGE-WASM-001: Component Framework Architecture

**Workspace Standards:**
- §2.1: 3-Layer Import Organization
- §3.2: chrono DateTime<Utc> Standard
- §5.1: Dependency Management
- §6.1: YAGNI Principles
- Microsoft Rust Guidelines (M-DESIGN-FOR-AI, M-DI-HIERARCHY, etc.)

---

**Next Technical Tasks:**
1. Implement Ed25519 key generation in keygen.rs
2. Create project templates for init.rs
3. Implement language detection and build orchestration in build.rs
4. Implement component signing in sign.rs
5. Implement multi-source installation in install.rs
