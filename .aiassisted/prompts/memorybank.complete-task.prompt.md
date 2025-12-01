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

# Input Arguments

```
$USER_INPUT = $1
$TASK_ID = Extract from `$USER_INPUT` related with the task id 
```

# Instructions

You need to get current focused project at the: `$MEMORY_BANK_PROJECTS`.

You need to check if given `$TASK_ID` is valid or not and need to ensure that it has already _implemented_ or not.

If given `$TASK_ID` is valid and implemented, you need to mark it as _completed_ and also need to update the memory bank states.

If given `$TASK_ID` is valid but not implemented, you need to return the error message.