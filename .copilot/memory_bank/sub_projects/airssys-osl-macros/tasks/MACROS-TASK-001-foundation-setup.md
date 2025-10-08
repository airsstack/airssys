# Task: Foundation Setup and Workspace Integration

**Task ID:** MACROS-TASK-001  
**Priority:** Critical  
**Status:** In Progress  
**Created:** 2025-10-08  
**Estimated Effort:** 4 hours  

## Task Overview
Set up the airssys-osl-macros proc-macro crate foundation, including workspace integration, basic crate structure, dependency configuration, and development environment setup.

## Task Description
Create the complete foundation for the proc-macro crate that will provide ergonomic macros for airssys-osl abstractions. This includes workspace member registration, proc-macro crate configuration, dependency setup, and basic project structure.

## Dependencies
- **Blocked by:** None (first task)
- **Blocks:** MACROS-TASK-002 (#[executor] macro implementation)
- **Related:** 
  - OSL-TASK-009 (airssys-osl framework removal and helpers)
  - Workspace standards compliance

## Acceptance Criteria

### 1. Workspace Member Setup
- ✅ airssys-osl-macros added to workspace Cargo.toml members
- ✅ Workspace dependencies configured for macros crate
- ✅ Feature flag integration planned with airssys-osl

### 2. Crate Structure Created
- ✅ `airssys-osl-macros/Cargo.toml` with proc-macro configuration
- ✅ `airssys-osl-macros/src/lib.rs` with macro exports
- ✅ `airssys-osl-macros/src/executor.rs` placeholder for #[executor] macro
- ✅ `airssys-osl-macros/src/utils.rs` for shared utilities
- ✅ README.md with project overview

### 3. Dependencies Configured
- ✅ syn = "2.0" with required features
- ✅ quote = "1.0" for code generation
- ✅ proc-macro2 = "1.0" for token stream handling
- ✅ Dev dependencies: airssys-osl, async-trait, tokio, trybuild

### 4. Test Infrastructure
- ✅ `tests/unit/` directory for unit tests
- ✅ `tests/integration.rs` for integration tests
- ✅ `tests/ui/` directory for trybuild UI tests
- ✅ Basic test structure and examples

### 5. Quality Gates
- ✅ cargo check passes
- ✅ Zero compiler warnings
- ✅ README and basic documentation created
- ✅ Workspace standards compliance (§2.1, §4.3, §5.1)

## Implementation Details

### Workspace Configuration

#### Add to Workspace Members
```toml
# In /airssys/Cargo.toml
[workspace]
members = [
    "airssys-osl",
    "airssys-osl-macros",  # ← Add this
    "airssys-rt",
    "airssys-wasm",
    "airssys-wasm-component",
]
```

#### Workspace Dependencies
```toml
# In /airssys/Cargo.toml
[workspace.dependencies]
# AirsSys Foundation Crates
airssys-osl = { path = "airssys-osl" }
airssys-osl-macros = { path = "airssys-osl-macros" }

# Proc-macro dependencies
syn = { version = "2.0", features = ["full", "visit", "visit-mut"] }
quote = "1.0"
proc-macro2 = "1.0"
```

### Crate Configuration

#### Cargo.toml
```toml
[package]
name = "airssys-osl-macros"
version = "0.1.0"
edition = "2021"
authors = ["AirsSys Contributors"]
description = "Procedural macros for airssys-osl core abstractions"
license = "MIT OR Apache-2.0"
repository = "https://github.com/airsstack/airssys"
keywords = ["os", "system", "macros", "proc-macro"]
categories = ["os", "development-tools::procedural-macro-helpers"]

[lib]
proc-macro = true

[dependencies]
syn = { workspace = true }
quote = { workspace = true }
proc-macro2 = { workspace = true }

[dev-dependencies]
airssys-osl = { workspace = true }
async-trait = { workspace = true }
tokio = { workspace = true }
trybuild = "1.0"
```

#### lib.rs Structure
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

// Layer 2: Third-party imports (empty for now)

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

#### executor.rs Placeholder
```rust
//! #[executor] macro implementation

use proc_macro2::TokenStream;
use syn::Result;

pub fn expand(input: TokenStream) -> Result<TokenStream> {
    // Placeholder: Return input unchanged for now
    Ok(input)
}
```

#### utils.rs Placeholder
```rust
//! Shared utilities for macro implementations

use syn::{Ident, Result};

/// Maps method names to operation types
pub fn map_method_name_to_operation(name: &Ident) -> Option<(&'static str, &'static str)> {
    match name.to_string().as_str() {
        "file_read" => Some(("FileReadOperation", "filesystem")),
        "file_write" => Some(("FileWriteOperation", "filesystem")),
        // More mappings added in MACROS-TASK-002
        _ => None,
    }
}
```

### Test Infrastructure

#### Directory Structure
```
tests/
├── unit/                    # Unit tests for utilities
│   └── mapping_tests.rs
├── integration.rs           # Integration tests with airssys-osl
└── ui/                      # UI tests for error messages (trybuild)
    ├── invalid_signature.rs
    └── invalid_signature.stderr
```

#### Basic Integration Test
```rust
// tests/integration.rs
#[test]
fn test_placeholder() {
    // Placeholder test - real tests in MACROS-TASK-002
    assert!(true);
}
```

### README.md
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

## Usage

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
```

## Testing Plan

### Phase 1: Setup Validation
1. `cargo check` in airssys-osl-macros directory
2. `cargo check --workspace` from root
3. Verify no compiler warnings
4. Verify workspace dependency resolution

### Phase 2: Basic Tests
1. Run placeholder integration test
2. Verify test infrastructure works
3. Add unit test for method name mapping

### Phase 3: Quality Gates
1. Run `cargo clippy` - zero warnings
2. Run `cargo doc` - documentation builds
3. Verify workspace standards compliance

## Documentation Requirements

### Code Documentation
- Comprehensive rustdoc for lib.rs
- Module-level documentation for executor.rs, utils.rs
- Examples in documentation (placeholder for now)

### Project Documentation
- README.md with overview and status
- Integration notes in airssys-osl documentation
- Memory bank updates (product_context, tech_context, progress)

## Quality Checklist

### Before Task Completion
- [ ] Workspace member added to Cargo.toml
- [ ] Proc-macro crate structure created
- [ ] All dependencies configured correctly
- [ ] Test infrastructure in place
- [ ] README.md created
- [ ] cargo check passes (workspace and crate)
- [ ] Zero compiler warnings
- [ ] Zero clippy warnings
- [ ] Documentation complete
- [ ] Memory bank updated
- [ ] Git commit with proper message

### Validation Commands
```bash
# From workspace root
cargo check --workspace
cargo clippy --workspace --all-targets
cargo doc --workspace --no-deps

# From airssys-osl-macros directory
cargo check
cargo test
cargo doc --no-deps
```

## Next Steps
After completion, proceed to **MACROS-TASK-002**: Implement #[executor] macro core functionality.

## Notes
- Keep structure minimal and focused (YAGNI)
- Follow workspace standards strictly (§2.1, §4.3, §5.1)
- Proc-macro crates have special constraints (host compilation, no runtime deps)
- Foundation must be solid before macro implementation
