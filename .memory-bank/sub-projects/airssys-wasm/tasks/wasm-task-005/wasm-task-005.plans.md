# WASM-TASK-005: Implementation Plans

## Plan References

- **ADR-WASM-027:** WIT Interface Design (Lines 215-293: capabilities.wit specification)
- **ADR-WASM-026:** Implementation Roadmap (Phase 1, Task 005)
- **KNOWLEDGE-WASM-013:** Core WIT Package Structure

## Implementation Actions

### Action 1: Create capabilities.wit with Interface Declaration

**Objective:** Create capability security model interface file

**Steps:**
1. Create file `wit/core/capabilities.wit`
2. Add package declaration: `package airssys:core@1.0.0;`
3. Add documentation comment
4. Add interface declaration: `interface capabilities { ... }`
5. Add `use types.{component-id};` import

**Reference:** ADR-WASM-027, lines 217-222

### Action 2: Implement Filesystem Permissions

**Objective:** Define filesystem access control types

**Steps:**
1. Add `filesystem-action` enum (read, write, delete, list-dir)
2. Add `filesystem-permission` record

**Reference:** ADR-WASM-027, lines 224-236

### Action 3: Implement Network Permissions

**Objective:** Define network access control types

**Steps:**
1. Add `network-action` enum (outbound, inbound)
2. Add `network-permission` record

**Reference:** ADR-WASM-027, lines 238-249

### Action 4: Implement Storage and Messaging Permissions

**Objective:** Define storage and messaging access control

**Steps:**
1. Add `storage-action` enum + `storage-permission` record
2. Add `messaging-action` enum + `messaging-permission` record

**Reference:** ADR-WASM-027, lines 251-275

### Action 5: Implement Permission Aggregation Types

**Objective:** Define complete permission set structures

**Steps:**
1. Add `requested-permissions` record (aggregates all permission types)
2. Add `capability-grant` record (grant result with expiry)

**Reference:** ADR-WASM-027, lines 278-291

## Verification Commands

```bash
# Validate WIT syntax
cd airssys-wasm
wasm-tools component wit wit/core/ && echo "✓ WIT validation passed"

# Check permissions defined
grep -c "permission" wit/core/capabilities.wit
```

## Success Criteria

- All verification commands pass
- File content matches ADR-WASM-027 lines 217-292
- WIT validation succeeds
