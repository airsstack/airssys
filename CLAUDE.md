# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

```bash
# Build
cargo build --workspace
cargo build --package airssys-wasm   # Single crate

# Test
cargo test --workspace               # All tests
cargo test --package airssys-wasm    # Single crate
cargo test --lib                     # Unit tests only
cargo test --test '*'                # Integration tests only
cargo test test_name                 # Single test by name

# Quality checks (MUST pass before any code is complete)
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check

# Benchmarks (airssys-rt)
cargo bench --package airssys-rt
```

## Architecture Overview

AirsSys is a Rust workspace with four main crates providing system programming components for the AirsStack ecosystem:

### Crate Dependency Graph

```
airssys-wasm (WASM plugin platform)
    ├── airssys-rt (actor runtime)
    └── airssys-osl (OS abstractions)
            └── airssys-osl-macros (proc macros, optional)
```

### airssys-osl (OS Layer Framework)
Cross-platform OS abstractions (filesystem, process, network) with built-in ACL/RBAC security and audit logging. Use helper functions for one-line operations:
```rust
use airssys_osl::helpers::*;
write_file("/path", data, "user").await?;
```

### airssys-rt (Actor Runtime)
Erlang-inspired actor system with supervision trees. Performance: ~625ns spawn, 4.7M msgs/sec.
- `Actor` trait with async `handle_message`
- `MessageBroker` for routing
- Supervisors with OneForOne/OneForAll/RestForOne strategies

### airssys-wasm (WASM Plugin Platform)
WebAssembly component framework where **each WASM component = one actor**. Uses deny-by-default capability-based security.

**Six-layer architecture with strict import rules (ADR-WASM-023):**
```
system/ → messaging/ → component/ → runtime/ → security/ → core/
```
Dependencies flow right-to-left only. Verify with:
```bash
# Must return nothing
grep -rn "use crate::actor" airssys-wasm/src/runtime/
grep -rn "use crate::runtime\|use crate::actor" airssys-wasm/src/security/
```

## Mandatory Patterns

### Import Organization (3 layers, always)
```rust
// Layer 1: std
use std::collections::HashMap;

// Layer 2: External crates
use serde::{Deserialize, Serialize};

// Layer 3: Internal
use crate::core::config::ComponentConfig;
```

### No FQN in Type Annotations
```rust
// Correct
use std::path::PathBuf;
fn process(path: PathBuf) -> Result<PathBuf, Error>

// Forbidden
fn process(path: std::path::PathBuf) -> ...
```

### Time Handling
Always use `chrono::DateTime<Utc>`. Never use `std::time::SystemTime` for business logic.

### mod.rs Files
Only module declarations and submodule re-exports. No type re-exports, no glob re-exports, no implementation code.

### Error Handling
Use `thiserror` for error types. Implement `From` traits for conversion.

## Clippy Configuration

The workspace denies `unwrap_used`, `expect_used`, and `panic`. Use proper error handling.

## Key Directories

- `.memory-bank/` - Multi-project memory bank (see Memory Bank section below)
- `.aiassisted/instructions/` - AI assistant instructions and guidelines
- `site-mkdocs/` - Documentation site (`mkdocs serve` for local preview)
- `airssys-wasm/wit/` - WebAssembly Interface Type definitions

## Memory Bank System

**CRITICAL**: AI assistant memory resets between sessions. You MUST rely on the Memory Bank to understand and continue work.

### What is the Memory Bank?

The Memory Bank is a structured documentation system that maintains project context, decisions, and progress across sessions. It supports multiple sub-projects with workspace-level shared context.

**Full instructions**: `.aiassisted/instructions/multi-project-memory-bank.instructions.md`

### At Session Start - MUST Read

1. `.memory-bank/current-context.md` - Active sub-project
2. `.memory-bank/workspace/` - Shared patterns, architecture, standards
3. `.memory-bank/sub-projects/<active>/` - Sub-project specific context

### Memory Bank Structure

```
.memory-bank/
├── current-context.md           # Active sub-project tracker
├── workspace/                   # Shared across all sub-projects
│   ├── project-brief.md         # Workspace vision & objectives
│   ├── shared-patterns.md       # Core patterns & standards
│   ├── workspace-architecture.md
│   └── workspace-progress.md
├── templates/docs/              # Documentation templates
└── sub-projects/<name>/         # Per sub-project context
    ├── project-brief.md         # Sub-project goals & scope
    ├── product-context.md       # Why it exists, problems solved
    ├── active-context.md        # Current focus & next steps
    ├── system-patterns.md       # Architecture & patterns
    ├── tech-context.md          # Tech stack & constraints
    ├── progress.md              # Status & known issues
    ├── tasks/                   # Task tracking
    │   ├── _index.md            # Task registry
    │   └── <task-id>/           # Individual task files
    └── docs/                    # Technical documentation
        ├── debts/               # Technical debt records
        ├── knowledges/          # Architecture & patterns docs
        └── adr/                 # Architecture Decision Records
```

### Key Commands

- `show-memory-bank [sub-project]` - Display memory bank state
- `update-memory-bank [sub-project]` - Review and update all files
- `switch-context [sub-project]` - Change active sub-project
- `show-tasks [sub-project]` - Display tasks
- `add-task [sub-project] [name]` - Create new task
- `save-context [description]` - Save context snapshot

### Task Rules (MANDATORY)

- **Single action per task** - Each task does ONE thing
- **Tests included** - Module creation includes tests in same task
- **Plans reference docs** - Every plan references relevant ADRs and Knowledge docs
- **Both test types required** - Unit tests (src/) AND integration tests (tests/)

### Documentation Triggers

Create documentation when:
- **Technical Debt**: Any `TODO(DEBT)` comments, shortcuts, or compromises
- **Knowledge Docs**: New patterns, non-obvious algorithms, integrations, security paths
- **ADRs**: Technology selections, architectural choices, significant trade-offs
