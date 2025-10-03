# airssys-wasm-component Active Context

## Current Focus
**Phase:** Project Setup and Architecture Foundation  
**Status:** New sub-project created with complete architecture design  
**Priority:** High - Plugin framework builder for eliminating extern C complexity

## Strategic Vision
**airssys-wasm-component** is a dedicated procedural macro crate that provides macro helpers to eliminate `extern "C"` complexity for engineers building WASM components. Inspired by CosmWasm's approach but adapted for the universal AirsSys WASM ecosystem.

## Architecture Decisions Made

### Serde Pattern Adoption
**Decision:** Follow serde's separation pattern with dedicated macro crate
**Rationale:**
- **Flexible dependency model** - Users choose their level of magic
- **Faster compilation** - Core crate compiles without proc-macro overhead
- **Clear architecture** - Separation between traits and implementation helpers
- **Industry standard** - Proven pattern used by major crates

### Core Design Principles
- **Eliminate extern "C"** - Engineers write clean Rust code, macros handle WASM details
- **CosmWasm-inspired** - Similar developer experience but for universal computing
- **Optional enhancement** - Developers can still implement traits manually
- **Zero lock-in** - Compiles to standard WASM without runtime dependencies

## Project Structure Created
```
airssys-wasm-component/
├── Cargo.toml                 # Proc-macro crate configuration
├── src/
│   ├── lib.rs                 # Macro exports and documentation
│   ├── component.rs           # #[component] macro implementation
│   ├── derive.rs              # Derive macros (ComponentOperation, etc.)
│   ├── codegen.rs             # Code generation utilities
│   └── utils.rs               # Helper functions
├── tests/
│   ├── integration.rs         # Integration tests
│   └── ui/                    # Compile-time UI tests
└── README.md                  # Crate documentation
```

## Macro System Design

### Primary Macros
1. **`#[component]`** - Main component macro that generates WASM exports
2. **`#[derive(ComponentOperation)]`** - For operation message types
3. **`#[derive(ComponentResult)]`** - For result message types
4. **`#[derive(ComponentConfig)]`** - For configuration types

### Developer Experience Goal
**What Engineers Write (Clean):**
```rust
#[component(name = "my-processor")]
pub struct MyProcessor {
    state: ProcessorState,
}

impl Component for MyProcessor {
    fn init(&mut self, config: MyConfig) -> Result<(), ComponentError> {
        // Clean initialization logic
    }
    
    fn execute(&mut self, operation: MyOperation) -> Result<MyResult, ComponentError> {
        // Clean business logic
    }
}
```

**What Gets Generated (Hidden):**
- All extern "C" WASM export functions
- Memory management (allocate/deallocate)
- Multicodec serialization/deserialization
- Error handling and result encoding
- Component lifecycle management

## Current Work Items
1. **Setup Complete** ✅ - Project structure and memory bank created
2. **Core Macro Implementation** - Implement #[component] macro
3. **Derive Macros** - Implement trait derive macros
4. **Code Generation** - Complete WASM export generation
5. **Testing Framework** - UI tests and integration tests

## Next Steps
1. **Implement #[component] macro** - Core functionality for WASM export generation
2. **Implement derive macros** - ComponentOperation, ComponentResult, ComponentConfig
3. **Create test suite** - Comprehensive macro testing with trybuild
4. **Documentation** - Complete macro usage guides and examples

## Integration with AirsSys Ecosystem
- **Dependency on airssys-wasm** - Uses core traits and types
- **Follows workspace standards** - Consistent with AirsSys coding patterns
- **Memory bank integration** - Complete documentation and decision tracking

## Context for Future Sessions
- Project structure established following serde pattern
- Architecture designed for CosmWasm-like developer experience
- Focus on macro implementation for eliminating extern "C" complexity
- Ready for core macro development and implementation

## Implementation Priorities
1. **#[component] macro** - Primary macro for WASM export generation
2. **Derive macros** - Supporting macros for trait implementations
3. **Testing framework** - Robust macro validation
4. **Documentation** - Complete usage guides and examples