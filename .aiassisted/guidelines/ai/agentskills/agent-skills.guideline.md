---
description: Guidelines for creating and using Agent Skills - an open format for extending AI agent capabilities.
globs: "**/*"
---

# Agent Skills Guidelines

Agent Skills are a lightweight, open format for extending AI agent capabilities with specialized knowledge and workflows. This guideline covers the specification, structure, and best practices for creating effective skills.

## Overview

A skill is a folder containing a `SKILL.md` file with metadata and instructions that tell an agent how to perform a specific task. Skills can also bundle scripts, templates, and reference materials.

**Key benefits:**
- **Progressive disclosure**: Agents load only metadata at startup, full instructions on activation
- **Portable**: Skills are just files, easy to edit, version, and share
- **Extensible**: Can range from simple instructions to executable code and assets

## Directory Structure

```
skill-name/
├── SKILL.md          # Required: instructions + metadata
├── scripts/          # Optional: executable code
├── references/       # Optional: documentation
└── assets/           # Optional: templates, resources
```

## SKILL.md Format

Every skill starts with a `SKILL.md` file containing YAML frontmatter and Markdown instructions.

### Required Frontmatter

```yaml
---
name: skill-name
description: A description of what this skill does and when to use it.
---
```

### Optional Frontmatter Fields

```yaml
---
name: pdf-processing
description: Extract text and tables from PDF files, fill forms, merge documents.
license: Apache-2.0
compatibility: Requires git, docker, jq
metadata:
  author: example-org
  version: "1.0"
allowed-tools: Bash(git:*) Read
---
```

### Field Specifications

| Field | Required | Constraints |
|-------|----------|-------------|
| `name` | Yes | Max 64 chars. Lowercase letters, numbers, hyphens only. Must not start/end with hyphen. Must match directory name. |
| `description` | Yes | Max 1024 chars. Non-empty. Describes what the skill does and when to use it. |
| `license` | No | License name or reference to bundled license file. |
| `compatibility` | No | Max 500 chars. Environment requirements (tools, network access, etc.). |
| `metadata` | No | Arbitrary key-value mapping for additional metadata. |
| `allowed-tools` | No | Space-delimited list of pre-approved tools. (Experimental) |

### Name Field Rules

- Must be 1-64 characters
- Only lowercase alphanumeric characters and hyphens (`a-z`, `0-9`, `-`)
- Cannot start or end with `-`
- Cannot contain consecutive hyphens (`--`)
- Must match the parent directory name

**Valid examples:**
```yaml
name: pdf-processing
name: data-analysis
name: code-review
```

**Invalid examples:**
```yaml
name: PDF-Processing  # uppercase not allowed
name: -pdf            # cannot start with hyphen
name: pdf--processing # consecutive hyphens not allowed
```

### Description Field Best Practices

Write in **third person** and include:
1. What the skill does
2. When to use it
3. Specific keywords for discovery

**Good example:**
```yaml
description: Extracts text and tables from PDF files, fills forms, and merges documents. Use when working with PDF files or when the user mentions PDFs, forms, or document extraction.
```

**Poor example:**
```yaml
description: Helps with PDFs.
```

## Naming Conventions

Use **gerund form** (verb + -ing) for skill names:

- `processing-pdfs`
- `analyzing-spreadsheets`
- `committing-changes`
- `reviewing-code`

Alternative acceptable patterns:
- Noun phrases: `pdf-processing`, `code-review`
- Action-oriented: `process-pdfs`, `review-code`

## Progressive Disclosure

Structure skills for efficient context usage:

1. **Metadata** (~100 tokens): `name` and `description` loaded at startup for all skills
2. **Instructions** (< 5000 tokens recommended): Full `SKILL.md` body loaded on activation
3. **Resources** (as needed): Files in `scripts/`, `references/`, `assets/` loaded only when required

**Guidelines:**
- Keep `SKILL.md` under 500 lines
- Move detailed reference material to separate files
- Use relative paths from skill root for file references
- Keep file references one level deep from `SKILL.md`

## Body Content Structure

The Markdown body should include:

1. **Quick start / Overview**: Immediate value
2. **Step-by-step instructions**: Clear workflow
3. **Examples**: Input/output pairs
4. **Edge cases**: Common problems and solutions
5. **References**: Links to detailed files when needed

### Example Structure

```markdown
# Skill Name

## When to use this skill
Use this skill when...

## Quick start
[Basic usage example]

## Detailed workflow
1. Step one...
2. Step two...

## Examples
[Input/output pairs]

## Advanced features
See [references/advanced.md](references/advanced.md) for details.
```

## Optional Directories

### scripts/

Contains executable code. Scripts should:
- Be self-contained or document dependencies
- Include helpful error messages
- Handle edge cases gracefully
- Solve problems rather than punt to Claude

### references/

Contains additional documentation loaded on demand:
- `REFERENCE.md` - Technical reference
- Domain-specific files (`finance.md`, `legal.md`, etc.)
- Keep files focused and single-purpose

### assets/

Contains static resources:
- Templates (document, configuration)
- Images (diagrams, examples)
- Data files (lookup tables, schemas)

## Best Practices

### Conciseness

Only add context the agent doesn't already have. Challenge each piece of information:
- "Does the agent really need this explanation?"
- "Can I assume the agent knows this?"
- "Does this paragraph justify its token cost?"

### Degrees of Freedom

Match specificity to task fragility:

**High freedom** (text-based instructions): Multiple approaches valid, context-dependent decisions
**Medium freedom** (pseudocode/parameterized scripts): Preferred pattern exists, some variation acceptable
**Low freedom** (specific scripts, no parameters): Operations are fragile, consistency critical

### Feedback Loops

Implement validation loops for critical operations:
1. Execute action
2. Validate result
3. Fix errors if any
4. Repeat until success

### Avoid Anti-Patterns

- Don't use Windows-style paths (use forward slashes)
- Don't offer too many options (provide defaults)
- Don't include time-sensitive information
- Don't create deeply nested references
- Don't assume packages are installed

## Validation

Use the skills-ref library to validate skills:

```bash
skills-ref validate ./my-skill
```

## Integration

Skills are discovered by agents scanning configured directories. The `<available_skills>` XML format is used for Claude:

```xml
<available_skills>
  <skill>
    <name>pdf-processing</name>
    <description>Extracts text and tables from PDF files.</description>
    <location>/path/to/skills/pdf-processing/SKILL.md</location>
  </skill>
</available_skills>
```

## References

- [Agent Skills Specification](https://agentskills.io/specification)
- [Agent Skills Best Practices](https://platform.claude.com/docs/en/agents-and-tools/agent-skills/best-practices)
- [Example Skills Repository](https://github.com/anthropics/skills)
- [skills-ref Reference Library](https://github.com/agentskills/agentskills/tree/main/skills-ref)
