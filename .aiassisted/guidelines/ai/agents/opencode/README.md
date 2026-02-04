# OpenCode Customization Guidelines

This directory contains comprehensive guidelines for customizing OpenCode, a powerful AI-assisted coding tool.

## Available Guidelines

### 1. [OpenCode Agents Guide](opencode-agents-guide.md)
**Purpose**: Comprehensive guide to creating and configuring specialized AI agents in OpenCode.

**Topics Covered**:
- Agent types (primary agents vs subagents)
- Configuration methods (JSON and Markdown)
- All configuration options with detailed examples
- Permission system and tool access control
- Temperature and model selection
- Task permissions for orchestration
- Best practices and troubleshooting
- Real-world agent examples (documentation, security, testing, etc.)

**Use When**: You need to create custom agents for specific workflows, configure agent behavior, or understand the agent system.

### 2. [OpenCode Commands Guide](opencode-commands-guide.md)
**Purpose**: Complete reference for creating custom commands for repetitive tasks.

**Topics Covered**:
- Command configuration (JSON and Markdown)
- Template syntax (arguments, shell output, file references)
- All configuration options
- Extensive real-world examples:
  - Testing commands
  - Code review commands
  - Documentation commands
  - Refactoring commands
  - Git workflow commands
  - Project setup commands
- Advanced patterns for multi-file analysis
- Best practices and troubleshooting

**Use When**: You want to automate repetitive prompts, create project-specific workflows, or build a command library.

### 3. [OpenCode Skills Guide](opencode-skills-guide.md)
**Purpose**: In-depth guide to creating reusable instruction sets that agents can load on-demand.

**Topics Covered**:
- Skill discovery system and file structure
- SKILL.md format with frontmatter specifications
- Name validation rules and requirements
- Permission system for skill access control
- Per-agent skill permissions
- Example skills library:
  - Git workflows
  - Code review
  - Security audits
  - Testing strategies
  - Documentation generation
- Best practices for skill design
- Migration from Claude skills
- Troubleshooting guide

**Use When**: You need reusable, shareable instruction sets that multiple agents can leverage, or when building a knowledge base.

## Quick Reference

### When to Use What

| Use Case | Tool | Example |
|----------|------|---------|
| Need different tool permissions for different tasks | **Agents** | Build agent (full access) vs Plan agent (read-only) |
| Repetitive prompt with arguments | **Commands** | `/test <filename>` runs tests for specific file |
| Reusable systematic process | **Skills** | Security checklist, code review criteria |
| Complex orchestration | **Agents** | Primary agent that delegates to specialized subagents |
| Project-specific workflows | **Commands** | `/deploy-staging` with project-specific steps |
| Domain expertise | **Skills** | Rust safety review, OWASP security checks |

### File Locations

#### Global Configuration
```
~/.config/opencode/
├── opencode.json          # Main config
├── agents/                # Global agents
│   └── <name>.md
├── commands/              # Global commands
│   └── <name>.md
└── skills/                # Global skills
    └── <name>/
        └── SKILL.md
```

#### Project Configuration
```
.opencode/
├── opencode.json          # Project config
├── agents/                # Project agents
│   └── <name>.md
├── commands/              # Project commands
│   └── <name>.md
└── skills/                # Project skills
    └── <name>/
        └── SKILL.md
```

## Getting Started

### 1. Creating Your First Agent

**Simplest approach - use the CLI**:
```bash
opencode agent create
```

**Manual approach - create a markdown file**:

`.opencode/agents/reviewer.md`:
```markdown
---
description: Reviews code for quality and best practices
mode: subagent
tools:
  write: false
  edit: false
---

You are a code reviewer. Focus on:
- Code quality and best practices
- Potential bugs and edge cases
- Performance implications
- Security considerations
```

**Usage**: Switch with Tab or invoke with `@reviewer`

### 2. Creating Your First Command

`.opencode/commands/test.md`:
```markdown
---
description: Run tests with coverage
agent: build
---

Run the full test suite with coverage:
!`npm test -- --coverage`

Analyze the results and suggest improvements.
```

**Usage**: Type `/test` in the TUI

### 3. Creating Your First Skill

`.opencode/skills/git-release/SKILL.md`:
```markdown
---
name: git-release
description: Create consistent releases and changelogs
---

## What I Do
- Draft release notes from merged PRs
- Propose version bumps
- Generate changelog

## When to Use Me
Use when preparing a tagged release.
```

**Usage**: Agent automatically sees it and can load with `skill({ name: "git-release" })`

## Integration Examples

### Complete Workflow Example: Testing Pipeline

**1. Agent** (`.opencode/agents/test-expert.md`):
```markdown
---
description: Testing specialist with full test tool access
mode: subagent
tools:
  bash:
    "*": "ask"
    "npm test*": "allow"
    "pytest*": "allow"
permission:
  skill:
    "test-*": "allow"
---

You are a testing expert. Use available test-* skills.
```

**2. Command** (`.opencode/commands/test-coverage.md`):
```markdown
---
description: Analyze test coverage
agent: test-expert
subtask: true
---

Current test coverage:
!`npm test -- --coverage --json`

Load the test-strategy skill and analyze coverage.
Suggest improvements based on the strategy.
```

**3. Skill** (`.opencode/skills/test-strategy/SKILL.md`):
```markdown
---
name: test-strategy
description: Comprehensive testing strategy guidelines
---

## Coverage Goals
- Critical paths: 100%
- Business logic: 90%+
- Utilities: 80%+

## Test Types
1. Unit (70%)
2. Integration (20%)
3. E2E (10%)
```

**Usage**: Type `/test-coverage` → Invokes `test-expert` agent → Agent loads `test-strategy` skill → Analyzes with strategy

## Best Practices Summary

### Agents
- ✅ Use descriptive names and descriptions
- ✅ Start with minimal permissions, add as needed
- ✅ Match temperature to task type
- ✅ Use subagents for specialized tasks
- ❌ Don't give all agents full tool access
- ❌ Don't create too many primary agents

### Commands
- ✅ Single purpose per command
- ✅ Use shell output for dynamic context
- ✅ Document expected arguments
- ✅ Keep shell commands read-only when possible
- ❌ Don't include destructive operations without safeguards
- ❌ Don't create overly complex multi-purpose commands

### Skills
- ✅ Focus on single domain or task type
- ✅ Use checklists and structured formats
- ✅ Provide clear "when to use" guidance
- ✅ Follow strict naming conventions
- ❌ Don't mix multiple unrelated domains
- ❌ Don't forget to match directory name to skill name

## Troubleshooting Quick Reference

| Issue | Check | Solution |
|-------|-------|----------|
| Agent not appearing | File location, disable flag | Move to correct dir, set `disable: false` |
| Command not found | File name, frontmatter | Ensure name matches, valid YAML |
| Skill not loading | SKILL.md spelling, name validation | Use all caps, follow naming rules |
| Permission denied | Permission config, wildcards | Check global + agent-specific permissions |
| Wrong model used | Model config hierarchy | Check agent > global config |

## Resources

- **Official Docs**: 
  - [Agents](https://opencode.ai/docs/agents/)
  - [Commands](https://opencode.ai/docs/commands/)
  - [Skills](https://opencode.ai/docs/skills/)
- **Community**: [OpenCode Discord](https://opencode.ai/discord)
- **Source Code**: [GitHub](https://github.com/anomalyco/opencode)
- **Issues**: [Report bugs](https://github.com/anomalyco/opencode/issues)

## Contributing

These guidelines are maintained as part of the `aiassisted` project. To suggest improvements:

1. Open an issue describing the improvement
2. Submit a PR with updated guidelines
3. Follow the project's contribution guidelines

## License

These guidelines are provided under the same license as the `aiassisted` project (MIT License).

---

**Last Updated**: 2026-01-25  
**OpenCode Version**: 1.0+  
**Maintainer**: aiassisted project
