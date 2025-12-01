# [WASM-TASK-012] - Block 11: CLI Tool

**Status:** not-started  
**Added:** 2025-10-20  
**Updated:** 2025-11-30  
**Priority:** High - Developer Experience Layer  
**Layer:** 4 - Developer Experience  
**Block:** 11 of 11  
**Estimated Effort:** 4-5 weeks  

## ⚠️ CRITICAL: Sub-Project Context

**This task implements the `airssys-wasm-cli` crate** - a separate binary crate in the AirsSys workspace.

### Workspace Structure Reference
```
airssys/
├── airssys-wasm/              # Core framework library (Blocks 1-9)
├── airssys-wasm-component/    # Procedural macros (Block 10)
└── airssys-wasm-cli/          # 🎯 THIS TASK (Block 11)
    ├── Cargo.toml             # [[bin]] name = "airssys-wasm"
    ├── src/
    │   ├── main.rs            # CLI entry point
    │   ├── commands/          # 14 command modules
    │   │   ├── keygen.rs
    │   │   ├── init.rs
    │   │   ├── build.rs
    │   │   ├── sign.rs
    │   │   ├── install.rs
    │   │   ├── update.rs
    │   │   ├── uninstall.rs
    │   │   ├── list.rs
    │   │   ├── info.rs
    │   │   ├── status.rs
    │   │   ├── logs.rs
    │   │   ├── verify.rs
    │   │   ├── config.rs
    │   │   └── completions.rs
    │   ├── cli_config.rs      # Configuration
    │   ├── error.rs           # Error handling
    │   └── utils.rs           # Shared utilities
    └── tests/                 # CLI integration tests
```

### Key Distinctions
- ✅ **This task**: Implements `airssys-wasm-cli/` (CLI binary tool)
- ❌ **NOT this task**: Core library (`airssys-wasm/`), Macros (`airssys-wasm-component/`)
- 📚 **Complete reference**: See **KNOWLEDGE-WASM-015** for full workspace architecture

### Dependencies
This CLI tool depends on:
- `airssys-wasm` (core library) - For component runtime and lifecycle management
- Layer 2 features (Blocks 4-7) - For full command functionality:
  - Block 4 (Security) - For signature verification
  - Block 7 (Lifecycle) - For install/update/uninstall
  - Block 6 (Storage) - For component management
  - Block 5 (Messaging) - For status/logs commands

### Current Foundation Status
- ✅ Project structure complete (10% overall)
- ✅ All 14 command modules created with stubs
- ✅ Command structure defined
- ✅ Compiles successfully
- ⏳ **Awaiting Layer 2**: Full implementation requires Blocks 4-7 complete

## Overview

Implement comprehensive command-line interface tool providing 14 commands for complete component lifecycle management: keygen (Ed25519 key generation), init (project templates), build (compilation and packaging), sign (cryptographic signing), install/update/uninstall (lifecycle operations), list/info/status (discovery and inspection), logs (audit trail viewing), verify (signature validation), config (tool configuration), and completions (shell integration) supporting multi-source installation (Git/Local/URL) achieving intuitive developer experience.

## Context

**Current State:**
- **airssys-wasm-cli crate**: Foundation complete (10%)
- **Location**: `airssys/airssys-wasm-cli/` directory
- **Architecture**: KNOWLEDGE-WASM-015 (Workspace Architecture) - **ESSENTIAL REFERENCE**
- **CLI Specification**: KNOWLEDGE-WASM-010 (CLI Tool Specification) - **COMPLETE**
- **Stub implementations**: All 14 commands have stubs
- **Dependencies**: Lifecycle system (Block 7), SDK (Block 10), Security (Block 4) ready
- **Component.toml spec**: KNOWLEDGE-WASM-010 complete

**Problem Statement:**
Component development and operations need unified CLI:
1. **Lifecycle Management** - Install, update, uninstall components
2. **Development Workflow** - Init, build, sign components
3. **Security Operations** - Key generation, signature verification
4. **Debugging Support** - Logs, status, component info
5. **Configuration Management** - Tool and component configuration
6. **Shell Integration** - Tab completion, helpful output

Requirements:
- 14 comprehensive commands covering full workflow
- Multi-source installation (Git, Local, URL)
- Ed25519 cryptographic operations
- Intuitive command structure and help
- Rich output formatting (tables, colors)
- Shell completions (bash, zsh, fish, powershell)
- Configuration file support (~/.airswasm/config.toml)

**Why This Block Matters:**
Without CLI tool:
- No way to install components
- No component lifecycle management
- Manual build and signing processes
- Poor developer experience
- Limited operational visibility

This block completes the developer experience layer.

## Objectives

### Primary Objective
Implement comprehensive CLI tool with 14 commands (keygen, init, build, sign, install, update, uninstall, list, info, status, logs, verify, config, completions) supporting multi-source installation, Ed25519 signing, intuitive UX, rich output formatting, and shell integration achieving complete component lifecycle management.

### Secondary Objectives
- Command execution <1s for local operations
- Rich terminal UI (colors, tables, progress bars)
- Comprehensive help system (--help for all commands)
- Error messages actionable (suggest fixes)
- Shell completions for major shells (bash, zsh, fish, powershell)
- Config file support for preferences

## Scope

### In Scope
1. **Keygen Command** - Ed25519 key pair generation
2. **Init Command** - Project initialization from templates
3. **Build Command** - Component compilation and packaging
4. **Sign Command** - Ed25519 component signing
5. **Install Command** - Multi-source component installation
6. **Update Command** - Component version updates
7. **Uninstall Command** - Component removal
8. **List Command** - Installed component listing
9. **Info Command** - Component metadata display
10. **Status Command** - Component runtime status
11. **Logs Command** - Audit log viewing and search
12. **Verify Command** - Signature verification
13. **Config Command** - Tool configuration management
14. **Completions Command** - Shell completion generation

### Out of Scope
- GUI/TUI interface (Phase 1 is CLI only)
- Remote component management (Phase 2)
- Component marketplace search (Phase 2)
- Interactive installation wizard (Phase 2)
- Component dependency graph visualization (Phase 2)

## Implementation Plan

### Phase 1: CLI Foundation and Configuration (Week 1)

#### Task 1.1: CLI Framework Setup
**Deliverables:**
- clap v4 CLI framework integration
- Command structure definition (14 commands)
- Global flags (--verbose, --quiet, --color)
- Error handling and reporting
- CLI framework documentation

**Success Criteria:**
- All commands registered
- Help system comprehensive
- Global flags work
- Errors formatted nicely
- Framework well-structured

#### Task 1.2: Configuration System
**Deliverables:**
- Config file format (~/.airswasm/config.toml)
- Config loading and parsing
- Config validation
- Default configuration values
- Configuration documentation

**Success Criteria:**
- Config file loaded correctly
- Format intuitive
- Validation catches errors
- Defaults sensible
- Documentation clear

#### Task 1.3: Rich Terminal Output
**Deliverables:**
- Colored output (errors red, success green)
- Table formatting for lists
- Progress bars for long operations
- Spinner for operations in progress
- Output formatting documentation

**Success Criteria:**
- Colors work in all terminals
- Tables formatted well
- Progress visible
- Spinners indicate activity
- Output professional

---

### Phase 2: Security Commands (keygen, sign, verify) (Week 1-2)

#### Task 2.1: Keygen Command Implementation
**Deliverables:**
- Ed25519 key pair generation
- Key file output (PEM format)
- Public/private key separation
- Key generation options (output path)
- Keygen command documentation

**Success Criteria:**
- Keys generated securely
- PEM format standard
- Files written safely (permissions)
- Options flexible
- Documentation comprehensive

#### Task 2.2: Sign Command Implementation
**Deliverables:**
- Component signing with private key
- Signature file generation (.sig)
- Signature format (Ed25519 + metadata)
- Signing options (key path, output)
- Sign command documentation

**Success Criteria:**
- Signing works correctly
- Signature file valid
- Metadata included (timestamp, signer)
- Options flexible
- Clear documentation

#### Task 2.3: Verify Command Implementation
**Deliverables:**
- Signature verification with public key
- Trust store integration
- Verification output (success/failure)
- Detailed failure reasons
- Verify command documentation

**Success Criteria:**
- Verification accurate
- Trust store used correctly
- Output clear
- Failures explained
- Documentation comprehensive

---

### Phase 3: Development Commands (init, build) (Week 2)

#### Task 3.1: Init Command Implementation
**Deliverables:**
- Project initialization from templates
- Template selection (basic, service, worker, library)
- Component.toml generation
- Directory structure creation
- Init command documentation

**Success Criteria:**
- Templates work correctly
- Selection intuitive
- Component.toml valid
- Structure complete
- Documentation clear

#### Task 3.2: Build Command Implementation
**Deliverables:**
- Component compilation (cargo build --target wasm32-wasi)
- WASM optimization (wasm-opt integration)
- Component.toml validation
- Build artifact packaging
- Build command documentation

**Success Criteria:**
- Compilation works
- Optimization reduces size
- Validation catches errors
- Artifacts packaged correctly
- Build fast (<30s typical)

#### Task 3.3: Build Output and Caching
**Deliverables:**
- Build cache directory (.airswasm/cache)
- Incremental build support
- Build output formatting
- Build artifact metadata
- Caching documentation

**Success Criteria:**
- Caching speeds rebuilds
- Output informative
- Metadata accurate
- Cache manageable
- Documentation clear

---

### Phase 4: Lifecycle Commands (install, update, uninstall) (Week 2-3)

#### Task 4.1: Install Command Implementation
**Deliverables:**
- Multi-source installation (Git, Local, URL)
- Source URI parsing and validation
- Installation progress reporting
- Dependency resolution integration
- Install command documentation

**Success Criteria:**
- All sources work (Git, Local, URL)
- Progress visible
- Dependencies resolved
- Errors helpful
- Documentation comprehensive

#### Task 4.2: Update Command Implementation
**Deliverables:**
- Component version update
- Version constraint checking
- Update confirmation prompt
- Rollback on failure
- Update command documentation

**Success Criteria:**
- Updates work correctly
- Constraints respected
- Confirmation prevents accidents
- Rollback reliable
- Clear documentation

#### Task 4.3: Uninstall Command Implementation
**Deliverables:**
- Component removal
- Dependency checking (prevent breaking deps)
- Uninstall confirmation prompt
- Cleanup of component data
- Uninstall command documentation

**Success Criteria:**
- Removal works correctly
- Dependencies checked
- Confirmation prevents accidents
- Cleanup thorough
- Documentation clear

---

### Phase 5: Inspection Commands (list, info, status, logs) (Week 3-4)

#### Task 5.1: List Command Implementation
**Deliverables:**
- Installed component listing
- Table output (name, version, status)
- Filtering options (by status, name)
- Sorting options
- List command documentation

**Success Criteria:**
- Listing comprehensive
- Table formatted well
- Filtering works
- Sorting flexible
- Documentation clear

#### Task 5.2: Info Command Implementation
**Deliverables:**
- Component metadata display
- Detailed information (capabilities, dependencies)
- Version history
- Installation source
- Info command documentation

**Success Criteria:**
- Info comprehensive
- Format readable
- History complete
- Source visible
- Documentation clear

#### Task 5.3: Status Command Implementation
**Deliverables:**
- Component runtime status
- Health status display
- Resource usage (memory, CPU estimate)
- Uptime and message counts
- Status command documentation

**Success Criteria:**
- Status real-time
- Health visible
- Resources tracked
- Metrics useful
- Documentation clear

#### Task 5.4: Logs Command Implementation
**Deliverables:**
- Audit log viewing
- Log filtering (component, operation, time range)
- Log search (full-text)
- Log export (JSON)
- Logs command documentation

**Success Criteria:**
- Logs accessible
- Filtering flexible
- Search fast
- Export works
- Documentation comprehensive

---

### Phase 6: Shell Integration and Testing (Week 4-5)

#### Task 6.1: Config Command Implementation
**Deliverables:**
- Config get/set/list operations
- Config file editing support
- Config validation on save
- Config reset to defaults
- Config command documentation

**Success Criteria:**
- Config management intuitive
- Operations work correctly
- Validation prevents errors
- Reset safe
- Documentation clear

#### Task 6.2: Completions Command Implementation
**Deliverables:**
- Shell completion generation (bash, zsh, fish, powershell)
- Completion installation instructions
- Completion testing scripts
- Completion documentation

**Success Criteria:**
- Completions work for all shells
- Installation easy
- Testing automated
- Documentation comprehensive

#### Task 6.3: Comprehensive CLI Testing
**Deliverables:**
- Command execution tests
- Integration tests (with runtime)
- Error handling tests
- Shell completion tests
- CLI test suite

**Success Criteria:**
- Test coverage >90%
- All commands tested
- Error paths validated
- Completions tested
- CI pipeline green

---

## Success Criteria

### Definition of Done
This task is complete when:

1. ✅ **Security Commands Operational**
   - keygen, sign, verify working
   - Ed25519 operations correct
   - Trust store integration functional
   - Clear security documentation

2. ✅ **Development Commands Working**
   - init, build working
   - Templates functional
   - Build optimization working
   - Fast build times (<30s)

3. ✅ **Lifecycle Commands Complete**
   - install, update, uninstall working
   - Multi-source installation functional
   - Dependency resolution integrated
   - Rollback mechanisms reliable

4. ✅ **Inspection Commands Functional**
   - list, info, status, logs working
   - Output formatted well
   - Filtering and search operational
   - Real-time status accurate

5. ✅ **Configuration Working**
   - config command functional
   - Config file loaded correctly
   - Validation comprehensive
   - Defaults sensible

6. ✅ **Shell Integration Complete**
   - Completions for 4+ shells
   - Installation documented
   - Completions tested
   - Tab completion works

7. ✅ **UX Excellence**
   - Help system comprehensive
   - Error messages actionable
   - Output formatted professionally
   - Command execution fast (<1s local ops)

8. ✅ **Testing & Documentation Complete**
   - Test coverage >90%
   - All commands tested
   - Complete CLI guide
   - Developer satisfaction >95%

## Dependencies

### Upstream Dependencies
- ✅ airssys-wasm-cli foundation (10% complete) - **REQUIRED** for command structure
- ✅ WASM-TASK-008: Component Lifecycle (Block 7) - **REQUIRED** for install/update/uninstall
- ✅ WASM-TASK-011: Component SDK (Block 10) - **REQUIRED** for init/build commands
- ✅ WASM-TASK-010: Monitoring & Observability (Block 9) - **REQUIRED** for logs/status

### Downstream Dependencies (Blocks This Task)
- None - this is the final block completing the framework

### External Dependencies
- clap v4 (CLI framework)
- ed25519-dalek (Ed25519 operations)
- colored/termcolor (terminal colors)
- comfy-table (table formatting)
- indicatif (progress bars)
- clap_complete (shell completions)

## Risks and Mitigations

### Risk 1: Shell Completion Compatibility
**Impact:** Medium - Completions may not work in all shells  
**Probability:** Medium - Shell differences complex  
**Mitigation:**
- Test completions in all major shells
- Document known limitations
- Provide fallback instructions
- Community testing across platforms

### Risk 2: CLI Performance
**Impact:** Medium - Slow commands frustrate users  
**Probability:** Low - Most operations fast  
**Mitigation:**
- Benchmark all commands
- Optimize hot paths
- Progress indicators for slow operations
- Target <1s for local operations

### Risk 3: Error Message Quality
**Impact:** High - Poor errors confuse users  
**Probability:** Medium - Error messages require iteration  
**Mitigation:**
- User testing for error scenarios
- Suggest fixes in error messages
- Clear error categories
- Comprehensive error documentation

### Risk 4: Cross-Platform Compatibility
**Impact:** High - Tool must work on all platforms  
**Probability:** Medium - Platform differences complex  
**Mitigation:**
- CI testing on Linux, macOS, Windows
- Platform-specific code isolated
- Clear platform requirements
- Community testing

### Risk 5: Configuration Complexity
**Impact:** Low - Complex config confuses users  
**Probability:** Medium - Config can grow complex  
**Mitigation:**
- Sensible defaults (minimal config needed)
- Config validation with clear errors
- Configuration documentation comprehensive
- Examples for common scenarios

## Progress Tracking

**Overall Status:** not-started - 0%

### Phase Breakdown
| Phase | Description | Status | Estimated Duration | Notes |
|-------|-------------|--------|-------------------|-------|
| 1 | CLI Foundation and Configuration | not-started | Week 1 | Foundation |
| 2 | Security Commands | not-started | Week 1-2 | keygen, sign, verify |
| 3 | Development Commands | not-started | Week 2 | init, build |
| 4 | Lifecycle Commands | not-started | Week 2-3 | install, update, uninstall |
| 5 | Inspection Commands | not-started | Week 3-4 | list, info, status, logs |
| 6 | Shell Integration and Testing | not-started | Week 4-5 | config, completions, QA |

### Subtasks
| ID | Description | Status | Updated | Notes |
|----|-------------|--------|---------|-------|
| 1.1 | CLI Framework Setup | not-started | - | Foundation |
| 1.2 | Configuration System | not-started | - | Config |
| 1.3 | Rich Terminal Output | not-started | - | UX |
| 2.1 | Keygen Command Implementation | not-started | - | Key generation |
| 2.2 | Sign Command Implementation | not-started | - | Signing |
| 2.3 | Verify Command Implementation | not-started | - | Verification |
| 3.1 | Init Command Implementation | not-started | - | Project init |
| 3.2 | Build Command Implementation | not-started | - | Compilation |
| 3.3 | Build Output and Caching | not-started | - | Build optimization |
| 4.1 | Install Command Implementation | not-started | - | Installation |
| 4.2 | Update Command Implementation | not-started | - | Updates |
| 4.3 | Uninstall Command Implementation | not-started | - | Removal |
| 5.1 | List Command Implementation | not-started | - | Listing |
| 5.2 | Info Command Implementation | not-started | - | Metadata |
| 5.3 | Status Command Implementation | not-started | - | Runtime status |
| 5.4 | Logs Command Implementation | not-started | - | Audit logs |
| 6.1 | Config Command Implementation | not-started | - | Configuration |
| 6.2 | Completions Command Implementation | not-started | - | Shell integration |
| 6.3 | Comprehensive CLI Testing | not-started | - | Quality assurance |

## Progress Log

*No progress yet - task just created*

## Related Documentation

### ⭐ Essential Reading (MUST READ BEFORE STARTING)
- **KNOWLEDGE-WASM-015: Project Structure and Workspace Architecture** - **CRITICAL**
  - Explains the three sub-projects (airssys-wasm, airssys-wasm-component, airssys-wasm-cli)
  - Maps tasks to crates (this task = airssys-wasm-cli)
  - Clarifies dependency relationships and integration points
  - **READ THIS FIRST** to understand context

### ADRs
- **ADR-WASM-013: CLI Design Philosophy** - (Future) CLI UX decisions

### Knowledge Documentation
- **KNOWLEDGE-WASM-010: CLI Tool Specification** - **COMPLETE** CLI specification (ESSENTIAL)
- **KNOWLEDGE-WASM-009: Component Installation Architecture** - Lifecycle operations reference
- **KNOWLEDGE-WASM-012: SDK Design Patterns** - Init/build integration
- **KNOWLEDGE-WASM-001: Component Framework Architecture** - Core architecture

### Related Sub-Projects
- **airssys-wasm** (`../airssys-wasm/`) - Core library that CLI uses (Blocks 1-9)
- **airssys-wasm-component** (`../airssys-wasm-component/`) - SDK for init templates (Block 10)

### External References
- [clap CLI Framework](https://docs.rs/clap/)
- [Ed25519 Digital Signatures](https://ed25519.cr.yp.to/)
- [Shell Completion Guide](https://github.com/clap-rs/clap/tree/master/clap_complete)

## Notes

**Complete Command Reference:**

**Security Operations:**
- `component keygen [--output <path>]` - Generate Ed25519 key pair
- `component sign <component> [--key <path>]` - Sign component
- `component verify <component> [--key <path>]` - Verify signature

**Development Workflow:**
- `component init <name> [--template <type>]` - Initialize new component project
- `component build [--release] [--optimize]` - Build component WASM

**Lifecycle Management:**
- `component install <source> [--name <name>]` - Install component from Git/Local/URL
- `component update <name> [--version <constraint>]` - Update component version
- `component uninstall <name> [--force]` - Remove component

**Inspection and Debugging:**
- `component list [--status <filter>]` - List installed components
- `component info <name>` - Show detailed component information
- `component status <name>` - Show runtime status and health
- `component logs [--component <name>] [--operation <type>]` - View audit logs

**Configuration:**
- `component config get <key>` - Get configuration value
- `component config set <key> <value>` - Set configuration value
- `component config list` - List all configuration
- `component config reset` - Reset to default configuration

**Shell Integration:**
- `component completions <shell>` - Generate shell completions (bash, zsh, fish, powershell)

**CLI Design Principles:**
1. **Intuitive** - Commands follow natural language patterns
2. **Consistent** - Common flags work across commands
3. **Helpful** - Rich help system with examples
4. **Fast** - Operations complete quickly (<1s local)
5. **Safe** - Destructive operations require confirmation
6. **Informative** - Output shows progress and results clearly

**Example Usage Flow:**
```bash
# Generate signing key
component keygen --output ~/.airswasm/keys/my-key

# Create new component
component init my-service --template service
cd my-service

# Build component
component build --release --optimize

# Sign component
component sign target/wasm32-wasi/release/my-service.wasm \
  --key ~/.airswasm/keys/my-key.pem

# Install locally for testing
component install ./target/wasm32-wasi/release/my-service.wasm \
  --name my-service-dev

# Check status
component status my-service-dev

# View logs
component logs --component my-service-dev --operation http-request

# Update to new version
component update my-service-dev --version 0.2.0

# Uninstall when done
component uninstall my-service-dev
```

**Configuration File Format:**
```toml
# ~/.airswasm/config.toml

[runtime]
log_level = "info"
max_memory = "4GB"

[install]
default_source = "git"
trust_unsigned = false

[build]
optimization_level = "z"  # Size optimization
strip_debug = true

[output]
color = "auto"
format = "table"
```

**Rich Terminal Output Examples:**
```
# List command output (table format)
┌────────────────┬─────────┬──────────┬───────────┐
│ Name           │ Version │ Status   │ Uptime    │
├────────────────┼─────────┼──────────┼───────────┤
│ my-service     │ 0.2.0   │ healthy  │ 2d 5h     │
│ worker-pool    │ 1.0.1   │ degraded │ 12h 30m   │
│ file-processor │ 0.1.5   │ healthy  │ 1d 18h    │
└────────────────┴─────────┴──────────┴───────────┘

# Install command output (progress)
Installing my-service from git://github.com/user/my-service
  ✓ Cloning repository... done
  ✓ Verifying signature... done
  ✓ Resolving dependencies... done
  ⠋ Building component... 
```

**Error Message Examples:**
```
# Helpful error with suggestion
Error: Signature verification failed for 'my-service'

The signature does not match the component content.
This could mean:
  • The component was modified after signing
  • The wrong public key was used
  • The signature file is corrupted

Try:
  • Re-signing the component with: component sign my-service.wasm
  • Verifying with the correct key: component verify --key <path>

# Configuration error with fix
Error: Invalid configuration value for 'runtime.max_memory'

Expected: Memory size (e.g., "4GB", "512MB")
Got: "4000"

Fix with: component config set runtime.max_memory 4GB
```

**Shell Completion Example (zsh):**
```zsh
# After: component completions zsh > ~/.zsh/completions/_component
$ component in<TAB>
init     info     install  

$ component install --<TAB>
--help    --name    --version    --force
```

**Phase 2 Enhancements:**
- Interactive installation wizard (guided prompts)
- Component dependency graph visualization
- Remote component management (SSH/cluster)
- Component marketplace search integration
- TUI interface for log viewing (like `htop`)
- Component performance profiler
- Batch operations (install/update multiple components)
