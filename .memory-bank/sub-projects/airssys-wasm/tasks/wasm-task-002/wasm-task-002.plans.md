# WASM-TASK-002: Implementation Plans

## Plan References

- **ADR-WASM-027:** WIT Interface Design (Section: WIT Directory Structure)
- **ADR-WASM-026:** Implementation Roadmap (Phase 1, Task 002)
- **KNOWLEDGE-WASM-037:** Clean Slate Architecture (Section: WIT Build Strategy)
- **ADR-WASM-025:** Clean-Slate Rebuild Architecture

## Implementation Actions

### Action 1: Create WIT Root Directory

**Objective:** Establish the base `wit/` directory for WIT interface definitions

**Steps:**
1. Navigate to `airssys-wasm/` root
2. Create `wit/` directory
3. Verify directory is at the correct location (per ADR-WASM-027)

**Reference:** ADR-WASM-027, lines 40-52 (WIT Directory Structure)

**Verification:**
```bash
test -d airssys-wasm/wit && echo "✓ wit/ directory exists"
```

### Action 2: Create Core Package Directory

**Objective:** Create the `wit/core/` package directory for `airssys:core@1.0.0`

**Steps:**
1. Create `wit/core/` subdirectory
2. This directory will contain all interface files (types.wit, errors.wit, etc.)
3. Verify structure matches Component Model package layout

**Reference:** ADR-WASM-027, line 42 (Package: airssys:core@1.0.0)

**Verification:**
```bash
test -d airssys-wasm/wit/core && echo "✓ wit/core/ directory exists"
```

### Action 3: Create Package Configuration (deps.toml)

**Objective:** Define package metadata for the WIT package

**Steps:**
1. Create `wit/deps.toml` file
2. Add package name: `airssys:core`
3. Add version: `1.0.0`
4. Content per ADR-WASM-027, lines 483-487

**File Content:**
```toml
[package]
name = "airssys:core"
version = "1.0.0"
```

**Reference:** ADR-WASM-027, lines 483-487 (deps.toml Package Configuration)

**Verification:**
```bash
test -f airssys-wasm/wit/deps.toml && echo "✓ deps.toml exists"
cat airssys-wasm/wit/deps.toml | grep -q "airssys:core" && echo "✓ Package name correct"
```

## Verification Commands

Run after ALL actions complete:

```bash
# 1. Verify directory structure
cd airssys-wasm
test -d wit && test -d wit/core && echo "✓ Directory structure complete"

# 2. Verify deps.toml
test -f wit/deps.toml && echo "✓ deps.toml exists"
grep -q "airssys:core" wit/deps.toml && echo "✓ Package name correct"
grep -q "1.0.0" wit/deps.toml && echo "✓ Version correct"

# 3. Show structure
tree wit/
```

## Success Criteria

- Directory structure matches ADR-WASM-027
- All verification commands pass
- Ready for WASM-TASK-003
