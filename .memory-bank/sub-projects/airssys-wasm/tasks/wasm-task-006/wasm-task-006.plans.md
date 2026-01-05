# WASM-TASK-006: Implementation Plans

## Plan References

- **ADR-WASM-027:** WIT Interface Design (Lines 297-338: component-lifecycle.wit specification)
- **ADR-WASM-026:** Implementation Roadmap (Phase 1, Task 006)
- **KNOWLEDGE-WASM-013:** Core WIT Package Structure

## Implementation Actions

### Action 1: Create component-lifecycle.wit with Interface Declaration

**Objective:** Create guest component contract interface

**Steps:**
1. Create file `wit/core/component-lifecycle.wit`
2. Add package declaration
3. Add documentation comment: "Guest-implemented interface - components MUST export this"
4. Add interface declaration: `interface component-lifecycle { ... }`
5. Add use statements for types and errors

**Reference:** ADR-WASM-027, lines 300-305

### Action 2: Implement Lifecycle Functions

**Objective:** Define component initialization and shutdown

**Steps:**
1. Add `initialize` function (takes component-config, returns result)
2. Add `shutdown` function (graceful cleanup)

**Reference:** ADR-WASM-027, lines 308, 325

### Action 3: Implement Message Handling Functions

**Objective:** Define message processing contract

**Steps:**
1. Add `handle-message` function (fire-and-forget pattern)
2. Add `handle-callback` function (request-response pattern)

**Reference:** ADR-WASM-027, lines 311-316

### Action 4: Implement Metadata and Health Functions

**Objective:** Define component introspection

**Steps:**
1. Add `metadata` function
2. Add `component-metadata` record
3. Add `health` function

**Reference:** ADR-WASM-027, lines 319-322, 328-336

## Verification Commands

```bash
# Validate WIT syntax
cd airssys-wasm
wasm-tools component wit wit/core/ && echo "✓ WIT validation passed"

# Check guest export functions (should have 6 functions)
grep -c "^    [a-z-]*: func" wit/core/component-lifecycle.wit
```

## Success Criteria

- All verification commands pass
- File content matches ADR-WASM-027 lines 297-337
- 6 lifecycle functions defined
- WIT validation succeeds
