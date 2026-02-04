# OpenCode Agent Skills Guide

## Overview

Agent Skills in OpenCode are reusable instruction sets that agents can discover and load on-demand via the native `skill` tool. Skills enable you to define specialized behaviors that can be shared across agents and projects.

## Key Concepts

### On-Demand Loading
- Agents see **available skills** in their tool descriptions (name + description only)
- Agents **load full content** when needed via `skill({ name: "skill-name" })` tool call
- Reduces token usage by loading only when necessary

### Discovery Locations
OpenCode searches for skills in these locations (in order):
1. Project config: `.opencode/skills/<name>/SKILL.md`
2. Global config: `~/.config/opencode/skills/<name>/SKILL.md`
3. Claude-compatible (project): `.claude/skills/<name>/SKILL.md`
4. Claude-compatible (global): `~/.claude/skills/<name>/SKILL.md`

### Project-Local Discovery
For project paths, OpenCode walks up from your current working directory until it reaches the git worktree, loading any matching `skills/*/SKILL.md` files along the way.

## File Structure

### Required Structure
Each skill must be in its own directory with a `SKILL.md` file:

```
.opencode/skills/
├── git-release/
│   └── SKILL.md
├── code-review/
│   └── SKILL.md
└── security-audit/
    └── SKILL.md
```

### SKILL.md Format

All `SKILL.md` files must start with YAML frontmatter followed by skill content.

## Frontmatter Fields

### Required Fields

#### `name`
- **Length**: 1-64 characters
- **Format**: Lowercase alphanumeric with single hyphen separators
- **Pattern**: `^[a-z0-9]+(-[a-z0-9]+)*$`
- **Restrictions**:
  - Cannot start or end with `-`
  - Cannot contain consecutive `--`
  - **Must match the directory name** containing `SKILL.md`

**Valid examples**:
- `git-release`
- `code-review`
- `security-audit`
- `test-coverage`

**Invalid examples**:
- `git_release` (uses underscore)
- `-git-release` (starts with hyphen)
- `git--release` (consecutive hyphens)
- `Git-Release` (has uppercase)

#### `description`
- **Length**: 1-1024 characters
- **Purpose**: Shown to agents to help them decide when to use the skill
- **Best Practice**: Be specific enough for accurate selection

**Example**:
```yaml
description: Create consistent releases and changelogs from merged pull requests
```

### Optional Fields

#### `license`
Specify the skill's license.

```yaml
license: MIT
```

#### `compatibility`
Indicate which tools the skill is compatible with.

```yaml
compatibility: opencode
```

#### `metadata`
String-to-string key-value pairs for additional metadata.

```yaml
metadata:
  audience: maintainers
  workflow: github
  version: "1.0"
  author: team-name
```

### Unknown Fields
Unknown frontmatter fields are **ignored** by OpenCode.

## Creating Skills

### Basic Skill Example

`.opencode/skills/git-release/SKILL.md`:

```markdown
---
name: git-release
description: Create consistent releases and changelogs from merged pull requests
license: MIT
compatibility: opencode
metadata:
  audience: maintainers
  workflow: github
---

## What I Do

- Draft release notes from merged PRs
- Propose a version bump based on conventional commits
- Provide a copy-pasteable `gh release create` command

## When to Use Me

Use this skill when you are preparing a tagged release.
Ask clarifying questions if the target versioning scheme is unclear.

## Release Note Format

Generate release notes in this format:

### Features
- List new features from commits starting with `feat:`

### Bug Fixes
- List bug fixes from commits starting with `fix:`

### Breaking Changes
- Highlight any commits with `BREAKING CHANGE:` in body

## Version Bump Logic

- **Major**: Breaking changes present
- **Minor**: New features without breaking changes
- **Patch**: Only bug fixes and chores
```

### Advanced Skill Example

`.opencode/skills/code-review/SKILL.md`:

```markdown
---
name: code-review
description: Perform systematic code reviews focusing on quality, security, and performance
license: Apache-2.0
metadata:
  focus: quality-assurance
  languages: typescript,rust,python
---

## Review Checklist

### Code Quality
1. Naming conventions followed
2. Functions are single-purpose
3. No code duplication
4. Appropriate abstraction levels

### Security
1. Input validation present
2. No hardcoded secrets
3. Proper authentication/authorization
4. SQL injection prevention
5. XSS prevention

### Performance
1. No N+1 queries
2. Appropriate data structures
3. Caching opportunities identified
4. Algorithm complexity reasonable

### Testing
1. Critical paths have tests
2. Edge cases covered
3. Mock usage appropriate
4. Test names are descriptive

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

### Domain-Specific Skill

`.opencode/skills/rust-safety/SKILL.md`:

```markdown
---
name: rust-safety
description: Review Rust code for safety, correctness, and idiomatic patterns
license: MIT
metadata:
  language: rust
  focus: safety
---

## Rust-Specific Review

### Safety Analysis
1. Check for `unsafe` blocks - are they necessary?
2. Verify lifetime annotations are correct
3. Ensure no undefined behavior possibilities
4. Check for panic paths in production code

### Ownership & Borrowing
1. Verify ownership transfers are intentional
2. Check for unnecessary clones
3. Ensure borrows don't outlive owned data
4. Look for Rc/Arc usage - is it needed?

### Error Handling
1. Prefer Result over panic
2. Use ? operator consistently
3. Custom error types when appropriate
4. Avoid unwrap() in library code

### Idiomatic Patterns
1. Use iterators over manual loops
2. Leverage pattern matching
3. Use builder pattern for complex construction
4. Implement standard traits (Debug, Clone, etc.)

### Performance
1. Check for unnecessary allocations
2. Use appropriate collection types
3. Consider using Cow when appropriate
4. Profile before optimizing
```

## How Skills Appear to Agents

### In Tool Description

Agents see available skills in the `skill` tool description:

```xml
<available_skills>
  <skill>
    <name>git-release</name>
    <description>Create consistent releases and changelogs from merged pull requests</description>
  </skill>
  <skill>
    <name>code-review</name>
    <description>Perform systematic code reviews focusing on quality, security, and performance</description>
  </skill>
</available_skills>
```

### Loading a Skill

Agent loads skill by calling:

```javascript
skill({ name: "git-release" })
```

The full SKILL.md content is then provided to the agent.

## Permission System

### Global Permissions

Control which skills agents can access using pattern-based permissions:

```json
{
  "permission": {
    "skill": {
      "*": "allow",
      "internal-*": "deny",
      "experimental-*": "ask"
    }
  }
}
```

### Permission Levels

| Permission | Behavior |
|------------|----------|
| `allow` | Skill loads immediately without prompting |
| `deny` | Skill hidden from agent, access rejected |
| `ask` | User prompted for approval before loading |

### Pattern Matching

Supports wildcards:
- `internal-*` matches `internal-docs`, `internal-tools`, etc.
- `*-experimental` matches `feature-experimental`, `api-experimental`, etc.

### Rule Precedence

Rules are evaluated in order, and the **last matching rule wins**:

```json
{
  "permission": {
    "skill": {
      "*": "deny",              // Deny everything by default
      "approved-*": "allow",     // Allow approved skills
      "experimental-*": "ask"    // Ask for experimental
    }
  }
}
```

## Per-Agent Skill Permissions

### For Custom Agents (Markdown)

Override global permissions in agent frontmatter:

```markdown
---
description: Documentation agent with access to doc skills
permission:
  skill:
    "doc-*": "allow"
    "internal-*": "allow"
---

You are a documentation specialist.
```

### For Built-in Agents (JSON)

Override in `opencode.json`:

```json
{
  "agent": {
    "plan": {
      "permission": {
        "skill": {
          "internal-*": "allow",
          "security-*": "ask"
        }
      }
    }
  }
}
```

## Disabling Skills for Agents

### Complete Disable for Custom Agents

In agent markdown frontmatter:

```markdown
---
description: Simple agent without skill access
tools:
  skill: false
---

You have no access to skills.
```

### Complete Disable for Built-in Agents

In `opencode.json`:

```json
{
  "agent": {
    "plan": {
      "tools": {
        "skill": false
      }
    }
  }
}
```

When disabled, the `<available_skills>` section is omitted entirely from the agent's tool description.

## Example Skills Library

### Testing & Quality

`.opencode/skills/test-strategy/SKILL.md`:

```markdown
---
name: test-strategy
description: Design comprehensive testing strategies for new features
license: MIT
---

## Testing Pyramid

1. **Unit Tests (70%)**
   - Test individual functions/methods
   - Mock external dependencies
   - Fast execution, high coverage

2. **Integration Tests (20%)**
   - Test component interactions
   - Use test databases/services
   - Verify contracts between modules

3. **E2E Tests (10%)**
   - Test critical user journeys
   - Use production-like environment
   - Keep minimal, focus on high-value paths

## Test Design Principles

- **Arrange-Act-Assert**: Clear test structure
- **Given-When-Then**: BDD-style for readability
- **One assertion per test**: Focus on single behavior
- **Descriptive names**: Test name explains what and why

## Coverage Goals

- **Critical paths**: 100% coverage
- **Business logic**: 90%+ coverage
- **Utility functions**: 80%+ coverage
- **UI components**: Focus on logic, not rendering details
```

### Documentation

`.opencode/skills/doc-generation/SKILL.md`:

```markdown
---
name: doc-generation
description: Generate comprehensive API and code documentation
license: MIT
metadata:
  output: markdown
---

## Documentation Structure

### For Functions/Methods

```
functionName(param1, param2)

Brief one-line description.

Longer description explaining purpose, behavior, and any important details.

Parameters:
- param1 (Type): Description
- param2 (Type): Description

Returns:
- Type: Description

Throws:
- ErrorType: When and why

Examples:
```language
// Example usage
```
```

### For Classes

Include:
- Class purpose and responsibility
- Constructor parameters
- Public methods with descriptions
- Usage examples
- Related classes

### For Modules

Include:
- Module purpose
- Exported items
- Usage patterns
- Configuration options
```

### Security

`.opencode/skills/security-checklist/SKILL.md`:

```markdown
---
name: security-checklist
description: Comprehensive security review checklist for web applications
license: MIT
metadata:
  domain: web-security
  owasp: "2024"
---

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
```

## Best Practices

### Skill Design

1. **Single Responsibility**: Each skill should focus on one domain or task type
2. **Comprehensive Instructions**: Provide clear, step-by-step guidance
3. **Examples**: Include examples where helpful
4. **Checklists**: Use checklists for systematic reviews
5. **Format Specifications**: Define expected output formats

### Naming

1. **Descriptive**: Name clearly indicates skill purpose
2. **Consistent**: Use consistent naming patterns across related skills
3. **Domain-Prefixed**: Consider prefixing by domain (e.g., `rust-`, `security-`, `doc-`)
4. **Avoid Ambiguity**: Name should be unambiguous

### Description Writing

1. **Specific**: Clear about what the skill does
2. **Concise**: 1-2 sentences typically sufficient
3. **Action-Oriented**: Start with verb (e.g., "Create", "Review", "Analyze")
4. **Context**: Include when/why to use the skill

### Organization

1. **Directory Structure**: Group related skills in your project
2. **Version Control**: Track skills in git
3. **Documentation**: Maintain a README in skills directory
4. **Sharing**: Consider publishing useful skills

### Content Structure

1. **Clear Sections**: Use headers to organize content
2. **Lists**: Use lists for steps, checklists, criteria
3. **Examples**: Provide code examples where relevant
4. **Format Specifications**: Define expected outputs

## Troubleshooting

### Skill Not Loading

**Check:**
- [ ] File is named exactly `SKILL.md` (all caps)
- [ ] Frontmatter includes required `name` and `description`
- [ ] Name matches directory name exactly
- [ ] Name follows validation rules (lowercase, alphanumeric, hyphens)
- [ ] Directory is in correct location (`.opencode/skills/` or `~/.config/opencode/skills/`)

### Skill Not Appearing to Agent

**Check:**
- [ ] Permission is not set to `deny` for this skill
- [ ] Agent has `skill` tool enabled (`tools.skill` not `false`)
- [ ] Name is unique across all skill locations
- [ ] YAML frontmatter is valid

### Permission Issues

**Check:**
- [ ] Global permission configuration in `opencode.json`
- [ ] Agent-specific permission overrides
- [ ] Pattern matching rules (remember: last match wins)
- [ ] Skill name matches permission patterns

### Name Validation Errors

**Common issues:**
- Name contains uppercase letters → use lowercase only
- Name uses underscores → use hyphens instead
- Name starts/ends with hyphen → remove leading/trailing hyphens
- Name has consecutive hyphens → use single hyphens
- Name doesn't match directory name → make them identical

## Migration from Claude

If migrating from Claude's `.claude/skills/` structure:

1. **Compatible Structure**: OpenCode reads `.claude/skills/<name>/SKILL.md` files
2. **Frontmatter Compatibility**: Uses same frontmatter format
3. **Validation**: OpenCode has stricter name validation
4. **Gradual Migration**: Keep `.claude/` files while transitioning

**Migration steps**:
1. Copy `.claude/skills/` to `.opencode/skills/`
2. Validate all skill names follow OpenCode rules
3. Test skills load correctly
4. Update any custom tooling
5. Remove `.claude/skills/` when ready

## Additional Resources

- [OpenCode Skills Documentation](https://opencode.ai/docs/skills/)
- [OpenCode Agents Documentation](https://opencode.ai/docs/agents/)
- [OpenCode Permissions Documentation](https://opencode.ai/docs/permissions/)
- [OpenCode Custom Tools Documentation](https://opencode.ai/docs/custom-tools/)
