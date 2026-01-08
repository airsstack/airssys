# WASM-TASK-017: Implementation Plans

## Plan References
- **ADR-WASM-028:** Core Module Structure (PRIMARY specification for all types)
- **ADR-WASM-025:** Clean-Slate Rebuild Architecture (six-module architecture, layer-organized core/)
- **ADR-WASM-023:** Module Boundary Enforcement (core/ imports NOTHING)
- **ADR-WASM-026:** Implementation Roadmap (Phase 3 - lines 109-123)
- **KNOWLEDGE-WASM-037:** Rebuild Architecture - Clean Slate Design (layer-organized core/)
- **KNOWLEDGE-WASM-030:** Module Architecture Hard Requirements (module responsibilities)
- **KNOWLEDGE-WASM-033:** AI Fatal Mistakes - Lessons Learned (verification requirements)

**System Patterns:**
- Layer 1 foundation: core/ is the foundation that ALL other modules depend on
- Dependency Inversion: core/ provides abstractions that other modules depend on
- Layer-organized structure: core/component/ contains abstractions for component/ module

**PROJECTS_STANDARD.md Compliance:**
- §2.1 (3-Layer Imports): All files will organize imports (std only for core/)
- §2.2 (No FQN): Types will be imported and used by simple name
- §4.3 (Module Architecture): mod.rs files will only contain declarations
- §6.1 (YAGNI Principles): Only implement what's specified in ADR-WASM-028
- §6.2 (Avoid `dyn`): Use concrete types and trait bounds instead of dyn
- §6.4 (Quality Gates): Zero warnings, comprehensive unit tests

**Rust Guidelines Applied:**
- M-DESIGN-FOR-AI: Idiomatic APIs, thorough docs, testable code
- M-MODULE-DOCS: Module documentation with examples for all public items
- M-ERRORS-CANONICAL-STRUCTS: Error types will follow canonical structure (when needed)
- M-STATIC-VERIFICATION: All lints enabled, clippy with -D warnings
- M-FEATURES-ADDITIVE: Types will not break existing code

**Documentation Standards:**
- Diátaxis Type: Reference documentation for types and APIs
- Quality: Technical language, no marketing hyperbole per documentation-quality-standards.md
- Compliance: Standards Compliance Checklist will be added to task file

---

## Target Structure Reference

Per ADR-WASM-028:
```
core/component/
├── mod.rs           # Module declarations and re-exports
├── id.rs            # ComponentId
├── handle.rs        # ComponentHandle
├── message.rs       # ComponentMessage, MessageMetadata
└── traits.rs        # ComponentLifecycle trait
```

---

## Related Documentation

### Knowledge Documents
- **KNOWLEDGE-WASM-038:** Component Module Responsibility and Architecture (two-layer distinction, core/component/ vs component/)
- **KNOWLEDGE-WASM-037:** Rebuild Architecture - Clean Slate Design
- **KNOWLEDGE-WASM-031:** Foundational Architecture

### Purpose of This Knowledge

**KNOWLEDGE-WASM-038** provides critical context for this task:

1. **Core Module Responsibility (Layer 1):** This task creates `core/component/` which contains ONLY data structures and trait abstractions - no business logic, no actor integration, no external dependencies.

2. **Component Module Responsibility (Layer 3A):** Future tasks will implement `component/` module that integrates with airssys-rt (Actor, Child, ActorSystem) and uses the types created in this task.

3. **Dependency Inversion Pattern:** The `component/` module will depend on `core/component/` types but receive concrete RuntimeEngine and SecurityContext implementations via dependency injection from `system/` layer.

**Key Takeaway for Implementation:**
- ✅ Create data structures (ComponentId, ComponentHandle, ComponentMessage)
- ✅ Create trait definitions (ComponentLifecycle)
- ✅ Add unit tests for type validation
- ❌ NO business logic
- ❌ NO actor integration (that's Layer 3A)
- ❌ NO external imports (only std)
## Implementation Actions

### Action 1: Create core/component/id.rs with ComponentId
**Objective:** Implement ComponentId type for unique component identification

**Steps:**
1. Create file `airssys-wasm/src/core/component/id.rs`
2. Implement ComponentId struct with namespace, name, instance fields (per ADR-WASM-028 lines 81-112)
3. Add `new()` constructor accepting Into<String> for each field
4. Add `to_string_id()` method returning formatted identifier
5. Implement Display trait for formatted output
6. Add Debug, Clone, PartialEq, Eq, Hash derives (per ADR-WASM-028)
7. Add rustdoc documentation with examples

**Deliverables:**
- File: `airssys-wasm/src/core/component/id.rs` with ComponentId implementation

**ADR Constraints:**
- ADR-WASM-028 requires: ComponentId with namespace, name, instance fields
- ADR-WASM-023 requires: core/ imports ONLY std
- ADR-WASM-025 requires: Type must be usable by all modules

**PROJECTS_STANDARD.md Compliance:**
- §2.1: Imports will be `use std::fmt;` (std only)
- §2.2: Type annotations will use imported names (ComponentId, not `self::ComponentId`)
- §6.2: No dyn trait objects used

**Rust Guidelines:**
- M-DESIGN-FOR-AI: Into<String> for flexibility, idiomatic API
- M-MODULE-DOCS: Public API will have rustdoc with examples
- M-STATIC-VERIFICATION: Derives will be validated by clippy

**Documentation:**
- Diátaxis type: Reference documentation for ComponentId API
- Quality: Technical language describing purpose and usage
- Compliance: Examples showing ComponentId creation and formatting

**Unit Testing Plan:**
- Test 1: Verify ComponentId creation with valid strings
- Test 2: Verify to_string_id() format: "{namespace}/{name}/{instance}"
- Test 3: Verify Display trait output matches to_string_id()
- Test 4: Verify PartialEq/Eq comparisons work correctly
- Test 5: Verify Hash trait enables HashMap usage
- Test 6: Test edge cases: empty strings, special characters

**Verification:**
```bash
# File exists
ls -la airssys-wasm/src/core/component/id.rs

# Syntax check
cargo check -p airssys-wasm

# No external imports
grep -n "^use " airssys-wasm/src/core/component/id.rs | grep -v "^use std::"
# Expected: Only std imports
```

---

### Action 2: Create core/component/handle.rs with ComponentHandle
**Objective:** Implement ComponentHandle type for opaque component references

**Steps:**
1. Create file `airssys-wasm/src/core/component/handle.rs`
2. Import ComponentId from super::id (std-only import)
3. Implement ComponentHandle struct with id and handle_id fields (per ADR-WASM-028 lines 115-141)
4. Add private fields (id: ComponentId, handle_id: u64)
5. Add `new()` constructor taking ComponentId and u64
6. Add `id()` getter returning &ComponentId
7. Add `handle_id()` getter returning u64
8. Add Debug, Clone derives
9. Add rustdoc documentation

**Deliverables:**
- File: `airssys-wasm/src/core/component/handle.rs` with ComponentHandle implementation

**ADR Constraints:**
- ADR-WASM-028 requires: ComponentHandle with private id and handle_id fields
- ADR-WASM-023 requires: imports ONLY from std and sibling module (id.rs)

**PROJECTS_STANDARD.md Compliance:**
- §2.1: Imports will be `use super::id::ComponentId;` (internal only)
- §2.2: Type annotations will use ComponentId (not `super::id::ComponentId`)
- §6.2: No dyn trait objects

**Rust Guidelines:**
- M-DESIGN-FOR-AI: Opaque handle pattern (private fields, getters)
- M-MODULE-DOCS: Rustdoc explaining opaque handle semantics

**Documentation:**
- Diátaxis type: Reference documentation for ComponentHandle API
- Quality: Technical explanation of opaque handle pattern

**Unit Testing Plan:**
- Test 1: Verify ComponentHandle creation with valid ComponentId and u64
- Test 2: Verify id() getter returns reference to original ComponentId
- Test 3: Verify handle_id() getter returns original u64 value
- Test 4: Verify Clone trait creates independent copy
- Test 5: Test with multiple handles to different components

**Verification:**
```bash
# File exists
ls -la airssys-wasm/src/core/component/handle.rs

# Syntax check
cargo check -p airssys-wasm

# No std以外 imports
grep -n "^use " airssys-wasm/src/core/component/handle.rs
# Expected: Only super::id::ComponentId (no external crates)
```

---

### Action 3: Create core/component/message.rs with MessageMetadata and ComponentMessage
**Objective:** Implement message types for component communication

**Steps:**
1. Create file `airssys-wasm/src/core/component/message.rs`
2. Add stub imports for MessagePayload (will be created in later task)
3. Define MessageMetadata struct (per ADR-WASM-028 lines 146-169):
   - correlation_id: Option<String>
   - reply_to: Option<ComponentId>
   - timestamp_ms: u64
   - content_type: Option<String>
4. Implement Default trait for MessageMetadata
5. Define ComponentMessage struct (per ADR-WASM-028 lines 170-177):
   - sender: ComponentId
   - payload: MessagePayload (placeholder for now)
   - metadata: MessageMetadata
6. Add Debug, Clone derives
7. Add rustdoc documentation

**Note:** MessagePayload will be stubbed as `Vec<u8>` placeholder for now. Real MessagePayload will be created in core/messaging/payload.rs (future task).

**Deliverables:**
- File: `airssys-wasm/src/core/component/message.rs` with MessageMetadata and ComponentMessage

**ADR Constraints:**
- ADR-WASM-028 requires: MessageMetadata and ComponentMessage with specified fields
- ADR-WASM-023 requires: imports ONLY from std and sibling modules

**PROJECTS_STANDARD.md Compliance:**
- §2.1: Imports will be std and sibling modules only
- §2.2: No FQN in type annotations
- §6.1: YAGNI - only implement what's in ADR-WASM-028 (placeholder for MessagePayload)

**Rust Guidelines:**
- M-DESIGN-FOR-AI: Default trait for convenience, Option types for optional fields
- M-MODULE-DOCS: Rustdoc explaining message envelope structure

**Documentation:**
- Diátaxis type: Reference documentation for message types
- Quality: Technical description of message envelope fields

**Unit Testing Plan:**
- Test 1: Verify MessageMetadata::default() initializes all fields correctly
- Test 2: Verify ComponentMessage creation with all fields
- Test 3: Verify Default implementation sets correlation_id to None
- Test 4: Verify Default implementation sets reply_to to None
- Test 5: Verify Default implementation sets timestamp_ms to 0
- Test 6: Test message with various metadata combinations
- Test 7: Test message serialization/deserialization (if applicable)

**Verification:**
```bash
# File exists
ls -la airssys-wasm/src/core/component/message.rs

# Syntax check
cargo check -p airssys-wasm

# Verify MessagePayload placeholder
grep -n "MessagePayload" airssys-wasm/src/core/component/message.rs
# Expected: Placeholder comment or type alias
```

---

### Action 4: Create core/component/traits.rs with ComponentLifecycle trait
**Objective:** Define ComponentLifecycle trait for component lifecycle management

**Steps:**
1. Create file `airssys-wasm/src/core/component/traits.rs`
2. Import ComponentHandle and ComponentMessage from sibling modules
3. Add stub import for WasmError (will be created in core/errors/wasm.rs - future task)
4. Define ComponentLifecycle trait (per ADR-WASM-028 lines 183-199):
   - `initialize(&mut self) -> Result<(), WasmError>`
   - `shutdown(&mut self) -> Result<(), WasmError>`
   - `health_check(&self) -> bool`
5. Add Send + Sync trait bounds (per ADR-WASM-028)
6. Add rustdoc documentation with examples

**Note:** WasmError will be stubbed as placeholder for now. Real WasmError will be created in core/errors/wasm.rs (future task).

**Deliverables:**
- File: `airssys-wasm/src/core/component/traits.rs` with ComponentLifecycle trait

**ADR Constraints:**
- ADR-WASM-028 requires: ComponentLifecycle trait with three methods
- ADR-WASM-023 requires: imports ONLY from std and sibling modules

**PROJECTS_STANDARD.md Compliance:**
- §2.1: Imports will be std and sibling modules only
- §6.2: Trait bounds (Send + Sync) instead of dyn
- §6.4: All methods must be testable

**Rust Guidelines:**
- M-DESIGN-FOR-AI: Trait with Send + Sync bounds for thread safety
- M-MODULE-DOCS: Rustdoc explaining lifecycle contract

**Documentation:**
- Diátaxis type: Reference documentation for ComponentLifecycle trait
- Quality: Technical description of lifecycle methods

**Unit Testing Plan:**
- Test 1: Create mock struct implementing ComponentLifecycle
- Test 2: Verify initialize() is called correctly
- Test 3: Verify shutdown() is called correctly
- Test 4: Verify health_check() returns boolean
- Test 5: Test trait object creation (Box<dyn ComponentLifecycle>)
- Test 6: Test that trait is Send + Sync

**Verification:**
```bash
# File exists
ls -la airssys-wasm/src/core/component/traits.rs

# Syntax check
cargo check -p airssys-wasm

# Verify trait bounds
grep -n "Send + Sync" airssys-wasm/src/core/component/traits.rs
# Expected: Trait has Send + Sync bounds
```

---

### Action 5: Create core/component/mod.rs with module declarations
**Objective:** Create module file for component submodule

**Steps:**
1. Create file `airssys-wasm/src/core/component/mod.rs`
2. Add module declarations (per §4.3):
   - `pub mod id;`
   - `pub mod handle;`
   - `pub mod message;`
   - `pub mod traits;`
3. Add re-exports for ergonomic API (per §4.3):
   - `pub use id::ComponentId;`
   - `pub use handle::ComponentHandle;`
   - `pub use message::{ComponentMessage, MessageMetadata};`
   - `pub use traits::ComponentLifecycle;`
4. Add module-level rustdoc documentation
5. Verify NO implementation code in mod.rs (per §4.3)

**Deliverables:**
- File: `airssys-wasm/src/core/component/mod.rs` with declarations and re-exports

**ADR Constraints:**
- ADR-WASM-028 requires: mod.rs with declarations and re-exports
- ADR-WASM-023 requires: No implementation logic in mod.rs

**PROJECTS_STANDARD.md Compliance:**
- §4.3: mod.rs contains ONLY declarations and re-exports
- §2.1: No imports needed (declarations only)

**Rust Guidelines:**
- M-MODULE-DOCS: Module documentation explaining submodule purpose
- M-DESIGN-FOR-AI: Re-exports for ergonomic API surface

**Documentation:**
- Diátaxis type: Reference documentation for component submodule
- Quality: Technical description of submodule contents

**Unit Testing Plan:**
- No tests needed (mod.rs has no implementation)

**Verification:**
```bash
# File exists
ls -la airssys-wasm/src/core/component/mod.rs

# Verify no implementation code
grep -n "fn \|struct \|enum \|impl " airssys-wasm/src/core/component/mod.rs
# Expected: Only mod and pub use statements

# Verify all modules declared
grep -c "^pub mod " airssys-wasm/src/core/component/mod.rs
# Expected: 4 modules declared

# Verify re-exports
grep -c "^pub use " airssys-wasm/src/core/component/mod.rs
# Expected: 4 re-exports
```

---

### Action 6: Update airssys-wasm/src/core/mod.rs to export component submodule
**Objective:** Integrate component submodule into core module

**Steps:**
1. Read existing `airssys-wasm/src/core/mod.rs` (created in WASM-TASK-016)
2. Add module declaration: `pub mod component;`
3. Add re-export: `pub use component::*;` (if needed)
4. Verify module structure is correct
5. Add/update module-level documentation if needed

**Deliverables:**
- Updated file: `airssys-wasm/src/core/mod.rs` with component submodule

**ADR Constraints:**
- ADR-WASM-028 requires: core/mod.rs exports component submodule
- ADR-WASM-023 requires: No forbidden imports

**PROJECTS_STANDARD.md Compliance:**
- §4.3: mod.rs contains ONLY declarations
- §2.1: No imports needed (declarations only)

**Rust Guidelines:**
- M-MODULE-DOCS: Updated documentation including new submodule

**Documentation:**
- Diátaxis type: Reference documentation for core module
- Quality: Technical description including component submodule

**Unit Testing Plan:**
- No tests needed (mod.rs has no implementation)

**Verification:**
```bash
# Verify component module declared
grep -n "pub mod component" airssys-wasm/src/core/mod.rs
# Expected: Declaration present

# Verify no implementation added
grep -n "fn \|struct \|enum \|impl " airssys-wasm/src/core/mod.rs | grep -v "^//"
# Expected: Only mod statements (no implementation)

# Build check
cargo check -p airssys-wasm
```

---

## Unit Testing Plan Summary

**Testing Strategy:**
All unit tests will be placed in each module's `#[cfg(test)]` blocks following Rust conventions.

**Coverage Requirements:**
- Each public function/method must have at least one test
- Each struct must have instantiation tests
- Each trait must have mock implementation test
- Edge cases and error paths must be tested

**Test Distribution:**
- `id.rs`: 6 tests (creation, formatting, comparison, hashing, edge cases)
- `handle.rs`: 5 tests (creation, getters, clone, independence)
- `message.rs`: 7 tests (defaults, creation, metadata combinations, serialization)
- `traits.rs`: 6 tests (mock implementation, method calls, trait object)

**Total Tests:** 24 unit tests across 4 modules

**Verification:**
```bash
# Run all unit tests
cargo test -p airssys-wasm --lib

# Expected: All 24 tests pass
```

---

## Integration Testing Plan

**Status:** NOT APPLICABLE for this task

**Rationale:**
This task creates pure data structures and trait definitions in core/ (Layer 1). These types are foundational abstractions that do not require integration testing until:
1. Runtime module (Layer 2B) implements ComponentLifecycle trait
2. Component module (Layer 3A) uses ComponentHandle in real contexts
3. Messaging module (Layer 3B) uses ComponentMessage in real communication

**Future Testing:**
Integration tests will be added in:
- WASM-TASK-0XX (Runtime module) - Test ComponentLifecycle implementations
- WASM-TASK-0XX (Component module) - Test ComponentHandle usage
- WASM-TASK-0XX (Messaging module) - Test ComponentMessage flow

---

## Architecture Verification

### Module Boundary Verification (ADR-WASM-023)

**Core Module Rule:** core/ imports NOTHING except std

**Verification Commands (MUST PASS):**
```bash
# Verify core/component/ has no external imports
grep -rn "use crate::" airssys-wasm/src/core/component/
# Expected: Empty (no output)

# Verify core/component/ has no external crate imports
grep -rn "^use " airssys-wasm/src/core/component/ | grep -v "std::"
# Expected: Only super::module::Type imports (internal)
```

### Dependency Flow Verification

**Expected Dependency Flow:**
```
component/ (future) → core/component/ ✅
security/ (future) → core/component/ ✅
runtime/ (future) → core/component/ ✅
messaging/ (future) → core/component/ ✅

core/component/ → NOTHING ✅ (foundation)
```

### File Structure Verification

```bash
# Verify all files created
ls -la airssys-wasm/src/core/component/
# Expected: id.rs, handle.rs, message.rs, traits.rs, mod.rs

# Verify core/mod.rs updated
grep -n "pub mod component" airssys-wasm/src/core/mod.rs
# Expected: Declaration present
```

---

## Verification Commands

Run after ALL actions complete:

```bash
# 1. Build check
cargo build -p airssys-wasm
# Expected: Clean build, zero errors

# 2. Lint check with zero warnings
cargo clippy -p airssys-wasm --all-targets -- -D warnings
# Expected: Zero warnings

# 3. Unit tests
cargo test -p airssys-wasm --lib
# Expected: All 24 tests pass

# 4. Documentation check
cargo doc -p airssys-wasm --no-deps
# Expected: Documentation builds successfully

# 5. Module boundary verification (ADR-WASM-023)
grep -rn "use crate::" airssys-wasm/src/core/component/
# Expected: Empty (no forbidden imports)

# 6. File structure verification
ls -la airssys-wasm/src/core/component/
# Expected: 5 files (id.rs, handle.rs, message.rs, traits.rs, mod.rs)
```

---

## Success Criteria

- ✅ All 6 module files created (id.rs, handle.rs, message.rs, traits.rs, mod.rs)
- ✅ core/mod.rs updated to export component submodule
- ✅ All types match ADR-WASM-028 specifications exactly
- ✅ Zero compiler warnings (cargo clippy -D warnings)
- ✅ All 24 unit tests pass
- ✅ Module boundary verification passes (grep returns empty)
- ✅ core/component/ imports only std and sibling modules
- ✅ All public APIs have rustdoc documentation with examples
- ✅ Build passes cleanly (cargo build -p airssys-wasm)
- ✅ Types ready for use by security/, runtime/, component/, messaging/ modules

---

## Quality Standards Checklist

**Per PROJECTS_STANDARD.md §6.4 Implementation Quality Gates:**

- [ ] **Safety First:** No `unsafe` blocks (not needed for this task)
- [ ] **Zero Warnings:** All code compiles cleanly with clippy -D warnings
- [ ] **Comprehensive Tests:** 24 unit tests covering all public APIs
- [ ] **Security Logging:** N/A (no operations to log)
- [ ] **Resource Management:** N/A (no resources to manage)

**Per Rust Guidelines:**

- [ ] **M-DESIGN-FOR-AI:** Idiomatic APIs with Into<String>, Default trait
- [ ] **M-MODULE-DOCS:** All public items have rustdoc with examples
- [ ] **M-STATIC-VERIFICATION:** All code passes clippy with strict settings
- [ ] **M-FEATURES-ADDITIVE:** Types don't break existing code

**Per Documentation Standards:**

- [ ] **Reference Documentation:** All types documented as reference
- [ ] **No Hyperbole:** Technical language only
- [ ] **Standards Compliance:** Task file updated with checklist

---

## Notes

**Placeholder Types:**
- MessagePayload: Placeholder as `Vec<u8>` until core/messaging/payload.rs is created
- WasmError: Placeholder type alias until core/errors/wasm.rs is created

**Future Tasks:**
These placeholders will be replaced by real implementations in:
- WASM-TASK-0XX: Create core/messaging/payload.rs with MessagePayload
- WASM-TASK-0XX: Create core/errors/wasm.rs with WasmError

**Dependency Chain:**
This task is foundation for:
- Security module (uses ComponentId for capability scoping)
- Runtime module (uses ComponentHandle for WASM execution)
- Component module (uses ComponentLifecycle for lifecycle management)
- Messaging module (uses ComponentMessage for inter-component communication)

