# WASM-TASK-007: Implementation Plans

## Plan References

- **ADR-WASM-027:** WIT Interface Design (Lines 342-375: host-messaging.wit specification)
- **ADR-WASM-026:** Implementation Roadmap (Phase 1, Task 007)

## Implementation Actions

### Action 1: Create host-messaging.wit with Interface Declaration

**Steps:**
1. Create file `wit/core/host-messaging.wit`
2. Add package and interface declarations
3. Add use statements for types and errors

**Reference:** ADR-WASM-027, lines 345-350

### Action 2: Implement Messaging Functions

**Steps:**
1. Add `send` function (fire-and-forget)
2. Add `request` function (request-response)
3. Add `cancel-request` function
4. Add `broadcast` function
5. Add `self-id` function

**Reference:** ADR-WASM-027, lines 353-373

## Verification Commands

```bash
cd airssys-wasm
wasm-tools component wit wit/core/ && echo "✓ WIT validation passed"
```

## Success Criteria

- File content matches ADR-WASM-027 lines 342-374
- 5 messaging functions defined
