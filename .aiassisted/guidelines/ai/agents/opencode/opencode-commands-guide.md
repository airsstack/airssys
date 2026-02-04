# OpenCode Custom Commands Guide

## Overview

Custom commands in OpenCode allow you to create reusable prompts for repetitive tasks. Commands are invoked with `/` prefix in the TUI and can accept arguments, inject shell output, and reference files.

## Built-in Commands

OpenCode includes several built-in commands:
- `/init`: Initialize project
- `/undo`: Undo last action
- `/redo`: Redo last undone action
- `/share`: Share session
- `/help`: Show help

**Note**: Custom commands can override built-in commands if they share the same name.

## Configuration Methods

### Method 1: JSON Configuration

Define commands in `opencode.json`:

```json
{
  "$schema": "https://opencode.ai/config.json",
  "command": {
    "test": {
      "template": "Run the full test suite with coverage report and show any failures.\nFocus on the failing tests and suggest fixes.",
      "description": "Run tests with coverage",
      "agent": "build",
      "model": "anthropic/claude-3-5-sonnet-20241022"
    }
  }
}
```

Usage: `/test`

### Method 2: Markdown Files

Create markdown files in:
- **Global**: `~/.config/opencode/commands/`
- **Per-project**: `.opencode/commands/`

**File name becomes command name** (e.g., `test.md` → `/test`)

Example `.opencode/commands/test.md`:

```markdown
---
description: Run tests with coverage
agent: build
model: anthropic/claude-3-5-sonnet-20241022
---

Run the full test suite with coverage report and show any failures.
Focus on the failing tests and suggest fixes.
```

## Configuration Options

### Required Options

#### `template`
The prompt sent to the LLM when command is executed.

```json
{
  "command": {
    "test": {
      "template": "Run the full test suite with coverage report and show any failures.\nFocus on the failing tests and suggest fixes."
    }
  }
}
```

**For markdown files**: The markdown content (after frontmatter) becomes the template.

### Optional Options

#### `description`
Brief description shown in TUI when typing the command.

```json
{
  "command": {
    "test": {
      "description": "Run tests with coverage"
    }
  }
}
```

#### `agent`
Specify which agent should execute the command.

```json
{
  "command": {
    "review": {
      "agent": "plan"
    }
  }
}
```

- If agent is a **subagent** and `subtask` is not `false`, command triggers subagent invocation
- **Default**: Uses current agent if not specified

#### `subtask`
Force command to trigger subagent invocation.

```json
{
  "command": {
    "analyze": {
      "subtask": true
    }
  }
}
```

Forces agent to act as subagent even if `mode` is `primary` in agent configuration.

#### `model`
Override default model for this command.

```json
{
  "command": {
    "analyze": {
      "model": "anthropic/claude-3-5-sonnet-20241022"
    }
  }
}
```

## Template Syntax

Commands support special placeholders and syntax for dynamic content.

### Arguments

#### All Arguments: `$ARGUMENTS`
Pass all arguments to command using `$ARGUMENTS` placeholder.

`.opencode/commands/component.md`:
```markdown
---
description: Create a new component
---

Create a new React component named $ARGUMENTS with TypeScript support.
Include proper typing and basic structure.
```

Usage: `/component Button`
Result: `$ARGUMENTS` replaced with `Button`

#### Positional Arguments: `$1`, `$2`, `$3`, etc.
Access individual arguments using positional parameters.

`.opencode/commands/create-file.md`:
```markdown
---
description: Create a new file with content
---

Create a file named $1 in the directory $2
with the following content: $3
```

Usage: `/create-file config.json src "{ \"key\": \"value\" }"`

Replacements:
- `$1` → `config.json`
- `$2` → `src`
- `$3` → `{ "key": "value" }`

### Shell Output: `` !`command` ``

Inject bash command output into prompt using `` !`command` `` syntax.

#### Test Coverage Analysis

`.opencode/commands/analyze-coverage.md`:
```markdown
---
description: Analyze test coverage
---

Here are the current test results:
!`npm test`

Based on these results, suggest improvements to increase coverage.
```

#### Review Recent Changes

`.opencode/commands/review-changes.md`:
```markdown
---
description: Review recent changes
---

Recent git commits:
!`git log --oneline -10`

Review these changes and suggest any improvements.
```

**Execution Context**: Commands run in your project's root directory.

### File References: `@filename`

Include file content in command using `@` followed by filename.

`.opencode/commands/review-component.md`:
```markdown
---
description: Review component
---

Review the component in @src/components/Button.tsx.
Check for performance issues and suggest improvements.
```

The file content is automatically included in the prompt.

## Example Commands

### Testing Commands

#### Run Tests with Coverage

`.opencode/commands/test.md`:
```markdown
---
description: Run tests with coverage report
agent: build
---

Run the full test suite with coverage reporting.

!`npm test -- --coverage`

Analyze the coverage report and:
1. Identify files with low coverage
2. Suggest additional test cases
3. Highlight critical untested paths
```

Usage: `/test`

#### Test Specific File

`.opencode/commands/test-file.md`:
```markdown
---
description: Run tests for a specific file
agent: build
---

Run tests for $ARGUMENTS and analyze results.

!`npm test $ARGUMENTS`

If tests fail, suggest fixes based on the error output.
```

Usage: `/test-file src/utils/parser.test.ts`

### Code Review Commands

#### Review Pull Request

`.opencode/commands/review-pr.md`:
```markdown
---
description: Review current pull request
agent: plan
subtask: true
---

Review the current pull request changes:

!`git diff main...HEAD`

Analyze for:
- Code quality and best practices
- Potential bugs
- Performance implications
- Security concerns

Provide constructive feedback.
```

Usage: `/review-pr`

#### Review Uncommitted Changes

`.opencode/commands/review-changes.md`:
```markdown
---
description: Review uncommitted changes
agent: plan
---

Review uncommitted changes:

!`git diff`

Staged changes:

!`git diff --staged`

Provide feedback on code quality and suggest improvements.
```

Usage: `/review-changes`

### Documentation Commands

#### Generate Documentation

`.opencode/commands/doc.md`:
```markdown
---
description: Generate documentation for a file
agent: docs-writer
---

Generate comprehensive documentation for @$ARGUMENTS

Include:
- Module/class overview
- Function/method documentation
- Usage examples
- Type definitions
```

Usage: `/doc src/api/client.ts`

#### Update README

`.opencode/commands/update-readme.md`:
```markdown
---
description: Update README with recent changes
---

Current README:
@README.md

Recent changes:
!`git log --oneline -5`

Update the README to reflect recent changes and improvements.
Maintain the existing structure and style.
```

Usage: `/update-readme`

### Refactoring Commands

#### Extract Function

`.opencode/commands/extract.md`:
```markdown
---
description: Extract code into a new function
agent: build
---

In file @$1, extract the following code into a new function named $2:

$3

Consider:
- Appropriate parameter types
- Return type
- Error handling
- Side effects
```

Usage: `/extract src/utils.ts validateInput "if (!input) { throw new Error('Invalid'); }"`

#### Rename Symbol

`.opencode/commands/rename.md`:
```markdown
---
description: Rename a symbol across the codebase
agent: build
---

Rename "$1" to "$2" across the entire codebase.

Current usages:
!`git grep -n "$1"`

Update all occurrences while maintaining functionality.
```

Usage: `/rename oldFunctionName newFunctionName`

### Analysis Commands

#### Analyze Performance

`.opencode/commands/perf.md`:
```markdown
---
description: Analyze code performance
agent: plan
temperature: 0.1
---

Analyze performance of @$ARGUMENTS

!`git log --oneline -3 -- $ARGUMENTS`

Look for:
- Algorithmic complexity issues
- Unnecessary iterations or allocations
- Potential caching opportunities
- Database query optimization
- Memory usage concerns

Provide specific optimization recommendations.
```

Usage: `/perf src/services/data-processor.ts`

#### Security Audit

`.opencode/commands/security.md`:
```markdown
---
description: Perform security audit
agent: security-auditor
temperature: 0.1
---

Perform security audit of @$ARGUMENTS

Focus on:
- Input validation
- SQL injection vectors
- XSS vulnerabilities
- Authentication/authorization issues
- Sensitive data exposure
- Dependency vulnerabilities

Provide severity ratings and remediation steps.
```

Usage: `/security src/api/auth.ts`

### Git Workflow Commands

#### Create Commit Message

`.opencode/commands/commit.md`:
```markdown
---
description: Generate conventional commit message
agent: plan
---

Current staged changes:
!`git diff --staged`

Generate a conventional commit message following this format:
- type(scope): description
- Optional body explaining the changes
- Optional footer for breaking changes

Types: feat, fix, docs, refactor, test, chore
```

Usage: `/commit`

#### Prepare Release

`.opencode/commands/release.md`:
```markdown
---
description: Prepare release notes
agent: plan
---

Commits since last tag:
!`git log $(git describe --tags --abbrev=0)..HEAD --oneline`

Full diff:
!`git diff $(git describe --tags --abbrev=0)..HEAD`

Generate release notes with:
- Summary of changes
- Breaking changes (if any)
- New features
- Bug fixes
- Suggested version bump (semver)
```

Usage: `/release`

### Project Setup Commands

#### Initialize Project

`.opencode/commands/init-project.md`:
```markdown
---
description: Initialize new project with best practices
agent: build
---

Initialize a new $1 project named $2 with:
- Modern tooling and dependencies
- Linting and formatting configuration
- Testing setup
- CI/CD pipeline basics
- README with getting started guide

Use current best practices for $1 development.
```

Usage: `/init-project typescript my-api`

## Advanced Patterns

### Combining Multiple Features

`.opencode/commands/review-function.md`:
```markdown
---
description: Deep review of a specific function
agent: plan
temperature: 0.1
---

Function to review in @$1:

!`git grep -A 30 "function $2" $1`

Git history:
!`git log --oneline -5 -- $1`

Analyze:
1. Algorithm correctness
2. Edge cases handling
3. Performance characteristics
4. Code clarity and maintainability
5. Test coverage for this function

!`git grep -r "test.*$2" tests/`
```

Usage: `/review-function src/parser.ts parseExpression`

### Multi-File Analysis

`.opencode/commands/compare.md`:
```markdown
---
description: Compare two implementations
agent: plan
---

First implementation:
@$1

Second implementation:
@$2

Compare these implementations considering:
- Performance differences
- Maintainability
- Error handling
- Test coverage
- API design

Recommend which approach to use and why.
```

Usage: `/compare src/v1/handler.ts src/v2/handler.ts`

### Conditional Logic via Shell

`.opencode/commands/check-status.md`:
```markdown
---
description: Check project health
agent: plan
---

Git status:
!`git status --porcelain`

Test results:
!`npm test 2>&1 | tail -20 || echo "Tests failed"`

Lint status:
!`npm run lint 2>&1 | tail -20 || echo "Linting failed"`

Build status:
!`npm run build 2>&1 | tail -20 || echo "Build failed"`

Analyze the overall project health and suggest next steps.
```

Usage: `/check-status`

## Best Practices

### Command Design
1. **Single Purpose**: Each command should do one thing well
2. **Descriptive Names**: Use clear, action-oriented names (e.g., `/test`, `/review`, `/deploy`)
3. **Good Descriptions**: Write helpful descriptions for autocomplete
4. **Appropriate Agent**: Choose the right agent for the task (plan for analysis, build for changes)

### Template Writing
1. **Clear Instructions**: Be specific about what you want the agent to do
2. **Structured Output**: Request structured responses when needed
3. **Context Inclusion**: Use shell output and file references to provide necessary context
4. **Error Handling**: Consider edge cases in your templates

### Argument Usage
1. **Validate in Prompt**: Include validation instructions in the template
2. **Document Expected Args**: Use description to indicate what arguments are needed
3. **Provide Examples**: Show example usage in comments or documentation
4. **Use Positional for Clarity**: Use `$1`, `$2` when argument order matters

### Shell Command Safety
1. **Read-Only by Default**: Prefer read-only commands (git log, grep, etc.)
2. **Avoid Destructive Ops**: Don't include rm, force push, etc. unless absolutely necessary
3. **Error Tolerance**: Use `|| echo` to handle command failures gracefully
4. **Limit Output**: Use `head`, `tail`, or `--oneline` to keep output manageable

### Organization
1. **Global vs Project**: Put reusable commands in global config, project-specific in `.opencode/`
2. **Markdown for Long Prompts**: Use markdown files for complex multi-line templates
3. **JSON for Simple**: Use JSON config for short, simple commands
4. **Group Related Commands**: Use prefixes for related commands (e.g., `test-*`, `deploy-*`)

## Troubleshooting

### Command Not Found
- Check file name matches command name (without `/`)
- Verify frontmatter syntax is valid YAML
- Ensure file is in correct directory (`~/.config/opencode/commands/` or `.opencode/commands/`)

### Shell Output Not Working
- Verify backticks are correct: `` !`command` ``
- Check command runs successfully in terminal first
- Commands execute in project root directory
- Use absolute paths if needed

### File References Not Working
- Ensure file path is correct relative to project root
- Check file exists and is readable
- Use forward slashes even on Windows

### Arguments Not Substituting
- Verify placeholder syntax: `$ARGUMENTS`, `$1`, `$2`, etc.
- Check that arguments are being passed when invoking command
- Quote arguments with spaces: `/command "arg with spaces"`

### Wrong Agent Used
- Check `agent` config in frontmatter or JSON
- Verify agent name exists and is enabled
- If using `subtask: true`, ensure agent can act as subagent

## Additional Resources

- [OpenCode Commands Documentation](https://opencode.ai/docs/commands/)
- [OpenCode Agents Documentation](https://opencode.ai/docs/agents/)
- [OpenCode TUI Commands](https://opencode.ai/docs/tui#commands)
