# Plugin Framework Builder Architecture - airssys-wasm-component

**Document Type:** Knowledge Documentation  
**Created:** 2025-10-03  
**Status:** Complete Architecture Foundation  
**Priority:** Critical - Macro System Foundation  

## Architecture Vision

### Core Problem Statement
Create a **procedural macro crate** that eliminates `extern "C"` complexity for engineers building WASM components. Following the serde pattern, provide macro helpers that transform clean Rust code into WASM-compatible modules with complete boilerplate generation.

### Key Inspiration: Serde + CosmWasm Pattern
**Serde Pattern**: Separate core traits from procedural macro implementation
- **Core crate (airssys-wasm)**: Traits and types only, fast compilation
- **Macro crate (airssys-wasm-component)**: Procedural macros only, optional dependency

**CosmWasm Experience**: Eliminate low-level complexity through high-level macros
- Engineers write clean business logic
- Macros handle all WASM export boilerplate
- Zero `extern "C"` exposure to developers

## Architectural Decisions

### 1. Serde-Style Separation Strategy
**Decision**: Create dedicated proc-macro crate separate from core types
**Architecture**:
```
airssys-wasm ecosystem:
├── airssys-wasm/              # Core traits and types (no proc-macros)
│   ├── Component trait
│   ├── ComponentOperation trait
│   ├── ComponentResult trait
│   └── ComponentError types
└── airssys-wasm-component/    # Procedural macros ONLY
    ├── #[component] macro
    ├── #[derive(ComponentOperation)]
    └── #[derive(ComponentResult)]
```

**Benefits**:
- **Optional macro dependency** - Core types available without proc-macro overhead
- **Faster compilation** - Core crate compiles without macro processing
- **Clear separation** - Traits vs. implementation helpers
- **Industry standard** - Proven pattern from serde ecosystem

### 2. CosmWasm-Inspired Developer Experience
**Target Experience**:
```rust
// What developers write (clean, intuitive):
use airssys_wasm::{Component, ComponentError};
use airssys_wasm_component::{component, ComponentOperation, ComponentResult};

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
    type Operation = MyOperation;
    type Result = MyResult;
    
    fn execute(&mut self, operation: Self::Operation) -> Result<Self::Result, ComponentError> {
        match operation {
            MyOperation::Process { data } => {
                self.processed_count += 1;
                Ok(MyResult::Processed { 
                    result: format!("Processed: {}", data) 
                })
            }
            MyOperation::GetStatus => {
                Ok(MyResult::Status { active: true })
            }
        }
    }
}
```

**What macros generate (completely hidden)**:
```rust
// All of this is generated automatically:
#[no_mangle]
pub extern "C" fn component_init(config_ptr: *const u8, config_len: usize) -> u64 { }

#[no_mangle]
pub extern "C" fn component_execute(operation_ptr: *const u8, operation_len: usize) -> u64 { }

#[no_mangle]
pub extern "C" fn component_metadata() -> u64 { }

#[no_mangle]
pub extern "C" fn allocate(size: usize) -> *mut u8 { }

#[no_mangle]
pub extern "C" fn deallocate(ptr: *mut u8, size: usize) { }

// Plus: Instance management, serialization, error handling, etc.
```

### 3. Macro System Architecture

#### Primary Macro: `#[component]`
**Purpose**: Transform component struct into WASM-compatible module
**Responsibilities**:
- Generate all extern "C" WASM export functions
- Create static instance management
- Handle component lifecycle (init, execute, shutdown)
- Integrate multicodec serialization
- Provide comprehensive error handling

**Implementation Strategy**:
```rust
#[proc_macro_attribute]
pub fn component(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_component_attributes(args)?;
    let struct_def = parse_struct_definition(input)?;
    
    let generated = generate_component_implementation(&args, &struct_def);
    
    quote! {
        #struct_def                    // Original struct
        #generated                     // Generated WASM exports and boilerplate
    }
}
```

#### Derive Macro Suite
**`#[derive(ComponentOperation)]`**: For operation message types
- Implement ComponentOperation trait
- Generate multicodec serialization
- Add type metadata and validation

**`#[derive(ComponentResult)]`**: For result message types
- Implement ComponentResult trait
- Generate multicodec serialization
- Add error conversion utilities

**`#[derive(ComponentConfig)]`**: For configuration types
- Implement ComponentConfig trait
- Generate validation framework
- Add default implementations

### 4. Code Generation Strategy

#### Static Instance Management Pattern
```rust
// Generated by #[component] macro
static mut COMPONENT_INSTANCE: Option<MyComponent> = None;
static COMPONENT_INIT_ONCE: std::sync::Once = std::sync::Once::new();

#[no_mangle]
pub extern "C" fn component_init(config_ptr: *const u8, config_len: usize) -> u64 {
    COMPONENT_INIT_ONCE.call_once(|| {
        unsafe {
            COMPONENT_INSTANCE = Some(MyComponent::default());
        }
    });
    
    // Deserialize config, call user init, handle errors
    // Return encoded result
}
```

#### WASM Export Generation Pattern
```rust
// Generated WASM lifecycle exports
extern "C" fn component_init(...) -> u64;      // Component initialization
extern "C" fn component_execute(...) -> u64;   // Main execution entry
extern "C" fn component_metadata() -> u64;     // Component metadata
extern "C" fn component_health() -> u64;       // Health status
extern "C" fn component_shutdown() -> u64;     // Cleanup and shutdown

// Generated memory management exports
extern "C" fn allocate(size: usize) -> *mut u8;
extern "C" fn deallocate(ptr: *mut u8, size: usize);
```

#### Result Encoding Pattern
```rust
// Generated encoding utilities
fn encode_success<T: Serialize>(value: &T) -> u64 {
    let bytes = multicodec::encode(value).unwrap();
    let ptr = bytes.as_ptr() as u64;
    let len = bytes.len() as u64;
    std::mem::forget(bytes);
    (ptr << 32) | len
}

fn encode_error(message: &str) -> u64 {
    let error_bytes = format!(r#"{{"error":"{}"}}"#, message).into_bytes();
    let ptr = error_bytes.as_ptr() as u64;
    let len = error_bytes.len() as u64;
    std::mem::forget(error_bytes);
    (ptr << 32) | len | 0x8000000000000000 // Error flag
}
```

## Implementation Architecture

### Crate Structure
```
airssys-wasm-component/
├── src/
│   ├── lib.rs                 # Public macro exports
│   ├── component.rs           # #[component] macro implementation
│   ├── derive.rs              # Derive macro implementations
│   ├── codegen.rs             # Code generation utilities
│   └── utils.rs               # Helper functions
├── tests/
│   ├── integration.rs         # Integration tests
│   └── ui/                    # Compile-time UI tests
└── Cargo.toml                 # Proc-macro crate configuration
```

### Dependency Strategy
```toml
[dependencies]
# Macro development
proc-macro2 = "1.0"
quote = "1.0"
syn = { version = "2.0", features = ["full"] }

# Core types (minimal dependency)
airssys-wasm = { path = "../airssys-wasm", default-features = false, features = ["types-only"] }

# Utilities
uuid = { version = "1.0", features = ["v4"] }

[dev-dependencies]
# UI testing
trybuild = "1.0"
```

### Testing Strategy
- **UI Tests**: Compile-time macro behavior validation with trybuild
- **Integration Tests**: End-to-end macro functionality testing
- **Error Tests**: Invalid usage and error message validation
- **Documentation Tests**: Example code validation and correctness

## Developer Experience Design

### Flexibility Levels
**Level 1: Manual Implementation** (Core airssys-wasm only)
```rust
// Manual trait implementation without macros
impl Component for MyProcessor {
    // Manual implementation + manual WASM exports
}
```

**Level 2: Macro-Assisted** (With airssys-wasm-component)
```rust
// Enhanced experience with macro helpers
#[component(name = "my-processor")]
impl Component for MyProcessor {
    // Macros generate all WASM boilerplate
}
```

**Level 3: Full Convenience** (airssys-wasm with macro re-export)
```rust
// Single dependency with convenience features
use airssys_wasm::{Component, component}; // Both traits and macros
```

### Error Message Strategy
- **Helpful compilation errors**: Clear error messages for invalid macro usage
- **Suggestion-driven**: Provide specific suggestions for fixing issues
- **Documentation links**: Link to relevant documentation for complex errors
- **Progressive disclosure**: Simple errors first, detailed information on request

## Integration Architecture

### airssys-wasm Core Integration
- **Minimal dependency**: Only core types, not full crate functionality
- **Trait compatibility**: Generated code implements standard Component trait
- **Type system alignment**: Uses core error types and context system
- **Version coordination**: Semantic versioning for compatibility

### Multicodec Integration
- **Automatic serialization**: All component data automatically encoded/decoded
- **Format flexibility**: Support for multiple multicodec formats
- **Self-describing data**: All data includes format metadata
- **Error handling**: Robust serialization error handling and recovery

### AirsSys Ecosystem Integration
- **airssys-osl compatibility**: Generated code can integrate with OS layer
- **airssys-rt compatibility**: Component-as-actor pattern support
- **Configuration management**: Seamless configuration system integration
- **Logging integration**: Structured logging support in generated code

## Performance Characteristics

### Compilation Performance
- **Separate proc-macro crate**: Faster compilation for core-only users
- **Efficient macro expansion**: Optimized code generation algorithms
- **Minimal dependencies**: Only essential proc-macro dependencies
- **Incremental compilation**: Good incremental build performance

### Runtime Performance
- **Zero runtime overhead**: All macro work happens at compile time
- **Optimal generated code**: Minimal WASM binary size impact
- **Efficient serialization**: Fast multicodec integration
- **Memory efficiency**: Smart static instance management

### WASM Characteristics
- **Small binary size**: Minimal generated code footprint
- **Universal compatibility**: Works with any WASM runtime
- **Standard compliance**: No custom WASM features required
- **Debug support**: Preserves source maps and debug information

## Security Architecture

### Memory Safety
- **WASM boundary safety**: All pointer operations validated
- **Controlled allocation**: Bounded memory allocation patterns
- **Resource cleanup**: Guaranteed cleanup on errors and shutdown
- **No buffer overflows**: Strict bounds checking on all operations

### Input Validation
- **Multicodec validation**: All input data validated during deserialization
- **Type safety**: Strong typing prevents invalid data handling
- **Configuration validation**: User-defined validation hooks
- **Error containment**: Errors isolated and properly propagated

## Future Extensions

### Planned Enhancements
- **Async support**: Future async/await integration
- **Multi-instance support**: Multiple component instances per module
- **Custom host functions**: Domain-specific extension points
- **Performance optimization**: Advanced code generation optimizations

### Ecosystem Expansion
- **Language bindings**: Support for additional WASM languages
- **Tooling integration**: IDE support and debugging tools
- **Registry integration**: Component discovery and distribution
- **Monitoring support**: Built-in observability and metrics

---

**Status**: Complete architectural foundation documented
**Next Steps**: Begin core macro implementation with #[component] macro
**Implementation Priority**: Focus on basic WASM export generation first