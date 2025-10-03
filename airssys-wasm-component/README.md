# airssys-wasm-component

Procedural macros for AirsSys WASM component development. This crate provides macro helpers that eliminate the need to write `extern "C"` functions manually, inspired by CosmWasm's approach.

## Features

- **`#[component]`** - Main component macro that generates WASM exports
- **`#[derive(ComponentOperation)]`** - Derive macro for operation types
- **`#[derive(ComponentResult)]`** - Derive macro for result types  
- **`#[derive(ComponentConfig)]`** - Derive macro for configuration types

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
airssys-wasm = "0.1.0"
airssys-wasm-component = "0.1.0"

[lib]
crate-type = ["cdylib"]
```

Create a component:

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
                let result = format!("Processed: {}", data);
                Ok(MyResult::Processed { result })
            }
            MyOperation::GetStatus => {
                Ok(MyResult::Status { active: true })
            }
        }
    }
}

impl Default for MyProcessor {
    fn default() -> Self {
        Self { processed_count: 0 }
    }
}
```

## What the Macros Do

The `#[component]` macro automatically generates:

- ✅ All `extern "C"` WASM export functions
- ✅ Memory management (allocate/deallocate)
- ✅ Multicodec serialization/deserialization
- ✅ Error handling and result encoding
- ✅ Component lifecycle management
- ✅ Static instance management

**You never have to write `extern "C"` code!**

## Architecture

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