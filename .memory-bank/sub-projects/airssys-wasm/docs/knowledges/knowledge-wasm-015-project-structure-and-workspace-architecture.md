# KNOWLEDGE-WASM-015: Project Structure and Workspace Architecture

**Document ID:** KNOWLEDGE-WASM-015  
**Created:** 2025-11-30  
**Updated:** 2025-11-30  
**Category:** Core Architecture  
**Complexity:** High  
**Dependencies:** Workspace Standards, Cargo Workspace

## Overview

This document provides a comprehensive overview of the **airssys-wasm ecosystem structure**, documenting the relationship between the three sub-projects (airssys-wasm, airssys-wasm-component, airssys-wasm-cli) and their associated implementation tasks. This serves as the authoritative reference for understanding how the crates work together as a unified framework.

## Purpose

**Why This Documentation Exists:**
- Prevent confusion between the three separate but related crates
- Map implementation tasks (WASM-TASK-XXX) to their corresponding crates
- Document dependency relationships and integration points
- Provide clear guidance for developers working on specific tasks
- Ensure task descriptions reference the correct sub-project

**Key Questions This Document Answers:**
1. What are the three sub-projects and their purposes?
2. How do they relate to each other in the workspace?
3. Which WASM-TASK implements which crate?
4. What is the current implementation status of each sub-project?
5. How do the sub-projects integrate at runtime?

---

## Workspace Structure Overview

### The Three Sub-Projects

The airssys-wasm ecosystem consists of **three distinct Cargo crates** within the AirsSys workspace:

```
airssys/                                    # Workspace root
├── Cargo.toml                             # Workspace configuration
├── airssys-wasm/                          # 🏛️ Core Framework Library
│   ├── Cargo.toml                         # Core library crate
│   ├── src/                               # Implementation (~20,000+ lines planned)
│   ├── wit/                               # WIT interface definitions
│   ├── docs/                              # mdBook documentation
│   └── tests/                             # Integration tests
│
├── airssys-wasm-component/                # 🎨 Procedural Macro Crate
│   ├── Cargo.toml                         # Proc-macro crate
│   ├── src/                               # Macro implementation (~2,000+ lines planned)
│   └── tests/                             # UI tests for macros
│
└── airssys-wasm-cli/                      # 🛠️ Command-Line Tool
    ├── Cargo.toml                         # Binary crate
    ├── src/                               # CLI implementation (~5,000+ lines planned)
    │   ├── main.rs                        # Entry point
    │   └── commands/                      # 14 command modules
    └── tests/                             # CLI integration tests
```

### Workspace Configuration

```toml
# Root Cargo.toml
[workspace]
members = [
    "airssys-osl",
    "airssys-rt",
    "airssys-wasm",              # Core framework library
    "airssys-wasm-component",    # Procedural macros
    "airssys-wasm-cli",          # CLI tool
]
resolver = "2"

[workspace.dependencies]
# AirsSys Foundation Crates (MUST be at top)
airssys-osl = { path = "airssys-osl" }
airssys-rt = { path = "airssys-rt" }
airssys-wasm = { path = "airssys-wasm" }
airssys-wasm-component = { path = "airssys-wasm-component" }
# ... other dependencies
```

---

## Sub-Project 1: airssys-wasm (Core Framework Library)

### Purpose

The **core WASM component framework library** providing the foundational runtime, security, and component management infrastructure.

### Responsibilities

- **Component Runtime**: Wasmtime-based WASM execution engine
- **Component Lifecycle**: Loading, instantiation, execution, shutdown
- **Security System**: Capability-based security and sandboxing
- **Storage System**: Persistent component state management
- **Messaging System**: Inter-component communication
- **Actor Integration**: ComponentActor integration with airssys-rt
- **Deployment Engine**: Runtime component deployment and versioning
- **Monitoring**: Observability and metrics collection

### Crate Type

```toml
[package]
name = "airssys-wasm"
version = "0.1.0"

[lib]
# Library crate (not binary, not proc-macro)
```

### Primary Dependencies

```toml
[dependencies]
# AirsSys ecosystem
airssys-osl = { workspace = true }          # System access layer
airssys-rt = { workspace = true }           # Actor runtime

# WASM runtime
wasmtime = { version = "24.0" }
wasmtime-wasi = { version = "24.0" }
wit-bindgen = { version = "0.30" }

# Storage backends
sled = { version = "0.34" }                 # Default backend
rocksdb = { version = "0.22", optional = true }  # Optional backend

# Core runtime
tokio = { workspace = true }
async-trait = { version = "0.1" }

# Serialization
serde = { workspace = true }
borsh = { version = "1.5" }                 # Cross-language messaging
bincode = { version = "1.3" }               # Internal storage
```

### Current Status

- **Overall Progress**: 95% of Layer 1 (Foundation) complete
- **Completed**:
  - ✅ WASM-TASK-000: Core Abstractions (100%)
  - ✅ WASM-TASK-002: WASM Runtime Layer (100%)
  - ✅ WASM-TASK-003: WIT Interface System (95% - docs pending)
- **Next**: WASM-TASK-004: Actor System Integration (Block 3)

### Implementation Tasks

The following tasks implement this crate:

| Task | Block | Description | Status |
|------|-------|-------------|--------|
| WASM-TASK-000 | Layer 0 | Core Abstractions Design | ✅ 100% |
| WASM-TASK-002 | Block 1 | WASM Runtime Layer | ✅ 100% |
| WASM-TASK-003 | Block 2 | WIT Interface System | ✅ 95% |
| WASM-TASK-004 | Block 3 | Actor System Integration | ⏳ 0% (Next) |
| WASM-TASK-005 | Block 4 | Security & Isolation | ⏸️ Blocked |
| WASM-TASK-006 | Block 5 | Inter-Component Communication | ⏸️ Blocked |
| WASM-TASK-007 | Block 6 | Persistent Storage System | ⏸️ Blocked |
| WASM-TASK-008 | Block 7 | Component Lifecycle System | ⏸️ Blocked |
| WASM-TASK-009 | Block 8 | AirsSys-OSL Bridge | ⏸️ Blocked |
| WASM-TASK-010 | Block 9 | Monitoring & Observability | ⏸️ Blocked |

### Public API Surface

```rust
// Core library exports (simplified)
pub mod core {
    pub mod component;      // Component trait and types
    pub mod runtime;        // WASM runtime management
    pub mod capability;     // Security capabilities
    pub mod storage;        // Persistent storage
    pub mod messaging;      // Inter-component messaging
    pub mod lifecycle;      // Component lifecycle
    pub mod actor;          // Actor integration
}

pub mod prelude {
    // Common imports for component developers
}

// Main entry point for host applications
pub struct ComponentRuntime { /* ... */ }
```

### Usage by Other Sub-Projects

```rust
// Used by airssys-wasm-component (macros)
use airssys_wasm::core::component::Component;
use airssys_wasm::core::error::ComponentError;

// Used by airssys-wasm-cli (CLI tool)
use airssys_wasm::ComponentRuntime;
use airssys_wasm::core::lifecycle::InstallOptions;
```

---

## Sub-Project 2: airssys-wasm-component (Procedural Macro Crate)

### Purpose

**Procedural macros** that eliminate boilerplate for component development, following the **serde pattern** (separation of macros from core types).

### Responsibilities

- **#[component] Macro**: Main component attribute macro
- **#[derive(ComponentOperation)]**: Operation message types
- **#[derive(ComponentResult)]**: Result message types
- **#[derive(ComponentConfig)]**: Configuration types
- **Code Generation**: Automatic `extern "C"` function generation
- **Memory Management**: Allocation/deallocation helpers
- **Serialization Integration**: Multicodec integration

### Crate Type

```toml
[package]
name = "airssys-wasm-component"
version = "0.1.0"

[lib]
proc-macro = true  # Procedural macro crate
```

### Architecture Pattern: Serde Pattern

This crate follows the proven **serde pattern** used by `serde` and `serde_derive`:

```
Core Types          Procedural Macros
(airssys-wasm)  ←   (airssys-wasm-component)
     │                      │
     │                      │
     └──────────────────────┘
            │
         Component
       (user code)
```

**Benefits**:
- **Optional Dependency**: Core types available without macro overhead
- **Faster Builds**: Separate macro compilation
- **Flexible Usage**: Manual implementation remains possible
- **Industry Standard**: Proven architecture pattern

### Primary Dependencies

```toml
[dependencies]
# Macro development
proc-macro2 = "1.0"
quote = "1.0"
syn = { version = "2.0", features = ["full", "extra-traits"] }

# Minimal core types dependency
airssys-wasm = { workspace = true, default-features = false }

# Utilities
uuid = { version = "1.0", features = ["v4"] }
```

### Current Status

- **Overall Progress**: 25% (Foundation Complete)
- **Phase 1**: ✅ Project setup and architecture foundation COMPLETE
- **Phase 2**: ⏳ Actual macro logic implementation (Ready to begin)
- **Foundation**: All files compile, structure complete, placeholders ready

### Implementation Tasks

| Task | Block | Description | Status |
|------|-------|-------------|--------|
| WASM-TASK-011 | Block 10 | Component Development SDK | ⏸️ 25% (Foundation) |

**Task Details**:
- **Location**: `airssys-wasm-component/` directory
- **Effort**: 5-6 weeks for full implementation
- **Dependencies**: Core `airssys-wasm` trait definitions
- **Blockers**: None (can proceed independently of Blocks 3-9)

### Example Usage (Planned)

```rust
use airssys_wasm::{Component, ComponentError};
use airssys_wasm_component::{component, ComponentOperation, ComponentResult};

#[derive(ComponentConfig)]
pub struct MyConfig {
    pub setting: String,
}

#[derive(ComponentOperation)]
pub enum MyOperation {
    Process { data: String },
    GetStatus,
}

#[derive(ComponentResult)]
pub enum MyResult {
    Processed { result: String },
    Status { active: bool },
}

#[component(name = "my-processor", version = "1.0.0")]
pub struct MyProcessor {
    processed_count: usize,
}

impl Component for MyProcessor {
    type Config = MyConfig;
    type Operation = MyOperation;
    type Result = MyResult;
    
    fn init(&mut self, config: Self::Config) -> Result<(), ComponentError> {
        // Initialization logic
        Ok(())
    }
    
    fn execute(&mut self, operation: Self::Operation) -> Result<Self::Result, ComponentError> {
        match operation {
            MyOperation::Process { data } => {
                self.processed_count += 1;
                Ok(MyResult::Processed { 
                    result: format!("Processed: {}", data) 
                })
            }
            MyOperation::GetStatus => {
                Ok(MyResult::Status { 
                    active: self.processed_count > 0 
                })
            }
        }
    }
}
```

### Generated Code (What #[component] Produces)

The `#[component]` macro automatically generates:
- All required `extern "C"` functions for WASM compatibility
- Memory management (allocation/deallocation)
- Multicodec serialization/deserialization
- Error handling and encoding
- Component lifecycle hooks
- Static instance management

This eliminates ~200+ lines of boilerplate per component.

---

## Sub-Project 3: airssys-wasm-cli (Command-Line Tool)

### Purpose

**Command-line interface** for complete component lifecycle management, providing developer-friendly tooling for building, signing, installing, and managing WASM components.

### Responsibilities

- **Cryptographic Operations**: Ed25519 key generation and signing
- **Project Management**: Component project initialization
- **Build Pipeline**: Component compilation and packaging
- **Installation**: Multi-source installation (Git/Local/URL)
- **Lifecycle Management**: Install, update, uninstall operations
- **Inspection**: Component listing, info, status
- **Debugging**: Log viewing and streaming
- **Configuration**: Tool configuration management
- **Shell Integration**: Completion generation

### Crate Type

```toml
[package]
name = "airssys-wasm-cli"
version = "0.1.0"

[[bin]]
name = "airssys-wasm"  # Binary name
path = "src/main.rs"
```

### 14 Core Commands

```bash
# Cryptographic Security
airssys-wasm keygen        # Generate Ed25519 keypair

# Component Development
airssys-wasm init          # Initialize new component project
airssys-wasm build         # Build WASM component
airssys-wasm sign          # Sign component with private key

# Component Management
airssys-wasm install       # Install from Git/file/URL
airssys-wasm update        # Update installed component
airssys-wasm uninstall     # Remove component

# Discovery & Inspection
airssys-wasm list          # List installed components
airssys-wasm info          # Show component metadata
airssys-wasm status        # Check component health

# Operations & Debugging
airssys-wasm logs          # View component logs
airssys-wasm verify        # Verify signature

# Configuration
airssys-wasm config        # Manage configuration
airssys-wasm completions   # Generate shell completions
```

### Primary Dependencies

```toml
[dependencies]
# Core framework
airssys-wasm = { workspace = true }

# CLI framework
clap = { workspace = true }
clap_complete = { workspace = true }

# Cryptography
ed25519-dalek = { workspace = true }
rand = { workspace = true }
base64 = { workspace = true }

# Git operations
git2 = { workspace = true }

# HTTP client
reqwest = { workspace = true }

# Terminal UX
colored = { workspace = true }
indicatif = { workspace = true }
console = { workspace = true }
dialoguer = { workspace = true }

# Serialization
serde = { workspace = true }
serde_json = { workspace = true }
toml = { workspace = true }
```

### Current Status

- **Overall Progress**: 10% (Foundation Complete)
- **Structure**: All 14 command modules created with stubs
- **Ready For**: Full implementation after Layer 2 (Blocks 4-7)
- **Foundation**: Compiles successfully, command structure defined

### Implementation Tasks

| Task | Block | Description | Status |
|------|-------|-------------|--------|
| WASM-TASK-012 | Block 11 | CLI Tool | ⏸️ 10% (Foundation) |

**Task Details**:
- **Location**: `airssys-wasm-cli/` directory
- **Effort**: 4-5 weeks for full implementation
- **Dependencies**: Layer 2 complete (Blocks 4-7) for full functionality
- **Blockers**: Requires Security (Block 4), Lifecycle (Block 7), Storage (Block 6)

### Command Structure

```rust
// airssys-wasm-cli/src/commands/mod.rs
pub mod keygen;        // Ed25519 key generation
pub mod init;          // Project initialization
pub mod build;         // Component building
pub mod sign;          // Component signing
pub mod install;       // Component installation
pub mod update;        // Component updates
pub mod uninstall;     // Component removal
pub mod list;          // List components
pub mod info;          // Component metadata
pub mod status;        // Runtime status
pub mod logs;          // Log viewing
pub mod verify;        // Signature verification
pub mod config;        // Configuration management
pub mod completions;   // Shell completions
```

### Usage Examples

```bash
# Generate keypair for signing
airssys-wasm keygen

# Initialize new component project
airssys-wasm init my-component --description "My awesome component"
cd my-component

# Build component
airssys-wasm build --release

# Sign component
airssys-wasm sign target/wasm32-wasi/release/my-component.wasm

# Install component from Git
airssys-wasm install git@github.com:example/component.git

# Install from local file
airssys-wasm install ./my-component.wasm

# Install from URL
airssys-wasm install https://example.com/components/my-component.wasm

# List installed components
airssys-wasm list --detailed

# View component info
airssys-wasm info my-component

# Check component status
airssys-wasm status my-component

# View logs
airssys-wasm logs my-component --follow

# Update component
airssys-wasm update my-component --version 1.2.0

# Uninstall component
airssys-wasm uninstall my-component

# Verify signature
airssys-wasm verify ./my-component.wasm --public-key <key>

# Generate shell completions
airssys-wasm completions bash > /etc/bash_completion.d/airssys-wasm
```

---

## Dependency Relationships

### Crate Dependency Graph

```
                    ┌─────────────────┐
                    │  airssys-wasm   │
                    │  (Core Library) │
                    │   95% complete  │
                    └────────┬────────┘
                             │
                ┌────────────┴────────────┐
                │                         │
                ▼                         ▼
    ┌──────────────────────┐  ┌──────────────────────┐
    │ airssys-wasm-       │  │  airssys-wasm-cli    │
    │   component         │  │   (CLI Tool)         │
    │ (Proc Macros)       │  │   10% complete       │
    │  25% complete       │  │                      │
    └──────────────────────┘  └──────────────────────┘
```

**Dependency Rules**:
1. **airssys-wasm** (core) depends on: `airssys-osl`, `airssys-rt`, `wasmtime`
2. **airssys-wasm-component** depends on: `airssys-wasm` (minimal, core types only)
3. **airssys-wasm-cli** depends on: `airssys-wasm` (full library)

### Integration Flow

```rust
// Component Developer Workflow
┌─────────────────────────────────────────────────────┐
│ 1. Write component using airssys-wasm-component    │
│    #[component] macro                               │
└──────────────────┬──────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────┐
│ 2. Build component using airssys-wasm-cli          │
│    $ airssys-wasm build --release                  │
└──────────────────┬──────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────┐
│ 3. Sign component using airssys-wasm-cli           │
│    $ airssys-wasm sign component.wasm              │
└──────────────────┬──────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────┐
│ 4. Install component using airssys-wasm-cli        │
│    $ airssys-wasm install component.wasm           │
└──────────────────┬──────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────┐
│ 5. Component runs in airssys-wasm runtime          │
│    (Core library manages execution)                 │
└─────────────────────────────────────────────────────┘
```

---

## Task-to-Crate Mapping

### Complete Task Breakdown

| Task ID | Block | Crate | Description | Status |
|---------|-------|-------|-------------|--------|
| **WASM-TASK-000** | Layer 0 | airssys-wasm | Core Abstractions | ✅ 100% |
| **WASM-TASK-001** | Planning | Documentation | Implementation Roadmap | ✅ 100% |
| **WASM-TASK-002** | Block 1 | airssys-wasm | WASM Runtime Layer | ✅ 100% |
| **WASM-TASK-003** | Block 2 | airssys-wasm | WIT Interface System | ✅ 95% |
| **WASM-TASK-004** | Block 3 | airssys-wasm | Actor System Integration | ⏳ 0% (Next) |
| **WASM-TASK-005** | Block 4 | airssys-wasm | Security & Isolation | ⏸️ Blocked |
| **WASM-TASK-006** | Block 5 | airssys-wasm | Inter-Component Communication | ⏸️ Blocked |
| **WASM-TASK-007** | Block 6 | airssys-wasm | Persistent Storage | ⏸️ Blocked |
| **WASM-TASK-008** | Block 7 | airssys-wasm | Component Lifecycle | ⏸️ Blocked |
| **WASM-TASK-009** | Block 8 | airssys-wasm | AirsSys-OSL Bridge | ⏸️ Blocked |
| **WASM-TASK-010** | Block 9 | airssys-wasm | Monitoring & Observability | ⏸️ Blocked |
| **WASM-TASK-011** | Block 10 | airssys-wasm-component | Component Development SDK | ⏸️ 25% |
| **WASM-TASK-012** | Block 11 | airssys-wasm-cli | CLI Tool | ⏸️ 10% |

### Layer Classification

**Layer 1 (Foundation) - 95% Complete**:
- WASM-TASK-000, 002, 003 (airssys-wasm core) ✅
- WASM-TASK-004 (Actor Integration) ⏳ **NEXT**

**Layer 2 (Core Services) - Blocked by Block 3**:
- WASM-TASK-005, 006, 007, 008 (airssys-wasm features)

**Layer 3 (Integration) - Blocked by Layer 2**:
- WASM-TASK-009, 010 (airssys-wasm integration)

**Layer 4 (Developer Experience) - Independent/Blocked**:
- WASM-TASK-011 (airssys-wasm-component) - **Can proceed independently**
- WASM-TASK-012 (airssys-wasm-cli) - Requires Layer 2

---

## Runtime Integration Architecture

### How the Three Sub-Projects Work Together at Runtime

```
┌─────────────────────────────────────────────────────────────────┐
│                      Host Application                            │
│                                                                   │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │         airssys-wasm (Core Runtime)                         │ │
│  │                                                              │ │
│  │  ┌──────────────────────────────────────────────────────┐  │ │
│  │  │  ComponentRuntime                                     │  │ │
│  │  │  - Load components                                    │  │ │
│  │  │  - Execute components                                 │  │ │
│  │  │  - Manage security                                    │  │ │
│  │  │  - Handle messaging                                   │  │ │
│  │  │  - Integrate with airssys-rt actors                   │  │ │
│  │  └──────────────────────────────────────────────────────┘  │ │
│  │                                                              │ │
│  │  Loads and executes ──────────────────────────────────────┐ │ │
│  └────────────────────────────────────────────────────────│───┘ │
└─────────────────────────────────────────────────────────│───────┘
                                                           │
                                                           ▼
                           ┌───────────────────────────────────────┐
                           │     WASM Component                    │
                           │                                       │
                           │  Built with:                          │
                           │  - airssys-wasm-component macros      │
                           │  - airssys-wasm-cli build tool        │
                           │                                       │
                           │  #[component]                         │
                           │  struct MyComponent { ... }           │
                           │                                       │
                           │  impl Component for MyComponent {     │
                           │      fn execute(...) { ... }          │
                           │  }                                    │
                           └───────────────────────────────────────┘
```

### Development-Time vs Runtime

**Development Time** (Developer Machine):
1. Developer writes component using `airssys-wasm-component` macros
2. Developer uses `airssys-wasm-cli` to build, sign, and test
3. Component is packaged as `.wasm` file

**Runtime** (Production/Host System):
1. Host application uses `airssys-wasm` to load component
2. Component executes in `airssys-wasm` runtime
3. Security enforced by `airssys-wasm` capability system
4. Messaging handled by `airssys-wasm` + `airssys-rt` integration

---

## Implementation Timeline

### Current State (November 2025)

```
airssys-wasm              ████████████████████░░░░░  95% (Layer 1)
airssys-wasm-component    █████░░░░░░░░░░░░░░░░░░░  25% (Foundation)
airssys-wasm-cli          ██░░░░░░░░░░░░░░░░░░░░░░  10% (Foundation)
```

### Roadmap

**Phase 1: Block 3 - Actor Integration (Weeks 1-5)** ⭐ **CURRENT**
- Complete WASM-TASK-004 (airssys-wasm)
- Unblocks entire Layer 2

**Phase 2: Layer 2 - Core Services (Weeks 6-17)**
- WASM-TASK-005, 006, 007, 008 (airssys-wasm)
- Can parallelize these 4 tasks

**Phase 3: Layer 3 - Integration (Weeks 18-26)**
- WASM-TASK-009, 010 (airssys-wasm)

**Phase 4: Layer 4 - Developer Tools (Weeks 27-37)**
- WASM-TASK-011 (airssys-wasm-component) - 5-6 weeks
- WASM-TASK-012 (airssys-wasm-cli) - 4-5 weeks

**Note**: WASM-TASK-011 (macros) could potentially be worked on earlier in parallel with Layer 2, as it has minimal dependencies.

---

## Developer Guidance

### Working on airssys-wasm (Core Library)

**Location**: `airssys-wasm/src/`

**When to modify**:
- Implementing Blocks 1-9 (WASM-TASK-002 through WASM-TASK-010)
- Adding core runtime features
- Implementing integration with airssys-osl or airssys-rt
- Security, storage, messaging, lifecycle features

**Key files**:
```
src/core/
├── component.rs       # Component trait and types
├── runtime.rs         # WASM runtime management
├── actor.rs           # Actor system integration
├── capability.rs      # Security capabilities
├── storage.rs         # Persistent storage
├── messaging.rs       # Inter-component messaging
├── lifecycle.rs       # Component lifecycle
└── ...
```

### Working on airssys-wasm-component (Macros)

**Location**: `airssys-wasm-component/src/`

**When to modify**:
- Implementing WASM-TASK-011 (Block 10)
- Adding procedural macros for component development
- Code generation features
- Derive macro implementations

**Key files**:
```
src/
├── lib.rs             # Macro exports
├── component.rs       # #[component] macro
├── derive.rs          # Derive macros
├── codegen.rs         # Code generation utilities
└── utils.rs           # Helper functions
```

**Testing approach**:
- Use `trybuild` for UI tests (compile-time macro behavior)
- Test generated code correctness
- Validate error messages

### Working on airssys-wasm-cli (CLI Tool)

**Location**: `airssys-wasm-cli/src/`

**When to modify**:
- Implementing WASM-TASK-012 (Block 11)
- Adding new CLI commands
- Improving developer experience
- Integration with build tools

**Key files**:
```
src/
├── main.rs            # CLI entry point
├── commands/          # 14 command modules
│   ├── keygen.rs
│   ├── init.rs
│   ├── build.rs
│   ├── sign.rs
│   ├── install.rs
│   └── ...
├── cli_config.rs      # Configuration management
├── error.rs           # Error types
└── utils.rs           # Shared utilities
```

**Testing approach**:
- Use `assert_cmd` for CLI integration tests
- Test command outputs and exit codes
- Validate file operations and Git integration

---

## Common Pitfalls and Clarifications

### ❌ Common Mistakes

**Mistake 1**: Confusing the three crates
- ❌ Adding macro code to `airssys-wasm/src/`
- ✅ Macro code belongs in `airssys-wasm-component/src/`

**Mistake 2**: Wrong dependency direction
- ❌ `airssys-wasm` depending on `airssys-wasm-component`
- ✅ `airssys-wasm-component` depends on `airssys-wasm` (core types only)

**Mistake 3**: Assuming single crate
- ❌ "airssys-wasm includes CLI and macros"
- ✅ Three separate crates with distinct responsibilities

**Mistake 4**: Wrong task assignment
- ❌ Implementing CLI commands in WASM-TASK-004
- ✅ CLI commands are WASM-TASK-012 only

### ✅ Key Clarifications

**Q: Why three separate crates?**
- **A**: Follows Rust best practices (serde pattern), enables optional dependencies, faster compilation

**Q: Can I work on airssys-wasm-component before Layer 2 is complete?**
- **A**: Yes! The macro crate has minimal dependencies and can proceed independently

**Q: Why does airssys-wasm-cli need Layer 2 complete?**
- **A**: Commands like `install`, `update`, `status` require Layer 2 features (lifecycle, storage, security)

**Q: Where do WIT interfaces live?**
- **A**: In `airssys-wasm/wit/` directory (part of core library)

**Q: Where is Component.toml parsing implemented?**
- **A**: In `airssys-wasm/src/core/manifest.rs` (used by both core and CLI)

---

## Cross-References

### Related Memory Bank Documents

**Knowledge Documents**:
- KNOWLEDGE-WASM-001: Component Framework Architecture
- KNOWLEDGE-WASM-002: High-Level Overview
- KNOWLEDGE-WASM-010: CLI Tool Specification (airssys-wasm-cli)
- KNOWLEDGE-WASM-012: Module Structure Architecture (airssys-wasm)

**Task Documents**:
- task_011_block_10_component_development_sdk.md (airssys-wasm-component)
- task_012_block_11_cli_tool.md (airssys-wasm-cli)
- All other task_XXX files (airssys-wasm)

**ADRs**:
- ADR-WASM-XXX: All ADRs apply to airssys-wasm core architecture

**Technical Debt**:
- DEBT-WASM-003: Component Model v0.1 limitations (affects airssys-wasm)

### Related Workspace Standards

- §2.1: 3-Layer Import Organization (applies to all three crates)
- §4.3: Module Architecture (airssys-wasm module structure)
- §5.1: Dependency Management (workspace dependencies)
- §6.1: YAGNI Principles (all crates)
- §6.3: Microsoft Rust Guidelines (all crates)

---

## Summary

### Key Takeaways

1. **Three Distinct Crates**:
   - `airssys-wasm`: Core runtime library (95% complete)
   - `airssys-wasm-component`: Procedural macros (25% complete)
   - `airssys-wasm-cli`: CLI tool (10% complete)

2. **Task Distribution**:
   - Blocks 1-9 (WASM-TASK-002 to 010) → `airssys-wasm`
   - Block 10 (WASM-TASK-011) → `airssys-wasm-component`
   - Block 11 (WASM-TASK-012) → `airssys-wasm-cli`

3. **Dependency Flow**:
   - CLI depends on Core Library
   - Macros depend on Core Library (types only)
   - Core Library is foundation for both

4. **Current Focus**: Block 3 (Actor Integration) in `airssys-wasm`

5. **Parallel Opportunities**: 
   - WASM-TASK-011 (macros) can proceed independently
   - Documentation work is non-blocking

---

**Document Status**: Complete and authoritative  
**Next Review**: After Block 3 completion  
**Maintainer**: Update when adding new sub-projects or restructuring crates
