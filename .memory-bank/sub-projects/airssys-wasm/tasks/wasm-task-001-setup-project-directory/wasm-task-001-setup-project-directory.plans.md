# WASM-TASK-001: Implementation Plans

## Plan References

### Architecture Decision Records (ADRs)
- **ADR-WASM-002:** WASM Runtime Engine Selection (mandates Wasmtime 24.0 with component-model)
- **ADR-WASM-011:** Module Structure Organization (defines four-module structure)
- **ADR-WASM-023:** Module Boundary Enforcement (MANDATORY - defines import hierarchy)

### Knowledge Documents
- **KNOWLEDGE-WASM-031:** Foundational Architecture (READ FIRST - defines what airssys-wasm is)
- **KNOWLEDGE-WASM-030:** Module Architecture Hard Requirements (MANDATORY - defines module responsibilities)
- **KNOWLEDGE-WASM-012:** Module Structure Architecture (detailed module organization)
- **KNOWLEDGE-WASM-001:** Component Framework Architecture

## Architecture Compliance (MANDATORY)

### Four-Module Structure (ADR-WASM-011, ADR-WASM-023)
```
airssys-wasm/src/
├── core/      # Foundation - shared types, imports NOTHING
├── security/  # Security logic - imports core/
├── runtime/   # WASM execution - imports core/, security/
└── actor/     # Actor integration - imports core/, security/, runtime/
```

### Dependency Hierarchy (ADR-WASM-023 - FORBIDDEN IMPORTS)
```
ALLOWED:
  ✅ actor/    → runtime/
  ✅ actor/    → security/
  ✅ actor/    → core/
  ✅ runtime/  → security/
  ✅ runtime/  → core/
  ✅ security/ → core/

FORBIDDEN (NEVER):
  ❌ runtime/  → actor/
  ❌ security/ → runtime/
  ❌ security/ → actor/
  ❌ core/     → ANY MODULE
```

## Implementation Actions

### Action 1: Create airssys-wasm/Cargo.toml

**Objective:** Define package and workspace integration

**Reference:**
- **ADR-WASM-002:** Wasmtime 24.0 with component-model feature
- **Workspace Cargo.toml:** Dependency versions
- **PROJECTS_STANDARD.md:** Dependency layering (Layer 1: AirsSys, Layer 2: Runtime, Layer 3: External)

**Steps:**
1. Create `airssys-wasm/Cargo.toml` with:
   - Package metadata (name, version, edition, authors, license, repository, rust-version)
   - Workspace dependency: `airssys-osl = { workspace = true }`
   - Workspace dependency: `airssys-rt = { workspace = true }`
   - External dependencies:
     - `wasmtime = { version = "24.0", features = ["component-model", "async", "cranelift"] }`
     - `wasmtime-wasi = { version = "24.0" }`
     - `wit-bindgen = { version = "0.47.0", default-features = false }`
     - Standard dependencies: tokio, serde, thiserror, uuid, dashmap, async-trait
2. Order dependencies per PROJECTS_STANDARD.md §5.1 (Layer 1 → Layer 2 → Layer 3)
3. Enable workspace Lints (inherits from workspace)

**Deliverable:**
- `airssys-wasm/Cargo.toml` with all dependencies

**Verification:**
```bash
cargo build -p airssys-wasm
```

**Expected Result:** Package compiles (empty lib.rs is OK)

---

### Action 2: Create Module Directory Structure

**Objective:** Establish four-module architecture per ADR-WASM-023

**Reference:**
- **ADR-WASM-011:** Module Structure Organization (four-module structure)
- **KNOWLEDGE-WASM-030:** Module Architecture Hard Requirements (module responsibilities)
- **PROJECTS_STANDARD.md §4.3:** mod.rs must contain ONLY declarations

**Steps:**
1. Create `airssys-wasm/src/` directory
2. Create module directories:
   - `airssys-wasm/src/core/`
   - `airssys-wasm/src/security/`
   - `airssys-wasm/src/runtime/`
   - `airssys-wasm/src/actor/`
3. Create `mod.rs` in each module with ONLY module declarations:
   - NO implementation code in mod.rs
   - Re-exports where appropriate
4. Create placeholder `lib.rs` in each module initially (will be replaced with actual code later)

**Deliverables:**
- Four module directories with mod.rs files
- mod.rs files follow §4.3 compliance (declarations only)

**Verification:**
```bash
# Check directories exist
ls -la airssys-wasm/src/{core,security,runtime,actor}/

# Verify mod.rs are declarations-only
head -5 airssys-wasm/src/core/mod.rs
head -5 airssys-wasm/src/security/mod.rs
head -5 airssys-wasm/src/runtime/mod.rs
head -5 airssys-wasm/src/actor/mod.rs
```

**Expected Result:** All mod.rs files contain only `pub mod` declarations

---

### Action 3: Create Main lib.rs Entry Point

**Objective:** Define library structure and module exports

**Reference:**
- **ADR-WASM-011:** Module Structure Organization (prelude pattern)
- **KNOWLEDGE-WASM-012:** Module Structure Architecture (public API surface)

**Steps:**
1. Create `airssys-wasm/src/lib.rs` with:
   - Module declarations for all four modules
   - Public re-exports of key types
   - Prelude module with common imports ( ergonomic API)
2. Follow three-layer import organization (PROJECTS_STANDARD.md §2.1):
   - Layer 1: Standard library imports
   - Layer 2: External crate imports
   - Layer 3: Internal module imports

**Deliverable:**
- `airssys-wasm/src/lib.rs` with module declarations and prelude

**Verification:**
```bash
cargo build -p airssys-wasm
```

**Expected Result:** Library compiles with all modules exported

---

### Action 4: Create Test Fixture Directory

**Objective:** Prepare directory for WASM test fixtures

**Reference:**
- **MEMORY-BANK Instructions:** Fixture Management for Testing
- **KNOWLEDGE-WASM-018:** Component Definitions (fixture requirements)

**Steps:**
1. Create `airssys-wasm/tests/` directory
2. Create `airssys-wasm/tests/fixtures/` directory
3. Create `airssys-wasm/tests/fixtures/README.md` documenting:
   - What each fixture is
   - Why it exists
   - How it was generated

**Deliverables:**
- `airssys-wasm/tests/fixtures/` directory structure
- README.md for fixture documentation

**Verification:**
```bash
ls -la airssys-wasm/tests/fixtures/
cat airssys-wasm/tests/fixtures/README.md
```

**Expected Result:** Fixture directory ready for future test fixture creation

---

### Action 5: Create WIT Directory

**Objective:** Prepare directory for WebAssembly Interface Types definitions

**Reference:**
- **ADR-WASM-015:** WIT Package Structure Organization
- **KNOWLEDGE-WASM-004:** WIT Management Architecture

**Steps:**
1. Create `airssys-wasm/wit/` directory
2. Create README.md explaining WIT package structure:
   - 7-package structure (4 core + 3 extension)
   - Package naming pattern: `airssys:{directory}-{type}@{version}`

**Deliverables:**
- `airssys-wasm/wit/` directory structure
- README.md with package organization documentation

**Verification:**
```bash
ls -la airssys-wasm/wit/
cat airssys-wasm/wit/README.md
```

**Expected Result:** WIT directory ready for package definitions

---

## Verification Commands

Run after ALL actions complete:

### 1. Build Check
```bash
cargo build -p airssys-wasm
```
**Expected:** Clean build with no errors

### 2. Module Architecture Verification (ADR-WASM-023 - MANDATORY)
```bash
# ALL must return NOTHING (no output = clean architecture)

# Check 1: core/ imports NOTHING
grep -rn "use crate::" airssys-wasm/src/core/

# Check 2: security/ imports only core/
grep -rn "use crate::runtime" airssys-wasm/src/security/
grep -rn "use crate::actor" airssys-wasm/src/security/

# Check 3: runtime/ imports only core/, security/
grep -rn "use crate::actor" airssys-wasm/src/runtime/

# Check 4: Verify directory structure
ls -d airssys-wasm/src/
```
**Expected:** All grep commands return no output (clean architecture)

### 3. Lint Check
```bash
cargo clippy -p airssys-wasm --all-targets --all-features -- -D warnings
```
**Expected:** Zero warnings

### 4. Directory Structure Verification
```bash
tree -L 2 airssys-wasm/
# OR
find airssys-wasm/ -maxdepth 2 -type d | sort
```
**Expected Output:**
```
airssys-wasm/
├── Cargo.toml
├── src/
│   ├── actor/
│   ├── core/
│   ├── lib.rs
│   ├── runtime/
│   └── security/
├── tests/
│   └── fixtures/
└── wit/
```

## Success Criteria

All of the following MUST be true:

- [ ] **Cargo.toml created** with all workspace dependencies
- [ ] **Module structure created** with core/, security/, runtime/, actor/
- [ ] **lib.rs created** with module declarations and prelude
- [ ] **Tests/fixtures/ directory created** with README.md
- [ ] **WIT directory created** with README.md
- [ ] **Build succeeds**: `cargo build -p airssys-wasm` completes cleanly
- [ ] **Architecture compliant**: All module boundary verification commands return no output
- [ ] **Zero warnings**: `cargo clippy -- -D warnings` passes
- [ ] **§4.3 compliant**: All mod.rs files contain only declarations (no implementation code)

## Risk Mitigation

### Potential Issues and Solutions:

1. **Issue:** Cargo.toml has wrong dependency versions
   - **Solution:** Reference workspace Cargo.toml for exact versions

2. **Issue:** Module structure violates ADR-WASM-023
   - **Solution:** Run architecture verification commands immediately

3. **Issue:** mod.rs files contain implementation (violates §4.3)
   - **Solution:** Move implementation to separate files within modules

4. **Issue:** Wrong Wasmtime version or features
   - **Solution:** Reference ADR-WASM-002 (Wasmtime 24.0 with component-model)

## Notes

- This is a SETUP task - no actual implementation code is written yet
- All subsequent tasks will depend on this structure being correct
- Architecture violations here would cascade to ALL future tasks
- ADR-WASM-023 violations would cause circular dependency issues immediately

**CRITICAL:** This task MUST be audited and verified before proceeding to any code implementation.
