# airssys-osl-macros Project Setup Summary

**Created:** 2025-10-08  
**Status:** Foundation Complete - Workspace Setup Pending  
**Next Step:** Add to workspace Cargo.toml and create basic crate structure

## What Has Been Created

### Memory Bank Structure ✅
```
.memory-bank/sub_projects/airssys-osl-macros/
├── product_context.md          # Product vision and scope
├── tech_context.md             # Technical architecture details
├── progress.md                 # Progress tracking
├── system_patterns.md          # Implementation patterns
├── docs/
│   ├── adr/
│   │   └── _index.md
│   ├── knowledges/
│   │   └── _index.md
│   └── debts/
│       └── _index.md
└── tasks/
    ├── _index.md
    ├── MACROS-TASK-001-foundation-setup.md     # Current task
    └── MACROS-TASK-002-executor-macro.md       # Next task
```

### Documentation Created ✅

#### Product Context
- Project identity and vision
- Strategic goals (85% code reduction, zero-cost abstractions)
- Target users (app developers, framework integrators, library authors)
- Complete scope definition (#[executor] first, #[operation] future, #[middleware] maybe)
- Integration points with airssys-osl
- Success metrics and constraints

#### Technical Context  
- Proc-macro architecture patterns
- Operation name mapping table (10 operations)
- Method signature requirements and validation
- Code generation patterns with syn + quote
- Error handling strategy
- Testing approach (unit, integration, UI tests)
- Performance characteristics (compile-time only, zero runtime cost)
- Workspace standards compliance

#### System Patterns
- Token stream processing patterns
- Method parsing patterns
- Operation mapping patterns
- Code generation patterns
- Error handling patterns
- Testing patterns (unit, UI with trybuild, integration)
- Documentation patterns
- Performance optimization patterns

#### Task Definitions
- **MACROS-TASK-001**: Foundation setup (4 hours, 50% complete)
- **MACROS-TASK-002**: #[executor] macro implementation (2-3 weeks)
- Task index with dependencies and timeline

### Related Documentation ✅

#### airssys-osl Refactoring
- **OSL-TASK-009**: Remove framework and add helpers (2-3 days)
- **Architecture Refactoring Plan**: Complete strategic overview
- Phase 5 of OSL-TASK-008 marked as abandoned (replaced by new approach)

## What Needs to Be Done

### Step 1: Workspace Integration (30 min)

#### Update /airssys/Cargo.toml
```toml
[workspace]
members = [
    "airssys-osl",
    "airssys-osl-macros",  # ← ADD THIS
    "airssys-rt",
    "airssys-wasm",
    "airssys-wasm-component",
]

[workspace.dependencies]
# Add after Layer 1 comment
airssys-osl-macros = { path = "airssys-osl-macros" }

# Add in appropriate layer (Layer 3 or 4)
syn = { version = "2.0", features = ["full", "visit", "visit-mut"] }
quote = "1.0"
proc-macro2 = "1.0"
```

### Step 2: Create Crate Structure (30 min)

#### Create airssys-osl-macros/Cargo.toml
```toml
[package]
name = "airssys-osl-macros"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
rust-version.workspace = true
description = "Procedural macros for airssys-osl core abstractions"
keywords = ["os", "system", "macros", "proc-macro"]
categories = ["os", "development-tools::procedural-macro-helpers"]

[lib]
proc-macro = true

[dependencies]
syn = { workspace = true }
quote = { workspace = true }
proc-macro2 = { workspace = true }

[dev-dependencies]
airssys-osl = { path = "../airssys-osl" }
async-trait = { workspace = true }
tokio = { workspace = true }
trybuild = "1.0"

[lints]
workspace = true
```

#### Create airssys-osl-macros/src/lib.rs
```rust
//! Procedural macros for airssys-osl core abstractions.
//!
//! This crate provides ergonomic macros to reduce boilerplate when
//! implementing airssys-osl traits.
//!
//! # Available Macros
//!
//! - `#[executor]`: Generate `OSExecutor<O>` trait implementations

// Layer 1: Standard library imports
use proc_macro::TokenStream;

// Layer 2: Third-party imports (none yet)

// Layer 3: Internal imports
mod executor;
mod utils;

/// Generates `OSExecutor<O>` trait implementations from method names.
///
/// See module documentation for detailed usage.
#[proc_macro_attribute]
pub fn executor(_attr: TokenStream, item: TokenStream) -> TokenStream {
    executor::expand(item.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
```

#### Create airssys-osl-macros/src/executor.rs
```rust
//! #[executor] macro implementation

use proc_macro2::TokenStream;
use syn::Result;

pub fn expand(input: TokenStream) -> Result<TokenStream> {
    // Placeholder: Return input unchanged for now
    Ok(input)
}
```

#### Create airssys-osl-macros/src/utils.rs
```rust
//! Shared utilities for macro implementations

use syn::Ident;

/// Maps method names to operation types
pub fn map_method_name_to_operation(name: &Ident) -> Option<(&'static str, &'static str)> {
    match name.to_string().as_str() {
        "file_read" => Some(("FileReadOperation", "filesystem")),
        "file_write" => Some(("FileWriteOperation", "filesystem")),
        "file_delete" => Some(("FileDeleteOperation", "filesystem")),
        "directory_create" => Some(("DirectoryCreateOperation", "filesystem")),
        "process_spawn" => Some(("ProcessSpawnOperation", "process")),
        "process_kill" => Some(("ProcessKillOperation", "process")),
        "process_query" => Some(("ProcessQueryOperation", "process")),
        "tcp_connect" => Some(("TcpConnectOperation", "network")),
        "tcp_listen" => Some(("TcpListenOperation", "network")),
        "udp_bind" => Some(("UdpBindOperation", "network")),
        _ => None,
    }
}
```

#### Create airssys-osl-macros/README.md
```markdown
# airssys-osl-macros

Procedural macros for [airssys-osl](../airssys-osl) core abstractions.

## Overview

This crate provides ergonomic macros to reduce boilerplate when implementing
airssys-osl traits, particularly for custom executor implementations.

## Status

**Foundation Setup** - Basic structure in place, macro implementation pending.

## Planned Macros

- `#[executor]` - Generate OSExecutor<O> trait implementations (In Progress)
- `#[operation]` - Derive Operation trait (Planned)
- `#[middleware]` - Generate Middleware<O> implementations (Maybe)

## Usage (Future)

```rust
use airssys_osl::prelude::*;

#[executor]
impl MyExecutor {
    async fn file_read(&self, op: FileReadOperation, ctx: &ExecutionContext) 
        -> OSResult<ExecutionResult> 
    {
        // Custom implementation
        todo!()
    }
}
```

See [airssys-osl documentation](../airssys-osl) for complete integration details.

## Development

This is a proc-macro crate and must be compiled for the host platform.

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE))
- MIT license ([LICENSE-MIT](../LICENSE-MIT))

at your option.
```

#### Create test directories
```bash
mkdir -p airssys-osl-macros/tests/unit
mkdir -p airssys-osl-macros/tests/ui
touch airssys-osl-macros/tests/integration.rs
```

#### Create placeholder integration test
```rust
// airssys-osl-macros/tests/integration.rs

#[test]
fn test_placeholder() {
    // Placeholder test - real tests in MACROS-TASK-002
    assert!(true);
}
```

### Step 3: Validation (15 min)

```bash
# From workspace root
cargo check --workspace
cargo test --workspace --lib
cargo clippy --workspace --all-targets
cargo doc --workspace --no-deps
```

### Step 4: Git Commit

```bash
git add .
git commit -m "feat(macros): Initialize airssys-osl-macros proc-macro crate

- Add airssys-osl-macros to workspace members
- Configure proc-macro crate structure
- Add dependencies (syn, quote, proc-macro2)
- Create placeholder #[executor] macro
- Setup test infrastructure
- Complete memory bank documentation

Part of architecture refactoring (OSL-TASK-009, MACROS-TASK-001)
Ref: .memory-bank/sub_projects/airssys-osl/docs/architecture-refactoring-plan-2025-10.md"
```

## Summary of Complete Plan

### New Architecture (3 Usage Levels)
1. **Low-level**: Direct use of core abstractions
2. **Helpers**: One-line convenience functions (10 helpers in OSL-TASK-009)
3. **Macros**: #[executor] for custom executors (~85% code reduction)

### Tasks Created
- **MACROS-TASK-001**: Foundation Setup (In Progress, 50%)
- **MACROS-TASK-002**: Implement #[executor] Macro (Pending)
- **OSL-TASK-009**: Remove Framework and Add Helpers (Pending)

### Timeline
- **Week 1**: Complete workspace setup, start OSL-TASK-009
- **Week 2-3**: Complete OSL-TASK-009, work on MACROS-TASK-002
- **Week 4-5**: Complete MACROS-TASK-002, testing
- **Week 6**: Integration, documentation, polish

### Success Criteria
- ✅ airssys-osl-macros crate compiles and tests pass
- ✅ airssys-osl framework removed, helpers added
- ✅ All 165+ existing tests pass
- ✅ 20+ new tests for helpers and macros
- ✅ Zero warnings, full documentation
- ✅ ~30% code reduction overall

## Files Ready for Implementation

All memory bank files are complete and ready. The plan has been saved to:
- `airssys-osl/docs/architecture-refactoring-plan-2025-10.md`
- `airssys-osl-macros/product_context.md`
- `airssys-osl-macros/tech_context.md`
- `airssys-osl-macros/progress.md`
- `airssys-osl-macros/system_patterns.md`
- `airssys-osl-macros/tasks/MACROS-TASK-001-foundation-setup.md`
- `airssys-osl-macros/tasks/MACROS-TASK-002-executor-macro.md`
- `airssys-osl/tasks/OSL-TASK-009-remove-framework-add-helpers.md`

**Ready to proceed with workspace setup when approved!**
