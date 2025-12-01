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

You need to get current focused project at the: `$MEMORY_BANK_PROJECTS`.  You need to check if given `$TASK_ID` already has their action or implementation plans or not.

If given `$TASK_ID` already has their action or implementation plans, you need to return the action or implementation plans.

If given `$TASK_ID` does not have their action or implementation plans, you need to start analyze it's action and implementation plans based on current project's knowledges and ADR documents. Once you've done that, you need to return the action or implementation plans and asking for user's permissions to save the plans.