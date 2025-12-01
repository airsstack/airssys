# ADR-WASM-005: Capability-Based Security Model

**Status:** Accepted  
**Date:** 2025-10-19  
**Decision Makers:** Architecture Team  
**Related:** KNOWLEDGE-WASM-001 (Component Framework), KNOWLEDGE-WASM-004 (WIT Management), ADR-WASM-002 (Runtime Engine)

---

## Context

The airssys-wasm framework requires a security model that protects the host system from potentially malicious or buggy components while enabling legitimate functionality. Components may be written in multiple programming languages (Rust, JavaScript, Go, Python, etc.) and may require access to various system resources (filesystem, network, storage, etc.).

### The Problem

**Security Challenge:**
- Components run in WASM sandbox (memory isolated)
- BUT: Components need access to host resources (files, network, storage)
- Host must provide access through host functions
- **Risk:** Malicious components could abuse host functions to access unauthorized resources

**Example Threat Scenarios:**
```
Scenario 1: Filesystem Abuse
- Component declares: "I need to read config.toml"
- Reality: Component reads /etc/passwd, /home/user/.ssh/id_rsa
- Impact: Credential theft, data exfiltration

Scenario 2: Network Abuse
- Component declares: "I need to call api.example.com"
- Reality: Component connects to attacker C2 server, exfiltrates data
- Impact: Data breach, botnet participation

Scenario 3: Resource Exhaustion
- Component declares: "I need 100MB storage"
- Reality: Component writes 10GB, fills disk
- Impact: Denial of service, system instability
```

### Requirements

**Functional Requirements:**
- Fine-grained access control (specific files/paths, domains, storage namespaces)
- Deny-by-default principle (components have zero access unless granted)
- Declarative permission model (capabilities declared in manifest)
- Administrator review and approval workflow
- Runtime permission enforcement (checked at host function entry)
- Support for multiple programming languages (language-agnostic)

**Non-Functional Requirements:**
- Performance: Minimal overhead for permission checks (<1% of operation time)
- Usability: Not overly burdensome for developers (trusted sources auto-approved)
- Auditability: Comprehensive logging of all access attempts
- Integration: Work with existing airssys-osl security system (RBAC/ACL)
- Flexibility: Support both development and production workflows

**Developer Experience Requirements:**
- Trusted internal components install instantly (no approval delays)
- Unknown external components require review (security first)
- Development mode available for rapid iteration (bypass checks with warnings)
- Clear error messages when capabilities denied

---

## Decision

### Core Decision: Fine-Grained Capability-Based Security with Trust Levels

**We will implement a fine-grained capability-based security model with pattern matching for resources, combined with a trust-level system for installation workflows.**

### Key Design Principles

1. **Deny-by-Default**: Components start with zero permissions
2. **Least Privilege**: Grant only minimum permissions needed
3. **Explicit Declaration**: Capabilities declared in Component.toml manifest
4. **Trust-Based Approval**: Trusted sources auto-approved, unknown sources reviewed
5. **Runtime Enforcement**: Permissions checked at every host function call
6. **Layered Security**: Component capabilities + airssys-osl RBAC/ACL (defense in depth)
7. **Developer-Friendly**: Fast workflow for trusted sources, security for unknown sources

---

## Detailed Decisions

### Decision 1: Capability Granularity - Fine-Grained Pattern-Based

**Decision:** Use fine-grained capabilities with glob pattern matching.

**Capability Categories:**

#### 1.1 Filesystem Capabilities

**Pattern Syntax:**
```toml
[capabilities.filesystem]
read = [
    "/etc/myapp/config.toml",        # Specific file
    "/etc/myapp/*.toml",              # Glob: any .toml in directory
    "/var/data/myapp/**",             # Recursive: anything under path
]
write = [
    "/var/data/myapp/output/**",
    "/tmp/myapp/cache/*",
]
```

**Glob Pattern Rules:**
- `*` - Matches any characters except `/` (single directory level)
- `**` - Matches any characters including `/` (recursive, all subdirectories)
- `?` - Matches single character
- `[abc]` - Matches one of the characters

**Examples:**
```toml
"/etc/myapp/*.toml"           # Matches: /etc/myapp/config.toml
                               # Doesn't match: /etc/myapp/subdir/file.toml

"/var/data/myapp/**"          # Matches: /var/data/myapp/file.txt
                               # Matches: /var/data/myapp/a/b/c/file.txt

"/tmp/myapp/cache/session-?"  # Matches: /tmp/myapp/cache/session-1
                               # Matches: /tmp/myapp/cache/session-A
```

**Rationale:**
- **Security**: Only grants access to explicitly approved paths
- **Flexibility**: Patterns balance specificity with usability
- **Auditability**: Clear declaration of filesystem access requirements

#### 1.2 Network Capabilities

**Pattern Syntax:**
```toml
[capabilities.network]
outbound = [
    "api.example.com:443",            # Specific domain + port
    "*.cdn.example.com:443",          # Wildcard subdomain
    "192.168.1.100:8080",             # IP address + port
]
inbound = [
    "0.0.0.0:8080",                   # Listen on specific port
    "127.0.0.1:9000",                 # Localhost only
]
```

**Domain Pattern Rules:**
- Exact domain match: `api.example.com`
- Wildcard subdomain: `*.example.com` (matches `a.example.com`, `b.c.example.com`)
- Port specification: Required (`:443`, `:*` for any port)
- IP addresses: Supported (IPv4 and IPv6)

**Examples:**
```toml
"api.example.com:443"         # HTTPS to specific API
"*.cdn.example.com:443"       # Any CDN subdomain
"203.0.113.5:8080"            # Specific IP and port
"[2001:db8::1]:443"           # IPv6 address
```

**Rationale:**
- **Security**: Prevent components from calling arbitrary external services
- **Visibility**: Admin can see exactly which services component contacts
- **Compliance**: Enable network policy enforcement for regulatory requirements

#### 1.3 Storage Capabilities

**Pattern Syntax:**
```toml
[capabilities.storage]
namespaces = [
    "myapp:config",                   # Component-specific namespace
    "myapp:cache",
    "shared:public-data",             # Shared namespace (if approved)
]
max_size = "100MB"                    # Total storage quota
```

**Namespace Rules:**
- Format: `<prefix>:<name>`
- Component-isolated: `myapp:*` (only this component)
- Shared namespaces: `shared:*` (requires explicit approval)
- Quota enforcement: Host enforces total storage limits

**Rationale:**
- **Isolation**: Components cannot access each other's storage by default
- **Quota Management**: Prevent storage exhaustion attacks
- **Shared Data**: Enable controlled data sharing between components

---

### Decision 2: Permission Declaration Location - Component.toml Manifest

**Decision:** Capabilities declared in Component.toml manifest file.

**Manifest Format:**
```toml
[component]
name = "data-processor"
version = "1.0.0"
description = "Processes data files and uploads results"

[capabilities.filesystem]
read = [
    "/etc/myapp/config.toml",
    "/var/data/input/**",
]
write = [
    "/var/data/output/**",
    "/tmp/myapp/cache/*",
]

[capabilities.network]
outbound = [
    "api.example.com:443",
    "*.cdn.example.com:443",
]

[capabilities.storage]
namespaces = ["myapp:config", "myapp:cache"]
max_size = "100MB"

# Rationale for requested capabilities (documentation for reviewers)
[capabilities.rationale]
filesystem_read = "Read input data files and application configuration"
filesystem_write = "Write processed results and maintain temporary cache"
network_outbound = "Upload results to API and fetch dependencies from CDN"
storage = "Store configuration and cache processed data"
```

**Rationale:**
- **Visible Before Installation**: Admin can review before deploying
- **Versionable**: Capabilities tracked in version control (Git)
- **Reviewable**: Easy to audit in pull requests
- **Separate from Code**: Permissions not buried in implementation
- **Language-Agnostic**: Works for any WASM-compatible language

**Alternative Rejected: Code Attributes**
```rust
// Language-specific, not visible before installation
#[capability(filesystem.read = "/etc/myapp/*.toml")]
fn init() -> Result<()> { }
```
❌ Rejected: Doesn't work for JavaScript, Go, Python components

---

### Decision 3: Permission Grant Workflow - Trust Levels with Auto-Approval

**Decision:** Use trust-based approval system with auto-approval for trusted sources.

#### Workflow: Declare → Trust Check → Auto-Approve or Review → Enforce

**Step 1: Component Developer Declares Capabilities**
```toml
# Component.toml (in component repository)
[capabilities.filesystem]
read = ["/etc/myapp/config.toml"]
write = ["/var/data/myapp/**"]
```

**Step 2: Installation with Trust Level Check**
```bash
$ airssys-wasm install git@github.com:mycompany/my-component
```

**Step 3: Trust Level Determination**

Host checks component source against trust configuration:

```toml
# trust-config.toml (host-side configuration)

[[trusted-sources]]
type = "git-repository"
pattern = "git@github.com:mycompany/*"
description = "Internal company repositories"
auto_approve = true

[[trusted-sources]]
type = "signature"
public_key = "ed25519:AAAAC3NzaC1lZDI1NTE5..."
signer = "engineering@mycompany.com"
description = "Signed by engineering team"
auto_approve = true

[[trusted-sources]]
type = "local"
pattern = "file://./components/verified/*"
description = "Pre-verified local components"
auto_approve = true
```

**Step 4a: Trusted Source - Auto-Approve (Fast Path)**
```bash
📦 Installing: my-component v1.0.0
🔒 Source: git@github.com:mycompany/my-component
✅ Trusted source: mycompany (auto-approved)

Capabilities:
  ✓ Filesystem read:  /etc/myapp/config.toml
  ✓ Filesystem write: /var/data/myapp/**
  ✓ Network outbound: api.example.com:443

Installing... ████████████████████ 100%
✅ Installed successfully in 2.1s

No approval delay - productivity maintained!
```

**Step 4b: Unknown Source - Review Required (Security Path)**
```bash
📦 Installing: unknown-component v1.0.0
⚠️  Unknown source (not in trusted list)
🔒 Security review required

Requested Capabilities:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Filesystem:
  ✓ Read:  /etc/myapp/config.toml
  ⚠️ Write: /var/data/** (broad recursive write - review)
  
Network:
  ⚠️ Outbound: *.unknown-api.com:* (wildcard domain/port - review)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

⚠️  Security Concerns:
- Broad filesystem write access (entire /var/data tree)
- Wildcard network access (any subdomain, any port)

Options:
  1. Approve as-is (grant all requested capabilities)
  2. Modify capabilities (restrict permissions)
  3. Deny installation
  4. Add source to trusted list (auto-approve future)

Your choice [1/2/3/4]:
```

**Step 4c: Development Mode - Auto-Approve All (Dev Path)**
```bash
$ export AIRSSYS_WASM_DEV_MODE=true
$ airssys-wasm install ./my-local-component

📦 Installing: my-local-component (local)

⚠️  ⚠️  ⚠️  DEVELOPMENT MODE ACTIVE ⚠️  ⚠️  ⚠️
All security checks bypassed!

Capabilities (auto-approved):
  ⚠️ Filesystem: Unrestricted access
  ⚠️ Network: Unrestricted access
  ⚠️ Storage: Unlimited

Installing... ████████████████████ 100%
✅ Installed successfully in 0.7s

⚠️  Security Warning: Unrestricted access granted
    Only use for development/testing!
```

**Step 5: Runtime Enforcement**
```rust
// All host function calls checked against approved capabilities
component.read_file("/etc/myapp/config.toml")
// ✅ Allowed - matches approved pattern

component.read_file("/etc/passwd")
// ❌ Denied - doesn't match any approved pattern
```

**Rationale:**
- **Productivity**: Trusted sources install instantly (no delays)
- **Security**: Unknown sources require review (protection)
- **Flexibility**: Dev mode available for rapid development
- **Audit Trail**: All installations logged with trust level
- **Gradual Trust**: Can add sources to trusted list over time

---

### Decision 4: Runtime Enforcement Mechanism - Host Function Entry Checks

**Decision:** Check permissions at every host function entry point.

**Enforcement Architecture:**
```
Component calls host function
         ↓
Host function entry
         ↓
Check capability (pattern match)
         ↓
   Allowed? ────→ Execute operation
         ↓              ↓
       Denied      Audit log
         ↓              ↓
    Error returned   Return result
```

**Implementation:**
```rust
/// Host function: read_file
pub fn host_read_file(
    mut caller: Caller<'_, HostState>,
    path: String,
) -> Result<Vec<u8>, CapabilityError> {
    let state = caller.data();
    let component_id = &state.component_id;
    
    // === Enforcement Point: Check Capability ===
    state.capability_checker
        .check_filesystem_read(component_id, &path)
        .map_err(|e| {
            // Log denial
            error!(
                component_id = %component_id,
                path = %path,
                reason = %e,
                "Capability check failed"
            );
            
            // Audit log (security event)
            state.audit_logger.log_capability_denial(
                component_id,
                "filesystem.read",
                &path,
                &e.to_string(),
            );
            
            CapabilityError::Denied {
                component: component_id.clone(),
                operation: "filesystem.read",
                resource: path.clone(),
                reason: e.to_string(),
            }
        })?;
    
    // === Execute Operation (Only if allowed) ===
    let content = std::fs::read(&path)
        .map_err(|e| CapabilityError::IoError { 
            path: path.clone(), 
            error: e.to_string() 
        })?;
    
    // === Audit Log (Success) ===
    state.audit_logger.log_filesystem_access(
        component_id,
        "filesystem.read",
        &path,
        "allowed",
        content.len(),
    );
    
    Ok(content)
}
```

**Capability Checker Implementation:**
```rust
pub struct CapabilityChecker {
    // Component ID → Approved capabilities
    component_capabilities: Arc<RwLock<HashMap<ComponentId, ApprovedCapabilities>>>,
    
    // Pattern matching cache (performance optimization)
    pattern_cache: Arc<Mutex<LruCache<(ComponentId, String), bool>>>,
}

impl CapabilityChecker {
    pub fn check_filesystem_read(
        &self,
        component_id: &ComponentId,
        path: &str,
    ) -> Result<(), CapabilityError> {
        // 1. Check cache first (performance: ~50ns vs ~5μs)
        let cache_key = (component_id.clone(), path.to_string());
        if let Some(&allowed) = self.pattern_cache.lock().get(&cache_key) {
            return if allowed {
                Ok(())
            } else {
                Err(CapabilityError::Denied { /* cached denial */ })
            };
        }
        
        // 2. Get component's approved capabilities
        let capabilities = self.component_capabilities
            .read()
            .get(component_id)
            .ok_or(CapabilityError::ComponentNotFound {
                component: component_id.clone(),
            })?
            .clone();
        
        // 3. Check against approved read patterns
        let allowed = capabilities.filesystem.read_patterns
            .iter()
            .any(|pattern| pattern.matches(path));
        
        // 4. Cache result for future checks
        self.pattern_cache.lock().put(cache_key, allowed);
        
        if allowed {
            Ok(())
        } else {
            Err(CapabilityError::Denied {
                component: component_id.clone(),
                operation: "filesystem.read",
                resource: path.to_string(),
                reason: format!(
                    "Path '{}' does not match any approved patterns.\n\
                     Approved patterns: {:?}",
                    path, capabilities.filesystem.read_patterns
                ),
            })
        }
    }
}

/// Glob pattern with compiled matcher
#[derive(Debug, Clone)]
pub struct GlobPattern {
    pattern: String,
    matcher: glob::Pattern,
}

impl GlobPattern {
    pub fn new(pattern: &str) -> Result<Self, glob::PatternError> {
        Ok(Self {
            pattern: pattern.to_string(),
            matcher: glob::Pattern::new(pattern)?,
        })
    }
    
    pub fn matches(&self, path: &str) -> bool {
        self.matcher.matches(path)
    }
}
```

**Performance:**
```
Pattern check (uncached):  ~1-5 microseconds (μs)
Pattern check (cached):    ~0.05 microseconds (50 nanoseconds)
Actual file read:          ~1,000-5,000 microseconds (1-5 milliseconds)

Overhead: 0.1-0.5% of operation time (negligible)
```

**Rationale:**
- **Single Enforcement Point**: All checks at host function entry
- **Simple Implementation**: Clear, maintainable code
- **Clear Failure**: Component gets immediate error if denied
- **Comprehensive Audit**: Log all access attempts (allowed and denied)
- **Performance**: Pattern matching ~1-5μs, cached ~50ns

---

### Decision 5: Integration with airssys-osl Security - Layered Security

**Decision:** Implement layered security with component capabilities + airssys-osl RBAC/ACL.

**Security Layers:**
```
Layer 1: Component Capability Check (airssys-wasm)
         - Check against Component.toml approved patterns
         - Fast pattern matching (~1-5μs)
         - Component-specific permissions
         
         ↓ (If allowed)
         
Layer 2: airssys-osl Security Context (RBAC/ACL)
         - Check system-wide RBAC policies
         - Check ACL for resource access
         - User/role-based permissions
         
         ↓ (If allowed)
         
Layer 3: OS Permissions
         - Standard OS-level access control
         - File permissions, network firewall, etc.
         - Final safety net
```

**Implementation:**
```rust
pub fn host_read_file(
    mut caller: Caller<'_, HostState>,
    path: String,
) -> Result<Vec<u8>> {
    let state = caller.data();
    let component_id = &state.component_id;
    
    // ===== Layer 1: Component Capability Check =====
    state.capability_checker
        .check_filesystem_read(component_id, &path)?;
    
    // ===== Layer 2: airssys-osl Security Context =====
    
    // Create OSL security context for this component
    let security_context = SecurityContext {
        component_id: component_id.clone(),
        user: format!("component:{}", component_id),
        roles: state.component_osl_roles.clone(),
    };
    
    // Create OSL operation
    let operation = ReadFileOperation { 
        path: path.clone() 
    };
    
    // Execute through OSL middleware (RBAC/ACL checks)
    let content = state.osl_executor
        .execute(operation, security_context)
        .await?;
    
    // ===== Layer 3: OS Permissions =====
    // (Handled by OSL through secure system calls)
    
    // ===== Unified Audit Logging =====
    state.audit_logger.log(AuditEvent {
        timestamp: Utc::now(),
        component_id: component_id.clone(),
        user: security_context.user,
        operation: "filesystem.read",
        resource: path,
        result: "allowed",
        layers: vec!["component-capability", "osl-rbac", "os-permission"],
    });
    
    Ok(content)
}
```

**Component to OSL Role Mapping:**
```toml
# Host configuration: component-roles.toml

[component-roles]
# Map component IDs to OSL roles
"data-processor" = ["file-reader", "api-caller"]
"admin-tool" = ["admin", "file-writer"]
"monitoring-agent" = ["metrics-reader"]

# Default role for unmapped components
default_role = "guest"
```

**OSL RBAC Policy Example:**
```toml
# OSL RBAC policy (airssys-osl configuration)

[[roles]]
name = "file-reader"
permissions = [
    "filesystem:read:/var/data/**",
]

[[roles]]
name = "api-caller"
permissions = [
    "network:outbound:api.example.com:443",
]

[[roles]]
name = "admin"
permissions = [
    "filesystem:read:/etc/**",
    "filesystem:write:/etc/myapp/**",
    "network:outbound:*:*",
]
```

**Rationale:**
- **Defense in Depth**: Multiple security layers (if one fails, others protect)
- **Reuse OSL Infrastructure**: Leverage existing RBAC, ACL, audit logging
- **Unified Audit Trail**: All security events logged to OSL audit system
- **Consistent Policy**: Same security model across entire AirsSys ecosystem
- **Component-Specific + System-Wide**: Fine-grained component control + system policies

---

### Decision 6: Forbidden Capabilities - Always Deny List

**Decision:** Maintain list of capabilities that are always forbidden, regardless of trust level.

**Forbidden Patterns:**
```toml
# Host configuration: forbidden-capabilities.toml

[forbidden]
description = "Capabilities that are ALWAYS denied, even for trusted sources"

# Forbidden filesystem write patterns
filesystem_write = [
    "/etc/passwd",              # User database
    "/etc/shadow",              # Password hashes
    "/etc/sudoers",             # Sudo configuration
    "/boot/**",                 # Boot loader and kernel
    "/sys/**",                  # System kernel interface
    "/proc/**",                 # Process information
    "/dev/**",                  # Device files
]

# Forbidden filesystem read patterns (sensitive data)
filesystem_read = [
    "/etc/shadow",              # Password hashes
    "/root/.ssh/**",            # Root SSH keys
    "/home/*/.ssh/id_*",        # User SSH private keys
]

# Forbidden network patterns
network_outbound = [
    "*.internal.secret:*",      # Internal secrets API
    "localhost:22",             # SSH access
]

network_inbound = [
    "*:22",                     # SSH port
    "*:3306",                   # MySQL port
    "*:5432",                   # PostgreSQL port
]
```

**Enforcement:**
```rust
impl CapabilityChecker {
    pub fn check_filesystem_write(
        &self,
        component_id: &ComponentId,
        path: &str,
    ) -> Result<(), CapabilityError> {
        // 1. Check against forbidden list FIRST (before anything else)
        if self.is_forbidden_filesystem_write(path) {
            return Err(CapabilityError::ForbiddenCapability {
                component: component_id.clone(),
                operation: "filesystem.write",
                resource: path.to_string(),
                reason: format!(
                    "Path '{}' is in forbidden list. \
                     This capability cannot be granted under any circumstances.",
                    path
                ),
            });
        }
        
        // 2. Check component capabilities (if not forbidden)
        self.check_component_capability(component_id, path)?;
        
        Ok(())
    }
    
    fn is_forbidden_filesystem_write(&self, path: &str) -> bool {
        self.forbidden_patterns.filesystem_write
            .iter()
            .any(|pattern| pattern.matches(path))
    }
}
```

**Installation Behavior:**
```bash
$ airssys-wasm install malicious-component

📦 Analyzing: malicious-component v1.0.0

❌ FORBIDDEN CAPABILITY DETECTED
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Filesystem Write:
  ❌ /etc/passwd (FORBIDDEN - system critical)
  ❌ /etc/shadow (FORBIDDEN - password hashes)
  ❌ /boot/** (FORBIDDEN - system boot files)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Installation DENIED by security policy.

Forbidden capabilities cannot be granted under any circumstances,
even for trusted sources or in development mode.

Reason: Component requests write access to system-critical files
        that are always forbidden by host security policy.
```

**Rationale:**
- **Absolute Protection**: Some resources too critical to ever allow component access
- **Prevents Accidents**: Even trusted components can have bugs
- **Security Baseline**: Establishes minimum security floor
- **Clear Policy**: No exceptions, no negotiations

---

## Alternatives Considered

### Alternative 1: Coarse-Grained Capabilities

**Approach:**
```toml
[capabilities]
filesystem = true     # All filesystem access
network = true        # All network access
storage = true        # All storage access
```

**Evaluation:**

**Pros:**
- ✅ Simple to declare
- ✅ Easy to understand

**Cons:**
- ❌ **Too broad** - Violates least privilege principle
- ❌ **All-or-nothing** - Component gets everything or nothing
- ❌ **Cannot restrict** to specific paths/domains
- ❌ **Security risk** - Component can access far more than needed

**Example Problem:**
```
Component needs: Read /etc/myapp/config.toml
Grants: Full filesystem access (can read /etc/passwd, /home/*, etc.)
```

**Why Rejected:** Too coarse-grained, violates security principles, doesn't enable least privilege.

---

### Alternative 2: Capability-Based (Unforgeable Object References)

**Approach:**
```rust
// Inspired by E language capability-based security
// Host creates unforgeable capability token
let file_cap = host.grant_file_read_capability("/etc/config.toml");

// Component receives opaque capability token
component.call("init", file_cap)?;

// Component must use token to access resource
let content = component.read_file(file_cap)?;
```

**Evaluation:**

**Pros:**
- ✅ Theoretically elegant (academic security model)
- ✅ Unforgeable tokens (can't fake capabilities)
- ✅ Can be passed between components
- ✅ Dynamic revocation possible

**Cons:**
- ❌ **Complex implementation** - Token management, lifecycle, revocation
- ❌ **Language-specific** - Hard to express in WIT (cross-language)
- ❌ **Runtime overhead** - Token validation on every access
- ❌ **Developer confusion** - Unfamiliar paradigm
- ❌ **Overkill** - More complex than problem requires

**Why Rejected:** Too complex for our use case, hard to express in language-agnostic WIT interfaces, unfamiliar to most developers.

---

### Alternative 3: Automatic Permission Grant (No Review)

**Approach:**
```
1. Component declares capabilities in manifest
2. Host automatically grants everything
3. No review, no approval required
```

**Evaluation:**

**Pros:**
- ✅ Zero friction (instant installation)
- ✅ No developer delays

**Cons:**
- ❌ **Security nightmare** - Malicious components get full access
- ❌ **No visibility** - Admin doesn't know what's granted
- ❌ **No audit** - No record of permission grants
- ❌ **Violates least privilege** - No opportunity to restrict

**Example Threat:**
```
Malicious component declares:
[capabilities.filesystem]
write = ["/**"]  # Full filesystem write!

Host auto-grants → Component can destroy entire system
```

**Why Rejected:** Completely unacceptable from security perspective, enables trivial attacks.

---

### Alternative 4: Runtime Permission Prompts

**Approach:**
```
1. Component starts with zero permissions
2. Component requests permission at runtime when needed
3. Host prompts admin for approval during execution
4. Grant or deny dynamically
```

**Evaluation:**

**Pros:**
- ✅ Lazy permission granting (only when actually needed)
- ✅ Can deny if unexpected behavior

**Cons:**
- ❌ **Interrupts runtime** - Can't prompt admin during execution
- ❌ **Complex state** - Partial permissions over time
- ❌ **Unpredictable** - Component might fail mid-execution
- ❌ **Poor UX** - Constant interruptions for approvals
- ❌ **Not serverless-friendly** - No admin available at runtime

**Example Problem:**
```
Component running automated data processing job at 3 AM
Needs new permission → Prompts admin → Admin asleep → Job fails
```

**Why Rejected:** Interrupts execution, unpredictable, poor user experience, not suitable for automated/serverless environments.

---

### Alternative 5: Code Annotations/Attributes

**Approach:**
```rust
// Rust example
#[capability(filesystem.read = "/etc/myapp/*.toml")]
#[capability(network.outbound = "api.example.com:443")]
fn process_data() -> Result<()> {
    // Implementation
}
```

**Evaluation:**

**Pros:**
- ✅ Co-located with code (close to usage)
- ✅ Type-safe in Rust

**Cons:**
- ❌ **Language-specific** - Doesn't work for JavaScript, Go, Python, etc.
- ❌ **Not visible before installation** - Need to inspect code
- ❌ **Hard to extract** - Requires code parsing/analysis
- ❌ **Fragmented** - Permissions scattered across codebase

**Why Rejected:** airssys-wasm is language-agnostic framework, this approach only works for specific languages.

---

## Consequences

### Positive Consequences

✅ **Strong Security Foundation**
- **Deny-by-default**: Components start with zero permissions
- **Least privilege**: Only grant minimum permissions needed
- **Explicit declaration**: Clear visibility into component requirements
- **Runtime enforcement**: Permissions checked at every access
- **Audit trail**: Comprehensive logging of all access attempts

✅ **Developer Productivity Maintained**
- **Trusted sources**: Auto-approval for internal/signed components (instant install)
- **Development mode**: Bypass checks for rapid development iteration
- **Clear errors**: Helpful error messages when capabilities denied
- **No bureaucracy**: No approval delays for trusted sources

✅ **Defense in Depth**
- **Layered security**: Component capabilities + OSL RBAC + OS permissions
- **Multiple checkpoints**: If one layer fails, others protect
- **Unified audit**: All security events logged to OSL system
- **Consistent policy**: Same security model across AirsSys ecosystem

✅ **Fine-Grained Control**
- **Pattern matching**: Balance between specific and flexible
- **Path-based**: Restrict filesystem to specific directories/files
- **Domain-based**: Restrict network to specific services
- **Namespace-based**: Isolate component storage

✅ **Administrator Visibility**
- **Review workflow**: Admin sees exactly what component wants
- **Modification**: Can restrict permissions at install time
- **Trust management**: Can add sources to trusted list
- **Audit logs**: Complete visibility into all access attempts

✅ **Performance Optimized**
- **Minimal overhead**: Pattern matching ~1-5μs (~0.1-0.5% of operation)
- **Caching**: Subsequent checks ~50ns (100x faster)
- **Negligible impact**: Users don't notice performance difference
- **Scalable**: Efficient even with many components

✅ **Language-Agnostic**
- **Manifest-based**: Component.toml works for any language
- **WIT-compatible**: Can be expressed in WebAssembly Interface Types
- **Cross-language**: Rust, JavaScript, Go, Python all supported
- **Standard format**: TOML is widely understood

### Negative Consequences

⚠️ **Trust Configuration Required**
- **Issue**: Host must configure trusted sources initially
- **Impact**: Setup overhead before first component install
- **Mitigation**: 
  - Provide default trust configuration templates
  - CLI wizard for trust setup (`airssys-wasm setup-trust`)
  - Documentation with common trust patterns
  - Can start with empty trust list (review everything)

⚠️ **Pattern Matching Complexity**
- **Issue**: Developers must understand glob pattern syntax
- **Impact**: Learning curve for capability declarations
- **Mitigation:**
  - Clear documentation with examples
  - Pattern validation in CLI (`airssys-wasm validate-manifest`)
  - Helpful error messages for invalid patterns
  - Component templates with common patterns

⚠️ **Two-Layer Security Overhead**
- **Issue**: Both component capability and OSL checks add latency
- **Impact**: ~5-10μs total overhead per host function call
- **Mitigation:**
  - Overhead is negligible compared to I/O (0.1-0.5%)
  - Can optimize with caching (reduces to ~100ns)
  - Security benefit outweighs minimal cost
  - Acceptable trade-off for defense in depth

⚠️ **Unknown Source Review Friction**
- **Issue**: Installing unknown components requires manual review
- **Impact**: Slows down installation of external components
- **Mitigation:**
  - Expected behavior (security first)
  - Can add sources to trusted list after first review
  - Development mode available for testing unknown components
  - Review is security feature, not bug

⚠️ **Forbidden List Maintenance**
- **Issue**: Host must maintain list of forbidden patterns
- **Impact**: Maintenance overhead, may need updates
- **Mitigation:**
  - Provide sensible defaults (system-critical files/ports)
  - Documented rationale for each forbidden pattern
  - Can be customized per deployment environment
  - Review and update during security audits

⚠️ **Manifest Synchronization**
- **Issue**: Component.toml must stay in sync with actual code usage
- **Impact**: Out-of-sync manifest causes runtime errors
- **Mitigation:**
  - CLI validation tools (`airssys-wasm check-manifest`)
  - Runtime error messages indicate missing capabilities
  - Development mode helps identify required capabilities
  - Testing catches manifest issues early

### Neutral Consequences

📝 **Additional Configuration Files**
- Component.toml in every component (capability declarations)
- trust-config.toml on host (trust level definitions)
- forbidden-capabilities.toml on host (always-deny list)
- Trade-off: More files, but better organization and clarity

📝 **Learning Curve**
- Developers must learn glob pattern syntax
- Administrators must understand trust model
- Investment: Initial learning required
- Payoff: Better security understanding, proper permission design

📝 **Audit Log Volume**
- All host function calls logged (allowed and denied)
- Impact: More storage for audit logs
- Trade-off: Comprehensive audit trail vs. storage cost
- Acceptable: Security visibility worth storage cost

---

## Implementation Guidance

### Phase 1: Core Capability System (Weeks 1-3)

**Week 1: Data Structures and Parsing**
```rust
// src/security/capabilities/mod.rs
pub mod manifest;          // Component.toml parsing
pub mod patterns;          // Glob pattern matching
pub mod checker;           // Capability checking logic
pub mod trust;             // Trust level configuration
pub mod audit;             // Audit logging

// Data structures
pub struct ApprovedCapabilities {
    pub filesystem: FilesystemCapabilities,
    pub network: NetworkCapabilities,
    pub storage: StorageCapabilities,
}

pub struct FilesystemCapabilities {
    pub read_patterns: Vec<GlobPattern>,
    pub write_patterns: Vec<GlobPattern>,
}

pub struct NetworkCapabilities {
    pub outbound_patterns: Vec<NetworkPattern>,
    pub inbound_patterns: Vec<NetworkPattern>,
}

pub struct StorageCapabilities {
    pub namespaces: Vec<String>,
    pub max_size: u64,
}
```

**Week 2: Capability Checker Implementation**
```rust
// Core capability checking logic
pub struct CapabilityChecker {
    component_capabilities: Arc<RwLock<HashMap<ComponentId, ApprovedCapabilities>>>,
    forbidden_patterns: ForbiddenPatterns,
    pattern_cache: Arc<Mutex<LruCache<(ComponentId, String), bool>>>,
}

impl CapabilityChecker {
    pub fn check_filesystem_read(&self, component_id: &ComponentId, path: &str) -> Result<()>;
    pub fn check_filesystem_write(&self, component_id: &ComponentId, path: &str) -> Result<()>;
    pub fn check_network_outbound(&self, component_id: &ComponentId, endpoint: &str) -> Result<()>;
    pub fn check_network_inbound(&self, component_id: &ComponentId, endpoint: &str) -> Result<()>;
    pub fn check_storage_namespace(&self, component_id: &ComponentId, namespace: &str) -> Result<()>;
}
```

**Week 3: Integration with Host Functions**
```rust
// Integrate capability checks into all host functions
pub fn host_read_file(caller: Caller<'_, HostState>, path: String) -> Result<Vec<u8>> {
    // 1. Check capability
    caller.data().capability_checker.check_filesystem_read(&component_id, &path)?;
    
    // 2. Execute operation
    let content = std::fs::read(&path)?;
    
    // 3. Audit log
    caller.data().audit_logger.log_access(&component_id, "filesystem.read", &path);
    
    Ok(content)
}
```

### Phase 2: Trust System and Installation (Weeks 4-5)

**Week 4: Trust Configuration**
```rust
// src/security/trust/mod.rs
pub struct TrustConfig {
    pub trusted_sources: Vec<TrustedSource>,
    pub default_mode: TrustMode,
    pub development_mode: bool,
}

pub enum TrustedSource {
    GitRepository { pattern: String, auto_approve: bool },
    Signature { public_key: String, signer: String, auto_approve: bool },
    Local { pattern: String, auto_approve: bool },
}

pub enum TrustMode {
    ReviewRequired,
    Deny,
    Allow,  // Not recommended for production
}

impl TrustConfig {
    pub fn is_trusted(&self, source: &ComponentSource) -> bool;
    pub fn should_auto_approve(&self, source: &ComponentSource) -> bool;
}
```

**Week 5: Installation Workflow**
```rust
// src/installation/mod.rs
pub struct ComponentInstaller {
    trust_config: Arc<TrustConfig>,
    capability_validator: Arc<CapabilityValidator>,
}

impl ComponentInstaller {
    pub async fn install(&self, source: ComponentSource) -> Result<ComponentId> {
        // 1. Parse Component.toml
        let manifest = self.parse_manifest(&source).await?;
        
        // 2. Check trust level
        let trust_level = self.trust_config.check_trust(&source);
        
        // 3. Review workflow
        match trust_level {
            TrustLevel::Trusted => self.auto_approve_install(manifest).await?,
            TrustLevel::Unknown => self.review_and_approve(manifest).await?,
            TrustLevel::Development => self.dev_mode_install(manifest).await?,
        }
        
        // 4. Load and instantiate component
        let component_id = self.load_component(manifest).await?;
        
        Ok(component_id)
    }
}
```

### Phase 3: OSL Integration (Week 6)

**OSL Security Layer Integration**
```rust
// src/security/osl_integration/mod.rs
pub struct OslSecurityLayer {
    osl_executor: Arc<OslExecutor>,
    component_role_mapping: Arc<RwLock<HashMap<ComponentId, Vec<String>>>>,
}

impl OslSecurityLayer {
    pub async fn execute_with_osl_check<T>(
        &self,
        component_id: &ComponentId,
        operation: impl Operation<Output = T>,
    ) -> Result<T> {
        // Create OSL security context
        let context = SecurityContext {
            component_id: component_id.clone(),
            user: format!("component:{}", component_id),
            roles: self.get_component_roles(component_id)?,
        };
        
        // Execute through OSL middleware (RBAC/ACL checks)
        self.osl_executor.execute(operation, context).await
    }
}
```

### Phase 4: CLI Tools (Week 7)

**Developer Tools**
```bash
# Validate component manifest
$ airssys-wasm validate-manifest ./Component.toml
✅ Manifest is valid
   Capabilities: filesystem.read (2 patterns), network.outbound (1 pattern)

# Check capabilities needed by code
$ airssys-wasm analyze-capabilities ./src
⚠️  Code uses capabilities not in manifest:
   - filesystem.write: /var/log/myapp/debug.log
   
Suggested addition to Component.toml:
[capabilities.filesystem]
write = ["/var/log/myapp/debug.log"]

# Test component with different capability sets
$ airssys-wasm test --restrict-capabilities
Testing with minimal capabilities...
❌ Test failed: Capability denied (filesystem.write: /tmp/cache/data.bin)

# Setup trust configuration wizard
$ airssys-wasm setup-trust
Setting up trust configuration...
? Do you want to trust internal repositories? Yes
? Repository pattern: git@github.com:mycompany/*
✅ Added trusted source
```

### Testing Strategy

**Unit Tests:**
- Glob pattern matching (edge cases, wildcards, recursion)
- Capability checking logic (allow/deny decisions)
- Trust level determination (trusted/unknown source classification)
- Audit logging (correct event capture)

**Integration Tests:**
- Complete installation workflow (trusted, unknown, dev mode)
- Host function capability enforcement (all host functions)
- OSL integration (layered security checks)
- Component lifecycle with capabilities (load, execute, unload)

**Security Tests:**
- Forbidden capabilities enforcement (cannot bypass)
- Pattern matching edge cases (directory traversal, symlinks)
- Capability escalation prevention (component can't grant itself permissions)
- Audit log integrity (all events captured, no gaps)

**Performance Tests:**
- Pattern matching overhead (measure μs per check)
- Cache effectiveness (hit rate, performance gain)
- Large-scale component loading (1000+ components)
- Concurrent capability checks (thread safety)

---

## Future Enhancements

### Phase 2: Custom Capabilities (Future)

**Deferred Decision:** Custom capabilities (AI, database, crypto) will be added in future phases.

**Rationale for Deferral:**
- **Focus**: Build solid foundation with core capabilities first
- **YAGNI**: Don't add complexity until proven need
- **Learning**: Understand real-world usage patterns before extending

**Knowledge Document:** KNOWLEDGE-WASM-012 will document custom capability architecture for future reference.

**Potential Custom Capabilities:**
```toml
# Example: AI/ML capabilities (future)
[capabilities.custom.ai]
allow_model_loading = true
max_model_size = "5GB"
gpu_access = true
max_gpu_memory = "4GB"

# Example: Database capabilities (future)
[capabilities.custom.database]
allow_connection = true
allowed_hosts = ["db.internal.company.com"]
max_connections = 10

# Example: Cryptography capabilities (future)
[capabilities.custom.crypto]
allow_signing = true
signing_algorithms = ["ed25519", "rsa-2048"]
key_storage = "component-namespace"
```

### Phase 3: Advanced Trust Features

**Multi-Signature Support:**
```toml
[[trusted-sources]]
type = "multi-signature"
required_signers = [
    "engineering@mycompany.com",
    "security@mycompany.com",
]
threshold = 2  # Require 2 of 2 signatures
```

**Time-Based Capabilities:**
```toml
[capabilities.scheduling]
allowed_hours = "09:00-17:00"
allowed_days = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday"]
timezone = "America/New_York"
```

**Cost-Based Limits:**
```toml
[capabilities.external-services]
allowed_services = ["stripe", "aws-s3"]
max_cost_per_day = "$10.00"
```

---

## References

### Related ADRs
- **ADR-WASM-001**: Multicodec Compatibility Strategy
- **ADR-WASM-002**: WASM Runtime Engine Selection
- **ADR-WASM-007** (Planned): Storage Backend Selection

### Related Knowledge
- **KNOWLEDGE-WASM-001**: Component Framework Architecture
- **KNOWLEDGE-WASM-004**: WIT Management Architecture
- **KNOWLEDGE-WASM-007**: Component Storage Architecture
- **KNOWLEDGE-WASM-012** (Future): Custom Capabilities Architecture

### Security References
- **OWASP WASM Security**: https://owasp.org/www-community/vulnerabilities/WebAssembly_Security
- **Capability-Based Security**: https://en.wikipedia.org/wiki/Capability-based_security
- **Principle of Least Privilege**: https://en.wikipedia.org/wiki/Principle_of_least_privilege
- **Defense in Depth**: https://en.wikipedia.org/wiki/Defense_in_depth_(computing)

### Implementation References
- **glob crate**: https://docs.rs/glob/ (pattern matching)
- **cap-std crate**: https://docs.rs/cap-std/ (capability-based filesystem)
- **airssys-osl**: RBAC and ACL implementation

---

## Decision Log

| Date | Decision | Participants |
|------|----------|--------------|
| 2025-10-19 | Capability granularity: Fine-grained patterns | Architecture Team |
| 2025-10-19 | Declaration: Component.toml manifest | Architecture Team |
| 2025-10-19 | Workflow: Trust levels with auto-approval | Architecture Team |
| 2025-10-19 | Enforcement: Host function entry checks | Architecture Team |
| 2025-10-19 | Integration: Layered security with OSL | Architecture Team |
| 2025-10-19 | Custom capabilities: Defer to Phase 2+ | Architecture Team |

---

**Status:** ✅ **Accepted**  
**Implementation Priority:** Critical (Phase 1 Foundation)  
**Next Review:** After Phase 1 implementation or if security incidents identified

---

## Appendix: Complete Example Workflows

### Example 1: Trusted Internal Component (Fast Path)

**Component Repository:**
```toml
# git@github.com:mycompany/data-processor/Component.toml

[component]
name = "data-processor"
version = "1.0.0"

[capabilities.filesystem]
read = ["/var/data/input/**"]
write = ["/var/data/output/**"]

[capabilities.network]
outbound = ["api.example.com:443"]
```

**Installation:**
```bash
$ airssys-wasm install git@github.com:mycompany/data-processor

📦 Installing: data-processor v1.0.0
✅ Trusted source: mycompany
✅ Installed successfully in 2.1s
```

**Runtime:**
```rust
// Component calls host function
component.read_file("/var/data/input/file.txt")
// ✅ Allowed - matches pattern

component.write_file("/var/data/output/result.json")
// ✅ Allowed - matches pattern

component.read_file("/etc/passwd")
// ❌ Denied - doesn't match any pattern
```

### Example 2: Unknown Component with Review

**Installation:**
```bash
$ airssys-wasm install https://github.com/external/suspicious-tool

⚠️  Unknown source - review required

Requested capabilities:
  ⚠️ filesystem.write: /var/data/**
  ⚠️ network.outbound: *.external-api.com:*

[Approve/Modify/Deny]: Modify

Modified to:
  ✓ filesystem.write: /var/data/external-tool/**
  ✓ network.outbound: api.external-api.com:443

✅ Installed with modified capabilities
```

### Example 3: Development Mode

```bash
$ AIRSSYS_WASM_DEV_MODE=true airssys-wasm install ./my-component

⚠️ DEV MODE: Auto-approving all capabilities
✅ Installed successfully (unrestricted access)
```
