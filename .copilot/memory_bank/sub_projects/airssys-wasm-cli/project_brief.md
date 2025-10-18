# Project Brief: airssys-wasm-cli

**Sub-Project:** airssys-wasm-cli  
**Status:** Foundation Setup Phase (10% complete)  
**Created:** 2025-10-18  
**Last Updated:** 2025-10-18

---

## Project Identity

### What is airssys-wasm-cli?

`airssys-wasm-cli` is the command-line interface tool for managing WASM components in the AirsSys ecosystem. It provides developers and operators with a comprehensive set of commands for the complete component lifecycle: from initialization and building to signing, installation, and runtime management.

**Project Type:** Binary crate (CLI tool)  
**Workspace Member:** `airssys/airssys-wasm-cli/`  
**Binary Name:** `airssys-wasm`  
**Distribution:** crates.io, cargo install, pre-built binaries

### Problem Statement

WASM component development and deployment requires multiple complex operations:
- Cryptographic key management for component signing
- Building components from source (multiple languages)
- Signing components with Ed25519 signatures
- Installing components from various sources (Git, file, URL)
- Managing component lifecycle (update, uninstall)
- Monitoring component health and logs

Without a unified CLI tool, developers would need to:
- Manually manage cryptographic keys
- Invoke build tools directly for different languages
- Implement custom signing scripts
- Handle Git operations and HTTP downloads separately
- Write custom scripts for component management

### Solution

A single, unified CLI that handles all component lifecycle operations with:
- Modern UX (colored output, progress bars, interactive prompts)
- Cryptographic security (Ed25519 signing/verification)
- Multi-source installation (Git-agnostic, file, URL)
- Configuration management (~/.airssys/config.toml)
- Shell completions (bash, zsh, fish, powershell, elvish)

---

## Project Scope

### In Scope

**Component Development:**
- `keygen` - Generate Ed25519 keypairs
- `init` - Initialize component projects
- `build` - Build WASM components from source
- `sign` - Sign components with private key

**Component Distribution:**
- `install` - Install from Git/file/URL
- `update` - Update to newer versions
- `uninstall` - Remove components (with authorization)

**Component Management:**
- `list` - List installed components
- `info` - Show component details
- `status` - Check component health
- `logs` - View/stream component logs

**Tooling:**
- `verify` - Verify signatures
- `config` - Manage CLI configuration
- `completions` - Generate shell completions

### Out of Scope

- **Host Runtime Server** - Separate binary for running components
- **Visual Composition** - GUI tools (future)
- **Registry Service** - Centralized component registry (future)
- **Language Toolchains** - Uses existing cargo, go, etc.

### Dependencies

**Consumes:**
- `airssys-wasm` - Core WASM library for component operations
- External language toolchains (cargo, go build, etc.)

**Provides:**
- Developer-facing CLI for component lifecycle
- Component signing and verification utilities
- Installation and management workflows

---

## Success Criteria

### Phase 1: Foundation (✅ COMPLETE - 10%)
- [x] Project structure and workspace integration
- [x] All 14 command stubs implemented
- [x] Error handling infrastructure
- [x] Configuration management structure
- [x] UX utilities (progress, colors, prompts)
- [x] Zero compilation warnings
- [x] Memory bank documentation

### Phase 2: Core Commands (Planned - Q1 2026)
- [ ] Key generation (Ed25519)
- [ ] Component initialization with templates
- [ ] Build integration (Rust, Go, etc.)
- [ ] Cryptographic signing
- [ ] Multi-source installation

### Phase 3: Management Features (Planned - Q2 2026)
- [ ] Component updates and versioning
- [ ] Authorized uninstallation
- [ ] Component listing and info
- [ ] Health monitoring
- [ ] Log viewing/streaming

### Phase 4: Polish & Distribution (Planned - Q3 2026)
- [ ] Comprehensive error messages
- [ ] Shell completion testing
- [ ] Pre-built binaries (GitHub Releases)
- [ ] Homebrew formula
- [ ] Comprehensive user documentation

---

## Strategic Context

### Relationship to AirsSys Ecosystem

```
airssys-wasm (Core Library)
    ↑
    │ provides component APIs
    │
airssys-wasm-cli (This Project)
    ↓
    │ used by
    │
Component Developers & DevOps
```

### Integration Points

**airssys-wasm Integration:**
- Uses core library for component operations
- Delegates to WASM runtime for verification
- Accesses component registry storage

**Developer Workflow Integration:**
- `airssys-wasm init` → creates project
- `cargo build` / `go build` → builds WASM
- `airssys-wasm sign` → signs component
- `airssys-wasm install` → deploys component

**CI/CD Integration:**
- Scriptable for automated builds
- JSON output for programmatic usage
- Exit codes for pipeline integration

---

## Target Audience

### Primary Users

**Component Developers:**
- Initialize and build WASM components
- Sign components with their keys
- Test local installations

**DevOps Engineers:**
- Install components in production
- Manage component updates
- Monitor component health

**System Administrators:**
- Configure CLI settings
- Manage component lifecycle
- Review component logs

### User Experience Principles

1. **Familiar Patterns** - Follows cargo, git, kubectl conventions
2. **Helpful Errors** - Actionable error messages with suggestions
3. **Progressive Disclosure** - Simple defaults, advanced options available
4. **Fast Feedback** - Progress indicators for long operations
5. **Safe Operations** - Confirmations for destructive actions

---

## Technical Approach

### Architecture Overview

- **Binary Crate** - Standalone workspace member
- **CLI Framework** - clap with derive macros
- **Async Runtime** - Tokio for concurrent operations
- **UX Layer** - colored, indicatif, dialoguer
- **Crypto** - ed25519-dalek for signatures
- **Git** - git2 for repository operations
- **HTTP** - reqwest for URL downloads

### Design Patterns

**Command Pattern:**
- Each command in separate module
- Consistent signature: `async fn execute(args) -> Result<()>`
- Shared utilities for common operations

**Error Handling:**
- Structured error types (CliError enum)
- Context-rich error messages
- Conversion from external error types

**Configuration:**
- TOML-based config (~/.airssys/config.toml)
- Environment variable overrides
- Sensible defaults

---

## Project Metadata

**Repository:** https://github.com/airsstack/airssys  
**Sub-Project Path:** `airssys-wasm-cli/`  
**License:** MIT OR Apache-2.0  
**Rust Edition:** 2021  
**MSRV:** 1.88 (workspace rust-version)

**Related Sub-Projects:**
- airssys-wasm (core library dependency)
- airssys-wasm-component (proc macros for component development)

**Related Knowledge Documentation:**
- KNOWLEDGE-WASM-009: Component Installation Architecture
- KNOWLEDGE-WASM-010: CLI Tool Specification

**Related ADRs:**
- None yet (CLI design decisions inherited from KNOWLEDGE-WASM-010)

---

**Next Steps:**
1. Implement keygen command (Ed25519 key generation)
2. Implement init command (project scaffolding)
3. Implement build command (language-agnostic WASM building)
4. Implement sign command (component signing)
5. Implement install command (multi-source installation)
