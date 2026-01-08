# WASM-TASK-019: Implementation Plans

## Plan References
- **ADR-WASM-028:** Core Module Structure (primary specification)
- **ADR-WASM-025:** Clean-Slate Rebuild Architecture
- **ADR-WASM-026:** Implementation Roadmap (Phase 3 - lines 109-123)
- **KNOWLEDGE-WASM-037:** Rebuild Architecture Clean-Slate

## Target Structure Reference

Per ADR-WASM-028:
```
core/messaging/
├── mod.rs           # Module declarations and re-exports
├── correlation.rs   # CorrelationId
├── payload.rs       # MessagePayload
└── traits.rs        # MessageRouter, CorrelationTracker traits
```

## Implementation Actions

> ⚠️ **DETAILED PLANS TO BE ADDED**
> 
> This is a skeleton plan. Detailed implementation actions will be added when work begins on this task.

### Action 1: Create core/messaging/payload.rs
**Objective:** Implement MessagePayload type
**Status:** Not started

### Action 2: Create core/messaging/correlation.rs
**Objective:** Implement CorrelationId types
**Status:** Not started

### Action 3: Create core/messaging/traits.rs
**Objective:** Implement MessageRouter and CorrelationTracker traits
**Status:** Not started

### Action 4: Create core/messaging/mod.rs
**Objective:** Module declarations and re-exports
**Status:** Not started

## Verification Commands

```bash
# 1. Build check
cargo build -p airssys-wasm

# 2. Lint check
cargo clippy -p airssys-wasm --all-targets -- -D warnings

# 3. Verify internal imports only
grep -rn "use crate::" src/core/messaging/
# Should only show core/ internal imports
```

## Success Criteria
- All types from ADR-WASM-028 implemented
- Build passes with zero warnings
- Proper dependency on core/component/ types
