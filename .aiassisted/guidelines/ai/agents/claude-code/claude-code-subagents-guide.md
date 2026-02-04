# Claude Code Subagents Customization Guide

## Overview

Subagents in Claude Code are specialized AI assistants that handle specific types of tasks. Each subagent runs in its own context window with a custom system prompt, specific tool access, and independent permissions. This guide covers creating, configuring, and using custom subagents effectively.

## Why Use Subagents

Subagents help you:

- **Preserve context**: Keep exploration and implementation out of your main conversation
- **Enforce constraints**: Limit which tools a subagent can use
- **Reuse configurations**: Share user-level subagents across all projects
- **Specialize behavior**: Use focused system prompts for specific domains
- **Control costs**: Route tasks to faster, cheaper models like Haiku

Claude uses each subagent's description to decide when to delegate tasks automatically.

## Built-in Subagents

Claude Code includes several built-in subagents that activate automatically:

### Explore
**Model**: Haiku (fast, low-latency)  
**Tools**: Read-only tools (denied Write and Edit)  
**Purpose**: File discovery, code search, codebase exploration

Claude delegates to Explore for searching or understanding a codebase without making changes. This keeps exploration results out of your main conversation context.

**Thoroughness Levels**: When invoking, Claude specifies:
- **quick**: Targeted lookups
- **medium**: Balanced exploration
- **very thorough**: Comprehensive analysis

### Plan
**Model**: Inherits from main conversation  
**Tools**: Read-only tools (denied Write and Edit)  
**Purpose**: Codebase research for planning

Used during plan mode to gather context before presenting a plan. Prevents infinite nesting (subagents cannot spawn other subagents).

### General-purpose
**Model**: Inherits from main conversation  
**Tools**: All tools  
**Purpose**: Complex research, multi-step operations, code modifications

Claude delegates to general-purpose for tasks requiring both exploration and modification, complex reasoning, or multiple dependent steps.

### Other Built-in Subagents

| Agent | Model | When Claude uses it |
|-------|-------|---------------------|
| Bash | Inherits | Running terminal commands in separate context |
| statusline-setup | Sonnet | When you run `/statusline` to configure status line |
| Claude Code Guide | Haiku | When you ask questions about Claude Code features |

## Quick Start: Create Your First Subagent

### Using the `/agents` Command (Recommended)

The interactive approach for creating subagents:

**Step 1**: Open the subagents interface
```
/agents
```

**Step 2**: Create a new user-level agent
- Select **Create new agent**
- Choose **User-level** (saves to `~/.claude/agents/` for all projects)

**Step 3**: Generate with Claude
- Select **Generate with Claude**
- Describe the subagent:
```
A code improvement agent that scans files and suggests improvements
for readability, performance, and best practices. It should explain
each issue, show the current code, and provide an improved version.
```

**Step 4**: Select tools
- For read-only reviewer, select only **Read-only tools**
- All tools selected = inherits all from main conversation

**Step 5**: Select model
- Choose **Sonnet** (balances capability and speed)

**Step 6**: Choose a color
- Pick background color for UI identification

**Step 7**: Save and try it out
```
Use the code-improver agent to suggest improvements in this project
```

### Manual Creation

Subagents are Markdown files with YAML frontmatter. No restart needed after creation.

## Configuration

### Subagent Scope and Priority

Store subagents in different locations depending on scope. Higher-priority locations win when names conflict.

| Location | Scope | Priority | How to create |
|----------|-------|----------|---------------|
| `--agents` CLI flag | Current session | 1 (highest) | Pass JSON when launching Claude Code |
| `.claude/agents/` | Current project | 2 | Interactive or manual |
| `~/.claude/agents/` | All your projects | 3 | Interactive or manual |
| Plugin's `agents/` directory | Where plugin enabled | 4 (lowest) | Installed with plugins |

**Project subagents** (`.claude/agents/`): Check into version control for team collaboration

**User subagents** (`~/.claude/agents/`): Personal subagents available everywhere

**CLI-defined subagents**: Session-only, not saved to disk (useful for testing/automation):

```bash
claude --agents '{
  "code-reviewer": {
    "description": "Expert code reviewer. Use proactively after code changes.",
    "prompt": "You are a senior code reviewer. Focus on code quality, security, and best practices.",
    "tools": ["Read", "Grep", "Glob", "Bash"],
    "model": "sonnet"
  }
}'
```

**Plugin subagents**: Come from installed plugins, appear in `/agents` alongside custom ones

### File Structure

Subagent files use YAML frontmatter for configuration, followed by system prompt in Markdown:

```markdown
---
name: code-reviewer
description: Reviews code for quality and best practices
tools: Read, Glob, Grep
model: sonnet
---

You are a code reviewer. When invoked, analyze the code and provide
specific, actionable feedback on quality, security, and best practices.
```

**Important**: Subagents are loaded at session start. If you manually add a file, restart your session or use `/agents` to load immediately.

### Frontmatter Fields

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Unique identifier using lowercase letters and hyphens |
| `description` | Yes | When Claude should delegate to this subagent |
| `tools` | No | Tools the subagent can use. Inherits all if omitted |
| `disallowedTools` | No | Tools to deny, removed from inherited or specified list |
| `model` | No | Model to use: `sonnet`, `opus`, `haiku`, or `inherit` (default: `inherit`) |
| `permissionMode` | No | Permission mode: `default`, `acceptEdits`, `dontAsk`, `bypassPermissions`, or `plan` |
| `skills` | No | Skills to load into subagent context at startup (full content injected) |
| `hooks` | No | Lifecycle hooks scoped to this subagent |

### Model Selection

The `model` field controls which AI model the subagent uses:

- **Model alias**: `sonnet`, `opus`, or `haiku`
- **inherit**: Use same model as main conversation (default)
- **Omitted**: Defaults to `inherit`

Example:
```yaml
---
name: quick-explorer
description: Fast code exploration
model: haiku  # Use fast, cheap model
---
```

### Tool Access Control

#### Available Tools
Subagents can use any of Claude Code's internal tools. By default, subagents inherit all tools from the main conversation, including MCP tools.

**Restrict with allowlist**:
```yaml
---
name: safe-researcher
description: Research agent with restricted capabilities
tools: Read, Grep, Glob, Bash
---
```

**Restrict with denylist**:
```yaml
---
name: safe-researcher
description: Research agent with restricted capabilities
disallowedTools: Write, Edit
---
```

**Both allowlist and denylist**:
```yaml
---
name: safe-researcher
description: Research agent with restricted capabilities
tools: Read, Grep, Glob, Bash
disallowedTools: Write, Edit  # Removed from tools list
---
```

### Permission Modes

The `permissionMode` field controls how subagent handles permission prompts. Subagents inherit permission context from main conversation but can override the mode.

| Mode | Behavior |
|------|----------|
| `default` | Standard permission checking with prompts |
| `acceptEdits` | Auto-accept file edits |
| `dontAsk` | Auto-deny permission prompts (explicitly allowed tools still work) |
| `bypassPermissions` | Skip all permission checks ⚠️ |
| `plan` | Plan mode (read-only exploration) |

⚠️ **Warning**: Use `bypassPermissions` with caution. It skips all permission checks.

If parent uses `bypassPermissions`, it takes precedence and cannot be overridden.

### Preload Skills into Subagents

Use the `skills` field to inject skill content into subagent context at startup. This gives domain knowledge without requiring skill discovery during execution.

```yaml
---
name: api-developer
description: Implement API endpoints following team conventions
skills:
  - api-conventions
  - error-handling-patterns
---

Implement API endpoints. Follow the conventions and patterns from the preloaded skills.
```

**Key Points**:
- Full content of each skill is injected (not just made available)
- Subagents don't inherit skills from parent conversation
- Must list skills explicitly
- This is the inverse of running a skill in a subagent (skill's `context: fork`)

### Conditional Rules with Hooks

Use `PreToolUse` hooks to validate operations before execution. Useful when you need to allow some operations while blocking others.

Example: Subagent that only allows read-only database queries:

```yaml
---
name: db-reader
description: Execute read-only database queries
tools: Bash
hooks:
  PreToolUse:
    - matcher: "Bash"
      hooks:
        - type: command
          command: "./scripts/validate-readonly-query.sh"
---
```

Validation script (`./scripts/validate-readonly-query.sh`):

```bash
#!/bin/bash
# Reads JSON input from stdin, blocks SQL write operations

INPUT=$(cat)
COMMAND=$(echo "$INPUT" | jq -r '.tool_input.command // empty')

# Block SQL write operations (case-insensitive)
if echo "$COMMAND" | grep -iE '\b(INSERT|UPDATE|DELETE|DROP|CREATE|ALTER|TRUNCATE)\b' > /dev/null; then
  echo "Blocked: Only SELECT queries are allowed" >&2
  exit 2  # Exit code 2 blocks the operation
fi

exit 0
```

Claude Code passes hook input as JSON via stdin. Exit code 2 blocks the operation.

### Hooks in Subagents

Two ways to configure hooks:

#### 1. In Subagent's Frontmatter
Hooks that run only while that subagent is active:

| Event | Matcher input | When it fires |
|-------|---------------|---------------|
| `PreToolUse` | Tool name | Before subagent uses a tool |
| `PostToolUse` | Tool name | After subagent uses a tool |
| `Stop` | (none) | When subagent finishes |

Example:
```yaml
---
name: code-reviewer
description: Review code changes with automatic linting
hooks:
  PreToolUse:
    - matcher: "Bash"
      hooks:
        - type: command
          command: "./scripts/validate-command.sh $TOOL_INPUT"
  PostToolUse:
    - matcher: "Edit|Write"
      hooks:
        - type: command
          command: "./scripts/run-linter.sh"
---
```

`Stop` hooks in frontmatter are automatically converted to `SubagentStop` events.

#### 2. Project-level Hooks (in settings.json)
Hooks that respond to subagent lifecycle events in main session:

| Event | Matcher input | When it fires |
|-------|---------------|---------------|
| `SubagentStart` | Agent type name | When subagent begins execution |
| `SubagentStop` | Agent type name | When subagent completes |

Example (`settings.json`):
```json
{
  "hooks": {
    "SubagentStart": [
      {
        "matcher": "db-agent",
        "hooks": [
          { "type": "command", "command": "./scripts/setup-db-connection.sh" }
        ]
      }
    ],
    "SubagentStop": [
      {
        "matcher": "db-agent",
        "hooks": [
          { "type": "command", "command": "./scripts/cleanup-db-connection.sh" }
        ]
      }
    ]
  }
}
```

### Disable Specific Subagents

Prevent Claude from using specific subagents by adding to `deny` array in settings:

```json
{
  "permissions": {
    "deny": ["Task(Explore)", "Task(my-custom-agent)"]
  }
}
```

Use format `Task(subagent-name)`. Works for both built-in and custom subagents.

CLI alternative:
```bash
claude --disallowedTools "Task(Explore)"
```

## Working with Subagents

### Automatic Delegation

Claude automatically delegates based on:
- Task description in your request
- `description` field in subagent configurations
- Current context

**Encourage proactive delegation**: Include "use proactively" in description field.

**Request specific subagent explicitly**:
```
Use the test-runner subagent to fix failing tests
Have the code-reviewer subagent look at my recent changes
```

### Foreground vs Background Subagents

**Foreground subagents** (blocking):
- Block main conversation until complete
- Permission prompts passed through to you
- Can use `AskUserQuestion` for clarifying questions

**Background subagents** (concurrent):
- Run while you continue working
- Inherit parent's permissions and auto-deny anything not pre-approved
- If needs permission or clarification → tool call fails but subagent continues
- MCP tools not available
- Can resume in foreground if fails due to missing permissions

**Control**:
- Ask Claude to "run this in the background"
- Press **Ctrl+B** to background a running task
- Disable all background tasks: Set `CLAUDE_CODE_DISABLE_BACKGROUND_TASKS=1`

### Common Patterns

#### Isolate High-Volume Operations

Most effective use: Isolate operations that produce large output.

```
Use a subagent to run the test suite and report only the failing tests with their error messages
```

Running tests, fetching docs, or processing logs can consume significant context. Verbose output stays in subagent's context; only relevant summary returns to main conversation.

#### Run Parallel Research

For independent investigations, spawn multiple subagents simultaneously:

```
Research the authentication, database, and API modules in parallel using separate subagents
```

Each subagent explores independently, then Claude synthesizes findings.

⚠️ **Warning**: When subagents complete, results return to main conversation. Many subagents with detailed results can consume significant context.

#### Chain Subagents

Multi-step workflows using subagents in sequence:

```
Use the code-reviewer subagent to find performance issues, then use the optimizer subagent to fix them
```

Each subagent completes and returns results to Claude, which passes relevant context to next subagent.

### When to Use Subagents vs Main Conversation

**Use main conversation when**:
- Task needs frequent back-and-forth or iterative refinement
- Multiple phases share significant context (planning → implementation → testing)
- Making quick, targeted change
- Latency matters (subagents start fresh and may need time to gather context)

**Use subagents when**:
- Task produces verbose output you don't need in main context
- Want to enforce specific tool restrictions or permissions
- Work is self-contained and can return summary

**Consider Skills instead** when you want reusable prompts or workflows that run in main conversation context rather than isolated subagent context.

⚠️ **Note**: Subagents cannot spawn other subagents. For nested delegation, use Skills or chain subagents from main conversation.

### Manage Subagent Context

#### Resume Subagents

Each subagent invocation creates new instance with fresh context. To continue existing work instead of starting over, ask Claude to resume.

Resumed subagents retain:
- Full conversation history
- All previous tool calls and results
- All reasoning

Example:
```
Use the code-reviewer subagent to review the authentication module
[Agent completes]

Continue that code review and now analyze the authorization logic
[Claude resumes the subagent with full context]
```

Claude receives agent ID when subagent completes. Can ask Claude for ID explicitly, or find in transcript files at `~/.claude/projects/{project}/{sessionId}/subagents/`.

**Transcript persistence**:
- **Main conversation compaction**: Subagent transcripts unaffected (stored separately)
- **Session persistence**: Can resume after restarting Claude Code by resuming same session
- **Automatic cleanup**: Based on `cleanupPeriodDays` setting (default: 30 days)

#### Auto-compaction

Subagents support automatic compaction using same logic as main conversation.

**Default**: Auto-compaction triggers at ~95% capacity

**Override**: Set `CLAUDE_AUTOCOMPACT_PCT_OVERRIDE` to lower percentage (e.g., `50`)

Compaction events logged in subagent transcript files:
```json
{
  "type": "system",
  "subtype": "compact_boundary",
  "compactMetadata": {
    "trigger": "auto",
    "preTokens": 167189
  }
}
```

## Example Subagents

### Code Reviewer

Read-only subagent for code review without modification.

```markdown
---
name: code-reviewer
description: Expert code review specialist. Proactively reviews code for quality, security, and maintainability. Use immediately after writing or modifying code.
tools: Read, Grep, Glob, Bash
model: inherit
---

You are a senior code reviewer ensuring high standards of code quality and security.

When invoked:
1. Run git diff to see recent changes
2. Focus on modified files
3. Begin review immediately

Review checklist:
- Code is clear and readable
- Functions and variables are well-named
- No duplicated code
- Proper error handling
- No exposed secrets or API keys
- Input validation implemented
- Good test coverage
- Performance considerations addressed

Provide feedback organized by priority:
- Critical issues (must fix)
- Warnings (should fix)
- Suggestions (consider improving)

Include specific examples of how to fix issues.
```

### Debugger

Subagent that can analyze and fix issues.

```markdown
---
name: debugger
description: Debugging specialist for errors, test failures, and unexpected behavior. Use proactively when encountering any issues.
tools: Read, Edit, Bash, Grep, Glob
---

You are an expert debugger specializing in root cause analysis.

When invoked:
1. Capture error message and stack trace
2. Identify reproduction steps
3. Isolate the failure location
4. Implement minimal fix
5. Verify solution works

Debugging process:
- Analyze error messages and logs
- Check recent code changes
- Form and test hypotheses
- Add strategic debug logging
- Inspect variable states

For each issue, provide:
- Root cause explanation
- Evidence supporting the diagnosis
- Specific code fix
- Testing approach
- Prevention recommendations

Focus on fixing the underlying issue, not the symptoms.
```

### Data Scientist

Domain-specific subagent for data analysis.

```markdown
---
name: data-scientist
description: Data analysis expert for SQL queries, BigQuery operations, and data insights. Use proactively for data analysis tasks and queries.
tools: Bash, Read, Write
model: sonnet
---

You are a data scientist specializing in SQL and BigQuery analysis.

When invoked:
1. Understand the data analysis requirement
2. Write efficient SQL queries
3. Use BigQuery command line tools (bq) when appropriate
4. Analyze and summarize results
5. Present findings clearly

Key practices:
- Write optimized SQL queries with proper filters
- Use appropriate aggregations and joins
- Include comments explaining complex logic
- Format results for readability
- Provide data-driven recommendations

For each analysis:
- Explain the query approach
- Document any assumptions
- Highlight key findings
- Suggest next steps based on data

Always ensure queries are efficient and cost-effective.
```

### Database Query Validator

Allows Bash but validates to permit only read-only SQL queries.

```markdown
---
name: db-reader
description: Execute read-only database queries. Use when analyzing data or generating reports.
tools: Bash
hooks:
  PreToolUse:
    - matcher: "Bash"
      hooks:
        - type: command
          command: "./scripts/validate-readonly-query.sh"
---

You are a database analyst with read-only access. Execute SELECT queries to answer questions about the data.

When asked to analyze data:
1. Identify which tables contain the relevant data
2. Write efficient SELECT queries with appropriate filters
3. Present results clearly with context

You cannot modify data. If asked to INSERT, UPDATE, DELETE, or modify schema, explain that you only have read access.
```

Validation script (`./scripts/validate-readonly-query.sh`):

```bash
#!/bin/bash
# Blocks SQL write operations, allows SELECT queries

INPUT=$(cat)
COMMAND=$(echo "$INPUT" | jq -r '.tool_input.command // empty')

if [ -z "$COMMAND" ]; then
  exit 0
fi

# Block write operations (case-insensitive)
if echo "$COMMAND" | grep -iE '\b(INSERT|UPDATE|DELETE|DROP|CREATE|ALTER|TRUNCATE|REPLACE|MERGE)\b' > /dev/null; then
  echo "Blocked: Write operations not allowed. Use SELECT queries only." >&2
  exit 2
fi

exit 0
```

Make executable:
```bash
chmod +x ./scripts/validate-readonly-query.sh
```

## Best Practices

### Design Focused Subagents
Each subagent should excel at one specific task

### Write Detailed Descriptions
Claude uses description to decide when to delegate. Be specific.

### Limit Tool Access
Grant only necessary permissions for security and focus

### Check into Version Control
Share project subagents with your team

### Use Appropriate Models
- **Haiku**: Fast, cheap, simple tasks
- **Sonnet**: Balanced capability and speed
- **Opus**: Complex reasoning and analysis
- **Inherit**: Same as main conversation

### Organize by Scope
- **User-level** (`~/.claude/agents/`): Personal, reusable across projects
- **Project-level** (`.claude/agents/`): Team-shared, version-controlled
- **CLI**: Testing, automation, session-specific

## Troubleshooting

### Subagent Not Loading
- Check file is in correct location
- Verify frontmatter syntax is valid YAML
- Restart session or use `/agents` to reload
- Check for name conflicts (higher priority wins)

### Subagent Not Triggering
- Make description more specific and keyword-rich
- Include "use proactively" in description
- Try explicit invocation to test
- Check if subagent is disabled in permissions

### Permission Issues
- Verify `permissionMode` setting
- Check parent's permission mode (cannot override `bypassPermissions`)
- Review tool access (`tools` and `disallowedTools`)

### Hook Not Executing
- Verify script path is correct
- Ensure script is executable (`chmod +x`)
- Check hook matcher pattern
- Review exit codes (0 = allow, 2 = block)

### Background Subagent Failing
- Check if it needs permissions not pre-approved
- Try resuming in foreground
- Verify MCP tools aren't required (not available in background)

## Additional Resources

- [Claude Code Subagents Documentation](https://code.claude.com/docs/en/sub-agents)
- [Claude Code Skills Documentation](https://code.claude.com/docs/en/skills)
- [Claude Code Hooks Documentation](https://code.claude.com/docs/en/hooks)
- [Claude Code Plugins Documentation](https://code.claude.com/docs/en/plugins)
- [Claude Code IAM Documentation](https://code.claude.com/docs/en/iam)
