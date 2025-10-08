# airssys-osl-macros

Procedural macros for [airssys-osl](../airssys-osl) core abstractions.

## Overview

This crate provides ergonomic macros to reduce boilerplate when implementing
airssys-osl traits, particularly for custom executor implementations.

## Status

**Foundation Setup** - Basic structure in place, macro implementation pending.

See [progress tracking](../.copilot/memory_bank/sub_projects/airssys-osl-macros/progress.md) for current status.

## Planned Macros

- `#[executor]` - Generate OSExecutor<O> trait implementations (In Progress)
- `#[operation]` - Derive Operation trait (Planned)
- `#[middleware]` - Generate Middleware<O> implementations (Maybe)

## Usage (Future)

```rust
use airssys_osl::prelude::*;

#[executor]
impl MyExecutor {
    async fn file_read(&self, op: FileReadOperation, ctx: &ExecutionContext) 
        -> OSResult<ExecutionResult> 
    {
        // Custom implementation
        todo!()
    }
}
```

## Development

This is a proc-macro crate and must be compiled for the host platform.

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

## Documentation

See the [airssys-osl documentation](../airssys-osl) for complete integration details.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE))
- MIT license ([LICENSE-MIT](../LICENSE-MIT))

at your option.
