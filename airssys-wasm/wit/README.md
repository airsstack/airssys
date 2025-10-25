# airssys-wasm WIT Interface Documentation

**Package:** airssys-wasm  
**Version:** 1.0.0  
**Status:** Active Development  
**Last Updated:** 2025-10-25

## Overview

This directory contains WebAssembly Interface Types (WIT) interface definitions for the airssys-wasm component framework. WIT interfaces establish type-safe contracts between WASM components and the host runtime, enabling language-agnostic component development with fine-grained capability-based security.

## Directory Structure

```
wit/
├── core/                          # Required base interfaces
│   ├── component.wit              # Universal component lifecycle
│   ├── types.wit                  # Common types and errors
│   ├── capabilities.wit           # Capability permission types
│   └── host.wit                   # Core host services
├── extensions/                     # Optional domain extensions
│   ├── filesystem.wit             # File system operations
│   ├── network.wit                # Network operations
│   └── process.wit                # Process operations
├── examples/                       # Reference implementations
│   └── basic-component.wit        # Basic component example
└── README.md                       # This file
```

## Package Naming Conventions

All WIT packages follow the pattern: `airssys:{category}-{type}@{version}`

### Core Packages (Required)
- `airssys:component-core@1.0.0` - Universal component interface (component.wit, types.wit)
- `airssys:host-core@1.0.0` - Core host services interface (host.wit)
- `airssys:host-extensions@1.0.0` - Optional host extensions (filesystem.wit, network.wit, process.wit)

### Package Version Strategy
We follow semantic versioning (MAJOR.MINOR.PATCH):

- **MAJOR**: Breaking changes (incompatible API changes)
- **MINOR**: Backward-compatible new features
- **PATCH**: Backward-compatible bug fixes

**Current Version**: 1.0.0
- Initial stable release
- Complete foundation interfaces
- Production-ready API

## Interface Organization Philosophy

### Core Interfaces (core/)
**Purpose**: Fundamental interfaces that ALL components must implement or import.

**Characteristics**:
- Minimal surface area (YAGNI principle)
- Language-agnostic design
- Stable API (breaking changes require major version bump)
- Zero optional features (everything required)

**Files**:
- `component.wit` - 8 lifecycle methods every component must export
- `types.wit` - Common types, errors, metadata structures
- `capabilities.wit` - Permission types for security model
- `host.wit` - Always-available host services (logging, messaging, time, introspection)

### Extension Interfaces (extensions/)
**Purpose**: Optional interfaces for specific capabilities (filesystem, network, process, etc.).

**Characteristics**:
- Opt-in by capability declaration
- Permission-based access control
- Domain-specific functionality
- Can evolve independently of core

**Files**:
- `filesystem.wit` - File operations (read, write, stat, delete, list)
- `network.wit` - HTTP client operations
- `process.wit` - Process spawning and environment access

### Example Interfaces (examples/)
**Purpose**: Reference implementations and documentation.

**Characteristics**:
- Illustrate best practices
- Document common patterns
- Serve as templates for component developers
- Not imported by production components

## Integration with wit-bindgen

### Binding Generation

WIT interfaces are automatically compiled to Rust bindings during `cargo build` via the `build.rs` script:

```rust
// build.rs (configured in airssys-wasm/build.rs)
wit_bindgen_rust::generate!({
    path: "wit/core",
    world: "component",
    exports: { world: true },
    with: {
        "airssys:component-core": generate,
        "airssys:host-core": generate,
    },
});
```

### Generated Bindings Location

Generated Rust code is placed in `target/debug/build/airssys-wasm-*/out/bindings.rs` and included via:

```rust
// src/bindings.rs
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
```

### Rebuild Triggers

The build system automatically regenerates bindings when:
- Any `.wit` file in `wit/` directory changes
- `build.rs` script is modified
- Cargo.toml dependencies change

Manual regeneration:
```bash
cargo clean
cargo build
```

## Interface Design Standards

### Language-Agnostic Design
**CRITICAL**: All WIT interfaces must be implementable in any WASM-compatible language.

**Principles**:
- ✅ Use WIT primitive types: `u8`, `u16`, `u32`, `u64`, `s32`, `s64`, `f32`, `f64`, `string`, `list<T>`
- ✅ Use WIT complex types: `record`, `variant`, `enum`, `option<T>`, `result<T, E>`
- ✅ Use byte arrays for serialized data: `list<u8>`
- ❌ Avoid Rust-specific patterns (lifetimes, traits, impl blocks)
- ❌ Avoid language-specific serialization formats (use multicodec)

**Example - Language-Agnostic:**
```wit
/// File metadata - works in any language
record file-metadata {
    size: u64,
    is-directory: bool,
    modified: u64,  // Unix timestamp
    readonly: bool,
}
```

**Anti-Pattern - Rust-Specific:**
```wit
// ❌ DON'T DO THIS - not language-agnostic
record file-metadata {
    path: borrowed<path>,  // Rust lifetime concept
    metadata: std-fs-metadata,  // Rust standard library type
}
```

### Capability-Based Security Integration

Every host function that accesses protected resources must document required capabilities:

```wit
/// Read file contents
/// 
/// Capability: filesystem.read for path
/// Security: Pattern matching against declared filesystem.read permissions
/// Error: Returns permission-denied if path not allowed
read-file: func(path: string) -> result<list<u8>, file-error>;
```

**Permission Check Flow**:
1. Component calls `read-file("/data/input.txt")`
2. Host function entry validates capability
3. Check Component.toml `[capabilities.filesystem] read = ["/data/**"]`
4. Glob match: `/data/input.txt` matches `/data/**` ✅
5. Allow operation and proceed
6. Audit log: `{component: "my-app", action: "filesystem.read", path: "/data/input.txt", allowed: true}`

### Performance Targets

**Interface Validation**: <1ms
- WIT interface parsing and validation
- Type compatibility checking
- World definition validation

**Function Call Overhead**: <100ns
- Host function dispatch
- Type marshalling (WIT → Rust)
- Return value marshalling (Rust → WIT)

**Security Check Overhead**: <1-5μs
- Capability permission lookup
- Pattern matching (glob/wildcard)
- Quota enforcement
- Audit logging

## Component World Definition

The `component` world defines the complete contract for all components:

```wit
world component {
    /// Components MUST export this interface
    export component-lifecycle;
    
    /// Components MUST import these interfaces
    import airssys:host-core/host-services.{host-services};
}
```

**Contract**:
- **Export**: Component implements 8 lifecycle methods (init, execute, handle-message, handle-callback, metadata, health, shutdown, supported-codecs)
- **Import**: Component can call host services (logging, messaging, time, introspection)
- **Security**: Permission-based access control enforced at runtime

## Multicodec Integration

All message data uses multicodec self-describing format per ADR-WASM-001:

```wit
/// Self-describing message with multicodec prefix
/// First bytes: varint multicodec code (e.g., 0x63 for CBOR, 0x0200 for JSON)
/// Remaining bytes: Codec-encoded payload
execute: func(
    operation: list<u8>,  // Multicodec-prefixed data
    context: execution-context
) -> result<list<u8>, execution-error>;
```

**Multicodec Table** (see KNOWLEDGE-WASM-006):
- `0x63` - CBOR (recommended for inter-component messaging)
- `0x0200` - JSON (recommended for human-readable external APIs)
- `0x50` - Protobuf (recommended for high-performance scenarios)
- `0x0201` - MessagePack (alternative compact format)
- `0x701` - Borsh (AirsSys reservation for Rust-only internal storage)
- `0x702` - Bincode (AirsSys reservation for Rust-only performance critical paths)

## Testing WIT Interfaces

### Validation

Validate WIT syntax using `wasm-tools`:

```bash
# Validate all WIT files
wasm-tools component wit airssys-wasm/wit/core/

# Expected output: Lists interfaces and types without errors
```

### Integration Testing

Test generated bindings compile correctly:

```bash
# Clean build to regenerate bindings
cargo clean
cargo build

# Verify bindings generated
ls -la target/debug/build/airssys-wasm-*/out/bindings.rs
```

### Example Component Testing

Test that example components using WIT interfaces compile:

```bash
cd airssys-wasm-component-sdk
cargo build --target wasm32-wasi --example hello_world
```

## Interface Evolution Strategy

### Adding New Interfaces (MINOR version bump)

**Allowed**:
- ✅ Add new interface files (e.g., `database.wit`)
- ✅ Add new optional functions to existing interfaces
- ✅ Add new record fields with default values
- ✅ Add new enum variants at the end

**Process**:
1. Create new `.wit` file in appropriate directory
2. Document capabilities required
3. Update this README with new interface
4. Bump MINOR version (1.0.0 → 1.1.0)
5. Test backward compatibility

### Modifying Existing Interfaces (MAJOR version bump)

**Breaking Changes**:
- ❌ Remove functions or interfaces
- ❌ Remove or rename parameters
- ❌ Change parameter types incompatibly
- ❌ Change function semantics
- ❌ Remove enum variants

**Process**:
1. Create deprecation notice in current version
2. Provide migration guide
3. Create new version with breaking changes
4. Bump MAJOR version (1.0.0 → 2.0.0)
5. Maintain compatibility layer if feasible

## Documentation Standards

All WIT interfaces must follow professional documentation standards per workspace guidelines:

**Required**:
- ✅ Function-level documentation comments
- ✅ Parameter descriptions
- ✅ Return value descriptions
- ✅ Error case documentation
- ✅ Capability requirement documentation
- ✅ Usage examples

**Forbidden** (per documentation_terminology_standards.md):
- ❌ Hyperbolic language ("blazingly fast", "revolutionary")
- ❌ Self-promotional claims ("best-in-class", "industry-leading")
- ❌ Excessive emoticons and casual language
- ❌ Assumptions about unimplemented features
- ❌ Fictional examples or APIs

**Example - Professional Documentation**:
```wit
/// Read entire file contents into memory
/// 
/// Reads the complete file at the specified path. For large files (>1MB),
/// consider using streaming APIs to avoid memory constraints.
/// 
/// Parameters:
/// - path: Absolute file path (e.g., "/data/input.txt")
/// 
/// Returns:
/// - Ok(list<u8>): File contents as byte array
/// - Err(file-error): Error if file not found, permission denied, or I/O failure
/// 
/// Capability: Requires filesystem.read permission for path
/// Performance: <1ms for files <1MB, linear with file size for larger files
/// 
/// Example:
/// ```
/// let contents = read-file("/data/config.json")?;
/// let config = parse-json(contents)?;
/// ```
read-file: func(path: string) -> result<list<u8>, file-error>;
```

## Related Documentation

### Architecture Decision Records (ADRs)
- **ADR-WASM-005**: Capability-Based Security Model - Permission system design
- **ADR-WASM-009**: Component Communication Model - Messaging patterns and interfaces
- **ADR-WASM-002**: WASM Runtime Engine Selection - Runtime integration context

### Knowledge Documentation
- **KNOWLEDGE-WASM-004**: WIT Management Architecture - Primary reference for interface design
- **KNOWLEDGE-WASM-001**: Component Framework Architecture - Overall architecture context
- **KNOWLEDGE-WASM-005**: Inter-Component Messaging Architecture - Messaging interface patterns
- **KNOWLEDGE-WASM-006**: Multiformat Strategy - Multicodec integration

### External References
- [WebAssembly Component Model](https://github.com/WebAssembly/component-model)
- [WIT IDL Specification](https://github.com/WebAssembly/component-model/blob/main/design/mvp/WIT.md)
- [wit-bindgen Documentation](https://github.com/bytecodealliance/wit-bindgen)
- [WASI Preview 2 Interfaces](https://github.com/WebAssembly/WASI/tree/main/preview2)

## Contact and Support

**Project**: airssys-wasm  
**Component**: WIT Interface System (Block 2)  
**Task**: WASM-TASK-003  
**Status**: Phase 1 Implementation

For questions, issues, or contributions related to WIT interfaces, please refer to the project documentation in `.copilot/memory_bank/sub_projects/airssys-wasm/`.

---

**Document Version**: 1.0.0  
**Last Updated**: 2025-10-25  
**Maintained By**: AirsSys Architecture Team
