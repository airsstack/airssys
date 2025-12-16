# Task 5.2: Lifecycle Hooks and Custom State Management - README

**Date:** 2025-12-16  
**Status:** READY FOR IMPLEMENTATION (with design change)  
**Critical Update:** Custom state design changed from `Box<dyn Any>` to generic parameter

---

## 📚 Documentation Overview

This task has **THREE** documents:

### 1. Implementation Plan (Original)
**File:** `task-004-phase-5-task-5.2-lifecycle-hooks-custom-state-plan.md`  
**Size:** 1,109 lines  
**Purpose:** Comprehensive implementation guide

**Contains:**
- Overview & objectives
- LifecycleHooks trait design ✅ (VALID)
- ~~CustomState design~~ ❌ (SUPERSEDED - see DESIGN-CHANGE.md)
- EventCallback trait design ✅ (VALID)
- Implementation steps 1-12
- Testing strategy
- Performance targets

**Status:** Use with caution - Section 2 and Step 2 are outdated

---

### 2. Design Change Document (CRITICAL UPDATE)
**File:** `task-004-phase-5-task-5.2-DESIGN-CHANGE.md`  
**Size:** ~400 lines  
**Purpose:** Critical correction to custom state design

**Contains:**
- Problem explanation (Box<dyn Any> issues)
- Solution (generic parameter ComponentActor<S>)
- Complete code examples
- Updated implementation steps
- Migration impact
- Performance comparison

**Status:** ✅ **READ THIS FIRST** before implementing

---

### 3. This README
**File:** `task-004-phase-5-task-5.2-README.md`  
**Purpose:** Navigation guide for all documents

---

## 🚀 How to Implement Task 5.2

### Step 0: Read Documents in Order

1. **Read DESIGN-CHANGE.md** (5 minutes)
   - Understand why we changed from Box<dyn Any> to generic
   - Review new design: ComponentActor<S = ()>
   
2. **Read original plan** (15 minutes)
   - Section 1: LifecycleHooks ✅ (use as-is)
   - ~~Section 2: CustomState~~ ❌ (skip, use DESIGN-CHANGE instead)
   - Section 3: EventCallback ✅ (use as-is)
   - Implementation steps (with modifications from DESIGN-CHANGE)

---

## 📋 Corrected Implementation Plan

### Phase 1: Trait Definitions (2.5 hours)

**Step 1: Create LifecycleHooks Module** (1 hour)
- File: `src/actor/lifecycle/hooks.rs`
- Follow original plan Step 1 ✅
- No changes needed

**Step 2: Update ComponentActor to Generic** (1.5 hours) ⚠️ **CHANGED**
- File: `src/actor/component/component_actor.rs`
- Follow DESIGN-CHANGE.md Step 2 ✅
- Add generic parameter `<S = ()>`
- Add `with_state()` / `with_state_mut()` methods
- DO NOT create `src/actor/lifecycle/state.rs`

**Step 3: Create EventCallback Module** (45 minutes)
- File: `src/actor/lifecycle/callbacks.rs`
- Follow original plan Step 3 ✅
- No changes needed

---

### Phase 2: Trait Implementation Updates (1 hour) ⚠️ **NEW STEP**

**Step 4: Update Actor and Child Trait Impls** (1 hour)
- Files: `src/actor/component/actor_impl.rs`, `child_impl.rs`
- Follow DESIGN-CHANGE.md Step 4 ✅
- Add generic bounds: `impl<S: Send + Sync> Actor for ComponentActor<S>`
- Add generic bounds: `impl<S: Send + Sync> Child for ComponentActor<S>`

---

### Phase 3: Hook Integration (3.5 hours)

**Step 5: Integrate Hooks into Child::start()** (1 hour)
- Follow original plan Step 5 ✅
- No changes needed

**Step 6: Integrate Hooks into Child::stop()** (1 hour)
- Follow original plan Step 6 ✅
- No changes needed

**Step 7: Integrate Hooks into Actor::handle_message()** (1.5 hours)
- Follow original plan Step 7 ✅
- No changes needed

**Step 8: Helper Methods for Hook Execution** (45 minutes)
- Follow original plan Step 8 ✅
- No changes needed

---

### Phase 4: Testing & Documentation (3 hours)

**Step 9: Unit Tests** (1 hour)
- Follow original plan Step 9 ✅
- Add state-specific tests for generic parameter

**Step 10: Integration Tests** (1.5 hours)
- Follow original plan Step 10 ✅
- Test state persistence across messages

**Step 11: Documentation & Examples** (1 hour)
- Follow original plan Step 11 ✅
- Document generic state usage

**Step 12: Code Review & Final Verification** (1 hour)
- Follow original plan Step 12 ✅
- Verify generic bounds compile correctly

---

## ⚙️ Key Design Decisions

### ✅ What We're Using

1. **LifecycleHooks Trait**
   ```rust
   pub trait LifecycleHooks: Send + Sync {
       fn pre_start(&mut self, ctx: &LifecycleContext) -> HookResult;
       fn post_start(&mut self, ctx: &LifecycleContext) -> HookResult;
       fn pre_stop(&mut self, ctx: &LifecycleContext) -> HookResult;
       fn post_stop(&mut self, ctx: &LifecycleContext) -> HookResult;
       fn on_message_received(&mut self, ctx: &LifecycleContext, msg: &ComponentMessage) -> HookResult;
       fn on_error(&mut self, ctx: &LifecycleContext, error: &WasmError) -> HookResult;
       fn on_restart(&mut self, ctx: &LifecycleContext, reason: RestartReason) -> HookResult;
   }
   ```

2. **Generic State Parameter** ⚠️ **CHANGED**
   ```rust
   pub struct ComponentActor<S = ()> 
   where S: Send + Sync + 'static
   {
       hooks: Box<dyn LifecycleHooks>,
       custom_state: Arc<RwLock<S>>,  // NOT Box<dyn Any>!
       event_callback: Option<Arc<dyn EventCallback>>,
   }
   
   impl<S: Send + Sync + 'static> ComponentActor<S> {
       pub async fn with_state<F, R>(&self, f: F) -> R
       where F: FnOnce(&S) -> R { ... }
       
       pub async fn with_state_mut<F, R>(&self, f: F) -> R
       where F: FnOnce(&mut S) -> R { ... }
   }
   ```

3. **EventCallback Trait**
   ```rust
   pub trait EventCallback: Send + Sync {
       fn on_message_received(&self, component_id: ComponentId);
       fn on_message_processed(&self, component_id: ComponentId, latency: Duration);
       fn on_error_occurred(&self, component_id: ComponentId, error: &WasmError);
       fn on_restart_triggered(&self, component_id: ComponentId, reason: RestartReason);
       fn on_health_changed(&self, component_id: ComponentId, new_health: HealthStatus);
   }
   ```

---

## 🎯 Performance Targets (UPDATED)

| Metric | Target | Notes |
|--------|--------|-------|
| Hook execution | <50μs | Unchanged from original |
| **State access** | **<50ns** | **Was <1μs, now 2.5x faster** |
| Callback dispatch | <10μs | Unchanged from original |
| Message throughput | >10k/sec | Unchanged from original |

---

## 📦 File Structure (UPDATED)

### New Files Created
```
src/actor/lifecycle/
├── mod.rs (60 lines)
├── hooks.rs (250 lines) - LifecycleHooks trait
├── callbacks.rs (150 lines) - EventCallback trait
└── executor.rs (200 lines) - Hook execution helpers

❌ NO state.rs - integrated into ComponentActor!
```

### Files Modified
```
src/actor/component/
├── component_actor.rs (+150 lines) - Add generic parameter <S>
├── actor_impl.rs (+120 lines) - Add generic bounds + hooks
└── child_impl.rs (+160 lines) - Add generic bounds + hooks

src/actor/mod.rs (+10 lines)
src/lib.rs (+5 lines)
```

**Total Code Volume:**
- New: ~660 lines (hooks + callbacks + executor, NO state.rs)
- Modified: ~445 lines (generic refactoring + integration)
- Tests: ~700 lines
- **Total:** ~1,805 lines

---

## ✅ Success Criteria

### Compilation
- [ ] ComponentActor<S> compiles with generic parameter
- [ ] All trait impls compile with generic bounds
- [ ] Default to `()` works (backward compatible)
- [ ] Zero compiler warnings
- [ ] Zero clippy warnings

### Functionality
- [ ] LifecycleHooks trait with 7 methods
- [ ] EventCallback trait with 5 methods
- [ ] Generic state with `with_state()` / `with_state_mut()`
- [ ] Hooks fire at correct lifecycle points
- [ ] Event callbacks fire correctly

### Testing
- [ ] 20-30 comprehensive tests passing
- [ ] Unit tests: hooks, state, callbacks
- [ ] Integration tests: full lifecycle
- [ ] Performance tests: targets met

### Performance
- [ ] Hook overhead: <50μs ✅
- [ ] State access: <50ns ✅ (2.5x better than original <1μs target)
- [ ] Callback dispatch: <10μs ✅
- [ ] Message throughput: >10k/sec ✅

### Quality
- [ ] Code quality: 9.5/10
- [ ] 100% rustdoc coverage
- [ ] Standards compliance: §2.1-§6.3

---

## ⚠️ Critical Notes for Implementation

### 1. DO NOT Create CustomState Module
❌ `src/actor/lifecycle/state.rs` should NOT exist  
✅ State is integrated into ComponentActor via generic parameter

### 2. Update All Trait Implementations
All impls need generic bounds:
```rust
impl<S: Send + Sync + 'static> Actor for ComponentActor<S> { ... }
impl<S: Send + Sync + 'static> Child for ComponentActor<S> { ... }
```

### 3. Backward Compatibility
Default generic `<S = ()>` maintains compatibility:
```rust
// Old code still works
let actor = ComponentActor::new(id, ());  // explicit
let actor = ComponentActor::default();     // uses ()

// New code with state
let actor = ComponentActor::new(id, MyState::default());
```

### 4. Testing Strategy
Test both stateless and stateful actors:
- Stateless: `ComponentActor<()>`
- With state: `ComponentActor<CounterState>`, etc.

---

## 📞 Quick Reference

### When Following Original Plan
- ✅ **Step 1** (LifecycleHooks): Use as-is
- ❌ **Step 2** (CustomState): Replace with DESIGN-CHANGE.md Step 2
- ✅ **Step 3** (EventCallback): Use as-is
- ➕ **Step 4** (NEW): Add trait impl updates from DESIGN-CHANGE.md
- ✅ **Steps 5-12**: Use as-is (with renumbering)

### When in Doubt
1. Check DESIGN-CHANGE.md first
2. Refer to original plan for hooks/callbacks
3. Ask if unclear (better safe than refactor)

---

## 🚀 Ready to Start?

**Command to implement:**
```bash
@memorybank-implementer WASM-TASK-004 Phase 5 Task 5.2
```

**Before implementation:**
1. ✅ Read DESIGN-CHANGE.md
2. ✅ Review this README
3. ✅ Skim original plan (note Step 2 is outdated)
4. ✅ Understand generic ComponentActor<S> design

**Estimated Time:** 4.5-6.5 hours (0.5h more than original due to refactoring)

---

**Status:** ✅ READY FOR IMPLEMENTATION with corrected design

