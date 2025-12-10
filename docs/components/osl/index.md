# Introduction

Welcome to **AirsSys OSL** (OS Layer Framework), a secure foundation for system programming that addresses the critical challenges of modern operating system interaction.

## Problem Statement

Modern system programming requires direct interaction with operating system primitives, but this comes with significant risks:

- **Security Vulnerabilities**: Direct OS calls can expose applications to security threats
- **Platform Inconsistencies**: Different operating systems provide different APIs and behaviors  
- **Audit Trail Gaps**: Most OS operations lack comprehensive logging for security monitoring
- **Error-Prone Operations**: Low-level system programming is susceptible to memory safety and resource management errors
- **Complexity Overhead**: Applications must implement security, logging, and cross-platform compatibility repeatedly

## Solution Approach

AirsSys OSL addresses these challenges by providing a secure, cross-platform, and well-instrumented abstraction layer over operating system functionality. It enables applications to perform system-level operations safely while maintaining comprehensive audit trails and enforcing security policies.

## Target Use Cases

### Secure Application Development
- Applications requiring file system operations with security guarantees
- Network services needing controlled access to system resources
- Process management with audit trails and security policies
- External tool integration with sandboxing and monitoring

### AirsStack Ecosystem Integration
- Foundation layer for airssys-rt actor system process management
- Security primitives for airssys-wasm component sandboxing
- Comprehensive logging for ecosystem-wide security monitoring
- Standardized resource management across all AirsStack components

### Enterprise System Administration
- Automated system management with comprehensive audit trails
- Secure script execution with policy enforcement
- Resource monitoring and management
- Integration with enterprise security and monitoring systems

## User Experience Goals

### Developer Experience
- **Simple API**: Intuitive interfaces that abstract OS complexity
- **Safety by Default**: Secure defaults that prevent common vulnerabilities
- **Comprehensive Documentation**: Clear examples and usage patterns
- **Excellent Error Messages**: Detailed, actionable error information
- **Performance Transparency**: Clear understanding of performance characteristics

### Operations Experience
- **Comprehensive Logging**: Detailed activity logs for all system operations
- **Policy Configuration**: Flexible security policy configuration
- **Monitoring Integration**: Easy integration with existing monitoring systems
- **Troubleshooting Support**: Rich diagnostic information for issue resolution
- **Compliance Support**: Audit trails meeting enterprise compliance requirements

## Quick Start

### Installation

Add AirsSys OSL to your `Cargo.toml`:

```toml
[dependencies]
airssys-osl = { version = "0.1", features = ["macros"] }
tokio = { version = "1.0", features = ["full"] }
```

### Basic Usage - Helper Functions API (Recommended)

The helper functions API provides the easiest way to get started with AirsSys OSL:

```rust
use airssys_osl::helpers::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Simple file operations with built-in security
    let data = b"Hello, World!".to_vec();
    write_file("/tmp/test.txt", data, "admin").await?;
    
    let content = read_file("/tmp/test.txt", "admin").await?;
    println!("Read: {}", String::from_utf8_lossy(&content));
    
    Ok(())
}
```

### Advanced Usage - Core Primitives API

For advanced use cases requiring custom middleware or fine-grained control:

```rust
use airssys_osl::core::{
    context::{SecurityContext, ExecutionContext},
    operation::OperationType,
    executor::ExecutionResult,
};

let security_context = SecurityContext::new("user123".to_string());
let execution_context = ExecutionContext::new(security_context);

assert_eq!(execution_context.principal(), "user123");
```

## Current Status

**Phase**: Core Foundation Implemented  
**Helper Functions API**: ✅ Fully functional (10 helper functions available)  
**Core Primitives**: ✅ Fully functional with comprehensive testing (60 tests passing)  
**Framework API**: ⏳ Planned for OSL-TASK-006

### What Works Today
- ✅ **Core primitives**: Full execution context, security context, and operation abstractions
- ✅ **Logger middleware**: Complete implementation with Console, File, and Tracing loggers
- ✅ **Security middleware**: ACL and RBAC implementation with audit logging  
- ✅ **Helper functions**: 10 functions for filesystem, process, and network operations
- ✅ **Comprehensive testing**: 60 tests covering all implemented functionality

### Coming Soon
- 🔄 **OSLFramework builder API**: High-level framework with middleware orchestration (OSL-TASK-006)

## Getting Help

- **Repository**: [GitHub Repository](https://github.com/airsstack/airssys)
- **Issues**: Report bugs and request features on [GitHub Issues](https://github.com/airsstack/airssys/issues)

## License

AirsSys OSL is dual-licensed under the Apache License 2.0 and MIT License.