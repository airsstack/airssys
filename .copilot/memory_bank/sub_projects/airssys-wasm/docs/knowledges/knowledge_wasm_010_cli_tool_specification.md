# KNOWLEDGE-WASM-010: CLI Tool Specification (airssys-wasm-cli)

**Status:** Complete  
**Created:** 2025-10-18  
**Last Updated:** 2025-10-18  
**Project Status:** Foundation setup complete - standalone workspace member  
**Related ADRs:** None yet  
**Related Tasks:** None yet  
**Dependencies:** KNOWLEDGE-WASM-009

---

## Table of Contents

1. [Overview](#overview)
2. [Project Structure](#project-structure)
3. [CLI Design Philosophy](#cli-design-philosophy)
4. [Installation and Setup](#installation-and-setup)
5. [Command Structure](#command-structure)
6. [Core Commands](#core-commands)
7. [Configuration Management](#configuration-management)
8. [Output Formatting](#output-formatting)
9. [Error Handling](#error-handling)
10. [Implementation Architecture](#implementation-architecture)
11. [Examples and Use Cases](#examples-and-use-cases)
12. [Future Enhancements](#future-enhancements)

---

## Overview

### Purpose

The `airssys-wasm-cli` tool provides a command-line interface for component developers to manage the complete lifecycle of WASM components: initialization, building, signing, installation, updates, and uninstallation. The CLI is designed to be intuitive, powerful, and aligned with modern CLI best practices.

**Project Location:** `airssys/airssys-wasm-cli/` (standalone workspace member)

### Scope

**In Scope:**
- Component project initialization
- Cryptographic key management
- Component signing and verification
- Installation from multiple sources (Git, file, URL)
- Component updates and uninstallation
- Multi-component orchestration
- Configuration management
- Status monitoring and logging

**Out of Scope (Separate Tools):**
- Host runtime server (separate binary)
- Visual component composition (future GUI tool)
- Component registry service (future infrastructure)

---

## Project Structure

`airssys-wasm-cli` is a **standalone workspace member** within the AirsSys monorepo:

```
airssys/
├── Cargo.toml                    # Workspace root
├── airssys-wasm/                 # Core WASM library (dependency)
├── airssys-wasm-component/       # Procedural macros (independent)
└── airssys-wasm-cli/             # CLI tool (this project)
    ├── Cargo.toml                # Binary crate configuration
    ├── README.md                 # CLI documentation
    ├── src/
    │   ├── main.rs               # CLI entry point
    │   ├── commands/             # Command implementations
    │   │   ├── mod.rs
    │   │   ├── keygen.rs
    │   │   ├── init.rs
    │   │   ├── build.rs
    │   │   ├── sign.rs
    │   │   ├── install.rs
    │   │   ├── update.rs
    │   │   ├── uninstall.rs
    │   │   ├── list.rs
    │   │   ├── info.rs
    │   │   ├── logs.rs
    │   │   ├── status.rs
    │   │   ├── verify.rs
    │   │   ├── config.rs
    │   │   └── completions.rs
    │   ├── config.rs             # Configuration management
    │   ├── error.rs              # Error types
    │   └── utils.rs              # Shared utilities
    ├── tests/                    # Integration tests
    └── docs/                     # CLI-specific documentation
```

**Workspace Integration:**

```toml
# Root Cargo.toml
[workspace]
members = [
    "airssys-wasm",
    "airssys-wasm-component",
    "airssys-wasm-cli",           # ← CLI as workspace member
]

# airssys-wasm-cli/Cargo.toml
[dependencies]
airssys-wasm = { workspace = true }  # Uses local workspace dependency
clap = { workspace = true }
# ... other CLI-specific dependencies
```

**Distribution:**
- Published separately to crates.io
- Can be installed via `cargo install airssys-wasm-cli`
- Pre-built binaries via GitHub Releases
- Binary name: `airssys-wasm` (not `airssys-wasm-cli`)

**Development Status:** Foundation setup phase (10% complete)
- ✅ Directory structure created
- ✅ Cargo.toml with dependencies configured
- ✅ All 14 command stubs implemented
- ✅ Error handling and utilities in place
- ⏳ Command implementations pending (future work)

### Design Philosophy

**Modern CLI Best Practices:**
- **Intuitive**: Commands follow natural language patterns
- **Progressive Disclosure**: Simple commands for beginners, advanced options for experts
- **Helpful**: Clear error messages with actionable suggestions
- **Fast**: Minimal startup time, efficient operations
- **Reliable**: Robust error handling, safe defaults
- **Scriptable**: Machine-readable output formats (JSON, YAML)

---

## CLI Design Philosophy

### Influenced By

**Excellent CLI Examples:**
- `cargo` (Rust) - Clear commands, helpful errors, excellent UX
- `git` (Git) - Powerful, composable, standard command structure
- `kubectl` (Kubernetes) - Resource management, declarative configuration
- `solana` (Solana CLI) - Crypto key management, deployment workflows
- `near` (NEAR CLI) - Developer-friendly blockchain tooling

### Core Principles

**1. Convention Over Configuration**
- Smart defaults that work for 90% of use cases
- Override only when necessary
- Auto-detection of common patterns

**2. Fail Fast with Helpful Messages**
```bash
# BAD: Cryptic error
Error: failed to load component

# GOOD: Actionable error message
Error: Failed to load WASM binary

Caused by:
  File not found: target/wasm32-unknown-unknown/release/my_plugin.wasm

Hint: Did you forget to build your component?
  Run: cargo build --target wasm32-unknown-unknown --release
```

**3. Progressive Disclosure**
```bash
# Simple command (beginner-friendly)
airssys-wasm install --from-git https://github.com/user/plugin

# Advanced command (power user)
airssys-wasm install \
  --from-git https://github.com/user/plugin \
  --ref v1.0.0 \
  --build-from-source \
  --capabilities capabilities.json \
  --deployment-strategy canary \
  --host https://prod.company.com \
  --verify-signature \
  --log-level debug
```

**4. Safe by Default**
```bash
# Dangerous operations require confirmation
airssys-wasm uninstall comp_abc123
# Warning: This will permanently delete component "data-processor"
# Type 'data-processor' to confirm: _

# Or force with flag (scripting)
airssys-wasm uninstall comp_abc123 --force
```

---

## Installation and Setup

### Installation Methods

```bash
# Method 1: Cargo (Rust developers)
cargo install airssys-wasm-cli

# Method 2: Pre-built binaries (GitHub Releases)
# macOS (Intel)
curl -L https://github.com/airsstack/airssys/releases/download/v1.0.0/airssys-wasm-cli-macos-x86_64.tar.gz | tar xz
sudo mv airssys-wasm-cli /usr/local/bin/

# macOS (Apple Silicon)
curl -L https://github.com/airsstack/airssys/releases/download/v1.0.0/airssys-wasm-cli-macos-aarch64.tar.gz | tar xz
sudo mv airssys-wasm-cli /usr/local/bin/

# Linux
curl -L https://github.com/airsstack/airssys/releases/download/v1.0.0/airssys-wasm-cli-linux-x86_64.tar.gz | tar xz
sudo mv airssys-wasm-cli /usr/local/bin/

# Method 3: Package managers (future)
brew install airssys-wasm-cli  # macOS
apt install airssys-wasm-cli   # Debian/Ubuntu
dnf install airssys-wasm-cli   # Fedora/RHEL

# Verify installation
airssys-wasm --version
# airssys-wasm-cli 1.0.0
```

### Initial Setup

```bash
# One-time setup wizard
airssys-wasm init

# Interactive prompts:
# ? Generate new keypair? (Y/n): Y
# ? Keypair location (~/.airssys/keypair.json): [Enter]
# ✓ Keypair generated
#   Public Key: a1b2c3d4e5f6789abc...
#   
# ? Default host URL (http://localhost:8080): [Enter]
# ✓ Configuration saved to ~/.airssys/config.toml
# 
# Setup complete! You can now use airssys-wasm to manage components.
# 
# Next steps:
#   - Create a component: airssys-wasm new my-plugin
#   - View help: airssys-wasm --help
```

### Environment Variables

```bash
# Configuration via environment variables
export AIRSSYS_WASM_HOST=https://runtime.company.com
export AIRSSYS_WASM_KEYPAIR=~/.airssys/keypair.json
export AIRSSYS_WASM_LOG_LEVEL=debug

# Override config file settings
airssys-wasm install --from-git <url>
# Uses AIRSSYS_WASM_HOST automatically
```

---

## Command Structure

### Command Hierarchy

```
airssys-wasm
├── keygen              # Generate new keypair
├── init                # Initialize new component project
├── build               # Build component
├── sign                # Sign component
├── install             # Install component to host
├── update              # Update existing component
├── uninstall           # Uninstall component
├── list                # List installed components
├── info                # Show component information
├── logs                # View component logs
├── status              # Check component status
├── verify              # Verify component signature
├── config              # Manage configuration
│   ├── get             # Get config value
│   ├── set             # Set config value
│   └── list            # List all config
└── completions         # Generate shell completions
```

### Global Flags

```bash
--host <URL>           # Override host runtime URL
--keypair <PATH>       # Override keypair file path
--config <PATH>        # Override config file path
--log-level <LEVEL>    # Set log level (error, warn, info, debug, trace)
--output <FORMAT>      # Output format (text, json, yaml)
--no-color             # Disable colored output
--quiet                # Minimal output
--verbose              # Verbose output
--help                 # Show help
--version              # Show version
```

---

## Core Commands

### 1. keygen - Generate Keypair

Generate new Ed25519 keypair for signing components.

```bash
# Basic usage
airssys-wasm keygen

# Specify output location
airssys-wasm keygen --output ~/.airssys/my-key.json

# With passphrase encryption (future)
airssys-wasm keygen --output ~/.airssys/my-key.json --encrypt

# Show public key only (for sharing)
airssys-wasm keygen --show-public-key

# Output:
# Generated new Ed25519 keypair
# 
# Public Key:  a1b2c3d4e5f6789abcdef0123456789abcdef0123456789abcdef0123456789a
# Private Key: (saved to /Users/dev/.airssys/keypair.json)
# 
# ⚠️  KEEP YOUR PRIVATE KEY SAFE!
# 
# This key controls your components. Store it securely:
#   - Password manager (1Password, LastPass, Bitwarden)
#   - Hardware security module (YubiKey, Ledger)
#   - Encrypted backup (separate from your code)
# 
# Never commit this key to version control!
```

**Implementation:**
```rust
pub struct KeygenCommand {
    output: PathBuf,
    encrypt: bool,
    show_public_key: bool,
}

impl KeygenCommand {
    pub fn execute(&self) -> Result<()> {
        // Generate keypair
        let keypair = ComponentSigner::generate_keypair();
        
        // Encrypt if requested (future)
        let key_data = if self.encrypt {
            self.encrypt_keypair(&keypair)?
        } else {
            KeyData {
                public_key: hex::encode(keypair.public.as_bytes()),
                secret_key: hex::encode(keypair.secret.as_bytes()),
            }
        };
        
        // Save to file
        let json = serde_json::to_string_pretty(&key_data)?;
        std::fs::write(&self.output, json)?;
        
        // Set secure permissions (Unix)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&self.output)?.permissions();
            perms.set_mode(0o600); // rw-------
            std::fs::set_permissions(&self.output, perms)?;
        }
        
        // Print output
        println!("Generated new Ed25519 keypair\n");
        println!("Public Key:  {}", key_data.public_key);
        println!("Private Key: (saved to {})", self.output.display());
        println!("\n⚠️  KEEP YOUR PRIVATE KEY SAFE!");
        
        Ok(())
    }
}
```

### 2. init - Initialize Component Project

Create new component project with boilerplate code.

```bash
# Interactive initialization
airssys-wasm init

# With project name
airssys-wasm init my-plugin

# With language selection
airssys-wasm init my-plugin --lang rust

# With template
airssys-wasm init my-plugin --template data-processor

# Output:
# Creating new component project: my-plugin
# 
# ? Component type: 
#   ❯ Data Processor
#     API Gateway
#     Event Handler
#     Custom
# 
# ? Programming language:
#   ❯ Rust
#     Go
#     C/C++
#     JavaScript/TypeScript
# 
# ? Include examples? (Y/n): Y
# 
# ✓ Project created successfully!
# 
# Project structure:
#   my-plugin/
#   ├── Component.toml
#   ├── Cargo.toml
#   ├── src/
#   │   └── lib.rs
#   ├── wit/
#   │   └── component.wit
#   └── examples/
#       └── basic_usage.rs
# 
# Next steps:
#   cd my-plugin
#   airssys-wasm build
#   airssys-wasm install --from-file ./target/release/my_plugin.wasm
```

**Generated Component.toml:**
```toml
[package]
name = "my-plugin"
version = "0.1.0"
description = "My awesome WASM component"
authors = ["Your Name <you@example.com>"]
license = "MIT"

[build]
type = "rust"
target = "wasm32-unknown-unknown"
artifact_path = "target/wasm32-unknown-unknown/release/my_plugin.wasm"

[capabilities]
[capabilities.file_system]
read = []
write = []

[capabilities.network]
outbound = []
inbound = []

[capabilities.storage]
quota_mb = 10
persistent = true

[security]
# Run 'airssys-wasm sign' to add signature

[deployment]
strategy = "blue-green"
min_instances = 1
max_instances = 10
```

### 3. build - Build Component

Build WASM component from source.

```bash
# Build current project
airssys-wasm build

# Build with specific profile
airssys-wasm build --release

# Build with optimizations
airssys-wasm build --release --optimize

# Build and show output
airssys-wasm build --verbose

# Output:
# Building component: my-plugin v0.1.0
# 
# Build type: Rust
# Target: wasm32-unknown-unknown
# Profile: release
# 
# Running: cargo build --target wasm32-unknown-unknown --release
# 
#    Compiling my-plugin v0.1.0 (/Users/dev/my-plugin)
#     Finished release [optimized] target(s) in 15.2s
# 
# ✓ Build successful!
#   Artifact: target/wasm32-unknown-unknown/release/my_plugin.wasm
#   Size: 245.3 KB
```

**Implementation:**
```rust
pub struct BuildCommand {
    project_dir: Option<PathBuf>,
    profile: BuildProfile,
    optimize: bool,
    verbose: bool,
}

impl BuildCommand {
    pub async fn execute(&self) -> Result<()> {
        let project_dir = self.project_dir.clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap());
        
        // Read manifest
        let manifest_path = project_dir.join("Component.toml");
        let manifest = ComponentManifest::from_toml_file(&manifest_path)?;
        
        println!("Building component: {} v{}", 
            manifest.package.name, manifest.package.version);
        
        // Build using engine
        let build_engine = BuildEngine::new();
        let wasm_bytes = build_engine
            .build_component(&project_dir, &manifest)
            .await?;
        
        let artifact_path = project_dir.join(&manifest.build.artifact_path);
        let size_kb = wasm_bytes.len() as f64 / 1024.0;
        
        println!("\n✓ Build successful!");
        println!("  Artifact: {}", artifact_path.display());
        println!("  Size: {:.1} KB", size_kb);
        
        Ok(())
    }
}
```

### 4. sign - Sign Component

Sign component with private key.

```bash
# Sign component (uses default keypair)
airssys-wasm sign

# Sign with specific keypair
airssys-wasm sign --keypair ~/.airssys/my-key.json

# Sign specific files
airssys-wasm sign \
  --manifest Component.toml \
  --wasm target/release/my_plugin.wasm \
  --keypair ~/.airssys/keypair.json

# Output:
# Signing component: my-plugin v0.1.0
# 
# WASM Binary: target/wasm32-unknown-unknown/release/my_plugin.wasm
# Size: 245.3 KB
# Hash: sha256:abc123def456...
# 
# Keypair: /Users/dev/.airssys/keypair.json
# Public Key: a1b2c3d4e5f6...
# 
# ✓ Component signed successfully!
# 
# Updated Component.toml:
#   [security]
#   author_public_key = "a1b2c3d4e5f6..."
#   signature = "0123456789ab..."
# 
# Your component is now ready to install:
#   airssys-wasm install --from-file ./target/release/my_plugin.wasm
```

### 5. install - Install Component

Install component to host runtime.

```bash
# Install from Git (clone → build → deploy)
airssys-wasm install --from-git https://github.com/user/my-plugin
airssys-wasm install --from-git https://github.com/user/my-plugin --ref v1.0.0

# Install from local file (fast iteration)
airssys-wasm install \
  --from-file ./target/release/my_plugin.wasm \
  --manifest Component.toml

# Install from URL (pre-built binary)
airssys-wasm install \
  --from-url https://releases.company.com/plugins/processor-v1.0.0.wasm \
  --manifest https://releases.company.com/plugins/Component.toml

# With host override
airssys-wasm install --from-git <url> --host https://prod.company.com

# With capability overrides
airssys-wasm install --from-git <url> --capabilities custom-caps.toml

# Dry run (validate without deploying)
airssys-wasm install --from-git <url> --dry-run

# Output:
# Installing component from Git...
# 
# Repository: https://github.com/user/my-plugin
# Reference: v1.0.0
# 
# [1/6] Cloning repository...        ✓ (2.3s)
# [2/6] Checking out v1.0.0...       ✓ (0.1s)
# [3/6] Reading manifest...          ✓
# [4/6] Building component...        ✓ (18.5s)
# [5/6] Verifying signature...       ✓
#   Author: a1b2c3d4...
#   Signed: 2025-10-18T10:30:00Z
# [6/6] Deploying to host...         ✓ (1.2s)
# 
# ✓ Component installed successfully!
# 
# Component ID: comp_abc123
# Name: my-plugin
# Version: 1.0.0
# Status: Running
# Instances: 2
# 
# View logs:
#   airssys-wasm logs comp_abc123 --follow
```

**Implementation:**
```rust
pub struct InstallCommand {
    source: InstallSource,
    host: Option<String>,
    capabilities: Option<PathBuf>,
    dry_run: bool,
}

impl InstallCommand {
    pub async fn execute(&self) -> Result<()> {
        let installer = ComponentInstaller::new()?;
        
        let options = InstallOptions {
            host_url: self.host.clone()
                .or_else(|| config::get_default_host()),
            capability_overrides: self.load_capabilities()?,
            deployment_strategy: DeploymentStrategy::BlueGreen,
            dry_run: self.dry_run,
        };
        
        let component_id = installer
            .install(self.source.clone(), options)
            .await?;
        
        println!("\n✓ Component installed successfully!");
        println!("\nComponent ID: {}", component_id);
        
        Ok(())
    }
}
```

### 6. update - Update Component

Update existing component.

```bash
# Update from Git
airssys-wasm update comp_abc123 \
  --from-git https://github.com/user/my-plugin \
  --ref v1.1.0

# Update from local build
airssys-wasm update comp_abc123 \
  --from-file ./target/release/my_plugin.wasm \
  --manifest Component.toml

# With deployment strategy
airssys-wasm update comp_abc123 \
  --from-git <url> \
  --strategy canary

# Output:
# Updating component: comp_abc123
# 
# Current version: 1.0.0
# New version: 1.1.0
# 
# [1/5] Loading new component...     ✓
# [2/5] Verifying signature...       ✓
#   Author matches: ✓
# [3/5] Validating version...        ✓
#   1.1.0 > 1.0.0: ✓
# [4/5] Deploying update...          ✓
#   Strategy: Canary (10% traffic)
# [5/5] Monitoring health...         ✓
# 
# ✓ Component updated successfully!
# 
# Rollback available:
#   airssys-wasm rollback comp_abc123
```

### 7. uninstall - Uninstall Component

Uninstall component from host.

```bash
# Uninstall (requires confirmation)
airssys-wasm uninstall comp_abc123

# Force uninstall (no confirmation)
airssys-wasm uninstall comp_abc123 --force

# With specific keypair
airssys-wasm uninstall comp_abc123 --keypair ~/.airssys/my-key.json

# Output:
# Component: data-processor (comp_abc123)
# Version: 1.0.0
# Status: Running
# Instances: 2
# 
# ⚠️  Warning: This will permanently delete this component.
# 
# Type 'data-processor' to confirm: data-processor
# 
# [1/3] Generating uninstall proof... ✓
# [2/3] Verifying ownership...        ✓
#   Keypair matches author: ✓
# [3/3] Uninstalling component...     ✓
# 
# ✓ Component uninstalled successfully!
```

### 8. list - List Components

List installed components.

```bash
# List all components
airssys-wasm list

# List with specific host
airssys-wasm list --host https://prod.company.com

# JSON output (for scripting)
airssys-wasm list --output json

# Output (table format):
# ID          NAME             VERSION  STATUS   INSTANCES  MEMORY    CPU
# comp_abc123 data-processor   1.0.0    Running  2          256 MB    12%
# comp_def456 api-gateway      2.1.0    Running  3          128 MB    8%
# comp_ghi789 notifier         1.5.0    Running  1          64 MB     2%

# Output (JSON format):
# [
#   {
#     "id": "comp_abc123",
#     "name": "data-processor",
#     "version": "1.0.0",
#     "status": "running",
#     "instances": 2,
#     "memory_mb": 256,
#     "cpu_percent": 12
#   }
# ]
```

### 9. info - Component Information

Show detailed component information.

```bash
# Show component info
airssys-wasm info comp_abc123

# With capabilities
airssys-wasm info comp_abc123 --show-capabilities

# Output:
# Component: data-processor (comp_abc123)
# 
# Package:
#   Name: data-processor
#   Version: 1.0.0
#   Description: High-performance data processor
#   Author: developer@example.com
#   Repository: https://github.com/user/data-processor
#   License: MIT
# 
# Security:
#   Author Public Key: a1b2c3d4e5f6...
#   Signed At: 2025-10-18T10:30:00Z
#   WASM Hash: sha256:abc123def456...
# 
# Deployment:
#   Status: Running
#   Strategy: Blue-Green
#   Instances: 2
#   Health: Healthy
# 
# Resources:
#   Memory: 256 MB / 512 MB (50%)
#   CPU: 12% / 50% (24%)
#   Storage: 45 MB / 100 MB (45%)
# 
# Capabilities:
#   File System:
#     Read: /data/**, /config/**
#     Write: /output/**, /logs/**
#   Network:
#     Outbound: api.example.com:443
#     Inbound: 8080
#   Storage:
#     Quota: 100 MB
#     Persistent: true
```

### 10. logs - View Component Logs

Stream or view component logs.

```bash
# Follow logs (stream)
airssys-wasm logs comp_abc123 --follow

# Last 100 lines
airssys-wasm logs comp_abc123 --tail 100

# Filter by level
airssys-wasm logs comp_abc123 --level error

# Since timestamp
airssys-wasm logs comp_abc123 --since "2025-10-18T10:00:00Z"

# JSON output
airssys-wasm logs comp_abc123 --output json

# Output:
# [2025-10-18 10:30:15] INFO  Component initialized
# [2025-10-18 10:30:16] INFO  Processing data from /data/input.json
# [2025-10-18 10:30:17] DEBUG Read 1024 bytes
# [2025-10-18 10:30:18] INFO  Processed 500 records
# [2025-10-18 10:30:19] WARN  Rate limit approaching (80%)
# [2025-10-18 10:30:20] INFO  Output written to /output/result.json
```

### 11. status - Check Component Status

Check component health and status.

```bash
# Check single component
airssys-wasm status comp_abc123

# Check all components
airssys-wasm status --all

# Output:
# Component: data-processor (comp_abc123)
# 
# Status: Running ✓
# Health: Healthy ✓
# Uptime: 2 days 5 hours 23 minutes
# 
# Instances:
#   Instance 1: Running ✓ (Memory: 128 MB, CPU: 10%)
#   Instance 2: Running ✓ (Memory: 132 MB, CPU: 14%)
# 
# Recent Activity:
#   10:30:20 - Processed batch (500 records)
#   10:29:50 - Processed batch (500 records)
#   10:29:20 - Processed batch (500 records)
```

### 12. verify - Verify Signature

Verify component signature without installing.

```bash
# Verify local component
airssys-wasm verify \
  --wasm ./target/release/my_plugin.wasm \
  --manifest Component.toml

# Verify from Git
airssys-wasm verify --from-git https://github.com/user/my-plugin --ref v1.0.0

# Output:
# Verifying component signature...
# 
# Component: my-plugin v1.0.0
# WASM Hash: sha256:abc123def456...
# 
# Signature Verification:
#   ✓ WASM hash matches manifest
#   ✓ Component name matches
#   ✓ Version matches
#   ✓ Timestamp valid (2025-10-18T10:30:00Z)
#   ✓ Ed25519 signature cryptographically valid
# 
# Author:
#   Public Key: a1b2c3d4e5f6...
# 
# ✓ Signature verification successful!
#   This component was signed by the claimed author.
```

### 13. config - Configuration Management

Manage CLI configuration.

```bash
# Show all configuration
airssys-wasm config list

# Get specific value
airssys-wasm config get host

# Set value
airssys-wasm config set host https://runtime.company.com
airssys-wasm config set keypair ~/.airssys/production-key.json

# Unset value (use default)
airssys-wasm config unset host

# Show config file location
airssys-wasm config path

# Output (list):
# Configuration (~/.airssys/config.toml):
# 
# host = "http://localhost:8080"
# keypair = "/Users/dev/.airssys/keypair.json"
# log_level = "info"
# output_format = "text"
```

### 14. completions - Shell Completions

Generate shell completion scripts.

```bash
# Generate for bash
airssys-wasm completions bash > ~/.local/share/bash-completion/completions/airssys-wasm

# Generate for zsh
airssys-wasm completions zsh > ~/.zsh/completions/_airssys-wasm

# Generate for fish
airssys-wasm completions fish > ~/.config/fish/completions/airssys-wasm.fish

# Generate for PowerShell
airssys-wasm completions powershell > ~/Documents/PowerShell/Scripts/airssys-wasm.ps1
```

---

## Configuration Management

### Configuration File Format

**Location:** `~/.airssys/config.toml`

```toml
# Default host runtime URL
host = "http://localhost:8080"

# Default keypair location
keypair = "/Users/dev/.airssys/keypair.json"

# Logging configuration
log_level = "info"  # error, warn, info, debug, trace
log_format = "text" # text, json

# Output configuration
output_format = "text"  # text, json, yaml
colorize = true
interactive = true

# Build configuration
[build]
default_profile = "release"
optimize = true
cache_builds = true

# Git configuration
[git]
default_ref = "main"
ssh_key = "~/.ssh/id_ed25519"

# Network configuration
[network]
timeout_seconds = 30
retry_attempts = 3
```

### Configuration Precedence

```
1. Command-line flags (highest priority)
2. Environment variables
3. Configuration file
4. Built-in defaults (lowest priority)
```

**Example:**
```bash
# All of these set the host:
airssys-wasm install --host https://prod.company.com <url>           # 1. CLI flag
export AIRSSYS_WASM_HOST=https://staging.company.com                # 2. Env var
echo 'host = "http://localhost:8080"' > ~/.airssys/config.toml      # 3. Config file
# (built-in default: http://localhost:8080)                          # 4. Default

# CLI flag wins
```

---

## Output Formatting

### Text Format (Default)

Human-readable, colorized output:

```bash
airssys-wasm list

# Output:
# ID          NAME             VERSION  STATUS   INSTANCES  MEMORY    CPU
# comp_abc123 data-processor   1.0.0    Running  2          256 MB    12%
# comp_def456 api-gateway      2.1.0    Running  3          128 MB    8%
# comp_ghi789 notifier         1.5.0    Running  1          64 MB     2%
```

### JSON Format (Scripting)

Machine-readable JSON:

```bash
airssys-wasm list --output json

# Output:
# [
#   {
#     "id": "comp_abc123",
#     "name": "data-processor",
#     "version": "1.0.0",
#     "status": "running",
#     "instances": 2,
#     "memory_mb": 256,
#     "cpu_percent": 12
#   }
# ]
```

### YAML Format

Human-readable structured output:

```bash
airssys-wasm info comp_abc123 --output yaml

# Output:
# id: comp_abc123
# name: data-processor
# version: 1.0.0
# package:
#   authors:
#     - developer@example.com
#   license: MIT
# security:
#   author_public_key: a1b2c3d4...
#   signed_at: 2025-10-18T10:30:00Z
```

### Progress Indicators

```bash
# Spinner for single operations
[1/6] Cloning repository...        ⠋

# Progress bar for downloads
Downloading WASM binary...
[████████████░░░░░░░░] 65% (1.2 MB / 1.8 MB)

# Step-by-step with checkmarks
[1/6] Cloning repository...        ✓ (2.3s)
[2/6] Checking out v1.0.0...       ✓ (0.1s)
[3/6] Building component...        ✓ (18.5s)
```

---

## Error Handling

### Error Message Design

**Principles:**
1. Clear problem description
2. Root cause explanation
3. Actionable solution
4. Reference documentation link

**Example 1: Missing Toolchain**
```bash
airssys-wasm build

# Error: Build failed

# Caused by:
#   Rust toolchain not found
# 
# The 'cargo' command is not available. Rust must be installed to build
# components written in Rust.
# 
# Solution:
#   Install Rust toolchain:
#     curl --proto '=https' --tlsf v1.2 -sSf https://sh.rustup.rs | sh
# 
#   Then install the WASM target:
#     rustup target add wasm32-unknown-unknown
# 
# Documentation: https://docs.airssys.dev/installation#rust-setup
```

**Example 2: Signature Verification Failure**
```bash
airssys-wasm install --from-file my_plugin.wasm --manifest Component.toml

# Error: Signature verification failed
# 
# Caused by:
#   WASM hash mismatch
#   Expected: sha256:abc123def456...
#   Actual:   sha256:xyz789abc123...
# 
# This means the WASM binary has been modified since it was signed.
# 
# Possible causes:
#   1. Binary was rebuilt but not re-signed
#   2. Binary was tampered with
#   3. Wrong binary file specified
# 
# Solution:
#   If you just rebuilt the component, sign it again:
#     airssys-wasm sign --manifest Component.toml --wasm my_plugin.wasm
# 
#   If you suspect tampering, rebuild from source:
#     airssys-wasm build
#     airssys-wasm sign
```

**Example 3: Authorization Failure**
```bash
airssys-wasm update comp_abc123 --from-git <url>

# Error: Update authorization failed
# 
# Caused by:
#   Author mismatch
#   Existing component author: a1b2c3d4e5f6...
#   New component author:      x9y8z7w6v5u4...
# 
# Only the original component author can update this component.
# 
# Your current keypair:
#   Public Key: x9y8z7w6v5u4...
#   Location: ~/.airssys/keypair.json
# 
# Component author's public key:
#   Public Key: a1b2c3d4e5f6...
# 
# Solution:
#   Use the correct keypair:
#     airssys-wasm update comp_abc123 --keypair ~/.airssys/original-key.json
# 
#   Or contact the component author for access.
```

### Exit Codes

```rust
pub enum ExitCode {
    Success = 0,
    GeneralError = 1,
    CommandLineError = 2,
    ConfigurationError = 3,
    NetworkError = 4,
    AuthenticationError = 5,
    AuthorizationError = 6,
    ValidationError = 7,
    BuildError = 8,
    SignatureError = 9,
}
```

**Usage in scripts:**
```bash
#!/bin/bash
airssys-wasm install --from-git <url>
if [ $? -eq 0 ]; then
    echo "Installation successful"
else
    echo "Installation failed with code $?"
    exit 1
fi
```

---

## Implementation Architecture

### CLI Framework: clap

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "airssys-wasm")]
#[command(version, about = "WASM Component Framework CLI", long_about = None)]
struct Cli {
    /// Global flags
    #[arg(long, global = true, help = "Host runtime URL")]
    host: Option<String>,
    
    #[arg(long, global = true, help = "Keypair file path")]
    keypair: Option<PathBuf>,
    
    #[arg(long, global = true, value_enum, help = "Log level")]
    log_level: Option<LogLevel>,
    
    #[arg(long, global = true, value_enum, help = "Output format")]
    output: Option<OutputFormat>,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate new Ed25519 keypair
    Keygen {
        #[arg(short, long, default_value = "~/.airssys/keypair.json")]
        output: PathBuf,
    },
    
    /// Initialize new component project
    Init {
        /// Project name
        name: Option<String>,
        
        #[arg(long)]
        lang: Option<Language>,
    },
    
    /// Build component
    Build {
        #[arg(long)]
        release: bool,
        
        #[arg(long)]
        optimize: bool,
    },
    
    /// Sign component
    Sign {
        #[arg(long)]
        manifest: Option<PathBuf>,
        
        #[arg(long)]
        wasm: Option<PathBuf>,
    },
    
    /// Install component
    Install {
        #[command(flatten)]
        source: InstallSourceArgs,
        
        #[arg(long)]
        capabilities: Option<PathBuf>,
        
        #[arg(long)]
        dry_run: bool,
    },
    
    /// Update component
    Update {
        /// Component ID
        component_id: String,
        
        #[command(flatten)]
        source: InstallSourceArgs,
    },
    
    /// Uninstall component
    Uninstall {
        /// Component ID
        component_id: String,
        
        #[arg(long)]
        force: bool,
    },
    
    /// List components
    List,
    
    /// Show component info
    Info {
        /// Component ID
        component_id: String,
        
        #[arg(long)]
        show_capabilities: bool,
    },
    
    /// View logs
    Logs {
        /// Component ID
        component_id: String,
        
        #[arg(short, long)]
        follow: bool,
        
        #[arg(long)]
        tail: Option<usize>,
    },
    
    /// Check status
    Status {
        /// Component ID (omit for all)
        component_id: Option<String>,
        
        #[arg(long)]
        all: bool,
    },
    
    /// Configuration management
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(clap::Args)]
#[group(required = true, multiple = false)]
struct InstallSourceArgs {
    #[arg(long, help = "Install from Git repository")]
    from_git: Option<String>,
    
    #[arg(long, requires = "manifest", help = "Install from local file")]
    from_file: Option<PathBuf>,
    
    #[arg(long, requires = "manifest_url", help = "Install from URL")]
    from_url: Option<String>,
    
    #[arg(long, help = "Git reference (tag/branch/commit)")]
    ref_: Option<String>,
    
    #[arg(long, help = "Manifest file (for local/URL)")]
    manifest: Option<PathBuf>,
    
    #[arg(long, help = "Manifest URL (for URL source)")]
    manifest_url: Option<String>,
}
```

### Main Entry Point

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI
    let cli = Cli::parse();
    
    // Initialize logging
    init_logging(cli.log_level.unwrap_or(LogLevel::Info))?;
    
    // Load configuration
    let config = Config::load()?;
    
    // Execute command
    match cli.command {
        Commands::Keygen { output } => {
            KeygenCommand { output }.execute()?;
        }
        Commands::Install { source, capabilities, dry_run } => {
            let source = parse_install_source(source)?;
            let cmd = InstallCommand {
                source,
                host: cli.host.or(config.host),
                capabilities,
                dry_run,
            };
            cmd.execute().await?;
        }
        // ... other commands
    }
    
    Ok(())
}
```

---

## Examples and Use Cases

### Use Case 1: First-Time Component Developer

```bash
# 1. Initial setup
airssys-wasm init
# Generate keypair, save to ~/.airssys/keypair.json

# 2. Create project
airssys-wasm init my-first-plugin --lang rust

# 3. Develop component
cd my-first-plugin
# ... write code in src/lib.rs ...

# 4. Build
airssys-wasm build

# 5. Sign
airssys-wasm sign

# 6. Install to local host
airssys-wasm install \
  --from-file target/wasm32-unknown-unknown/release/my_first_plugin.wasm \
  --manifest Component.toml

# 7. Check status
airssys-wasm status comp_abc123

# 8. View logs
airssys-wasm logs comp_abc123 --follow
```

### Use Case 2: Production Deployment

```bash
# 1. Install from tagged release
airssys-wasm install \
  --from-git https://github.com/company/production-plugin.git \
  --ref v2.1.0 \
  --host https://prod.company.com \
  --capabilities production-caps.toml

# 2. Monitor deployment
airssys-wasm status --all --host https://prod.company.com

# 3. View logs
airssys-wasm logs comp_abc123 --host https://prod.company.com --follow

# 4. Update to new version (canary deployment)
airssys-wasm update comp_abc123 \
  --from-git https://github.com/company/production-plugin.git \
  --ref v2.2.0 \
  --host https://prod.company.com

# 5. Rollback if issues detected
airssys-wasm rollback comp_abc123 --host https://prod.company.com
```

### Use Case 3: CI/CD Integration

```yaml
# .github/workflows/deploy.yml
name: Deploy Component

on:
  push:
    tags:
      - 'v*'

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install airssys-wasm-cli
        run: cargo install airssys-wasm-cli
      
      - name: Install from Git (production)
        env:
          AIRSSYS_WASM_HOST: ${{ secrets.PROD_HOST }}
          AIRSSYS_WASM_KEYPAIR_JSON: ${{ secrets.DEPLOYMENT_KEYPAIR }}
        run: |
          echo "$AIRSSYS_WASM_KEYPAIR_JSON" > /tmp/keypair.json
          airssys-wasm install \
            --from-git https://github.com/${{ github.repository }} \
            --ref ${{ github.ref_name }} \
            --keypair /tmp/keypair.json \
            --output json > deployment-result.json
      
      - name: Save component ID
        run: |
          COMPONENT_ID=$(jq -r '.component_id' deployment-result.json)
          echo "COMPONENT_ID=$COMPONENT_ID" >> $GITHUB_ENV
      
      - name: Monitor deployment
        run: |
          airssys-wasm status ${{ env.COMPONENT_ID }} --output json
```

---

## Future Enhancements

### Phase 2: Enhanced Features

**1. Interactive Mode**
```bash
airssys-wasm interactive
# Opens interactive shell with autocomplete
> install --from-git https://github.com/user/plugin
> status comp_abc123
> logs comp_abc123 --follow
> exit
```

**2. Component Templates**
```bash
airssys-wasm init my-plugin --template data-processor
# Templates: data-processor, api-gateway, event-handler, ml-inference
```

**3. Dependency Management**
```bash
# Install component with dependencies
airssys-wasm install --file componentfile.yml
# Automatically resolves and installs dependencies
```

**4. Development Mode**
```bash
# Hot reload during development
airssys-wasm dev
# Watches for changes, auto-rebuilds, auto-deploys
```

### Phase 3: Advanced Features

**1. Component Marketplace Integration**
```bash
# Search public components
airssys-wasm search "data processor"

# Install from marketplace
airssys-wasm install --from-registry airssys/official-data-processor
```

**2. Monitoring Dashboard**
```bash
# Launch web-based dashboard
airssys-wasm dashboard
# Opens http://localhost:9090 with component monitoring UI
```

**3. Rollback Automation**
```bash
# Automatic rollback on health check failure
airssys-wasm update comp_abc123 \
  --from-git <url> \
  --auto-rollback \
  --health-threshold 95%
```

---

## References

### CLI Design Resources
- [Command Line Interface Guidelines](https://clig.dev/)
- [The Rustup Book](https://rust-lang.github.io/rustup/)
- [Cargo Book](https://doc.rust-lang.org/cargo/)

### Libraries
- **clap**: Command-line argument parsing
- **tokio**: Async runtime
- **serde**: Serialization/deserialization
- **indicatif**: Progress bars and spinners
- **colored**: Terminal colors
- **dialoguer**: Interactive prompts

---

**Document Status:** Complete - Ready for implementation

**Next Steps:**
1. Implement CLI using clap and tokio
2. Create comprehensive test suite
3. Write user documentation and tutorials
4. Package for distribution (Cargo, GitHub Releases, package managers)
