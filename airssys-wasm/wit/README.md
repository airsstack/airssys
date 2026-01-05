# WIT Directory

This directory contains WebAssembly Interface Types (WIT) definitions for airssys-wasm.

## Purpose

WIT defines the interface between WASM components and the host application.

## Package Structure

Following ADR-WASM-015, this directory will contain:

### Core Packages (4)
1. `airssys:core-types@0.1.0` - Core data types (ComponentId, ComponentMessage, etc.)
2. `airssys:host-runtime@0.1.0` - Host runtime interfaces (send-message, send-request)
3. `airssys:storage@0.1.0` - Storage interfaces (get, set, delete)
4. `airssys:security@0.1.0` - Security interfaces (check-capability, request-permission)

### Extension Packages (3)
5. `airssys:http-client@0.1.0` - HTTP client interfaces
6. `airssys:timer@0.1.0` - Timer and scheduling interfaces
7. `airssys:logging@0.1.0` - Logging interfaces

## Package Naming Pattern

All WIT packages follow the pattern: `airssys:{directory}-{type}@{version}`

## Documentation

For each package:
- Document the WIT interfaces
- Document the host functions provided
- Document the component exports expected

*(WIT packages will be defined in subsequent tasks)*
