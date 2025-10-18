# airssys-wasm-cli

Command-line tool for managing WASM components in the AirsSys ecosystem.

## Overview

`airssys-wasm-cli` provides a comprehensive CLI for component lifecycle management, including building, signing, installing, and managing WASM components with cryptographic security.

## Features

- 🔑 **Cryptographic Security**: Ed25519 digital signatures for component ownership
- 📦 **Multi-Source Installation**: Install from Git repositories, local files, or remote URLs
- 🛠️ **Build Automation**: Automated WASM component building from source
- 🔍 **Component Management**: List, inspect, update, and uninstall components
- 📝 **TOML Manifests**: Human-readable component configuration
- 🎨 **Modern UX**: Colored output, progress indicators, and shell completions

## Installation

### From Source (Development)

```bash
# Clone the airssys monorepo
git clone https://github.com/airsstack/airssys
cd airssys

# Build and install the CLI
cargo install --path airssys-wasm-cli
```

### From crates.io (Future)

```bash
cargo install airssys-wasm-cli
```

### Pre-built Binaries (Future)

Download pre-built binaries from the [GitHub Releases](https://github.com/airsstack/airssys/releases) page.

## Quick Start

### 1. Generate a Keypair

```bash
airssys-wasm keygen
```

This creates an Ed25519 keypair in `~/.airssys/keypair.json` for signing your components.

### 2. Initialize a New Component

```bash
airssys-wasm init my-plugin --description "My awesome plugin"
cd my-plugin
```

### 3. Build Your Component

```bash
airssys-wasm build --release
```

### 4. Sign Your Component

```bash
airssys-wasm sign target/wasm32-wasi/release/my-plugin.wasm
```

### 5. Install a Component

```bash
# From Git repository
airssys-wasm install git@github.com:example/my-component.git

# From local file
airssys-wasm install ./my-component.wasm

# From remote URL
airssys-wasm install https://example.com/components/my-component.wasm
```

## Commands

### Component Development

- `keygen` - Generate Ed25519 keypair for signing
- `init` - Initialize a new component project
- `build` - Build WASM component from source
- `sign` - Sign component with your private key

### Component Management

- `install` - Install component from Git/file/URL
- `update` - Update installed component
- `uninstall` - Remove installed component
- `list` - List all installed components
- `info` - Show detailed component information

### Operations

- `logs` - View or stream component logs
- `status` - Check component health and status
- `verify` - Verify component signature

### Configuration

- `config` - Manage CLI configuration
- `completions` - Generate shell completions

## Command Reference

### `airssys-wasm keygen`

Generate a new Ed25519 keypair for component signing.

```bash
airssys-wasm keygen [OPTIONS]

Options:
  -o, --output <PATH>    Output path for keypair [default: ~/.airssys/keypair.json]
  -f, --force            Overwrite existing keypair
```

### `airssys-wasm init`

Initialize a new component project with directory structure and manifest.

```bash
airssys-wasm init <NAME> [OPTIONS]

Arguments:
  <NAME>    Component name

Options:
  -d, --description <TEXT>    Component description
  -a, --author <NAME>         Component author
  -e, --example               Initialize with example code
```

### `airssys-wasm build`

Build a WASM component from source code.

```bash
airssys-wasm build [OPTIONS]

Options:
  -p, --path <PATH>       Component directory [default: .]
  -r, --release           Build in release mode
  -o, --output <PATH>     Output path for WASM binary
```

### `airssys-wasm sign`

Sign a WASM component with your Ed25519 private key.

```bash
airssys-wasm sign <COMPONENT> [OPTIONS]

Arguments:
  <COMPONENT>    Path to WASM component file

Options:
  -k, --keypair <PATH>    Keypair file [default: ~/.airssys/keypair.json]
  -o, --output <PATH>     Output path for signed component
```

### `airssys-wasm install`

Install a component from various sources.

```bash
airssys-wasm install <SOURCE> [OPTIONS]

Arguments:
  <SOURCE>    Git URL, local path, or remote URL

Options:
  -b, --branch <NAME>     Git branch or tag
  -c, --commit <HASH>     Git commit hash
      --skip-verify       Skip signature verification
  -f, --force             Force installation
```

### `airssys-wasm update`

Update an installed component to a newer version.

```bash
airssys-wasm update <NAME> [OPTIONS]

Arguments:
  <NAME>    Component name

Options:
  -v, --version <VER>     Update to specific version
      --skip-verify       Skip signature verification
```

### `airssys-wasm uninstall`

Uninstall a component (requires signature authorization).

```bash
airssys-wasm uninstall <NAME> [OPTIONS]

Arguments:
  <NAME>    Component name

Options:
  -k, --keypair <PATH>    Keypair for authorization
  -f, --force             Force without confirmation
```

### `airssys-wasm list`

List all installed components.

```bash
airssys-wasm list [OPTIONS]

Options:
  -d, --detailed          Show detailed information
  -s, --status <STATUS>   Filter by status (running, stopped, all)
```

### `airssys-wasm info`

Show detailed information about a component.

```bash
airssys-wasm info <NAME>

Arguments:
  <NAME>    Component name
```

### `airssys-wasm verify`

Verify component signature and integrity.

```bash
airssys-wasm verify <COMPONENT> [OPTIONS]

Arguments:
  <COMPONENT>    Path to component file

Options:
  -p, --public-key <KEY>    Expected public key (base64)
```

### `airssys-wasm config`

Manage CLI configuration.

```bash
airssys-wasm config <SUBCOMMAND>

Subcommands:
  show      Show current configuration
  set       Set configuration value
  get       Get configuration value
  reset     Reset to defaults
```

### `airssys-wasm completions`

Generate shell completions for the CLI.

```bash
airssys-wasm completions <SHELL>

Arguments:
  <SHELL>    Shell type [bash, zsh, fish, powershell, elvish]

Examples:
  airssys-wasm completions bash > /etc/bash_completion.d/airssys-wasm
  airssys-wasm completions zsh > ~/.zsh/completions/_airssys-wasm
```

## Configuration

Configuration file location: `~/.airssys/config.toml`

```toml
# Default storage backend (sled or rocksdb)
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

## Component.toml Manifest

Example component manifest:

```toml
[component]
name = "my-plugin"
version = "0.1.0"
description = "My awesome WASM plugin"
authors = ["Your Name <you@example.com>"]

[component.metadata]
license = "MIT"
repository = "https://github.com/yourname/my-plugin"
homepage = "https://example.com"
tags = ["plugin", "example"]

[build]
language = "rust"
target = "wasm32-wasi"

[permissions]
filesystem = { read = ["/data"], write = ["/tmp"] }
network = { allowed_hosts = ["api.example.com"] }
environment = { allowed_vars = ["CONFIG_PATH"] }

[signature]
public_key = "base64-encoded-public-key"
algorithm = "ed25519"
```

## Development Status

⚠️ **Current Status**: Foundation setup phase - stub implementations

This CLI is part of the `airssys-wasm` framework, currently in architecture design phase (15% complete). Command implementations are stubs and will be completed during the implementation phase.

## Architecture

The CLI is organized into the following modules:

- `commands/` - Command implementations (14 commands)
- `config.rs` - Configuration management
- `error.rs` - Error types and handling
- `utils.rs` - Shared utilities (progress bars, formatting, user interaction)

## Contributing

This project is part of the [AirsSys](https://github.com/airsstack/airssys) monorepo. Please see the main repository for contribution guidelines.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE))
- MIT License ([LICENSE-MIT](../LICENSE-MIT))

at your option.

## Related Projects

- **[airssys-wasm](../airssys-wasm/)** - Core WASM framework library
- **[airssys-wasm-component](../airssys-wasm-component/)** - Procedural macros for Rust component development
- **[airssys-osl](../airssys-osl/)** - OS Layer abstraction framework
- **[airssys-rt](../airssys-rt/)** - Lightweight actor runtime

## Documentation

- [Installation Architecture](../.copilot/memory_bank/sub_projects/airssys-wasm/docs/knowledges/knowledge_wasm_009_component_installation_architecture.md)
- [CLI Tool Specification](../.copilot/memory_bank/sub_projects/airssys-wasm/docs/knowledges/knowledge_wasm_010_cli_tool_specification.md)
- [Component Framework Architecture](../.copilot/memory_bank/sub_projects/airssys-wasm/docs/knowledges/knowledge_wasm_001_component_framework_architecture.md)

---

**AirsSys** - System programming components for the AirsStack ecosystem
