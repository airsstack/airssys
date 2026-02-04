# Claude Code Skills Customization Guide

## Overview

Skills in Claude Code extend what Claude can do by providing reusable instruction sets. Create a `SKILL.md` file with instructions, and Claude adds it to its toolkit. Claude uses skills when relevant, or you can invoke one directly with `/skill-name`.

Skills follow the [Agent Skills](https://agentskills.io) open standard, which works across multiple AI tools. Claude Code extends the standard with additional features like invocation control, subagent execution, and dynamic context injection.

**Note**: Custom slash commands have been merged into skills. Files in `.claude/commands/review.md` and `.claude/skills/review/SKILL.md` both create `/review` and work the same way. Existing `.claude/commands/` files keep working. Skills add optional features: directory for supporting files, frontmatter control, and automatic loading.

## Getting Started

### Create Your First Skill

This example creates a skill that teaches Claude to explain code using visual diagrams and analogies.

**Step 1: Create the skill directory**

Personal skills are available across all projects:

```bash
mkdir -p ~/.claude/skills/explain-code
```

**Step 2: Write SKILL.md**

Every skill needs a `SKILL.md` file with two parts:
1. **YAML frontmatter** (between `---` markers): Tells Claude when to use the skill
2. **Markdown content**: Instructions Claude follows when skill is invoked

The `name` field becomes the `/slash-command`, and the `description` helps Claude decide when to load automatically.

Create `~/.claude/skills/explain-code/SKILL.md`:

```yaml
---
name: explain-code
description: Explains code with visual diagrams and analogies. Use when explaining how code works, teaching about a codebase, or when the user asks "how does this work?"
---

When explaining code, always include:

1. **Start with an analogy**: Compare the code to something from everyday life
2. **Draw a diagram**: Use ASCII art to show the flow, structure, or relationships
3. **Walk through the code**: Explain step-by-step what happens
4. **Highlight a gotcha**: What's a common mistake or misconception?

Keep explanations conversational. For complex concepts, use multiple analogies.
```

**Step 3: Test the skill**

Two ways to test:

**Let Claude invoke it automatically** by asking something that matches the description:
```
How does this code work?
```

**Or invoke it directly** with the skill name:
```
/explain-code src/auth/login.ts
```

Either way, Claude should include an analogy and ASCII diagram in its explanation.

## Skill Locations and Discovery

### Where Skills Live

Where you store a skill determines who can use it:

| Location | Path | Applies to |
|----------|------|------------|
| Enterprise | See managed settings | All users in your organization |
| Personal | `~/.claude/skills/<skill-name>/SKILL.md` | All your projects |
| Project | `.claude/skills/<skill-name>/SKILL.md` | This project only |
| Plugin | `<plugin>/skills/<skill-name>/SKILL.md` | Where plugin is enabled |

**Priority**: enterprise > personal > project

Plugin skills use `plugin-name:skill-name` namespace, so they cannot conflict with other levels.

If you have files in `.claude/commands/`, those work the same way, but if a skill and a command share the same name, the skill takes precedence.

### Automatic Discovery from Nested Directories

When you work with files in subdirectories, Claude Code automatically discovers skills from nested `.claude/skills/` directories.

Example: If editing `packages/frontend/file.ts`, Claude Code also looks for skills in `packages/frontend/.claude/skills/`.

This supports monorepo setups where packages have their own skills.

### Skill Directory Structure

Each skill is a directory with `SKILL.md` as the entrypoint:

```
my-skill/
├── SKILL.md           # Main instructions (required)
├── template.md        # Template for Claude to fill in
├── examples/
│   └── sample.md      # Example output showing expected format
└── scripts/
    └── validate.sh    # Script Claude can execute
```

**Required**: `SKILL.md` contains main instructions

**Optional**: Other files let you build more powerful skills:
- Templates for Claude to fill in
- Example outputs showing expected format
- Scripts Claude can execute
- Detailed reference documentation

Reference these files from your `SKILL.md` so Claude knows what they contain and when to load them.

**Note**: Files in `.claude/commands/` still work and support the same frontmatter. Skills are recommended since they support additional features like supporting files.

## Configuration

### Types of Skill Content

Skills can contain any instructions, but thinking about invocation helps guide what to include:

#### Reference Content

Adds knowledge Claude applies to current work. Conventions, patterns, style guides, domain knowledge. Runs inline so Claude can use it alongside conversation context.

```yaml
---
name: api-conventions
description: API design patterns for this codebase
---

When writing API endpoints:
- Use RESTful naming conventions
- Return consistent error formats
- Include request validation
```

#### Task Content

Step-by-step instructions for specific action, like deployments, commits, or code generation. Often invoked directly with `/skill-name` rather than letting Claude decide. Add `disable-model-invocation: true` to prevent automatic triggering.

```yaml
---
name: deploy
description: Deploy the application to production
context: fork
disable-model-invocation: true
---

Deploy the application:
1. Run the test suite
2. Build the application
3. Push to the deployment target
```

### Frontmatter Reference

Configure skill behavior using YAML frontmatter fields between `---` markers:

```yaml
---
name: my-skill
description: What this skill does
disable-model-invocation: true
allowed-tools: Read, Grep
---

Your skill instructions here...
```

All fields are optional. Only `description` is recommended so Claude knows when to use the skill.

| Field | Required | Description |
|-------|----------|-------------|
| `name` | No | Display name. If omitted, uses directory name. Lowercase letters, numbers, hyphens only (max 64 chars) |
| `description` | Recommended | What skill does and when to use it. Claude uses this to decide when to apply skill. If omitted, uses first paragraph |
| `argument-hint` | No | Hint shown during autocomplete. Example: `[issue-number]` or `[filename] [format]` |
| `disable-model-invocation` | No | Set to `true` to prevent Claude from auto-loading. Use for workflows you trigger manually. Default: `false` |
| `user-invocable` | No | Set to `false` to hide from `/` menu. Use for background knowledge. Default: `true` |
| `allowed-tools` | No | Tools Claude can use without asking permission when skill is active |
| `model` | No | Model to use when this skill is active |
| `context` | No | Set to `fork` to run in forked subagent context |
| `agent` | No | Which subagent type to use when `context: fork` is set |
| `hooks` | No | Hooks scoped to this skill's lifecycle |

### Available String Substitutions

Skills support string substitution for dynamic values:

| Variable | Description |
|----------|-------------|
| `$ARGUMENTS` | All arguments passed when invoking skill. If not present in content, arguments appended as `ARGUMENTS: <value>` |
| `${CLAUDE_SESSION_ID}` | Current session ID. Useful for logging, creating session-specific files, or correlating output |

**Example using substitutions**:

```yaml
---
name: session-logger
description: Log activity for this session
---

Log the following to logs/${CLAUDE_SESSION_ID}.log:

$ARGUMENTS
```

### Add Supporting Files

Skills can include multiple files in their directory. This keeps `SKILL.md` focused on essentials while letting Claude access detailed reference material only when needed.

Large reference docs, API specifications, or example collections don't need to load into context every time the skill runs.

```
my-skill/
├── SKILL.md (required - overview and navigation)
├── reference.md (detailed API docs - loaded when needed)
├── examples.md (usage examples - loaded when needed)
└── scripts/
    └── helper.py (utility script - executed, not loaded)
```

Reference supporting files from `SKILL.md` so Claude knows what each file contains and when to load it:

```markdown
## Additional resources

- For complete API details, see [reference.md](reference.md)
- For usage examples, see [examples.md](examples.md)
```

💡 **Tip**: Keep `SKILL.md` under 500 lines. Move detailed reference material to separate files.

### Control Who Invokes a Skill

By default, both you and Claude can invoke any skill. Two frontmatter fields restrict this:

#### `disable-model-invocation: true`
Only you can invoke the skill. Use for workflows with side effects or that you want to control timing.

You don't want Claude deciding to deploy because your code looks ready.

```yaml
---
name: deploy
description: Deploy the application to production
disable-model-invocation: true
---

Deploy $ARGUMENTS to production:

1. Run the test suite
2. Build the application
3. Push to the deployment target
4. Verify the deployment succeeded
```

#### `user-invocable: false`
Only Claude can invoke the skill. Use for background knowledge that isn't actionable as a command.

A `legacy-system-context` skill explains how an old system works. Claude should know this when relevant, but `/legacy-system-context` isn't a meaningful action for users to take.

### Invocation and Context Loading

| Frontmatter | You can invoke | Claude can invoke | When loaded into context |
|-------------|----------------|-------------------|--------------------------|
| (default) | Yes | Yes | Description always in context, full skill loads when invoked |
| `disable-model-invocation: true` | Yes | No | Description not in context, full skill loads when you invoke |
| `user-invocable: false` | No | Yes | Description always in context, full skill loads when invoked |

**Note**: In regular session, skill descriptions are loaded into context so Claude knows what's available, but full skill content only loads when invoked. Subagents with preloaded skills work differently: full skill content is injected at startup.

### Restrict Tool Access

Use `allowed-tools` field to limit which tools Claude can use when skill is active.

This skill creates read-only mode where Claude can explore files but not modify them:

```yaml
---
name: safe-reader
description: Read files without making changes
allowed-tools: Read, Grep, Glob
---
```

### Pass Arguments to Skills

Both you and Claude can pass arguments when invoking a skill. Arguments available via `$ARGUMENTS` placeholder.

This skill fixes a GitHub issue by number:

```yaml
---
name: fix-issue
description: Fix a GitHub issue
disable-model-invocation: true
---

Fix GitHub issue $ARGUMENTS following our coding standards.

1. Read the issue description
2. Understand the requirements
3. Implement the fix
4. Write tests
5. Create a commit
```

When you run `/fix-issue 123`, Claude receives "Fix GitHub issue 123 following our coding standards..."

If you invoke a skill with arguments but skill doesn't include `$ARGUMENTS`, Claude Code appends `ARGUMENTS: <your input>` to end of skill content so Claude still sees what you typed.

## Advanced Patterns

### Inject Dynamic Context

The `` !`command` `` syntax runs shell commands before the skill content is sent to Claude. The command output replaces the placeholder, so Claude receives actual data, not the command itself.

This skill summarizes a pull request by fetching live PR data with GitHub CLI:

```yaml
---
name: pr-summary
description: Summarize changes in a pull request
context: fork
agent: Explore
allowed-tools: Bash(gh:*)
---

## Pull request context
- PR diff: !`gh pr diff`
- PR comments: !`gh pr view --comments`
- Changed files: !`gh pr diff --name-only`

## Your task
Summarize this pull request...
```

When this skill runs:

1. Each `` !`command` `` executes immediately (before Claude sees anything)
2. The output replaces the placeholder in skill content
3. Claude receives fully-rendered prompt with actual PR data

This is preprocessing, not something Claude executes. Claude only sees the final result.

💡 **Tip**: To enable extended thinking in a skill, include the word "ultrathink" anywhere in your skill content.

### Run Skills in a Subagent

Add `context: fork` to frontmatter when you want a skill to run in isolation. The skill content becomes the prompt that drives the subagent. It won't have access to your conversation history.

⚠️ **Warning**: `context: fork` only makes sense for skills with explicit instructions. If your skill contains guidelines like "use these API conventions" without a task, the subagent receives the guidelines but no actionable prompt, and returns without meaningful output.

Skills and subagents work together in two directions:

| Approach | System prompt | Task | Also loads |
|----------|---------------|------|------------|
| Skill with `context: fork` | From agent type (`Explore`, `Plan`, etc.) | SKILL.md content | CLAUDE.md |
| Subagent with `skills` field | Subagent's markdown body | Claude's delegation message | Preloaded skills + CLAUDE.md |

With `context: fork`, you write the task in your skill and pick an agent type to execute it. For the inverse (defining a custom subagent that uses skills as reference material), see Subagents guide.

#### Example: Research Skill Using Explore Agent

This skill runs research in a forked Explore agent. The skill content becomes the task, and the agent provides read-only tools optimized for codebase exploration:

```yaml
---
name: deep-research
description: Research a topic thoroughly
context: fork
agent: Explore
---

Research $ARGUMENTS thoroughly:

1. Find relevant files using Glob and Grep
2. Read and analyze the code
3. Summarize findings with specific file references
```

When this skill runs:

1. A new isolated context is created
2. The subagent receives the skill content as its prompt ("Research $ARGUMENTS thoroughly...")
3. The `agent` field determines the execution environment (model, tools, and permissions)
4. Results are summarized and returned to your main conversation

The `agent` field specifies which subagent configuration to use. Options include built-in agents (`Explore`, `Plan`, `general-purpose`) or any custom subagent from `.claude/agents/`. If omitted, uses `general-purpose`.

### Restrict Claude's Skill Access

By default, Claude can invoke any skill that doesn't have `disable-model-invocation: true` set. Built-in commands like `/compact` and `/init` are not available through the Skill tool.

Three ways to control which skills Claude can invoke:

#### 1. Disable All Skills

Deny the Skill tool in `/permissions`:

```
# Add to deny rules:
Skill
```

#### 2. Allow or Deny Specific Skills

Using permission rules:

```
# Allow only specific skills
Skill(commit)
Skill(review-pr:*)

# Deny specific skills
Skill(deploy:*)
```

Permission syntax: `Skill(name)` for exact match, `Skill(name:*)` for prefix match with any arguments.

#### 3. Hide Individual Skills

Add `disable-model-invocation: true` to frontmatter. This removes the skill from Claude's context entirely.

**Note**: The `user-invocable` field only controls menu visibility, not Skill tool access. Use `disable-model-invocation: true` to block programmatic invocation.

## Sharing Skills

Skills can be distributed at different scopes depending on your audience:

- **Project skills**: Commit `.claude/skills/` to version control
- **Plugins**: Create a `skills/` directory in your plugin
- **Managed**: Deploy organization-wide through managed settings

## Example Skills

### Code Review Skill

```yaml
---
name: review-code
description: Reviews code for quality, security, and best practices
allowed-tools: Read, Grep, Glob, Bash
---

You are a code reviewer. Perform systematic code review:

## Review Checklist

### Code Quality
- Clear and readable code
- Well-named functions and variables
- No code duplication
- Appropriate abstraction levels

### Security
- Input validation present
- No hardcoded secrets
- Proper authentication/authorization
- SQL injection prevention
- XSS prevention

### Performance
- No N+1 queries
- Appropriate data structures
- Caching opportunities identified
- Algorithm complexity reasonable

### Testing
- Critical paths have tests
- Edge cases covered
- Mock usage appropriate
- Test names are descriptive

## Output Format

Provide feedback as:

**✅ Strengths**
- List what's done well

**⚠️ Concerns**
- List issues found with severity (High/Medium/Low)

**💡 Suggestions**
- List improvement recommendations

**📝 Nitpicks**
- List minor style/consistency issues
```

### Commit Message Generator

```yaml
---
name: commit
description: Generate conventional commit message from staged changes
disable-model-invocation: true
---

Generate a conventional commit message for staged changes:

!`git diff --staged`

Format: `type(scope): description`

Types: feat, fix, docs, refactor, test, chore

Rules:
- Use imperative mood ("add" not "added")
- No period at end
- Keep under 72 characters
- Include body if needed to explain "why"
- Include breaking changes footer if applicable
```

### API Documentation Generator

```yaml
---
name: doc-api
description: Generate comprehensive API documentation
allowed-tools: Read, Write
---

Generate API documentation for $ARGUMENTS

## Documentation Structure

### For each endpoint:

**Endpoint**: `METHOD /path`

**Description**: What this endpoint does

**Parameters**:
- `param1` (Type, required/optional): Description
- `param2` (Type, required/optional): Description

**Request Body**:
```json
{
  "example": "request"
}
```

**Response**:
```json
{
  "example": "response"
}
```

**Status Codes**:
- 200: Success description
- 400: Bad request description
- 401: Unauthorized description
- 500: Server error description

**Examples**:
```bash
curl -X METHOD https://api.example.com/path \
  -H "Authorization: Bearer TOKEN" \
  -d '{"example": "request"}'
```
```

### Test Coverage Analyzer

```yaml
---
name: test-coverage
description: Analyze test coverage and suggest improvements
context: fork
agent: general-purpose
---

Analyze test coverage for $ARGUMENTS:

!`npm test -- --coverage --json`

Based on coverage report:

1. **Identify gaps**: Which files/functions have low coverage?
2. **Prioritize**: Which uncovered code is most critical?
3. **Suggest tests**: What specific test cases should be added?
4. **Provide examples**: Show example test code for highest-priority gaps

Focus on high-value tests that catch real bugs.
```

### Visual Codebase Explorer

This skill generates an interactive HTML tree view of your codebase. See the full example in the Advanced Patterns section of the official documentation.

```yaml
---
name: codebase-visualizer
description: Generate an interactive collapsible tree visualization of your codebase. Use when exploring a new repo, understanding project structure, or identifying large files.
allowed-tools: Bash(python:*)
---

# Codebase Visualizer

Generate an interactive HTML tree view that shows your project's file structure with collapsible directories.

## Usage

Run the visualization script from your project root:

```bash
python ~/.claude/skills/codebase-visualizer/scripts/visualize.py .
```

This creates `codebase-map.html` in the current directory and opens it in your default browser.

## What the visualization shows

- **Collapsible directories**: Click folders to expand/collapse
- **File sizes**: Displayed next to each file
- **Colors**: Different colors for different file types
- **Directory totals**: Shows aggregate size of each folder
```

Create `~/.claude/skills/codebase-visualizer/scripts/visualize.py` with the visualization script (see official documentation for full script).

### Security Audit Skill

```yaml
---
name: security-audit
description: Perform comprehensive security audit based on OWASP Top 10
allowed-tools: Read, Grep, Glob, Bash
---

Perform security audit of $ARGUMENTS

## OWASP Top 10 Checks

### 1. Injection
- [ ] All inputs validated and sanitized
- [ ] Parameterized queries for SQL
- [ ] No eval() or similar dynamic execution
- [ ] Template engines escape by default

### 2. Broken Authentication
- [ ] Password strength requirements
- [ ] MFA supported
- [ ] Session timeout implemented
- [ ] Secure session management

### 3. Sensitive Data Exposure
- [ ] Data encrypted in transit (TLS)
- [ ] Data encrypted at rest
- [ ] No secrets in source code
- [ ] Proper key management

### 4. XML External Entities (XXE)
- [ ] XML parsing has XXE disabled
- [ ] Input validation for XML

### 5. Broken Access Control
- [ ] Authorization checks on all endpoints
- [ ] Principle of least privilege
- [ ] No direct object references exposed

### 6. Security Misconfiguration
- [ ] Default credentials changed
- [ ] Error messages don't leak info
- [ ] Security headers configured
- [ ] Unnecessary features disabled

### 7. Cross-Site Scripting (XSS)
- [ ] Output encoding implemented
- [ ] Content Security Policy configured
- [ ] Input validation for user content

### 8. Insecure Deserialization
- [ ] Avoid deserializing untrusted data
- [ ] Integrity checks on serialized objects

### 9. Using Components with Known Vulnerabilities
- [ ] Dependencies up to date
- [ ] Vulnerability scanning in CI
- [ ] No dependencies with known CVEs

### 10. Insufficient Logging & Monitoring
- [ ] Security events logged
- [ ] Logs don't contain sensitive data
- [ ] Monitoring and alerting configured

## Output Format

For each finding:
- **Severity**: Critical/High/Medium/Low
- **Issue**: Description of the vulnerability
- **Location**: File and line number
- **Impact**: What could happen if exploited
- **Remediation**: How to fix it
```

## Troubleshooting

### Skill Not Triggering

If Claude doesn't use your skill when expected:

1. Check description includes keywords users would naturally say
2. Verify skill appears in `What skills are available?`
3. Try rephrasing request to match description more closely
4. Invoke it directly with `/skill-name` if skill is user-invocable

### Skill Triggers Too Often

If Claude uses your skill when you don't want it:

1. Make description more specific
2. Add `disable-model-invocation: true` if you only want manual invocation

### Claude Doesn't See All My Skills

Skill descriptions are loaded into context so Claude knows what's available. If you have many skills, they may exceed the character budget (default 15,000 characters). Run `/context` to check for a warning about excluded skills.

To increase limit, set `SLASH_COMMAND_TOOL_CHAR_BUDGET` environment variable.

### Skill Not Loading

- Verify `SKILL.md` file exists in skill directory
- Check frontmatter syntax is valid YAML
- Ensure skill name follows rules (lowercase, alphanumeric, hyphens, max 64 chars)
- Check for conflicts with other skills (higher priority wins)

### Dynamic Context Not Injecting

- Verify `` !`command` `` syntax is correct (backticks inside)
- Test command runs successfully in terminal
- Check command output isn't too large
- Ensure command is allowed by tool permissions

### Subagent Fork Not Working

- Verify `context: fork` is set in frontmatter
- Check `agent` field specifies valid subagent type
- Ensure skill content includes actionable task (not just guidelines)
- Review subagent's tool permissions

## Best Practices

### Skill Design
1. **Single Purpose**: Each skill should do one thing well
2. **Clear Description**: Use keywords users would naturally say
3. **Explicit Instructions**: Be specific about what Claude should do
4. **Supporting Files**: Keep SKILL.md under 500 lines, move details to separate files

### Invocation Control
1. **Manual Workflows**: Use `disable-model-invocation: true` for deployments, commits, etc.
2. **Background Knowledge**: Use `user-invocable: false` for context that isn't a command
3. **Argument Hints**: Provide `argument-hint` to guide users

### Organization
1. **Project Skills**: Check into version control for team collaboration
2. **Personal Skills**: Put reusable skills in `~/.claude/skills/`
3. **Naming**: Use descriptive, kebab-case names

### Performance
1. **Tool Restrictions**: Use `allowed-tools` to limit what skill can do
2. **Model Selection**: Choose appropriate model for task complexity
3. **Fork When Needed**: Use `context: fork` for isolated, high-volume operations

## Additional Resources

- [Claude Code Skills Documentation](https://code.claude.com/docs/en/skills)
- [Claude Code Subagents Documentation](https://code.claude.com/docs/en/sub-agents)
- [Claude Code Hooks Documentation](https://code.claude.com/docs/en/hooks)
- [Agent Skills Open Standard](https://agentskills.io)
- [Claude Code Plugins Documentation](https://code.claude.com/docs/en/plugins)
