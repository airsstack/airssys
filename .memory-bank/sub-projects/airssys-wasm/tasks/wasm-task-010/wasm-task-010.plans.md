# WASM-TASK-010: Implementation Plans

## Plan References

- **ADR-WASM-027:** WIT Interface Design (Lines 462-477: world.wit specification)
- **ADR-WASM-026:** Implementation Roadmap (Phase 1, Task 010)

## Implementation Actions

### Action 1: Create world.wit with World Definition

**Objective:** Define the component world tying all interfaces together

**Steps:**
1. Create file `wit/core/world.wit`
2. Add package declaration: `package airssys:core@1.0.0;`
3. Add documentation comment: "The main world that guest components implement"
4. Define world: `world component { ... }`

**Reference:** ADR-WASM-027, lines 465-468

### Action 2: Add Host-Provided Imports

**Objective:** Specify interfaces the host provides to guests

**Steps:**
1. Add `import host-messaging;`
2. Add `import host-services;`
3. Add `import storage;`

**Reference:** ADR-WASM-027, lines 470-472

### Action 3: Add Guest Export

**Objective:** Specify interface guests must implement

**Steps:**
1. Add `export component-lifecycle;`

**Reference:** ADR-WASM-027, line 475

## Verification Commands

```bash
# Validate complete WIT package
cd airssys-wasm
wasm-tools component wit wit/core/ && echo "✓ Complete WIT package validated"

# Check world definition
grep -q "world component" wit/core/world.wit && echo "✓ World defined"
```

## Success Criteria

- File content matches ADR-WASM-027 lines 462-477
- WIT package validation succeeds
