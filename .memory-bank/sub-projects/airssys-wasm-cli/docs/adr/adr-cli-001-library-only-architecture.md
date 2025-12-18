# ADR-CLI-001: Library-Only Architecture

**ADR ID:** ADR-CLI-001  
**Created:** 2025-12-18  
**Updated:** 2025-12-18  
**Status:** Accepted  
**Deciders:** Architecture Team, Platform Team

---

## Title

Library-Only Architecture for airssys-wasm-cli: No Binary, 100% Composable

---

## Context

### Problem Statement

The airssys-wasm-cli project needs to provide CLI functionality for component management, but must also integrate seamlessly into the airsstack binary as a subcommand. The traditional approach of creating a standalone binary CLI tool creates integration challenges and code duplication.

**Key Challenges**:
1. How to provide CLI functionality for both standalone and composed usage?
2. How to avoid code duplication between airssys-wasm-cli and airsstack?
3. How to ensure consistent behavior across different distribution models?
4. How to maximize reusability and testability of CLI logic?

### Business Context

**Business Drivers**:
- **Unified User Experience**: Users should have a consistent CLI experience whether using standalone tools or the airsstack unified binary
- **Flexible Distribution**: Need to support multiple distribution models (standalone, integrated, minimal)
- **Rapid Development**: Minimize development time by reusing CLI logic across tools
- **Maintainability**: Single source of truth for CLI commands reduces maintenance burden

**Business Requirements**:
- Support airsstack integration as primary use case
- Allow standalone CLI tools if needed in the future
- Enable third-party tools to reuse CLI commands
- Minimize binary distribution size

### Technical Context

**Current Landscape**:
- **airsstack**: Main binary that orchestrates all AirsStack ecosystem tools
- **airssys-wasm-cli**: CLI for airssys-wasm component management
- **Clap 4.x**: Modern CLI framework with excellent composition support
- **Workspace Architecture**: Monorepo with multiple packages

**Constraints**:
- Must integrate with existing airsstack binary architecture
- Must follow workspace standards (§2.1, §3.2, §4.3, §5.1)
- Must maintain zero-warning policy
- Must be testable without process spawning

### Stakeholders

**Primary Stakeholders**:
- **Platform Team**: Owns airsstack binary and integration
- **CLI Users**: Developers using command-line tools
- **Third-Party Developers**: Potential consumers of CLI library

**Secondary Stakeholders**:
- **DevOps Teams**: Binary distribution and deployment
- **Security Team**: Audit and compliance concerns

---

## Decision

### Summary

**Decision**: Implement airssys-wasm-cli as a **100% library crate with zero binary components**. All CLI functionality will be exported as Clap-based structures that can be composed by any binary application, primarily airsstack.

**Key Points**:
1. airssys-wasm-cli has NO `[[bin]]` section in Cargo.toml
2. All commands are Clap `Args` and `Subcommand` structures
3. Each command has a public `execute()` function
4. Commands are exported from `lib.rs` for external composition
5. airsstack (or other binaries) compose and route commands

### Rationale

**Primary Reasons**:

1. **Maximum Reusability**: Library code can be used by any binary
   - airsstack can compose all or subset of commands
   - Third-party tools can integrate specific commands
   - Future binaries can pick and choose functionality

2. **Integration with airsstack**: Natural fit for subcommand architecture
   - airsstack: `airsstack wasm trust add-git ...`
   - Clean composition using Clap's subcommand system
   - Single binary distribution for users

3. **Testability**: Direct function calls without process spawning
   - Unit tests can call `execute()` directly
   - Fast test execution
   - Easy mocking and dependency injection

4. **Maintainability**: Single source of truth
   - CLI logic lives in one place
   - Changes propagate automatically to all binaries
   - No code duplication

5. **Flexibility**: Support multiple distribution models
   - Integrated (airsstack) - primary
   - Standalone (future if needed) - create thin binary wrapper
   - Minimal (subset) - compose only needed commands

### Assumptions

1. **Clap Stability**: Clap 4.x API remains stable across minor versions
2. **airsstack Integration**: airsstack will use Clap for its CLI (confirmed)
3. **Workspace Architecture**: Monorepo structure continues
4. **No Native Binary Need**: No immediate need for standalone airssys-wasm-cli binary
5. **Rust Ecosystem**: Library-only CLI pattern is well-understood in Rust ecosystem (e.g., clap_cargo, clap_verbosity_flag)

---

## Considered Options

### Option 1: Binary Only (Traditional CLI)

**Description**: Create a traditional standalone binary CLI tool.

**Implementation**:
```toml
[[bin]]
name = "airssys-wasm"
path = "src/main.rs"
```

**Pros**:
- ✅ Simple, well-understood pattern
- ✅ Standalone distribution out of the box
- ✅ No workspace coordination needed

**Cons**:
- ❌ Cannot be composed by airsstack without code duplication
- ❌ Users would need multiple separate binaries
- ❌ Testing requires process spawning
- ❌ Inflexible for future use cases

**Implementation Effort**: Low  
**Risk Level**: Low  
**Verdict**: ❌ Rejected - Does not meet airsstack integration requirement

---

### Option 2: Library + Binary (Hybrid)

**Description**: Provide both a library (for composition) and a binary (for standalone use).

**Implementation**:
```toml
[lib]
name = "airssys_wasm_cli"
path = "src/lib.rs"

[[bin]]
name = "airssys-wasm"
path = "src/main.rs"
```

**Pros**:
- ✅ Supports both composition and standalone usage
- ✅ Maximum flexibility
- ✅ Users can choose distribution model

**Cons**:
- ⚠️ More complex (two artifacts)
- ⚠️ Need to maintain main.rs as thin wrapper
- ⚠️ Binary distribution overhead
- ⚠️ Potential for drift between lib and bin

**Implementation Effort**: Medium  
**Risk Level**: Low  
**Verdict**: ⚠️ Possible, but adds complexity without clear benefit given airsstack is primary use case

---

### Option 3: Library Only (SELECTED)

**Description**: Implement as 100% library with zero binary components. Binaries (like airsstack) compose the library.

**Implementation**:
```toml
[package]
name = "airssys-wasm-cli"
version = "0.1.0"

[lib]
name = "airssys_wasm_cli"
path = "src/lib.rs"

# NO [[bin]] section
```

```rust
// src/lib.rs
pub mod commands {
    pub mod trust;
    // ... other commands
}
```

```rust
// airsstack/src/main.rs
use airssys_wasm_cli::commands::trust::{TrustArgs, execute as execute_trust};

#[derive(Subcommand)]
enum Commands {
    #[command(subcommand)]
    Wasm(WasmCommands),
}

#[derive(Subcommand)]
enum WasmCommands {
    Trust(TrustArgs),
    // ... other commands
}
```

**Pros**:
- ✅ Maximum reusability across binaries
- ✅ Perfect airsstack integration
- ✅ Direct function testing (no process spawning)
- ✅ Single source of truth
- ✅ Flexible for future binaries
- ✅ Smaller workspace footprint (no binary)
- ✅ Well-established pattern in Rust (clap_cargo, etc.)

**Cons**:
- ⚠️ Requires a binary to be useful (airsstack exists)
- ⚠️ Slightly more complex for developers unfamiliar with pattern

**Implementation Effort**: Low (simpler than hybrid)  
**Risk Level**: Low (well-established pattern)  
**Verdict**: ✅ **SELECTED** - Best fit for requirements

---

## Implementation

### Implementation Plan

**Phase 1: Foundation Setup** (Complete ✅)
- Created library crate with NO `[[bin]]` section
- Established command stub structure
- Set up exports in lib.rs

**Phase 2: Command Implementation** (In Progress)
- Implement each command as Clap structures
- Export from lib.rs
- Document composition pattern

**Phase 3: airsstack Integration** (Future)
- Integrate commands into airsstack binary
- Test composed CLI
- Document usage

**Phase 4: Documentation** (Ongoing)
- Write composition examples
- Document for third-party consumers
- Create usage guides

### Timeline

| Phase | Duration | Status |
|-------|----------|--------|
| Phase 1: Foundation | 1 day | ✅ Complete (2025-10-18) |
| Phase 2: Commands | 4-6 weeks | 📋 Planned (Q1 2026) |
| Phase 3: Integration | 1 week | 📋 Planned (Q2 2026) |
| Phase 4: Documentation | Ongoing | 🚧 In Progress |

### Resources Required

**Development Resources**:
- 1 developer for command implementation
- Platform team for airsstack integration
- Documentation team for user guides

**Tools**:
- Clap 4.x with derive feature
- Cargo workspace
- Integration testing tools (assert_cmd for future standalone binary if needed)

### Dependencies

**Hard Dependencies**:
- Clap 4.x (same version as airsstack)
- airssys-wasm core library (for command logic)
- Tokio (async runtime)

**Soft Dependencies**:
- airsstack binary (primary consumer)
- Shell completion support (via clap_complete)

---

## Implications

### System Impact

**Positive Impacts**:
1. **Simplified Architecture**: One library, multiple binaries (if needed)
2. **Reduced Complexity**: No binary to maintain in airssys-wasm-cli
3. **Better Testability**: Direct function calls in tests
4. **Flexible Distribution**: Easy to create binaries for different use cases

**Negative Impacts**:
1. **No Standalone Tool**: Cannot use airssys-wasm-cli directly (must go through airsstack)
   - **Mitigation**: airsstack IS the intended user interface
2. **Pattern Unfamiliarity**: Some developers may be unfamiliar with library-only CLI pattern
   - **Mitigation**: Clear documentation and examples (KNOWLEDGE-CLI-002)

### Performance Impact

**Positive**:
- ✅ Faster compilation: No binary to build in airssys-wasm-cli
- ✅ Smaller workspace artifacts
- ✅ Incremental compilation benefits (library cached)

**Neutral**:
- ➖ Runtime performance identical (library code compiled into binary)
- ➖ No performance difference vs traditional binary approach

**Negative**:
- None identified

### Security Impact

**Positive**:
- ✅ Reduced attack surface: Library has no `main()`, cannot execute directly
- ✅ Binary controls initialization and security setup
- ✅ Clear separation between presentation (CLI) and logic (core library)
- ✅ Easier to audit: Single library codebase

**Considerations**:
- ⚠️ Binaries must properly validate and sanitize inputs
- ⚠️ Security policies enforced at binary level (not library level)
- ⚠️ Dependency injection allows better security control

**Mitigations**:
- Document security requirements for binary implementers
- Provide secure defaults in execute functions
- Include security examples in documentation

### Scalability Impact

**Positive**:
- ✅ Easy to add new binaries as needs grow
- ✅ Commands can be selectively included (feature flags possible)
- ✅ Third-party tools can integrate specific commands

**Neutral**:
- ➖ No scalability difference vs traditional binary

### Maintainability Impact

**Positive**:
- ✅ Single source of truth for CLI logic
- ✅ Changes automatically propagate to all binaries
- ✅ Easier refactoring (no binary to update)
- ✅ Clearer separation of concerns

**Considerations**:
- ⚠️ Breaking changes in library affect all consuming binaries
- ⚠️ Must maintain stable API (semver important)

**Best Practices**:
- Use semantic versioning strictly
- Document breaking changes clearly
- Provide migration guides for API changes

---

## Compliance

### Workspace Standards

**Standards Applied**:
- ✅ **§2.1**: 3-Layer Import Organization - All imports follow workspace standard
- ✅ **§4.3**: Module Architecture - lib.rs only re-exports, logic in submodules
- ✅ **§5.1**: Dependency Management - Uses workspace dependencies
- ✅ **§6.1**: YAGNI Principles - No premature binary, only what's needed

**Compliance Impact**:
- This decision fully supports workspace standards
- Simplifies compliance (one less binary to audit)
- Clearer module boundaries

### Technical Debt

**Debt Created**:
- ❌ None - This is the clean architecture from the start

**Debt Resolved**:
- ✅ Prevents future debt from duplicated CLI logic
- ✅ Avoids binary maintenance burden
- ✅ Eliminates need for command synchronization

**Debt Avoided**:
- ✅ No duplication between standalone CLI and airsstack
- ✅ No binary distribution complexity
- ✅ No version drift between library and binary

---

## Monitoring and Validation

### Success Criteria

**Functional Success**:
- [ ] All CLI commands implemented as library exports
- [ ] airsstack successfully composes commands
- [ ] Users can execute commands via airsstack
- [ ] Third-party tools can reuse commands
- [ ] Tests can call execute() directly

**Quality Success**:
- [ ] Zero compilation warnings
- [ ] Zero clippy warnings
- [ ] >90% test coverage
- [ ] Clear documentation with examples
- [ ] Positive user feedback

**Integration Success**:
- [ ] airsstack integration complete
- [ ] Shell completions work
- [ ] Help text properly nested
- [ ] Error handling consistent

### Key Metrics

**Development Metrics**:
- Lines of code reused (vs duplicated)
- Test execution time (vs process spawning)
- Compilation time (library vs binary)

**Usage Metrics**:
- Number of consuming binaries
- Third-party integration count
- User satisfaction scores

**Quality Metrics**:
- Test coverage percentage
- Warning/error counts
- Issue reports related to composition

### Review Schedule

**Initial Review**: 2026-Q1 (after Phase 2 completion)
- Validate composition pattern works as expected
- Assess airsstack integration success
- Gather user feedback

**Ongoing Reviews**: Quarterly
- Check for new use cases
- Evaluate third-party adoption
- Assess need for standalone binary

**Major Review**: 2026-Q4
- Comprehensive evaluation of decision
- Consider creating standalone binary if demand exists
- Document lessons learned

---

## Risks and Mitigations

### Identified Risks

**Risk 1: User Confusion**
- **Description**: Users may expect standalone `airssys-wasm` binary
- **Impact**: Medium - User experience concern
- **Probability**: Low - airsstack is primary interface
- **Mitigation**:
  - Clear documentation that airsstack is the intended CLI
  - Redirect users to airsstack in all docs
  - Provide migration guide if coming from standalone expectation
  - Consider thin binary wrapper if demand is high

**Risk 2: Clap Version Incompatibility**
- **Description**: Different Clap versions between library and binary
- **Impact**: High - Compilation errors, runtime issues
- **Probability**: Low - Workspace dependencies enforce consistency
- **Mitigation**:
  - Use workspace dependencies for Clap
  - Pin Clap version explicitly
  - Test with multiple Clap versions
  - Document version requirements

**Risk 3: Breaking Changes in Library**
- **Description**: Library changes break consuming binaries
- **Impact**: Medium - Requires binary updates
- **Probability**: Medium - Expected during development
- **Mitigation**:
  - Follow semantic versioning strictly
  - Deprecate before removing
  - Maintain changelog with breaking changes
  - Provide migration guides

**Risk 4: Third-Party Integration Challenges**
- **Description**: Third-party developers struggle to integrate library
- **Impact**: Low - Nice-to-have feature
- **Probability**: Medium - Pattern may be unfamiliar
- **Mitigation**:
  - Comprehensive documentation (KNOWLEDGE-CLI-002)
  - Working examples in repository
  - Integration guide
  - Support channel for questions

### Contingency Plans

**If standalone binary becomes necessary**:
1. Create thin wrapper crate (1-2 days)
2. Implement main.rs that routes to library
3. Publish binary alongside library
4. Library architecture unchanged

**If Clap compatibility issues arise**:
1. Pin exact Clap version
2. Test with multiple versions
3. Document compatibility matrix
4. Consider version-specific features

**If third-party adoption is low**:
1. Reassess need for library-only pattern
2. Simplify API if too complex
3. Improve documentation
4. Host workshops/tutorials

---

## References

### Related Documents

**ADRs**:
- None (this is the first ADR for airssys-wasm-cli)

**Technical Debt**:
- None related to this decision

**Knowledge Docs**:
- **KNOWLEDGE-CLI-002**: Composable CLI Pattern - Implementation details
- **KNOWLEDGE-CLI-001**: CLI Implementation Foundation - Foundation setup

**External References**:
- [Clap Documentation](https://docs.rs/clap) - CLI framework
- [clap_cargo](https://crates.io/crates/clap_cargo) - Real-world library-only CLI example
- [clap_verbosity_flag](https://crates.io/crates/clap_verbosity_flag) - Another example
- [Cargo Book: Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)

### Workspace Standards
- **§2.1**: 3-Layer Import Organization
- **§4.3**: Module Architecture (mod.rs only re-exports)
- **§5.1**: Dependency Management (workspace deps)
- **§6.1**: YAGNI Principles

### Microsoft Rust Guidelines
- **M-DESIGN-FOR-AI**: Clear API design for composability
- **M-CANONICAL-DOCS**: Comprehensive documentation
- **M-EXAMPLES**: Extensive examples in docs

---

## History

### Status Changes
- **2025-12-18**: Status changed to **Accepted** - Architecture team approved library-only pattern

### Updates
- **2025-12-18**: Initial ADR created and approved

### Reviews
- **2025-12-18**: Reviewed by Architecture Team - Approved
- **2025-12-18**: Reviewed by Platform Team - Approved

---

**Template Version:** 1.0  
**Last Updated:** 2025-12-18

---

## Appendix: Alternative Approaches Explored

### Approach A: Proc Macro Composition
**Idea**: Use proc macros to automatically generate composition code.

**Rejected Because**:
- Unnecessary complexity
- Clap's derive macros already provide needed functionality
- Would obscure composition logic
- Harder to debug

### Approach B: Plugin System
**Idea**: Dynamic plugin loading for commands.

**Rejected Because**:
- Over-engineering for current needs
- Runtime overhead
- Security concerns
- Complexity not justified

### Approach C: Separate Binary with IPC
**Idea**: Standalone binary communicates with airsstack via IPC.

**Rejected Because**:
- Significant complexity
- Performance overhead
- Harder to maintain
- No clear benefit over library composition

---

## Appendix: Real-World Validation

### Successful Library-Only CLIs in Rust Ecosystem

1. **clap_cargo** (11,000+ downloads/day)
   - Provides reusable Cargo workspace arguments
   - Used by multiple cargo plugins
   - Well-documented, stable API

2. **clap_verbosity_flag** (30,000+ downloads/day)
   - Reusable verbosity flag handling
   - Used by hundreds of CLIs
   - Simple, effective pattern

3. **console** / **dialoguer**
   - Terminal UI libraries (not CLIs, but similar pattern)
   - Composed by many CLI tools
   - Proven composability

**Conclusion**: Library-only CLI pattern is well-established and successful in Rust ecosystem.
