# Variables

```
$ROOT_PROJECT = $(git rev-parse --show-toplevel)
$AIASSISTED_DIR = $ROOT_PROJECT/.aiassisted/
$INSTRUCTION = $AIASSISTED_DIR/instructions/multi-project-memory-bank.instructions.md
$MEMORY_BANK_PROJECTS = $ROOT_PROJECT/.memory-bank/ 
```

# References 

You need to read and observe current memory bank instructions defined at: `$INSTRUCTION`.

You also need to observe and explore, current `$MEMORY_BANK_PROJECTS` directory.

# Input Arguments

```
$USER_INPUT = $1
$PROJECT_NAME = Extract from `$USER_INPUT` related with the project name 
```

# Instructions 

Based on the _memory-bank_ instruction, you need to switch its focused project to the requested project name: `$PROJECT_NAME`.