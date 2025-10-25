# wit-bindgen Integration Guide for airssys-wasm

**Version:** 1.0.0  
**Audience:** Phase 3 Implementation Team

---

## Quick Start

### Prerequisites

```bash
# Install required tools
cargo install wit-bindgen-cli --version 0.47.0
cargo install wasm-tools --version 1.240.0
rustup target add wasm32-wasip1
```

### Basic Workflow

```bash
# 1. Validate WIT definitions
wasm-tools component wit wit/

# 2. Generate bindings (manual)
wit-bindgen rust --out-dir src/generated --world airssys-world wit/

# 3. Build component
cargo build --target wasm32-wasip1 --release

# 4. Inspect generated component
wasm-tools component wit target/wasm32-wasip1/release/airssys_wasm.wasm
```

---

## Integration Methods

### Method 1: build.rs (RECOMMENDED)

**Use:** Production builds, CI/CD

**Setup:**
1. Copy `build.rs.template` to `build.rs`
2. Configure in `Cargo.toml` (no extra dependencies needed)
3. Build normally with `cargo build`

**Advantages:**
- Automatic regeneration on WIT changes
- Integrated with Cargo workflow
- Works in CI without manual steps

### Method 2: Manual CLI

**Use:** Development, debugging, one-off generation

**Command:**
```bash
wit-bindgen rust \
    --out-dir src/generated \
    --world airssys-world \
    --ownership borrowing-duplicate-if-necessary \
    --format \
    wit/
```

**Advantages:**
- Direct control over generation
- Easier debugging of generation issues
- No build script overhead

---

## Configuration Reference

### wit-bindgen Options for airssys-wasm

| Option | Value | Rationale |
|--------|-------|-----------|
| `--out-dir` | `src/generated` | Clear separation from hand-written code |
| `--world` | `airssys-world` | Main component world (Phase 2) |
| `--ownership` | `borrowing-duplicate-if-necessary` | Optimal for host/guest patterns |
| `--format` | (flag) | Generate readable code with rustfmt |

### Environment Variables

| Variable | Purpose | Example |
|----------|---------|---------|
| `WIT_BINDGEN` | Custom wit-bindgen path | `/usr/local/bin/wit-bindgen` |
| `WASM_TOOLS` | Custom wasm-tools path | `/usr/local/bin/wasm-tools` |
| `AIRSSYS_WORLD` | Override world name | `test-world` |
| `AIRSSYS_BUILD_VERBOSE` | Enable verbose logging | `1` |

---

## Troubleshooting

### Error: "failed to parse package"

**Cause:** WIT syntax error

**Solution:**
```bash
wasm-tools component wit wit/ 2>&1 | less
# Fix errors shown, rebuild
```

### Error: "can't find crate for `core`"

**Cause:** wit-bindgen macro incompatibility with wasm targets

**Solution:** Use CLI-based generation (build.rs method)

### Error: "wit-bindgen: command not found"

**Cause:** wit-bindgen not installed

**Solution:**
```bash
cargo install wit-bindgen-cli --version 0.47.0
```

---

## Complete Reference

See:
- `wit_bindgen_core_concepts.md` - Detailed wit-bindgen documentation
- `multi_package_binding_patterns.md` - Multi-package strategies
- `build.rs.template` - Production-ready build script
- `cargo_configuration_guide.md` - Cargo.toml setup

---

**Document Status:** ✅ Complete
