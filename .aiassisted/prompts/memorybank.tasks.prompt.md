# Variables

```
$ROOT_PROJECT = $(git rev-parse --show-toplevel)
$AIASSISTED_DIR = $ROOT_PROJECT/.aiassisted/
$INSTRUCTION = $AIASSISTED_DIR/instructions/multi-project-memory-bank.instructions.md
$MEMORY_BANK_PROJECTS = $ROOT_PROJECT/.memory-bank/ 
```

# Contexts 

You need to read and observe current memory bank instructions defined at: `$INSTRUCTION`.

You also need to observe and explore, current `$MEMORY_BANK_PROJECTS` directory.

# Instructions

You need to get current focused project at the: `$MEMORY_BANK_PROJECTS`.

Based on _current project_, you need to get all remaining tasks from the memory bank.

You need to return the list of tasks in the following format:

```markdown
- [ ] Task 1
- [ ] Task 2
- [ ] Task 3
```