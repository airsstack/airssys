---
name: git-commit
description: Generates commit messages following Conventional Commits 1.0.0 specification. Use when the user asks to commit changes, write commit messages, or when staging and committing code changes.
metadata:
  author: aiassisted
  version: "1.0"
---

# Git Commit

Generates descriptive commit messages following the Conventional Commits specification by analyzing staged changes.

## When to use this skill

Use this skill when:
- The user asks to commit changes to the repository
- The user needs help writing commit messages
- Staging and committing code changes
- Following conventional commits format is required

## Workflow

1. Check repository status:
   ```bash
   git status
   ```

2. Review staged and unstaged changes:
   ```bash
   git diff --staged
   git diff
   ```

3. Review recent commit history for style consistency:
   ```bash
   git log --oneline -10
   ```

4. Stage relevant files (prefer specific files over `git add -A`):
   ```bash
   git add <specific-files>
   ```

5. Generate commit message following the format below

6. Create commit using HEREDOC for proper formatting:
   ```bash
   git commit -m "$(cat <<'EOF'
   <type>(<scope>): <description>

   [optional body]

   [optional footer]
   EOF
   )"
   ```

## Commit Message Format

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### Types

| Type | Description |
|------|-------------|
| `feat` | A new feature |
| `fix` | A bug fix |
| `docs` | Documentation only changes |
| `style` | Changes that don't affect code meaning (formatting, etc.) |
| `refactor` | Code change that neither fixes a bug nor adds a feature |
| `perf` | Code change that improves performance |
| `test` | Adding or correcting tests |
| `build` | Changes to build system or external dependencies |
| `ci` | Changes to CI configuration files and scripts |
| `chore` | Other changes that don't modify src or test files |
| `revert` | Reverts a previous commit |

### Rules

**Subject line:**
- Use imperative mood ("add" not "added")
- No period at the end
- Keep under 50 characters (max 72)

**Body (optional):**
- Use imperative mood
- Wrap lines at 72 characters
- Explain *what* and *why* vs. *how*

**Footer (optional):**
- Reference issues: `Closes #123`
- Breaking changes: `BREAKING CHANGE: description`

### Breaking Changes

Indicate breaking changes by:
1. Appending "!" after type/scope: `feat(api)!: remove endpoint`
2. Footer: `BREAKING CHANGE: The endpoint has been removed`

## Examples

**Feature:**
```
feat(auth): add login with google
```

**Bug fix:**
```
fix: prevent infinite loop in user validation
```

**Breaking change:**
```
feat(api)!: remove deprecated endpoint /v1/users

BREAKING CHANGE: The /v1/users endpoint has been removed. Use /v2/users instead.
```

**Documentation:**
```
docs: update readme with setup instructions
```

**With co-author (AI assisted):**
```
feat(templates): add recursive directory copying

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
```

## Detailed Reference

For complete Conventional Commits specification details, see [references/conventional-commits.md](references/conventional-commits.md).
