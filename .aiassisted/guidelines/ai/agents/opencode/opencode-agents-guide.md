# OpenCode Agents Customization Guide

## Overview

OpenCode agents are specialized AI assistants that can be configured for specific tasks and workflows. This guide covers how to create, configure, and use custom agents effectively.

## Agent Types

### Primary Agents
- **Purpose**: Main assistants you interact with directly
- **Navigation**: Use **Tab** key or configured `switch_agent` keybind to cycle through
- **Tool Access**: Configured via permissions
- **Built-in Examples**:
  - **Build**: Default agent with all tools enabled for full development work
  - **Plan**: Restricted agent for analysis and planning without making changes

### Subagents
- **Purpose**: Specialized assistants invoked for specific tasks
- **Invocation Methods**:
  - Automatically by primary agents based on descriptions
  - Manually via `@` mention (e.g., `@general help me search for this function`)
- **Built-in Examples**:
  - **General**: General-purpose for complex questions and multi-step tasks (full tool access except todo)
  - **Explore**: Fast, read-only agent for codebase exploration

### Session Navigation
When subagents create child sessions:
- **<Leader>+Right** (or `session_child_cycle`): Cycle forward through sessions
- **<Leader>+Left** (or `session_child_cycle_reverse`): Cycle backward through sessions

## Configuration Methods

### Method 1: JSON Configuration

Place configuration in `opencode.json` config file:

```json
{
  "$schema": "https://opencode.ai/config.json",
  "agent": {
    "build": {
      "mode": "primary",
      "model": "anthropic/claude-sonnet-4-20250514",
      "prompt": "{file:./prompts/build.txt}",
      "tools": {
        "write": true,
        "edit": true,
        "bash": true
      }
    },
    "code-reviewer": {
      "description": "Reviews code for best practices and potential issues",
      "mode": "subagent",
      "model": "anthropic/claude-sonnet-4-20250514",
      "prompt": "You are a code reviewer. Focus on security, performance, and maintainability.",
      "tools": {
        "write": false,
        "edit": false
      }
    }
  }
}
```

### Method 2: Markdown Files

Create markdown files in:
- **Global**: `~/.config/opencode/agents/`
- **Per-project**: `.opencode/agents/`

**File name becomes agent name** (e.g., `review.md` → `review` agent)

Example `~/.config/opencode/agents/review.md`:

```markdown
---
description: Reviews code for quality and best practices
mode: subagent
model: anthropic/claude-sonnet-4-20250514
temperature: 0.1
tools:
  write: false
  edit: false
  bash: false
---

You are in code review mode. Focus on:
- Code quality and best practices
- Potential bugs and edge cases
- Performance implications
- Security considerations

Provide constructive feedback without making direct changes.
```

## Configuration Options

### Required Options

#### `description`
Brief description of agent's purpose and when to use it.

```json
{
  "agent": {
    "review": {
      "description": "Reviews code for best practices and potential issues"
    }
  }
}
```

### Core Options

#### `temperature`
Control response randomness and creativity (0.0-1.0):
- **0.0-0.2**: Very focused and deterministic (ideal for analysis, planning)
- **0.3-0.5**: Balanced with some creativity (general development)
- **0.6-1.0**: More creative and varied (brainstorming, exploration)

```json
{
  "agent": {
    "plan": {
      "temperature": 0.1
    },
    "creative": {
      "temperature": 0.8
    }
  }
}
```

**Default**: Model-specific (typically 0, or 0.55 for Qwen models)

#### `maxSteps`
Maximum agentic iterations before forcing text-only response.

```json
{
  "agent": {
    "quick-thinker": {
      "description": "Fast reasoning with limited iterations",
      "maxSteps": 5
    }
  }
}
```

When limit reached, agent summarizes work and recommends remaining tasks.

#### `prompt`
Path to custom system prompt file (relative to config location).

```json
{
  "agent": {
    "review": {
      "prompt": "{file:./prompts/code-review.txt}"
    }
  }
}
```

#### `model`
Override model for this agent. Uses format `provider/model-id`.

```json
{
  "agent": {
    "plan": {
      "model": "anthropic/claude-haiku-4-20250514"
    }
  }
}
```

**Default Behavior**:
- Primary agents: Use globally configured model
- Subagents: Use model of invoking primary agent

#### `mode`
Determines how agent can be used: `primary`, `subagent`, or `all`.

```json
{
  "agent": {
    "review": {
      "mode": "subagent"
    }
  }
}
```

**Default**: `all` if not specified

#### `disable`
Disable the agent completely.

```json
{
  "agent": {
    "review": {
      "disable": true
    }
  }
}
```

#### `hidden`
Hide subagent from `@` autocomplete menu (only for `mode: subagent`).

```json
{
  "agent": {
    "internal-helper": {
      "mode": "subagent",
      "hidden": true
    }
  }
}
```

Agent can still be invoked programmatically via Task tool.

### Tool Configuration

Control which tools are available to the agent.

```json
{
  "$schema": "https://opencode.ai/config.json",
  "tools": {
    "write": true,
    "bash": true
  },
  "agent": {
    "plan": {
      "tools": {
        "write": false,
        "bash": false
      }
    }
  }
}
```

**Agent-specific config overrides global config.**

#### Wildcard Tool Control

Disable all tools from an MCP server:

```json
{
  "agent": {
    "readonly": {
      "tools": {
        "mymcp_*": false,
        "write": false,
        "edit": false
      }
    }
  }
}
```

### Permission Configuration

Control what actions an agent can take. Available for `edit`, `bash`, and `webfetch` tools.

**Permission Levels**:
- `"ask"`: Prompt for approval before running
- `"allow"`: Allow without approval
- `"deny"`: Disable the tool

#### Global Permissions

```json
{
  "$schema": "https://opencode.ai/config.json",
  "permission": {
    "edit": "deny"
  }
}
```

#### Per-Agent Override

```json
{
  "permission": {
    "edit": "deny"
  },
  "agent": {
    "build": {
      "permission": {
        "edit": "ask"
      }
    }
  }
}
```

#### Markdown Agent Permissions

```markdown
---
description: Code review without edits
mode: subagent
permission:
  edit: deny
  bash:
    "*": ask
    "git diff": allow
    "git log*": allow
    "grep *": allow
  webfetch: deny
---

Only analyze code and suggest changes.
```

#### Bash Command Permissions

Fine-grained control with glob patterns:

```json
{
  "agent": {
    "build": {
      "permission": {
        "bash": {
          "*": "ask",
          "git status *": "allow",
          "git push": "ask",
          "grep *": "allow"
        }
      }
    }
  }
}
```

**Rule Precedence**: Last matching rule wins. Put `*` wildcard first, specific rules after.

### Task Permissions

Control which subagents an agent can invoke via Task tool.

```json
{
  "agent": {
    "orchestrator": {
      "mode": "primary",
      "permission": {
        "task": {
          "*": "deny",
          "orchestrator-*": "allow",
          "code-reviewer": "ask"
        }
      }
    }
  }
}
```

**Behavior**:
- `deny`: Removes subagent from Task tool description entirely
- Rules evaluated in order, **last matching rule wins**
- Users can always `@` invoke any subagent directly

### Additional Provider-Specific Options

Any options not recognized by OpenCode are **passed through to the provider** as model options.

Example with OpenAI reasoning models:

```json
{
  "agent": {
    "deep-thinker": {
      "description": "Agent that uses high reasoning effort for complex problems",
      "model": "openai/gpt-5",
      "reasoningEffort": "high",
      "textVerbosity": "low"
    }
  }
}
```

Check provider documentation for available parameters. Run `opencode models` to see available models.

## Creating Agents Interactively

Use the built-in command:

```bash
opencode agent create
```

This interactive wizard will:
1. Ask where to save (global or project-specific)
2. Request agent description
3. Generate appropriate system prompt and identifier
4. Let you select tool access
5. Create markdown file with configuration

## Common Use Cases

### Development Agents
- **Build agent**: Full development work with all tools enabled
- **Debug agent**: Investigation focused with bash and read tools enabled

### Review & Analysis Agents
- **Plan agent**: Analysis and planning without making changes
- **Review agent**: Code review with read-only access plus documentation tools
- **Security auditor**: Identify vulnerabilities without modification

### Documentation Agents
- **Docs agent**: Documentation writing with file operations but no system commands

## Example Agent Configurations

### Documentation Writer

`~/.config/opencode/agents/docs-writer.md`:

```markdown
---
description: Writes and maintains project documentation
mode: subagent
tools:
  bash: false
---

You are a technical writer. Create clear, comprehensive documentation.

Focus on:
- Clear explanations
- Proper structure
- Code examples
- User-friendly language
```

### Security Auditor

`~/.config/opencode/agents/security-auditor.md`:

```markdown
---
description: Performs security audits and identifies vulnerabilities
mode: subagent
tools:
  write: false
  edit: false
---

You are a security expert. Focus on identifying potential security issues.

Look for:
- Input validation vulnerabilities
- Authentication and authorization flaws
- Data exposure risks
- Dependency vulnerabilities
- Configuration security issues
```

### Code Analyzer

`.opencode/agents/analyze.md`:

```markdown
---
description: Analyzes code quality and suggests improvements
mode: subagent
temperature: 0.1
model: anthropic/claude-sonnet-4-20250514
tools:
  write: false
  edit: false
  bash:
    "*": deny
    "git diff*": allow
    "git log*": allow
---

You are a code analyst. Review code for:

1. **Quality**: Adherence to best practices and patterns
2. **Performance**: Identify bottlenecks and optimization opportunities
3. **Maintainability**: Code structure, naming, and documentation
4. **Complexity**: Suggest simplifications where appropriate

Provide actionable recommendations with examples.
```

### Test Coverage Agent

`.opencode/agents/test-coverage.md`:

```markdown
---
description: Analyzes test coverage and suggests test improvements
mode: subagent
temperature: 0.2
tools:
  write: true
  edit: true
  bash:
    "*": ask
    "npm test*": allow
    "pytest*": allow
    "cargo test*": allow
---

You are a testing expert. Focus on:

1. Analyze existing test coverage
2. Identify untested code paths
3. Suggest new test cases for edge cases
4. Recommend testing strategies (unit, integration, e2e)
5. Write test implementations when requested

Prioritize high-value tests that catch real bugs.
```

## Best Practices

### Agent Design
1. **Single Responsibility**: Each agent should have a clear, focused purpose
2. **Descriptive Names**: Use names that clearly indicate the agent's function
3. **Appropriate Temperature**: Match temperature to task (low for analysis, higher for creativity)
4. **Tool Minimalism**: Only enable tools the agent actually needs

### Permission Strategy
1. **Principle of Least Privilege**: Start with minimal permissions, add as needed
2. **Ask for Destructive Operations**: Use `"ask"` for operations like `git push`
3. **Deny by Default**: Use `"*": "deny"` then allow specific operations
4. **Document Permissions**: Explain why certain permissions are needed in comments

### Prompt Engineering
1. **Clear Instructions**: Be explicit about what the agent should and shouldn't do
2. **Structured Guidance**: Use numbered lists or sections for complex behaviors
3. **Examples**: Include examples of expected behavior when helpful
4. **Constraints**: Clearly state limitations and boundaries

### Organization
1. **Global vs Project**: Put reusable agents in global config, project-specific in `.opencode/`
2. **Markdown for Complex**: Use markdown files for agents with longer prompts
3. **JSON for Simple**: Use JSON config for simple, configuration-heavy agents
4. **Consistent Naming**: Use lowercase with hyphens (e.g., `code-reviewer`, `test-writer`)

## Troubleshooting

### Agent Not Appearing
- Check file name matches agent name
- Verify frontmatter syntax is valid YAML
- Ensure `disable: true` is not set
- For subagents, verify `mode: subagent` is set

### Wrong Model Being Used
- Check if agent-specific `model` config is set
- Verify global model configuration
- For subagents, check if invoking primary agent has model override

### Permission Issues
- Review permission hierarchy: agent-specific > global config
- Check for wildcard patterns that might conflict
- Verify bash command patterns match exactly (use `*` for globs)
- Remember: last matching rule wins

### Tool Not Available
- Check both global `tools` config and agent-specific `tools` config
- Verify tool is enabled in OpenCode installation
- For MCP tools, check MCP server is running

## Additional Resources

- [OpenCode Agents Documentation](https://opencode.ai/docs/agents/)
- [OpenCode Tools Documentation](https://opencode.ai/docs/tools/)
- [OpenCode Permissions Documentation](https://opencode.ai/docs/permissions/)
- [OpenCode Models Documentation](https://opencode.ai/docs/models/)
