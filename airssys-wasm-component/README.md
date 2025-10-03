# airssys-wasm-component

Procedural macros for AirsSys WASM component development. This crate provides macro helpers that eliminate the need to write `extern "C"` functions manually, following CosmWasm's approach to simplify WASM component creation.

## Architecture

This crate implements the serde pattern - a proven architecture that separates macro implementation from core types. The separation enables:

- **Optional dependencies** - Core types available without macro compilation overhead
- **Faster builds** - Separate procedural macro compilation 
- **Flexible usage** - Manual implementation remains possible
- **Industry standard** - Follows established Rust ecosystem patterns

## Core Features

### Procedural Macros
- **`#[component]`** - Main component macro that generates WASM exports and boilerplate
- **`#[derive(ComponentOperation)]`** - Derive macro for operation message types
- **`#[derive(ComponentResult)]`** - Derive macro for result message types  
- **`#[derive(ComponentConfig)]`** - Derive macro for configuration types

### Code Generation
- **WASM export functions** - Automatic `extern "C"` function generation
- **Memory management** - Allocation and deallocation function generation
- **Serialization support** - Integration with multicodec for data encoding
- **Error handling** - Comprehensive error transformation and encoding

### Developer Experience
- **Zero boilerplate** - Focus on business logic, not WASM details
- **Type safety** - Compile-time validation and code generation
- **Modern Rust** - syn v2 compatibility with latest procedural macro patterns
- **Clear errors** - Helpful compilation error messages

## Current Status

**Implementation Status**: Foundation Complete (25% overall progress)
- **Phase 1**: Project setup and architecture foundation ✅ COMPLETED
- **Phase 2**: Actual macro logic implementation (Ready to begin)

The crate structure is complete with placeholder implementations. All code compiles successfully and is ready for actual macro functionality implementation.

## Usage Example

Add dependencies to your `Cargo.toml`:

```toml
[dependencies]
airssys-wasm = "0.1.0"
airssys-wasm-component = "0.1.0"

[lib]
crate-type = ["cdylib"]
```

Create a component (placeholder example for planned functionality):

```rust
use airssys_wasm::{Component, ComponentError};
use airssys_wasm_component::{component, ComponentOperation, ComponentResult, ComponentConfig};

#[derive(ComponentConfig)]
pub struct MyConfig {
    pub setting: String,
}

#[derive(ComponentOperation)]
pub enum MyOperation {
    Process { data: String },
    GetStatus,
}

#[derive(ComponentResult)]
pub enum MyResult {
    Processed { result: String },
    Status { active: bool },
}

#[component(name = "my-processor", version = "1.0.0")]
pub struct MyProcessor {
    processed_count: usize,
}

impl Component for MyProcessor {
    type Config = MyConfig;
    type Operation = MyOperation;
    type Result = MyResult;
    
    fn init(&mut self, config: Self::Config) -> Result<(), ComponentError> {
        // Initialization logic
        Ok(())
    }
    
    fn execute(&mut self, operation: Self::Operation) -> Result<Self::Result, ComponentError> {
        match operation {
            MyOperation::Process { data } => {
                self.processed_count += 1;
                Ok(MyResult::Processed { 
                    result: format!("Processed: {}", data) 
                })
            }
            MyOperation::GetStatus => {
                Ok(MyResult::Status { 
                    active: self.processed_count > 0 
                })
            }
        }
    }
}
```

The `#[component]` macro generates all necessary WASM exports and boilerplate code automatically.

## Generated Code

The `#[component]` macro automatically generates:

- **WASM export functions** - All required `extern "C"` functions for WASM compatibility
- **Memory management** - Allocation and deallocation functions for cross-language data passing
- **Serialization support** - Multicodec integration for data encoding/decoding
- **Error handling** - Robust error transformation and encoding
- **Component lifecycle** - Initialization, execution, and cleanup management
- **Static instance management** - Safe component instance handling

This eliminates the need to manually write any `extern "C"` code or handle WASM-specific concerns.

## Technical Foundation

### Dependencies
- **syn v2** - Modern procedural macro parsing and AST manipulation
- **quote** - Code generation and token stream creation
- **proc-macro2** - Procedural macro infrastructure
- **airssys-wasm** - Core types and trait definitions (minimal dependency)

### Project Structure
```
airssys-wasm-component/
├── src/
│   ├── lib.rs          # Macro exports and public API
│   ├── component.rs    # #[component] macro implementation
│   ├── derive.rs       # Derive macro implementations
│   ├── codegen.rs      # Code generation utilities
│   └── utils.rs        # Helper functions and parsing
├── tests/
│   ├── integration.rs  # Integration tests
│   └── ui/             # Compile-time UI tests
└── README.md
```

## Development Status

This crate follows a phased development approach:

- **Phase 1: Foundation** ✅ COMPLETED - Project structure, compilation success, workspace integration
- **Phase 2: Implementation** (Ready to begin) - Actual macro logic and code generation
- **Phase 3: Testing** (Planned) - Comprehensive UI tests and integration testing

Current implementation provides a complete foundation with placeholder functionality. All code compiles successfully and the structure is ready for actual macro implementation.

## Contributing

This crate is part of the AirsSys ecosystem. Development follows workspace standards including:

- **3-layer import organization** - Standard library, third-party, internal modules
- **Microsoft Rust Guidelines** - Production-quality Rust development standards
- **Zero warnings** - All code must compile without warnings
- **Comprehensive testing** - UI tests for macro behavior validation

## License

Licensed under either of
- Apache License, Version 2.0
- MIT License

## Related Projects

- **airssys-wasm** - Core WASM component framework and runtime
- **airssys-osl** - OS Layer Framework for system programming
- **airssys-rt** - Lightweight Erlang-Actor model runtime

This crate follows the serde pattern:

- **airssys-wasm** - Core traits and types (no proc-macros)
- **airssys-wasm-component** - Procedural macros only

This separation allows developers to choose their level of magic:
- Use core traits for manual implementation
- Use macros for convenient code generation

## Status

🚧 **Under Development** - This crate is currently being implemented as part of the AirsSys WASM Component Framework.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.