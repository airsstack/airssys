# AirsSys Workspace Project Brief

## Vision
AirsSys is a collection of system programming components designed to facilitate the development of applications within the AirsStack ecosystem. It provides essential tools and libraries for managing system resources, handling low-level operations, and ensuring efficient performance.

## Project Objectives
1. **System Programming Excellence**: Provide robust, efficient system-level components
2. **AirsStack Integration**: Seamless integration with the broader AirsStack ecosystem
3. **Security-First Design**: Enhanced activity logging and robust security policies
4. **Cross-Platform Compatibility**: Support for multiple operating systems and architectures
5. **Performance Optimization**: Efficient resource utilization and high-performance operations

## Workspace Architecture
The AirsSys workspace consists of three core components:

### airssys-osl (OS Layer Framework)
- **Purpose**: Handle all low-level OS system programming
- **Features**: Enhanced activity logs, robust security policies
- **Responsibilities**:
  - Filesystem management
  - Process management
  - Network management
  - Utils management (external program integration: docker, gh CLI, etc.)

### airssys-rt (Runtime)
- **Purpose**: Erlang-Actor model runtime system
- **Inspiration**: BEAM runtime approach with lightweight actor model
- **Core Principles**:
  - Encapsulation: Private internal state per actor
  - Asynchronous Message Passing: Immutable messages, no shared memory
  - Mailbox and Sequential Processing: One-at-a-time message processing
- **Features**: Supervisor-based process management, high-concurrency support

### airssys-wasm (WASM Pluggable System)
- **Purpose**: WebAssembly runtime environment for secure component execution
- **Features**:
  - Lightweight WASM VM for executing WASM binaries
  - AirsSys component integration
  - Synchronous and asynchronous execution models
  - Security sandboxing with deny-by-default policy
- **Benefits**: Secure, efficient sandboxed code execution replacing isolated processes

## Technical Standards
All sub-projects must follow:
- Workspace standards enforcement (reference: workspace-standards-enforcement.instructions.md)
- Zero-warning compilation policy
- Comprehensive testing and documentation
- Security-first design principles
- Performance optimization guidelines

## Success Metrics
- Clean compilation across all components
- Comprehensive test coverage (>90%)
- Security audit compliance
- Performance benchmarks meeting target specifications
- Seamless AirsStack ecosystem integration