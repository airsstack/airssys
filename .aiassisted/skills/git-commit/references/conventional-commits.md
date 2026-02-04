# Conventional Commits Reference

These instructions define the standard for commit messages in this workspace, based on the [Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/) specification.

## Format

The commit message should be structured as follows:

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

## Types

The following types are allowed:

- **feat**: A new feature
- **fix**: A bug fix
- **docs**: Documentation only changes
- **style**: Changes that do not affect the meaning of the code (white-space, formatting, missing semi-colons, etc)
- **refactor**: A code change that neither fixes a bug nor adds a feature
- **perf**: A code change that improves performance
- **test**: Adding missing tests or correcting existing tests
- **build**: Changes that affect the build system or external dependencies (example scopes: gulp, broccoli, npm)
- **ci**: Changes to our CI configuration files and scripts (example scopes: Travis, Circle, BrowserStack, SauceLabs)
- **chore**: Other changes that don't modify src or test files
- **revert**: Reverts a previous commit

## Breaking Changes

Breaking changes MUST be indicated in one of the following ways:
1. Appending a `!` after the type/scope, e.g., `feat!: ...` or `feat(api)!: ...`
2. Including a footer with `BREAKING CHANGE: <description>`

## Rules

1. **Subject Line**:
   - Use the imperative mood ("add" not "added", "change" not "changed").
   - No period at the end.
   - Keep it short (preferably under 50 chars, max 72).

2. **Body** (Optional):
   - Use the imperative mood.
   - Wrap lines at 72 characters.
   - Explain *what* and *why* vs. *how*.

3. **Footer** (Optional):
   - Reference issues (e.g., `Closes #123`).
   - Mention breaking changes.

## Examples

### Feature
```
feat(auth): add login with google
```

### Bug Fix
```
fix: prevent infinite loop in user validation
```

### Breaking Change
```
feat(api)!: remove deprecated endpoint /v1/users
```
OR
```
feat(api): remove deprecated endpoint /v1/users

BREAKING CHANGE: The /v1/users endpoint has been removed. Use /v2/users instead.
```

### Documentation
```
docs: update readme with setup instructions
```
