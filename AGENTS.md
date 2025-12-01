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
