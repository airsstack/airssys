---
name: doc-code
description: Creates and reviews documentation embedded in source code including doc comments, API documentation, and inline documentation. Use when writing code documentation, reviewing doc quality, or adding examples to code.
license: MIT
metadata:
  author: aiassisted
  version: "1.0"
---

# Code Documentation Skill

Creates and reviews documentation embedded directly in source code, including doc comments, API documentation, and inline comments.

## When to Use

Use this skill when:
- Writing documentation comments for functions, types, or modules
- Adding examples to code documentation
- Reviewing code documentation quality
- Creating API reference documentation
- Adding inline comments to explain complex logic

## Documentation Types

### 1. API Documentation (Doc Comments)

Documentation for public interfaces that appears in generated docs.

**Purpose:** Describe what code does, how to use it, and what to expect.

**Characteristics:**
- Rendered into HTML documentation
- Often includes runnable examples
- Follows language-specific conventions

### 2. Inline Comments

Comments within code explaining implementation details.

**Purpose:** Explain *why* code works a certain way, not *what* it does.

**Characteristics:**
- Not rendered in external docs
- Explains non-obvious decisions
- Documents workarounds or edge cases

## Instructions

### Step 1: Load Language-Specific Guidelines

Based on the programming language, load the appropriate documentation guide:

| Language | Guideline File |
|----------|----------------|
| Rust | `.aiassisted/guidelines/rust/rust-documentation-guide.md` |
| Other | Apply general principles below |

### Step 2: Apply Documentation Quality Standards

Load and follow:
- `.aiassisted/guidelines/documentation/documentation-quality-standards.md`

Key requirements:
- No hyperbolic language
- No marketing speak
- Technical accuracy over promotional claims
- State facts, not opinions

### Step 3: Document Public Items

Every public item needs documentation covering:

#### Required Sections

| Section | When Required |
|---------|---------------|
| **Summary** | Always - first line, one sentence |
| **Description** | When behavior needs explanation |
| **Examples** | Always for public APIs |
| **Errors** | When function can fail |
| **Panics** | When function can panic |
| **Safety** | For unsafe code |

#### Documentation Template

```
[One-line summary of what this does]

[Detailed description - what it does, when to use it, important behavior]

# Examples

[Working, copy-paste-ready examples]

# Errors

[What errors can occur and when]

# Panics

[Conditions that cause panics]

# Safety

[Safety requirements for unsafe code]
```

### Step 4: Write Quality Examples

Examples must be:
- **Complete**: Can be copied and run
- **Focused**: Demonstrate one concept
- **Realistic**: Show actual use cases
- **Tested**: Verified to compile and run

#### Example Guidelines

```
Good example:
- Shows typical usage
- Handles errors appropriately
- Uses meaningful variable names
- Includes necessary imports

Bad example:
- Uses placeholder values ("foo", "bar")
- Ignores error handling
- Missing imports or context
- Demonstrates edge cases before basics
```

### Step 5: Write Effective Inline Comments

For inline comments:

**Do Comment:**
- Non-obvious algorithms or logic
- Workarounds for bugs or limitations
- Business rules encoded in code
- Performance-critical sections
- Security-sensitive code

**Don't Comment:**
- What the code literally does (code is self-documenting)
- Obvious operations
- Every line or block
- Redundant information

#### Inline Comment Examples

```
// Bad: Explains what (obvious)
// Add 1 to counter
counter += 1;

// Good: Explains why (non-obvious)
// Off-by-one adjustment: API returns 0-indexed positions but UI expects 1-indexed
counter += 1;

// Bad: Redundant with code
// Check if user is admin
if user.is_admin() { ... }

// Good: Documents business rule
// Only admins can delete archived records per compliance policy
if user.is_admin() { ... }
```

### Step 6: Review Documentation

When reviewing documentation:

#### Check for Completeness

- [ ] All public items documented
- [ ] Examples present and working
- [ ] Error conditions documented
- [ ] Panic conditions documented
- [ ] Safety requirements documented

#### Check for Quality

- [ ] Summary is one sentence
- [ ] No hyperbolic language
- [ ] Technical accuracy
- [ ] Examples are runnable
- [ ] Links are valid

#### Check for Style

- [ ] Consistent formatting
- [ ] Line length under 100 chars
- [ ] Proper grammar and spelling
- [ ] Follows language conventions

## Language-Specific Guidance

### Rust

```rust
/// Creates a new configuration with default values.
///
/// This function initializes a `Config` struct with sensible defaults
/// suitable for most use cases. Use [`Config::builder`] for customization.
///
/// # Examples
///
/// ```rust
/// use my_crate::Config;
///
/// let config = Config::new();
/// assert_eq!(config.timeout, Duration::from_secs(30));
/// ```
///
/// # Errors
///
/// Returns [`ConfigError::MissingEnv`] if required environment variables
/// are not set.
pub fn new() -> Result<Config, ConfigError> { ... }
```

### General (Other Languages)

Apply these principles regardless of language:

1. **First line is summary** - stands alone in listings
2. **Examples before edge cases** - show typical usage first
3. **Link to related items** - connect related functionality
4. **State facts, not opinions** - avoid "better", "best", "powerful"

## Common Documentation Patterns

### Constructor/Factory Functions

```
Summary: Creates a new X with Y.

Description: Brief explanation of what gets created and initial state.

Examples: Show basic creation and common configurations.

Errors: What can go wrong during creation.
```

### Configuration Types

```
Summary: Configuration for X.

Description: What this configures, default behavior.

Field docs: Each field documented individually.

Examples: Show common configuration patterns.
```

### Trait/Interface Types

```
Summary: Defines behavior for X.

Description: When to implement, what it provides.

Method docs: Each method documented with contract.

Examples: Show implementation example.
```

### Error Types

```
Summary: Errors that can occur during X.

Description: General error handling guidance.

Variant docs: Each variant with when it occurs.

Examples: Show error handling patterns.
```

## Quality Checklist

Before completing documentation:

- [ ] Summary is clear and under 80 characters
- [ ] All public items have documentation
- [ ] Examples compile and run correctly
- [ ] Error conditions are documented
- [ ] No forbidden terminology (see quality standards)
- [ ] Links to related items work
- [ ] Grammar and spelling are correct
- [ ] Consistent formatting throughout

## References

- `.aiassisted/guidelines/documentation/documentation-quality-standards.md`
- `.aiassisted/guidelines/rust/rust-documentation-guide.md` (for Rust)
- [Write the Docs Guide](https://www.writethedocs.org/guide/)
