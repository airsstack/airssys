# Product Context: airssys-wasm-cli

**Sub-Project:** airssys-wasm-cli  
**Last Updated:** 2025-10-18  
**Product Identity:** Developer-first CLI tool for WASM component lifecycle management

---

## Product Vision

### The Problem

**For developers building WebAssembly components**, managing the complete lifecycle from development to deployment is complex and fragmented:

- **No unified tooling**: Developers use multiple tools (cargo, wasm-pack, custom scripts) for build, sign, install operations
- **Security friction**: Manual signing and verification processes are error-prone
- **Installation complexity**: No standard way to install from Git, local paths, or registries
- **Poor visibility**: No centralized way to view installed components, logs, or health status
- **DevOps gaps**: Updating and managing components in production is manual and risky

### The Solution

**airssys-wasm** is a comprehensive CLI tool that provides a **unified, developer-friendly interface** for the complete WASM component lifecycle:

```bash
# Generate signing keys once
airssys-wasm keygen

# Initialize new component project
airssys-wasm init my-component

# Build and sign component
airssys-wasm build
airssys-wasm sign --key ~/.airssys/keys/private.pem

# Install from multiple sources
airssys-wasm install ./my-component.wasm          # Local
airssys-wasm install github:org/repo              # Git
airssys-wasm install registry:my-component@1.0.0  # Registry

# Manage deployed components
airssys-wasm list
airssys-wasm status my-component
airssys-wasm logs my-component --follow
airssys-wasm update my-component

# Security verification
airssys-wasm verify my-component.wasm
```

---

## Target Audience

### Primary Users

**Component Developers** (60% of users)
- **Needs**: Fast development workflow, easy testing, clear error messages
- **Pain Points**: Fragmented tooling, manual build steps, unclear deployment process
- **Value Prop**: Single command for build → sign → test → install workflow
- **Key Commands**: `init`, `build`, `sign`, `install` (local)

**DevOps Engineers** (30% of users)
- **Needs**: Reliable deployment, health monitoring, easy rollback
- **Pain Points**: No visibility into component health, manual updates, unclear failure modes
- **Value Prop**: Production-ready management with monitoring and automation
- **Key Commands**: `install` (registry), `update`, `status`, `logs`, `list`

**System Administrators** (10% of users)
- **Needs**: Security verification, audit trails, component inventory
- **Pain Points**: No trust verification, unclear provenance, scattered logs
- **Value Prop**: Security-first operations with comprehensive audit trails
- **Key Commands**: `verify`, `list`, `info`, `logs`, `uninstall`

### User Personas

**"Alex" - Full-Stack Developer**
- Builds WASM components for web applications
- Wants fast iteration cycles
- Expects modern CLI UX (colors, progress bars, good errors)
- Primarily uses `init`, `build`, `install` locally

**"Jordan" - Platform Engineer**
- Manages WASM components in Kubernetes clusters
- Needs automation and scripting capabilities
- Requires reliability and clear exit codes
- Primarily uses `install`, `update`, `status`, `logs`

**"Taylor" - Security Engineer**
- Audits component deployments for compliance
- Needs signature verification and provenance tracking
- Requires detailed logs and audit trails
- Primarily uses `verify`, `info`, `list`

---

## User Experience Philosophy

### Core Principles

**1. Progressive Disclosure**
- Simple commands work out-of-the-box with sensible defaults
- Advanced options available via flags for power users
- Help text is comprehensive but not overwhelming

**2. Fast Feedback**
- Operations provide immediate progress indication
- Errors are clear, actionable, and suggest fixes
- Success confirmations are visible and satisfying

**3. Safety by Default**
- Destructive operations require confirmation
- Dry-run mode available for risky commands
- Clear warnings before overwriting files

**4. Consistency**
- All commands follow same argument patterns
- Output format is predictable and parseable
- Error handling is uniform across commands

### Design Patterns

**Command Structure:**
```bash
airssys-wasm <COMMAND> [OPTIONS] [ARGS]
```

**Flag Conventions:**
- `--help, -h`: Show help
- `--version, -V`: Show version
- `--verbose, -v`: Verbose output
- `--quiet, -q`: Minimal output
- `--yes, -y`: Skip confirmations
- `--dry-run`: Show what would happen without executing

**Output Conventions:**
- ✓ Success: Green text with checkmark
- ✗ Error: Red text with cross
- ⚠ Warning: Yellow text with warning symbol
- ℹ Info: Blue text with info symbol

---

## User Workflows

### Workflow 1: New Component Development

**User Goal:** Create, develop, and test a new component

**Steps:**
1. Generate signing key (one-time setup)
   ```bash
   airssys-wasm keygen
   # Output: ✓ Keypair generated at ~/.airssys/keys/
   ```

2. Initialize component project
   ```bash
   airssys-wasm init my-auth-component
   # Interactive prompts:
   # - Component name: my-auth-component
   # - Template: basic / advanced / custom
   # Output: ✓ Component initialized at ./my-auth-component
   ```

3. Develop component (edit code)

4. Build component
   ```bash
   cd my-auth-component
   airssys-wasm build
   # Output: 
   # Building component... [========>] 90%
   # ✓ Build complete: ./target/wasm32-wasi/release/my-auth-component.wasm
   ```

5. Sign component
   ```bash
   airssys-wasm sign --key ~/.airssys/keys/private.pem
   # Output: ✓ Component signed successfully
   ```

6. Install locally for testing
   ```bash
   airssys-wasm install ./target/wasm32-wasi/release/my-auth-component.wasm
   # Output: ✓ Component installed: my-auth-component@0.1.0
   ```

7. Test and iterate (repeat steps 3-6)

**Pain Points Addressed:**
- No need to remember multiple tool commands
- Automatic signing step prevents forgetting
- Local installation for testing without deployment

### Workflow 2: Production Deployment

**User Goal:** Deploy component to production environment

**Steps:**
1. Install from registry
   ```bash
   airssys-wasm install registry:my-auth-component@1.2.0
   # Output:
   # Checking cache... not found
   # Downloading from registry... [========>] 100%
   # Verifying signature... ✓
   # Installing component... ✓
   # ✓ Component installed: my-auth-component@1.2.0
   ```

2. Verify installation
   ```bash
   airssys-wasm status my-auth-component
   # Output:
   # Component: my-auth-component
   # Version: 1.2.0
   # Status: ✓ Running
   # Health: ✓ Healthy
   # Uptime: 2m 34s
   ```

3. Monitor logs
   ```bash
   airssys-wasm logs my-auth-component --follow
   # Output: (streaming logs)
   # [INFO] Component started
   # [INFO] Handling authentication request...
   ```

4. Update component (when new version available)
   ```bash
   airssys-wasm update my-auth-component
   # Output:
   # Current version: 1.2.0
   # Latest version: 1.3.0
   # Update component? [y/N]: y
   # Downloading... [========>] 100%
   # Verifying signature... ✓
   # Stopping old version... ✓
   # Installing new version... ✓
   # ✓ Component updated: my-auth-component@1.3.0
   ```

**Pain Points Addressed:**
- Automatic signature verification (no manual steps)
- Health monitoring built-in
- Safe updates with clear prompts
- Easy rollback if needed

### Workflow 3: Security Audit

**User Goal:** Verify component integrity and provenance

**Steps:**
1. List all installed components
   ```bash
   airssys-wasm list
   # Output:
   # Installed Components:
   # - my-auth-component@1.2.0 (registry)
   # - logging-filter@2.1.0 (git:github.com/org/repo)
   # - custom-validator@0.3.0 (local)
   ```

2. Get detailed component information
   ```bash
   airssys-wasm info my-auth-component
   # Output:
   # Component: my-auth-component
   # Version: 1.2.0
   # Source: registry:my-auth-component@1.2.0
   # Signature: ✓ Valid (signed by: alice@example.com)
   # Installed: 2025-10-15 14:23:11
   # Size: 2.3 MB
   # Dependencies: none
   ```

3. Verify component signature
   ```bash
   airssys-wasm verify my-auth-component.wasm
   # Output:
   # Verifying signature...
   # Public key: 0x1234...abcd
   # Signature: ✓ Valid
   # Signed by: alice@example.com
   # Signed on: 2025-10-15 10:15:32
   ```

4. Review audit logs
   ```bash
   airssys-wasm logs my-auth-component --since 24h
   # Output: (filtered logs from last 24 hours)
   ```

**Pain Points Addressed:**
- Quick inventory of all components
- Easy verification of signatures
- Clear provenance information
- Audit trail readily available

---

## User Experience Details

### Onboarding Experience

**First-Time User Flow:**

1. **Installation** (via package manager)
   ```bash
   # macOS
   brew install airssys-wasm
   
   # Linux
   curl -sSL https://get.airsstack.org/cli | sh
   
   # Windows
   scoop install airssys-wasm
   ```

2. **Initial Setup** (automatic on first command)
   ```bash
   airssys-wasm keygen
   # Output:
   # Welcome to AirsSys WASM CLI!
   # This tool helps you manage WebAssembly components.
   # 
   # First, let's generate a signing key for your components.
   # 
   # Generating Ed25519 keypair...
   # ✓ Private key saved: ~/.airssys/keys/private.pem
   # ✓ Public key saved: ~/.airssys/keys/public.pem
   # 
   # ⚠ Keep your private key secure! It's used to sign components.
   # 
   # Next steps:
   # - Initialize a component: airssys-wasm init my-component
   # - View help: airssys-wasm --help
   ```

3. **Configuration** (optional customization)
   ```bash
   airssys-wasm config set registry https://my-registry.example.com
   # Output: ✓ Registry updated
   
   airssys-wasm config set install-dir /opt/airssys/components
   # Output: ✓ Install directory updated
   ```

**Help System:**
- `--help` flag on every command
- Rich help text with examples
- Command suggestions for typos
- Quick reference guide in `airssys-wasm help`

### Error Experience

**Error Philosophy:**
- Errors explain **what** went wrong
- Errors suggest **how** to fix it
- Errors provide **context** (file paths, command args)

**Error Examples:**

**Good Error (Component Not Found):**
```
✗ Error: Component 'my-component' not found

Possible causes:
  - Component name is incorrect (did you mean 'my-auth-component'?)
  - Component not installed (run 'airssys-wasm list' to see installed components)
  - Component removed (check 'airssys-wasm logs' for uninstall records)

Suggestions:
  - Install component: airssys-wasm install registry:my-component
  - List all components: airssys-wasm list
```

**Good Error (Invalid Signature):**
```
✗ Error: Signature verification failed for 'component.wasm'

Details:
  - Signature: Invalid
  - Expected key: 0x1234...abcd
  - Actual key: 0x5678...efgh
  - File: /path/to/component.wasm

This component may be:
  - Corrupted during download
  - Tampered with
  - Signed with a different key

Suggestions:
  - Re-download component: airssys-wasm install --force
  - Verify source authenticity before installing
  - Contact component author to confirm signing key
```

**Good Error (Missing Configuration):**
```
✗ Error: Configuration file not found

Expected location: ~/.airssys/config.toml

This usually happens on first use. Let's create a default configuration.

Create default config? [Y/n]: y
✓ Configuration created with default values

Edit configuration: airssys-wasm config edit
View configuration: airssys-wasm config show
```

### Progress Indication

**Progress Bar (Downloads):**
```
Downloading my-component@1.2.0...
[========================================>] 2.3 MB / 2.3 MB (100%) ETA: 0s
```

**Spinner (Processing):**
```
Building component... ⠋
```

**Multi-Step Progress:**
```
Installing my-component@1.2.0...
  ✓ Checking cache
  ✓ Downloading component (2.3 MB)
  ✓ Verifying signature
  ✓ Extracting files
  ✓ Registering component
✓ Installation complete
```

---

## Configuration Experience

### Config File Structure

**Location:** `~/.airssys/config.toml`

**Default Config:**
```toml
# AirsSys WASM CLI Configuration
# Edit with: airssys-wasm config edit

[registry]
url = "https://registry.airsstack.org"
timeout = 30

[install]
directory = "~/.airssys/components"
verify_signatures = true
auto_update = false

[cache]
directory = "~/.airssys/cache"
max_size_mb = 1024
cleanup_age_days = 30

[keys]
private_key = "~/.airssys/keys/private.pem"
public_key = "~/.airssys/keys/public.pem"

[logging]
level = "info"
directory = "~/.airssys/logs"
max_size_mb = 100
max_age_days = 7
```

### Config Management

**View Configuration:**
```bash
airssys-wasm config show
# Output: (displays current config in TOML format)
```

**Edit Configuration:**
```bash
airssys-wasm config edit
# Opens config file in $EDITOR
```

**Set Individual Values:**
```bash
airssys-wasm config set registry.url https://my-registry.example.com
airssys-wasm config set install.directory /opt/components
airssys-wasm config set logging.level debug
```

**Get Individual Values:**
```bash
airssys-wasm config get registry.url
# Output: https://registry.airsstack.org
```

**Reset to Defaults:**
```bash
airssys-wasm config reset
# Output:
# ⚠ This will reset configuration to defaults.
# Current config will be backed up to ~/.airssys/config.toml.backup
# Continue? [y/N]: y
# ✓ Configuration reset to defaults
```

---

## Advanced User Features

### Scripting and Automation

**Machine-Readable Output (JSON):**
```bash
airssys-wasm list --format json
# Output: {"components": [{"name": "my-component", "version": "1.2.0", ...}]}

airssys-wasm status my-component --format json
# Output: {"name": "my-component", "status": "running", "health": "healthy", ...}
```

**Exit Codes:**
- `0`: Success
- `1`: General error
- `2`: Command parse error
- `3`: Component not found
- `4`: Signature verification failed
- `5`: Network error
- `6`: Permission denied

**Non-Interactive Mode:**
```bash
# Skip confirmations with --yes flag
airssys-wasm update my-component --yes

# Quiet output for scripting
airssys-wasm install registry:my-component --quiet
```

### Shell Completion

**Generate Completions:**
```bash
airssys-wasm completions bash > ~/.airssys/completions/airssys-wasm.bash
source ~/.airssys/completions/airssys-wasm.bash
```

**Completion Features:**
- Command name completion
- Flag completion with descriptions
- Component name completion (from installed components)
- File path completion for local installs

### Batch Operations

**Multiple Component Updates:**
```bash
# Update all components
airssys-wasm update --all

# Update specific components
airssys-wasm update my-component-1 my-component-2 my-component-3
```

**Bulk Installation:**
```bash
# Install from manifest file
airssys-wasm install --from-file components.txt

# components.txt:
# registry:my-component-1@1.0.0
# registry:my-component-2@2.1.0
# github:org/repo@v3.0.0
```

---

## Performance Expectations

### Response Time Targets

| Operation | Target | Acceptable | Notes |
|-----------|--------|------------|-------|
| `keygen` | <100ms | <500ms | CPU-bound Ed25519 generation |
| `init` | <200ms | <1s | Template extraction + file creation |
| `build` | Depends | Depends | Component build time varies |
| `sign` | <50ms | <200ms | Ed25519 signing is fast |
| `install` (cached) | <500ms | <2s | Local file operations |
| `install` (network) | <5s | <30s | Network-dependent |
| `update` | <5s | <30s | Network-dependent |
| `list` | <100ms | <500ms | Local metadata query |
| `info` | <100ms | <500ms | Local metadata query |
| `status` | <200ms | <1s | Runtime health check |
| `logs` | <500ms | <2s | Log file access |
| `verify` | <200ms | <1s | Signature verification |
| `config` | <50ms | <200ms | Config file I/O |

### User Perception

**Instant (0-100ms):** No feedback needed
- Simple config operations
- Help text display

**Fast (100-1000ms):** Spinner indication
- Local file operations
- Metadata queries
- Signature operations

**Responsive (1-5s):** Progress bar
- Network operations
- Component downloads
- Installation operations

**Long (5s+):** Progress bar + ETA
- Large component downloads
- Batch operations
- Build operations

---

## Accessibility Considerations

### Color Blindness Support

**Color + Symbol Redundancy:**
```
✓ Success (green + checkmark)
✗ Error (red + cross)
⚠ Warning (yellow + warning triangle)
ℹ Info (blue + info circle)
```

**Disable Colors:**
```bash
airssys-wasm --no-color install my-component
# or
export NO_COLOR=1
airssys-wasm install my-component
```

### Screen Reader Support

**Structured Output:**
- Plain text fallback when colors disabled
- Clear semantic structure (headings, lists)
- Avoid ASCII art in critical output

**Verbose Mode:**
```bash
airssys-wasm --verbose install my-component
# Provides detailed text descriptions of all operations
```

---

## Future UX Enhancements

### Planned Features (Phase 4+)

**Interactive Installation Wizard:**
```bash
airssys-wasm install --interactive
# Guided flow:
# 1. Select installation source (local/git/registry)
# 2. Enter component identifier
# 3. Select version (if multiple available)
# 4. Configure options (if component supports)
# 5. Confirm and install
```

**Component Search:**
```bash
airssys-wasm search "authentication"
# Output:
# Found 5 components matching "authentication":
# 1. auth-jwt@2.1.0 - JWT authentication component
# 2. auth-oauth@1.5.0 - OAuth2 authentication
# ...
```

**Dependency Visualization:**
```bash
airssys-wasm deps my-component --tree
# Output:
# my-component@1.2.0
# ├── logger@3.0.0
# │   └── serializer@1.1.0
# └── validator@2.5.0
```

**Health Dashboard (TUI):**
```bash
airssys-wasm dashboard
# Interactive terminal UI showing:
# - All installed components
# - Real-time status
# - Resource usage
# - Recent logs
```

---

## User Documentation Strategy

### Documentation Levels

**1. Inline Help (`--help`)**
- Command syntax and flags
- Brief description
- Common examples

**2. Man Pages**
- Detailed command reference
- All flags and options
- Environment variables
- Exit codes

**3. Online Docs (website)**
- Getting started tutorial
- Workflow guides
- Troubleshooting
- API reference

**4. Examples Repository**
- Real-world component examples
- Integration patterns
- CI/CD configurations

### Help Content Quality

**Good Help Text Example:**
```bash
airssys-wasm install --help

# Output:
Install a WebAssembly component

Usage: airssys-wasm install [OPTIONS] <SOURCE>

Arguments:
  <SOURCE>  Component source:
            - Local path: ./component.wasm
            - Git repository: github:org/repo[@version]
            - Registry: registry:name[@version]

Options:
  -f, --force           Overwrite existing component
      --verify          Verify signature (default: true)
      --no-verify       Skip signature verification
      --cache           Use cached version if available
  -q, --quiet           Suppress output
  -v, --verbose         Verbose output
  -h, --help            Print help

Examples:
  # Install from local file
  airssys-wasm install ./my-component.wasm

  # Install from GitHub
  airssys-wasm install github:org/repo

  # Install specific version from registry
  airssys-wasm install registry:my-component@1.2.0

  # Force reinstall without verification (not recommended)
  airssys-wasm install --force --no-verify ./component.wasm
```

---

## Success Metrics

### User Satisfaction Indicators

**Adoption Metrics:**
- Installation count
- Daily active users
- Retention rate (7-day, 30-day)

**Engagement Metrics:**
- Commands per user session
- Component management operations per day
- Feature usage distribution

**Quality Metrics:**
- Error rate (% of commands that fail)
- Average operation duration
- Support ticket volume
- GitHub issue count

**Developer Experience Metrics:**
- Time to first component (onboarding speed)
- Commands to complete workflow
- User-reported satisfaction (surveys)

### Target Benchmarks (v1.0)

- **Time to first component:** <10 minutes
- **Error rate:** <2% of all operations
- **Command success rate:** >98%
- **User satisfaction:** >4.5/5 stars
- **Documentation clarity:** >4.0/5 stars

---

## Related Documentation

- project_brief.md - Project goals and scope
- tech_context.md - Technical implementation
- system_patterns.md - UX implementation patterns
- active_context.md - Current development focus
