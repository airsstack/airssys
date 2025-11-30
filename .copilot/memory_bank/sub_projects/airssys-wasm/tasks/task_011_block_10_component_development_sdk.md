# [WASM-TASK-011] - Block 10: Component Development SDK

**Status:** not-started  
**Added:** 2025-10-20  
**Updated:** 2025-11-30  
**Priority:** High - Developer Experience Layer  
**Layer:** 4 - Developer Experience  
**Block:** 10 of 11  
**Estimated Effort:** 5-6 weeks  

## ⚠️ CRITICAL: Sub-Project Context

**This task implements the `airssys-wasm-component` crate** - a separate procedural macro crate in the AirsSys workspace.

### Workspace Structure Reference
```
airssys/
├── airssys-wasm/              # Core framework library (Blocks 1-9)
├── airssys-wasm-component/    # 🎯 THIS TASK (Block 10)
│   ├── Cargo.toml             # [lib] proc-macro = true
│   ├── src/
│   │   ├── lib.rs             # Macro exports
│   │   ├── component.rs       # #[component] macro
│   │   ├── derive.rs          # Derive macros
│   │   ├── codegen.rs         # Code generation
│   │   └── utils.rs           # Helpers
│   └── tests/                 # UI tests (trybuild)
└── airssys-wasm-cli/          # CLI tool (Block 11)
```

### Key Distinctions
- ✅ **This task**: Implements `airssys-wasm-component/` (procedural macros)
- ❌ **NOT this task**: Core library (`airssys-wasm/`), CLI tool (`airssys-wasm-cli/`)
- 📚 **Complete reference**: See **KNOWLEDGE-WASM-015** for full workspace architecture

### Architecture Pattern: Serde Pattern
This crate follows the proven **serde pattern** (separation of macros from core types):
- `airssys-wasm` provides core `Component` trait and types
- `airssys-wasm-component` provides procedural macros (this task)
- Developers can use macros (convenience) or implement traits manually (control)

### Current Foundation Status
- ✅ Project structure complete (25% overall)
- ✅ All modules created (lib.rs, component.rs, derive.rs, codegen.rs, utils.rs)
- ✅ Placeholder implementations compile successfully
- ✅ Workspace integration complete
- ⏳ **Ready for Phase 2**: Actual macro logic implementation

## Overview

Implement comprehensive Component Development SDK providing procedural macros (#[component], #[handler]) for zero-boilerplate component development, Component.toml validation and schema tooling, builder patterns for testing and local development, mock host functions for unit testing, multi-language examples (Rust, AssemblyScript, TinyGo), and documentation generator achieving <5-minute component creation time.

## Context

**Current State:**
- **airssys-wasm-component crate**: Foundation complete (25%)
- **Location**: `airssys/airssys-wasm-component/` directory
- **Architecture**: KNOWLEDGE-WASM-015 (Workspace Architecture) - **ESSENTIAL REFERENCE**
- **SDK Design**: KNOWLEDGE-WASM-012 (SDK Design Patterns)
- **Component.toml spec**: KNOWLEDGE-WASM-010 (manifest schema)
- **Macro foundation**: Serde pattern architecture ready
- **WIT interfaces**: Complete in Block 2 (WASM-TASK-003)

**Problem Statement:**
Component development currently requires:
1. **Boilerplate Code** - Manual WIT interface implementation
2. **Configuration Complexity** - Component.toml manual creation
3. **Testing Challenges** - No mock host functions for unit tests
4. **Language Barriers** - Limited to Rust without examples
5. **Documentation Effort** - Manual API documentation writing
6. **Debugging Difficulty** - Limited local development tools

Requirements:
- Zero-boilerplate component creation (procedural macros)
- Component.toml generation and validation
- Builder patterns for testing and development
- Mock host functions for isolated unit testing
- Multi-language examples (Rust, AssemblyScript, TinyGo)
- Automatic documentation generation
- Fast component creation (<5 minutes from idea to running)

**Why This Block Matters:**
Without SDK:
- Component development slow and error-prone
- High barrier to entry (boilerplate intimidates)
- Limited language support restricts ecosystem
- Testing difficult (requires full runtime)
- Poor developer experience

This block enables rapid component ecosystem growth.

## Objectives

### Primary Objective
Implement comprehensive Component Development SDK with procedural macros (#[component], #[handler]), Component.toml validation tooling, builder patterns for testing, mock host functions, multi-language examples (Rust, AssemblyScript, TinyGo), and documentation generator achieving <5-minute component creation time.

### Secondary Objectives
- Reduce component boilerplate to <20 lines
- Enable unit testing without runtime (<1s test execution)
- Support 3+ languages (Rust, AssemblyScript, TinyGo)
- Generate API documentation automatically
- Provide 10+ complete example components
- Achieve 95%+ developer satisfaction (survey)

## Scope

### In Scope
1. **Procedural Macros** - #[component], #[handler], #[capability]
2. **Component.toml Tooling** - Validation, generation, schema
3. **Builder Patterns** - Testing helpers, local development
4. **Mock Host Functions** - Isolated unit testing support
5. **Testing Utilities** - Assertion helpers, test fixtures
6. **Multi-Language Examples** - Rust, AssemblyScript, TinyGo
7. **Documentation Generator** - API docs from code
8. **Project Templates** - Quick-start templates (component init)

### Out of Scope
- IDE plugins/extensions (Phase 2)
- Visual component builder (Phase 2)
- Component marketplace integration (Phase 2)
- Automatic test generation (Phase 2)
- Component profiler (Phase 2, use observability)

## Implementation Plan

### Phase 1: Procedural Macro Implementation (Week 1-2)

#### Task 1.1: #[component] Macro
**Deliverables:**
- Component struct annotation macro
- WIT interface implementation generation
- Lifecycle hook generation (init, shutdown)
- Component metadata extraction
- Component macro documentation

**Success Criteria:**
- Macro generates correct WIT implementations
- Lifecycle hooks callable
- Metadata extracted correctly
- Compilation errors clear
- Documentation comprehensive

#### Task 1.2: #[handler] Macro
**Deliverables:**
- Message handler annotation macro
- Handler method signature validation
- Routing table generation
- Error handling integration
- Handler macro documentation

**Success Criteria:**
- Handlers registered automatically
- Signatures validated at compile time
- Routing efficient
- Errors handled gracefully
- Clear macro documentation

#### Task 1.3: #[capability] Macro
**Deliverables:**
- Capability declaration macro
- Compile-time capability validation
- Component.toml generation integration
- Permission pattern checking
- Capability macro documentation

**Success Criteria:**
- Capabilities declared in code
- Validation at compile time
- Component.toml updated automatically
- Patterns validated
- Documentation clear

---

### Phase 2: Component.toml Tooling (Week 2)

#### Task 2.1: Component.toml Schema Definition
**Deliverables:**
- JSON Schema for Component.toml
- Schema validation library integration
- Schema documentation
- Example Component.toml files
- Schema tooling documentation

**Success Criteria:**
- Schema comprehensive
- Validation catches errors
- Examples demonstrate features
- Documentation clear
- Tooling easy to use

#### Task 2.2: Component.toml Generation
**Deliverables:**
- Auto-generation from macro annotations
- Template-based generation (init command)
- Field completion and defaults
- Generation configuration options
- Generation tooling documentation

**Success Criteria:**
- Generation from macros works
- Templates comprehensive
- Defaults sensible
- Configuration flexible
- Tool easy to use

#### Task 2.3: Component.toml Validation CLI
**Deliverables:**
- Validation CLI command (component validate)
- Error messages with line numbers
- Warning for best practices
- Validation in CI/CD workflows
- Validation documentation

**Success Criteria:**
- Validation comprehensive
- Error messages helpful
- Warnings actionable
- CI integration smooth
- Documentation complete

---

### Phase 3: Builder Patterns and Testing Utilities (Week 3)

#### Task 3.1: ComponentBuilder Pattern
**Deliverables:**
- ComponentBuilder for testing setup
- Configuration builder methods
- Dependency injection support
- Builder pattern documentation
- Builder examples

**Success Criteria:**
- Builder intuitive to use
- Configuration flexible
- DI works correctly
- Documentation comprehensive
- Examples demonstrate usage

#### Task 3.2: Mock Host Functions
**Deliverables:**
- Mock filesystem operations
- Mock network operations
- Mock process operations
- Mock behavior configuration
- Mock documentation

**Success Criteria:**
- All host functions mockable
- Behavior configurable (success, failure, latency)
- Mocks isolated (no real I/O)
- Configuration flexible
- Testing easy

#### Task 3.3: Testing Assertion Helpers
**Deliverables:**
- Component state assertions
- Message assertions (sent, received)
- Metric assertions (counters, gauges)
- Error assertions
- Assertion documentation

**Success Criteria:**
- Assertions comprehensive
- Error messages clear
- Failures easy to debug
- API ergonomic
- Documentation with examples

---

### Phase 4: Multi-Language Support (Week 3-4)

#### Task 4.1: Rust Component Examples
**Deliverables:**
- Hello World component (minimal)
- HTTP client component (network)
- File processor component (filesystem)
- Worker pool component (concurrency)
- State machine component (storage)
- Rust examples documentation

**Success Criteria:**
- Examples cover common patterns
- Code well-commented
- Runnable out-of-box
- Performance demonstrated
- Documentation comprehensive

#### Task 4.2: AssemblyScript Component Support
**Deliverables:**
- AssemblyScript bindings for WIT interfaces
- Build tooling integration (asc compiler)
- Example components (3+ examples)
- AssemblyScript guide
- Language support documentation

**Success Criteria:**
- Bindings functional
- Build process smooth
- Examples runnable
- Guide comprehensive
- TypeScript developers can use easily

#### Task 4.3: TinyGo Component Support
**Deliverables:**
- TinyGo bindings for WIT interfaces
- Build tooling integration (tinygo compiler)
- Example components (3+ examples)
- TinyGo guide
- Language support documentation

**Success Criteria:**
- Bindings functional
- Build process smooth
- Examples runnable
- Guide comprehensive
- Go developers can use easily

---

### Phase 5: Documentation and Templates (Week 4-5)

#### Task 5.1: Documentation Generator
**Deliverables:**
- API documentation from code comments
- Component.toml documentation generation
- Capability documentation
- Cross-reference links
- Doc generator documentation

**Success Criteria:**
- Documentation auto-generated
- API docs comprehensive
- Cross-references work
- Output professional
- Generator easy to use

#### Task 5.2: Project Templates
**Deliverables:**
- Component init template (basic)
- Service component template (HTTP)
- Worker component template (background jobs)
- Library component template (utilities)
- Template documentation

**Success Criteria:**
- Templates cover common use cases
- Init command creates from template
- Templates customizable
- Code quality high
- Documentation clear

#### Task 5.3: Developer Workflow Guide
**Deliverables:**
- Complete developer workflow guide
- Quick start tutorial (<5 minutes)
- Best practices documentation
- Troubleshooting guide
- FAQ documentation

**Success Criteria:**
- Guide covers full workflow
- Tutorial gets developers started fast
- Best practices comprehensive
- Troubleshooting helpful
- FAQ answers common questions

---

### Phase 6: Advanced Features and Testing (Week 5-6)

#### Task 6.1: Component Debugging Tools
**Deliverables:**
- Debug mode flag (verbose logging)
- Trace message flow tool
- State inspection tool
- Breakpoint integration (future hook)
- Debugging documentation

**Success Criteria:**
- Debug mode helpful
- Message flow traceable
- State inspectable
- Tools intuitive
- Documentation comprehensive

#### Task 6.2: Performance Testing Utilities
**Deliverables:**
- Component load testing helpers
- Latency measurement tools
- Throughput benchmarking
- Resource usage profiling
- Performance testing documentation

**Success Criteria:**
- Load testing easy to set up
- Latency measured accurately
- Throughput benchmarked
- Resource usage visible
- Documentation with examples

#### Task 6.3: Comprehensive SDK Testing
**Deliverables:**
- Macro expansion tests
- Component.toml validation tests
- Mock host function tests
- Multi-language example tests
- SDK integration tests

**Success Criteria:**
- Test coverage >95%
- All macros tested
- Examples validate
- Cross-language tested
- CI pipeline green

---

## Success Criteria

### Definition of Done
This task is complete when:

1. ✅ **Procedural Macros Operational**
   - #[component], #[handler], #[capability] working
   - Boilerplate reduced to <20 lines
   - Compilation errors clear
   - Documentation comprehensive

2. ✅ **Component.toml Tooling Complete**
   - Schema defined and validated
   - Auto-generation from macros
   - Validation CLI working
   - CI/CD integration documented

3. ✅ **Builder Patterns Available**
   - ComponentBuilder functional
   - Testing setup intuitive
   - Mock host functions working
   - Unit tests isolated (<1s execution)

4. ✅ **Testing Utilities Ready**
   - Assertion helpers comprehensive
   - Mocks configurable
   - Test fixtures available
   - Testing ergonomic

5. ✅ **Multi-Language Support**
   - Rust examples complete (5+)
   - AssemblyScript support working (3+ examples)
   - TinyGo support working (3+ examples)
   - Language guides comprehensive

6. ✅ **Documentation Generator Functional**
   - API docs auto-generated
   - Output professional quality
   - Cross-references working
   - Generator easy to use

7. ✅ **Project Templates Available**
   - 4+ templates (basic, service, worker, library)
   - Init command creates from templates
   - Templates high quality
   - Documentation clear

8. ✅ **Testing & Documentation Complete**
   - Test coverage >95%
   - Developer guide comprehensive
   - Tutorial <5 minutes to completion
   - Developer satisfaction >95%

## Dependencies

### Upstream Dependencies
- ✅ airssys-wasm-component foundation (25% complete) - **REQUIRED** for macro implementation
- ✅ WASM-TASK-003: WIT Interface System (Block 2) - **REQUIRED** for interface generation
- ✅ KNOWLEDGE-WASM-012: SDK Design Patterns - **COMPLETE**
- ✅ KNOWLEDGE-WASM-010: Component.toml Specification - **COMPLETE**

### Downstream Dependencies (Blocks This Task)
- WASM-TASK-012: CLI Tool (Block 11) - needs SDK for component init command
- Component ecosystem - all developers depend on SDK

### External Dependencies
- syn v2 (procedural macro parsing)
- quote (code generation)
- jsonschema (Component.toml validation)
- AssemblyScript compiler (asc)
- TinyGo compiler

## Risks and Mitigations

### Risk 1: Procedural Macro Complexity
**Impact:** High - Complex macros hard to maintain  
**Probability:** Medium - Macro expansion is complex  
**Mitigation:**
- Keep macros simple and focused
- Extensive macro expansion tests
- Clear error messages from macros
- Comprehensive macro documentation

### Risk 2: Multi-Language Support Maintenance
**Impact:** Medium - Supporting 3+ languages is work  
**Probability:** High - Languages evolve  
**Mitigation:**
- Prioritize Rust (first-class support)
- Community support for other languages
- Clear language support tiers
- Automated testing for all languages

### Risk 3: Mock Host Function Accuracy
**Impact:** Medium - Inaccurate mocks mislead tests  
**Probability:** Medium - Mocks can drift from reality  
**Mitigation:**
- Share implementation with real host functions
- Test mocks against real implementation
- Document mock limitations
- Regular mock validation

### Risk 4: Developer Experience Debt
**Impact:** High - Poor DX limits ecosystem growth  
**Probability:** Medium - DX requires iteration  
**Mitigation:**
- User testing with developers
- Quick feedback iteration
- Developer satisfaction surveys
- Prioritize ergonomics over features

### Risk 5: Template Maintenance Burden
**Impact:** Low - Templates can become outdated  
**Probability:** High - Templates need updates  
**Mitigation:**
- Automated template validation
- Templates in CI pipeline
- Clear template ownership
- Community template contributions

## Progress Tracking

**Overall Status:** not-started - 0%

### Phase Breakdown
| Phase | Description | Status | Estimated Duration | Notes |
|-------|-------------|--------|-------------------|-------|
| 1 | Procedural Macro Implementation | not-started | Week 1-2 | Core SDK |
| 2 | Component.toml Tooling | not-started | Week 2 | Configuration |
| 3 | Builder Patterns and Testing | not-started | Week 3 | Testing support |
| 4 | Multi-Language Support | not-started | Week 3-4 | Ecosystem growth |
| 5 | Documentation and Templates | not-started | Week 4-5 | Developer guides |
| 6 | Advanced Features and Testing | not-started | Week 5-6 | Polish & QA |

### Subtasks
| ID | Description | Status | Updated | Notes |
|----|-------------|--------|---------|-------|
| 1.1 | #[component] Macro | not-started | - | Core macro |
| 1.2 | #[handler] Macro | not-started | - | Handler macro |
| 1.3 | #[capability] Macro | not-started | - | Capability macro |
| 2.1 | Component.toml Schema Definition | not-started | - | Schema |
| 2.2 | Component.toml Generation | not-started | - | Auto-gen |
| 2.3 | Component.toml Validation CLI | not-started | - | Validation |
| 3.1 | ComponentBuilder Pattern | not-started | - | Builder |
| 3.2 | Mock Host Functions | not-started | - | Mocks |
| 3.3 | Testing Assertion Helpers | not-started | - | Assertions |
| 4.1 | Rust Component Examples | not-started | - | Rust |
| 4.2 | AssemblyScript Component Support | not-started | - | AssemblyScript |
| 4.3 | TinyGo Component Support | not-started | - | TinyGo |
| 5.1 | Documentation Generator | not-started | - | Docs |
| 5.2 | Project Templates | not-started | - | Templates |
| 5.3 | Developer Workflow Guide | not-started | - | Guide |
| 6.1 | Component Debugging Tools | not-started | - | Debugging |
| 6.2 | Performance Testing Utilities | not-started | - | Performance |
| 6.3 | Comprehensive SDK Testing | not-started | - | Quality assurance |

## Progress Log

*No progress yet - task just created*

## Related Documentation

### ⭐ Essential Reading (MUST READ BEFORE STARTING)
- **KNOWLEDGE-WASM-015: Project Structure and Workspace Architecture** - **CRITICAL**
  - Explains the three sub-projects (airssys-wasm, airssys-wasm-component, airssys-wasm-cli)
  - Maps tasks to crates (this task = airssys-wasm-component)
  - Clarifies dependency relationships and serde pattern
  - **READ THIS FIRST** to understand context

### ADRs
- **ADR-WASM-012: SDK Macro Architecture** - (Future) Macro design decisions

### Knowledge Documentation
- **KNOWLEDGE-WASM-012: SDK Design Patterns** - Primary SDK design reference
- **KNOWLEDGE-WASM-010: Component.toml Specification** - Manifest schema
- **KNOWLEDGE-WASM-004: WIT Management Architecture** - Interface generation
- **KNOWLEDGE-WASM-001: Component Framework Architecture** - Core architecture

### Related Sub-Projects
- **airssys-wasm** (`../airssys-wasm/`) - Core library providing Component trait
- **airssys-wasm-cli** (`../airssys-wasm-cli/`) - CLI tool that uses this SDK (WASM-TASK-012)

### External References
- [Rust Procedural Macros](https://doc.rust-lang.org/book/ch19-06-macros.html)
- [serde pattern](https://github.com/serde-rs/serde) - Architecture inspiration
- [AssemblyScript](https://www.assemblyscript.org/)
- [TinyGo](https://tinygo.org/)
- [JSON Schema](https://json-schema.org/)

## Notes

**Zero-Boilerplate Goal:**
Traditional WASM component:
```rust
// Before: ~100 lines of boilerplate
impl MyComponent {
    // Manual WIT interface implementation
    // Manual lifecycle hooks
    // Manual message routing
    // Manual capability declarations
}
```

With SDK:
```rust
// After: ~15 lines total
#[component]
#[capability("filesystem:read:/data/*")]
pub struct MyComponent {
    data: String,
}

#[handler("process_data")]
fn process(&mut self, input: String) -> Result<String, Error> {
    // Business logic only
    Ok(format!("Processed: {}", input))
}
```

**Macro Design Philosophy:**
- **Simple over powerful** - Easy 80% use cases, escape hatch for 20%
- **Clear errors** - Compilation errors point to exact issue
- **Minimal magic** - Generated code inspectable (cargo expand)
- **Idiomatic Rust** - Macros feel like native language feature

**Mock Host Function Example:**
```rust
#[test]
fn test_file_processing() {
    let mut mock_fs = MockFilesystem::new();
    mock_fs.expect_read("/data/input.txt")
          .returning(Ok("test data"));
    
    let component = MyComponent::builder()
        .filesystem(mock_fs)
        .build();
    
    let result = component.process_file("/data/input.txt");
    assert_eq!(result.unwrap(), "Processed: test data");
}
```

**Multi-Language Strategy:**
- **Tier 1: Rust** - First-class support, best DX, all features
- **Tier 2: AssemblyScript, TinyGo** - Official support, good DX, most features
- **Tier 3: Community** - C, Python, etc. community-maintained bindings

**Component.toml Auto-Generation:**
Macros extract metadata and generate Component.toml:
```toml
[component]
name = "my-component"
version = "0.1.0"

[capabilities]
filesystem = ["read:/data/*"]
network = ["http:example.com"]

[dependencies]
other-component = "^1.0"
```

**Testing Philosophy:**
- **Unit tests**: Isolated with mocks (<1s execution)
- **Integration tests**: Real runtime (slower but comprehensive)
- **Component tests**: Published component tested in CI

**Documentation Generator Output:**
- API reference from rustdoc comments
- Capability documentation from macro annotations
- Usage examples from doc tests
- Component.toml reference
- Deployment guide

**Project Template Structure:**
```
component-name/
├── Component.toml
├── Cargo.toml
├── src/
│   ├── lib.rs          # Component implementation
│   └── tests.rs        # Unit tests
├── examples/
│   └── example.rs      # Usage examples
└── README.md           # Component documentation
```

**Developer Workflow (<5 minutes):**
1. `component init my-app --template service` (30 seconds)
2. Edit `src/lib.rs` - add handler (2 minutes)
3. `component build` (1 minute)
4. `component test` (30 seconds)
5. `component run --local` (30 seconds)
Total: ~4.5 minutes from idea to running component

**Performance Testing Example:**
```rust
#[bench]
fn bench_message_processing(b: &mut Bencher) {
    let component = MyComponent::new();
    b.iter(|| {
        component.handle_message(test_message());
    });
}
```

**Phase 2 Enhancements:**
- IDE plugins (VS Code, IntelliJ)
- Visual component builder (drag-drop workflow)
- Automatic test generation from examples
- Component profiler (CPU, memory)
- Component marketplace SDK integration
- Snippet library for common patterns
