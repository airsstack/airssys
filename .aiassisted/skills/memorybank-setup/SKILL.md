---
name: memorybank-setup
description: Initialize and configure multi-project memory bank structure. Use when the user asks to set up a memory bank, initialize project context, or create the .memory-bank directory.
license: MIT
metadata:
  author: aiassisted
  version: "1.0"
  category: project-setup
---

# Memory Bank Setup

Initialize and configure the multi-project memory bank structure for AI-assisted development workflows.

## When to Use This Skill

Use this skill when:
- The user asks to "set up memory bank" or "initialize memory bank"
- Setting up a new project with context management
- The user wants AI assistants to maintain context between sessions
- Creating the `.memory-bank/` directory structure

## Prerequisites

Before running this skill, verify:

1. Check if `.memory-bank/` already exists:
   ```bash
   ls -la .memory-bank/ 2>/dev/null || echo "No existing memory bank found"
   ```

2. If it exists, ask the user if they want to:
   - Skip setup (preserve existing)
   - Reset and recreate (will lose existing data)

## Setup Checklist

### Step 1: Create Root Structure

Create the following directories and files:

```bash
mkdir -p .memory-bank
mkdir -p .memory-bank/workspace
mkdir -p .memory-bank/templates/docs
mkdir -p .memory-bank/context-snapshots
mkdir -p .memory-bank/sub-projects
```

### Step 2: Create README.md (Entry Point)

Create `.memory-bank/README.md`:

```markdown
# Memory Bank

> **WARNING**: Read the instructions before modifying any files in this directory.

This directory contains the multi-project memory bank for AI-assisted development.

## Instructions

Full documentation: `.aiassisted/instructions/multi-project-memory-bank.instructions.md`

## Structure

- `workspace/` - Shared workspace-level context
- `sub-projects/` - Individual sub-project contexts
- `templates/` - Documentation templates
- `context-snapshots/` - Saved context states
- `current-context.md` - Active sub-project tracker

## Quick Start

1. Read the full instructions first
2. Update `current-context.md` to set active sub-project
3. Create sub-project in `sub-projects/<name>/`
```

### Step 3: Create current-context.md

Create `.memory-bank/current-context.md`:

```markdown
# Current Context

**Active Sub-Project:** (none)

## Context Switch Log

| Date | From | To | Reason |
|------|------|-----|--------|
| YYYY-MM-DD | - | (first project) | Initial setup |
```

### Step 4: Create Workspace Files

Create `.memory-bank/workspace/project-brief.md`:

```markdown
# Workspace Project Brief

## Vision

[Describe the overall workspace/repository vision]

## Objectives

1. [Primary objective]
2. [Secondary objective]

## Architecture Overview

[High-level architecture description]

## Standards

Reference: `PROJECTS_STANDARD.md` (if exists)
```

Create `.memory-bank/workspace/shared-patterns.md`:

```markdown
# Shared Patterns

## Core Patterns

[Document patterns shared across sub-projects]

## Architecture Patterns

[Document architectural patterns]

## Methodology

[Document development methodology]
```

Create `.memory-bank/workspace/workspace-architecture.md`:

```markdown
# Workspace Architecture

## High-Level Structure

[Describe workspace structure]

## Sub-Project Relationships

[Document how sub-projects relate]

## Integration Points

[Document integration points]
```

Create `.memory-bank/workspace/workspace-progress.md`:

```markdown
# Workspace Progress

## Milestones

| Milestone | Status | Target Date |
|-----------|--------|-------------|
| [Milestone 1] | pending | YYYY-MM-DD |

## Strategic Decisions

[Document strategic decisions]

## Cross-Project Status

[Track cross-project progress]
```

### Step 5: Create Template Files

Create `.memory-bank/templates/docs/technical-debt-template.md`:

```markdown
# DEBT-XXX: [Title]

**Created:** YYYY-MM-DD
**Severity:** [low/medium/high/critical]
**Status:** [open/in-progress/resolved]

## Context

[What led to this technical debt]

## Description

[What the debt is]

## Impact

[How it affects the system]

## Resolution Plan

[How to resolve it]

## Tracking

- [ ] Issue created: #XXX
- [ ] Assigned to: [person/team]
- [ ] Target resolution: YYYY-MM-DD
```

Create `.memory-bank/templates/docs/knowledge-template.md`:

```markdown
# KNOWLEDGE-XXX: [Title]

**Created:** YYYY-MM-DD
**Category:** [architecture/patterns/performance/integration/security/domain]

## Overview

[Brief overview of the knowledge]

## Context

[Why this knowledge is important]

## Details

[Detailed explanation]

## Code Examples

[Relevant code examples]

## References

- [Related documentation]
- [External resources]
```

Create `.memory-bank/templates/docs/adr-template.md`:

```markdown
# ADR-XXX: [Title]

**Status:** [proposed/accepted/deprecated/superseded]
**Created:** YYYY-MM-DD
**Updated:** YYYY-MM-DD

## Context

[What is the issue we're seeing that motivates this decision]

## Decision

[What is the change we're proposing/have agreed to]

## Consequences

### Positive

- [Benefit 1]

### Negative

- [Drawback 1]

### Neutral

- [Note 1]

## Alternatives Considered

### Alternative 1

[Description and why rejected]
```

Create `.memory-bank/templates/docs/documentation-guidelines.md`:

```markdown
# Documentation Guidelines

## When to Create Documentation

### Technical Debt
- Any `TODO(DEBT)` comments in code
- Architectural shortcuts or compromises
- Known limitations or incomplete features

### Knowledge Documentation
- New architectural patterns
- Non-obvious algorithm choices
- External system integrations
- Performance optimizations
- Security-critical code paths

### Architecture Decision Records
- Technology selection decisions
- Architectural pattern choices
- Decisions affecting scalability/security/performance

## Quality Standards

- All code examples must compile
- Maintain cross-references
- Regular maintenance reviews

## Naming Conventions

- Use kebab-case for all files
- Prefix with type: DEBT-XXX, KNOWLEDGE-XXX, ADR-XXX
```

Create `.memory-bank/templates/docs/debt-index-template.md`:

```markdown
# Technical Debt Registry

## Open Debts

| ID | Title | Severity | Status | Created |
|----|-------|----------|--------|---------|
| DEBT-001 | [Title] | [severity] | open | YYYY-MM-DD |

## Resolved Debts

| ID | Title | Resolved | Resolution |
|----|-------|----------|------------|
```

Create `.memory-bank/templates/docs/adr-index-template.md`:

```markdown
# Architecture Decision Records

## Index

| ID | Title | Status | Date |
|----|-------|--------|------|
| ADR-001 | [Title] | [status] | YYYY-MM-DD |

## By Status

### Accepted

### Proposed

### Deprecated

### Superseded
```

### Step 6: Create First Sub-Project (Optional)

Ask the user if they want to create a first sub-project. If yes:

```bash
mkdir -p .memory-bank/sub-projects/<project-name>
mkdir -p .memory-bank/sub-projects/<project-name>/tasks
mkdir -p .memory-bank/sub-projects/<project-name>/docs/debts
mkdir -p .memory-bank/sub-projects/<project-name>/docs/knowledges
mkdir -p .memory-bank/sub-projects/<project-name>/docs/adr
```

Create the following files in `.memory-bank/sub-projects/<project-name>/`:

- `project-brief.md` - Foundation document
- `product-context.md` - Why the project exists
- `active-context.md` - Current work focus
- `system-patterns.md` - Technical patterns
- `tech-context.md` - Technology stack
- `progress.md` - Current status
- `tasks/_index.md` - Task registry
- `docs/debts/_index.md` - Debt registry
- `docs/adr/_index.md` - ADR registry

## Validation Checklist

After setup, verify:

- [ ] `.memory-bank/README.md` exists
- [ ] `.memory-bank/current-context.md` exists
- [ ] `.memory-bank/workspace/` contains 4 files
- [ ] `.memory-bank/templates/docs/` contains 6 files
- [ ] `.memory-bank/context-snapshots/` exists (empty is OK)
- [ ] `.memory-bank/sub-projects/` exists

Run verification:

```bash
echo "Verifying memory bank structure..."
test -f .memory-bank/README.md && echo "OK: README.md"
test -f .memory-bank/current-context.md && echo "OK: current-context.md"
test -d .memory-bank/workspace && echo "OK: workspace/"
test -d .memory-bank/templates/docs && echo "OK: templates/docs/"
test -d .memory-bank/context-snapshots && echo "OK: context-snapshots/"
test -d .memory-bank/sub-projects && echo "OK: sub-projects/"
echo "Memory bank setup complete!"
```

## Reference

For complete Memory Bank documentation and workflows, see:
`.aiassisted/instructions/multi-project-memory-bank.instructions.md`
