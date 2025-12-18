# KNOWLEDGE-CLI-002: Composable CLI Pattern

**Document ID:** KNOWLEDGE-CLI-002  
**Created:** 2025-12-18  
**Updated:** 2025-12-18  
**Category:** Architecture / Patterns  
**Maturity:** Stable  
**Related ADR:** ADR-CLI-001

---

## Overview

The **Composable CLI Pattern** is an architectural approach where CLI tools are implemented as **100% library code** with zero binary components, exporting Clap-based command structures that can be composed by any binary application. This pattern enables maximum reusability, testability, and flexibility for CLI commands across multiple binaries.

**Key Principle**: CLI logic lives in a library, binaries only compose and route commands.

---

## Context

### Problem Statement

Traditional CLI tools are implemented as standalone binaries with monolithic main functions. This creates several challenges:

1. **Reusability**: Cannot reuse CLI commands in different binaries
2. **Composition**: Cannot combine multiple CLI tools into a unified interface
3. **Testing**: Difficult to test CLI logic without spawning processes
4. **Distribution**: Must distribute separate binaries for each tool
5. **Maintainability**: Duplicate code across similar CLI tools

### Scope

This pattern applies to:
- ✅ CLI tools that may be composed into larger applications
- ✅ CLI commands that need to be reusable across projects
- ✅ Projects requiring flexible binary distribution strategies
- ✅ Teams building CLI ecosystems with shared commands

This pattern does NOT apply to:
- ❌ Simple, standalone CLI utilities with no composition needs
- ❌ One-off scripts or tools
- ❌ CLIs with heavy binary-specific initialization

### Prerequisites

**Technical Requirements**:
- Rust with Clap 4.x (derive feature)
- Understanding of Cargo workspace architecture
- Familiarity with library vs binary patterns

**Architectural Requirements**:
- Clear separation between CLI presentation and business logic
- Well-defined API boundaries between CLI and core logic
- Composable command structure

---

## Technical Content

### Core Concepts

#### 1. Library-Only Architecture

**Structure**:
```
my-cli/
├── Cargo.toml          # type = "lib" (NO [[bin]] section)
├── src/
│   ├── lib.rs          # Public exports
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── trust.rs    # TrustArgs, TrustCommands, execute()
│   │   └── ...
│   └── utils.rs
```

**Key Files**:

```rust
// src/lib.rs
pub mod commands {
    pub mod trust;
    pub mod keygen;
    // ... other command modules
}

// Re-export for convenience
pub use commands::trust::{TrustArgs, TrustCommands};
```

```rust
// src/commands/trust.rs
use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub struct TrustArgs {
    #[command(subcommand)]
    pub command: TrustCommands,
}

#[derive(Debug, Subcommand)]
pub enum TrustCommands {
    Add { name: String },
    Remove { name: String },
    List,
}

pub async fn execute(args: &TrustArgs) -> Result<()> {
    match &args.command {
        TrustCommands::Add { name } => { /* ... */ }
        TrustCommands::Remove { name } => { /* ... */ }
        TrustCommands::List => { /* ... */ }
    }
    Ok(())
}
```

#### 2. Composing in a Binary

**Binary Project** (separate crate or workspace member):

```rust
// my-app/src/main.rs
use clap::{Parser, Subcommand};
use my_cli::commands::trust::{TrustArgs, execute as execute_trust};

#[derive(Debug, Parser)]
#[command(name = "my-app")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Trust management commands
    Trust(TrustArgs),
    
    /// Other commands...
    SomeOtherCommand { /* ... */ },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Trust(args) => execute_trust(&args).await?,
        Commands::SomeOtherCommand { /* ... */ } => { /* ... */ }
    }
    
    Ok(())
}
```

**Result**: Binary composes library commands seamlessly.

#### 3. Multiple Binary Composition

**Example**: Three different binaries use the same CLI library:

```rust
// Binary 1: Full-featured CLI
enum Commands {
    Trust(TrustArgs),
    Keygen(KeygenArgs),
    Build(BuildArgs),
    // ... all commands
}

// Binary 2: Minimal CLI (only trust management)
enum Commands {
    Trust(TrustArgs),
}

// Binary 3: Enterprise CLI (adds custom commands)
enum Commands {
    Trust(TrustArgs),
    Enterprise(EnterpriseArgs),  // Custom command
}
```

All three binaries reuse the same `TrustArgs` and `execute()` implementation.

---

### Implementation Details

#### Pattern Structure

**Library Side**:
1. Define Clap structures (`Args`, `Subcommand` derives)
2. Implement `execute()` function(s)
3. Export from `lib.rs`
4. Zero `[[bin]]` sections in Cargo.toml

**Binary Side**:
1. Import library command structures
2. Compose into top-level `Commands` enum
3. Route to library `execute()` functions
4. Handle top-level concerns (logging, config, etc.)

#### Code Examples

**Example 1: Simple Command**

```rust
// Library: my-cli/src/commands/hello.rs
use clap::Args;

#[derive(Debug, Args)]
pub struct HelloArgs {
    /// Name to greet
    name: String,
}

pub async fn execute(args: &HelloArgs) -> Result<()> {
    println!("Hello, {}!", args.name);
    Ok(())
}
```

**Example 2: Complex Subcommands**

```rust
// Library: my-cli/src/commands/database.rs
use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub struct DatabaseArgs {
    #[command(subcommand)]
    pub command: DatabaseCommands,
}

#[derive(Debug, Subcommand)]
pub enum DatabaseCommands {
    Connect { url: String },
    Query { sql: String },
    Migrate { version: Option<u32> },
}

pub async fn execute(args: &DatabaseArgs) -> Result<()> {
    match &args.command {
        DatabaseCommands::Connect { url } => connect_db(url).await,
        DatabaseCommands::Query { sql } => run_query(sql).await,
        DatabaseCommands::Migrate { version } => migrate_db(*version).await,
    }
}
```

**Example 3: Flatten Pattern (Alternative)**

```rust
// Binary: Flatten all commands at top level
use clap::Parser;

#[derive(Debug, Parser)]
struct Cli {
    #[command(flatten)]
    trust: Option<TrustArgs>,
    
    #[command(flatten)]
    keygen: Option<KeygenArgs>,
}
```

**Note**: Subcommand pattern is generally preferred over flatten for clarity.

---

### Configuration

**Cargo.toml (Library)**:
```toml
[package]
name = "my-cli"
version = "0.1.0"
edition = "2021"

# NO [[bin]] section!

[dependencies]
clap = { version = "4.5", features = ["derive"] }
tokio = { version = "1.47", features = ["full"] }
# ... other deps
```

**Cargo.toml (Binary)**:
```toml
[package]
name = "my-app"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "my-app"
path = "src/main.rs"

[dependencies]
my-cli = { path = "../my-cli" }
clap = { version = "4.5", features = ["derive"] }
tokio = { version = "1.47", features = ["full"] }
```

---

## Usage Patterns

### Common Use Cases

#### Use Case 1: Multi-Binary Distribution

**Scenario**: Distribute both a full-featured CLI and a minimal CLI.

**Implementation**:
```
workspace/
├── my-cli/          # Library (all commands)
├── my-app-full/     # Binary (all commands)
└── my-app-lite/     # Binary (subset of commands)
```

**Benefits**:
- Single source of truth for command logic
- Different binaries for different user needs
- Minimal code duplication

#### Use Case 2: Plugin Architecture

**Scenario**: Allow third-party plugins to extend CLI.

**Implementation**:
```rust
// Plugin API
pub trait CliCommand {
    fn args(&self) -> Box<dyn Args>;
    fn execute(&self, args: &dyn Args) -> Result<()>;
}

// Binary dynamically loads plugins
let plugins = load_plugins()?;
for plugin in plugins {
    // Register plugin commands
}
```

#### Use Case 3: Testing CLI Logic

**Scenario**: Test CLI commands without spawning processes.

**Implementation**:
```rust
#[tokio::test]
async fn test_trust_add() {
    let args = TrustArgs {
        command: TrustCommands::Add {
            name: "test".to_string(),
        },
    };
    
    let result = execute(&args).await;
    assert!(result.is_ok());
}
```

**Benefits**:
- Fast unit tests (no process spawning)
- Easy mocking and dependency injection
- Direct assertion on return values

---

### Best Practices

#### 1. Export Pattern
✅ **DO**: Export both args and execute functions
```rust
pub use commands::trust::{TrustArgs, TrustCommands, execute as execute_trust};
```

❌ **DON'T**: Hide implementation details from binaries
```rust
mod commands { /* private */ }
```

#### 2. Async Execution
✅ **DO**: Make execute functions async for I/O operations
```rust
pub async fn execute(args: &TrustArgs) -> Result<()> { /* ... */ }
```

❌ **DON'T**: Block in synchronous code
```rust
pub fn execute(args: &TrustArgs) -> Result<()> {
    std::fs::read_to_string("file.txt")?; // Blocks!
}
```

#### 3. Error Handling
✅ **DO**: Return structured errors
```rust
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, CliError>;
```

❌ **DON'T**: Use string errors or panic
```rust
pub fn execute(args: &TrustArgs) -> Result<(), String> { /* bad */ }
```

#### 4. Dependency Injection
✅ **DO**: Accept dependencies as parameters
```rust
pub async fn execute(args: &TrustArgs, config: &Config) -> Result<()> {
    // Use injected config
}
```

❌ **DON'T**: Hardcode dependencies
```rust
pub async fn execute(args: &TrustArgs) -> Result<()> {
    let config = Config::global(); // Global state!
}
```

---

### Antipatterns

#### ❌ Antipattern 1: Binary in Library

**Problem**: Including a `[[bin]]` section in the library crate.

```toml
# DON'T DO THIS in library Cargo.toml
[[bin]]
name = "my-cli"
path = "src/main.rs"
```

**Why**: Defeats the purpose of composability.

**Solution**: Keep library and binary in separate crates.

#### ❌ Antipattern 2: Tight Coupling

**Problem**: CLI commands directly access global state or singletons.

```rust
pub async fn execute(args: &TrustArgs) -> Result<()> {
    let db = Database::global(); // Bad!
    db.query(/* ... */)?;
    Ok(())
}
```

**Why**: Makes testing difficult, prevents dependency injection.

**Solution**: Pass dependencies as parameters or use builder pattern.

#### ❌ Antipattern 3: Side Effects in Constructors

**Problem**: Clap structures with side effects in `default()` or constructors.

```rust
impl Default for TrustArgs {
    fn default() -> Self {
        // Bad: reads from filesystem
        let config = std::fs::read_to_string("config.toml").unwrap();
        Self { /* ... */ }
    }
}
```

**Why**: Breaks Clap's parsing, causes unexpected I/O.

**Solution**: Keep Clap structures pure, perform I/O in `execute()`.

---

## Performance Considerations

### Performance Characteristics

**Library Compilation**:
- **Build Time**: Similar to standard library (no binary overhead)
- **Size**: No binary artifact generated
- **Reuse**: Compiled once, used by multiple binaries

**Binary Compilation**:
- **Build Time**: Depends on composed commands (incremental builds help)
- **Size**: Only includes used commands (dead code elimination)
- **Runtime**: No performance difference vs monolithic binary

### Optimization Opportunities

#### 1. Feature Flags for Optional Commands

```toml
[features]
default = ["trust", "keygen"]
trust = []
keygen = []
build = []
all = ["trust", "keygen", "build"]
```

```rust
#[cfg(feature = "trust")]
pub mod trust;

#[cfg(feature = "keygen")]
pub mod keygen;
```

**Benefit**: Binaries only link used commands.

#### 2. Lazy Initialization

```rust
pub async fn execute(args: &TrustArgs) -> Result<()> {
    // Load heavy dependencies only when needed
    let config_manager = ConfigManager::new()?;
    // ... execute
}
```

**Benefit**: Faster startup for simple commands.

#### 3. Parallel Compilation

**Workspace Structure**:
```toml
[workspace]
members = [
    "my-cli",      # Library
    "my-app-full", # Binary 1
    "my-app-lite", # Binary 2
]
```

**Benefit**: Cargo compiles library once, binaries in parallel.

---

## Integration Points

### Dependencies

**Clap Integration**:
- Library: `clap = { version = "4.5", features = ["derive"] }`
- Binary: Same version (ensure compatibility)

**Async Runtime**:
- Library: Depends on tokio for async execute functions
- Binary: Provides `#[tokio::main]` or equivalent

**Error Handling**:
- Library: Exports `CliError` and `Result` types
- Binary: Converts to application-level errors if needed

### Compatibility

**Version Requirements**:
- Clap: Same major version across library and binaries
- Tokio: Same major version (1.x compatible)
- Rust Edition: Same edition (2021 recommended)

**Breaking Changes**:
- Clap structure changes = breaking change (semver major bump)
- Execute signature changes = breaking change
- Error type changes = breaking change (consider non_exhaustive)

### Migration Paths

**Monolithic to Composable**:

1. **Step 1**: Extract commands to library crate
2. **Step 2**: Move business logic to library
3. **Step 3**: Create thin binary that imports library
4. **Step 4**: Remove duplicate code from binary
5. **Step 5**: Add tests in library

**Timeline**: 1-2 days for small CLIs, 1-2 weeks for large CLIs.

---

## Security Considerations

### Security Implications

**Reduced Attack Surface**:
- Library has no `main()`, cannot be executed directly
- Binaries control initialization and environment setup
- Clear separation of concerns

**Dependency Injection Benefits**:
- Easier to inject security policies
- Configuration can be validated before command execution
- Audit trails can be enforced at binary level

### Threat Model

**Library Threats**:
- ⚠️ Malicious dependencies (mitigated by cargo audit)
- ⚠️ Code injection via Clap arguments (mitigated by Clap's parsing)

**Binary Threats**:
- ⚠️ Unauthorized command execution (mitigated by proper authentication)
- ⚠️ Privilege escalation (mitigated by least privilege principle)

**Mitigations**:
- Use `cargo audit` regularly
- Validate all user inputs
- Implement proper authentication at binary level
- Log all command executions for audit trail

### Compliance

**Dependency Compliance**:
- All dependencies must pass `cargo audit`
- Use workspace dependencies for version consistency
- Pin versions for reproducible builds

**Code Compliance**:
- Follow workspace standards (§2.1, §3.2, §4.3, §5.1)
- Microsoft Rust Guidelines (M-DESIGN-FOR-AI, M-EXAMPLES)
- Zero warnings policy (`-D warnings`)

---

## Comparison Table

| Aspect | Binary CLI | Library CLI (Composable) |
|--------|------------|--------------------------|
| **Reusability** | ❌ Low - monolithic binary | ✅ High - library can be reused |
| **Testing** | ⚠️ Process spawning required | ✅ Direct function calls |
| **Composition** | ❌ Cannot combine | ✅ Easy composition |
| **Distribution** | ⚠️ One binary per tool | ✅ Multiple binaries from one library |
| **Build Time** | ⚠️ Rebuild entire binary | ✅ Incremental - library cached |
| **Maintainability** | ❌ Duplicate code | ✅ Single source of truth |
| **Flexibility** | ❌ Fixed command set | ✅ Pick and choose commands |
| **Complexity** | ✅ Simpler (one crate) | ⚠️ More complex (workspace) |
| **Use Case** | Simple, standalone tools | Ecosystems, composable CLIs |

---

## Real-World Examples

### Example 1: clap_cargo

**Library**: `clap_cargo`  
**Purpose**: Reusable Cargo workspace arguments

```rust
use clap_cargo::Workspace;

#[derive(Parser)]
struct Cli {
    #[command(flatten)]
    workspace: Workspace,
}
```

**Benefit**: Multiple tools reuse cargo workspace argument parsing.

### Example 2: clap_verbosity_flag

**Library**: `clap_verbosity_flag`  
**Purpose**: Reusable verbosity flag (-v, -vv, -vvv)

```rust
use clap_verbosity_flag::Verbosity;

#[derive(Parser)]
struct Cli {
    #[command(flatten)]
    verbose: Verbosity,
}
```

**Benefit**: Consistent verbosity handling across tools.

### Example 3: AirsSys Ecosystem

**Structure**:
```
airssys-wasm-cli/     # Library (all commands)
├── commands/
│   ├── trust.rs
│   ├── keygen.rs
│   └── ...

airsstack/            # Binary (full CLI)
├── Cargo.toml
└── src/main.rs       # Composes airssys-wasm-cli

airsstack-lite/       # Binary (minimal CLI)
├── Cargo.toml
└── src/main.rs       # Composes subset of commands
```

**Benefit**: Single CLI library, multiple distribution options.

---

## Implementation Checklist

### Library Implementation
- [ ] Create library crate (NO `[[bin]]` section)
- [ ] Define Clap structures (`Args`, `Subcommand`)
- [ ] Implement `execute()` functions (async recommended)
- [ ] Export from `lib.rs`
- [ ] Add rustdoc comments
- [ ] Write unit tests
- [ ] Add integration tests (optional)
- [ ] Verify `cargo doc` works

### Binary Implementation
- [ ] Create binary crate with `[[bin]]` section
- [ ] Import library command structures
- [ ] Define top-level `Commands` enum
- [ ] Implement `main()` with command routing
- [ ] Add binary-specific initialization (logging, config, etc.)
- [ ] Write CLI integration tests (assert_cmd)
- [ ] Generate shell completions
- [ ] Document usage in README

### Quality Assurance
- [ ] `cargo check` passes
- [ ] `cargo clippy` zero warnings
- [ ] `cargo test` all tests pass
- [ ] `cargo doc --no-deps` zero warnings
- [ ] Examples in docs work
- [ ] README has composition examples

---

## Maintenance

### Review Schedule

**Quarterly Reviews**:
- Check for new Clap features that improve composability
- Review binary usage patterns
- Update documentation with new examples

**On Clap Major Upgrades**:
- Test all composition patterns
- Update examples and documentation
- Check for breaking changes in derive macros

### Update Triggers

**Update Documentation When**:
- New composition pattern discovered
- Clap API changes
- User reports confusion about composition
- New real-world example emerges

**Refactor When**:
- Binaries show significant code duplication
- Common patterns emerge across commands
- Performance issues identified

### Owner/Maintainer

**Primary Contact**: Architecture Team  
**Review Frequency**: Quarterly  
**Last Review**: 2025-12-18

---

## References

### Related Documentation
- **ADR-CLI-001**: Library-Only Architecture Decision
- **TASK-CLI-002**: Trust Command Implementation
- **KNOWLEDGE-CLI-001**: CLI Implementation Foundation

### External References
- [Clap Documentation](https://docs.rs/clap)
- [clap_cargo](https://crates.io/crates/clap_cargo) - Real-world example
- [clap_verbosity_flag](https://crates.io/crates/clap_verbosity_flag) - Real-world example

### Workspace Standards
- **§2.1**: 3-Layer Import Organization
- **§4.3**: Module Architecture (mod.rs only re-exports)
- **§5.1**: Dependency Management
- **§6.1**: YAGNI Principles

---

## History

### Version History
- **2025-12-18**: v1.0 - Initial documentation, stable pattern

### Review History
- **2025-12-18**: Created and reviewed by Architecture Team - Approved

---

**Template Version:** 1.0  
**Last Updated:** 2025-12-18
