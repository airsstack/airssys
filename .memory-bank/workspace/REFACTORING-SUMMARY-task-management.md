# Memory Bank Task Management Refactoring Summary

**Date:** 2025-12-17  
**Author:** Memory Bank Manager  
**Status:** Refactoring Complete - Awaiting User Approval

---

## Executive Summary

Refactored the Multi-Project Memory Bank instructions to address critical issues with scattered task information and inconsistent task taxonomies. The new instructions introduce:

1. **Centralized Single-File Task Management** - All task information in ONE file
2. **Consistent Task Taxonomy** - Standardized Task → Phase → Subtask hierarchy
3. **Comprehensive Task Templates** - Templates for both simple and complex multi-phase tasks
4. **Strict File Organization Rules** - Clear prohibitions on scattered task files

---

## Problem Analysis

### Problem 1: Task Information Scattered Across Multiple Files

**Example: WASM-TASK-004** (80+ files)
- Main task file: `task-004-block-3-actor-system-integration.md`
- Per-phase plans: `task-004-phase-1-task-1.3-actor-trait-implementation-plan.md`
- Per-phase completions: `task-004-phase-1-task-1.2-completion-summary.md`
- Status tracking: `WASM-TASK-004-STATUS.md`
- Checkpoints: `task-004-phase-6-task-6.1-checkpoint-1.md`
- Audit reports: `task-004-phase-2-task-2.1-audit-report.md`
- ... 70+ more scattered files

**Issues:**
- Navigation nightmare (80+ files to understand progress)
- No single source of truth
- Duplication across files
- Lost context

### Problem 2: Inconsistent Phase/Subtask Taxonomy

**Observed Inconsistencies:**
- WASM-TASK-004: Phase → Task (e.g., Phase 1 → Task 1.1, 1.2, 1.3)
- RT-TASK-003: Direct Subtasks (e.g., Subtask 3.1, 3.2, 3.3)
- RT-TASK-007: Phase-only file (`task-007-phase4-plan.md`, no parent)
- OSL-TASK-002: Phase completions (`phase-4-1-completion.md`, sub-phase?)

**Issues:**
- No standard hierarchy (Phase → Task vs Task → Subtask)
- Unclear depth limits
- Inconsistent numbering (1.1 vs 4a vs 4-1)
- Mixed terminology ("Task" means different things)

---

## Solution: Refactored Instructions

### Key Changes

#### 1. Task Taxonomy & Hierarchy (NEW SECTION)

**Standardized 3-Level Hierarchy:**
```
TASK (Top Level)
  └─ PHASE (Optional: For tasks >4 weeks)
      └─ SUBTASK (Mandatory: Granular work units)
```

**Level Definitions Table:**

| Level | ID Format | Example | Duration | When to Use | Max Count |
|-------|-----------|---------|----------|-------------|-----------|
| Task | [PREFIX]-TASK-### | WASM-TASK-004 | 1-12 weeks | Top-level work item | N/A |
| Phase | Phase N | Phase 1, Phase 2 | 1-4 weeks | Major milestone (>4 weeks total) | 8 per task |
| Subtask | N.M | 1.1, 1.2, 3.5 | <1 week | Granular work unit | 10 per phase |

**Hierarchy Rules:**
1. Task: Always required. One file = one task.
2. Phase: Optional. Use ONLY if task duration >4 weeks. Max 8 phases.
3. Subtask: Always required. Max 10 subtasks per phase.
4. No deeper nesting allowed.

#### 2. Single File Per Task Mandate (NEW SECTION)

**CRITICAL RULE:**
Each task MUST be tracked in ONE canonical file: `tasks/task-[id]-[name].md`

**What MUST Be in the Task File:**
- Task metadata
- Original request
- Thought process
- **Complete implementation plan** (all phases, all subtasks)
- **All progress tracking tables** (one per phase)
- **All progress logs** (chronological, consolidated)
- **All completion summaries** (inline, not separate files)
- Dependencies
- Definition of done

**What is FORBIDDEN:**
- ❌ Separate plan files per phase/subtask
- ❌ Separate completion files per phase/subtask
- ❌ Separate status tracking files
- ❌ Separate checkpoint files
- ❌ Separate audit/review files

**Allowed Exceptions:**
- ✅ External reference docs (ADRs, knowledge docs)
- ✅ Technical debt tracking (DEBT files)
- ✅ Historical snapshots (context-snapshots/)
- ✅ Task index (_index.md)

#### 3. Comprehensive Task Templates (UPDATED)

**Two Templates Provided:**

1. **Simple Task Template** (No phases)
   - Direct subtasks (1.1, 1.2, 1.3)
   - Single progress tracking table
   - Consolidated progress log
   - Inline completion summary

2. **Complex Task Template** (With phases)
   - Multiple phases (Phase 1, Phase 2, ...)
   - Subtasks per phase (1.1, 1.2, 2.1, 2.2)
   - Progress tracking table per phase
   - Phase completion summaries inline
   - Task completion summary at end
   - Overall progress metrics

#### 4. Task Update Protocol (NEW SECTION)

**When updating task progress, you MUST:**
1. Update Progress Tracking Table (mark subtask status)
2. Add Progress Log Entry (document work done)
3. Update Phase Status (recalculate percentage)
4. Update Overall Status (recalculate task percentage)
5. Update Task Metadata (update "Updated" date)
6. Update _index.md (sync task status)

#### 5. Critical Formatting Rules (ENHANCED)

Added new rules:
- **PHASE COMPLETION SUMMARIES**: Must be inline, not separate files
- **PROGRESS LOG CHRONOLOGICAL**: Reverse chronological order (newest first)
- Maintained existing rules (NO EMPTY CELLS, DATE FORMAT, etc.)

---

## File Changes

### Modified Files

**File:** `.aiassisted/instructions/multi-project-memory-bank.instructions.md.new`
- **Size:** 29KB (was 20KB)
- **Lines:** 821 (was 529, +292 lines)
- **Changes:**
  - Added "Task Taxonomy & Hierarchy" section (~100 lines)
  - Added "Single File Per Task Mandate" section (~80 lines)
  - Replaced "Individual Task Structure" with two templates (~200 lines)
  - Added "Task Update Protocol" section (~30 lines)
  - Enhanced "Critical Formatting Rules" (~20 lines)

**Status:** Created as `.new` file pending user approval

---

## Impact Assessment

### Positive Impacts

1. **Eliminates Scattered Files**
   - Agents will know exactly where to find task information
   - No more hunting through 80+ files
   - Single source of truth

2. **Consistent Taxonomy**
   - Clear Task → Phase → Subtask hierarchy
   - Standardized numbering (Phase N, N.M format)
   - Maximum depth limits prevent chaos

3. **Better Navigation**
   - One file per task = easier to read
   - Inline completion summaries = context preserved
   - Progress logs in chronological order

4. **Improved Tracking**
   - Progress tracking tables per phase
   - Phase completion percentages
   - Overall task completion metrics

### Migration Requirements

**Existing Tasks Need Consolidation:**

For tasks like WASM-TASK-004 with scattered files:
1. Create/update main task file with all sections
2. Copy plan content from phase plan files
3. Copy completion summaries inline
4. Copy progress from checkpoint files
5. Consolidate progress logs chronologically
6. Archive old scattered files to `context-snapshots/`
7. Update `_index.md`

**Estimated Effort:** 2-3 hours per complex task

---

## Next Steps

### Step 1: User Approval

**User must approve:**
- Replace `.aiassisted/instructions/multi-project-memory-bank.instructions.md` with `.new` version
- Commit new instructions to repository

### Step 2: Update Subagent Prompts

**Update Memory Bank Manager system prompt:**
- Reference new task taxonomy rules
- Enforce single-file mandate
- Validate task structure before updates
- Reject scattered file patterns

### Step 3: Migration Guide (Optional)

**Create migration process for existing tasks:**
- Document consolidation steps
- Provide examples (before/after)
- Script to detect scattered task files
- Automated consolidation tool (future enhancement)

---

## Validation

### Requirements Met

✅ **Centralized task information** - Single file per task mandate  
✅ **Consistent taxonomy** - Task → Phase → Subtask hierarchy defined  
✅ **Clear rules** - Explicit prohibitions on scattered files  
✅ **Comprehensive templates** - Both simple and complex task templates  
✅ **Update protocol** - Step-by-step task update instructions  

### Backward Compatibility

⚠️ **Breaking Change**: Existing scattered task files do not comply with new rules.

**Migration Path:**
- New instructions documented in `.new` file
- Old instructions still functional until replaced
- Migration can be done incrementally per task
- No immediate action required for completed tasks

---

## Recommendation

**Approve and implement the refactored instructions:**

1. **Immediate Action:**
   - Review `.multi-project-memory-bank.instructions.md.new`
   - Approve replacement of old instructions
   - Commit new instructions

2. **Near-Term Action (Next Task):**
   - Apply new template to next task creation
   - Use single-file approach
   - Validate new structure works

3. **Long-Term Action (As Needed):**
   - Migrate existing complex tasks incrementally
   - Archive scattered files to snapshots
   - Monitor agent compliance

---

## Files for Review

**New Instructions:**
- `.aiassisted/instructions/multi-project-memory-bank.instructions.md.new` (821 lines)

**Current Instructions (for comparison):**
- `.aiassisted/instructions/multi-project-memory-bank.instructions.md` (529 lines)

**This Summary:**
- `.memory-bank/workspace/REFACTORING-SUMMARY-task-management.md`

---

**Status:** Ready for user approval and implementation.
