# WASM-TASK-012: Implementation Plans

## Plan References

- **ADR-WASM-027:** WIT Interface Design (Lines 506-515: wit-bindgen Integration)
- **ADR-WASM-026:** Implementation Roadmap (Phase 1, Task 012)
- **KNOWLEDGE-WASM-037:** Clean Slate Architecture (Lines 265-273: WIT Build Strategy)

## Implementation Actions

### Action 1: Add wit-bindgen Dependency

**Objective:** Add wit-bindgen to Cargo.toml

**Steps:**
1. Open `airssys-wasm/Cargo.toml`
2. Add wit-bindgen dependency (check workspace version or use 0.36+)
3. Ensure wasmtime dependency is compatible

**Reference:** ADR-WASM-027, line 507

### Action 2: Add Macro Invocation

**Objective:** Generate Rust bindings from WIT

**Steps:**
1. In appropriate module (likely `src/lib.rs` or dedicated module)
2. Add macro invocation per ADR-WASM-027:
   ```rust
   wit_bindgen::generate!({
       world: "component",
       path: "wit/core",
   });
   ```

**Reference:** ADR-WASM-027, lines 510-514; KNOWLEDGE-WASM-037, lines 266-272

**CRITICAL:** Do NOT use build.rs (KNOWLEDGE-WASM-037, line 265)

### Action 3: Verify Bindings Generation

**Objective:** Ensure bindings compile

**Steps:**
1. Run `cargo build -p airssys-wasm`
2. Verify no compilation errors
3. Check that generated types are accessible

**Verification:**
```bash
cd airssys-wasm
cargo build -p airssys-wasm && echo "✓ Bindings generated"
cargo clippy -p airssys-wasm -- -D warnings && echo "✓ No warnings"
```

### Action 4: Document Generated Types

**Objective:** Create documentation for using generated bindings

**Steps:**
1. Document location of generated types
2. Add usage examples in comments
3. Prepare for Phase 2 module implementation

## Verification Commands

```bash
cd airssys-wasm

# 1. Build check
cargo build -p airssys-wasm && echo "✓ Build successful"

# 2. Lint check
cargo clippy -p airssys-wasm --all-targets -- -D warnings && echo "✓ No warnings"

# 3. Check macro is present
grep -q "wit_bindgen::generate" src/lib.rs && echo "✓ Macro invocation present"
```

## Success Criteria

- All verification commands pass
- Bindings generated successfully
- Ready for Phase 2 (Project Restructuring)
