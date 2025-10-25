# Cargo Configuration Guide for airssys-wasm

**Version:** 1.0.0  
**Date:** 2025-10-25

---

## Recommended Cargo.toml Configuration

### Complete Configuration

```toml
[package]
name = "airssys-wasm"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
rust-version.workspace = true

[lib]
crate-type = ["cdylib", "rlib"]
# cdylib: WebAssembly component output
# rlib: Rust library for integration

[dependencies]
# ═══════════════════════════════════════════════════════════
# CORE RUNTIME DEPENDENCIES
# ═══════════════════════════════════════════════════════════

# Serialization (workspace dependency)
serde = { workspace = true }
serde_json = { workspace = true }

# Error handling
thiserror = { workspace = true }
anyhow = { workspace = true }

# Time handling (chrono DateTime<Utc> standard §3.2)
chrono = { workspace = true }

# Async runtime
tokio = { workspace = true, features = ["macros", "rt", "fs"] }
async-trait = { workspace = true }

# UUIDs
uuid = { workspace = true }

# ═══════════════════════════════════════════════════════════
# WASMTIME DEPENDENCIES
# ═══════════════════════════════════════════════════════════

# Wasmtime runtime with component model support
wasmtime = { version = "24.0", features = ["component-model", "async", "cranelift"] }
wasmtime-wasi = { version = "24.0" }

# TOML parsing for Component.toml configuration
toml = { workspace = true }

# ═══════════════════════════════════════════════════════════
# NOTE: NO wit-bindgen dependency needed!
# CLI-generated bindings are self-contained
# ═══════════════════════════════════════════════════════════

[dev-dependencies]
# WAT compiler for test fixtures
wat = "1.240"

# Temporary file/directory management
tempfile = "3.8"

# Criterion for benchmarking
criterion = { version = "0.5", features = ["async_tokio", "html_reports"] }

# ═══════════════════════════════════════════════════════════
# BUILD DEPENDENCIES
# ═══════════════════════════════════════════════════════════

[build-dependencies]
# NO build dependencies needed
# build.rs invokes CLI tools via Command::new()

# ═══════════════════════════════════════════════════════════
# FEATURES
# ═══════════════════════════════════════════════════════════

[features]
default = ["core"]

# Core packages (always included)
core = []

# Extension packages (optional)
filesystem = []
network = []
process = []

# All extensions
all-extensions = ["filesystem", "network", "process"]

# ═══════════════════════════════════════════════════════════
# BENCHMARKS
# ═══════════════════════════════════════════════════════════

[[bench]]
name = "component_loading"
harness = false

[[bench]]
name = "component_execution"
harness = false

# ═══════════════════════════════════════════════════════════
# METADATA
# ═══════════════════════════════════════════════════════════

[package.metadata]
# Document required tool versions
wasm-tools-version = "1.240.0"
wit-bindgen-version = "0.47.0"

[lints]
workspace = true
```

---

## Key Configuration Decisions

### 1. No wit-bindgen Runtime Dependency

**Why:** CLI-generated bindings are self-contained and include necessary runtime support inline.

**Alternative (macro approach):**
```toml
# Only needed if using macro-based generation
wit-bindgen = { version = "0.47.0", default-features = false, features = ["macros"] }
```

**Recommendation:** Use CLI approach (no dependency)

### 2. Crate Types

**`cdylib`:** Required for WebAssembly component output
**`rlib`:** Allows integration with other Rust crates

**Both included for maximum flexibility**

### 3. Wasmtime Version Pinning

**Version:** 24.0 (matches existing airssys-wasm)

**Features:**
- `component-model`: Component Model support
- `async`: Async execution support
- `cranelift`: JIT compilation backend

### 4. Feature Flags Strategy

**Core features:** Always included
**Extension features:** Optional, opt-in

**Usage:**
```bash
# Build with specific features
cargo build --features filesystem,network

# Build with all extensions
cargo build --features all-extensions
```

---

## Workspace Integration

### workspace.dependencies

Ensure workspace Cargo.toml has:

```toml
[workspace.dependencies]
# AirsSys crates
airssys-wasm = { path = "airssys-wasm" }

# Core dependencies (versions centralized)
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
anyhow = "1.0"
chrono = { version = "0.4", features = ["serde"] }
tokio = { version = "1.47", features = ["full"] }
async-trait = "0.1"
uuid = { version = "1.0", features = ["v4", "serde"] }
toml = "0.8"
```

---

## Tool Version Requirements

### Required CLI Tools

**wit-bindgen:**
- Version: 0.47.0
- Install: `cargo install wit-bindgen-cli --version 0.47.0`

**wasm-tools:**
- Version: 1.240.0  
- Install: `cargo install wasm-tools --version 1.240.0`

**rustup targets:**
- `wasm32-wasip1`: `rustup target add wasm32-wasip1`

### Version Checking Script

```bash
#!/bin/bash
# tools/check_versions.sh

echo "Checking required tool versions..."

# Check wit-bindgen
WIT_BINDGEN_VERSION=$(wit-bindgen --version | awk '{print $2}')
if [ "$WIT_BINDGEN_VERSION" != "0.47.0" ]; then
    echo "❌ wit-bindgen version mismatch: $WIT_BINDGEN_VERSION (expected 0.47.0)"
    exit 1
fi

# Check wasm-tools
WASM_TOOLS_VERSION=$(wasm-tools --version | awk '{print $2}')
if [ "$WASM_TOOLS_VERSION" != "1.240.0" ]; then
    echo "❌ wasm-tools version mismatch: $WASM_TOOLS_VERSION (expected 1.240.0)"
    exit 1
fi

echo "✅ All tool versions correct"
```

---

## CI/CD Configuration

### GitHub Actions

```yaml
name: Build and Test

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: wasm32-wasip1
          
      - name: Cache tools
        uses: actions/cache@v4
        with:
          path: ~/.cargo/bin
          key: ${{ runner.os }}-tools-${{ hashFiles('**/Cargo.toml') }}
      
      - name: Install wit-bindgen
        run: cargo install wit-bindgen-cli --version 0.47.0
        
      - name: Install wasm-tools
        run: cargo install wasm-tools --version 1.240.0
        
      - name: Validate WIT
        run: wasm-tools component wit wit/
        
      - name: Build (native)
        run: cargo build --all-features
        
      - name: Build (WASM)
        run: cargo build --target wasm32-wasip1 --release
        
      - name: Run tests
        run: cargo test --all-features
```

---

**Document Status:** ✅ Complete  
**Next:** build.rs.template implementation
