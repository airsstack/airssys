# How to Configure Component Trust Levels

## Overview

**Trust levels** classify WASM components by their source and determine how quickly they can be installed. AirsSys uses a three-tier trust model to balance security with developer productivity:

- **Trusted**: Components from verified sources (instant installation)
- **Unknown**: Components with unverified origins (require security review)
- **DevMode**: Components in development (security checks bypassed with warnings)

This guide shows you how to configure trust sources and manage trust decisions for your component ecosystem.

---

## Trust Levels Explained

### Trusted Components

Components from configured trusted sources are **instantly approved** without manual review.

**When to use:**
- Internal organization components
- Well-known open-source projects
- Components signed with your organization's key
- Components in your local development workspace

**Benefits:**
- ✓ Instant installation for productivity
- ✓ Automated CI/CD workflows
- ✓ Minimal friction for internal teams

**Risk:** Lower if sources are carefully vetted

---

### Unknown Components

Components from unverified sources require **manual security review** before installation.

**When to use:**
- Third-party components from unknown sources
- Public registry components
- Community-contributed components
- Components from partners without pre-established trust

**Benefits:**
- ✓ Security review before deployment
- ✓ Approval audit trail
- ✓ Time to evaluate component permissions

**Workflow:**
1. Component installation requested
2. Security team reviews Component.toml
3. Team approves or rejects
4. Component installed (if approved)

---

### DevMode

Development mode **bypasses security checks** for rapid local iteration with visible warnings.

**When to use:**
- Local development and testing
- Rapid prototyping
- Component debugging
- CI/CD development environments

**⚠️ CRITICAL: Never use DevMode in production!**

DevMode produces visible warnings:
```
⚠️  ⚠️  ⚠️  DEVELOPMENT MODE ACTIVE ⚠️  ⚠️  ⚠️
Component: my-component
Security checks BYPASSED!
DO NOT use in production!
Timestamp: 2025-12-20T15:30:00Z
```

---

## Trust Configuration Format

Trust sources are configured in a TOML file, typically `trust-config.toml` in your project root.

### Basic Structure

```toml
[trust]
dev_mode = false                  # Global DevMode setting

[[trust.sources]]
type = "git"
# Git configuration...

[[trust.sources]]
type = "signing_key"
# Signing key configuration...

[[trust.sources]]
type = "local"
# Local path configuration...
```

---

## Trusted Source Types

### Git Repository Sources

Trust all components from specified Git repositories.

**Configuration:**
```toml
[[trust.sources]]
type = "git"
url_pattern = "https://github.com/myorg/*"
branch = "main"
description = "Internal organization repositories"
```

**How it works:**
1. Extract Git URL and branch from component metadata
2. Check if URL matches `url_pattern`
3. Verify component is from configured `branch`
4. If both match: **Trusted**

**Parameters:**

| Parameter | Required | Description |
|-----------|----------|-------------|
| `url_pattern` | Yes | Glob pattern for Git URLs (supports `*` wildcard) |
| `branch` | No | If specified, only trust components from this branch |
| `description` | Yes | Human-readable explanation of this trust source |

**Examples:**

```toml
# Trust all repos in organization
[[trust.sources]]
type = "git"
url_pattern = "https://github.com/myorg/*"
description = "Internal org repos"

# Trust specific repository
[[trust.sources]]
type = "git"
url_pattern = "https://github.com/myorg/trusted-components"
branch = "main"
description = "Vetted components repository"

# Trust multiple GitHub accounts
[[trust.sources]]
type = "git"
url_pattern = "https://github.com/{alice,bob}/*"
description = "Trusted maintainers"
```

---

### Signing Key Sources

Trust components signed with specific cryptographic keys.

**Configuration:**
```toml
[[trust.sources]]
type = "signing_key"
public_key = "ed25519:ABC123DEF456..."
signer = "security-team@example.com"
description = "Official security team signature"
```

**How it works:**
1. Extract component signature from metadata
2. Verify signature using configured `public_key`
3. If signature is valid: **Trusted**

**Parameters:**

| Parameter | Required | Description |
|-----------|----------|-------------|
| `public_key` | Yes | Ed25519 public key in format `ed25519:base64_encoded_key` |
| `signer` | Yes | Email or name identifying key holder |
| `description` | Yes | Explanation of when this key is used |

**Examples:**

```toml
# Trust official security team
[[trust.sources]]
type = "signing_key"
public_key = "ed25519:JXZgHMq9cGk3+/Q7FX..."
signer = "security-team@example.com"
description = "Official security team signing key"

# Trust specific maintainer
[[trust.sources]]
type = "signing_key"
public_key = "ed25519:xY9bR7pQmK2+zL/M1..."
signer = "alice@example.com"
description = "Alice's personal signing key for components"
```

---

### Local Path Sources

Trust components in specified local filesystem paths (development).

**Configuration:**
```toml
[[trust.sources]]
type = "local"
path_pattern = "/home/dev/components/*"
description = "Local development components"
```

**How it works:**
1. Extract component file path
2. Check if path matches `path_pattern`
3. If matched: **Trusted**

**Parameters:**

| Parameter | Required | Description |
|-----------|----------|-------------|
| `path_pattern` | Yes | Glob pattern for local paths (supports `*` and `**`) |
| `description` | Yes | Explanation of this local trust source |

**Examples:**

```toml
# Trust workspace components
[[trust.sources]]
type = "local"
path_pattern = "/Users/alice/workspace/components/*"
description = "Alice's workspace components"

# Trust all local components
[[trust.sources]]
type = "local"
path_pattern = "/home/*/dev/components/**/*"
description = "Developer workspace components"
```

---

## Complete Configuration Examples

### Example 1: Single Organization

Simple setup for organization-internal development.

```toml
[trust]
dev_mode = false

# All components from org repositories are trusted
[[trust.sources]]
type = "git"
url_pattern = "https://github.com/myorg/*"
description = "Internal organization repositories"

# Local development components in team workspace
[[trust.sources]]
type = "local"
path_pattern = "/home/dev/workspace/*"
description = "Team development workspace"
```

**Behavior:**
- ✓ Components from `https://github.com/myorg/foo` → Trusted
- ✓ Components from `/home/dev/workspace/components/` → Trusted
- ✗ Components from `https://github.com/other-org/foo` → Unknown

---

### Example 2: Multiple Signers

Organization with multiple authorized signers.

```toml
[trust]
dev_mode = false

# Official release signing key
[[trust.sources]]
type = "signing_key"
public_key = "ed25519:JXZgHMq9cGk3+/Q7FX..."
signer = "releases@myorg.com"
description = "Official release signing key"

# Security team review/approval
[[trust.sources]]
type = "signing_key"
public_key = "ed25519:xY9bR7pQmK2+zL/M1..."
signer = "security-review@myorg.com"
description = "Security team review approvals"

# Architecture team
[[trust.sources]]
type = "signing_key"
public_key = "ed25519:aBcDeF123456789..."
signer = "architecture@myorg.com"
description = "Architecture team approved components"
```

**Behavior:**
- ✓ Components signed with any configured key → Trusted
- ✗ Components unsigned or signed with unknown key → Unknown

---

### Example 3: Development with DevMode

Local development setup with DevMode enabled.

```toml
[trust]
dev_mode = true  # ⚠️  WARNING: Security checks bypassed

[[trust.sources]]
type = "local"
path_pattern = "/home/dev/workspace/*"
description = "Local development components"
```

**Behavior:**
- ✓ All components (regardless of source) → DevMode
- ⚠️  Security warnings printed for each component
- ✗ DO NOT use in production!

---

### Example 4: Tiered Trust

Production setup with multiple trust tiers.

```toml
[trust]
dev_mode = false

# Tier 1: Official releases
[[trust.sources]]
type = "signing_key"
public_key = "ed25519:JXZgHMq9cGk3+/Q7FX..."
signer = "releases@myorg.com"
description = "Official releases (instant approval)"

# Tier 2: Internal organization repos
[[trust.sources]]
type = "git"
url_pattern = "https://github.com/myorg/*"
branch = "main"
description = "Internal org repos (instant approval)"

# Tier 3: Public open-source (example - not auto-approved)
# These would be Unknown and require manual approval
```

**Workflow:**
1. Official releases → Instant installation (Tier 1)
2. Internal org repositories → Instant installation (Tier 2)
3. External components → Security review required (Unknown)

---

## Approval Workflow Configuration

### Approval Queue Setup

```toml
[trust.approval]
enabled = true
notification_email = "security-review@example.com"
default_timeout_days = 7          # Auto-reject after 7 days
require_multiple_approvers = true # Need 2+ approvals
```

### Approval Rules

```toml
[trust.approval.rules]
# Components > 5MB require architecture review
large_component_threshold_mb = 5
require_architecture_review = true

# Network-enabled components require additional review
requires_network = true
network_review_required = true

# External components require longer review
external_component_review_days = 14
```

---

## Trust Decision Flowchart

```
Component Installation Initiated
           ↓
Is DevMode Enabled?
    ├─ YES → TrustLevel::DevMode ⚠️ (with warnings)
    └─ NO → Continue
             ↓
Extract Component Source
(Git URL, file path, signature, etc.)
             ↓
Check Against Trust Registry
             ↓
        Source Matched?
    ├─ YES → TrustLevel::Trusted ✓
    │        Proceed to approval
    │
    └─ NO → TrustLevel::Unknown ⏳
             Require manual review
             ↓
        Manual Approval Decision
        ├─ APPROVED → Install component
        ├─ REJECTED → Block installation
        └─ PENDING → Hold in approval queue
```

---

## Managing Trust Decisions

### Approving Unknown Components

When a component from an unknown source is requested:

1. **Review Requested**: Email notification sent to security team
2. **Analyze Component.toml**: Review all declared capabilities
3. **Make Decision**:
   - ✓ **Approve**: Grant installation permission (one-time or persistent)
   - ✗ **Reject**: Block installation with explanation
   - ? **Request Info**: Ask component provider for clarification

### One-Time vs. Persistent Approval

```toml
# One-time approval: specific component version only
[trust.approvals]
"third-party/component:1.0.0" = "approved-2025-12-20"

# Persistent approval: component family
# (requires explicit decision to add to trust sources)
[[trust.sources]]
type = "git"
url_pattern = "https://github.com/third-party/component"
description = "Third-party component (approved 2025-12-20)"
```

---

## Trust Inheritance and Revocation

### Inheritance

Trust is inherited through component dependencies. If component A trusts component B, and B depends on C:

```
ComponentA (Trusted)
  └─ ComponentB (from ComponentA's trusted source)
      └─ ComponentC (evaluated independently)
```

ComponentC is evaluated against your trust configuration, not inherited through A or B.

### Revocation

To revoke trust for a previously trusted source:

1. **Remove from Configuration**: Delete the `[[trust.sources]]` entry
2. **Notify Teams**: Communicate trust revocation to development teams
3. **Update Deployed Components**: Decide whether to allow already-deployed instances

```toml
# REMOVED THIS SECTION:
# [[trust.sources]]
# type = "git"
# url_pattern = "https://github.com/old-org/*"

# Future components from this source will be Unknown
# Existing deployments are NOT automatically affected
```

---

## Best Practices

### ✅ Use Specific URL Patterns

Avoid overly broad patterns that might match unintended sources.

```toml
# ❌ WRONG: Too broad
url_pattern = "https://github.com/*"  # Trusts ALL GitHub!

# ✓ CORRECT: Specific organization
url_pattern = "https://github.com/myorg/*"  # Only your org
```

### ✅ Require Multiple Signers

Use signing keys for critical components, requiring multiple approvals.

```toml
[trust.approval]
require_multiple_approvers = true

[[trust.sources]]
type = "signing_key"
signer = "alice@example.com"
# ...

[[trust.sources]]
type = "signing_key"
signer = "bob@example.com"
# ...
```

### ✅ Monitor Trust Decisions

Maintain an audit log of all trust-related decisions.

```toml
[trust.audit]
log_file = "/var/log/component-trust-audit.log"
log_approvals = true
log_rejections = true
log_trusted_installations = true
```

### ✅ Review Trust Configuration Regularly

Trust configuration should be reviewed quarterly to remove obsolete entries.

```
Quarterly Review Checklist:
- [ ] Remove expired or unused trust sources
- [ ] Update signing key rotations
- [ ] Review approval workflow metrics
- [ ] Verify no overly-broad patterns remain
- [ ] Audit all trust-related changes
```

### ✅ Document Trust Decisions

Include comments explaining why sources are trusted.

```toml
# RATIONALE: These are internal org repos with code review
# requiring 2+ maintainer approvals before merge
[[trust.sources]]
type = "git"
url_pattern = "https://github.com/myorg/*"
description = "Internal org repos (code review enforced)"

# RATIONALE: Alice is principal architect with security
# clearance for component review and approval
[[trust.sources]]
type = "signing_key"
signer = "alice@example.com"
description = "Alice (principal architect)"
```

---

## Troubleshooting Trust Issues

### Component Still Requires Approval

**Problem**: Component from trusted source still waiting for approval.

**Solutions:**
1. Verify Git URL or path exactly matches `url_pattern`
2. Check branch name matches configured `branch`
3. For signing keys: verify signature is valid
4. Check DevMode is not globally disabled

---

### DevMode Warnings Not Appearing

**Problem**: DevMode enabled but no warnings shown.

**Solution**: Verify DevMode is enabled and warnings go to configured output:

```toml
[trust]
dev_mode = true

[trust.devmode_warnings]
output = "stderr"  # or "log_file"
timestamp = true
```

---

## References

- **Capability Declaration**: [capability-declaration-guide.md](capability-declaration-guide.md)
- **Security Architecture**: [security-architecture.md](security-architecture.md)
- **Best Practices**: [security-best-practices.md](security-best-practices.md)
- **Examples**: [examples/](examples/)
- **Troubleshooting**: [troubleshooting-security.md](troubleshooting-security.md)
