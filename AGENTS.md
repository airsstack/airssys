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
- **airssys-osl**: `.copilot/memory_bank/sub_projects/airssys-osl/` (Currently active - 85% complete)
- **airssys-wasm-component**: `.copilot/memory_bank/sub_projects/airssys-wasm-component/` (Foundation complete - 25% complete)
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

#### §7.3 Diátaxis Documentation Framework (MANDATORY)
**All sub-projects MUST organize documentation following the Diátaxis framework - a systematic approach to technical documentation authoring.**

**Framework Overview:**

Diátaxis identifies four distinct needs and four corresponding forms of documentation, organized around user needs:

```
                    PRACTICAL STEPS          THEORETICAL KNOWLEDGE
                    ===============          =====================
LEARNING-ORIENTED   │  TUTORIALS    │       │  EXPLANATION     │
(Study)             │  (Learning)   │       │  (Understanding) │
                    │               │       │                  │
────────────────────┼───────────────┼───────┼──────────────────┼────────
TASK-ORIENTED       │  HOW-TO       │       │  REFERENCE       │
(Work)              │  GUIDES       │       │  (Information)   │
                    │  (Goals)      │       │                  │
                    ═══════════════          =====================
```

**The Four Documentation Types:**

**1. TUTORIALS (Learning-Oriented)**
- **Purpose**: Learning experiences that take users through practical steps
- **User Need**: "I want to learn by doing something meaningful"
- **Characteristics**:
  - Practical activity with achievable goals
  - Learning-oriented, not completion-oriented
  - Teacher-student relationship (tutorial guides the learner)
  - Focus on acquisition of skills and knowledge
- **Key Principles**:
  - Show where the learner will be going (set expectations)
  - Deliver visible results early and often
  - Maintain narrative of the expected (reassure learner they're on track)
  - Point out what to notice (close learning loops)
  - Ruthlessly minimize explanation (link to it instead)
  - Focus on concrete steps, not abstractions
  - Ignore options and alternatives (stay focused)
  - Aspire to perfect reliability (every step must work)
- **Language**: "We will...", "First, do x. Now do y.", "Notice that...", "You have built..."
- **Example Structure**:
  ```markdown
  # Tutorial: Building Your First Secure File Operation
  
  In this tutorial, we will create a simple application that reads and writes
  files with security middleware. You will learn how to use the OSL framework's
  security features.
  
  ## What You'll Build
  A command-line tool that securely reads and writes configuration files.
  
  ## Step 1: Set Up Your Project
  First, create a new Rust project:
  ```
  cargo new secure-file-app
  cd secure-file-app
  ```
  
  You should see output confirming the project was created...
  ```

**2. HOW-TO GUIDES (Task-Oriented)**
- **Purpose**: Directions that guide readers through problems to achieve specific goals
- **User Need**: "I need to accomplish this specific task"
- **Characteristics**:
  - Goal-oriented and problem-focused
  - Assumes user knows what they want to achieve
  - Serves the work of already-competent users
  - Addresses real-world complexity and use-cases
- **Key Principles**:
  - Address real-world complexity (adaptable to use-cases)
  - Omit the unnecessary (practical usability over completeness)
  - Provide executable instructions (contract: if situation X, then steps Y)
  - Describe logical sequence (ordered in meaningful way)
  - Seek flow (smooth progress through user's thinking patterns)
  - Pay attention to naming (titles say exactly what guide shows)
- **Language**: "This guide shows you how to...", "If you want x, do y.", "Refer to reference guide for..."
- **Example Structure**:
  ```markdown
  # How to Configure Custom RBAC Policies
  
  This guide shows you how to create and configure custom Role-Based Access
  Control (RBAC) policies for your application.
  
  ## Prerequisites
  - Existing OSL application
  - Understanding of your application's permission requirements
  
  ## Steps
  
  ### 1. Define Your Permissions
  Identify the permissions your application needs:
  ```rust
  let read_perm = Permission::new("file:read", "Read file access");
  let write_perm = Permission::new("file:write", "Write file access");
  ```
  
  ### 2. Create Roles
  If you need role hierarchies, configure them now...
  ```

**3. REFERENCE (Information-Oriented)**
- **Purpose**: Technical descriptions of the machinery and how to operate it
- **User Need**: "I need accurate, authoritative information about this"
- **Characteristics**:
  - Information-oriented, describes the product
  - Austere and uncompromising
  - Wholly authoritative (no doubt or ambiguity)
  - Like a map of the territory
  - Structured according to the machinery itself
- **Key Principles**:
  - Describe and only describe (neutral description)
  - Adopt standard patterns (consistency aids effectiveness)
  - Respect structure of machinery (docs mirror code structure)
  - Provide examples (illustrate without distracting)
  - State facts, list features, provide warnings
- **Language**: "Class X inherits from Y...", "Sub-commands are: a, b, c...", "You must use a. Never d."
- **Example Structure**:
  ```markdown
  # API Reference: SecurityMiddleware
  
  ## Module: airssys_osl::middleware::security
  
  ### Struct: SecurityMiddleware
  
  ```rust
  pub struct SecurityMiddleware { /* fields */ }
  ```
  
  A middleware component that enforces security policies.
  
  #### Methods
  
  ##### `new()`
  ```rust
  pub fn new() -> Self
  ```
  Creates a new SecurityMiddleware instance with default configuration.
  
  **Returns**: SecurityMiddleware instance
  
  **Example**:
  ```rust
  let middleware = SecurityMiddleware::new();
  ```
  
  ##### `with_policy()`
  ```rust
  pub fn with_policy<P: SecurityPolicy>(self, policy: P) -> Self
  ```
  Adds a security policy to the middleware.
  
  **Parameters**:
  - `policy`: A type implementing SecurityPolicy trait
  
  **Returns**: Self for method chaining
  ```

**4. EXPLANATION (Understanding-Oriented)**
- **Purpose**: Discursive treatment that deepens understanding
- **User Need**: "I want to understand the context, reasoning, and implications"
- **Characteristics**:
  - Understanding-oriented, permits reflection
  - Deepens and broadens knowledge
  - Higher and wider perspective than other types
  - Makes sense to read away from the product (bath-reading documentation)
  - Brings clarity, context, and connections
- **Key Principles**:
  - Make connections (weave web of understanding)
  - Provide context (background, history, design decisions)
  - Talk about the subject (bigger picture, alternatives, why)
  - Admit opinion and perspective (discuss tradeoffs)
  - Keep closely bounded (prevent scope creep)
- **Language**: "The reason for x is...", "W is better than z because...", "Some prefer w because..."
- **Example Structure**:
  ```markdown
  # Understanding Security Context Architecture
  
  ## Background
  
  The security context architecture in AirsSys OSL emerged from the need to
  separate concerns between operation definition and security enforcement. This
  separation allows operations to declare their permission requirements without
  being coupled to specific security policy implementations.
  
  ## Design Rationale
  
  Historically, many frameworks tightly couple security checks to business logic,
  leading to scattered authorization code. We chose a middleware-based approach
  for several reasons:
  
  1. **Separation of Concerns**: Operations focus on what they do, not who can do it
  2. **Composability**: Multiple security policies can be combined
  3. **Testability**: Security logic can be tested independently
  
  ## Architectural Tradeoffs
  
  The attribute-based approach offers flexibility but introduces complexity...
  
  Some teams prefer declarative security (annotations/attributes) as it keeps
  security visible in code. Our approach favors runtime flexibility, which is
  better suited for systems with dynamic security requirements.
  
  ## Alternative Approaches
  
  Other security architectures we considered include...
  ```

**Documentation Organization Standards:**

**Directory Structure Following Diátaxis:**
```
{sub-project}/docs/src/
├── SUMMARY.md
├── introduction.md
├── tutorials/              # TUTORIALS - Learning-oriented
│   ├── getting-started.md
│   ├── first-secure-app.md
│   └── building-middleware.md
├── guides/                 # HOW-TO GUIDES - Task-oriented  
│   ├── configure-rbac.md
│   ├── custom-policies.md
│   └── integration.md
├── reference/              # REFERENCE - Information-oriented
│   ├── api/
│   │   ├── core.md
│   │   ├── middleware.md
│   │   └── operations.md
│   └── cli.md
└── explanation/            # EXPLANATION - Understanding-oriented
    ├── architecture.md
    ├── security-model.md
    └── design-decisions.md
```

**Content Placement Guidelines:**

**When to use TUTORIALS:**
- New users learning the product
- Introducing core concepts through practice
- Building confidence through success
- Teaching fundamental workflows
- Example: "Tutorial: Your First Secure File Operation"

**When to use HOW-TO GUIDES:**
- Solving specific problems
- Accomplishing particular tasks
- Real-world scenarios and use-cases
- Multiple approaches to goals
- Example: "How to Configure Custom Security Policies"

**When to use REFERENCE:**
- API documentation
- Configuration options
- Command-line interface
- Data structures and types
- Error codes and messages
- Example: "API Reference: SecurityMiddleware"

**When to use EXPLANATION:**
- Architecture and design rationale
- Conceptual overviews
- Historical context and evolution
- Comparison of approaches
- Performance characteristics
- Example: "Understanding the Security Context Architecture"

**Quality Checklist for Each Type:**

**Tutorial Quality:**
- [ ] Clear learning objective stated upfront
- [ ] Every step produces visible result
- [ ] Minimal explanation (links to explanation docs)
- [ ] Concrete examples only (no abstractions)
- [ ] Tested end-to-end reliability
- [ ] Success achievable by following exactly

**How-To Guide Quality:**
- [ ] Specific goal/problem clearly stated
- [ ] Assumes user competence
- [ ] Focuses on practical steps
- [ ] Adaptable to real-world variations
- [ ] Omits unnecessary completeness
- [ ] Title describes exactly what it shows

**Reference Quality:**
- [ ] Neutral, objective description
- [ ] Consistent structure and patterns
- [ ] Complete and accurate information
- [ ] Mirrors code/product structure
- [ ] No instruction or explanation
- [ ] Examples illustrate without teaching

**Explanation Quality:**
- [ ] Provides context and background
- [ ] Makes connections to related topics
- [ ] Discusses alternatives and tradeoffs
- [ ] Admits opinions where appropriate
- [ ] Bounded scope (doesn't try to explain everything)
- [ ] Can be read away from product

**Integration with Existing Standards:**

Diátaxis complements existing AirsSys documentation requirements:
- **§7.1 mdBook**: Technical delivery mechanism for Diátaxis content
- **§7.2 Quality Standards**: Professional tone applied across all four types
- **Memory Bank**: Technical debt, knowledge docs, ADRs map to EXPLANATION category
- **Rustdoc**: Generated API docs fit REFERENCE category
- **README files**: Mix of TUTORIAL (getting started) and REFERENCE (quick facts)

**Migration Strategy:**

For existing documentation:
1. Audit current docs and categorize by Diátaxis type
2. Identify gaps (missing tutorials, how-tos, explanations, reference)
3. Reorganize content into Diátaxis structure
4. Fill critical gaps (prioritize tutorials and how-to guides)
5. Ensure each type follows its quality principles

**Success Metrics:**

Documentation following Diátaxis should demonstrate:
- Users can get started quickly (effective tutorials)
- Users can solve real problems (useful how-to guides)
- Users can find accurate information (reliable reference)
- Users understand the system deeply (comprehensive explanation)
- Reduced "where do I find X?" questions
- Increased user confidence and satisfaction

**Further Reading:**
- Official Diátaxis site: https://diataxis.fr/
- Complete framework theory: https://diataxis.fr/theory/
- Quality guidelines: https://diataxis.fr/quality/

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

### airssys-osl (OS Layer Framework) - CURRENTLY ACTIVE
- **Phase**: API Ergonomics Foundation Complete - Framework Implementation Ready
- **Priority**: Critical path - foundation for other components
- **Focus**: Complete framework implementation with middleware pipeline and operation builders
- **Integration**: Provides primitives for airssys-rt and airssys-wasm
- **Status**: 85% complete - Ready for OSL-TASK-006 implementation

### airssys-wasm-component (Procedural Macro Crate) - FOUNDATION COMPLETE
- **Phase**: Foundation complete, ready for macro implementation
- **Priority**: High - core tooling for WASM component framework
- **Focus**: Procedural macros for WASM component development, syn v2 compatibility
- **Integration**: Provides macros for airssys-wasm components, serde pattern architecture
- **Status**: 25% complete - Ready for Phase 2 macro logic implementation

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

### PRIMARY DEVELOPMENT BEHAVIOR RULES (CRITICAL)

**⚠️ MANDATORY: These rules MUST be followed before ANY code implementation work:**

#### 🚫 Rule 1: DO NOT USE ASSUMPTIONS
- **Never assume** architecture patterns, implementation approaches, or technical decisions
- **Always verify** by reading memory bank documentation, ADRs, and knowledge base
- **Always reference** documented patterns, decisions, and constraints
- **Evidence-based only**: All implementation decisions must be backed by documented sources

#### 🚫 Rule 2: DO NOT SKIP ISSUES OR PROBLEMS - DISCUSS FIRST
- **Never ignore** compiler warnings, test failures, or integration issues
- **Never bypass** problems with temporary workarounds without discussion
- **Always discuss** architectural uncertainties, design tradeoffs, or implementation challenges
- **Seek clarification** when requirements are ambiguous or constraints conflict
- **Document problems**: Create technical debt entries for known issues requiring future resolution

#### 📚 Rule 3: ALWAYS REFER TO MEMORY BANK KNOWLEDGE
- **Knowledge directory**: Review all relevant knowledge documentation before implementation
- **ADR directory**: Check Architecture Decision Records for established patterns and constraints
- **Task detail directory**: Read task specifications, requirements, and implementation guidance
- **Related directories**: Explore technical debt, system patterns, and progress context
- **Cross-reference**: Link related documentation to understand complete context

#### ✅ Rule 4: ALWAYS FOLLOW TECHNICAL STANDARDS
- **Workspace standards**: Strict compliance with §2.1-§5.1 shared patterns (imports, modules, dependencies)
- **Microsoft Rust Guidelines**: Follow complete guidelines in `workspace/microsoft_rust_guidelines.md`
- **Code quality**: Zero warnings, comprehensive testing, proper error handling
- **Documentation**: Accurate, professional, sourced technical documentation
- **Security**: Security-first development with comprehensive logging and validation

#### 💬 Rule 5: ASK WHEN UNCONFIDENT
- **Architectural decisions**: Request guidance for significant architectural choices
- **Performance tradeoffs**: Discuss performance vs. maintainability decisions
- **Security implications**: Confirm security-sensitive implementation approaches
- **Breaking changes**: Verify API changes and backward compatibility concerns
- **Uncertainty resolution**: Always prefer asking over guessing or assuming

**These rules serve as PRIMARY GUIDELINES for AI models before detail code implementation work. Violating these rules may result in incorrect implementations, technical debt, or architectural inconsistencies.**

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