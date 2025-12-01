# [WASM-TASK-007] - Block 6: Persistent Storage System

**Status:** not-started  
**Added:** 2025-10-20  
**Updated:** 2025-10-20  
**Priority:** Critical Path - Core Services Layer  
**Layer:** 2 - Core Services  
**Block:** 6 of 11  
**Estimated Effort:** 4-5 weeks  

## Overview

Implement the persistent key-value storage system for component state management using NEAR-style API with pluggable StorageBackend trait abstraction, supporting Sled (default) and RocksDB (optional) backends, with prefix-based namespace isolation, application-level quota tracking, and export/import tooling achieving <1ms typical get/set operations.

## Context

**Current State:**
- Architecture complete: KNOWLEDGE-WASM-007 (Component Storage Architecture)
- Technology selected: NEAR-style KV API, Sled default, RocksDB optional
- Backend comparison: KNOWLEDGE-WASM-008 (detailed analysis)
- Design: StorageBackend trait abstraction for pluggable backends

**Problem Statement:**
Components need durable persistent storage for:
1. **Configuration** - Settings, preferences across restarts
2. **Application State** - User data, session information
3. **Cache** - Performance optimization data
4. **Processing State** - Checkpoints, intermediate results
5. **Integration State** - API tokens, sync bookmarks

Requirements:
- Simple key-value API (no complex queries needed)
- Component isolation (namespace per component)
- Quota enforcement (prevent storage abuse)
- Backend flexibility (Sled fast builds, RocksDB proven stability)
- Migration tooling (export/import between backends)

**Why This Block Matters:**
Without persistent storage:
- Components lose all state on restart
- No configuration persistence possible
- No stateful applications
- Framework limited to ephemeral use cases

This block enables real-world stateful components.

## Objectives

### Primary Objective
Implement NEAR-style key-value storage API with StorageBackend trait abstraction, Sled default backend, RocksDB optional backend, prefix-based namespace isolation, application-level quota tracking, and export/import tooling achieving <1ms typical get/set operations.

### Secondary Objectives
- Achieve <1ms P50, <5ms P99 for get/set operations
- Support 10GB+ storage per component (configurable quota)
- Zero data loss on component crash or host restart
- Backend migration without data loss (export/import)
- Clear storage debugging and inspection tools

## Scope

### In Scope
1. **NEAR-Style KV API** - get, set, delete, has, keys, scan_prefix
2. **StorageBackend Trait** - Pluggable backend abstraction
3. **Sled Backend** - Default (pure Rust, fast builds)
4. **RocksDB Backend** - Optional (proven production stability)
5. **Namespace Isolation** - Prefix-based per-component isolation
6. **Quota System** - Application-level quota tracking and enforcement
7. **Export/Import Tool** - JSON Lines format for migration
8. **Storage Host Functions** - WIT interfaces for component access

### Out of Scope
- SQL/NoSQL query languages (just key-value)
- Transactions (actor model sequential processing eliminates need)
- Distributed storage across hosts (single-host Phase 1)
- Storage replication (Phase 2)
- Storage encryption at rest (Phase 2, use OS-level encryption)

## Implementation Plan

### Phase 1: Storage API and Trait Design (Week 1)

#### Task 1.1: NEAR-Style KV API Design
**Deliverables:**
- Storage trait with get/set/delete/has methods
- keys() iterator for key enumeration
- scan_prefix() for prefix queries
- API documentation with examples
- Error types for storage operations

**Success Criteria:**
- API matches NEAR Protocol patterns
- Clear method signatures
- Comprehensive error handling
- Examples demonstrate all operations
- Rust documentation complete

#### Task 1.2: StorageBackend Trait Abstraction
**Deliverables:**
- StorageBackend trait definition
- Backend lifecycle methods (open, close, flush)
- Atomic operation support
- Backend-agnostic tests
- Backend trait documentation

**Success Criteria:**
- Trait supports multiple backends
- Backend-agnostic test suite
- Clear backend implementation contract
- Trait extensible for future backends
- Documentation comprehensive

#### Task 1.3: Component Namespace Design
**Deliverables:**
- Namespace prefix strategy (`component:<id>:`)
- Key composition and parsing
- Namespace isolation validation
- Component ID to namespace mapping
- Namespace documentation

**Success Criteria:**
- Each component has isolated namespace
- Cross-component access impossible
- Key format consistent
- Namespace efficient (minimal overhead)
- Clear namespace conventions

---

### Phase 2: Sled Backend Implementation (Week 1-2)

#### Task 2.1: Sled Integration
**Deliverables:**
- Sled dependency integration
- SledBackend struct implementing StorageBackend
- Database file management
- Configuration options (cache size, sync mode)
- Sled backend documentation

**Success Criteria:**
- Sled backend implements StorageBackend trait
- Database persists correctly
- Configuration options work
- Builds fast (pure Rust benefit)
- Clear integration documentation

#### Task 2.2: Sled Operations Implementation
**Deliverables:**
- get/set/delete operations
- keys() iterator using Sled scan
- scan_prefix() using Sled range queries
- Batch operations for efficiency
- Operation error handling

**Success Criteria:**
- All KV operations functional
- Iterators work correctly
- Prefix scans efficient
- Batch operations optimize performance
- Errors handled gracefully

#### Task 2.3: Sled Performance Optimization
**Deliverables:**
- Sled configuration tuning
- Cache size optimization
- Flush mode configuration (safety vs performance)
- Performance benchmarks
- Optimization documentation

**Success Criteria:**
- Get/set <1ms P50, <5ms P99
- Configuration balances safety and speed
- Benchmarks reproducible
- No data loss on crashes
- Performance documented

---

### Phase 3: RocksDB Backend Implementation (Week 2-3)

#### Task 3.1: RocksDB Integration
**Deliverables:**
- RocksDB dependency (rust-rocksdb crate)
- RocksDBBackend struct implementing StorageBackend
- Database directory management
- Column family support (future multi-tenant optimization)
- RocksDB backend documentation

**Success Criteria:**
- RocksDB backend implements StorageBackend trait
- Database persists correctly
- Configuration options work
- Production stability proven
- Clear integration documentation

#### Task 3.2: RocksDB Operations Implementation
**Deliverables:**
- get/set/delete operations
- keys() iterator using RocksDB iterator
- scan_prefix() using RocksDB prefix seek
- Write batches for atomicity
- Operation error handling

**Success Criteria:**
- All KV operations functional
- Iterators efficient
- Prefix scans optimized
- Write batches work correctly
- Errors handled gracefully

#### Task 3.3: RocksDB vs Sled Comparison
**Deliverables:**
- Performance benchmark comparison
- Space efficiency comparison
- Build time comparison
- Feature matrix comparison
- Backend selection guide

**Success Criteria:**
- Clear performance trade-offs documented
- Space usage compared
- Build time differences quantified
- Selection criteria clear
- Guide helps users choose backend

---

### Phase 4: Namespace Isolation and Quota System (Week 3-4)

#### Task 4.1: Namespace Prefix Enforcement
**Deliverables:**
- Automatic prefix addition to all keys
- Prefix validation on operations
- Cross-namespace access prevention
- Namespace isolation tests
- Isolation documentation

**Success Criteria:**
- Component keys automatically prefixed
- Cross-component access impossible
- Isolation verified through testing
- No bypass vulnerabilities
- Clear isolation guarantees

#### Task 4.2: Application-Level Quota Tracking
**Deliverables:**
- Quota tracking per component
- Storage space calculation (approximate)
- Quota enforcement on set operations
- Quota exceeded error handling
- Quota monitoring API

**Success Criteria:**
- Quotas enforced per component
- Space usage tracked (within 10% accuracy)
- Set operations fail when quota exceeded
- Clear quota error messages
- Quota usage queryable

#### Task 4.3: Quota Configuration System
**Deliverables:**
- Quota declaration in Component.toml
- Default quota values (e.g., 100MB)
- Per-component quota override
- Quota validation at component load
- Quota configuration documentation

**Success Criteria:**
- Quotas configurable per component
- Default quotas reasonable
- Configuration validated
- Clear quota declaration syntax
- Examples demonstrate configuration

---

### Phase 5: Storage Host Functions (Week 4)

#### Task 5.1: WIT Storage Interface
**Deliverables:**
- storage.wit interface definition
- Host function signatures (storage-get, storage-set, etc.)
- Error result types
- Permission annotations (from Block 4)
- WIT documentation

**Success Criteria:**
- Complete storage WIT interface
- Clear function signatures
- Error handling comprehensive
- Permission annotations present
- Examples demonstrate usage

#### Task 5.2: Host Function Implementation
**Deliverables:**
- storage-get host function
- storage-set host function
- storage-delete host function
- storage-has host function
- storage-keys iterator host function
- Error translation (backend errors → WASM traps)

**Success Criteria:**
- All host functions operational
- Components can access storage
- Errors propagate correctly
- Iterator works efficiently
- Comprehensive testing

#### Task 5.3: Capability-Based Storage Security
**Deliverables:**
- Storage capability checks (integration with Block 4)
- Namespace permission patterns
- Quota enforcement at host functions
- Security audit logging
- Security integration tests

**Success Criteria:**
- Capabilities checked before storage access
- Namespace permissions enforced
- Quotas prevent abuse
- Security events logged
- No bypass vulnerabilities

---

### Phase 6: Export/Import Tooling and Testing (Week 4-5)

#### Task 6.1: Export/Import JSON Lines Format
**Deliverables:**
- JSON Lines export format specification
- Export tool (dump all component storage)
- Import tool (load from JSON Lines)
- Format validation
- Migration documentation

**Success Criteria:**
- Export captures all component data
- Import restores data correctly
- Format human-readable (JSON Lines)
- Backend-agnostic (works with both Sled and RocksDB)
- Clear migration guide

#### Task 6.2: Backend Migration Tool
**Deliverables:**
- Migrate command (Sled → RocksDB, RocksDB → Sled)
- Data integrity verification
- Progress reporting for large migrations
- Rollback on error
- Migration tool documentation

**Success Criteria:**
- Migration preserves all data
- No data loss during migration
- Progress visible for long operations
- Errors rolled back cleanly
- Tool easy to use

#### Task 6.3: Comprehensive Storage Testing
**Deliverables:**
- Storage API test suite
- Backend-agnostic tests (run against both backends)
- Quota enforcement tests
- Crash recovery tests (fsync validation)
- Performance benchmarks

**Success Criteria:**
- Test coverage >95%
- Both backends pass same test suite
- Quota enforcement validated
- Crash recovery works (no data loss)
- Performance targets met (<1ms P50)

---

## Success Criteria

### Definition of Done
This task is complete when:

1. ✅ **NEAR-Style KV API Implemented**
   - get, set, delete, has, keys, scan_prefix functional
   - API matches NEAR Protocol patterns
   - Clear error handling
   - Comprehensive documentation

2. ✅ **StorageBackend Trait Working**
   - Pluggable backend abstraction
   - Backend lifecycle management
   - Backend-agnostic tests passing
   - Clear implementation contract

3. ✅ **Sled Backend Operational**
   - Default backend working
   - All operations functional
   - Performance targets met (<1ms P50)
   - Pure Rust fast build benefit

4. ✅ **RocksDB Backend Operational**
   - Optional backend working
   - All operations functional
   - Production stability proven
   - Clear selection guide

5. ✅ **Namespace Isolation Enforced**
   - Component storage isolated (prefix-based)
   - Cross-component access impossible
   - Isolation verified through testing
   - No bypass vulnerabilities

6. ✅ **Quota System Working**
   - Application-level quotas enforced
   - Quota tracking per component
   - Configuration in Component.toml
   - Quota exceeded errors clear

7. ✅ **Export/Import Tooling Complete**
   - JSON Lines format export/import
   - Backend migration tool working
   - Data integrity verified
   - Clear migration documentation

8. ✅ **Testing & Documentation Complete**
   - Test coverage >95%
   - Both backends tested
   - Performance benchmarks met
   - Complete storage guide

## Dependencies

### Upstream Dependencies
- ✅ WASM-TASK-005: Security & Isolation (Block 4) - **REQUIRED** for capability checks
- ✅ WASM-TASK-003: WIT Interface System (Block 2) - **REQUIRED** for storage.wit
- ✅ KNOWLEDGE-WASM-007: Component Storage Architecture - **COMPLETE**
- ✅ KNOWLEDGE-WASM-008: Storage Backend Comparison - **COMPLETE**

### Downstream Dependencies (Blocks This Task)
- WASM-TASK-008: Component Lifecycle (Block 7) - needs storage for component version data
- WASM-TASK-010: Monitoring & Observability (Block 9) - needs storage metrics
- WASM-TASK-011: Component SDK (Block 10) - needs storage API wrappers

### External Dependencies
- Sled crate (v0.34+, pure Rust embedded database)
- RocksDB crate (rust-rocksdb, C++ bindings)
- serde and serde_json for export/import

## Risks and Mitigations

### Risk 1: Sled Stability Concerns
**Impact:** Medium - Sled less battle-tested than RocksDB  
**Probability:** Medium - Sled is production-ready but younger  
**Mitigation:**
- Make RocksDB optional fallback
- Extensive testing with Sled
- Monitor Sled project health
- Document backend selection criteria

### Risk 2: RocksDB Build Complexity
**Impact:** Medium - C++ compilation adds build time  
**Probability:** High - RocksDB requires C++ toolchain  
**Mitigation:**
- Make RocksDB optional feature
- Document C++ dependency requirements
- Provide Sled as zero-hassle default
- CI tests both backends

### Risk 3: Quota Tracking Accuracy
**Impact:** Low - Approximate quotas might be too generous or strict  
**Probability:** Medium - Calculating exact storage is expensive  
**Mitigation:**
- Document quota as approximate (within 10%)
- Err on the side of being generous
- Provide tools to inspect actual usage
- Make quotas configurable

### Risk 4: Performance Not Meeting Targets
**Impact:** High - Slow storage impacts all components  
**Probability:** Low - Both backends are fast  
**Mitigation:**
- Benchmark continuously during development
- Optimize hot paths (caching, batching)
- Leverage backend-specific optimizations
- Profile and address bottlenecks early

### Risk 5: Data Loss on Crashes
**Impact:** Critical - Data loss unacceptable  
**Probability:** Low - Both backends support fsync  
**Mitigation:**
- Configure backends for durability (fsync)
- Test crash recovery extensively
- Document durability guarantees
- Monitor fsync performance impact

## Progress Tracking

**Overall Status:** not-started - 0%

### Phase Breakdown
| Phase | Description | Status | Estimated Duration | Notes |
|-------|-------------|--------|-------------------|-------|
| 1 | Storage API and Trait Design | not-started | Week 1 | Foundation |
| 2 | Sled Backend Implementation | not-started | Week 1-2 | Default backend |
| 3 | RocksDB Backend Implementation | not-started | Week 2-3 | Optional backend |
| 4 | Namespace Isolation and Quota System | not-started | Week 3-4 | Security |
| 5 | Storage Host Functions | not-started | Week 4 | Component access |
| 6 | Export/Import Tooling and Testing | not-started | Week 4-5 | Migration & QA |

### Subtasks
| ID | Description | Status | Updated | Notes |
|----|-------------|--------|---------|-------|
| 1.1 | NEAR-Style KV API Design | not-started | - | API foundation |
| 1.2 | StorageBackend Trait Abstraction | not-started | - | Pluggable backends |
| 1.3 | Component Namespace Design | not-started | - | Isolation |
| 2.1 | Sled Integration | not-started | - | Default backend |
| 2.2 | Sled Operations Implementation | not-started | - | KV operations |
| 2.3 | Sled Performance Optimization | not-started | - | Performance target |
| 3.1 | RocksDB Integration | not-started | - | Optional backend |
| 3.2 | RocksDB Operations Implementation | not-started | - | KV operations |
| 3.3 | RocksDB vs Sled Comparison | not-started | - | Selection guide |
| 4.1 | Namespace Prefix Enforcement | not-started | - | Isolation |
| 4.2 | Application-Level Quota Tracking | not-started | - | Abuse prevention |
| 4.3 | Quota Configuration System | not-started | - | Configuration |
| 5.1 | WIT Storage Interface | not-started | - | Interface definition |
| 5.2 | Host Function Implementation | not-started | - | Component access |
| 5.3 | Capability-Based Storage Security | not-started | - | Security integration |
| 6.1 | Export/Import JSON Lines Format | not-started | - | Migration format |
| 6.2 | Backend Migration Tool | not-started | - | Migration tooling |
| 6.3 | Comprehensive Storage Testing | not-started | - | Quality assurance |

## Progress Log

*No progress yet - task just created*

## Related Documentation

### ADRs
- **ADR-WASM-007: Storage Backend Selection** - (Future) Backend selection rationale
- **ADR-WASM-005: Capability-Based Security Model** - Storage permission integration

### Knowledge Documentation
- **KNOWLEDGE-WASM-007: Component Storage Architecture** - Primary storage design reference
- **KNOWLEDGE-WASM-008: Storage Backend Comparison** - Sled vs RocksDB detailed analysis
- **KNOWLEDGE-WASM-004: WIT Management Architecture** - Storage WIT interfaces

### External References
- [NEAR Protocol Storage](https://docs.near.org/concepts/storage/storage-staking)
- [Sled Documentation](https://docs.rs/sled/)
- [RocksDB Documentation](https://github.com/facebook/rocksdb/wiki)
- [JSON Lines Format](https://jsonlines.org/)

## Notes

**NEAR-Style API Choice:**
NEAR Protocol has excellent storage API design - simple, intuitive, language-agnostic.
We adopt their patterns but exclude economic models (rent, staking).

**No Transactions Needed:**
Actor model sequential processing eliminates need for transactions.
Components process messages one at a time, no concurrent access to same storage.

**Namespace Prefix Pattern:**
`component:<component_id>:` prefix ensures isolation.
All component keys automatically prefixed, cross-component access impossible.

**Sled vs RocksDB Trade-offs:**
- **Sled**: Pure Rust, fast builds, great for development, production-ready
- **RocksDB**: Battle-tested, proven at scale, C++ build complexity

**Default: Sled, Optional: RocksDB**
Best of both worlds: fast builds by default, proven stability as option.

**Application-Level Quotas:**
Tracking exact storage expensive. Approximate quotas (within 10%) acceptable.
Components declare quotas in Component.toml, enforced at host functions.

**Export/Import Critical:**
Backend migration must be safe and reliable. JSON Lines format:
- Human-readable
- Line-oriented (streaming)
- Standard tooling support

**Performance Targets:**
<1ms P50, <5ms P99 for get/set operations.
Both Sled and RocksDB easily achieve this for typical workloads.

**Crash Recovery:**
Configure backends with fsync for durability. Accept small performance penalty
for data safety. Test crash scenarios extensively.

**Integration with Block 4:**
All storage access checked against capabilities. Storage capability pattern:
`storage:namespace:*` or `storage:namespace:keys:*` for fine-grained control.

**Phase 2 Enhancements:**
- Storage encryption at rest
- Storage replication
- Distributed storage across hosts
- Advanced quota policies (time-based retention)
- Storage compression
