---
name: doc-project
description: Creates and manages project-level documentation including READMEs, tutorials, how-to guides, and architectural docs. Use when creating user-facing documentation, writing tutorials, or organizing project documentation following Diataxis framework.
license: MIT
metadata:
  author: aiassisted
  version: "1.0"
---

# Project Documentation Skill

Creates and manages project-level documentation following the Diataxis framework, including READMEs, tutorials, how-to guides, reference docs, and explanations.

## When to Use

Use this skill when:
- Creating or updating README files
- Writing tutorials for new users
- Creating how-to guides for specific tasks
- Documenting architecture and design decisions
- Organizing project documentation structure
- Reviewing project documentation quality

## Documentation Framework

This skill follows the **Diataxis** framework, which organizes documentation into four types based on user needs:

```
                PRACTICAL STEPS       THEORETICAL KNOWLEDGE
                ===============       =====================
LEARNING        | TUTORIALS   |       | EXPLANATION     |
(Study)         | (Learning)  |       | (Understanding) |
----------------+-------------+-------+-----------------+
TASK-ORIENTED   | HOW-TO      |       | REFERENCE       |
(Work)          | GUIDES      |       | (Information)   |
                ===============       =====================
```

## Instructions

### Step 1: Load Required Guidelines

Load these guidelines before writing:

1. **Diataxis Framework**: `.aiassisted/guidelines/documentation/diataxis-guidelines.md`
2. **Quality Standards**: `.aiassisted/guidelines/documentation/documentation-quality-standards.md`

### Step 2: Identify Documentation Type

Determine which type of documentation is needed:

| User Need | Documentation Type |
|-----------|-------------------|
| "I want to learn" | **Tutorial** |
| "I want to accomplish X" | **How-To Guide** |
| "I need to look up Y" | **Reference** |
| "I want to understand Z" | **Explanation** |

### Step 3: Apply Type-Specific Guidelines

#### Tutorials (Learning-Oriented)

**Purpose:** Learning experiences through practical steps

**Structure:**
```markdown
# Tutorial: [What You'll Build]

In this tutorial, we will [outcome]. You will learn [skills].

## Prerequisites

- [Required knowledge or setup]

## What You'll Build

[Brief description with screenshot/diagram if applicable]

## Step 1: [Action]

First, [instruction].

[Expected result or what to notice]

## Step 2: [Action]

Now, [instruction].

## Step 3: [Action]

[Continue pattern]

## Summary

You have now [accomplishment]. You learned how to [skills].

## Next Steps

- [Link to related tutorial]
- [Link to how-to guide for customization]
```

**Principles:**
- Every step produces visible results
- Focus on doing, not explaining
- Show expected output at each step
- Ruthlessly minimize explanation
- Aspire to perfect reliability

**Language:** "We will...", "First, do X. Now do Y.", "Notice that...", "You have built..."

#### How-To Guides (Task-Oriented)

**Purpose:** Directions to achieve specific goals

**Structure:**
```markdown
# How to [Accomplish Goal]

This guide shows you how to [specific outcome].

## Prerequisites

- [What user must have or know]

## Steps

### 1. [Action]

[Instructions]

### 2. [Action]

[Instructions]

If you need [variant], see [link to alternative guide].

### 3. [Action]

[Instructions]

## Verification

[How to confirm success]

## Troubleshooting

- **Problem X**: [Solution]
- **Problem Y**: [Solution]

## Related

- [Link to reference for details]
- [Link to explanation for context]
```

**Principles:**
- Focus on achieving the goal
- Assume user competence
- Address real-world complexity
- Omit unnecessary completeness
- Title says exactly what guide does

**Language:** "This guide shows you how to...", "If you want X, do Y.", "Refer to [reference] for details."

#### Reference (Information-Oriented)

**Purpose:** Accurate, authoritative technical descriptions

**Structure:**
```markdown
# [Component Name] Reference

## Overview

[Brief description of what this is]

## API / Interface

### `function_name(param: Type) -> ReturnType`

[Description]

**Parameters:**
- `param`: [Description]

**Returns:** [Description]

**Example:**
```code
[Usage example]
```

### `another_function(...)`

[Continue pattern]

## Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `x`    | string | `"default"` | [What it does] |

## Error Codes

| Code | Meaning |
|------|---------|
| E001 | [Description] |
```

**Principles:**
- Describe, don't instruct
- Austere and complete
- Mirror the structure of the code
- Consistent patterns throughout
- Examples illustrate without teaching

**Language:** "X inherits from Y...", "Options are: a, b, c...", "Returns [type]. Throws [error] when..."

#### Explanation (Understanding-Oriented)

**Purpose:** Deepen understanding through discussion

**Structure:**
```markdown
# Understanding [Topic]

## Background

[Context and history]

## How It Works

[Conceptual explanation]

## Design Decisions

[Why it works this way]

The approach was chosen because [reasoning].

Alternative approaches include [alternatives], which trade off [tradeoffs].

## Architecture

[Diagrams and component relationships]

## Tradeoffs

[Honest discussion of limitations and benefits]

## Related Concepts

- [Link to related explanation]
- [Link to how-to for practical application]
```

**Principles:**
- Provide context and background
- Make connections between concepts
- Discuss alternatives and tradeoffs
- Admit opinions where appropriate
- Can be read away from the product

**Language:** "The reason for X is...", "W is better than Z because...", "Some prefer W because...", "Historically..."

### Step 4: Apply Quality Standards

All documentation must follow quality standards:

**Do:**
- Use precise, technical terminology
- State capabilities factually
- Provide evidence for claims
- Indicate implementation status

**Don't:**
- Use hyperbolic language ("revolutionary", "blazing fast")
- Make absolute claims ("best", "universal", "zero downtime")
- Use self-promotional language ("our superior solution")
- Use marketing buzzwords without definition

See `.aiassisted/guidelines/documentation/documentation-quality-standards.md` for complete list.

### Step 5: Structure Project Documentation

Recommended directory structure:

```
project/
├── README.md              # Entry point (mix of tutorial + reference)
├── CONTRIBUTING.md        # How-to for contributors
├── CHANGELOG.md           # Reference for versions
├── docs/
│   ├── tutorials/         # Learning-oriented
│   │   ├── getting-started.md
│   │   └── first-project.md
│   ├── guides/            # Task-oriented
│   │   ├── configuration.md
│   │   └── deployment.md
│   ├── reference/         # Information-oriented
│   │   ├── api.md
│   │   └── cli.md
│   └── explanation/       # Understanding-oriented
│       ├── architecture.md
│       └── design-decisions.md
```

## README Structure

Every README should include:

```markdown
# Project Name

[One sentence description - what it is and what problem it solves]

## Features

- [Feature 1]
- [Feature 2]

## Installation

```bash
[Installation commands]
```

## Quick Start

```code
[Minimal working example]
```

## Documentation

- [Tutorials](docs/tutorials/)
- [How-To Guides](docs/guides/)
- [Reference](docs/reference/)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[License info]
```

## Documentation Review Checklist

### Content Quality

- [ ] Follows Diataxis type guidelines
- [ ] No hyperbolic or marketing language
- [ ] Technical claims are accurate
- [ ] Implementation status is clear
- [ ] Examples are complete and tested

### Structure

- [ ] Clear navigation between doc types
- [ ] Consistent formatting
- [ ] Appropriate links between sections
- [ ] No orphaned pages

### User Experience

- [ ] Entry point is clear (README)
- [ ] Path to first success is obvious
- [ ] Reference information is findable
- [ ] Explanations provide context

### Type-Specific Quality

**Tutorials:**
- [ ] Every step produces visible result
- [ ] Can be followed exactly
- [ ] Minimal explanation

**How-To Guides:**
- [ ] Specific goal stated
- [ ] Assumes competence
- [ ] Addresses real variations

**Reference:**
- [ ] Complete and accurate
- [ ] Consistent structure
- [ ] No instruction or explanation

**Explanation:**
- [ ] Provides context
- [ ] Discusses tradeoffs
- [ ] Bounded scope

## Common Patterns

### Migrating Existing Docs

1. Audit current docs
2. Categorize by Diataxis type
3. Identify gaps
4. Reorganize into structure
5. Fill gaps (prioritize tutorials and how-tos)
6. Apply quality standards

### Creating New Project Docs

1. Start with README (entry point)
2. Create getting-started tutorial
3. Add essential reference (API, CLI)
4. Create how-tos for common tasks
5. Add explanations for complex topics

### Improving Existing Docs

1. Load Diataxis guidelines
2. Identify current doc type
3. Check against type-specific quality
4. Apply quality standards
5. Add cross-links to related docs

## References

- `.aiassisted/guidelines/documentation/diataxis-guidelines.md`
- `.aiassisted/guidelines/documentation/documentation-quality-standards.md`
- [Diataxis Framework](https://diataxis.fr/)
- [Write the Docs Guide](https://www.writethedocs.org/guide/)
