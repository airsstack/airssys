# Instructions: Generate AGENTS.md Context

**Objective**: Generate or update the `AGENTS.md` file at `$ROOT_PROJECT/AGENTS.md`. This file serves as the primary entry point for AI agents, providing high-level context, a visual project structure, and an index of available documentation.

## 1. Preparation
- **Read**: `README.md` (for project summary).
- **Scan**: `.aiassisted/guidelines/` and `.aiassisted/instructions/` (for documentation index).
- **Scan**: Project directory structure (to generate a tree view).

## 2. Output Format (AGENTS.md)
You MUST generate the file using the following structure. Do not include the full content of referenced files; use links only.

### Section 1: Context Variables
Include this exact block at the top:
Include this exact block at the top:
```markdown
```
$ROOT_PROJECT = $(git rev-parse --show-toplevel)
```
```

### Section 2: Project Intelligence
- Write a **concise summary** (max 5 lines) of the project based on `README.md`.
- Focus on: What is this project? What is its architecture? What are the core components?

### Section 3: Project Structure
- Generate a tree view of the project structure.
- Use a command like `tree -L 2` or `ls -R` logic to show the layout.
- **IMPORTANT**: Do not hardcode the output. Use a placeholder or a comment indicating that the agent should dynamically generate this view when reading the file, OR if generating a static file, run the command to populate it.
- If generating a static artifact, the content should look like:
```text
.
├── .aiassisted
│   ├── guidelines
│   └── instructions
├── src
│   └── ...
└── ...
```

### Section 4: Project Standards (CRITICAL)
- **Reference**: `$ROOT_PROJECT/PROJECTS_STANDARD.md`
- **Description**: "This file contains the MANDATORY project-specific standards, including code patterns, module architecture, and documentation rules. These standards OVERRIDE generic guidelines if conflicts occur."
- **Instruction**: "Agents MUST read and follow these standards before writing any code."

### Section 5: Operational Protocols
- List all files found in `.aiassisted/instructions/`.
- Format: `- <Title>: $ROOT_PROJECT/.aiassisted/instructions/<filename> - <Short Summary>`

### Section 6: Guidelines & Standards
- List all files found in `.aiassisted/guidelines/`.
- Format: `- <Title>: $ROOT_PROJECT/.aiassisted/guidelines/<filename> - <Short Summary>`

## 3. Constraints
- **NO Context Pollution**: Do NOT copy-paste content from guidelines or instructions.
- **Links**: ALWAYS use the `$ROOT_PROJECT` variable prefix for paths (e.g., `$ROOT_PROJECT/.aiassisted/guidelines/naming.md`).
- **Generic**: Do not hardcode project-specific details (like "Airssys"); derive them from the current project files.
