# WASM-TASK-013: Implementation Plans

## Plan References
- **ADR-WASM-025:** Clean-Slate Rebuild Architecture
- **ADR-WASM-026:** Implementation Roadmap (Phase 2)
- **KNOWLEDGE-WASM-037:** Rebuild Architecture Clean-Slate

## Implementation Actions

### Action 1: Rename actor/ directory to component/

**Objective:** Rename directory to align with WASM Component Model terminology

**Steps:**
1. Rename `src/actor/` to `src/component/`
2. Git will track this as a rename operation

**Verification:**
```bash
ls -la src/component/
# Should show component/ directory exists
```

### Action 2: Update module documentation in mod.rs

**Objective:** Update documentation to reflect component terminology

**Steps:**
1. Update `src/component/mod.rs` header documentation
2. Change "Actor Module" to "Component Module"
3. Update description to reference WASM components instead of actors

**Reference:** KNOWLEDGE-WASM-037 (lines on component terminology)

### Action 3: Update lib.rs module declaration

**Objective:** Update lib.rs to declare component module instead of actor

**Steps:**
1. Change `pub mod actor;` to `pub mod component;`
2. Update documentation comments referencing actor to component
3. Update dependency diagram in documentation

**File:** `src/lib.rs`

**Verification:**
```bash
cargo build -p airssys-wasm
cargo clippy -p airssys-wasm --all-targets -- -D warnings
```

## Verification Commands

Run after ALL actions complete:
```bash
# 1. Directory structure check
ls -la src/component/

# 2. Build check
cargo build -p airssys-wasm

# 3. Lint check
cargo clippy -p airssys-wasm --all-targets -- -D warnings

# 4. No actor references in modules
grep -rn "mod actor" src/
# Should return empty
```

## Success Criteria
- Directory renamed successfully
- Build passes with zero warnings
- No remaining references to actor module
