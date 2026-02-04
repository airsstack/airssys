# Rust main.rs and lib.rs Crate Structure Guide

This guide covers best practices for structuring Rust projects that have both `main.rs` (binary crate) and `lib.rs` (library crate) in the same package.

## Understanding the Two-Crate Model

When a Cargo package contains both `src/main.rs` and `src/lib.rs`, Cargo treats them as **two separate crates**:

1. **Library crate** (`lib.rs`) - The crate root for your library, named after the package
2. **Binary crate** (`main.rs`) - A separate binary crate, also named after the package

This is a crucial distinction: they share the same `Cargo.toml` dependencies but are compiled as separate crates with separate module trees.

## The Import Rule

### Correct: Use Package Name

In `main.rs`, import from the library using the **package name**:

```rust
// main.rs - CORRECT
use mypackage::core::types::Error;
use mypackage::infra::StdFileSystem;
use mypackage::config::TomlConfigStore;

fn main() {
    // Use library functionality
}
```

### Incorrect: Re-declaring Modules

Do NOT re-declare modules that already exist in `lib.rs`:

```rust
// main.rs - INCORRECT
mod core;      // This creates a SEPARATE module tree!
mod infra;     // Duplicates what's already in lib.rs
mod config;

use core::types::Error;  // This is NOT the same as lib.rs's core module
```

### Why This Matters

When you use `mod` in `main.rs`, you're creating a **new module tree** for the binary crate. This means:

- Code is compiled twice (once for lib, once for binary)
- Types are incompatible between the two trees
- Changes in one module tree don't affect the other
- You lose the benefits of having a library crate

## Recommended Project Structure

### Structure 1: Thin Binary Wrapper (Recommended for CLI tools)

```
src/
├── lib.rs          # Library crate root - declares all modules
├── main.rs         # Binary crate - thin wrapper importing from library
├── core/           # Shared modules (owned by lib.rs)
├── config/
├── infra/
└── cli.rs          # Binary-only module (declared only in main.rs)
```

**lib.rs:**
```rust
//! Library crate - exports reusable functionality

pub mod core;
pub mod config;
pub mod infra;
pub mod content;

// Re-export common types for convenience
pub use core::types::{Error, Result};
```

**main.rs:**
```rust
//! Binary crate - thin CLI wrapper

// Binary-only modules (not in lib.rs)
mod cli;

// Import from the library crate using package name
use mypackage::config::TomlConfigStore;
use mypackage::core::infra::{FileSystem, Logger};
use mypackage::infra::{StdFileSystem, ColoredLogger};

fn main() {
    let cli = cli::parse();
    // Use library functionality...
}
```

### Structure 2: Binary in src/bin/ Directory

An alternative that avoids overlapping crate roots:

```
src/
├── lib.rs              # Library crate root
├── bin/
│   └── mypackage.rs    # Binary crate
├── core/
├── config/
└── infra/
```

This structure makes it clear that the binary is separate from the library.

## When to Use Each Approach

### Use main.rs + lib.rs when:

- Building a CLI tool with reusable library functionality
- Want to test library logic independently via `cargo test`
- Need to expose functionality for other crates to use
- Have a single binary that wraps library functionality

### Use workspace structure when:

- Have multiple binaries
- Library and binary have significantly different dependencies
- Need independent versioning for library and binary
- Project is large and compile times matter

### Use bin/ directory when:

- Want clear separation without workspace overhead
- Have multiple binaries sharing the same library
- Prefer explicit directory structure

## The `crate::` Keyword Behavior

Understanding `crate::` is critical:

| Context | `crate::` refers to |
|---------|---------------------|
| Inside `lib.rs` or modules declared in `lib.rs` | Library crate root |
| Inside `main.rs` or modules declared in `main.rs` | Binary crate root |

```rust
// In lib.rs or its modules
use crate::core::types::Error;  // Refers to library's core module

// In main.rs
use crate::cli::Args;  // Refers to binary's cli module (if declared with mod)
use mypackage::core::types::Error;  // Refers to library's core module
```

## Common Mistakes

### Mistake 1: Duplicate Module Declarations

```rust
// lib.rs
pub mod core;
pub mod config;

// main.rs - WRONG!
mod core;    // Creates separate, incompatible module
mod config;
```

### Mistake 2: Using crate:: to Access Library

```rust
// main.rs - WRONG!
use crate::core::types::Error;  // This doesn't exist in binary crate!

// main.rs - CORRECT
use mypackage::core::types::Error;  // Use package name
```

### Mistake 3: Forgetting pub in lib.rs

```rust
// lib.rs - WRONG!
mod core;  // Private! Can't be accessed from main.rs

// lib.rs - CORRECT
pub mod core;  // Public, accessible from main.rs
```

## Practical Example

Given a package named `aiassisted`:

**Cargo.toml:**
```toml
[package]
name = "aiassisted"
```

**lib.rs:**
```rust
pub mod config;
pub mod content;
pub mod core;
pub mod infra;

pub use core::types::{Error, Result};
```

**main.rs:**
```rust
// Binary-only module for CLI parsing
mod cli;

// Import from the library using package name
use aiassisted::config::TomlConfigStore;
use aiassisted::content::{InstallCommand, UpdateCommand};
use aiassisted::core::infra::{FileSystem, HttpClient, Logger};
use aiassisted::infra::{StdFileSystem, ReqwestClient, ColoredLogger};

use cli::{Cli, Commands};

fn main() {
    let cli = Cli::parse();
    // ...
}
```

## Summary

| Aspect | Guideline |
|--------|-----------|
| Module declaration | Declare shared modules only in `lib.rs` |
| Imports in main.rs | Use package name: `use mypackage::module` |
| Binary-only modules | Declare with `mod` only in `main.rs` |
| `crate::` keyword | Only use for binary-specific code in main.rs |
| Testing | Test library functionality, not binary wrappers |

## References

- [Packages and Crates - The Rust Book](https://doc.rust-lang.org/book/ch07-01-packages-and-crates.html)
- [Separating Modules into Different Files - The Rust Book](https://doc.rust-lang.org/book/ch07-05-separating-modules-into-different-files.html)
- [main.rs and lib.rs at same level - Rust Forum](https://users.rust-lang.org/t/main-rs-and-lib-rs-at-same-level/42499)
- [API Guidelines Discussion: main.rs and lib.rs pattern](https://github.com/rust-lang/api-guidelines/discussions/167)
