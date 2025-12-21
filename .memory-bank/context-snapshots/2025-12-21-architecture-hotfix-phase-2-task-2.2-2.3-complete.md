# Context Snapshot: Architecture Hotfix Phase 2 - Task 2.2 & 2.3 Complete

**Date:** 2025-12-21  
**Session Type:** Implementation  
**Task:** WASM-TASK-006-HOTFIX Phase 2 - Duplicate Runtime Remediation

## Session Summary

Implemented Task 2.2 (Add WasmEngine Injection) and Task 2.3 (Rewrite Child::start()) as part of the Duplicate Runtime Remediation hotfix.

## Key Achievements

### 1. Added Component Model Fields to ComponentActor

**New fields in `ComponentActor` struct:**
- `component_engine: Option<Arc<WasmEngine>>` - Shared WASM engine (Component Model API)
- `component_handle: Option<ComponentHandle>` - Handle to loaded component

**New methods:**
- `with_component_engine(engine)` - Builder pattern to inject WasmEngine
- `component_engine()` - Get reference to engine
- `component_handle()` - Get reference to component handle
- `set_component_handle(handle)` - Set component handle (internal)
- `uses_component_model()` - Check if Component Model is configured

### 2. Added Component Model Path in Child::start()

**Implementation strategy:** Incremental migration (both paths work)

**When `component_engine` is set:**
1. Uses `WasmEngine::load_component()` (Component Model API - CORRECT)
2. Stores handle in `component_handle` field
3. Logs: "Using Component Model API (correct architecture per ADR-WASM-002)"

**When `component_engine` is NOT set:**
1. Uses legacy path (wasmtime::Module - DEPRECATED)
2. Logs warning: "Using LEGACY core WASM API (deprecated - migrate to Component Model)"
3. Maintains backward compatibility during migration

### 3. Added Unit Tests

**10 new tests added:**
- `test_component_model_engine_not_set_by_default`
- `test_with_component_engine_builder`
- `test_component_handle_not_loaded_by_default`
- `test_set_component_handle`
- `test_uses_component_model_reflects_engine`
- `test_legacy_and_component_model_coexistence`
- `test_legacy_path_used_without_engine`
- `test_uses_component_model_detection`
- `test_component_engine_accessor`
- `test_component_handle_accessors`

## Files Modified

| File | Changes |
|------|---------|
| `src/actor/component/component_actor.rs` | Added imports, fields, accessors, tests |
| `src/actor/component/child_impl.rs` | Added Component Model path, tests |

## Verification Results

```
cargo test --lib
# Result: 962 tests passing (10 new tests added)

cargo clippy --lib -- -D warnings
# Result: 0 warnings
```

## Technical Decisions

### 1. Incremental Migration Approach
- **Decision:** Keep legacy code alongside new Component Model code
- **Rationale:** Minimizes risk, allows gradual migration of callers
- **Trade-off:** More code temporarily, but safer migration path

### 2. Optional Engine Field
- **Decision:** `Option<Arc<WasmEngine>>` instead of required parameter
- **Rationale:** Preserves backward compatibility with existing `new()` calls
- **Migration path:** Callers can opt-in to Component Model via `with_component_engine()`

### 3. Deprecation Logging
- **Decision:** Log warning when legacy path is used
- **Rationale:** Alerts developers to migrate without breaking their code
- **Message:** Clearly states "deprecated" and "migrate to Component Model"

## Remaining Work (Phase 2)

| Task | Status | Description |
|------|--------|-------------|
| 2.1 | Deferred | Delete workaround code (after full migration) |
| 2.2 | ✅ Complete | Add WasmEngine injection |
| 2.3 | ✅ Complete | Rewrite Child::start() |
| 2.4 | Not Started | Rewrite Actor::handle() |
| 2.5 | Not Started | Extend WasmEngine if needed |
| 2.6 | Not Started | Update all tests |

## Architecture Impact

**Before (WRONG):**
```
ComponentActor
  └── WasmRuntime (local)
        └── wasmtime::Engine (core WASM API)
              └── wasmtime::Module (no Component Model)
```

**After (CORRECT - when engine configured):**
```
ComponentActor
  └── component_engine: Arc<WasmEngine>
        └── wasmtime::component::Component (Component Model API)
              └── WIT interfaces functional
              └── Generated bindings used
```

## Next Steps

1. **Task 2.4:** Rewrite Actor::handle() to use Component Model for message handling
2. **Task 2.5:** Add `call_handle_message()` method to WasmEngine if not present
3. **Task 2.6:** Update integration tests to use Component Model path
4. **Post-migration:** Delete legacy workaround code (Task 2.1)

## References

- ADR-WASM-002: WASM Runtime Engine Selection (mandates Component Model)
- ADR-WASM-021: Duplicate Runtime Remediation
- KNOWLEDGE-WASM-027: Duplicate WASM Runtime - Fatal Architecture Violation
- task-006-architecture-remediation-phase-2-duplicate-runtime.md
