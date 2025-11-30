# Getting Started with airssys-wasm

**Document Type:** Tutorial (Diátaxis)  
**Audience:** Developers building host applications that run WASM components  
**Prerequisites:** Basic Rust knowledge, familiarity with WebAssembly concepts

## Overview

This guide walks you through setting up your development environment and building your first application that hosts WASM components using airssys-wasm.

## Prerequisites

### Required Tools

To build airssys-wasm, you need:

1. **Rust 1.80 or later**
   ```bash
   # Check your Rust version
   rustc --version
   
   # Update if needed
   rustup update
   ```

2. **wasm-tools 1.240.0**
   ```bash
   cargo install wasm-tools --version 1.240.0
   
   # Verify installation
   wasm-tools --version
   ```

3. **wit-bindgen 0.47.0**
   ```bash
   cargo install wit-bindgen-cli --version 0.47.0
   
   # Verify installation
   wit-bindgen --version
   ```

### System Requirements

- **Operating System:** Linux, macOS, or Windows
- **RAM:** 4GB minimum (8GB recommended for development)
- **Disk Space:** 2GB for toolchain and dependencies

## Installation

### Step 1: Clone the Repository

```bash
# Clone the AirsSys workspace
git clone https://github.com/airsstack/airssys
cd airssys/airssys-wasm
```

### Step 2: Build the Library

```bash
# Standard build (validates WIT and generates bindings)
cargo build

# Expected output:
# - WIT validation passes
# - Bindings generated in src/generated/
# - Library compiles successfully
```

**Build Time Expectations:**
- **First build:** ~10 seconds (includes dependency compilation and binding generation)
- **Incremental builds:** ~2 seconds (no WIT changes)
- **Incremental with WIT changes:** ~4 seconds (re-validates and regenerates bindings)

### Step 3: Verify Installation

```bash
# Run the test suite
cargo test

# Expected: 250+ tests passing
```

## Build System Architecture

The build system uses a two-stage approach:

### Stage 1: WIT Validation

The build script (`build.rs`) validates all WIT packages using `wasm-tools`:

```bash
# Manual validation (optional)
wasm-tools component wit wit/core
```

This ensures:
- WIT syntax is correct
- Type references are valid
- Package dependencies are acyclic
- All interfaces are well-formed

### Stage 2: Binding Generation

After validation, `wit-bindgen` generates Rust bindings:

```bash
# Manual binding generation (optional)
wit-bindgen rust wit/core
```

Generated code includes:
- **Type-safe Rust structs** for all WIT records and variants
- **Trait definitions** for component lifecycle interfaces
- **Import stubs** for host services (logging, messaging, timing)
- **Total:** ~154KB of generated Rust code (2,794 lines)

**Output Location:** `src/generated/airssys_component.rs`

## Build Commands Reference

### Standard Builds

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Check without building
cargo check
```

### Clean Builds

Useful after making WIT changes:

```bash
# Remove build artifacts and rebuild
cargo clean && cargo build
```

### Verbose Output

Enable verbose build output to see WIT validation details:

```bash
# Set environment variable for verbose output
AIRSSYS_BUILD_VERBOSE=1 cargo build
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test permission_parsing

# Run tests with output
cargo test -- --nocapture
```

### Documentation

```bash
# Generate and open documentation
cargo doc --open

# Generate documentation for dependencies
cargo doc --all --open
```

## Generated Code Overview

The build system generates comprehensive Rust bindings from WIT definitions:

### Generated Files

- **Location:** `src/generated/airssys_component.rs`
- **Size:** ~154KB (2,794 lines)
- **Generated from:** 16 WIT files (2,214 lines of WIT definitions)

### What Gets Generated

#### 1. Core Types
```rust
// Example: ComponentId type from WIT
pub struct ComponentId {
    pub value: String,
}

// Example: Capability variant from WIT
pub enum Capability {
    FileRead(FileReadCapability),
    FileWrite(FileWriteCapability),
    // ... more variants
}
```

#### 2. Component Lifecycle Traits
```rust
// Generated component lifecycle trait
pub trait Component {
    fn init(&mut self, config: ComponentConfig) -> Result<(), ComponentError>;
    fn execute(&mut self, input: ComponentInput) -> Result<ComponentOutput, ComponentError>;
    fn shutdown(&mut self) -> Result<(), ComponentError>;
}
```

#### 3. Host Function Imports
```rust
// Generated host function stubs
mod host {
    pub fn log(level: LogLevel, message: String);
    pub fn send_message(target: ComponentId, message: Vec<u8>);
    pub fn get_time() -> Timestamp;
}
```

## Build Configuration

### Cargo.toml Configuration

The `Cargo.toml` includes build dependencies for WIT processing:

```toml
[build-dependencies]
# WIT validation and binding generation handled by build.rs
```

### Build Script (build.rs)

The build script (`build.rs`, 176 lines) handles:

1. **WIT Validation**
   - Validates all WIT packages in `wit/` directory
   - Checks syntax, types, and dependencies
   - Fails build if validation errors occur

2. **Binding Generation**
   - Generates Rust code from WIT definitions
   - Outputs to `src/generated/`
   - Included automatically in library

3. **Incremental Build Optimization**
   - Detects WIT file changes
   - Skips regeneration if WIT unchanged
   - Rebuilds only when necessary

## Next Steps

Now that you have airssys-wasm installed and building:

1. **Learn the Architecture:** Read [Architecture Overview](../architecture/overview.md)
2. **Explore Components:** See [Component Development Guide](component-development.md)
3. **Build Your First Host App:** Follow [Host Application Tutorial](host-application-tutorial.md) (coming soon)
4. **Configure Permissions:** Learn about [Component.toml Configuration](../reference/component-toml-spec.md)

## Troubleshooting

If you encounter issues during setup, see the [Troubleshooting Guide](troubleshooting.md) for solutions to common problems.

## Summary

You've learned:
- ✅ How to install required tools (Rust, wasm-tools, wit-bindgen)
- ✅ How to build airssys-wasm from source
- ✅ How the two-stage build system works (validation + generation)
- ✅ What code gets generated from WIT definitions
- ✅ How to run tests and generate documentation

You're now ready to start building WASM component host applications with airssys-wasm!
