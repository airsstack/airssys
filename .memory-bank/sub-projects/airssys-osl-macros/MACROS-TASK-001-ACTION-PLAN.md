# MACROS-TASK-001 Action Plan

**Task:** Foundation Setup and Workspace Integration  
**Estimated Time:** 4 hours  
**Current Progress:** 50% (Memory bank complete)  
**Date Created:** 2025-10-08

---

## Overview

This action plan details the step-by-step execution of MACROS-TASK-001, setting up the airssys-osl-macros proc-macro crate foundation.

## Phase 1: Workspace Integration (30 minutes)

### Action 1.1: Update Workspace Cargo.toml - Add Members
**File:** `/Users/hiraq/Projects/airsstack/airssys/Cargo.toml`

**Changes:**
```toml
[workspace]
members = [
    "airssys-osl", 
    "airssys-osl-macros",  # ← ADD THIS LINE
    "airssys-rt",
    "airssys-wasm",
    "airssys-wasm-component",
]
```

### Action 1.2: Update Workspace Cargo.toml - Add Dependencies
**File:** `/Users/hiraq/Projects/airsstack/airssys/Cargo.toml`

**Add after Layer 1 (AirsSys Foundation Crates):**
```toml
[workspace.dependencies]
# Layer 1: AIRS Foundation Crates (MUST be at top)
airssys-osl-macros = { path = "airssys-osl-macros" }
```

**Add in Layer 4 or create new section for Proc-Macro Tools:**
```toml
# Proc-macro development dependencies
syn = { version = "2.0", features = ["full", "visit", "visit-mut"] }
quote = "1.0"
proc-macro2 = "1.0"
trybuild = "1.0"
```

---

## Phase 2: Create Crate Structure (1 hour)

### Action 2.1: Create airssys-osl-macros/Cargo.toml
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
trybuild = { workspace = true }

[lints]
workspace = true
```

### Action 2.2: Create src/lib.rs
```rust
//! Procedural macros for airssys-osl core abstractions.
//!
//! This crate provides ergonomic macros to reduce boilerplate when
//! implementing airssys-osl traits.
//!
//! # Available Macros
//!
//! - `#[executor]`: Generate `OSExecutor<O>` trait implementations
//!
//! # Example
//!
//! ```rust,ignore
//! use airssys_osl::prelude::*;
//!
//! #[executor]
//! impl MyExecutor {
//!     async fn file_read(&self, op: FileReadOperation, ctx: &ExecutionContext) 
//!         -> OSResult<ExecutionResult> 
//!     {
//!         // Custom implementation
//!         todo!()
//!     }
//! }
//! ```

// Layer 1: Standard library imports
use proc_macro::TokenStream;

// Layer 2: Third-party imports (none yet)

// Layer 3: Internal imports
mod executor;
mod utils;

/// Generates `OSExecutor<O>` trait implementations from method names.
///
/// This attribute macro reduces boilerplate by automatically generating trait
/// implementations from method names. See crate documentation for details.
#[proc_macro_attribute]
pub fn executor(_attr: TokenStream, item: TokenStream) -> TokenStream {
    executor::expand(item.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
```

### Action 2.3: Create src/executor.rs
```rust
//! #[executor] macro implementation
//!
//! This module will contain the core logic for parsing impl blocks
//! and generating OSExecutor trait implementations.

use proc_macro2::TokenStream;
use syn::Result;

/// Expands the #[executor] attribute macro.
///
/// Currently a placeholder - returns input unchanged.
/// Full implementation in MACROS-TASK-002.
pub fn expand(input: TokenStream) -> Result<TokenStream> {
    // Placeholder: Return input unchanged for now
    // Real implementation in MACROS-TASK-002
    Ok(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn test_expand_placeholder() {
        let input = quote! {
            impl MyExecutor {
                async fn file_read(&self) {}
            }
        };
        
        let result = expand(input.clone());
        assert!(result.is_ok());
    }
}
```

### Action 2.4: Create src/utils.rs
```rust
//! Shared utilities for macro implementations

use syn::Ident;

/// Maps method names to operation types.
///
/// Returns (OperationType, ModulePath) tuple.
/// Full mapping table will be added in MACROS-TASK-002.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filesystem_operations() {
        let ident = syn::parse_str::<Ident>("file_read").unwrap();
        assert_eq!(
            map_method_name_to_operation(&ident),
            Some(("FileReadOperation", "filesystem"))
        );
    }

    #[test]
    fn test_invalid_operation() {
        let ident = syn::parse_str::<Ident>("invalid_op").unwrap();
        assert_eq!(map_method_name_to_operation(&ident), None);
    }
}
```

---

## Phase 3: Test Infrastructure (30 minutes)

### Directory Structure
```
tests/
├── unit/
│   └── mapping_tests.rs
├── integration.rs
└── ui/
    └── .gitkeep
```

### Action 3.1: Create tests/integration.rs
```rust
//! Integration tests for airssys-osl-macros
//!
//! Real tests will be added in MACROS-TASK-002

#[test]
fn test_placeholder() {
    // Placeholder test - verifies test infrastructure works
    assert!(true);
}

#[test]
fn test_crate_compiles() {
    // If this test runs, the crate compiled successfully
    assert_eq!(2 + 2, 4);
}
```

### Action 3.2: Create tests/unit/mapping_tests.rs
```rust
//! Unit tests for operation name mapping

use airssys_osl_macros; // Import to ensure it compiles

#[test]
fn test_unit_infrastructure() {
    // Placeholder for unit test infrastructure
    assert!(true);
}
```

### Action 3.3: Create tests/ui/.gitkeep
(Empty file to preserve directory)

---

## Phase 4: Documentation (30 minutes)

### Action 4.1: Create README.md
```markdown
# airssys-osl-macros

Procedural macros for [airssys-osl](../airssys-osl) core abstractions.

## Overview

This crate provides ergonomic macros to reduce boilerplate when implementing
airssys-osl traits, particularly for custom executor implementations.

## Status

**Foundation Setup** - Basic structure in place, macro implementation pending.

See [progress tracking](../.memory-bank/sub_projects/airssys-osl-macros/progress.md) for current status.

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

## Development

This is a proc-macro crate and must be compiled for the host platform.

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

## Documentation

See the [airssys-osl documentation](../airssys-osl) for complete integration details.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE))
- MIT license ([LICENSE-MIT](../LICENSE-MIT))

at your option.
```

---

## Phase 5: Validation & Quality Gates (1 hour)

### Commands to Execute:

```bash
# Compile checks
cd airssys-osl-macros
cargo check

cd ..
cargo check --workspace

# Run tests
cargo test --workspace

# Clippy
cargo clippy --workspace --all-targets --all-features

# Documentation
cargo doc --workspace --no-deps
```

**Expected Results:**
- ✅ Zero compilation errors
- ✅ Zero warnings
- ✅ All tests pass
- ✅ Documentation builds successfully

---

## Phase 6: Memory Bank Update & Git Commit (30 minutes)

### Action 6.1: Update progress.md
Update status to 100% (MACROS-TASK-001 complete)

### Action 6.2: Git Commit
```bash
git add .
git commit -m "feat(macros): Initialize airssys-osl-macros proc-macro crate

- Add airssys-osl-macros to workspace members
- Configure proc-macro crate structure (lib, proc-macro = true)
- Add dependencies: syn, quote, proc-macro2
- Create placeholder #[executor] macro
- Setup test infrastructure (unit, integration, ui directories)
- Create comprehensive README and documentation

Completes: MACROS-TASK-001 (Foundation Setup)
Next: MACROS-TASK-002 (Implement #[executor] macro)

Part of architecture refactoring plan (October 2025)
Ref: .memory-bank/sub_projects/airssys-osl/docs/architecture-refactoring-plan-2025-10.md"
```

---

## Execution Summary

### Total Files Created: 9 files
1. Workspace Cargo.toml (modified)
2. airssys-osl-macros/Cargo.toml
3. airssys-osl-macros/src/lib.rs
4. airssys-osl-macros/src/executor.rs
5. airssys-osl-macros/src/utils.rs
6. airssys-osl-macros/tests/integration.rs
7. airssys-osl-macros/tests/unit/mapping_tests.rs
8. airssys-osl-macros/tests/ui/.gitkeep
9. airssys-osl-macros/README.md

### Time Breakdown:
- Phase 1 (Workspace): 30 min
- Phase 2 (Crate): 1 hour
- Phase 3 (Tests): 30 min
- Phase 4 (Docs): 30 min
- Phase 5 (Validation): 1 hour
- Phase 6 (Commit): 30 min
- **Total: 4 hours**

---

## Quality Checklist
- [ ] Workspace member added ✓
- [ ] Dependencies configured ✓
- [ ] Source files created ✓
- [ ] Test infrastructure ready ✓
- [ ] README comprehensive ✓
- [ ] cargo check passes ✓
- [ ] cargo test passes ✓
- [ ] Zero warnings ✓
- [ ] Zero clippy warnings ✓
- [ ] Documentation builds ✓
- [ ] Memory bank updated ✓
- [ ] Workspace standards (§2.1, §4.3, §5.1) ✓

---

**Status:** Ready for execution
**Next Task:** MACROS-TASK-002 (Implement #[executor] macro)
