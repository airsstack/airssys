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
2. ✅ **FOLLOW** the structure defined in lines 71-122 (Core Files, tasks/, docs/ structure)
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

Per instructions lines 71-122:

```
.memory-bank/sub-projects/{project}/
├── (6 CORE FILES ONLY - NO OTHER FILES AT ROOT)
│   ├── active-context.md
│   ├── product-context.md  
│   ├── progress.md
│   ├── project-brief.md
│   ├── system-patterns.md
│   └── tech-context.md
│
├── tasks/                          # Task planning & tracking
│   ├── task-[id]-[name].md
│   └── _index.md
│
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
│
└── context-snapshots/              # Historical session context
    └── YYYY-MM-DD-[description].md
```

**NO OTHER FILES OR DIRECTORIES ARE ALLOWED!**

### Naming Conventions (Per Instructions)

All files use **kebab-case** naming:

| Type | Location | Convention | Example |
|------|----------|------------|---------|
| **Task** | `tasks/` | `task-[id]-[name].md` | `task-005-block-4-security-and-isolation-layer.md` |
| **Knowledge** | `docs/knowledges/` | Per template guidelines | `knowledge-wasm-020-airssys-osl-security-integration.md` |
| **ADR** | `docs/adr/` | Per template guidelines | `adr-wasm-005-capability-based-security-model.md` |
| **Debt** | `docs/debts/` | Per template guidelines | `debt-wasm-004-task-1.3-deferred-implementation.md` |
| **Snapshot** | `context-snapshots/` | `YYYY-MM-DD-[description].md` | `2025-12-17-wasm-task-005-phase-1-planning-session.md` |

### Templates (MUST Use)

Per instructions lines 124-173:

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
- ✅ Follow the structure EXACTLY as defined (lines 71-122)
- ✅ Use templates from `templates/docs/` as specified (lines 56-63, 124-173)
- ✅ Never create files outside designated locations
- ✅ Always ask if uncertain
- ✅ 100% compliance with NO EXCEPTIONS

**Violation = Immediate correction + explanation + guarantee of no recurrence**

---

## 8. Mandatory Testing Requirements (CRITICAL - NO EXCEPTIONS)

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


**Reference:** `.aiassisted/instructions/multi-project-memory-bank.instructions.md` (lines 11-822)

