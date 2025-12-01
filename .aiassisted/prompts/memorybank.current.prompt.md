# Variables

```
$ROOT_PROJECT = $(git rev-parse --show-toplevel)
$AIASSISTED_DIR = $ROOT_PROJECT/.aiassisted/
$INSTRUCTION = $AIASSISTED_DIR/instructions/multi-project-memory-bank.instructions.md
$MEMORY_BANK_PROJECTS = $ROOT_PROJECT/.memory-bank/ 
```

# References 

You need to read and observe current memory bank instructions defined at: `$INSTRUCTION`.

# Instructions 

Based on the original _memory-bank_ instructions, you need to get current focused project from `$MEMORY_BANK_PROJECTS` directory, it's summary and also its remaining tasks.

For the remaining tasks, you need to get the task name, description, priority and status. Do not need to give users too much information.
