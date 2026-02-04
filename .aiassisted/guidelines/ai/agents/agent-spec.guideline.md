---
description: Specification for creating platform-agnostic AI agents that compile to Claude Code and OpenCode formats.
globs: "**/*"
---

# Agent Specification

This document defines the standard format for creating platform-agnostic AI agents. Agents are stored in a portable format and compiled to platform-specific configurations during setup.

## Overview

An agent is a folder containing an `AGENT.md` file with metadata and a system prompt. The format is designed to be:

- **Platform-agnostic**: Single source compiles to multiple platforms
- **Portable**: Just files, easy to edit, version, and share
- **Validated**: References to skills are validated against the skill registry

## Directory Structure

```
.aiassisted/
└── agents/
    └── agent-name/
        └── AGENT.md
```

Each agent lives in its own directory. The directory name must match the `name` field in the frontmatter.

## AGENT.md Format

Every agent requires an `AGENT.md` file containing YAML frontmatter and a Markdown system prompt.

### Frontmatter Fields

```yaml
---
name: code-reviewer
description: Reviews code for quality, security, and best practices. Use after writing or modifying code.
capabilities: read-only
model-tier: balanced
skills:
  - review-codes
---
```

| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| `name` | Yes | - | Unique identifier, must match directory name |
| `description` | Yes | - | When to use this agent (max 1024 chars) |
| `capabilities` | No | `read-write` | Tool access level |
| `model-tier` | No | `balanced` | Model selection hint |
| `skills` | No | `[]` | Skills to preload (validated against registry) |

### Name Field Rules

The `name` field must follow these rules:

- 1-64 characters in length
- Only lowercase alphanumeric characters and hyphens (`a-z`, `0-9`, `-`)
- Cannot start or end with a hyphen
- Cannot contain consecutive hyphens (`--`)
- Must match the parent directory name

**Valid examples:**
```yaml
name: code-reviewer
name: debugger
name: test-runner
```

**Invalid examples:**
```yaml
name: Code-Reviewer    # uppercase not allowed
name: -debugger        # cannot start with hyphen
name: test--runner     # consecutive hyphens not allowed
```

### Capabilities Field

Controls what tools the agent can access:

| Value | Description |
|-------|-------------|
| `read-only` | Can read and search, cannot modify files |
| `read-write` | Full access to all tools (default) |

### Model Tier Field

Hints at which model tier to use:

| Value | Use Case |
|-------|----------|
| `fast` | Simple tasks, high throughput, lower cost |
| `balanced` | General purpose, good capability/cost ratio (default) |
| `capable` | Complex reasoning, analysis, high-stakes tasks |

### Skills Field

List of skill names to preload into the agent's context. Each skill must exist in the skill registry at `.aiassisted/skills/`.

```yaml
skills:
  - review-codes
  - policy-rust
```

**Note:** Skills are only supported by Claude Code. When compiling to OpenCode, this field is ignored.

## System Prompt

The Markdown body after the frontmatter becomes the agent's system prompt:

```yaml
---
name: code-reviewer
description: Reviews code for quality and best practices.
capabilities: read-only
model-tier: balanced
---

You are a senior code reviewer ensuring high standards of code quality.

## When Invoked

1. Identify the files to review
2. Analyze for quality, security, and maintainability
3. Provide actionable feedback organized by priority

## Review Checklist

- Code is clear and readable
- Functions and variables are well-named
- Proper error handling
- No exposed secrets or API keys
```

## Platform Compilation

Agents are compiled to platform-specific formats during `aiassisted agents setup`.

### Claude Code

**Target:** `.claude/agents/{name}.md`

| Source Field | Target Field |
|--------------|--------------|
| `name` | `name` |
| `description` | `description` |
| `capabilities: read-only` | `disallowedTools: Write, Edit` |
| `capabilities: read-write` | (no restriction) |
| `model-tier: fast` | `model: haiku` |
| `model-tier: balanced` | `model: sonnet` |
| `model-tier: capable` | `model: opus` |
| `skills` | `skills` (passed through) |

**Example output:**
```yaml
---
name: code-reviewer
description: Reviews code for quality and best practices.
disallowedTools: Write, Edit
model: sonnet
skills:
  - review-codes
---

[System prompt body]
```

### OpenCode

**Target:** `.opencode/agents/{name}.md`

| Source Field | Target Field |
|--------------|--------------|
| `name` | (filename becomes name) |
| `description` | `description` |
| `capabilities: read-only` | `tools: { write: false, edit: false }` |
| `capabilities: read-write` | (default tools) |
| `model-tier: fast` | `model: anthropic/claude-haiku-4-20250514` |
| `model-tier: balanced` | `model: anthropic/claude-sonnet-4-20250514` |
| `model-tier: capable` | `model: anthropic/claude-opus-4-20250514` |
| `skills` | (ignored) |

**Example output:**
```yaml
---
description: Reviews code for quality and best practices.
mode: subagent
model: anthropic/claude-sonnet-4-20250514
tools:
  write: false
  edit: false
---

[System prompt body]
```

## Validation Rules

The following validations are performed:

1. **Required fields**: `name` and `description` must be present
2. **Name format**: Must follow naming rules and match directory name
3. **Description length**: Non-empty, maximum 1024 characters
4. **Capabilities value**: Must be `read-only` or `read-write` if specified
5. **Model tier value**: Must be `fast`, `balanced`, or `capable` if specified
6. **Skills exist**: Each referenced skill must exist in `.aiassisted/skills/`

## Best Practices

### Write Clear Descriptions

The description helps the AI decide when to use this agent. Be specific:

**Good:**
```yaml
description: Reviews code for quality, security, and best practices. Use after writing or modifying code, or when the user asks for code review.
```

**Poor:**
```yaml
description: Reviews code.
```

### Match Capabilities to Purpose

- Use `read-only` for agents that analyze, review, or explore
- Use `read-write` for agents that need to modify files

### Choose Appropriate Model Tier

- `fast`: Quick lookups, simple formatting, high-volume tasks
- `balanced`: Most development tasks, code review, debugging
- `capable`: Complex analysis, architectural decisions, nuanced reasoning

### Keep System Prompts Focused

- Be specific about what the agent should do
- Include step-by-step instructions when helpful
- Avoid redundant information the model already knows

### Reference Skills for Domain Knowledge

Instead of embedding large amounts of context in the system prompt, reference skills:

```yaml
skills:
  - policy-rust
  - review-codes
```

## Example Agents

### Code Reviewer

```yaml
---
name: code-reviewer
description: Reviews code for quality, security, and best practices. Use after writing or modifying code.
capabilities: read-only
model-tier: balanced
skills:
  - review-codes
---

You are a senior code reviewer ensuring high standards of code quality and security.

## When Invoked

1. Run git diff to see recent changes
2. Focus on modified files
3. Begin review immediately

## Review Checklist

- Code is clear and readable
- Functions and variables are well-named
- No duplicated code
- Proper error handling
- No exposed secrets or API keys
- Input validation implemented

## Feedback Format

Organize feedback by priority:
- **Critical**: Must fix before merge
- **Warning**: Should fix
- **Suggestion**: Consider improving

Include specific examples of how to fix issues.
```

### Debugger

```yaml
---
name: debugger
description: Debugging specialist for errors, test failures, and unexpected behavior. Use when encountering issues.
capabilities: read-write
model-tier: balanced
---

You are an expert debugger specializing in root cause analysis.

## When Invoked

1. Capture error message and stack trace
2. Identify reproduction steps
3. Isolate the failure location
4. Implement minimal fix
5. Verify solution works

## Debugging Process

- Analyze error messages and logs
- Check recent code changes
- Form and test hypotheses
- Add strategic debug logging if needed
- Inspect variable states

## Output Format

For each issue, provide:
- Root cause explanation
- Evidence supporting the diagnosis
- Specific code fix
- Testing approach
- Prevention recommendations

Focus on fixing the underlying issue, not the symptoms.
```

### Explorer

```yaml
---
name: explorer
description: Fast codebase exploration for finding files, searching code, and understanding structure.
capabilities: read-only
model-tier: fast
---

You are a codebase exploration specialist optimized for speed.

## When Invoked

1. Understand what the user is looking for
2. Use efficient search strategies
3. Return concise, relevant results

## Search Strategies

- Use glob patterns for file discovery
- Use grep for content search
- Read files to understand context
- Summarize findings clearly

## Output Format

- List relevant files with brief descriptions
- Include file paths for easy navigation
- Highlight the most relevant matches first
```

## CLI Commands

```bash
# List available core agents
aiassisted agents

# Setup agents for Claude Code
aiassisted agents setup --platform claude-code

# Setup agents for OpenCode
aiassisted agents setup --platform opencode

# Update core agents from upstream
aiassisted agents update
```

## References

- [Claude Code Subagents Guide](claude-code/claude-code-subagents-guide.md)
- [OpenCode Agents Guide](opencode/opencode-agents-guide.md)
- [Agent Skills Specification](../agentskills/agent-skills.guideline.md)
