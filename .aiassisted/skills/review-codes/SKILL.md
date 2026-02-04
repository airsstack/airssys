---
name: review-codes
description: Review code changes for quality, design, and maintainability. Use when reviewing pull requests, code changes, or performing code reviews. Applies general engineering code review standards.
license: MIT
compatibility: opencode
metadata:
  focus: code-review
  type: generic
---

# Code Review Skill

Review code changes following the project's Code Review Process Guidelines.

## When to Use

Use this skill when:
- Reviewing pull requests or merge requests
- Performing code reviews on any codebase
- Evaluating code quality and design decisions
- Providing constructive feedback on code changes

## Instructions

### Step 1: Load the Guidelines

Read the code review process guidelines:
- **Primary**: `.aiassisted/guidelines/engineering/code-review-process.md`

### Step 2: Understand the Change

1. Read the change description (PR title, commit messages)
2. Ask: Does this change make sense? Is it solving the right problem?
3. If the change should not exist, explain why and suggest alternatives

### Step 3: Identify Main Components

Find the primary files with significant logical changes. Review these first to understand context.

**Communicate major design issues immediately** - don't wait until reviewing all files.

### Step 4: Review Systematically

Check each area from the guidelines:

| Area | What to Check |
|------|---------------|
| **Design** | Do interactions make sense? Right place in codebase? |
| **Functionality** | Does it work as intended? Edge cases? Concurrency issues? |
| **Complexity** | Is it more complex than necessary? Over-engineered? |
| **Testing** | Appropriate tests? Will they catch regressions? |
| **Naming** | Descriptive? Follows conventions? |
| **Comments** | Explain "why" not "what"? Accurate? |
| **Style** | Follows style guide? Consistent? |
| **Documentation** | README/API docs updated if needed? |

### Step 5: Provide Feedback

Format your review using severity labels:

```
**Nit:** Consider renaming `x` to `userCount` for clarity.

**Optional:** This could use early return to reduce nesting.

**FYI:** The new `async/await` syntax simplifies this pattern.

This function has a potential null pointer issue on line 42.

**Blocking:** This introduces a SQL injection vulnerability.
```

#### Severity Labels

| Label | Meaning |
|-------|---------|
| **Nit:** | Minor suggestion, optional |
| **Optional:** | Consider this, not required |
| **FYI:** | Educational, no action needed |
| *(no label)* | Should be addressed before approval |
| **Blocking:** | Must be fixed before merge |

### Step 6: Make a Decision

**Approve** when:
- Change improves overall code health
- Functions correctly and is well-tested
- Follows project standards
- No security vulnerabilities

**Request Changes** when:
- Change worsens code health
- Contains bugs or logic errors
- Missing tests or security issues
- Approach is fundamentally flawed

## Review Comment Guidelines

### Good Comments

- Focus on the code, not the developer
- Explain the reasoning and impact
- Reference best practices or documentation
- Point out problems; let developers find solutions

### Avoid

- "Why did you do it this way?" (judgmental)
- "This doesn't look right." (vague)
- "Use X instead." (prescriptive without explanation)

## Core Principle

> **Approve a change once it definitely improves the overall code health of the system, even if it is not perfect.**

Balance developer velocity with code quality. Reviews should be collaborative, not adversarial.

## Reference

The complete code review process is documented in:
`.aiassisted/guidelines/engineering/code-review-process.md`

Always load and follow the guidelines as the authoritative source.
