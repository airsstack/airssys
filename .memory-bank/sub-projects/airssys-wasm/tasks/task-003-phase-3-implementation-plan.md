# Phase 3 Implementation Plan - Build System Integration

**Version:** 1.0.0  
**Timeline:** Day 7-9 (6 hours total)  
**Prerequisites:** Phase 2 complete (7 WIT packages implemented)

---

## Overview

Phase 3 implements build system integration for airssys-wasm, enabling automatic binding generation from WIT definitions and complete component build workflow.

---

## Day 7: build.rs Implementation (2 hours)

### Task 3.1.1: Create build.rs (30 min)

**Actions:**
- [ ] Copy `build.rs.template` to `build.rs`
- [ ] Review and customize for airssys-wasm
- [ ] Add world name (from Phase 2)
- [ ] Test basic execution

**Validation:**
```bash
cargo clean
cargo build 2>&1 | grep "WIT bindings generated"
```

### Task 3.1.2: Configure Cargo.toml (15 min)

**Actions:**
- [ ] Review current `Cargo.toml`
- [ ] Ensure no wit-bindgen runtime dependency
- [ ] Verify crate-type includes `cdylib`
- [ ] Add metadata for tool versions

**Validation:**
```bash
cargo check --lib
```

### Task 3.1.3: Test Binding Generation (45 min)

**Actions:**
- [ ] Run `cargo build`
- [ ] Verify `src/generated/` created
- [ ] Inspect generated code
- [ ] Check for compilation errors
- [ ] Test incremental builds

**Validation:**
```bash
# Clean build
cargo clean && cargo build
# Incremental (no changes)
cargo build
# Incremental (WIT change)
touch wit/core/types/types.wit && cargo build
```

### Task 3.1.4: Error Handling Testing (30 min)

**Actions:**
- [ ] Introduce WIT syntax error
- [ ] Verify build.rs catches error
- [ ] Check error message clarity
- [ ] Fix error, rebuild
- [ ] Test missing tool scenario

**Validation:**
Error messages are clear and actionable

---

## Day 8: Integration and Testing (2 hours)

### Task 3.2.1: Generated Code Integration (45 min)

**Actions:**
- [ ] Create `src/lib.rs` integration
- [ ] Import generated bindings
- [ ] Implement stub exports
- [ ] Test compilation

**Example:**
```rust
// src/lib.rs
#[allow(warnings)]
mod generated;

pub use generated::*;

pub struct AirsysWasmRuntime;

impl generated::exports::airssys::core_component::component_lifecycle::Guest 
    for AirsysWasmRuntime 
{
    fn init(config: generated::airssys::core_component::component_lifecycle::ComponentConfig) 
        -> Result<(), generated::airssys::core_types::types::ComponentError> 
    {
        // TODO: Implement
        Ok(())
    }
    // ... other methods
}

generated::export!(AirsysWasmRuntime with_types_in generated);
```

### Task 3.2.2: Build Component (30 min)

**Actions:**
- [ ] Build for wasm32-wasip1
- [ ] Verify .wasm output
- [ ] Use wasm-tools to inspect
- [ ] Document build command

**Commands:**
```bash
cargo build --target wasm32-wasip1 --release
wasm-tools component wit target/wasm32-wasip1/release/airssys_wasm.wasm
```

### Task 3.2.3: Cross-Platform Testing (45 min)

**Actions:**
- [ ] Test on Linux (if available)
- [ ] Test on macOS
- [ ] Test on Windows (if available)
- [ ] Document platform-specific issues

---

## Day 9: Validation and Documentation (2 hours)

### Task 3.3.1: CI Integration (45 min)

**Actions:**
- [ ] Update GitHub Actions workflow
- [ ] Add tool installation steps
- [ ] Add WIT validation step
- [ ] Test CI build

**CI Configuration:**
```yaml
- name: Install wit-bindgen
  run: cargo install wit-bindgen-cli --version 0.47.0
  
- name: Install wasm-tools
  run: cargo install wasm-tools --version 1.240.0
  
- name: Validate WIT
  run: wasm-tools component wit wit/
  
- name: Build
  run: cargo build --all-features
```

### Task 3.3.2: Performance Validation (30 min)

**Actions:**
- [ ] Measure binding generation time
- [ ] Measure incremental build time
- [ ] Compare to targets (see strategy doc)
- [ ] Document findings

**Expected:**
- Clean build: ~10s total
- Incremental (no WIT): ~2s
- Binding generation: ~2s

### Task 3.3.3: Documentation (45 min)

**Actions:**
- [ ] Update airssys-wasm README
- [ ] Document build requirements
- [ ] Document common issues
- [ ] Create developer guide

**README Additions:**
```markdown
## Building

### Prerequisites
- Rust 1.80+
- `wit-bindgen` 0.47.0
- `wasm-tools` 1.240.0

### Build Commands
\`\`\`bash
# Native build
cargo build

# WASM component
cargo build --target wasm32-wasip1 --release
\`\`\`

### Troubleshooting
See `docs/build/troubleshooting_guide.md`
```

---

## Success Criteria

### Must Complete

✅ build.rs executes without errors  
✅ Bindings generate for all 7 packages  
✅ Generated code compiles  
✅ Component builds for wasm32-wasip1  
✅ Incremental builds work correctly  
✅ CI pipeline validates WIT and builds  

### Quality Gates

✅ Zero compiler warnings in generated code  
✅ Error messages are clear and actionable  
✅ Build time within performance targets  
✅ Documentation complete and accurate  
✅ Cross-platform compatibility verified  

---

## Validation Checklist

Before marking Phase 3 complete:

- [ ] All Day 7 tasks completed
- [ ] All Day 8 tasks completed
- [ ] All Day 9 tasks completed
- [ ] All success criteria met
- [ ] All quality gates passed
- [ ] Documentation reviewed
- [ ] CI green on main branch
- [ ] Phase 3 completion report written

---

## Deliverables

1. ✅ `build.rs` - Production build script
2. ✅ `Cargo.toml` - Updated configuration
3. ✅ `src/generated/` - Generated bindings (in .gitignore)
4. ✅ `src/lib.rs` - Integration code
5. ✅ `.github/workflows/build.yml` - CI configuration
6. ✅ Updated README.md
7. ✅ Phase 3 completion report

---

## Handoff to Next Phase

**After Phase 3:**
- Build system fully functional
- Ready for actual implementation
- Component can be built and validated
- CI/CD pipeline operational

**Next Phase:** Component implementation (core functionality)

---

**Document Status:** ✅ Complete  
**Ready for:** Phase 3 Day 7 execution
