# WASM-TASK-005 Phase 2 Task 2.1: Trust Level Implementation - IMPLEMENTATION PLAN

**Task:** Trust Level Implementation  
**Status:** 📋 PLANNED (Ready to Start)  
**Date Created:** 2025-12-17  
**Estimated Duration:** 2-3 days (15 hours)  
**Prerequisites:** ✅ Phase 1 complete (Tasks 1.1-1.3)

---

## Executive Summary

**What**: Implement a WASM-specific trust-level system that classifies component sources as **Trusted** (instant approval), **Unknown** (requires review), or **DevMode** (bypass security for development) based on source verification.

**Why**: Components from different sources pose different security risks. Trusted internal components should install instantly for developer productivity, while unknown third-party components need security review. Development mode enables rapid local iteration without security friction, while maintaining awareness through warnings.

**How**: Create a `TrustLevel` enum, `TrustSource` registry for trusted Git repositories and signing keys, trust determination logic that checks component sources against the registry, and a configuration file format for managing trusted sources. The system integrates with the approval workflow (Task 2.2) to determine whether components need manual review.

**Architecture Position**: This module sits between component installation and approval workflow, determining the security posture for each component before capability enforcement.

---

## Implementation Strategy

### Core Design Principles

1. **Security First**: Default to Unknown (review required) unless explicitly trusted
2. **Developer Friendly**: Trusted sources bypass approval for productivity
3. **Explicit Configuration**: Trust must be explicitly configured (no auto-trust)
4. **Revocable Trust**: Trust sources can be added/removed without code changes
5. **Audit Trail**: All trust decisions logged for security forensics

### Trust Level Workflow

```text
Component Installation Initiated
          ↓
Extract Component Source (Git URL, file path, etc.)
          ↓
Check Against Trust Registry
          ↓
     ┌────┴────┐
     │ Trusted? │
     └────┬────┘
          ├─ YES → TrustLevel::Trusted
          │        ↓
          │   Auto-approve (Task 2.2)
          │        ↓
          │   Instant install ✅
          │
          ├─ NO → TrustLevel::Unknown
          │       ↓
          │   Manual review (Task 2.2)
          │       ↓
          │   Approval queue ⏳
          │
          └─ DEV MODE → TrustLevel::DevMode
                  ↓
             Bypass security with warnings ⚠️
                  ↓
             Unrestricted install 🔧
```

---

## Data Structure Specifications

### 1. TrustLevel Enum

```rust
/// Trust level classification for component sources.
/// 
/// Determines installation workflow:
/// - Trusted: Instant install (no approval delay)
/// - Unknown: Review required (enters approval queue)
/// - DevMode: Bypass security (development only, logged warnings)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrustLevel {
    /// Verified trusted source - instant approval.
    /// 
    /// Sources:
    /// - Internal Git repositories
    /// - Cryptographically signed components
    /// - Pre-verified local components
    /// 
    /// # Workflow
    /// 1. Source matches trusted pattern
    /// 2. Auto-approve installation
    /// 3. Apply declared capabilities
    /// 4. No manual review needed
    Trusted,
    
    /// Unknown source - requires manual review.
    /// 
    /// Sources:
    /// - External Git repositories not in trusted list
    /// - Unsigned components
    /// - First-time sources
    /// 
    /// # Workflow
    /// 1. Source doesn't match any trusted pattern
    /// 2. Enter approval queue
    /// 3. Admin reviews capabilities
    /// 4. Approve/modify/deny
    Unknown,
    
    /// Development mode - bypass security checks.
    /// 
    /// **WARNING:** Disables all security enforcement.
    /// Only use for local development/testing!
    /// 
    /// # Workflow
    /// 1. DevMode flag explicitly enabled
    /// 2. Bypass all capability checks
    /// 3. Grant unrestricted access
    /// 4. Log warnings for audit trail
    /// 
    /// # Security
    /// - MUST NOT be used in production
    /// - All operations logged with DevMode flag
    /// - Visible warnings in console output
    DevMode,
}

impl TrustLevel {
    /// Returns true if this trust level requires approval.
    pub fn requires_approval(&self) -> bool {
        matches!(self, TrustLevel::Unknown)
    }
    
    /// Returns true if this trust level bypasses security.
    pub fn bypasses_security(&self) -> bool {
        matches!(self, TrustLevel::DevMode)
    }
    
    /// Returns security posture description for logging.
    pub fn security_posture(&self) -> &'static str {
        match self {
            TrustLevel::Trusted => "secure-trusted",
            TrustLevel::Unknown => "secure-review-required",
            TrustLevel::DevMode => "insecure-dev-mode",
        }
    }
}
```

### 2. TrustSource Types

```rust
/// Trusted source configuration.
/// 
/// Defines patterns and verification methods for trusted component sources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrustSource {
    /// Git repository source (URL pattern matching).
    /// 
    /// # Examples
    /// ```toml
    /// [[trust.sources]]
    /// type = "git"
    /// url = "https://github.com/mycompany/*"
    /// branch = "main"
    /// description = "Internal company repositories"
    /// ```
    GitRepository {
        /// URL pattern (supports wildcards: `*`, `?`)
        url_pattern: String,
        
        /// Optional branch restriction
        branch: Option<String>,
        
        /// Human-readable description
        description: String,
    },
    
    /// Cryptographically signed component (Ed25519 signature).
    /// 
    /// # Examples
    /// ```toml
    /// [[trust.sources]]
    /// type = "signing_key"
    /// public_key = "ed25519:AAAAC3NzaC1lZDI1NTE5AAAAIJbpYR..."
    /// signer = "engineering@mycompany.com"
    /// description = "Signed by engineering team"
    /// ```
    SigningKey {
        /// Ed25519 public key (hex-encoded)
        public_key: String,
        
        /// Signer identity (email, name, etc.)
        signer: String,
        
        /// Human-readable description
        description: String,
    },
    
    /// Local filesystem path (for pre-verified components).
    /// 
    /// # Examples
    /// ```toml
    /// [[trust.sources]]
    /// type = "local"
    /// path_pattern = "/opt/verified-components/*"
    /// description = "Pre-verified local components"
    /// ```
    LocalPath {
        /// Filesystem path pattern
        path_pattern: String,
        
        /// Human-readable description
        description: String,
    },
}

impl TrustSource {
    /// Checks if this source matches the given component source.
    pub fn matches(&self, component_source: &ComponentSource) -> bool;
    
    /// Returns source type string for logging.
    pub fn source_type(&self) -> &'static str;
}
```

### 3. ComponentSource (Component Origin)

```rust
/// Component source information extracted during installation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentSource {
    /// Component from Git repository.
    Git {
        url: String,
        branch: String,
        commit: String,
    },
    
    /// Component from signed artifact.
    Signed {
        signature: String,
        public_key: String,
    },
    
    /// Component from local filesystem.
    Local {
        path: PathBuf,
    },
}

impl ComponentSource {
    /// Extracts source from component metadata.
    pub fn from_metadata(metadata: &ComponentMetadata) -> Result<Self, TrustError>;
    
    /// Returns source identifier string for logging.
    pub fn identifier(&self) -> String;
}
```

### 4. TrustRegistry (Main Service)

```rust
/// Trust registry managing trusted sources and trust determination.
/// 
/// # Responsibilities
/// - Load trust configuration from TOML file
/// - Match component sources against trusted patterns
/// - Determine TrustLevel for components
/// - Support DevMode override
/// 
/// # Thread Safety
/// - Uses Arc<RwLock<>> for concurrent access
/// - Read-heavy workload (install checks)
/// - Write-light workload (config updates)
pub struct TrustRegistry {
    /// Trusted sources (protected by RwLock)
    sources: Arc<RwLock<Vec<TrustSource>>>,
    
    /// Development mode flag
    dev_mode_enabled: Arc<AtomicBool>,
    
    /// Audit logger
    audit_logger: Arc<SecurityAuditLogger>,
}

impl TrustRegistry {
    /// Creates registry from configuration file.
    pub fn from_config(config_path: &Path) -> Result<Self, TrustError>;
    
    /// Determines trust level for component source.
    pub fn determine_trust_level(
        &self,
        component_id: &str,
        source: &ComponentSource,
    ) -> TrustLevel;
    
    /// Adds trusted source dynamically (requires admin permission).
    pub fn add_trusted_source(&self, source: TrustSource) -> Result<(), TrustError>;
    
    /// Removes trusted source by index.
    pub fn remove_trusted_source(&self, index: usize) -> Result<(), TrustError>;
    
    /// Lists all trusted sources.
    pub fn list_trusted_sources(&self) -> Vec<TrustSource>;
    
    /// Enables/disables development mode (global flag).
    pub fn set_dev_mode(&self, enabled: bool);
    
    /// Checks if development mode is enabled.
    pub fn is_dev_mode(&self) -> bool;
}
```

---

## Configuration File Format

### trust-config.toml Schema

```toml
# Trust Configuration for airssys-wasm
# Version: 1.0

[trust]
# Development mode: Bypass all security checks
# WARNING: NEVER enable in production!
dev_mode = false

# Trusted Git repositories
[[trust.sources]]
type = "git"
url = "https://github.com/mycompany/*"
branch = "main"
description = "Internal company repositories"

[[trust.sources]]
type = "git"
url = "https://github.com/verified-org/wasm-*"
branch = "stable"
description = "Verified external organization"

# Trusted signing keys
[[trust.sources]]
type = "signing_key"
public_key = "ed25519:AAAAC3NzaC1lZDI1NTE5AAAAIJbpYR+abc123..."
signer = "engineering@mycompany.com"
description = "Engineering team signing key"

[[trust.sources]]
type = "signing_key"
public_key = "ed25519:AAAAC3NzaC1lZDI1NTE5AAAAIJbpYR+xyz789..."
signer = "security@mycompany.com"
description = "Security team signing key"

# Pre-verified local components
[[trust.sources]]
type = "local"
path_pattern = "/opt/verified-components/*"
description = "Pre-verified components in /opt"

[[trust.sources]]
type = "local"
path_pattern = "/usr/local/airssys/components/*"
description = "System-installed components"
```

### Configuration Validation Rules

| Rule | Description | Error Message |
|------|-------------|---------------|
| **Valid TOML** | File must parse as valid TOML | "Invalid TOML syntax: {error}" |
| **Required Section** | `[trust]` section must exist | "Missing [trust] section" |
| **Valid Type** | `type` must be "git", "signing_key", or "local" | "Invalid source type: {type}" |
| **Non-Empty Pattern** | URL/path patterns must not be empty | "Empty {field} pattern" |
| **Valid Key Format** | Public keys must start with "ed25519:" | "Invalid public key format" |
| **No Duplicate Sources** | Same source cannot appear twice | "Duplicate source: {identifier}" |

---

## Implementation Steps (15 Steps, ~15 hours)

### Step 1: Create Trust Module Structure (30 min)
- Create `airssys-wasm/src/security/trust.rs`
- Add module declaration to `security/mod.rs`
- Add 3-layer imports (§2.1)
- Define module-level rustdoc
- **Checkpoint**: `cargo check` passes

### Step 2: Implement TrustLevel Enum (45 min)
- `TrustLevel` enum with 3 variants
- Helper methods (`requires_approval()`, `bypasses_security()`, `security_posture()`)
- Derive traits (Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)
- 5 unit tests
- **Checkpoint**: TrustLevel tests pass

### Step 3: Implement TrustSource Types (1 hour)
- `TrustSource` enum with 3 variants (GitRepository, SigningKey, LocalPath)
- `source_type()` helper method
- Serde deserialization support
- 3 unit tests (one per variant)
- **Checkpoint**: TrustSource deserialization works

### Step 4: Implement ComponentSource Types (1 hour)
- `ComponentSource` enum with 3 variants (Git, Signed, Local)
- `from_metadata()` extraction method
- `identifier()` for logging
- 3 unit tests
- **Checkpoint**: ComponentSource extraction works

### Step 5: Implement Pattern Matching Logic (2 hours)
- `TrustSource::matches()` implementation for GitRepository
- Wildcard support (`*`, `?` in URLs/paths)
- Branch restriction logic
- 10 unit tests (valid/invalid patterns, edge cases)
- **Checkpoint**: Pattern matching tests pass

### Step 6: Implement Signing Key Verification (1.5 hours)
- `TrustSource::matches()` for SigningKey
- Ed25519 public key parsing
- Signature verification stub (integration with Task 2.3)
- 5 unit tests
- **Checkpoint**: Key format validation works

### Step 7: Implement Local Path Matching (1 hour)
- `TrustSource::matches()` for LocalPath
- Filesystem path pattern matching
- Absolute path requirements
- 5 unit tests
- **Checkpoint**: Path matching tests pass

### Step 8: Implement TrustRegistry Core (2 hours)
- `TrustRegistry` struct with Arc<RwLock<>>
- `from_config()` TOML parsing
- `determine_trust_level()` main logic
- DevMode flag handling
- 8 unit tests
- **Checkpoint**: Trust determination works

### Step 9: Implement Trust Configuration Parser (1.5 hours)
- TOML deserialization for trust-config.toml
- Configuration validation
- Error handling for malformed configs
- 5 unit tests (valid, invalid, edge cases)
- **Checkpoint**: Config parsing tests pass

### Step 10: Implement Dynamic Trust Management (1 hour)
- `add_trusted_source()` method
- `remove_trusted_source()` method
- `list_trusted_sources()` method
- Thread safety tests
- **Checkpoint**: Dynamic operations work

### Step 11: Implement Audit Logging (1 hour)
- Integrate airssys-osl SecurityAuditLogger
- Log all trust determinations
- Log DevMode usage (WARNING level)
- 3 integration tests
- **Checkpoint**: Audit logs captured

### Step 12: Comprehensive Test Suite (1.5 hours)
- 10 positive tests (trusted sources)
- 10 negative tests (unknown sources)
- 5 DevMode tests
- 5 edge case tests (empty config, invalid patterns)
- **Checkpoint**: 30+ tests pass

### Step 13: Trust Module Documentation (1.5 hours)
- Module-level rustdoc with examples
- Function rustdoc for all public APIs
- Configuration file documentation
- Security considerations
- **Checkpoint**: Zero rustdoc warnings

### Step 14: Examples and Integration Guide (1 hour)
- `examples/security_trust_basic.rs`
- `examples/security_trust_devmode.rs`
- `examples/security_trust_config.rs`
- **Checkpoint**: All examples run

### Step 15: Final Quality Gates (30 min)
- `cargo clippy --all-targets` (zero warnings)
- `cargo test --all-targets` (all pass)
- `cargo doc --no-deps` (zero warnings)
- **Checkpoint**: All quality gates pass

---

## Test Plan (30+ Test Scenarios)

### Positive Tests (10 tests)

| Test ID | Scenario | Expected Output |
|---------|----------|-----------------|
| T01 | Git URL matches trusted pattern | TrustLevel::Trusted |
| T02 | Signed component with known key | TrustLevel::Trusted |
| T03 | Local path matches trusted pattern | TrustLevel::Trusted |
| T04 | DevMode enabled | TrustLevel::DevMode |
| T05 | Wildcard URL pattern match | TrustLevel::Trusted |
| T06 | Branch restriction match | TrustLevel::Trusted |
| T07 | Multiple trusted sources | TrustLevel::Trusted (first match) |
| T08 | Add trusted source dynamically | Source added successfully |
| T09 | Remove trusted source | Source removed successfully |
| T10 | List all trusted sources | Vec<TrustSource> returned |

### Negative Tests (10 tests)

| Test ID | Scenario | Expected Output |
|---------|----------|----------------|
| N01 | Git URL not in trusted list | TrustLevel::Unknown |
| N02 | Unknown signing key | TrustLevel::Unknown |
| N03 | Local path not trusted | TrustLevel::Unknown |
| N04 | Branch doesn't match restriction | TrustLevel::Unknown |
| N05 | Invalid URL pattern syntax | ParseError |
| N06 | Invalid public key format | ParseError |
| N07 | Empty trust config | TrustLevel::Unknown (all) |
| N08 | Duplicate trusted source | ValidationError |
| N09 | Remove non-existent source | NotFoundError |
| N10 | Malformed trust-config.toml | TomlParseError |

### Edge Case Tests (10 tests)

| Test ID | Scenario | Expected Behavior |
|---------|----------|-------------------|
| E01 | Very long URL (1000 chars) | Match correctly |
| E02 | Unicode in Git URL | Match correctly |
| E03 | Mixed case URL patterns | Case-sensitive match |
| E04 | DevMode + trusted source | TrustLevel::DevMode (takes precedence) |
| E05 | Empty sources array | TrustLevel::Unknown (all) |
| E06 | Whitespace in patterns | Trim and match |
| E07 | Multiple wildcard patterns | Match correctly |
| E08 | Symlink in local path | Resolve and match |
| E09 | Concurrent trust checks | Thread-safe |
| E10 | Config file reload | New sources applied |

---

## Performance Targets

### Trust Determination Performance
- **Pattern Matching**: <100μs per source check
- **Total Trust Check**: <1ms for 100 trusted sources
- **Config Loading**: <10ms for 1000-line config file
- **Memory Footprint**: <1KB per trusted source

### Optimization Strategies
1. **Pre-compiled Patterns**: Compile wildcard patterns at load time
2. **RwLock Read Optimization**: Read-heavy workload (99% reads)
3. **Early Exit**: Return Trusted on first match
4. **DevMode Short-Circuit**: Check DevMode flag first (atomic read, <1ns)

---

## Integration Points

### Task 2.2 Integration (Approval Workflow)

```rust
// Task 2.2 will use TrustLevel to determine workflow

pub fn install_component(
    component_id: &str,
    source: &ComponentSource,
) -> Result<ComponentId, InstallError> {
    // Step 1: Determine trust level (THIS TASK)
    let trust_level = trust_registry.determine_trust_level(component_id, source);
    
    // Step 2: Route to appropriate workflow (TASK 2.2)
    match trust_level {
        TrustLevel::Trusted => {
            // Auto-approve workflow
            auto_approve_install(component_id, source).await?
        }
        TrustLevel::Unknown => {
            // Manual review workflow
            review_and_approve(component_id, source).await?
        }
        TrustLevel::DevMode => {
            // Bypass security workflow
            dev_mode_install(component_id, source).await?
        }
    }
}
```

### Task 2.3 Integration (Trust Configuration)

```rust
// Task 2.3 will provide trust-config.toml management CLI

$ airssys-wasm trust add-git "https://github.com/mycompany/*"
✅ Added trusted Git repository: https://github.com/mycompany/*

$ airssys-wasm trust list
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Trusted Sources (3)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
1. Git: https://github.com/mycompany/*
   Description: Internal company repositories
   
2. SigningKey: ed25519:AAAAC3Nz...
   Signer: engineering@mycompany.com
   
3. Local: /opt/verified-components/*
   Description: Pre-verified components
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## Security Considerations

### Critical Security Properties

1. **Deny-by-Default**: Unknown sources always require review
2. **Explicit Trust**: Trust must be explicitly configured (no auto-trust)
3. **Audit Trail**: All trust determinations logged
4. **DevMode Warnings**: DevMode usage prominently logged
5. **No Bypass**: Cannot bypass Unknown → Trusted without config

### Threat Model

| Threat | Mitigation |
|--------|------------|
| **Malicious Source Spoofing** | URL verification, signing key validation |
| **Pattern Bypass** | Strict wildcard matching, no regex injection |
| **DevMode Abuse** | Prominent warnings, audit logging |
| **Config Tampering** | File permissions, integrity checks (Task 2.3) |
| **Trust Escalation** | Dynamic trust changes require admin permission |

### DevMode Safety

```rust
// DevMode produces visible warnings
if trust_level == TrustLevel::DevMode {
    warn!(
        "⚠️  ⚠️  ⚠️  DEVELOPMENT MODE ACTIVE ⚠️  ⚠️  ⚠️\n\
         Component: {}\n\
         Security checks BYPASSED!\n\
         DO NOT use in production!",
        component_id
    );
    
    audit_logger.log_security_event(SecurityEvent::DevModeUsage {
        component_id: component_id.to_string(),
        timestamp: Utc::now(),
        warning: "Unrestricted access granted in DevMode",
    });
}
```

---

## Quality Gates

### Cargo Clippy Requirements
- **Command**: `cargo clippy --all-targets --all-features -- -D warnings`
- **Target**: Zero warnings (deny warnings)
- **Enforced Lints**: `unwrap_used`, `expect_used`, `panic` (deny)

### Rustdoc Requirements
- **Command**: `cargo doc --no-deps --document-private-items`
- **Target**: Zero rustdoc warnings
- **Standards**: Microsoft Rust Guidelines (M-MODULE-DOCS, M-CANONICAL-DOCS)

### Test Coverage Targets
- **Unit Test Coverage**: >90% (all trust logic)
- **Integration Test Coverage**: 5+ integration tests
- **Edge Case Coverage**: 10+ edge case tests
- **Total Tests**: 30+ test cases

---

## Timeline Estimate

| Step | Description | Time | Cumulative |
|------|-------------|------|------------|
| 1 | Trust module structure | 30 min | 30 min |
| 2 | TrustLevel enum | 45 min | 1.25 hours |
| 3 | TrustSource types | 1 hour | 2.25 hours |
| 4 | ComponentSource types | 1 hour | 3.25 hours |
| 5 | Pattern matching logic | 2 hours | 5.25 hours |
| 6 | Signing key verification | 1.5 hours | 6.75 hours |
| 7 | Local path matching | 1 hour | 7.75 hours |
| 8 | TrustRegistry core | 2 hours | 9.75 hours |
| 9 | Config parser | 1.5 hours | 11.25 hours |
| 10 | Dynamic trust management | 1 hour | 12.25 hours |
| 11 | Audit logging | 1 hour | 13.25 hours |
| 12 | Comprehensive test suite | 1.5 hours | 14.75 hours |
| 13 | Trust module documentation | 1.5 hours | 16.25 hours |
| 14 | Examples and integration | 1 hour | 17.25 hours |
| 15 | Final quality gates | 30 min | **15 hours** |

**Total Duration**: 15 hours ≈ **2-3 days** (6-8 hour workdays)

**Breakdown by Activity**:
- Core implementation: 10.25 hours (68%)
- Testing: 1.5 hours (10%)
- Documentation: 2.5 hours (17%)
- Quality assurance: 0.75 hours (5%)

---

## Risk Assessment

### Technical Risks

| Risk | Severity | Probability | Mitigation |
|------|----------|-------------|------------|
| **Pattern Matching Complexity** | Medium | Medium | Use glob crate, comprehensive tests |
| **DevMode Misuse in Production** | High | Low | Prominent warnings, audit logs |
| **Config File Tampering** | High | Low | File permissions, integrity checks |
| **Thread Safety Issues** | Medium | Low | RwLock, atomic operations |

---

## Standards Compliance

### PROJECTS_STANDARD.md
- §2.1: 3-layer import organization ✅
- §4.3: Module architecture (mod.rs only re-exports) ✅
- §5.1: Dependency management ✅
- §6.1: YAGNI principles ✅

### Microsoft Rust Guidelines
- M-DESIGN-FOR-AI: Clear API, extensive docs ✅
- M-CANONICAL-DOCS: Comprehensive public API docs ✅
- M-EXAMPLES: Examples for all public functions ✅

### ADR Compliance
- ADR-WASM-005: Capability-Based Security Model ✅
- ADR-WASM-010: Implementation Strategy ✅

---

## Next Steps After Task 2.1

### Task 2.2: Approval Workflow Engine (2-3 days)
- Approval state machine (Pending → Approved/Rejected)
- Review queue management
- Approval decision persistence
- Integration with TrustLevel (this task)

### Task 2.3: Trust Configuration System (1-2 days)
- CLI for trust management
- Config file validation
- Trust source verification
- DevMode enable/disable controls

---

## Approval Status

**Planner**: Memory Bank Planner  
**Date**: 2025-12-17  
**Status**: ✅ **APPROVED** - Ready for implementation

This plan provides a comprehensive blueprint for implementing the trust-level system with clear security boundaries, developer-friendly workflows, and production-ready documentation.

**Ready to Start:** Task 2.1 implementation can begin immediately after Phase 1 completion.
