# Fix Plan: WASM-TASK-006 Taxonomy Violation

**Task ID:** WASM-TASK-006-TAXONOMY-FIX
**Created:** 2025-12-30
**Status:** PLANNING
**Priority:** HIGH - Memory Bank Compliance
**Estimated Effort:** 2-3 hours

---

## Executive Summary

The Memory Bank taxonomy is violated with 16 scattered files for WASM-TASK-006. Per Memory Bank instructions, tasks should be single files: `tasks/task-[id]-[name].md`. This plan consolidates scattered files into the canonical task file and archives historical documents to context-snapshots/.

---

## Problem Statement

### Current State

```
.memory-bank/sub-projects/airssys-wasm/tasks/
├── task-006-block-5-inter-component-communication.md (1432 lines) ✅ CANONICAL
├── task-006-block-5-inter-component-communication.md.backup ❌ DELETE
├── task-006-phase-1-task-1.1-plan.md.backup ❌ DELETE
├── task-006-architecture-remediation-critical.md (781 lines) ⚠️ DECIDE
├── task-006-architecture-remediation-phase-2-duplicate-runtime.md (645 lines) ⚠️ DECIDE
├── task-006-hotfix-module-boundary-violations.md (216 lines) ⚠️ DECIDE
├── task-006-phase-1-task-1.1-plan.md (49KB) ⏳ ARCHIVE
├── task-006-phase-1-task-1.1-remediation-plan.md (23KB) ⏳ ARCHIVE
├── task-006-phase-1-task-1.2-plan.md (37KB) ⏳ ARCHIVE
├── task-006-phase-1-task-1.2-remediation-plan.md (24KB) ⏳ ARCHIVE
├── task-006-phase-1-task-1.3-plan.md (21KB) ⏳ ARCHIVE
├── task-006-phase-2-task-2.2-plan.md (15KB) ⏳ ARCHIVE
├── task-006-phase-2-task-2.2-revised-plan.md (15KB) ⏳ ARCHIVE
├── task-006-phase-2-task-2.3-fire-and-forget-performance-plan.md (18KB) ⏳ ARCHIVE
└── task-006-phase-3-task-3.3-plan.md (23KB) ⏳ ARCHIVE
```

**Violation:** Single-file rule per Memory Bank instructions
- 16 files in tasks/ directory for one task
- Historical planning documents should be in context-snapshots/
- Backup files should be deleted

---

## File Analysis

### 1. Backup Files (DELETE)

| File | Size | Action | Reason |
|------|------|--------|--------|
| `task-006-block-5-inter-component-communication.md.backup` | 24KB | DELETE | Backup of canonical file |
| `task-006-phase-1-task-1.1-plan.md.backup` | 46KB | DELETE | Backup of plan file |

**Rationale:** These are `.backup` files created by editors/git operations. The canonical files exist. No unique information.

**Verification:**
```bash
# Confirm backup content is older version
diff <(head -20 task-006-block-5-inter-component-communication.md.backup) \
     <(head -20 task-006-block-5-inter-component-communication.md)
# Expected: Shows differences (backup is older)
```

---

### 2. Architecture/Hotfix Files (DECIDE - NEED UNIQUE INFO CHECK)

#### 2.1 `task-006-architecture-remediation-critical.md` (781 lines)

**Purpose:** Critical architecture remediation plan addressing:
- Duplicate WASM runtime violation (actor/ using core WASM API instead of Component Model)
- Circular dependency (runtime/ → actor/)
- Three phases of remediation (4-6 days effort)

**Key Content:**
- Phase 1: Fix circular dependency (2.5-4.5 hours)
  - Move ComponentMessage to core/
  - Relocate messaging_subscription.rs
  - Add CI layer dependency enforcement
- Phase 2: Fix duplicate runtime (24-36 hours)
  - Delete workaround code from component_actor.rs
  - Add WasmEngine injection to ComponentActor
  - Rewrite Child::start() to use WasmEngine
  - Rewrite Actor::handle() for Component Model
- Phase 3: Verification & documentation

**Unique Information NOT in Canonical File:**
1. Detailed remediation steps for duplicate runtime issue
2. Code templates for ComponentModel integration
3. Implementation checklists for each task
4. CI layer dependency enforcement script
5. Test requirements for Component Model usage

**Canonical File Reference:** Lines 13395-13410 in canonical file mention "architecture remediation" but don't include the detailed plan.

**Recommendation:** ARCHIVE as context snapshot

---

#### 2.2 `task-006-architecture-remediation-phase-2-duplicate-runtime.md` (645 lines)

**Purpose:** Phase 2 of the architecture remediation (duplicate runtime fix)

**Key Content:**
- Task 2.1: Delete workaround code (DEFERRED - incremental approach)
- Task 2.2: Add WasmEngine injection (COMPLETE 2025-12-21)
- Task 2.3: Rewrite Child::start() (COMPLETE 2025-12-21)
- Task 2.4: Rewrite Actor::handle() (COMPLETE 2025-12-21)
- Task 2.5: Extend WasmEngine (COMPLETE)
- Task 2.6: Update all tests (COMPLETE)

**Unique Information NOT in Canonical File:**
1. Implementation status for each task (2.2, 2.3, 2.4, 2.5, 2.6 complete)
2. Code templates for Component Model integration
3. Dual-path architecture (legacy + Component Model)
4. Verification commands for Component Model usage

**Canonical File Reference:** Progress log mentions "Phase 2 COMPLETE" at lines 12876-12892 but doesn't include implementation details.

**Recommendation:** ARCHIVE as context snapshot

---

#### 2.3 `task-006-hotfix-module-boundary-violations.md` (216 lines)

**Purpose:** Module boundary violation hotfix plan

**Key Content:**
- Violation #1: core/ → runtime/ (src/core/config.rs:82)
  - Imports: `CpuConfig`, `MemoryConfig`, `ResourceConfig`, `ResourceLimits` from runtime/
  - Impact: Inverts entire dependency hierarchy
- Violation #2: runtime/ → actor/ (src/runtime/messaging.rs:78)
  - Imports: `CorrelationTracker` from actor/
  - Impact: Creates circular dependency potential
- Fix options:
  - Option A: Move CorrelationTracker to core/
  - Option B: Move MessagingService to actor/
- Verification commands

**Unique Information NOT in Canonical File:**
1. Specific grep commands for detecting violations
2. Detailed fix options (A vs B) with trade-offs
3. File-by-file modification checklist
4. Verification commands that must pass
5. History of false claims about completion

**Canonical File Reference:** Status Update at lines 13997-14330 mentions stub file discovery but doesn't include the boundary violation fix plan.

**Recommendation:** ARCHIVE as context snapshot (current unresolved violations)

---

### 3. Phase Plan Files (ARCHIVE - Historical Planning Documents)

| File | Size | Description | Phase |
|------|------|-------------|-------|
| `task-006-phase-1-task-1.1-plan.md` | 49KB | MessageBroker Setup for Components | 1.1 |
| `task-006-phase-1-task-1.1-remediation-plan.md` | 23KB | Remediation for 1.1 (message delivery stubbed) | 1.1 |
| `task-006-phase-1-task-1.2-plan.md` | 37KB | ComponentActor Message Reception | 1.2 |
| `task-006-phase-1-task-1.2-remediation-plan.md` | 24KB | Remediation for 1.2 (tests don't prove functionality) | 1.2 |
| `task-006-phase-1-task-1.3-plan.md` | 21KB | ActorSystem Event Subscription Infrastructure | 1.3 |
| `task-006-phase-2-task-2.2-plan.md` | 15KB | handle-message Component Export | 2.2 |
| `task-006-phase-2-task-2.2-revised-plan.md` | 15KB | Revised plan for 2.2 | 2.2 |
| `task-006-phase-2-task-2.3-fire-and-forget-performance-plan.md` | 18KB | Fire-and-Forget Performance | 2.3 |
| `task-006-phase-3-task-3.3-plan.md` | 23KB | Timeout and Cancellation | 3.3 |

**Rationale for Archiving:**
- All tasks 1.1, 1.2, 1.3, 2.1, 2.2, 2.3, 3.1, 3.2 are COMPLETE
- The canonical file contains summary of what was done
- Detailed planning documents are historical artifacts
- Per Memory Bank instructions, historical context belongs in context-snapshots/

**Information Status Check:**
1. Task 1.1 plan: Complete (line 1098)
2. Task 1.2 plan: Complete (line 1216)
3. Task 1.3 plan: Complete (line 1636)
4. Task 2.2 plan: Complete (line 1086)
5. Task 2.3 plan: Complete (line 1034)
6. Task 3.3 plan: Not started (line 682)

**Unique Information:**
- Detailed implementation plans for each task
- Testing strategies
- Performance targets
- Risk assessments
- These are valuable for historical reference but don't need to be in the canonical task file

**Recommendation:** ARCHIVE all to context-snapshots/ with descriptive dates

---

## Consolidation Plan

### Action 1: Consolidate Unique Architecture Info to Canonical File

**Goal:** Extract unique information from architecture/hotfix files and add to canonical task file.

**What to Add:**

```markdown
## Architecture Remediation History

### 2025-12-21: Architecture Hotfix - Duplicate Runtime Remediation

**Problem:** `actor/component/` created its own WASM runtime using core WASM API (`wasmtime::Module`) instead of using existing `runtime/WasmEngine` which uses Component Model API (`wasmtime::component::Component`). This violated ADR-WASM-002.

**Impact:**
- WIT interfaces 100% non-functional
- Generated bindings (154KB) completely unused
- Type safety bypassed (manual byte manipulation)
- 260+ lines of workaround code

**Solution:**
1. Phase 1: Fix circular dependency (runtime/ → actor/)
   - Move ComponentMessage to core/
   - Relocate messaging_subscription.rs to actor/component/
   - Add CI layer dependency enforcement

2. Phase 2: Fix duplicate runtime
   - Add WasmEngine injection to ComponentActor
   - Rewrite Child::start() to use WasmEngine (Component Model)
   - Rewrite Actor::handle() for Component Model typed calls
   - Update all tests

3. Phase 3: Verification
   - Zero `wasmtime::Module` in actor/
   - Zero `WasmBumpAllocator` anywhere
   - Zero `HandleMessageParams` anywhere
   - All tests pass

**Status:** Phase 1 COMPLETE, Phase 2 COMPLETE (incremental approach taken)

**Reference:** ADR-WASM-021, ADR-WASM-022

---

### 2025-12-22: Module Boundary Violations

**Problem:** Two critical module boundary violations:

**Violation #1:** core/ → runtime/ (src/core/config.rs:82)
```rust
use crate::runtime::limits::{CpuConfig, MemoryConfig, ResourceConfig, ResourceLimits};
```
- Inverts dependency hierarchy
- core/ should have ZERO crate imports

**Violation #2:** runtime/ → actor/ (src/runtime/messaging.rs:78)
```rust
use crate::actor::message::CorrelationTracker;
```
- runtime/ should only import from core/, security/
- Creates circular dependency potential

**Required Fix (Per ADR-WASM-023):**
```
core/     → imports NOTHING from crate
security/ → imports core/ only
runtime/  → imports core/, security/ only
actor/    → imports core/, security/, runtime/
```

**Status:** NOT COMPLETE (awaiting fix)

**Reference:** ADR-WASM-023
```

**Location in Canonical File:** Add after "Progress Log" section, before "Status Update"

---

### Action 2: Update Canonical File

**File:** `.memory-bank/sub-projects/airssys-wasm/tasks/task-006-block-5-inter-component-communication.md`

**Changes:**
1. Add "Architecture Remediation History" section (see above)
2. Update task index to show consolidated status
3. Remove references to scattered plan files (replace with inline summaries)
4. Add archive references in "Related Documentation" section

---

### Action 3: Delete Backup Files

**Commands:**
```bash
rm .memory-bank/sub-projects/airssys-wasm/tasks/task-006-block-5-inter-component-communication.md.backup
rm .memory-bank/sub-projects/airssys-wasm/tasks/task-006-phase-1-task-1.1-plan.md.backup
rm .memory-bank/sub-projects/airssys-wasm/tasks/task-hotfix-001-messaging-module-architecture-refactoring.md.backup
```

**Verification:**
```bash
# Confirm deletions
ls -lh .memory-bank/sub-projects/airssys-wasm/tasks/*.backup
# Expected: No such file or directory
```

---

### Action 4: Archive Architecture/Hotfix Files

**Files to Archive:**
1. `task-006-architecture-remediation-critical.md`
   → `2025-12-21-task-006-architecture-remediation-critical.md`

2. `task-006-architecture-remediation-phase-2-duplicate-runtime.md`
   → `2025-12-21-task-006-architecture-remediation-phase-2-duplicate-runtime.md`

3. `task-006-hotfix-module-boundary-violations.md`
   → `2025-12-22-task-006-hotfix-module-boundary-violations.md`

**Commands:**
```bash
# Move to context-snapshots
mv .memory-bank/sub-projects/airssys-wasm/tasks/task-006-architecture-remediation-critical.md \
   .memory-bank/sub-projects/airssys-wasm/context-snapshots/2025-12-21-task-006-architecture-remediation-critical.md

mv .memory-bank/sub-projects/airssys-wasm/tasks/task-006-architecture-remediation-phase-2-duplicate-runtime.md \
   .memory-bank/sub-projects/airssys-wasm/context-snapshots/2025-12-21-task-006-architecture-remediation-phase-2-duplicate-runtime.md

mv .memory-bank/sub-projects/airssys-wasm/tasks/task-006-hotfix-module-boundary-violations.md \
   .memory-bank/sub-projects/airssys-wasm/context-snapshots/2025-12-22-task-006-hotfix-module-boundary-violations.md
```

---

### Action 5: Archive Phase Plan Files

**Files to Archive:**

| Source File | Target Name | Date |
|-------------|--------------|------|
| `task-006-phase-1-task-1.1-plan.md` | `2025-12-20-task-006-phase-1-task-1.1-plan.md` | 2025-12-20 |
| `task-006-phase-1-task-1.1-remediation-plan.md` | `2025-12-21-task-006-phase-1-task-1.1-remediation-plan.md` | 2025-12-21 |
| `task-006-phase-1-task-1.2-plan.md` | `2025-12-21-task-006-phase-1-task-1.2-plan.md` | 2025-12-21 |
| `task-006-phase-1-task-1.2-remediation-plan.md` | `2025-12-21-task-006-phase-1-task-1.2-remediation-plan.md` | 2025-12-21 |
| `task-006-phase-1-task-1.3-plan.md` | `2025-12-21-task-006-phase-1-task-1.3-plan.md` | 2025-12-21 |
| `task-006-phase-2-task-2.2-plan.md` | `2025-12-21-task-006-phase-2-task-2.2-plan.md` | 2025-12-21 |
| `task-006-phase-2-task-2.2-revised-plan.md` | `2025-12-21-task-006-phase-2-task-2.2-revised-plan.md` | 2025-12-21 |
| `task-006-phase-2-task-2.3-fire-and-forget-performance-plan.md` | `2025-12-22-task-006-phase-2-task-2.3-fire-and-forget-performance-plan.md` | 2025-12-22 |
| `task-006-phase-3-task-3.3-plan.md` | `2025-12-21-task-006-phase-3-task-3.3-plan.md` | 2025-12-21 |

**Commands:**
```bash
# Archive all phase plan files
mv .memory-bank/sub-projects/airssys-wasm/tasks/task-006-phase-1-task-1.1-plan.md \
   .memory-bank/sub-projects/airssys-wasm/context-snapshots/2025-12-20-task-006-phase-1-task-1.1-plan.md

mv .memory-bank/sub-projects/airssys-wasm/tasks/task-006-phase-1-task-1.1-remediation-plan.md \
   .memory-bank/sub-projects/airssys-wasm/context-snapshots/2025-12-21-task-006-phase-1-task-1.1-remediation-plan.md

# ... (continue for all files)
```

**Alternative:** Single bash script to move all files

```bash
#!/bin/bash
# Archive WASM-TASK-006 phase plan files

cd .memory-bank/sub-projects/airssys-wasm/tasks/

# Define file mappings
declare -A files=(
    ["task-006-phase-1-task-1.1-plan.md"]="2025-12-20-task-006-phase-1-task-1.1-plan.md"
    ["task-006-phase-1-task-1.1-remediation-plan.md"]="2025-12-21-task-006-phase-1-task-1.1-remediation-plan.md"
    ["task-006-phase-1-task-1.2-plan.md"]="2025-12-21-task-006-phase-1-task-1.2-plan.md"
    ["task-006-phase-1-task-1.2-remediation-plan.md"]="2025-12-21-task-006-phase-1-task-1.2-remediation-plan.md"
    ["task-006-phase-1-task-1.3-plan.md"]="2025-12-21-task-006-phase-1-task-1.3-plan.md"
    ["task-006-phase-2-task-2.2-plan.md"]="2025-12-21-task-006-phase-2-task-2.2-plan.md"
    ["task-006-phase-2-task-2.2-revised-plan.md"]="2025-12-21-task-006-phase-2-task-2.2-revised-plan.md"
    ["task-006-phase-2-task-2.3-fire-and-forget-performance-plan.md"]="2025-12-22-task-006-phase-2-task-2.3-fire-and-forget-performance-plan.md"
    ["task-006-phase-3-task-3.3-plan.md"]="2025-12-21-task-006-phase-3-task-3.3-plan.md"
)

# Move files
for src in "${!files[@]}"; do
    dst="${files[$src]}"
    if [ -f "$src" ]; then
        mv "$src" "../context-snapshots/$dst"
        echo "✅ Moved: $src → $dst"
    else
        echo "⚠️  Not found: $src"
    fi
done

echo ""
echo "Archive complete!"
```

---

## File Action Table

| File | Action | Reason | Target Location |
|------|--------|--------|----------------|
| `task-006-block-5-inter-component-communication.md.backup` | DELETE | N/A (backup file) |
| `task-006-phase-1-task-1.1-plan.md.backup` | DELETE | N/A (backup file) |
| `task-hotfix-001-messaging-module-architecture-refactoring.md.backup` | DELETE | N/A (backup file) |
| `task-006-architecture-remediation-critical.md` | ARCHIVE | `context-snapshots/2025-12-21-task-006-architecture-remediation-critical.md` |
| `task-006-architecture-remediation-phase-2-duplicate-runtime.md` | ARCHIVE | `context-snapshots/2025-12-21-task-006-architecture-remediation-phase-2-duplicate-runtime.md` |
| `task-006-hotfix-module-boundary-violations.md` | ARCHIVE | `context-snapshots/2025-12-22-task-006-hotfix-module-boundary-violations.md` |
| `task-006-phase-1-task-1.1-plan.md` | ARCHIVE | `context-snapshots/2025-12-20-task-006-phase-1-task-1.1-plan.md` |
| `task-006-phase-1-task-1.1-remediation-plan.md` | ARCHIVE | `context-snapshots/2025-12-21-task-006-phase-1-task-1.1-remediation-plan.md` |
| `task-006-phase-1-task-1.2-plan.md` | ARCHIVE | `context-snapshots/2025-12-21-task-006-phase-1-task-1.2-plan.md` |
| `task-006-phase-1-task-1.2-remediation-plan.md` | ARCHIVE | `context-snapshots/2025-12-21-task-006-phase-1-task-1.2-remediation-plan.md` |
| `task-006-phase-1-task-1.3-plan.md` | ARCHIVE | `context-snapshots/2025-12-21-task-006-phase-1-task-1.3-plan.md` |
| `task-006-phase-2-task-2.2-plan.md` | ARCHIVE | `context-snapshots/2025-12-21-task-006-phase-2-task-2.2-plan.md` |
| `task-006-phase-2-task-2.2-revised-plan.md` | ARCHIVE | `context-snapshots/2025-12-21-task-006-phase-2-task-2.2-revised-plan.md` |
| `task-006-phase-2-task-2.3-fire-and-forget-performance-plan.md` | ARCHIVE | `context-snapshots/2025-12-22-task-006-phase-2-task-2.3-fire-and-forget-performance-plan.md` |
| `task-006-phase-3-task-3.3-plan.md` | ARCHIVE | `context-snapshots/2025-12-21-task-006-phase-3-task-3.3-plan.md` |
| `task-006-block-5-inter-component-communication.md` | KEEP | Canonical file - consolidate unique info |
| **TOTAL** | **3 DELETE, 12 ARCHIVE, 1 KEEP** | |

---

## Implementation Steps

### Step 1: Backup Current State (5 minutes)

```bash
cd /Users/hiraq/Projects/airsstack/airssys/.memory-bank/sub-projects/airssys-wasm

# Create temporary backup
tar -czf tasks-backup-before-cleanup-$(date +%Y%m%d).tar.gz tasks/
echo "✅ Backup created: tasks-backup-before-cleanup-$(date +%Y%m%d).tar.gz"
```

**Rationale:** Safety rollback in case of mistakes

---

### Step 2: Delete Backup Files (2 minutes)

```bash
cd tasks/

rm -v task-006-block-5-inter-component-communication.md.backup
rm -v task-006-phase-1-task-1.1-plan.md.backup
rm -v task-hotfix-001-messaging-module-architecture-refactoring.md.backup

echo "✅ Backup files deleted"
```

**Verification:**
```bash
ls -lh *.backup 2>&1
# Expected: No such file or directory
```

---

### Step 3: Consolidate Architecture Info to Canonical File (15 minutes)

```bash
# Edit canonical file to add Architecture Remediation History section
vim task-006-block-5-inter-component-communication.md
```

**Insert location:** After line 13987 (end of Progress Log), before Status Update section (line 13989)

**Content to add:** See "Action 1: Consolidate Unique Architecture Info to Canonical File" section above

---

### Step 4: Archive Architecture/Hotfix Files (5 minutes)

```bash
cd tasks/

# Archive critical remediation plans
mv task-006-architecture-remediation-critical.md \
   ../context-snapshots/2025-12-21-task-006-architecture-remediation-critical.md

mv task-006-architecture-remediation-phase-2-duplicate-runtime.md \
   ../context-snapshots/2025-12-21-task-006-architecture-remediation-phase-2-duplicate-runtime.md

mv task-006-hotfix-module-boundary-violations.md \
   ../context-snapshots/2025-12-22-task-006-hotfix-module-boundary-violations.md

echo "✅ Architecture/hotfix files archived"
```

---

### Step 5: Archive Phase Plan Files (5 minutes)

```bash
cd tasks/

# Archive phase 1 plans
mv task-006-phase-1-task-1.1-plan.md \
   ../context-snapshots/2025-12-20-task-006-phase-1-task-1.1-plan.md

mv task-006-phase-1-task-1.1-remediation-plan.md \
   ../context-snapshots/2025-12-21-task-006-phase-1-task-1.1-remediation-plan.md

mv task-006-phase-1-task-1.2-plan.md \
   ../context-snapshots/2025-12-21-task-006-phase-1-task-1.2-plan.md

mv task-006-phase-1-task-1.2-remediation-plan.md \
   ../context-snapshots/2025-12-21-task-006-phase-1-task-1.2-remediation-plan.md

mv task-006-phase-1-task-1.3-plan.md \
   ../context-snapshots/2025-12-21-task-006-phase-1-task-1.3-plan.md

# Archive phase 2 plans
mv task-006-phase-2-task-2.2-plan.md \
   ../context-snapshots/2025-12-21-task-006-phase-2-task-2.2-plan.md

mv task-006-phase-2-task-2.2-revised-plan.md \
   ../context-snapshots/2025-12-21-task-006-phase-2-task-2.2-revised-plan.md

mv task-006-phase-2-task-2.3-fire-and-forget-performance-plan.md \
   ../context-snapshots/2025-12-22-task-006-phase-2-task-2.3-fire-and-forget-performance-plan.md

# Archive phase 3 plans
mv task-006-phase-3-task-3.3-plan.md \
   ../context-snapshots/2025-12-21-task-006-phase-3-task-3.3-plan.md

echo "✅ Phase plan files archived"
```

---

### Step 6: Verification (5 minutes)

```bash
cd /Users/hiraq/Projects/airsstack/airssys/.memory-bank/sub-projects/airssys-wasm/tasks/

# 1. Count remaining task-006 files (should be 1)
echo "Task 006 files remaining:"
ls -1 task-006*.md | wc -l
ls -1 task-006*.md

# 2. Verify no backup files
echo ""
echo "Backup files remaining:"
ls -1 *.backup 2>&1 || echo "None (as expected)"

# 3. Verify archived files exist
echo ""
echo "Archived files in context-snapshots:"
ls -1 ../context-snapshots/2025-*-task-006*.md | wc -l
ls -1 ../context-snapshots/2025-*-task-006*.md

# 4. Verify canonical file exists and is updated
echo ""
echo "Canonical file status:"
wc -l task-006-block-5-inter-component-communication.md
grep -c "Architecture Remediation History" task-006-block-5-inter-component-communication.md
```

**Expected Results:**
1. Task 006 files remaining: 1 (canonical file only)
2. Backup files remaining: None
3. Archived files in context-snapshots: 12 (3 architecture + 9 phase plans)
4. Canonical file exists and contains "Architecture Remediation History" section

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Accidentally deleting unique information | Low | High | Step 1 creates backup tarball |
| Breaking references from canonical file | Medium | Medium | Update canonical file before archiving |
| Incorrect archive naming | Low | Low | Follow existing pattern in context-snapshots/ |
| File system permission issues | Very Low | Medium | Verify write permissions before starting |
| Loss of historical context | Very Low | Low | Files are archived, not deleted |

---

## Success Criteria Checklist

### Pre-Cleanup Verification
- [ ] Backup tarball created successfully
- [ ] All files analyzed for unique information
- [ ] Canonical file read and understood
- [ ] Architecture remediation info extracted
- [ ] Archive names follow existing pattern

### During Cleanup
- [ ] 3 backup files deleted
- [ ] 12 files archived to context-snapshots/
- [ ] Canonical file updated with Architecture Remediation History
- [ ] No errors during mv operations
- [ ] All files moved successfully

### Post-Cleanup Verification
- [ ] Only 1 task-006*.md file remains in tasks/
- [ ] That file is `task-006-block-5-inter-component-communication.md`
- [ ] No *.backup files remain in tasks/
- [ ] 12 new files exist in context-snapshots/
- [ ] Canonical file contains new section
- [ ] All archive names follow YYYY-MM-DD-description.md pattern

### Memory Bank Compliance
- [ ] Single-file rule satisfied (one task file)
- [ ] Historical context properly archived
- [ ] Archive location: context-snapshots/ (not tasks/)
- [ ] Archive naming: kebab-case with date prefix
- [ ] No orphaned files in tasks/

---

## References

### Memory Bank Instructions
- `.aiassisted/instructions/multi-project-memory-bank.instructions.md` (lines 71-122)

### Existing Snapshots (Naming Pattern)
- `2025-10-23-wasm-task-004-status.md`
- `2025-12-21-wasm-task-006-phase-2-task-2.1-planning-complete.md`
- `2025-12-22-architecture-violations-discovered.md`

### Canonical Task File
- `.memory-bank/sub-projects/airssys-wasm/tasks/task-006-block-5-inter-component-communication.md`

---

## Summary

**Total Files:** 16
**Action Plan:**
- 3 DELETE (backup files)
- 12 ARCHIVE (architecture/hotfix + phase plans)
- 1 KEEP (canonical file)

**Expected Outcome:**
- Single task file in tasks/ directory
- 12 historical documents in context-snapshots/
- Memory Bank taxonomy compliant
- No unique information lost
- Canonical file enriched with architecture remediation history

**Time Estimate:** 30-40 minutes

---

**Created:** 2025-12-30
**Status:** PLANNING - Ready for execution
**Next Step:** Execute implementation steps
