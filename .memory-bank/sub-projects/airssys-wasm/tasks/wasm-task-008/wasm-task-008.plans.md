# WASM-TASK-008: Implementation Plans

## Plan References

- **ADR-WASM-027:** WIT Interface Design (Lines 379-419: host-services.wit specification)
- **ADR-WASM-026:** Implementation Roadmap (Phase 1, Task 008)

## Implementation Actions

### Action 1: Create host-services.wit with Interface Declaration

**Steps:**
1. Create file `wit/core/host-services.wit`
2. Add package and interface declarations
3. Add use statements

**Reference:** ADR-WASM-027, lines 382-387

### Action 2: Implement Service Functions

**Steps:**
1. Add `log` function
2. Add `current-time` and `current-time-millis` functions
3. Add `sleep-millis` function
4. Add `list-components` function
5. Add `get-component-metadata` function
6. Add `component-info` record

**Reference:** ADR-WASM-027, lines 390-417

## Verification Commands

```bash
cd airssys-wasm
wasm-tools component wit wit/core/ && echo "✓ WIT validation passed"
```

## Success Criteria

- File content matches ADR-WASM-027 lines 379-418
