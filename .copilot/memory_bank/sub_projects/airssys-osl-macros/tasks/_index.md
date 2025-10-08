# Task Index

## Active Tasks

### MACROS-TASK-001: Foundation Setup and Workspace Integration
**Status:** In Progress  
**Priority:** Critical  
**File:** [MACROS-TASK-001-foundation-setup.md](./MACROS-TASK-001-foundation-setup.md)

## Pending Tasks

### MACROS-TASK-002: Implement #[executor] Macro
**Status:** Pending (Blocked by MACROS-TASK-001)  
**Priority:** High  
**File:** [MACROS-TASK-002-executor-macro.md](./MACROS-TASK-002-executor-macro.md)

### MACROS-TASK-003: Integration with airssys-osl
**Status:** Pending (Blocked by MACROS-TASK-002)  
**Priority:** High  
**File:** Not yet created

## Planned Tasks

### MACROS-TASK-004: #[operation] Derive Macro (Future)
**Status:** Planned  
**Priority:** Medium  
**File:** Not yet created

### MACROS-TASK-005: #[middleware] Macro (Maybe)
**Status:** Maybe  
**Priority:** Low  
**File:** Not yet created

## Completed Tasks

None yet - project just started

## Task Dependencies

```mermaid
graph TD
    T001[MACROS-TASK-001: Foundation Setup] --> T002[MACROS-TASK-002: #executor Macro]
    T002 --> T003[MACROS-TASK-003: Integration]
    T003 --> T004[MACROS-TASK-004: #operation Macro]
    T003 -.-> T005[MACROS-TASK-005: #middleware Macro]
```

## Task Status Summary

| Task ID | Name | Status | Priority | Effort | Completion |
|---------|------|--------|----------|--------|------------|
| MACROS-TASK-001 | Foundation Setup | In Progress | Critical | 4h | 50% |
| MACROS-TASK-002 | #[executor] Macro | Pending | High | 2-3w | 0% |
| MACROS-TASK-003 | Integration | Pending | High | 1w | 0% |
| MACROS-TASK-004 | #[operation] Macro | Planned | Medium | 1-2w | 0% |
| MACROS-TASK-005 | #[middleware] Macro | Maybe | Low | 1-2w | 0% |

## Related Tasks in Other Sub-Projects

### airssys-osl
- **OSL-TASK-009**: Remove Framework and Add Helpers (Related - uses macro patterns)
- **OSL-TASK-008**: Platform Executors (Related - provides executors to test macros)
