# Claude Code Customization Guidelines

This directory contains comprehensive guidelines for customizing Claude Code, an AI-assisted coding tool that supports subagents and skills.

## Available Guidelines

### 1. [Claude Code Subagents Guide](claude-code-subagents-guide.md)
**Purpose**: Complete guide to creating and configuring specialized AI subagents in Claude Code.

**Topics Covered**:
- Built-in subagents (Explore, Plan, General-purpose)
- Creating subagents with `/agents` command or manually
- Configuration methods (YAML frontmatter and Markdown)
- Subagent scope and priority (CLI, project, user, plugin)
- All configuration options:
  - Model selection (Sonnet, Opus, Haiku, inherit)
  - Tool access control (allowlist and denylist)
  - Permission modes (default, acceptEdits, dontAsk, bypassPermissions, plan)
  - Preloading skills into subagents
  - Conditional rules with hooks
- Working with subagents:
  - Automatic delegation and explicit invocation
  - Foreground vs background execution
  - Common patterns (isolate high-volume ops, parallel research, chaining)
  - When to use subagents vs main conversation
  - Managing subagent context (resume, auto-compaction)
- Real-world examples:
  - Code reviewer (read-only)
  - Debugger (can modify)
  - Data scientist (domain-specific)
  - Database query validator (with hooks)
- Best practices and troubleshooting

**Use When**: You need specialized agents for specific workflows, want to isolate tasks, or need to enforce tool/permission constraints.

### 2. [Claude Code Skills Guide](claude-code-skills-guide.md)
**Purpose**: Comprehensive reference for creating reusable instruction sets that extend Claude's capabilities.

**Topics Covered**:
- Skills vs Commands (merged functionality)
- Creating your first skill (step-by-step)
- Skill locations and discovery (enterprise, personal, project, plugin)
- Automatic discovery from nested directories (monorepo support)
- Skill directory structure and supporting files
- Configuration with frontmatter:
  - Types of skill content (reference vs task)
  - All frontmatter fields
  - String substitutions ($ARGUMENTS, ${CLAUDE_SESSION_ID})
  - Control who invokes (disable-model-invocation, user-invocable)
  - Tool access restrictions
  - Passing arguments
- Advanced patterns:
  - Inject dynamic context with `` !`command` ``
  - Run skills in subagent (context: fork)
  - Restrict Claude's skill access
- Sharing skills (project, plugins, managed)
- Example skills:
  - Code review
  - Commit message generator
  - API documentation generator
  - Test coverage analyzer
  - Visual codebase explorer
  - Security audit
- Troubleshooting and best practices

**Use When**: You want to create reusable prompts, automate repetitive workflows, or build a library of domain-specific knowledge.

## Quick Reference

### When to Use What

| Use Case | Tool | Example |
|----------|------|---------|
| Isolated task with own context | **Subagent** | Explore agent for codebase search without polluting main context |
| Reusable instructions for Claude | **Skill (reference)** | API conventions that apply to all endpoint work |
| Workflow you trigger manually | **Skill (task)** | `/deploy` command with specific deployment steps |
| Enforce tool restrictions | **Subagent** | Read-only agent that can't modify files |
| Domain-specific specialist | **Subagent** | Data scientist agent with SQL expertise |
| Dynamic context injection | **Skill** | Pull request summary that fetches live PR data |
| Run in isolated context | **Skill with context: fork** | Research skill that runs in Explore agent |

### File Locations

#### Personal Configuration
```
~/.claude/
├── agents/                # User-level subagents
│   └── <name>.md          # Available in all projects
└── skills/                # User-level skills
    └── <name>/
        └── SKILL.md       # Available in all projects
```

#### Project Configuration
```
.claude/
├── agents/                # Project-level subagents
│   └── <name>.md          # This project only
├── skills/                # Project-level skills
│   └── <name>/
│       └── SKILL.md       # This project only
└── commands/              # Legacy (still works)
    └── <name>.md          # Creates /name command
```

#### Nested Discovery (Monorepos)
```
packages/
├── frontend/
│   └── .claude/
│       └── skills/        # Auto-discovered when working in packages/frontend/
│           └── react-conventions/
│               └── SKILL.md
└── backend/
    └── .claude/
        └── skills/        # Auto-discovered when working in packages/backend/
            └── api-patterns/
                └── SKILL.md
```

## Getting Started

### 1. Creating Your First Subagent

**Interactive approach (recommended)**:
```
/agents
```
Then select **Create new agent** → **User-level** → **Generate with Claude**

**Manual approach**:

`.claude/agents/code-reviewer.md`:
```markdown
---
name: code-reviewer
description: Reviews code for quality and best practices. Use proactively after writing code.
tools: Read, Grep, Glob, Bash
model: sonnet
---

You are a code reviewer. When invoked, analyze the code and provide
specific, actionable feedback on quality, security, and best practices.
```

**Usage**: Claude delegates automatically, or invoke explicitly:
```
Use the code-reviewer subagent to review my recent changes
```

### 2. Creating Your First Skill

**Create skill directory**:
```bash
mkdir -p ~/.claude/skills/explain-code
```

**Create SKILL.md**:

`~/.claude/skills/explain-code/SKILL.md`:
```yaml
---
name: explain-code
description: Explains code with visual diagrams and analogies. Use when explaining how code works.
---

When explaining code, always include:

1. **Start with an analogy**: Compare to something from everyday life
2. **Draw a diagram**: Use ASCII art to show flow or structure
3. **Walk through the code**: Explain step-by-step
4. **Highlight a gotcha**: What's a common mistake?
```

**Usage**: Claude uses automatically when you ask "how does this work?", or invoke directly:
```
/explain-code src/auth/login.ts
```

## Integration Examples

### Complete Workflow: Code Review Pipeline

**1. Subagent** (`.claude/agents/reviewer.md`):
```markdown
---
name: reviewer
description: Expert code reviewer with read-only access
tools: Read, Grep, Glob, Bash
model: sonnet
skills:
  - code-quality-standards
  - security-checklist
---

You are a senior code reviewer. Use the preloaded skills to ensure
code meets our quality and security standards.
```

**2. Skill** (`.claude/skills/code-quality-standards/SKILL.md`):
```yaml
---
name: code-quality-standards
description: Code quality standards for this project
---

## Code Quality Standards

- Functions under 50 lines
- No code duplication
- Clear naming (no abbreviations)
- Proper error handling
- Test coverage > 80%
```

**3. Skill** (`.claude/skills/security-checklist/SKILL.md`):
```yaml
---
name: security-checklist
description: Security checklist based on OWASP Top 10
---

## Security Checklist

- Input validation
- No hardcoded secrets
- SQL injection prevention
- XSS prevention
- Proper authentication
```

**Usage**: Claude delegates to `reviewer` subagent → Subagent has both skills preloaded → Reviews code against standards

### Skill with Dynamic Context

**Skill** (`.claude/skills/pr-summary/SKILL.md`):
```yaml
---
name: pr-summary
description: Summarize pull request changes
context: fork
agent: Explore
allowed-tools: Bash(gh:*)
---

## Pull request context
- PR diff: !`gh pr diff`
- PR comments: !`gh pr view --comments`
- Changed files: !`gh pr diff --name-only`

Summarize:
1. What changed
2. Why (from PR description and comments)
3. Potential issues
4. Suggested improvements
```

**Usage**: `/pr-summary` → Runs in isolated Explore agent → Fetches live PR data → Returns summary to main conversation

## Key Differences: Subagents vs Skills

| Aspect | Subagent | Skill |
|--------|----------|-------|
| **Context** | Runs in own context window | Runs in current context (or fork) |
| **Invocation** | Claude delegates based on description | Claude loads or you invoke with `/name` |
| **System Prompt** | Markdown body becomes system prompt | Uses agent's system prompt (or fork's) |
| **Tools** | Configured via `tools` field | Configured via `allowed-tools` field |
| **Purpose** | Specialized agent for task type | Reusable instructions/workflow |
| **Best For** | Isolating high-volume ops, enforcing constraints | Reference knowledge, manual workflows |

## Subagents + Skills Together

### Approach 1: Skill with `context: fork`
You write the task in the skill, pick an agent type to execute it.

```yaml
---
name: deep-research
context: fork
agent: Explore
---

Research $ARGUMENTS thoroughly...
```

### Approach 2: Subagent with `skills` field
You define custom subagent that uses skills as reference material.

```yaml
---
name: api-developer
skills:
  - api-conventions
  - error-handling
---

You implement API endpoints following preloaded patterns...
```

| Approach | System Prompt | Task | Also Loads |
|----------|---------------|------|------------|
| Skill with `context: fork` | From agent type | SKILL.md content | CLAUDE.md |
| Subagent with `skills` field | Subagent's markdown body | Claude's delegation | Preloaded skills + CLAUDE.md |

## Best Practices Summary

### Subagents
- ✅ Design focused subagents (one task type each)
- ✅ Write detailed descriptions for automatic delegation
- ✅ Limit tool access to minimum needed
- ✅ Use appropriate model for task complexity
- ✅ Check into version control (project subagents)
- ✅ Include "use proactively" in description for automatic invocation
- ❌ Don't create too many subagents (context budget)
- ❌ Don't use bypassPermissions without careful consideration

### Skills
- ✅ Single purpose per skill
- ✅ Clear, keyword-rich descriptions
- ✅ Keep SKILL.md under 500 lines
- ✅ Use supporting files for detailed reference
- ✅ Add `disable-model-invocation: true` for manual-only workflows
- ✅ Use `context: fork` for isolated, high-volume operations
- ✅ Check into version control (project skills)
- ❌ Don't mix reference and task content in same skill
- ❌ Don't use `context: fork` for guidelines without actionable task

## Troubleshooting Quick Reference

| Issue | Check | Solution |
|-------|-------|----------|
| Subagent not loading | File location, YAML syntax | Move to correct dir, fix frontmatter |
| Subagent not triggering | Description specificity | Add keywords, include "use proactively" |
| Skill not triggering | Description matches usage | Add natural keywords, test with explicit invocation |
| Skill triggers too often | Description too broad | Make more specific, add `disable-model-invocation: true` |
| Dynamic context not injecting | Command syntax | Verify `` !`command` `` format, test command in terminal |
| Hook not executing | Script path, permissions | Check path, make executable (`chmod +x`) |
| Context fork not working | Task vs guidelines | Ensure skill has actionable task, not just reference |
| Too many skills excluded | Character budget | Increase `SLASH_COMMAND_TOOL_CHAR_BUDGET` env var |

## Advanced Patterns

### Read-only Database Subagent with Validation Hook

Allows Bash access but validates to permit only SELECT queries:

```markdown
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

You are a database analyst with read-only access...
```

Hook script blocks write operations, allows SELECT only.

### Visual Output Generator Skill

Generates interactive HTML files that open in browser:

```yaml
---
name: codebase-visualizer
description: Generate interactive tree visualization of codebase
allowed-tools: Bash(python:*)
---

Run visualization script:
```bash
python ~/.claude/skills/codebase-visualizer/scripts/visualize.py .
```

Creates interactive HTML with:
- Collapsible directory tree
- File size indicators
- Type-based coloring
```

### Parallel Research with Multiple Subagents

```
Research the authentication, database, and API modules in parallel using separate subagents
```

Claude spawns three subagents:
1. One for authentication analysis
2. One for database analysis  
3. One for API analysis

Each explores independently, returns findings, Claude synthesizes.

⚠️ Warning: Many subagents returning detailed results can consume significant context.

## Resources

- **Official Docs**:
  - [Subagents](https://code.claude.com/docs/en/sub-agents)
  - [Skills](https://code.claude.com/docs/en/skills)
  - [Hooks](https://code.claude.com/docs/en/hooks)
  - [Plugins](https://code.claude.com/docs/en/plugins)
  - [IAM/Permissions](https://code.claude.com/docs/en/iam)
- **Standards**: [Agent Skills Open Standard](https://agentskills.io)

## Contributing

These guidelines are maintained as part of the `aiassisted` project. To suggest improvements:

1. Open an issue describing the improvement
2. Submit a PR with updated guidelines
3. Follow the project's contribution guidelines

## License

These guidelines are provided under the same license as the `aiassisted` project (MIT License).

---

**Last Updated**: 2026-01-25  
**Claude Code Version**: Compatible with latest  
**Maintainer**: aiassisted project
