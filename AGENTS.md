# AGENTS.md

## Project Overview

**AirsSys** is a collection of system programming components for the AirsStack ecosystem, consisting of four main sub-projects:

- **airssys-osl**: OS Layer Framework for low-level system programming with security and activity logging
- **airssys-rt**: Lightweight Erlang-Actor model runtime system for high-concurrency applications  
- **airssys-wasm**: WebAssembly pluggable system for secure component execution
- **airssys-wasm-component**: Procedural macro crate for simplified WASM component development

## Memory Bank System (CRITICAL)

This project uses a **Multi-Project Memory Bank** system for context management and documentation. **You MUST read and follow the memory bank instructions** before any code work.

### Memory Bank Location
- **Instructions**: `.copilot/instructions/multi_project_memory_bank.instructions.md`
- **Memory Bank Root**: `.copilot/memory_bank/`
- **Current Context**: `.copilot/memory_bank/current_context.md`

### Before ANY Task
1. **Read current context**: Check `.copilot/memory_bank/current_context.md` for active sub-project
2. **Read workspace files**: Review all files in `.copilot/memory_bank/workspace/`
3. **Read sub-project files**: Review all core files for the active sub-project
4. **Check workspace standards**: Follow patterns in `workspace/shared_patterns.md` (§2.1-§5.1)
5. **⚠️ CRITICAL: Explore Knowledge Base**: ALWAYS review knowledge documentation before starting any task

### Active Sub-Projects
- **airssys-wasm-component**: `.copilot/memory_bank/sub_projects/airssys-wasm-component/` (Currently active - 25% complete)
- **airssys-osl**: `.copilot/memory_bank/sub_projects/airssys-osl/` (Foundation complete - 75% complete)
- **airssys-rt**: `.copilot/memory_bank/sub_projects/airssys-rt/` (Planned for Q1 2026)  
- **airssys-wasm**: `.copilot/memory_bank/sub_projects/airssys-wasm/` (Architecture complete - Q3 2026+)

### Memory Bank Commands
- `update_memory_bank [sub_project]`: Review and update memory bank files
- `add_task [sub_project] [task_name]`: Create new task with proper tracking
- `switch_context [sub_project]`: Change active sub-project context
- `show_memory_bank_summary`: Display current memory bank state
- `explore_knowledge_base [sub_project]`: Review all knowledge documentation before starting work

## Development Environment

### Setup Commands
```bash
# Initialize cargo workspace (if not exists)
cargo init --lib

# Install mdBook for documentation (one-time setup)
cargo install mdbook

# Check code quality
cargo check --workspace
cargo clippy --workspace --all-targets --all-features

# Run tests
cargo test --workspace

# Documentation development
mdbook serve docs           # Serve documentation locally
mdbook build docs           # Build documentation  
mdbook test docs            # Test code examples
```

### Project Structure
```
airssys/
├── .copilot/
│   ├── memory_bank/           # Multi-project memory bank system
│   └── instructions/          # AI agent instructions
├── airssys-osl/              # OS Layer Framework
│   └── docs/                 # mdBook documentation
├── airssys-rt/               # Runtime system
│   └── docs/                 # mdBook documentation (future)
├── airssys-wasm/             # WASM pluggable system
│   └── docs/                 # mdBook documentation (future)
└── airssys-wasm-component/   # Procedural macro crate
    └── docs/                 # mdBook documentation (future)
```

## Code Style and Standards (MANDATORY)

### Workspace Standards Compliance
**ALL code MUST follow these mandatory patterns from `workspace/shared_patterns.md`:**

#### §2.1 3-Layer Import Organization (MANDATORY)
```rust
// Layer 1: Standard library imports
use std::collections::HashMap;
use std::time::Duration;

// Layer 2: Third-party crate imports  
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

// Layer 3: Internal module imports
use crate::shared::protocol::core::McpMethod;
use crate::transport::http::config::HttpConfig;
```

#### §3.2 chrono DateTime<Utc> Standard (MANDATORY)
```rust
// ✅ CORRECT - Always use chrono DateTime<Utc>
use chrono::{DateTime, Utc};
let now = Utc::now();

// ❌ FORBIDDEN - Never use std::time for business logic
use std::time::SystemTime; // Only std::time::Instant for performance measuring
```

#### §4.3 Module Architecture (MANDATORY)
- **mod.rs files**: ONLY module declarations and re-exports, NO implementation code
- **Separation of concerns**: Clear module boundaries with proper abstractions

#### §5.1 Dependency Management (MANDATORY)
- **Workspace dependencies**: Use `[workspace.dependencies]` for version management
- **Layer-based organization**: AirsSys crates first, then core runtime, then external

#### §6.1 YAGNI Principles (MANDATORY)
- **Build only what's needed**: Implement features only when explicitly required
- **Avoid speculative generalization**: Don't build for imaginary future requirements
- **Simple solutions first**: Prefer direct solutions over elaborate architectures
- **Remove unused complexity**: Eliminate capabilities() methods and abstractions until proven necessary

#### §6.2 Avoid `dyn` Patterns (MANDATORY)
- **Prefer static dispatch**: Use generic constraints instead of trait objects
- **Type safety first**: Compile-time type checking over runtime dispatch
- **Hierarchy**: Concrete types > Generics > `dyn` traits (last resort)
```rust
// ✅ CORRECT - Use generic constraints
pub trait MyTrait<T: Operation> {
    fn process(&self, operation: T) -> Result<(), MyError>;
}

// ❌ FORBIDDEN - Avoid dyn trait objects  
pub fn process(handler: Box<dyn MyTrait>) -> Result<(), MyError>;
```

#### §6.3 Microsoft Rust Guidelines Integration (MANDATORY)
**Follow Complete Microsoft Rust Guidelines for production-quality Rust development.**

**ALL AirsSys components MUST comply with the comprehensive technical standards documented in:**
📋 **`.copilot/memory_bank/workspace/microsoft_rust_guidelines.md`** (Complete Guidelines)

**Key Mandatory Patterns:**
- **M-DESIGN-FOR-AI**: AI-optimized development with idiomatic APIs, thorough docs, strong types
- **M-DI-HIERARCHY**: Concrete types > generics > dyn traits (strict hierarchy)
- **M-AVOID-WRAPPERS**: No smart pointers in public APIs
- **M-SIMPLE-ABSTRACTIONS**: Prevent cognitive nesting, limit to 1 level deep
- **M-ERRORS-CANONICAL-STRUCTS**: Structured errors with `Backtrace` and helper methods
- **M-SERVICES-CLONE**: Services implement cheap `Clone` via `Arc<Inner>` pattern
- **M-ESSENTIAL-FN-INHERENT**: Core functionality in inherent methods
- **M-MOCKABLE-SYSCALLS**: All I/O and system calls must be mockable
- **M-UNSAFE/M-UNSOUND**: Strict safety requirements - no exceptions

**Reference Documents:**
- **Complete Standards**: `.copilot/memory_bank/workspace/microsoft_rust_guidelines.md` 
- **Original Source**: [Microsoft Rust Guidelines](https://microsoft.github.io/rust-guidelines/)
- **AI Agent Text**: [Complete Guidelines](https://microsoft.github.io/rust-guidelines/agents/all.txt)

#### §7.1 mdBook Documentation Standards (MANDATORY)
**All sub-projects MUST maintain comprehensive mdBook documentation for detailed technical documentation:**

**mdBook Features:**
- **Modern book format**: Create professional online documentation from Markdown files
- **Search functionality**: Built-in search across all documentation content
- **Responsive design**: Mobile-friendly documentation that works on all devices
- **Live reload**: Real-time preview during documentation development
- **Code highlighting**: Syntax highlighting for multiple programming languages
- **Mathematical expressions**: Support for LaTeX-style math rendering
- **Customizable themes**: Light and dark themes with customization options
- **Git integration**: Links to source repository and edit functionality

**Directory Structure Standard:**
```
{sub-project}/
├── docs/
│   ├── book.toml           # mdBook configuration
│   ├── src/
│   │   ├── SUMMARY.md      # Book navigation structure  
│   │   ├── introduction.md # Project overview and getting started
│   │   ├── architecture/   # System architecture documentation
│   │   ├── api/           # API reference documentation
│   │   ├── guides/        # User guides and tutorials
│   │   └── reference/     # Technical reference materials
│   └── book/              # Generated output (git-ignored)
```

**Development Commands:**
```bash
# Install mdBook (one-time setup)
cargo install mdbook

# Initialize new documentation (done for airssys-osl)
mdbook init docs

# Development workflow
mdbook build docs           # Build documentation
mdbook serve docs           # Serve locally for development  
mdbook test docs            # Test code examples in documentation
```

**Integration Requirements:**
- Documentation builds validated in CI pipeline
- Generated docs deployable to GitHub Pages
- Code examples in documentation automatically tested
- Documentation updates required for all public API changes

#### §7.2 Documentation Quality Standards (MANDATORY)
**All documentation MUST maintain professional software engineering standards:**

**Accuracy and Truthfulness:**
- **No assumptions**: Document only what is actually implemented or officially planned
- **No fictional content**: All examples, APIs, and features must be real or explicitly marked as planned/pending  
- **Source all claims**: Reference memory bank, code, or official specifications for all technical statements
- **Current status clarity**: Clearly indicate implementation status (completed, in-progress, planned, pending)

**Professional Tone and Language:**
- **No excessive emoticons**: Professional technical documentation avoids casual emoji usage
- **No hyperbole**: Avoid exaggerated claims like "blazingly fast", "revolutionary", "game-changing"
- **No self-promotional language**: Avoid subjective claims like "best-in-class", "cutting-edge", "industry-leading"
- **Objective terminology**: Use precise, measurable, and factual language

**Content Standards:**
```markdown
// ✅ CORRECT - Factual, sourced, professional
AirsSys OSL provides cross-platform OS abstraction following documented 
architecture specifications. Current implementation status: foundation setup phase.
Performance targets: <1ms file operations (documented in tech_context.md).

// ❌ FORBIDDEN - Assumptions, hyperbole, unsourced claims  
AirsSys OSL is the most advanced 🚀 cross-platform framework that will
revolutionize system programming! Lightning-fast performance guaranteed! ⚡
```

**Documentation Verification Requirements:**
- **Memory bank alignment**: All technical content must align with memory bank specifications
- **Implementation verification**: API examples must reflect actual or documented planned implementations
- **Status accuracy**: Current phase and capability descriptions must be factually accurate
- **No speculative features**: Do not document features without official planning documentation

### Code Quality Requirements
- **Zero warnings**: All code must compile without warnings
- **Comprehensive testing**: >90% test coverage required
- **Security-first**: All system operations must include security considerations
- **Documentation**: Comprehensive rustdoc for all public APIs

## Testing Instructions

### Test Commands
```bash
# Run all tests
cargo test --workspace

# Test specific sub-project (future)
cargo test --package airssys-osl
cargo test --package airssys-rt  
cargo test --package airssys-wasm

# Run with coverage
cargo tarpaulin --workspace --out html
```

### Test Organization
- **Unit tests**: Individual component testing within each crate
- **Integration tests**: Cross-component interaction testing in `tests/` directories
- **Security tests**: Security validation and penetration testing
- **Performance tests**: Benchmarking and performance regression testing

### Test Requirements
- All public functions must have unit tests
- Integration tests for component interactions
- Property-based testing for complex algorithms using `proptest`
- Security testing for all system operations

## Build Instructions

### Cargo Workspace Configuration
```toml
[workspace]
members = [
    "airssys-osl",
    "airssys-rt", 
    "airssys-wasm",
    "airssys-wasm-component"
]
resolver = "2"

[workspace.dependencies]
# AirsSys Foundation Crates (MUST be at top)
airssys-osl = { path = "airssys-osl" }
airssys-rt = { path = "airssys-rt" }
airssys-wasm = { path = "airssys-wasm" }
airssys-wasm-component = { path = "airssys-wasm-component" }

# Core Runtime Dependencies  
tokio = { version = "1.47", features = ["full"] }
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
thiserror = { version = "1.0" }
```

### Available Tasks (VS Code)
- `cargo check`: Code validation
- `cargo test`: Run test suite
- `cargo clippy`: Lint checking
- `cargo test airs-mcp`: Test MCP components (future)

## Security Considerations

### Security-First Development
- **Comprehensive logging**: All system operations must be logged for security audit
- **Input validation**: Validate all inputs at system boundaries
- **Principle of least privilege**: Minimal permissions by default
- **Secure defaults**: All configurations default to secure settings

### Security Review Requirements  
- All security-related code requires security team review
- External system integrations require security assessment
- Regular security audits for public releases
- Vulnerability scanning integration in CI/CD

## Documentation Requirements

### Technical Documentation System
The project uses a comprehensive technical documentation framework:

#### Documentation Types
- **Technical Debt**: Track shortcuts and compromises (`docs/debts/`)  
- **Knowledge Docs**: Architectural patterns and domain expertise (`docs/knowledges/`)
- **Architecture Decision Records**: Significant technical decisions (`docs/adr/`)

#### Documentation Triggers
- **Technical Debt**: Required for any `TODO(DEBT)` comments or standards violations
- **Knowledge Docs**: Required for complex algorithms, reusable patterns, or performance-critical code
- **ADRs**: Required for technology selections, architectural patterns, or system scalability decisions

#### Templates
- Use exact templates from `.copilot/memory_bank/templates/docs/`
- Follow naming conventions and maintain required index files
- Cross-reference related documentation appropriately

## Sub-Project Specific Instructions

### airssys-wasm-component (Procedural Macro Crate) - CURRENTLY ACTIVE
- **Phase**: Foundation complete, ready for macro implementation
- **Priority**: High - core tooling for WASM component framework
- **Focus**: Procedural macros for WASM component development, syn v2 compatibility
- **Integration**: Provides macros for airssys-wasm components, serde pattern architecture

### airssys-osl (OS Layer Framework) - FOUNDATION COMPLETE
- **Phase**: Foundation setup and architecture design
- **Priority**: Critical path - foundation for other components
- **Focus**: Security framework, activity logging, cross-platform OS abstraction
- **Integration**: Provides primitives for airssys-rt and airssys-wasm

### airssys-rt (Runtime System) - PLANNED Q1 2026
- **Phase**: Planning and architecture design  
- **Priority**: High - core runtime component
- **Focus**: Lightweight actor model, supervisor trees, message passing
- **Dependencies**: Requires airssys-osl foundation

### airssys-wasm (WASM System) - FUTURE Q3 2026+
- **Phase**: Future planning and research
- **Priority**: Medium - ecosystem completion component
- **Focus**: WebAssembly Component Model, capability-based security
- **Dependencies**: Requires airssys-osl and airssys-rt foundation

## Git and PR Instructions

### Commit Message Format
- Follow conventional commits: `type(scope): description`
- Examples: `feat(osl): add filesystem security framework`, `docs(rt): update actor model patterns`

### PR Requirements
- **Title format**: `[component] Description`
- **Pre-commit checks**: 
  ```bash
  cargo clippy --workspace --all-targets --all-features
  cargo test --workspace
  ```
- **Documentation**: Update memory bank files for significant changes
- **Standards compliance**: Verify workspace standards adherence (§2.1-§5.1)

### Branch Naming
- `feature/osl-security-framework`
- `fix/rt-message-passing`  
- `docs/wasm-component-model`

## Performance Requirements

### Target Metrics
- **airssys-osl**: <1ms file operations, <10ms process spawning
- **airssys-rt**: 10,000+ concurrent actors, <1ms message delivery
- **airssys-wasm**: <10ms component instantiation, <512KB memory per component

### Performance Testing
- Continuous benchmarking with `criterion`
- Performance regression detection in CI
- Resource usage monitoring and optimization
- Cross-platform performance validation

## Integration Testing

### Component Integration
- **airssys-osl ↔ airssys-rt**: Process management and security context integration
- **airssys-osl ↔ airssys-wasm**: Sandboxing and resource isolation integration  
- **airssys-rt ↔ airssys-wasm**: Actor-based component hosting integration

### Integration Test Strategy
- Mock external dependencies appropriately
- Test failure scenarios and recovery mechanisms
- Validate security boundaries between components
- Performance impact testing of integrated systems

## AI Agent Specific Notes

### Critical Workflow
1. **Always start with memory bank context**: Read current context and active sub-project files
2. **⚠️ MANDATORY: Knowledge Base Exploration**: Before any task implementation, MUST review all relevant knowledge documentation
3. **Follow workspace standards**: Strict adherence to §2.1-§5.1 patterns
4. **Update documentation**: Memory bank files must be updated with any significant changes
5. **Validate compliance**: Ensure zero warnings and standards compliance before completion

### Knowledge Base Exploration Protocol (MANDATORY)

**For ANY task, MUST review these knowledge sources in order:**

#### 1. Sub-Project Knowledge Documentation
- **Read ALL knowledge docs**: `.copilot/memory_bank/sub_projects/{sub-project}/docs/knowledges/`
- **Check knowledge index**: Review `_index.md` for complete knowledge catalog
- **Identify relevant patterns**: Match task requirements to documented knowledge patterns
- **Note implementation examples**: Extract concrete code examples and best practices

#### 2. Architecture Decision Records (ADRs)
- **Read ALL ADRs**: `.copilot/memory_bank/sub_projects/{sub-project}/docs/adr/`  
- **Check ADR index**: Review `_index.md` for decision history and rationale
- **Understand constraints**: Identify architectural constraints and requirements from decisions
- **Follow established patterns**: Implement according to accepted architectural decisions

#### 3. Technical Context and Patterns
- **System patterns**: Review `system_patterns.md` for established technical patterns
- **Tech context**: Check `tech_context.md` for performance targets and constraints  
- **Progress context**: Review `progress.md` for current implementation status
- **Shared patterns**: Follow workspace standards in `workspace/shared_patterns.md`

#### 4. Task-Specific Research
- **Task dependencies**: Review task files for dependencies and constraints
- **Implementation guides**: Check for task-specific implementation guidance
- **Testing requirements**: Understand testing patterns and coverage requirements
- **Documentation requirements**: Identify documentation updates needed

### Anti-Pattern: NO ASSUMPTIONS POLICY

**🚫 FORBIDDEN PRACTICES:**
- **Never assume architecture patterns** - Always reference documented decisions
- **Never assume performance characteristics** - Always check documented targets  
- **Never assume implementation approaches** - Always review knowledge documentation first
- **Never assume error handling patterns** - Always follow documented error handling strategies
- **Never assume integration patterns** - Always check documented integration approaches

**✅ REQUIRED PRACTICES:**
- **Evidence-based implementation**: All code must follow documented patterns from knowledge base
- **Reference documented decisions**: Cite specific ADRs and knowledge docs in implementation
- **Follow established examples**: Use code examples from knowledge documentation as templates
- **Validate against constraints**: Ensure implementation meets documented performance and design constraints

### Common Patterns
- **Error handling**: Use `thiserror` for structured errors with contextual information
- **Async operations**: Prefer `async/await` with Tokio runtime
- **Security logging**: All system operations require audit trail logging
- **Resource management**: Implement proper cleanup and resource lifecycle management

### Development Phases
- **Phase 1 (Current)**: airssys-osl foundation and memory bank completion
- **Phase 2 (Q1 2026)**: airssys-rt implementation and integration
- **Phase 3 (Q3 2026+)**: airssys-wasm implementation and ecosystem completion

Remember: The memory bank system is the authoritative source of project context. Always consult it before making any code changes or architectural decisions.