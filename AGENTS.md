```
$ROOT_PROJECT = $(git rev-parse --show-toplevel)
```

# Project Context & Agent Protocols

## 1. Project Intelligence
**AirsSys** is a collection of system programming components for the AirsStack ecosystem, designed to facilitate low-level operations and efficient performance. It includes:
- **airssys-osl**: OS Layer Framework for system programming with security and logging.
- **airssys-rt**: Lightweight Erlang-Actor model runtime system.
- **airssys-wasm**: WebAssembly pluggable system for secure component execution.
- **airssys-wasm-component**: Procedural macros for simplified WASM development.

## 2. Project Structure
```text
.
├── .aiassisted
│   ├── guidelines
│   └── instructions
├── AGENTS.md
├── PROJECTS_STANDARD.md
├── README.md
├── airssys-osl
│   ├── docs
│   └── src
├── airssys-rt
│   ├── docs
│   └── src
├── airssys-wasm
│   ├── docs
│   └── src
├── airssys-wasm-cli
│   ├── docs
│   └── src
└── airssys-wasm-component
    └── src
```

## 3. Project Standards (CRITICAL)
- **Reference**: `$ROOT_PROJECT/PROJECTS_STANDARD.md`
- **Description**: This file contains the MANDATORY project-specific standards, including code patterns, module architecture, and documentation rules. These standards OVERRIDE generic guidelines if conflicts occur.
- **Instruction**: Agents MUST read and follow these standards before writing any code.

## 4. Operational Protocols
Agents MUST follow these specific operational protocols found in `.aiassisted/instructions`:
- AI Prompt Engineering & Safety: $ROOT_PROJECT/.aiassisted/instructions/ai-prompt-engineering-safety-best-practices.instructions.md - Comprehensive guide for creating safe, effective, and unbiased prompts for AI systems.
- Multi-Project Memory Bank: $ROOT_PROJECT/.aiassisted/instructions/multi-project-memory-bank.instructions.md - Protocol for maintaining project context, documentation, and task management across multiple sub-projects.
- Rust Instructions: $ROOT_PROJECT/.aiassisted/instructions/rust.instructions.md - Detailed workflow and best practices for autonomous Rust development, including safety and testing.
- Setup Agents Context: $ROOT_PROJECT/.aiassisted/instructions/setup-agents-context.instructions.md - Instructions for generating and maintaining this AGENTS.md context file.

## 5. Guidelines & Standards
Agents MUST adhere to the following guidelines found in `.aiassisted/guidelines`:
- Diátaxis Guidelines: $ROOT_PROJECT/.aiassisted/guidelines/documentation/diataxis-guidelines.md - Framework for organizing documentation into Tutorials, How-To Guides, Reference, and Explanation.
- Documentation Quality: $ROOT_PROJECT/.aiassisted/guidelines/documentation/documentation-quality-standards.md - Standards for professional, objective, and accurate technical documentation, avoiding hyperbole.
- Task Documentation: $ROOT_PROJECT/.aiassisted/guidelines/documentation/task-documentation-standards.md - Mandatory patterns for documenting tasks, including standards compliance and technical debt.
- Microsoft Rust Guidelines: $ROOT_PROJECT/.aiassisted/guidelines/rust/microsoft-rust-guidelines.md - Production-quality Rust standards optimized for AI collaboration, covering API design and safety.

## 6. Git Commit Policy (CRITICAL - NO EXCEPTIONS)
**MANDATORY RULE**: Agents are STRICTLY FORBIDDEN from creating git commits or executing git commit commands without EXPLICIT user approval.

### Commit Workflow Requirements:
1. **Always Present Changes First**: Before any commit, agents MUST:
   - Show all modified, added, or deleted files using `git status`
   - Display the full diff of changes using `git diff` and `git diff --staged`
   - Provide a clear summary of what changed and why

2. **Await Explicit Approval**: After presenting changes, agents MUST:
   - Wait for the user to explicitly approve the commit with phrases like:
     - "commit these changes"
     - "create a commit"
     - "go ahead and commit"
   - NEVER assume approval from general statements like "looks good" or "nice work"

3. **Draft Commit Message**: Only after receiving explicit approval, agents should:
   - Analyze the changes following conventional commit standards
   - Draft a meaningful commit message
   - Present the proposed commit message to the user for review

4. **Execute Commit**: Only proceed with `git commit` after:
   - User has explicitly approved the changes
   - User has reviewed and approved the commit message (or explicitly delegated this)

### Prohibited Actions:
- Creating commits during task completion without asking
- Auto-committing after running tests or builds
- Committing as part of "cleanup" or "finalization" steps
- Using `git commit` in any automated workflow without user interaction

### Exception:
The ONLY exception is when the user explicitly requests in their initial message: "commit the changes when done" or similar unambiguous pre-approval.

## 7. Memory Bank Documentation Rules (CRITICAL - MANDATORY ENFORCEMENT)

**Authority:** `.aiassisted/instructions/multi-project-memory-bank.instructions.md`

### STRICT RULE: Follow Memory Bank Instructions EXACTLY

**Agents MUST:**
1. ✅ **READ** `.aiassisted/instructions/multi-project-memory-bank.instructions.md` BEFORE creating ANY documentation
2. ✅ **FOLLOW** the structure defined (Core Files, tasks/, docs/ structure)
3. ✅ **USE** kebab-case naming for ALL files
4. ✅ **PLACE** files in ONLY the designated locations
5. ✅ **ASK** the user if uncertain about classification or location

**Agents MUST NOT:**
- ❌ Create files based on assumptions
- ❌ Create files outside designated locations
- ❌ Create non-standard directories
- ❌ Use non-kebab-case naming
- ❌ Guess when uncertain

### Allowed File Locations (Exhaustive)

```
.memory-bank/
├── current-context.md               # Active sub-project tracker
├── workspace/                      # Workspace-level shared files
│   ├── project-brief.md
│   ├── shared-patterns.md
│   ├── workspace-architecture.md
│   └── workspace-progress.md
├── templates/                      # Documentation templates
│   └── docs/
│       ├── technical-debt-template.md
│       ├── knowledge-template.md
│       ├── adr-template.md
│       ├── documentation-guidelines.md
│       ├── debt-index-template.md
│       └── adr-index-template.md
├── context-snapshots/              # Historical session context
│   └── YYYY-MM-DD-[description].md
└── sub-projects/
    └── {project}/
        ├── (6 CORE FILES ONLY)
        │   ├── active-context.md
        │   ├── product-context.md
        │   ├── progress.md
        │   ├── project-brief.md
        │   ├── system-patterns.md
        │   └── tech-context.md
        ├── tasks/                          # Task planning & tracking
        │   ├── _index.md                   # Task registry
        │   └── <task-identifier>/          # Task directory (NEW FORMAT)
        │       ├── <task-identifier>.md    # Task file (objectives, deliverables)
        │       └── <task-identifier>.plans.md # Plans file (actions, ADR refs)
        ├── docs/                           # Technical documentation
        │   ├── knowledges/                # Architectural knowledge
        │   │   ├── [files following template]
        │   │   └── _index.md
        │   ├── adr/                       # Architecture decisions
        │   │   ├── [files following template]
        │   │   └── _index.md
        │   └── debts/                     # Technical debt
        │       ├── [files following template]
        │       └── _index.md
        └── context-snapshots/              # Sub-project session context
            └── YYYY-MM-DD-[description].md
```

**NO OTHER FILES OR DIRECTORIES ARE ALLOWED!**

### Naming Conventions (Per Instructions)

All files use **kebab-case** naming:

| Type | Location | Convention | Example |
|------|----------|------------|---------|
| **Task Directory** | `tasks/` | `<task-identifier>/` | `wasm-task-001/` |
| **Task File** | `tasks/<id>/` | `<task-identifier>.md` | `wasm-task-001.md` |
| **Plans File** | `tasks/<id>/` | `<task-identifier>.plans.md` | `wasm-task-001.plans.md` |
| **Knowledge** | `docs/knowledges/` | Per template guidelines | `knowledge-wasm-020-airssys-osl-security-integration.md` |
| **ADR** | `docs/adr/` | Per template guidelines | `adr-wasm-005-capability-based-security-model.md` |
| **Debt** | `docs/debts/` | Per template guidelines | `debt-wasm-004-task-1.3-deferred-implementation.md` |
| **Snapshot** | `context-snapshots/` | `YYYY-MM-DD-[description].md` | `2025-12-17-wasm-task-005-phase-1-planning-session.md` |

### Templates (MUST Use)

- **Technical Debt**: Use `templates/docs/technical-debt-template.md`
- **Knowledge**: Use `templates/docs/knowledge-template.md`
- **ADR**: Use `templates/docs/adr-template.md`
- **Documentation Guidelines**: Follow `templates/docs/documentation-guidelines.md`

### Mandatory Workflow

**Before creating ANY documentation:**

1. **Classify**: What type? (task, knowledge, ADR, debt, snapshot)
2. **Locate**: Which directory per instructions? (tasks/, docs/knowledges/, docs/adr/, docs/debts/, context-snapshots/)
3. **Name**: Apply kebab-case convention
4. **Template**: Use appropriate template if applicable
5. **Index**: Update corresponding `_index.md`
6. **Create**: Only if ALL steps pass

**IF UNCERTAIN:**
- ❌ **DO NOT GUESS**
- ✅ **ASK THE USER**

### Consequences of Violations

**If agent violates these rules:**
1. 🚨 User will immediately call out the violation
2. 🔧 Agent must immediately correct the violation
3. 📝 Agent must update all references
4. ⚠️ Agent must explain what was wrong
5. 💯 Agent must commit to 100% compliance going forward

### Enforcement Commitment

**This agent commits to:**
- ✅ Read Memory Bank instructions before creating ANY file
- ✅ Follow the structure EXACTLY as defined
- ✅ Use templates from `templates/docs/` as specified
- ✅ Never create files outside designated locations
- ✅ Always ask if uncertain
- ✅ 100% compliance with NO EXCEPTIONS

**Violation = Immediate correction + explanation + guarantee of no recurrence**

---

## 8. Task Management (NEW STRUCTURE - UPDATED 2026-01-04)

### Task Structure (MANDATORY - NEW FORMAT)

**CRITICAL RULE:**
- Each task = ONE directory
- Two files per task: task file + plans file
- **SINGLE action per task** - DO ONE THING, DO IT RIGHT
- Plans MUST reference ADRs and Knowledge documents

### Directory Structure

```
tasks/
├── _index.md                              # Task registry
└── <task-identifier>/                    # Task directory (e.g., wasm-task-001/)
    ├── <task-identifier>.md              # Task file (objectives, deliverables, checklist)
    └── <task-identifier>.plans.md        # Plans file (actions with ADR/Knowledge references)
```

### Task File Format (<task-identifier>.md)

**Contains:**
- Task metadata (status, dates, priority, duration)
- Original request
- Thought process
- Deliverables checklist
- Success criteria
- Progress tracking
- Standards compliance checklist
- Definition of done

### Plans File Format (<task-identifier>.plans.md)

**Contains:**
- Detailed implementation plan
- Step-by-step actions
- References to ADRs and Knowledge documents
- Verification commands to run

### Task Registry (_index.md)

**Format:**
```markdown
# Tasks Index

## Pending
- [task-001] task-name - Task description (YYYY-MM-DD)

## In Progress
- [task-002] task-name - Task description (YYYY-MM-DD)

## Completed
- [task-003] task-name - Task description (YYYY-MM-DD)

## Abandoned
- [task-004] task-name - Task description (YYYY-MM-DD)
```

### Single Action Rule (MANDATORY)

**CRITICAL:**
- Each task contains EXACTLY ONE action
- NO multiple objectives per task
- NO mixed deliverables
- DO ONE THING, DO IT RIGHT

**Examples:**
- ✅ CORRECT: "Setup airssys-wasm project directory" (single action)
- ✅ CORRECT: "Implement core/ types module" (single action)
- ✅ CORRECT: "Write unit tests for ComponentMessage" (single action)
- ❌ WRONG: "Setup project AND implement core types" (two actions - split into two tasks)
- ❌ WRONG: "Implement actor system integration" (too broad - break into smaller tasks)

### Plan References Rule (MANDATORY)

**CRITICAL:**
- EVERY plan MUST reference relevant ADRs
- EVERY plan MUST reference relevant Knowledge documents
- NO assumptions - all decisions backed by documentation

### Task Update Protocol

When working on a task:
1. Update `<task-id>.md` progress tracking
2. Add progress log entry with date
3. Update task status (pending/in_progress/complete)
4. Update `_index.md` task list
5. NEVER modify `<task-id>.plans.md` after approved

### Task Commands

- `show-tasks [sub-project]` - Display all tasks
- `show-task [sub-project] [task-id]` - Display task details and plans
- `update-task [sub-project] [task-id]` - Update task progress

---

## 9. Mandatory Testing Requirements (CRITICAL - NO EXCEPTIONS)

### The Testing Mandate

**ZERO EXCEPTIONS POLICY**: No code is considered complete without BOTH unit tests AND integration tests.

This is enforced across ALL agents and ALL tasks. There are NO waivers, NO compromises, NO shortcuts.

### What Must Be True For Code to Be "Complete":

1. ✅ **Unit Tests Exist** - In module #[cfg(test)] blocks
   - Test success paths
   - Test error/edge cases  
   - Test actual functionality (not just APIs)
   - All passing: `cargo test --lib`

2. ✅ **Integration Tests Exist** - In tests/ directory
   - Test end-to-end workflows
   - Test component/module interaction
   - Test real message/data flow
   - All passing: `cargo test --test [name]`

3. ✅ **Code Quality** - Zero warnings
   - `cargo build` - clean build
   - `cargo clippy --all-targets --all-features -- -D warnings` - zero warnings

### What Does NOT Count as Complete:

- ❌ Tests that only validate helper APIs or configuration
- ❌ Missing unit tests OR missing integration tests (BOTH required)
- ❌ Failing tests
- ❌ Incomplete/placeholder tests
- ❌ Code with compiler warnings
- ❌ Code with clippy warnings

### Enforcement Points:

**PLANNING** (@memorybank-planner):
- ❌ REJECT plans without Unit Testing Plan section
- ❌ REJECT plans without Integration Testing Plan section
- Must specify what will be tested and how

**IMPLEMENTATION** (@memorybank-implementer):
- 🛑 HALT if unit tests missing from module
- 🛑 HALT if integration tests missing from tests/
- 🛑 HALT if any tests failing
- 🛑 HALT if any compiler/clippy warnings

**REVIEW** (@rust-reviewer):
- 🛑 REJECT code without BOTH unit AND integration tests
- 🛑 REJECT if tests are failing
- 🛑 REJECT if tests only validate APIs (not functionality)
- 🛑 REJECT if any warnings present

**COMPLETION** (@memorybank-auditor):
- 🛑 HALT task completion if unit tests missing
- 🛑 HALT task completion if integration tests missing
- 🛑 HALT task completion if any tests failing
- 🛑 HALT task completion if tests only validate APIs
- ✅ REQUIRE test results in completion summary

### Test Quality Requirements:

**UNIT TESTS MUST:**
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_feature_success_path() {
        // Instantiate real types
        // Call functions with valid inputs
        // Assert expected behavior
    }
    
    #[test]
    fn test_feature_error_case() {
        // Test error handling
        // Verify error messages/types
    }
    
    #[test]
    fn test_feature_edge_cases() {
        // Test boundary conditions
        // Test special values
    }
}
```

**INTEGRATION TESTS MUST:**
```rust
// tests/messaging-integration-tests.rs
#[test]
fn test_end_to_end_message_flow() {
    // Create real components
    // Send actual messages
    // Verify actual behavior
    // Show complete workflow
}
```

**INTEGRATION TESTS MUST NOT:**
```rust
// ❌ WRONG - Only tests metrics API
#[test]
fn test_metrics_snapshot() {
    let metrics = Metrics::new();
    metrics.record_something();
    assert_eq!(metrics.snapshot().count, 1);
}
```

### Verification Commands:

Every completed task must verify:
```bash
# All unit tests pass
cargo test --lib

# All integration tests pass  
cargo test --test '*'

# Build is clean
cargo build

# Zero warnings
cargo clippy --all-targets --all-features -- -D warnings
```

### Non-Negotiable Commitment

**This policy is ABSOLUTE:**
- ✅ Every task requires testing
- ✅ Every code change requires testing
- ✅ Every plan must include testing
- ✅ Every implementation must include testing
- ✅ Every review must verify testing
- ✅ Every completion must verify testing

**Violations are NOT acceptable and will be escalated immediately.**

---

## 10. Project High-Level Overview (CRITICAL - MUST UNDERSTAND)

### ⚠️ MANDATORY: Understand What Each Project IS Before Any Work

**Every agent MUST understand the high-level purpose of each sub-project BEFORE planning, implementing, reviewing, or auditing any task.**

---

### 10.1 airssys-wasm: WASM Plugin/Extension Platform

**What It Is:**
A WebAssembly-based plugin/extension platform similar to smart contracts on NEAR or Polkadot. It allows third-party developers to write sandboxed, secure extensions that run within a host application.

**Core Concept - The Two Entities:**
```
┌──────────────────────────────────────────────────────────────────┐
│                         HOST APPLICATION                         │
│  (Your Rust application using airssys-wasm as a library)        │
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐           │
│  │  Component A │  │  Component B │  │  Component C │           │
│  │  (WASM)      │  │  (WASM)      │  │  (WASM)      │           │
│  │  = 1 Actor   │  │  = 1 Actor   │  │  = 1 Actor   │           │
│  │              │  │              │  │              │           │
│  │ ┌──────────┐ │  │ ┌──────────┐ │  │ ┌──────────┐ │           │
│  │ │ Mailbox  │ │  │ │ Mailbox  │ │  │ │ Mailbox  │ │           │
│  │ └──────────┘ │  │ └──────────┘ │  │ └──────────┘ │           │
│  │ ┌──────────┐ │  │ ┌──────────┐ │  │ ┌──────────┐ │           │
│  │ │ Storage  │ │  │ │ Storage  │ │  │ │ Storage  │ │           │
│  │ │(isolated)│ │  │ │(isolated)│ │  │ │(isolated)│ │           │
│  │ └──────────┘ │  │ └──────────┘ │  │ └──────────┘ │           │
│  └──────────────┘  └──────────────┘  └──────────────┘           │
│                                                                  │
│  Communication: Erlang-style mailbox messaging (via airssys-rt) │
└──────────────────────────────────────────────────────────────────┘
```

**Key Architectural Principles:**
1. **Each WASM Component = One Actor** (managed by airssys-rt)
2. **Isolated Storage** - Each component has its own persistent storage namespace
3. **Mailbox Communication** - Components communicate via async message passing
4. **Deny-by-Default Security** - Components have NO capabilities until explicitly granted
5. **WIT Interfaces** - Components interact with host via WebAssembly Interface Types

**Inspiration:** Smart contract platforms (NEAR, Polkadot, Solana)

**Reference Documents:**
- `KNOWLEDGE-WASM-031`: Foundational Architecture (READ FIRST)
- `KNOWLEDGE-WASM-002`: High-Level Overview
- `ADR-WASM-018`: Three-Layer Architecture

---

### 10.2 airssys-rt: Erlang-Actor Model Runtime

**What It Is:**
A lightweight actor system inspired by Erlang/BEAM. Provides the foundation for concurrent, fault-tolerant applications.

**Core Concepts:**
- **Actors**: Lightweight processes with private state
- **Messages**: Immutable, async communication between actors
- **Mailboxes**: Queue of messages processed one at a time
- **Supervisors**: Manage actor lifecycle and restart on failure

**Why airssys-wasm Uses It:**
- Each WASM component is wrapped in an actor (ComponentActor)
- Message routing between components uses the MessageBroker
- Supervision provides fault tolerance for component crashes

---

### 10.3 airssys-osl: OS Layer Framework

**What It Is:**
Operating system abstraction layer providing secure access to filesystem, network, and process management with comprehensive audit logging.

**Why airssys-wasm Uses It:**
- Security policies (ACL, RBAC)
- Capability validation
- Audit logging for all component operations
- Filesystem access mediation

---

## 11. Module Responsibility Maps (CRITICAL - MUST FOLLOW)

### ⚠️ MANDATORY: Know Module Boundaries Before Writing ANY Code

---

### 11.1 airssys-wasm Module Architecture

**The Four Root Modules:**

```
┌─────────────────────────────────────────────────────────────────────┐
│  DEPENDENCY DIRECTION (ONE WAY ONLY - NO REVERSE IMPORTS)          │
│                                                                     │
│   actor/  ───────►  runtime/  ───────►  security/  ───────►  core/ │
│     │                  │                    │                  │    │
│     │                  │                    │                  │    │
│     └──────────────────┴────────────────────┴──────────────────┘    │
│                         ALL can import from core/                   │
└─────────────────────────────────────────────────────────────────────┘
```

**Module Responsibilities:**

| Module | Purpose | OWNS | DOES NOT OWN |
|--------|---------|------|--------------|
| `core/` | Shared types & abstractions | ComponentId, ComponentMessage, traits, errors, configs | Any implementation logic |
| `security/` | Security logic | Capabilities, permissions, policies, validation | WASM execution, messaging |
| `runtime/` | WASM execution engine | WasmEngine, ComponentLoader, host functions | Message routing, actor lifecycle |
| `actor/` | Actor system integration | ComponentActor, ComponentRegistry, MessagePublisher, CorrelationTracker | WASM internals |

**FORBIDDEN IMPORTS (ADR-WASM-023):**

| ❌ FORBIDDEN | Reason |
|--------------|--------|
| `runtime/` → `actor/` | Runtime is lower level than actor |
| `security/` → `runtime/` | Security is lower level than runtime |
| `security/` → `actor/` | Security is lower level than actor |
| `core/` → anything | Core is the foundation, imports nothing |

**Verification Commands:**
```bash
# ALL MUST RETURN NOTHING for valid architecture
grep -rn "use crate::runtime" airssys-wasm/src/core/
grep -rn "use crate::actor" airssys-wasm/src/core/
grep -rn "use crate::security" airssys-wasm/src/core/
grep -rn "use crate::actor" airssys-wasm/src/runtime/
grep -rn "use crate::actor" airssys-wasm/src/security/
grep -rn "use crate::runtime" airssys-wasm/src/security/
```

**Reference Documents:**
- `ADR-WASM-023`: Module Boundary Enforcement (MANDATORY)
- `KNOWLEDGE-WASM-030`: Module Architecture Hard Requirements
- `KNOWLEDGE-WASM-032`: Module Boundary Violations Audit

---

## 12. MANDATORY ADR/Knowledge Reference Requirement (CRITICAL - NO ASSUMPTIONS)

### ⚠️ THE GOLDEN RULE: NO ADR/KNOWLEDGE = NO ASSUMPTIONS = ASK USER

**BEFORE planning, implementing, reviewing, or auditing ANY task:**

1. ✅ **IDENTIFY** relevant ADRs in `.memory-bank/sub-projects/[project]/docs/adr/`
2. ✅ **IDENTIFY** relevant Knowledges in `.memory-bank/sub-projects/[project]/docs/knowledges/`
3. ✅ **READ** all relevant documents COMPLETELY
4. ✅ **EXTRACT** architectural constraints and requirements
5. ✅ **VERIFY** your understanding matches documented architecture

### IF NO RELEVANT ADRs OR KNOWLEDGES EXIST:

```
🛑 STOP - DO NOT PROCEED WITH ASSUMPTIONS

❓ ASK: "I cannot find ADRs or Knowledges for [topic]. 
   Should I proceed with assumptions, or do you want to 
   create these references first?"
```

### Key Reference Documents by Topic:

**Module Architecture:**
- `ADR-WASM-023`: Module Boundary Enforcement
- `KNOWLEDGE-WASM-030`: Module Architecture Hard Requirements

**Three-Layer Architecture:**
- `ADR-WASM-018`: Three-Layer Architecture
- `KNOWLEDGE-WASM-018`: Component Definitions

**Messaging:**
- `ADR-WASM-009`: Component Communication Model
- `KNOWLEDGE-WASM-005`: Messaging Architecture
- `KNOWLEDGE-WASM-024`: Component Messaging Clarifications
- `KNOWLEDGE-WASM-029`: Messaging Patterns

**Security:**
- `ADR-WASM-005`: Capability-Based Security Model
- `KNOWLEDGE-WASM-020`: airssys-osl Security Integration

**Runtime Dependencies:**
- `ADR-WASM-019`: Runtime Dependency Management
- `KNOWLEDGE-WASM-019`: Runtime Dependency Architecture

### Index Files for Quick Lookup:

- **ADR Index**: `.memory-bank/sub-projects/[project]/docs/adr/_index.md`
- **Knowledge Index**: `.memory-bank/sub-projects/[project]/docs/knowledges/_index.md`

### Example Workflow:

```
Task: "Implement message routing in runtime/"

Step 1: Check ADR index for messaging-related ADRs
  → Found: ADR-WASM-009, ADR-WASM-020

Step 2: Check Knowledge index for messaging-related docs
  → Found: KNOWLEDGE-WASM-005, KNOWLEDGE-WASM-024, KNOWLEDGE-WASM-026

Step 3: Read ADR-WASM-009
  → Learn: Messages route through MessageBroker
  → Learn: ActorSystemSubscriber handles delivery

Step 4: Check module boundaries (ADR-WASM-023)
  → Learn: runtime/ CANNOT import from actor/
  → Realize: Message routing belongs in actor/, NOT runtime/

Step 5: Ask user if unclear
  → "ADR-WASM-023 says runtime/ cannot import from actor/. 
     This means message routing should be in actor/. 
     Should I implement there instead?"

Result: Correct implementation, no architecture violations
```

### NEVER DO THIS:

```
❌ "I'll implement message routing in runtime/ because it seems logical"
   → Wrong! Violates ADR-WASM-023

❌ "I don't see an ADR for this, so I'll make my own design decisions"
   → Wrong! Should ask user first

❌ "The ADR says X but I think Y is better"
   → Wrong! ADRs are authoritative unless user explicitly overrides
```

---

## 13. Architecture Verification Requirements (CRITICAL - MANDATORY FOR ALL CODE)

### ⚠️ EVERY Code Change MUST Pass Architecture Verification

**Before ANY code is considered complete, run these verification commands:**

### 13.1 airssys-wasm Architecture Verification

```bash
# Module Boundary Check (ADR-WASM-023)
# ALL MUST RETURN NOTHING

# Check 1: core/ has no forbidden imports
grep -rn "use crate::runtime" airssys-wasm/src/core/
grep -rn "use crate::actor" airssys-wasm/src/core/
grep -rn "use crate::security" airssys-wasm/src/core/

# Check 2: security/ has no forbidden imports
grep -rn "use crate::runtime" airssys-wasm/src/security/
grep -rn "use crate::actor" airssys-wasm/src/security/

# Check 3: runtime/ has no forbidden imports
grep -rn "use crate::actor" airssys-wasm/src/runtime/
```

**If ANY command returns results → CODE IS REJECTED**

### 13.2 Enforcement at Each Stage

**PLANNING** (@memorybank-planner):
- ❌ REJECT plans that would create forbidden imports
- ✅ Verify planned code locations match module responsibilities
- ✅ Reference ADR-WASM-023 in plan

**IMPLEMENTATION** (@memorybank-implementer):
- 🛑 HALT if about to create forbidden import
- ✅ Run architecture verification after each step
- ✅ Show grep output as proof

**REVIEW** (@rust-reviewer):
- 🛑 REJECT code with architecture violations
- ✅ Run all verification commands
- ✅ Include verification output in review

**AUDIT** (@memorybank-auditor):
- 🛑 HALT audit if architecture violations exist
- ✅ Run verification commands and include output
- ✅ Architecture verification = mandatory section in audit report

---

**Reference:** `.aiassisted/instructions/multi-project-memory-bank.instructions.md`
