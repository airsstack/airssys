# AGENTS.md

## Project Overview

**AirsSys** is a collection of system programming components for the AirsStack ecosystem, consisting of three main sub-projects:

- **airssys-osl**: OS Layer Framework for low-level system programming with security and activity logging
- **airssys-rt**: Lightweight Erlang-Actor model runtime system for high-concurrency applications  
- **airssys-wasm**: WebAssembly pluggable system for secure component execution

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

### Active Sub-Projects
- **airssys-osl**: `.copilot/memory_bank/sub_projects/airssys-osl/` (Currently active)
- **airssys-rt**: `.copilot/memory_bank/sub_projects/airssys-rt/` (Planned for Q1 2026)  
- **airssys-wasm**: `.copilot/memory_bank/sub_projects/airssys-wasm/` (Future - Q3 2026+)

### Memory Bank Commands
- `update_memory_bank [sub_project]`: Review and update memory bank files
- `add_task [sub_project] [task_name]`: Create new task with proper tracking
- `switch_context [sub_project]`: Change active sub-project context
- `show_memory_bank_summary`: Display current memory bank state

## Development Environment

### Setup Commands
```bash
# Initialize cargo workspace (if not exists)
cargo init --lib

# Check code quality
cargo check --workspace
cargo clippy --workspace --all-targets --all-features

# Run tests
cargo test --workspace
```

### Project Structure
```
airssys/
├── .copilot/
│   ├── memory_bank/           # Multi-project memory bank system
│   └── instructions/          # AI agent instructions
├── airssys-osl/              # OS Layer Framework  
├── airssys-rt/               # Runtime system
└── airssys-wasm/             # WASM pluggable system
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
    "airssys-wasm"
]
resolver = "2"

[workspace.dependencies]
# AirsSys Foundation Crates (MUST be at top)
airssys-osl = { path = "airssys-osl" }
airssys-rt = { path = "airssys-rt" }
airssys-wasm = { path = "airssys-wasm" }

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

### airssys-osl (OS Layer Framework) - CURRENTLY ACTIVE
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
2. **Follow workspace standards**: Strict adherence to §2.1-§5.1 patterns
3. **Update documentation**: Memory bank files must be updated with any significant changes
4. **Validate compliance**: Ensure zero warnings and standards compliance before completion

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