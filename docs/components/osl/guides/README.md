# User Guides

This section provides guidance for working with AirsSys OSL based on the documented architecture and current development status.

*Note: AirsSys OSL is currently in foundation setup phase. These guides represent the planned approach based on the documented specifications.*

## Available Guides

- **[Getting Started](./getting-started.md)**: Installation, setup, and development environment
- **[Configuration](./configuration.md)**: Configuration management (implementation planned)
- **[Security Setup](./security-setup.md)**: Security middleware configuration (implementation planned)
- **[Logging Configuration](./logging-config.md)**: Activity logging configuration (implementation planned)
- **[Testing Guide](./testing.md)**: Testing strategies for OSL-based applications

## Current Development Status

**Phase**: Foundation Setup and Architecture Design  
**Active Task**: Core Module Foundation (OSL-TASK-001)

### Architecture Implementation Progress
1. **Core Module (`src/core/`)** - Ready for implementation
   - Essential trait abstractions only
   - Generic-first design pattern
   - Following workspace standards §2.1-§6.3

2. **Middleware Modules** - Planned next phase
   - Security middleware (consolidated approach)
   - Activity logging middleware
   - Standalone middleware modules

3. **API Layer** - Future implementation
   - High-level user APIs
   - Platform-specific implementations

## Documented Patterns

### Security-First Approach
Based on documented security requirements:
- Deny-by-default access control
- Comprehensive audit logging
- All security concerns consolidated in `middleware/security/`
- Security middleware processes all operations before execution

### Technology Stack
Based on tech context documentation:
- **Rust 2021 Edition** with MSRV 1.75+
- **Tokio** async runtime with full features
- **Tracing** for structured logging
- **Chrono** for all time operations (workspace standard §3.2)

### Cross-Platform Support
Platform priority based on documentation:
- **Linux**: Primary platform with full feature support
- **macOS**: Secondary platform with feature parity  
- **Windows**: Tertiary platform with core functionality

## Getting Started with Development

For contributors interested in the current development phase:

1. **Review Memory Bank**: Read `.copilot/memory_bank/sub_projects/airssys-osl/`
2. **Check Current Task**: Review OSL-TASK-001 implementation specifications
3. **Follow Standards**: Adhere to workspace standards §2.1-§6.3
4. **Contribute**: Follow the documented architecture and task priorities

Each guide will be expanded as the implementation progresses following the established architectural documentation.