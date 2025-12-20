# Example 2: Unknown Component with Approval Workflow

## Scenario

You want to use a **third-party logging component** from a public registry. You:
- Don't have a pre-established trust relationship with the vendor
- Need security team approval before deploying
- Want an audit trail of approval decisions
- Plan to use it in production

## Component Configuration

**Component.toml (from third-party vendor):**
```toml
[package]
name = "advanced-logging"
version = "2.5.0"
description = "Enterprise logging with remote sinks"
author = "LogVendor Inc"
repository = "https://github.com/logvendor/advanced-logging"

[component]
name = "advanced-logging"
version = "2.5.0"

# No trust configuration (from unknown source)

# Filesystem capabilities: write logs
[[component.capabilities.filesystem]]
paths = ["/var/log/myapp/*.log"]
permissions = ["write"]

# Network capabilities: send to log aggregator
[[component.capabilities.network]]
endpoints = [
    "https://logs.logvendor.com/ingest",
    "https://logs.logvendor.com/health"
]
permissions = ["connect"]

# Storage for buffering if network unavailable
[[component.capabilities.storage]]
namespaces = ["log-buffer/*"]
permissions = ["read", "write"]
quota_gb = 1
```

## Trust Determination

When your organization requests this component:

```
Installation Request: advanced-logging:2.5.0
    ↓
Extract Source Information:
  - Repository: https://github.com/logvendor/advanced-logging
  - Package: from public registry
    ↓
Check Trust Registry:
  - url_pattern = "https://github.com/myorg/*" ✗ (doesn't match)
  - signing_key = "..." ✗ (not signed)
  - local_path = "..." ✗ (not local)
    ↓
Result: TrustLevel::Unknown
    ↓
Trigger Approval Workflow
```

## Approval Configuration

In your trust configuration (`trust-config.toml`):

```toml
[trust]
dev_mode = false

# Trusted sources (for your internal components)
[[trust.sources]]
type = "git"
url_pattern = "https://github.com/myorg/*"
description = "Internal organization repositories"

# Approval workflow for unknown components
[trust.approval]
enabled = true
notification_email = "security-review@myorg.com"
approval_timeout_days = 7
require_approvals = 1

# Email notification template
[trust.approval.notifications]
subject = "Component Approval Request: {component_name}:{version}"
body = """
A new component requires security review:

Name: {component_name}
Version: {version}
Source: {source}

Capabilities:
{capabilities_summary}

Please review and approve or reject:
https://approval.myorg.com/requests/{request_id}
"""
```

## Approval Workflow

### Step 1: Review Request Received

Security team receives email:

```
Subject: Component Approval Request: advanced-logging:2.5.0

A new component requires security review:

Name: advanced-logging
Version: 2.5.0
Source: https://github.com/logvendor/advanced-logging

Capabilities:
- Filesystem: /var/log/myapp/*.log [write]
- Network: https://logs.logvendor.com/* [connect]
- Storage: log-buffer/* (1GB quota) [read, write]

Approve: https://approval.myorg.com/requests/req-12345
```

### Step 2: Security Review

Security team analyzes:

```markdown
## Review Checklist

✓ **Vendor Analysis:**
  - LogVendor is established logging vendor (10+ years)
  - Active GitHub repository with regular updates
  - 2,000+ GitHub stars (community trust indicator)

✓ **Capability Analysis:**
  - Filesystem: Limited to myapp logs (appropriate)
  - Network: Only to official LogVendor endpoints
  - Storage: 1GB buffer quota (reasonable)

? **Questions:**
  - Why connect to 2 endpoints? Health check vs ingest?
  - Is 1GB buffer sufficient for offline scenarios?

✓ **Security Assessment:**
  - Capabilities are reasonable for logging component
  - No suspicious network endpoints
  - No unexpected filesystem access
  - Quota limits are appropriate

✓ **Recommendation:** APPROVE

## Approval Details

- Approver: security-team@myorg.com
- Timestamp: 2025-12-20T15:30:00Z
- Valid until: 2026-12-20 (1 year)
- Notes: Enterprise vendor, appropriate permissions
```

### Step 3: Approval Granted

```
Approval Granted
    ↓
Component can now be deployed
    ↓
Audit log records approval:
2025-12-20T15:30:00Z | advanced-logging | APPROVED | security-team | 1-year | Enterprise vendor
```

### Step 4: Component Installation

```
Installation Proceeds:
    ↓
WasmSecurityContext created:
  {
    component_id: "advanced-logging",
    capabilities: [filesystem, network, storage],
    trust_level: Unknown (but approved),
    approval_expires: 2026-12-20,
    audit_logger: active,
  }
    ↓
Component loaded and running
```

## Trust Decision Flowchart

```
Component Installation Requested
           ↓
Check against trust registry
    ├─ Matches trusted source → Trusted ✓
    └─ No match → Unknown ⏳
                  ↓
         DevMode enabled?
         ├─ YES → DevMode ⚠️
         └─ NO → Proceed to approval
                  ↓
         Create approval request
                  ↓
         Send notification to team
                  ↓
         Wait for decision
         ├─ APPROVED → Continue
         │            Audit log approval
         │            Install component
         │
         ├─ REJECTED → Block
         │             Audit log rejection
         │             Notify requestor
         │
         └─ EXPIRED (7 days) → Auto-reject
                                 Notify requestor
```

## Runtime Behavior

Once approved and installed, the component operates with its declared capabilities:

### Successful Operations

```
1. Component writes log entry
   - Path: /var/log/myapp/app.log
   - Host function: check_capability(component, path, "write")
   - Pattern match: /var/log/myapp/*.log ✓
   - Permission: "write" in ["write"] ✓
   - Result: GRANTED
   - Audit: 2025-12-20T15:35:00Z | advanced-logging | /var/log/myapp/app.log | write | granted

2. Component sends log to aggregator
   - Endpoint: https://logs.logvendor.com/ingest
   - Host function: check_capability(component, endpoint, "connect")
   - Pattern match: https://logs.logvendor.com/* ✓
   - Permission: "connect" in ["connect"] ✓
   - Result: GRANTED
   - Audit: 2025-12-20T15:35:01Z | advanced-logging | https://logs.logvendor.com/ingest | connect | granted

3. Component buffers to storage
   - Storage: log-buffer/2025-12-20/batch-001
   - Quota: Using 150MB of 1GB limit
   - Result: GRANTED
   - Audit: 2025-12-20T15:35:02Z | advanced-logging | log-buffer/2025-12-20/batch-001 | write | granted (usage: 150MB/1GB)
```

### Denied Operations

```
1. Component attempts unauthorized write
   - Path: /etc/passwd
   - Host function: check_capability(component, "/etc/passwd", "write")
   - Pattern match: /var/log/myapp/*.log does NOT match /etc/passwd ✗
   - Result: DENIED
   - Audit: 2025-12-20T15:35:03Z | advanced-logging | /etc/passwd | write | denied | Pattern mismatch

2. Component attempts unauthorized endpoint
   - Endpoint: https://data-exfil.attacker.com/steal
   - Host function: check_capability(component, endpoint, "connect")
   - Pattern match: https://logs.logvendor.com/* does NOT match attacker endpoint ✗
   - Result: DENIED
   - Audit: 2025-12-20T15:35:04Z | advanced-logging | https://data-exfil.attacker.com/steal | connect | denied | Endpoint not in whitelist

3. Component exceeds storage quota
   - Current usage: 950MB
   - Request: Write 100MB
   - Would exceed: 1050MB > 1000MB ✗
   - Result: DENIED
   - Audit: 2025-12-20T15:35:05Z | advanced-logging | log-buffer/* | write | denied | Quota exceeded
```

## Approval Re-evaluation

### Quarterly Review

```
Quarterly Check (2025-12-20 quarterly review):
  - Approval valid? Yes (expires 2026-12-20)
  - Component behavior normal? Yes
  - Any security incidents? No
  - Should we continue trusting? Yes
  - Action: Keep approved
```

### Re-approval Process

If approval expires or needs renewal:

```
Approval Expiration: 2026-12-20
    ↓
Component requests to stay installed
    ↓
Security team performs 2nd review:
  - Any incidents in past year?
  - Has vendor released updates? (verify they update regularly)
  - Still appropriate permissions?
    ↓
Decision: Re-approve for 1 more year
    ↓
Updated approval: Valid until 2027-12-20
```

## Rejection Scenario

If approval request is rejected:

```
Approval Rejection: Vendor changed business model
    ↓
Security team analysis:
  - Vendor now requires customer data sharing
  - Component code is now closed-source
  - Unacceptable terms of service
    ↓
Approval Result: REJECTED
    ↓
Notification sent to requestor:
  "Component advanced-logging:2.5.0 REJECTED

  Reason: Vendor business model change
  Details: New ToS requires customer data sharing
  Alternative: Evaluate internal-logging:1.0.0

  Approved by: security-team@myorg.com
  Timestamp: 2025-12-20T16:00:00Z"
    ↓
Component installation blocked
    ↓
Audit log: 2025-12-20T16:00:00Z | advanced-logging | REJECTED | security-team | Business model concerns
```

## Audit Trail Summary

All approval activities are permanently logged:

```
2025-12-20T15:00:00Z | advanced-logging | APPROVAL_REQUESTED | deployment-team | Component installation requested
2025-12-20T15:30:00Z | advanced-logging | APPROVED | security-team | 1-year validity | Enterprise vendor, appropriate permissions
2025-12-20T15:35:00Z | advanced-logging | INSTALLED | automation | Component installed to production
2025-12-20T15:35:00Z | advanced-logging | CAPABILITY_CHECK | /var/log/myapp/app.log | write | granted
2025-12-20T16:00:00Z | advanced-logging | CAPABILITY_CHECK | https://logs.logvendor.com/ingest | connect | granted
```

## Next Steps

- Review [trust-configuration-guide.md](../trust-configuration-guide.md) for approval workflow setup
- See [example-1-trusted-filesystem.md](example-1-trusted-filesystem.md) for trusted component workflow
- Read [troubleshooting-security.md](../troubleshooting-security.md) for approval issues
