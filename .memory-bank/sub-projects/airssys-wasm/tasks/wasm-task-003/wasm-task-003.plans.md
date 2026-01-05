# WASM-TASK-003: Implementation Plans

## Plan References

- **ADR-WASM-027:** WIT Interface Design (Lines 58-144: types.wit specification)
- **ADR-WASM-026:** Implementation Roadmap (Phase 1, Task 003)
- **KNOWLEDGE-WASM-013:** Core WIT Package Structure
- **KNOWLEDGE-WASM-037:** Clean Slate Architecture

## Implementation Actions

### Action 1: Create types.wit File with Package and Interface Declaration

**Objective:** Create the foundation types interface file

**Steps:**
1. Create file `wit/core/types.wit`
2. Add package declaration: `package airssys:core@1.0.0;`
3. Add interface declaration: `interface types { ... }`
4. Add documentation comment for the interface

**Reference:** ADR-WASM-027, lines 60-63

**Content:**
```wit
package airssys:core@1.0.0;

/// Foundation types - source of truth for all interfaces
interface types {
    // Types will be added in next actions
}
```

### Action 2: Implement Core Identity and Handle Types

**Objective:** Define component identification types

**Steps:**
1. Add `component-id` record (namespace, name, instance)
2. Add `component-handle` type (u64 opaque reference)
3. Add `correlation-id` and `request-id` string types

**Reference:** ADR-WASM-027, lines 66-79

**Types to add:**
- `record component-id`
- `type component-handle`
- `type correlation-id`
- `type request-id`

### Action 3: Implement Message and Payload Types

**Objective:** Define messaging data structures

**Steps:**
1. Add `message-payload` (list<u8>)
2. Add `timestamp` record
3. Add `message-metadata` record
4. Add `component-message` record

**Reference:** ADR-WASM-027, lines 82-103

### Action 4: Implement Configuration and Resource Types

**Objective:** Define component configuration and limits

**Steps:**
1. Add `resource-limits` record
2. Add `component-config` record

**Reference:** ADR-WASM-027, lines 106-117

### Action 5: Implement Enums

**Objective:** Define standard enumerations

**Steps:**
1. Add `log-level` enum
2. Add `health-status` enum
3. Add `execution-status` enum

**Reference:** ADR-WASM-027, lines 120-142

## Verification Commands

```bash
# 1. File exists
test -f airssys-wasm/wit/core/types.wit && echo "✓ types.wit exists"

# 2. Validate WIT syntax
cd airssys-wasm
wasm-tools component wit wit/core/ && echo "✓ WIT validation passed"

# 3. Check package declaration
grep -q "package airssys:core@1.0.0" wit/core/types.wit && echo "✓ Package correct"

# 4. Check interface declaration
grep -q "interface types" wit/core/types.wit && echo "✓ Interface declared"
```

## Success Criteria

- All verification commands pass
- File content exactly matches ADR-WASM-027 lines 60-143
- WIT validation succeeds
