# airssys-wasm-component Technical Context

## Technical Architecture Overview

### Procedural Macro Crate Design
**airssys-wasm-component** is implemented as a dedicated `proc-macro = true` crate that provides macro helpers for WASM component development. Following the serde pattern, it separates macro implementation from core types for optimal developer experience and compilation performance.

### Core Macro System

#### Primary Component Macro: `#[component]`
- **Purpose**: Transform clean component structs into WASM-compatible modules
- **Input**: Rust struct with Component trait implementation
- **Output**: Generated extern "C" exports, memory management, and serialization
- **Inspiration**: CosmWasm's approach but adapted for universal computing

#### Derive Macro Suite
- **`#[derive(ComponentOperation)]`**: For operation message types
- **`#[derive(ComponentResult)]`**: For result message types
- **`#[derive(ComponentConfig)]`**: For configuration types
- **Purpose**: Automatic trait implementations and multicodec serialization

### Code Generation Strategy

#### WASM Export Generation
The `#[component]` macro generates these extern "C" functions:
```c
// Component lifecycle
extern "C" component_init(config_ptr: *const u8, config_len: usize) -> u64;
extern "C" component_execute(operation_ptr: *const u8, operation_len: usize) -> u64;
extern "C" component_metadata() -> u64;
extern "C" component_health() -> u64;
extern "C" component_shutdown() -> u64;

// Memory management
extern "C" allocate(size: usize) -> *mut u8;
extern "C" deallocate(ptr: *mut u8, size: usize);
```

#### Static Instance Management
Generated code manages component instances using:
- **Static mutable instance**: `static mut COMPONENT_INSTANCE: Option<T>`
- **Initialization once**: `static COMPONENT_INIT_ONCE: std::sync::Once`
- **Thread safety**: Proper synchronization for WASM single-threaded model

#### Serialization Integration
- **Multicodec encoding**: Automatic encode/decode for all data
- **Error handling**: Robust error encoding with u64 return pattern
- **Memory safety**: Proper WASM memory management and cleanup

## Performance Characteristics

### Compilation Performance
- **Separate proc-macro crate**: Faster compilation for non-macro users
- **Minimal dependencies**: Only essential proc-macro dependencies
- **Efficient expansion**: Optimized code generation algorithms
- **Incremental compilation**: Good incremental build performance

### Runtime Performance
- **Zero runtime overhead**: All macro expansion happens at compile time
- **Optimal generated code**: Minimal WASM binary size overhead
- **Efficient memory usage**: Smart memory allocation strategies
- **Fast serialization**: Optimized multicodec integration

### WASM Binary Characteristics
- **Small binary size**: Minimal generated code overhead
- **Standard compliance**: No custom WASM features required
- **Universal compatibility**: Works with any WASM runtime
- **Debugging support**: Source maps and debug info preservation

## Security Considerations

### Memory Safety
- **Safe WASM boundaries**: All pointer handling is validated
- **Controlled allocation**: Bounded memory allocation patterns
- **Cleanup guarantees**: Proper resource cleanup on errors
- **No buffer overflows**: Strict bounds checking on all data

### Input Validation
- **Multicodec validation**: All input data validated during deserialization
- **Configuration validation**: User-defined validation hooks
- **Operation validation**: Type-safe operation handling
- **Error propagation**: Secure error information handling

### Component Isolation
- **Static instance isolation**: Each component has isolated state
- **No cross-component access**: Generated code prevents state leaking
- **Capability-based access**: Integration with airssys-wasm capability system
- **Sandboxed execution**: Standard WASM sandbox guarantees

## Integration Architecture

### airssys-wasm Core Integration
- **Trait compatibility**: Generated code implements core Component trait
- **Type system integration**: Uses core error types and context
- **Minimal dependency**: Only depends on core types, not full crate
- **Version compatibility**: Semantic versioning for compatibility

### Multicodec Integration
- **Automatic serialization**: Generated code handles all encode/decode
- **Format support**: Supports all multicodec formats
- **Self-describing data**: All data includes multicodec prefixes
- **Compatibility guarantees**: Forward/backward compatibility support

### AirsSys Ecosystem Integration
- **airssys-osl compatibility**: Generated code can use OS layer services
- **airssys-rt compatibility**: Component-as-actor integration support
- **Configuration integration**: Seamless configuration management
- **Logging integration**: Structured logging support in generated code

## Development Architecture

### Macro Implementation Structure
```rust
// Core macro processing pipeline
parse_attributes() -> ComponentConfig
parse_struct() -> ComponentDefinition
generate_wasm_exports() -> TokenStream
generate_trait_impl() -> TokenStream
generate_instance_management() -> TokenStream
```

### Code Generation Pipeline
1. **Attribute parsing**: Extract component configuration
2. **Struct analysis**: Analyze component struct definition
3. **Trait validation**: Validate Component trait implementation
4. **Export generation**: Generate all extern "C" functions
5. **Serialization integration**: Add multicodec support
6. **Error handling**: Add comprehensive error handling

### Testing Architecture
- **UI tests with trybuild**: Compile-time behavior validation
- **Integration tests**: End-to-end macro functionality
- **Error message tests**: Validation of helpful error messages
- **Generated code tests**: Validation of correct WASM output

## Constraints and Limitations

### Technical Constraints
- **Single-threaded WASM**: Generated code assumes single-threaded execution
- **Static lifetime**: Component instances must have static lifetime
- **Synchronous execution**: No async support in generated exports
- **Limited recursion**: Stack depth limitations in WASM environment

### API Constraints
- **Component trait requirement**: Users must implement Component trait
- **Serializable types**: All types must support multicodec serialization
- **Error type constraints**: Errors must be serializable and displayable
- **Configuration validation**: Config types must implement validation

### Performance Constraints
- **Binary size impact**: Generated code adds to WASM binary size
- **Compilation time**: Proc-macro compilation overhead
- **Memory usage**: Static instance memory allocation
- **Serialization overhead**: Multicodec encoding/decoding cost

## Future Architecture Considerations

### Extensibility Points
- **Custom host functions**: Support for domain-specific extensions
- **Alternative serialization**: Support for additional encoding formats
- **Async support**: Future async/await integration possibilities
- **Multi-instance support**: Support for multiple component instances

### Optimization Opportunities
- **Code generation optimization**: Smaller generated code footprint
- **Compilation speed**: Faster macro expansion algorithms
- **Runtime optimization**: More efficient generated code patterns
- **Memory optimization**: Better memory allocation strategies

### Integration Enhancements
- **Rich debugging**: Enhanced debugging support for generated code
- **Profiling integration**: Performance profiling hook points
- **Monitoring support**: Built-in metrics and observability
- **Hot reloading**: Support for development-time hot reloading

---

**Architecture Status**: Complete design, ready for implementation
**Next Technical Milestone**: Functional `#[component]` macro with basic WASM export generation