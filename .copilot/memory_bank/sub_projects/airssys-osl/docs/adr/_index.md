# airssys-osl Architecture Decision Records Index

**Sub-Project:** airssys-osl  
**Last Updated:** 2025-10-10  
**Total ADRs:** 4  
**Active ADRs:** 4  

## ADR Summary

### By Status
| Status | Count | Description |
|--------|-------|-------------|
| Proposed | 0 | Decisions under consideration |
| Accepted | 4 | Active architectural decisions |
| Deprecated | 0 | Decisions no longer applicable |
| Superseded | 0 | Decisions replaced by newer ones |

### By Category
| Category | Count | Description |
|----------|-------|-------------|
| Technology Selection | 0 | Framework, library, and tool choices |
| Architecture Patterns | 3 | System design and structural decisions |
| Security | 1 | Security model and implementation decisions |
| Performance | 0 | Performance optimization and target decisions |
| Integration | 0 | Integration approaches with other components |
| Platform | 0 | Cross-platform strategy decisions |

## Active Architecture Decision Records

### Architecture Patterns Category

#### ADR-025: Framework dyn Pattern Exception *(Accepted)*
**Date**: 2025-10-03  
**Status**: Accepted  
**Summary**: Allows `dyn` patterns in OSLFramework layer while maintaining generic-first patterns in core primitives for developer ergonomics vs performance optimization.

**Key Decisions**:
- Framework layer uses `Vec<Box<dyn Middleware<dyn Operation>>>` for flexibility
- Core primitives maintain `OSExecutor<O>` and `Middleware<O>` generic patterns
- Clear architectural boundary between ergonomic and performance layers

#### ADR-026: Framework as Primary API Strategy *(Accepted)*
**Date**: 2025-10-03  
**Status**: Accepted  
**Summary**: Establishes OSLFramework builder as the primary recommended API with explicit primitives available for advanced use cases.

**Key Decisions**:
- Documentation leads with framework examples (80% of use cases)
- Prelude module exports framework types primarily
- Progressive disclosure: Framework → Custom Middleware → Direct Primitives
- Advanced API clearly documented but not primary path

#### ADR-027: Builder Pattern Architecture Implementation *(Accepted)*
**Date**: 2025-10-03  
**Status**: Accepted  
**Summary**: Defines multi-level builder architecture with automatic middleware orchestration and progressive complexity support.

**Key Decisions**:
- OSLFramework + OSLFrameworkBuilder + Operation Builders pattern
- MiddlewarePipeline handles automatic middleware execution
- ExecutorRegistry provides automatic executor selection
- Three complexity levels: Simple (80%) → Custom (15%) → Advanced (5%)

### Security Category

#### ADR-028: ACL Permission Model and Glob Pattern Matching *(Accepted)*
**Date**: 2025-10-10  
**Status**: Accepted  
**Summary**: Defines ACL permission model using string-based permissions with glob pattern matching for flexible and extensible access control.

**Key Decisions**:
- String-based permissions (`Vec<String>`) instead of enum for maximum flexibility
- glob crate (v0.3) for resource and permission pattern matching
- Standardized context attributes: `ATTR_RESOURCE` and `ATTR_PERMISSION` constants
- Clear permission semantics: empty=none, "*"=all, glob patterns supported
- Accept breaking API changes in Phase 3 for correctness

**Related**: OSL-TASK-003 Phase 3 - ACL Implementation

## Planned Architecture Decision Records

### Technology Selection ADRs
- **ADR-001: Core Technology Stack**: Rust, Tokio, and foundational dependencies
- **ADR-002: Security Framework Selection**: Security policy engine and enforcement
- **ADR-003: Logging Framework Selection**: Activity logging and monitoring approach
- **ADR-004: Testing Framework Selection**: Testing strategy and tools

### Architecture Pattern ADRs
- **ADR-005: Security-First Architecture**: Security-first design approach and implementation
- **ADR-006: Cross-Platform Abstraction**: Platform abstraction strategy and boundaries
- **ADR-007: Resource Management Architecture**: Resource pooling and limiting design
- **ADR-008: Error Handling Strategy**: Structured error handling and propagation

### Security ADRs
- **ADR-009: Security Policy Model**: Security policy definition and enforcement model
- **ADR-010: Activity Logging Strategy**: Comprehensive operation logging approach
- **ADR-011: Access Control Model**: Fine-grained access control implementation
- **ADR-012: Threat Detection Strategy**: Built-in threat detection and response

### Performance ADRs
- **ADR-013: Async Operation Strategy**: Async/await patterns and runtime selection
- **ADR-014: Zero-Copy Operation Strategy**: Memory efficiency and zero-copy patterns
- **ADR-015: Performance Monitoring**: Performance metrics and monitoring approach
- **ADR-016: Resource Optimization**: CPU, memory, and I/O optimization strategies

### Integration ADRs
- **ADR-017: airssys-rt Integration**: Integration patterns with runtime system
- **ADR-018: airssys-wasm Integration**: Integration patterns with WASM system
- **ADR-019: External Tool Integration**: Integration with docker, gh CLI, etc.
- **ADR-020: Monitoring Integration**: Metrics, tracing, and health check integration

### Platform ADRs
- **ADR-021: Multi-Platform Strategy**: Support strategy for Linux, macOS, Windows
- **ADR-022: Platform-Specific APIs**: Handling of platform-specific functionality
- **ADR-023: Build and Distribution**: Cross-platform build and distribution strategy
- **ADR-024: Testing Strategy**: Cross-platform testing and validation approach

## ADR Creation Timeline

### Phase 1: Foundation ADRs (Current - Q4 2025)
Priority ADRs that must be decided before implementation begins:
1. **ADR-001**: Core Technology Stack
2. **ADR-005**: Security-First Architecture  
3. **ADR-006**: Cross-Platform Abstraction
4. **ADR-008**: Error Handling Strategy

### Phase 2: Implementation ADRs (Q1 2026)
ADRs needed during initial implementation:
1. **ADR-002**: Security Framework Selection
2. **ADR-003**: Logging Framework Selection
3. **ADR-009**: Security Policy Model
4. **ADR-013**: Async Operation Strategy

### Phase 3: Integration ADRs (Q2 2026)
ADRs for integration with other components:
1. **ADR-017**: airssys-rt Integration
2. **ADR-018**: airssys-wasm Integration
3. **ADR-019**: External Tool Integration
4. **ADR-020**: Monitoring Integration

## Decision Dependencies

### Critical Path Decisions
- **ADR-001** (Technology Stack) → Blocks all implementation ADRs
- **ADR-005** (Security Architecture) → Blocks security-related ADRs
- **ADR-006** (Cross-Platform) → Blocks platform-specific ADRs
- **ADR-008** (Error Handling) → Blocks implementation pattern ADRs

### Integration Dependencies
- **ADR-017** (airssys-rt) → Depends on airssys-rt architectural decisions
- **ADR-018** (airssys-wasm) → Depends on airssys-wasm architectural decisions
- **ADR-019** (External Tools) → Depends on security and platform decisions

## Decision Impact Analysis

### High Impact Decisions (Affect Entire System)
- **ADR-001**: Core Technology Stack - Foundation for all development
- **ADR-005**: Security-First Architecture - Affects all system operations
- **ADR-006**: Cross-Platform Abstraction - Affects code structure and maintainability
- **ADR-013**: Async Operation Strategy - Affects performance and complexity

### Medium Impact Decisions (Affect Major Components)
- **ADR-002**: Security Framework Selection - Affects security implementation
- **ADR-009**: Security Policy Model - Affects security configuration
- **ADR-014**: Zero-Copy Strategy - Affects performance characteristics
- **ADR-021**: Multi-Platform Strategy - Affects deployment and testing

### Cross-Component Impact
All integration ADRs (ADR-017 through ADR-020) will significantly impact:
- airssys-rt integration and process management
- airssys-wasm integration and sandboxing
- Overall AirsSys ecosystem architecture

## Quality Assurance

### Review Requirements
- **Technical Review**: All ADRs require technical review by airssys-osl team
- **Security Review**: Security-related ADRs require security team review  
- **Architecture Review**: High-impact ADRs require architectural review
- **Integration Review**: Integration ADRs require review by affected teams

### Documentation Standards
- Follow ADR template exactly (`templates/docs/adr-template.md`)
- Include comprehensive option analysis
- Document implementation and monitoring plans
- Reference workspace standards compliance (§2.1, §3.2, §4.3, §5.1)

### Update and Maintenance
- **Status Tracking**: Monitor implementation progress and effectiveness
- **Regular Review**: Quarterly review of decision effectiveness
- **Supersession Tracking**: Document when decisions are replaced or updated
- **Cross-Reference Maintenance**: Keep links to related docs current

---
**Template Version:** 1.0  
**Last Updated:** 2025-09-27