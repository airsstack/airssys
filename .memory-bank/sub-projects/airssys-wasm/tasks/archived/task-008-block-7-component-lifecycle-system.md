# [WASM-TASK-008] - Block 7: Component Lifecycle System

**Status:** not-started  
**Added:** 2025-10-20  
**Updated:** 2025-10-20  
**Priority:** Critical Path - Core Services Layer  
**Layer:** 2 - Core Services  
**Block:** 7 of 11  
**Estimated Effort:** 6-7 weeks  

## Overview

Implement complete component lifecycle management system with installation engine supporting Git/Local/URL sources, Ed25519 signature verification, immutable versioned storage (Merkle-DAG content-addressed), blue-green routing for zero-downtime updates, component registry with dependency resolution, and rollback mechanisms achieving <100ms version switching with zero message loss.

## Context

**Current State:**
- Architecture complete: KNOWLEDGE-WASM-009 (Component Lifecycle & Updates)
- Component.toml specification: KNOWLEDGE-WASM-010 (detailed manifest schema)
- Installation sources: Git repositories, local paths, URLs
- Signature verification: Ed25519 cryptographic signing
- Storage strategy: Content-addressed immutable storage

**Problem Statement:**
Production WASM component systems need:
1. **Installation** - Fetch from Git/Local/URL, validate signatures
2. **Versioning** - Multiple versions coexist, immutable storage
3. **Updates** - Zero-downtime version switching
4. **Dependency Resolution** - Transitive dependency handling
5. **Rollback** - Instant revert to previous working version
6. **Security** - Signature verification prevents tampering
7. **Registry** - Component discovery and metadata

Requirements:
- Support multiple installation sources
- Ed25519 signature verification mandatory for production
- Immutable versioned storage (content-addressed)
- Blue-green routing for zero-downtime updates
- Dependency resolution with version constraints
- One-command rollback to any previous version
- Component registry for discovery

**Why This Block Matters:**
Without lifecycle management:
- No safe way to install components
- No update mechanism (requires host restart)
- No rollback capability (single version risk)
- No dependency management
- No signature verification (security risk)

This block enables production-grade component deployment.

## Objectives

### Primary Objective
Implement complete component lifecycle system with installation engine (Git/Local/URL), Ed25519 signature verification, immutable Merkle-DAG storage, blue-green routing, component registry with dependency resolution, and rollback mechanisms achieving <100ms version switching with zero message loss.

### Secondary Objectives
- Support 100+ concurrent component instances
- Enable <100ms version switching (blue-green routing)
- Zero message loss during updates
- Dependency resolution with version constraints
- One-command rollback to any version
- Component registry with search and discovery

## Scope

### In Scope
1. **Installation Engine** - Git clone, local copy, URL download
2. **Signature Verification** - Ed25519 signing and verification
3. **Immutable Storage** - Content-addressed Merkle-DAG storage
4. **Component Registry** - Metadata storage and discovery
5. **Dependency Resolution** - Transitive dependency handling
6. **Blue-Green Routing** - Zero-downtime version switching
7. **Rollback Mechanism** - Instant revert to previous versions
8. **Component Metadata** - Version info, capabilities, dependencies

### Out of Scope
- Automatic dependency installation (manual Phase 1, auto Phase 2)
- Component marketplace (Phase 2)
- Binary caching/CDN (Phase 2)
- Delta updates (Phase 2, full reinstall Phase 1)
- Component hot-reload without restart (Phase 2)

## Implementation Plan

### Phase 1: Component Installation Engine (Week 1-2)

#### Task 1.1: Installation Source Abstraction
**Deliverables:**
- InstallSource trait (Git, Local, URL)
- Source URI parsing and validation
- Source-specific credential handling
- Error types for installation failures
- Installation abstraction documentation

**Success Criteria:**
- Trait supports all three sources
- URI parsing robust
- Credentials handled securely
- Clear error messages
- Comprehensive documentation

#### Task 1.2: Git Source Implementation
**Deliverables:**
- Git clone integration (git2-rs crate)
- Branch/tag/commit resolution
- Shallow clone optimization
- Credential handling (SSH keys, tokens)
- Git source documentation

**Success Criteria:**
- Git repositories cloned correctly
- Branch/tag/commit specified
- Shallow clone reduces bandwidth
- Authentication works (SSH, HTTPS)
- Clear Git error handling

#### Task 1.3: Local and URL Source Implementation
**Deliverables:**
- Local path copying (filesystem operations)
- URL download (reqwest HTTP client)
- Archive extraction (tar.gz, zip)
- Progress reporting for large downloads
- Local/URL source documentation

**Success Criteria:**
- Local paths copied correctly
- URL downloads work (HTTP/HTTPS)
- Archives extracted properly
- Progress visible for large files
- Error handling comprehensive

---

### Phase 2: Ed25519 Signature System (Week 2)

#### Task 2.1: Signature Generation Tool
**Deliverables:**
- Ed25519 key pair generation
- Component signing tool (CLI)
- Signature file format (.sig)
- Key management best practices
- Signing tool documentation

**Success Criteria:**
- Key pairs generated securely
- Components signed correctly
- Signature file format standard
- Clear key management guide
- Tool easy to use

#### Task 2.2: Signature Verification Implementation
**Deliverables:**
- Ed25519 verification integration (ed25519-dalek)
- Public key trust store
- Signature verification on install
- Unsigned component handling (DevMode only)
- Verification documentation

**Success Criteria:**
- Signatures verified correctly
- Tampered components rejected
- Trust store configurable
- DevMode allows unsigned (with warning)
- Verification fast (<10ms)

#### Task 2.3: Trust Model and Key Distribution
**Deliverables:**
- Trust store management (add/remove keys)
- Key revocation mechanism
- Developer key workflow guide
- Production key security guide
- Trust model documentation

**Success Criteria:**
- Trust store easy to manage
- Key revocation works
- Developer workflow clear
- Production security documented
- Trust model comprehensive

---

### Phase 3: Immutable Content-Addressed Storage (Week 3)

#### Task 3.1: Merkle-DAG Storage Design
**Deliverables:**
- Content-addressed storage schema
- Hash function selection (BLAKE3 or SHA256)
- Directory structure design (`components/<hash>/`)
- Deduplication strategy
- Storage design documentation

**Success Criteria:**
- Content-addressed by hash
- Hash function fast and secure
- Storage structure efficient
- Deduplication reduces space
- Design clearly documented

#### Task 3.2: Component Storage Operations
**Deliverables:**
- Store component (hash, write to storage)
- Retrieve component by hash
- List all stored versions
- Garbage collection for unused versions
- Storage operations documentation

**Success Criteria:**
- Components stored immutably
- Retrieval by hash fast
- Version enumeration works
- GC reclaims space safely
- Operations well-documented

#### Task 3.3: Versioned Component Metadata
**Deliverables:**
- Component metadata storage (name, version, hash)
- Version to hash mapping
- Latest version tracking
- Version history query
- Metadata documentation

**Success Criteria:**
- Metadata stored per version
- Version lookup fast
- Latest version easily found
- History queryable
- Clear metadata schema

---

### Phase 4: Component Registry and Dependencies (Week 3-4)

#### Task 4.1: Component Registry Implementation
**Deliverables:**
- Registry data structure (name → versions)
- Component registration on install
- Component unregistration on remove
- Registry persistence (JSON or SQLite)
- Registry documentation

**Success Criteria:**
- Registry stores all components
- Registration atomic
- Unregistration safe
- Registry persists across restarts
- Clear registry API

#### Task 4.2: Dependency Resolution System
**Deliverables:**
- Dependency parsing from Component.toml
- Transitive dependency resolution
- Version constraint satisfaction (semver)
- Dependency cycle detection
- Resolution algorithm documentation

**Success Criteria:**
- Dependencies parsed correctly
- Transitive deps resolved
- Version constraints satisfied
- Cycles detected and rejected
- Algorithm efficient

#### Task 4.3: Dependency Installation Workflow
**Deliverables:**
- Dependency installation ordering
- Missing dependency detection
- Dependency conflict resolution
- Installation progress reporting
- Workflow documentation

**Success Criteria:**
- Dependencies installed in order
- Missing deps reported clearly
- Conflicts detected early
- Progress visible
- Workflow intuitive

---

### Phase 5: Blue-Green Routing and Zero-Downtime Updates (Week 4-5)

#### Task 5.1: Component Routing Layer
**Deliverables:**
- Routing table (component name → actor address)
- Route update atomicity
- Message buffering during switch
- Routing layer documentation
- Routing performance benchmarks

**Success Criteria:**
- Routing table updated atomically
- Messages buffered (not dropped)
- Routing adds minimal latency (<10μs)
- Documentation clear
- Performance acceptable

#### Task 5.2: Blue-Green Deployment Strategy
**Deliverables:**
- Version switching algorithm
- Old version graceful shutdown
- New version warm-up period
- Message drain and redirect
- Blue-green strategy documentation

**Success Criteria:**
- Version switch <100ms
- Zero messages lost
- Old version drains gracefully
- New version ready before switch
- Strategy well-documented

#### Task 5.3: Atomic Update Execution
**Deliverables:**
- Update transaction design
- Rollback on failure
- Health check before switch
- Update status reporting
- Update execution documentation

**Success Criteria:**
- Updates atomic (all or nothing)
- Failures roll back automatically
- Health checks prevent bad switches
- Status visible during update
- Execution reliable

---

### Phase 6: Rollback, Monitoring, and Testing (Week 5-7)

#### Task 6.1: Rollback Mechanism
**Deliverables:**
- Rollback command implementation
- Version history tracking
- Instant rollback to any version
- Rollback confirmation prompts
- Rollback documentation

**Success Criteria:**
- Rollback instant (<100ms)
- Any previous version restored
- History preserved
- User confirmation prevents accidents
- Clear rollback guide

#### Task 6.2: Lifecycle Event Monitoring
**Deliverables:**
- Installation event logging
- Update event tracking
- Rollback event recording
- Lifecycle metrics (install time, update time)
- Monitoring documentation

**Success Criteria:**
- All lifecycle events logged
- Metrics collected accurately
- Event history queryable
- Metrics useful for debugging
- Monitoring integrated

#### Task 6.3: Comprehensive Lifecycle Testing
**Deliverables:**
- Installation test suite (all sources)
- Signature verification tests
- Update and rollback tests
- Dependency resolution tests
- Performance benchmarks

**Success Criteria:**
- Test coverage >95%
- All sources tested
- Signature tests comprehensive
- Update/rollback verified
- Performance targets met

---

## Success Criteria

### Definition of Done
This task is complete when:

1. ✅ **Installation Engine Working**
   - Git, Local, URL sources functional
   - Component fetching reliable
   - Credential handling secure
   - Clear installation errors

2. ✅ **Signature Verification Operational**
   - Ed25519 signing/verification working
   - Trust store manageable
   - Tampered components rejected
   - DevMode allows unsigned (with warning)

3. ✅ **Immutable Storage Implemented**
   - Content-addressed storage working
   - Multiple versions coexist
   - Deduplication reduces space
   - Storage efficient

4. ✅ **Component Registry Complete**
   - All components registered
   - Version metadata stored
   - Discovery and search working
   - Registry persists correctly

5. ✅ **Dependency Resolution Working**
   - Transitive dependencies resolved
   - Version constraints satisfied
   - Cycles detected
   - Installation ordering correct

6. ✅ **Blue-Green Routing Functional**
   - Zero-downtime updates working
   - Version switching <100ms
   - Zero message loss verified
   - Graceful old version shutdown

7. ✅ **Rollback Mechanism Reliable**
   - Instant rollback to any version
   - Version history preserved
   - Rollback safe and tested
   - Clear rollback workflow

8. ✅ **Testing & Documentation Complete**
   - Test coverage >95%
   - All sources tested
   - Performance benchmarks met
   - Complete lifecycle guide

## Dependencies

### Upstream Dependencies
- ✅ WASM-TASK-006: Inter-Component Communication (Block 5) - **REQUIRED** for routing layer
- ✅ WASM-TASK-007: Persistent Storage (Block 6) - **REQUIRED** for registry storage
- ✅ WASM-TASK-004: Actor System Integration (Block 3) - **REQUIRED** for component actors
- ✅ KNOWLEDGE-WASM-009: Component Lifecycle & Updates - **COMPLETE**
- ✅ KNOWLEDGE-WASM-010: Component.toml Specification - **COMPLETE**

### Downstream Dependencies (Blocks This Task)
- WASM-TASK-012: CLI Tool (Block 11) - needs lifecycle commands (install, update, rollback)
- WASM-TASK-010: Monitoring & Observability (Block 9) - needs lifecycle event metrics
- WASM-TASK-011: Component SDK (Block 10) - needs lifecycle API wrappers

### External Dependencies
- git2-rs (libgit2 bindings for Git operations)
- ed25519-dalek (Ed25519 signature library)
- reqwest (HTTP client for URL downloads)
- tar, flate2, zip (archive extraction)
- semver (semantic versioning)

## Risks and Mitigations

### Risk 1: Git Operations Complexity
**Impact:** High - Git is complex, many edge cases  
**Probability:** Medium - Git operations can fail in many ways  
**Mitigation:**
- Use mature git2-rs library
- Extensive error handling
- Clear error messages for users
- Fallback to URL downloads if Git fails

### Risk 2: Signature Verification Performance
**Impact:** Low - Verification adds latency  
**Probability:** Low - Ed25519 is fast  
**Mitigation:**
- Benchmark verification early
- Cache verification results
- Parallelize verification for dependencies
- Target <10ms per component

### Risk 3: Blue-Green Complexity
**Impact:** High - Message loss unacceptable  
**Probability:** Medium - Routing layer adds complexity  
**Mitigation:**
- Extensive testing of message handling
- Buffering ensures no message loss
- Health checks prevent bad switches
- Comprehensive integration tests

### Risk 4: Dependency Resolution Bugs
**Impact:** High - Wrong dependencies break components  
**Probability:** Medium - Resolution is complex  
**Mitigation:**
- Test with complex dependency graphs
- Use proven semver library
- Clear conflict error messages
- Manual resolution as fallback

### Risk 5: Storage Space Exhaustion
**Impact:** Medium - Multiple versions consume space  
**Probability:** Medium - Without GC, space grows  
**Mitigation:**
- Implement garbage collection early
- Warn when storage low
- Configurable retention policy
- Manual cleanup commands

## Progress Tracking

**Overall Status:** not-started - 0%

### Phase Breakdown
| Phase | Description | Status | Estimated Duration | Notes |
|-------|-------------|--------|-------------------|-------|
| 1 | Component Installation Engine | not-started | Week 1-2 | Multi-source support |
| 2 | Ed25519 Signature System | not-started | Week 2 | Security foundation |
| 3 | Immutable Content-Addressed Storage | not-started | Week 3 | Versioning foundation |
| 4 | Component Registry and Dependencies | not-started | Week 3-4 | Discovery & deps |
| 5 | Blue-Green Routing and Zero-Downtime | not-started | Week 4-5 | Update mechanism |
| 6 | Rollback, Monitoring, and Testing | not-started | Week 5-7 | Reliability & QA |

### Subtasks
| ID | Description | Status | Updated | Notes |
|----|-------------|--------|---------|-------|
| 1.1 | Installation Source Abstraction | not-started | - | Foundation |
| 1.2 | Git Source Implementation | not-started | - | Primary source |
| 1.3 | Local and URL Source Implementation | not-started | - | Alternative sources |
| 2.1 | Signature Generation Tool | not-started | - | Developer tooling |
| 2.2 | Signature Verification Implementation | not-started | - | Security enforcement |
| 2.3 | Trust Model and Key Distribution | not-started | - | Trust management |
| 3.1 | Merkle-DAG Storage Design | not-started | - | Storage foundation |
| 3.2 | Component Storage Operations | not-started | - | CRUD operations |
| 3.3 | Versioned Component Metadata | not-started | - | Metadata management |
| 4.1 | Component Registry Implementation | not-started | - | Discovery system |
| 4.2 | Dependency Resolution System | not-started | - | Dep resolution |
| 4.3 | Dependency Installation Workflow | not-started | - | Installation flow |
| 5.1 | Component Routing Layer | not-started | - | Routing foundation |
| 5.2 | Blue-Green Deployment Strategy | not-started | - | Update strategy |
| 5.3 | Atomic Update Execution | not-started | - | Transaction safety |
| 6.1 | Rollback Mechanism | not-started | - | Safety net |
| 6.2 | Lifecycle Event Monitoring | not-started | - | Observability |
| 6.3 | Comprehensive Lifecycle Testing | not-started | - | Quality assurance |

## Progress Log

*No progress yet - task just created*

## Related Documentation

### ADRs
- **ADR-WASM-008: Component Installation Strategy** - (Future) Installation source decisions
- **ADR-WASM-009: Blue-Green Update Mechanism** - (Future) Update strategy rationale

### Knowledge Documentation
- **KNOWLEDGE-WASM-009: Component Lifecycle & Updates** - Primary lifecycle design reference
- **KNOWLEDGE-WASM-010: Component.toml Specification** - Manifest schema and semantics
- **KNOWLEDGE-WASM-005: Inter-Component Communication Design** - Routing layer foundation

### External References
- [Ed25519 Signature Scheme](https://ed25519.cr.yp.to/)
- [Content-Addressed Storage](https://en.wikipedia.org/wiki/Content-addressable_storage)
- [Blue-Green Deployment](https://martinfowler.com/bliki/BlueGreenDeployment.html)
- [Semantic Versioning](https://semver.org/)

## Notes

**Immutable Storage Benefits:**
Content-addressed storage enables:
- Multiple versions coexist safely
- Rollback is instant (just change pointer)
- Deduplication saves space (same content, one copy)
- Integrity verification (hash mismatch = corruption)

**Blue-Green Routing Critical:**
Zero-downtime updates require:
1. New version starts in background
2. Health checks verify it's ready
3. Routing table updated atomically
4. Old version drains gracefully
5. Messages buffered during transition

**Message Loss Prevention:**
During version switch:
- Messages to old version buffered
- Routing table updated atomically
- Buffered messages replayed to new version
- No message dropped or lost

**Ed25519 Choice:**
Ed25519 chosen over RSA because:
- Fast signing and verification
- Small signatures (64 bytes)
- Small keys (32 bytes public, 64 bytes private)
- Modern, secure algorithm

**Dependency Resolution Complexity:**
Transitive dependency resolution is NP-complete in general.
We use semver constraints to limit search space.
Manual resolution as escape hatch for conflicts.

**Garbage Collection Strategy:**
Keep last N versions (configurable, default 5).
Mark versions used by running components as "in-use".
GC deletes unused versions older than threshold.

**DevMode vs Production:**
- **DevMode**: Unsigned components allowed (warning logged)
- **Production**: Unsigned components rejected

**Rollback Speed:**
Rollback is instant because:
- Old version still in storage (immutable)
- Just update routing table pointer
- No compilation or extraction needed
- Target: <100ms total rollback time

**Installation Source Priority:**
1. **Git**: Preferred for open-source, version control integration
2. **Local**: Development and testing
3. **URL**: Binary distribution, CDN delivery

**Component.toml Critical:**
Manifest contains:
- Component metadata (name, version, author)
- Dependencies with version constraints
- Capability declarations
- Storage quota
- Configuration schema

**Phase 2 Enhancements:**
- Automatic dependency installation (Phase 1 manual)
- Component marketplace and discovery service
- Binary caching and CDN integration
- Delta updates (only changed files)
- Hot reload without actor restart
- Distributed component registry
