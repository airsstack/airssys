# Component.toml Specification

**Document Type**: REFERENCE - Configuration specification

This document provides the complete specification for `Component.toml`, the manifest file that declares component metadata and required permissions.

## Overview

`Component.toml` is a TOML-format manifest file that every airssys-wasm component must include. The framework uses this manifest to:

1. Identify component name, version, and metadata
2. Enforce capability-based security through permission declarations
3. Validate component requirements before execution

## File Structure

```toml
# Required: Component metadata
[component]
name = "string"
version = "semver"
description = "string"    # optional

# Required: Permission declarations
[permissions]
filesystem = [ ... ]      # optional
network = [ ... ]         # optional  
process = [ ... ]         # optional

# Optional: Component configuration
[config]
key = "value"
```

## Component Section

**Required**: Every manifest must have a `[component]` section.

### Fields

#### `name` (required)

Component identifier. Must be:
- Lowercase alphanumeric characters
- Hyphens allowed (not at start/end)
- 3-64 characters

**Format**: `string`

**Examples**:
```toml
name = "file-processor"     # ✅ Valid
name = "my-api-v2"          # ✅ Valid
name = "MyComponent"        # ❌ Invalid (uppercase)
name = "my_component"       # ❌ Invalid (underscore)
name = "ab"                 # ❌ Invalid (too short)
```

#### `version` (required)

Semantic version following [SemVer 2.0.0](https://semver.org/).

**Format**: `"MAJOR.MINOR.PATCH"` or `"MAJOR.MINOR.PATCH-prerelease+build"`

**Examples**:
```toml
version = "1.0.0"           # ✅ Valid
version = "0.1.0-alpha.1"   # ✅ Valid  
version = "2.3.4+build.123" # ✅ Valid
version = "1.0"             # ❌ Invalid (missing patch)
version = "v1.0.0"          # ❌ Invalid (no 'v' prefix)
```

#### `description` (optional)

Human-readable component description.

**Format**: `string` (max 500 characters)

**Example**:
```toml
description = "Processes CSV files and transforms them to JSON format"
```

#### `author` (optional)

Component author or organization.

**Format**: `string` or `array of strings`

**Examples**:
```toml
author = "Jane Developer <jane@example.com>"
# or
author = ["Jane Developer", "Engineering Team"]
```

#### `license` (optional)

License identifier (SPDX format recommended).

**Format**: `string`

**Examples**:
```toml
license = "MIT"
license = "Apache-2.0"
license = "MIT OR Apache-2.0"
```

#### `repository` (optional)

Source code repository URL.

**Format**: `string` (URL)

**Example**:
```toml
repository = "https://github.com/example/my-component"
```

### Complete Component Section Example

```toml
[component]
name = "data-pipeline"
version = "2.1.3"
description = "Transforms data from various sources into structured formats"
author = ["Data Team <data@example.com>", "John Doe"]
license = "MIT"
repository = "https://github.com/example/data-pipeline"
```

## Permissions Section

**Required**: Every manifest must have a `[permissions]` section, even if empty.

The permissions section declares all capabilities the component requires. The framework enforces these declarations at runtime.

### Empty Permissions

If your component needs no system access:

```toml
[permissions]
# Component requires no system permissions
```

### Filesystem Permissions

Declare filesystem access requirements.

#### Permission Structure

```toml
[permissions]
filesystem = [
    { action = "ACTION", path = "PATH_PATTERN" }
]
```

#### Actions

| Action | Description | Example |
|--------|-------------|---------|
| `read` | Read file contents, list directories | Reading configuration files |
| `write` | Write/create/modify files | Writing logs or output |
| `delete` | Delete files or directories | Cleaning up temporary files |
| `metadata` | Read file metadata (size, permissions, timestamps) | Checking file existence |

#### Path Patterns

| Pattern | Matches | Example |
|---------|---------|---------|
| `/exact/path` | Exact path only | `/etc/app/config.toml` |
| `/path/*` | Direct children only | `/data/*` matches `/data/file.txt` but not `/data/sub/file.txt` |
| `/path/**` | All descendants | `/data/**` matches `/data/file.txt` and `/data/sub/file.txt` |
| `~/**` | User home directory | `~/Documents/**` |

#### Examples

```toml
[permissions]
filesystem = [
    # Read configuration file
    { action = "read", path = "/etc/myapp/config.toml" },
    
    # Write logs to directory
    { action = "write", path = "/var/log/myapp/**" },
    
    # Read/write temporary files
    { action = "read", path = "/tmp/myapp/**" },
    { action = "write", path = "/tmp/myapp/**" },
    { action = "delete", path = "/tmp/myapp/**" },
    
    # Read metadata without file content access
    { action = "metadata", path = "/data/**" }
]
```

#### Security Notes

- **Principle of least privilege**: Declare only necessary paths
- **Avoid wildcards**: Use specific paths when possible
- **No implicit permissions**: Every path must be explicitly declared
- **Case-sensitive**: Path matching is case-sensitive on Unix-like systems

### Network Permissions

Declare network access requirements.

#### Permission Structure

```toml
[permissions]
network = [
    { action = "ACTION", domain = "DOMAIN_PATTERN", port = PORT }
]
```

#### Actions

| Action | Description | Required Fields | Example |
|--------|-------------|-----------------|---------|
| `connect` | Establish outbound connections | `domain`, `port` | Connect to API server |
| `resolve` | DNS resolution | `domain` | Look up IP addresses |
| `bind` | Bind to local port (server) | `port` | Run HTTP server |
| `listen` | Accept inbound connections | `port` | Accept client connections |

#### Domain Patterns

| Pattern | Matches | Example |
|---------|---------|---------|
| `example.com` | Exact domain | `api.example.com` would NOT match |
| `*.example.com` | All subdomains | `api.example.com`, `cdn.example.com` |
| `**` | Any domain (use sparingly!) | All domains |

#### Port Specification

| Format | Description | Example |
|--------|-------------|---------|
| Specific port | Single port number | `443` |
| Port range | Range of ports | (Future feature) |
| Any port | (Not supported - declare specific ports) | ❌ |

#### Examples

```toml
[permissions]
network = [
    # Connect to HTTPS API
    { action = "connect", domain = "api.example.com", port = 443 },
    
    # Connect to multiple API endpoints
    { action = "connect", domain = "api-east.example.com", port = 443 },
    { action = "connect", domain = "api-west.example.com", port = 443 },
    
    # DNS resolution for subdomains
    { action = "resolve", domain = "*.example.com" },
    
    # Run HTTP server on port 8080
    { action = "bind", port = 8080 },
    { action = "listen", port = 8080 },
    
    # Connect to database
    { action = "connect", domain = "db.internal", port = 5432 },
    { action = "resolve", domain = "*.internal" }
]
```

#### Security Notes

- **Specific domains**: Avoid `**` wildcards unless absolutely necessary
- **Explicit ports**: Declare exact ports your component needs
- **DNS resolution**: Required separately from connection permissions
- **Server components**: Both `bind` and `listen` required for accepting connections

### Process Permissions

Declare process execution requirements.

#### Permission Structure

```toml
[permissions]
process = [
    { action = "ACTION", executable = "PATH" }
]
```

#### Actions

| Action | Description | Required Fields | Example |
|--------|-------------|-----------------|---------|
| `spawn` | Execute external process | `executable` | Run build command |
| `signal` | Send signals to processes | `signal` | Send SIGTERM |
| `wait` | Wait for process completion | (implicit with spawn) | Wait for child process |

#### Executable Paths

**Format**: Absolute path to executable

| Pattern | Security | Recommendation |
|---------|----------|----------------|
| `/usr/bin/command` | ✅ High | **Use absolute paths** |
| `command` | ❌ Low | ❌ Avoid (PATH-dependent) |
| `/path/to/**` | ❌ Very Low | ❌ Never use wildcards |

#### Signal Types

Standard POSIX signals:

| Signal | Description | Use Case |
|--------|-------------|----------|
| `SIGTERM` | Graceful termination | Request process shutdown |
| `SIGKILL` | Force termination | Force process stop |
| `SIGINT` | Interrupt (Ctrl+C) | Interrupt running process |
| `SIGHUP` | Hangup | Reload configuration |
| `SIGUSR1` | User-defined signal 1 | Custom behavior |
| `SIGUSR2` | User-defined signal 2 | Custom behavior |

#### Examples

```toml
[permissions]
process = [
    # Execute specific build tools
    { action = "spawn", executable = "/usr/bin/cargo" },
    { action = "spawn", executable = "/usr/bin/rustc" },
    { action = "spawn", executable = "/usr/bin/git" },
    
    # Send signals to spawned processes
    { action = "signal", signal = "SIGTERM" },
    { action = "signal", signal = "SIGKILL" },
    
    # Execute shell scripts (be specific!)
    { action = "spawn", executable = "/opt/myapp/scripts/process.sh" }
]
```

#### Security Notes

- **Absolute paths only**: Always use full executable paths
- **No shell execution**: Components cannot execute arbitrary shell commands
- **Argument validation**: Framework does not validate process arguments
- **Signal permissions**: Declare only signals your component uses

## Config Section

**Optional**: Application-specific configuration.

```toml
[config]
# Arbitrary key-value pairs
log_level = "info"
batch_size = 100
api_timeout_seconds = 30
features = ["compression", "caching"]
```

**Notes**:
- Supports TOML types: string, integer, float, boolean, array, table
- Accessible via `ComponentConfig.data` in component code
- Framework does not interpret these values (application-defined)

## Complete Example

```toml
###############################################################################
# Component.toml - Data Processing Pipeline Component
###############################################################################

[component]
name = "data-pipeline"
version = "2.1.0"
description = "Fetches data from API, processes it, and stores results"
author = "Data Engineering Team <data@example.com>"
license = "MIT"
repository = "https://github.com/example/data-pipeline"

[permissions]
# Read configuration from application directory
filesystem = [
    { action = "read", path = "/etc/myapp/config.toml" },
    { action = "read", path = "/etc/myapp/schemas/**" },
    { action = "write", path = "/var/log/myapp/**" },
    { action = "write", path = "/data/output/**" },
    { action = "metadata", path = "/data/**" }
]

# Connect to external API and database
network = [
    { action = "connect", domain = "api.example.com", port = 443 },
    { action = "connect", domain = "db.internal", port = 5432 },
    { action = "resolve", domain = "*.example.com" },
    { action = "resolve", domain = "*.internal" }
]

# Execute data processing scripts
process = [
    { action = "spawn", executable = "/usr/bin/python3" },
    { action = "spawn", executable = "/opt/myapp/bin/process_data" },
    { action = "signal", signal = "SIGTERM" }
]

[config]
# Application-specific configuration
log_level = "info"
batch_size = 1000
api_timeout_seconds = 60
retry_count = 3
enable_compression = true
output_format = "parquet"
```

## Validation Rules

The framework validates Component.toml at component load time:

### Component Section Validation

- ✅ `name`: Required, matches pattern `^[a-z0-9][a-z0-9-]{1,62}[a-z0-9]$`
- ✅ `version`: Required, valid SemVer 2.0.0
- ✅ `description`: Optional, max 500 characters
- ✅ `author`: Optional, string or array of strings
- ✅ `license`: Optional, string (SPDX identifier recommended)
- ✅ `repository`: Optional, valid URL

### Permission Validation

- ✅ Filesystem paths: Valid absolute or home-relative paths
- ✅ Network domains: Valid domain names or patterns
- ✅ Network ports: 1-65535
- ✅ Process executables: Absolute paths only
- ✅ Signals: Valid POSIX signal names

### Common Validation Errors

```
Error: Invalid component name 'MyComponent' (must be lowercase)
Error: Invalid version 'v1.0.0' (remove 'v' prefix)
Error: Filesystem path 'relative/path' must be absolute
Error: Network port 0 is invalid (must be 1-65535)
Error: Process executable 'command' must be absolute path
Error: Unknown permission action 'execute'
```

## Best Practices

### 1. Minimal Permissions

Declare only what you need:

```toml
# ✅ GOOD - Specific permissions
[permissions]
filesystem = [
    { action = "read", path = "/etc/myapp/config.toml" }
]

# ❌ BAD - Overly broad permissions
[permissions]
filesystem = [
    { action = "read", path = "/**" }  # DON'T DO THIS
]
```

### 2. Version Management

Follow SemVer strictly:

```toml
version = "1.0.0"  # Initial stable release
version = "1.1.0"  # Added features (backward compatible)
version = "1.1.1"  # Bug fixes
version = "2.0.0"  # Breaking changes
```

### 3. Documentation

Explain permissions in comments:

```toml
[permissions]
filesystem = [
    # Configuration files - read-only
    { action = "read", path = "/etc/myapp/**" },
    
    # Output directory - component writes processed results here
    { action = "write", path = "/data/output/**" },
    
    # Temporary processing files - full access needed
    { action = "read", path = "/tmp/myapp/**" },
    { action = "write", path = "/tmp/myapp/**" },
    { action = "delete", path = "/tmp/myapp/**" }
]
```

### 4. Security Review

Before deployment, review:

- Are all permissions actually required?
- Can any paths be more specific?
- Can any network domains be restricted?
- Are executable paths absolute?

## Framework Enforcement

The framework enforces Component.toml declarations at runtime:

### Permission Checking

Every system call is checked against declared permissions:

```rust
// Component attempts to read file
filesystem::read("/etc/myapp/config.toml")

// Framework checks:
// 1. Does Component.toml declare filesystem.read permission?
// 2. Does "/etc/myapp/config.toml" match any declared path pattern?
// 3. If yes to both: ALLOW operation
// 4. If no to either: DENY operation (PermissionDenied error)
```

### Error Handling

Permission violations result in runtime errors:

```rust
// Component.toml does not declare permission for this path
let result = filesystem::read("/etc/passwd");

// Result: Err(FileError::PermissionDenied)
// Framework logs security violation
```

## Schema Reference

For tool developers, the Component.toml JSON schema is available:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Component.toml Manifest",
  "type": "object",
  "required": ["component", "permissions"],
  "properties": {
    "component": {
      "type": "object",
      "required": ["name", "version"],
      "properties": {
        "name": {
          "type": "string",
          "pattern": "^[a-z0-9][a-z0-9-]{1,62}[a-z0-9]$"
        },
        "version": {
          "type": "string",
          "pattern": "^\\d+\\.\\d+\\.\\d+(-[0-9A-Za-z.-]+)?(\\+[0-9A-Za-z.-]+)?$"
        },
        "description": {
          "type": "string",
          "maxLength": 500
        }
      }
    },
    "permissions": {
      "type": "object",
      "properties": {
        "filesystem": { "type": "array" },
        "network": { "type": "array" },
        "process": { "type": "array" }
      }
    }
  }
}
```

## Related Documentation

- [Getting Started](../implementation/getting-started.md): Tutorial including Component.toml creation
- [Component Development](../implementation/component-development.md): Permission patterns and best practices
- [WIT Interface Reference](../api/wit-interfaces.md): Interface specifications
- [Security Model](../architecture/security-model.md): Capability-based security architecture

## Version History

- **v1.0** (2025-10-26): Initial specification
- **Current**: v1.0

## Future Enhancements

Planned features for future versions:

- Port ranges in network permissions
- Environment variable declarations
- Resource limits (memory, CPU)
- Dependency declarations
- Component composition metadata
