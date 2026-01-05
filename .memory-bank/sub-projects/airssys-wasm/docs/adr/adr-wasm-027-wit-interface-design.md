# ADR-WASM-027: WIT Interface Design

**ADR ID:** ADR-WASM-027  
**Created:** 2026-01-05  
**Status:** Accepted  
**Deciders:** Architecture Team  
**Category:** Interface Design / WIT Specification  
**Parent:** [ADR-WASM-026](adr-wasm-026-implementation-roadmap-clean-slate-rebuild.md) (Phase 1)

---

## Title

WIT Interface Design for airssys-wasm Component Model

---

## Context

This ADR defines the complete WIT interface specifications for the airssys-wasm component framework. WIT (WebAssembly Interface Types) defines the contract between host runtime and guest components.

### Design Principles

1. **Single Package**: All interfaces in `airssys:core@1.0.0` (Component Model v0.1 constraint)
2. **Type Reuse**: Use `use types.{...}` statements for cross-interface type sharing
3. **Bidirectional Contract**: Guest exports (component-lifecycle) + Host imports (messaging, services, storage)
4. **Component Model Compliant**: Compatible with wasmtime component model

### References

- [KNOWLEDGE-WASM-013](../knowledges/knowledge-wasm-013-core-wit-package-structure.md): WIT Package Structure
- [DEBT-WASM-003](../debts/debt-wasm-003-component-model-v0.1-type-import-limitation.md): v0.1 Limitations

---

## Decision

### WIT Directory Structure

```
wit/
├── core/                              # Package: airssys:core@1.0.0
│   ├── types.wit                      # Layer 0: Foundation types
│   ├── errors.wit                     # Layer 0B: Error types
│   ├── capabilities.wit               # Layer 1: Security/permissions
│   ├── component-lifecycle.wit        # Layer 2: Component contract (GUEST EXPORTS)
│   ├── host-messaging.wit             # Layer 3A: Messaging (HOST PROVIDES)
│   ├── host-services.wit              # Layer 3B: Services (HOST PROVIDES)
│   ├── storage.wit                    # Layer 3C: Storage (HOST PROVIDES)
│   └── world.wit                      # World definition
└── deps.toml                          # Package configuration
```

---

## Interface Specifications

### types.wit (Layer 0 - Foundation)

```wit
package airssys:core@1.0.0;

/// Foundation types - source of truth for all interfaces
interface types {
    /// Unique component identifier
    record component-id {
        namespace: string,
        name: string,
        instance: string,
    }

    /// Component handle (opaque reference for runtime)
    type component-handle = u64;

    /// Correlation ID for request-response patterns
    type correlation-id = string;

    /// Request ID (alias for consistency)
    type request-id = string;

    /// Message payload (raw bytes, typically multicodec-encoded)
    type message-payload = list<u8>;

    /// Timestamp with high precision
    record timestamp {
        seconds: u64,
        nanoseconds: u32,
    }

    /// Message metadata
    record message-metadata {
        correlation-id: option<correlation-id>,
        reply-to: option<component-id>,
        timestamp: timestamp,
        content-type: option<string>,
    }

    /// Complete message envelope
    record component-message {
        sender: component-id,
        payload: message-payload,
        metadata: message-metadata,
    }

    /// Resource limits for component execution
    record resource-limits {
        max-memory-bytes: u64,
        max-execution-time-ms: u64,
        max-fuel: option<u64>,
    }

    /// Component configuration during initialization
    record component-config {
        env-vars: list<tuple<string, string>>,
        config-data: option<message-payload>,
        resource-limits: resource-limits,
    }

    /// Log levels
    enum log-level {
        trace,
        debug,
        info,
        warn,
        error,
    }

    /// Health status
    enum health-status {
        healthy,
        degraded,
        unhealthy,
        unknown,
    }

    /// Execution status
    enum execution-status {
        success,
        failed,
        timeout,
        cancelled,
    }
}
```

---

### errors.wit (Layer 0B - Errors)

```wit
package airssys:core@1.0.0;

/// Error types for all operations
interface errors {
    use types.{correlation-id, component-id};

    /// WASM execution errors
    variant wasm-error {
        component-not-found(string),
        instantiation-failed(string),
        export-not-found(string),
        timeout,
        resource-limit-exceeded(string),
        invalid-component(string),
        runtime-error(string),
    }

    /// Component lifecycle errors
    variant component-error {
        initialization-failed(string),
        already-initialized,
        not-initialized,
        shutdown-failed(string),
        invalid-state(string),
    }

    /// Security-related errors
    variant security-error {
        capability-denied(string),
        policy-violation(string),
        invalid-context(string),
        permission-denied(string),
    }

    /// Messaging errors
    variant messaging-error {
        delivery-failed(string),
        correlation-timeout(correlation-id),
        invalid-message(string),
        queue-full,
        target-not-found(component-id),
    }

    /// Storage errors
    variant storage-error {
        not-found(string),
        already-exists(string),
        quota-exceeded,
        invalid-key(string),
        io-error(string),
    }

    /// Execution errors (for RPC operations)
    variant execution-error {
        invalid-operation(string),
        operation-failed(string),
        unsupported-codec(string),
        serialization-error(string),
    }
}
```

---

### capabilities.wit (Layer 1 - Security)

```wit
package airssys:core@1.0.0;

/// Capability-based security model
interface capabilities {
    use types.{component-id};

    /// Filesystem permission
    record filesystem-permission {
        action: filesystem-action,
        path-pattern: string,
    }

    /// Filesystem actions
    enum filesystem-action {
        read,
        write,
        delete,
        list-dir,
    }

    /// Network permission
    record network-permission {
        action: network-action,
        host-pattern: string,
        port: option<u16>,
    }

    /// Network actions
    enum network-action {
        outbound,
        inbound,
    }

    /// Storage permission
    record storage-permission {
        action: storage-action,
        namespace-pattern: string,
    }

    /// Storage actions
    enum storage-action {
        read,
        write,
        delete,
    }

    /// Messaging permission
    record messaging-permission {
        action: messaging-action,
        target-pattern: string,
    }

    /// Messaging actions
    enum messaging-action {
        send,
        request,
        broadcast,
    }

    /// Complete permission set requested by component
    record requested-permissions {
        filesystem: list<filesystem-permission>,
        network: list<network-permission>,
        storage: list<storage-permission>,
        messaging: list<messaging-permission>,
    }

    /// Capability grant result
    record capability-grant {
        component: component-id,
        permissions: requested-permissions,
        granted-at: u64,
        expires-at: option<u64>,
    }
}
```

---

### component-lifecycle.wit (Layer 2 - Guest Exports)

```wit
package airssys:core@1.0.0;

/// Guest-implemented interface - components MUST export this
interface component-lifecycle {
    use types.{component-config, component-message, message-payload, health-status};
    use errors.{component-error, wasm-error};

    /// Initialize component with configuration
    initialize: func(config: component-config) -> result<_, component-error>;

    /// Handle incoming message (fire-and-forget pattern)
    /// Returns optional response payload
    handle-message: func(msg: component-message) -> result<option<message-payload>, wasm-error>;

    /// Handle callback response (request-response pattern)
    /// Called when response to previous request arrives
    handle-callback: func(msg: component-message) -> result<_, wasm-error>;

    /// Get component metadata
    metadata: func() -> component-metadata;

    /// Health check
    health: func() -> health-status;

    /// Graceful shutdown and cleanup
    shutdown: func() -> result<_, component-error>;

    /// Component metadata
    record component-metadata {
        name: string,
        version: string,
        description: string,
        author: string,
        license: string,
        supported-operations: list<string>,
        stateful: bool,
    }
}
```

---

### host-messaging.wit (Layer 3A - Host Provides)

```wit
package airssys:core@1.0.0;

/// Host-implemented messaging interface
interface host-messaging {
    use types.{component-id, message-payload, correlation-id, request-id};
    use errors.{messaging-error};

    /// Send fire-and-forget message to another component
    send: func(target: component-id, payload: message-payload) -> result<_, messaging-error>;

    /// Send request expecting callback response
    /// Returns correlation-id for tracking
    request: func(
        target: component-id,
        payload: message-payload,
        timeout-ms: u64
    ) -> result<correlation-id, messaging-error>;

    /// Cancel pending request
    cancel-request: func(request-id: request-id) -> result<_, messaging-error>;

    /// Broadcast message to multiple components
    broadcast: func(
        targets: list<component-id>,
        payload: message-payload
    ) -> result<_, messaging-error>;

    /// Get current component's ID
    self-id: func() -> component-id;
}
```

---

### host-services.wit (Layer 3B - Host Provides)

```wit
package airssys:core@1.0.0;

/// Host-implemented general services
interface host-services {
    use types.{component-id, log-level, timestamp};
    use errors.{component-error};

    /// Structured logging
    log: func(
        level: log-level,
        message: string,
        context: option<list<tuple<string, string>>>
    );

    /// Get current time
    current-time: func() -> timestamp;

    /// Get current time in milliseconds
    current-time-millis: func() -> u64;

    /// Sleep for specified duration
    sleep-millis: func(duration-ms: u64);

    /// List all loaded components
    list-components: func() -> list<component-id>;

    /// Get metadata of another component
    get-component-metadata: func(id: component-id) -> result<component-info, component-error>;

    /// Basic component info
    record component-info {
        id: component-id,
        name: string,
        version: string,
        health: string,
    }
}
```

---

### storage.wit (Layer 3C - Host Provides)

```wit
package airssys:core@1.0.0;

/// Host-implemented storage interface (component-isolated)
interface storage {
    use types.{message-payload};
    use errors.{storage-error};

    /// Get value by key (within component's namespace)
    get: func(key: string) -> result<option<message-payload>, storage-error>;

    /// Set value by key
    set: func(key: string, value: message-payload) -> result<_, storage-error>;

    /// Delete value by key
    delete: func(key: string) -> result<_, storage-error>;

    /// Check if key exists
    exists: func(key: string) -> result<bool, storage-error>;

    /// List keys with optional prefix filter
    list-keys: func(prefix: option<string>) -> result<list<string>, storage-error>;

    /// Get storage usage info
    usage: func() -> result<storage-usage, storage-error>;

    /// Storage usage information
    record storage-usage {
        used-bytes: u64,
        quota-bytes: u64,
        key-count: u64,
    }
}
```

---

### world.wit (World Definition)

```wit
package airssys:core@1.0.0;

/// The main world that guest components implement
world component {
    /// Host-provided capabilities (components import these)
    import host-messaging;
    import host-services;
    import storage;

    /// Guest-implemented interfaces (components export these)
    export component-lifecycle;
}
```

---

### deps.toml (Package Configuration)

```toml
[package]
name = "airssys:core"
version = "1.0.0"
```

---

## Interface Summary

| File | Interface | Direction | Purpose |
|------|-----------|-----------|---------|
| `types.wit` | `types` | Shared | Foundation types (source of truth) |
| `errors.wit` | `errors` | Shared | All error variants |
| `capabilities.wit` | `capabilities` | Shared | Permission definitions |
| `component-lifecycle.wit` | `component-lifecycle` | **Guest exports** | Component contract |
| `host-messaging.wit` | `host-messaging` | **Host provides** | Inter-component messaging |
| `host-services.wit` | `host-services` | **Host provides** | General host services |
| `storage.wit` | `storage` | **Host provides** | Component-isolated storage |
| `world.wit` | `component` (world) | - | Ties imports/exports together |

---

## wit-bindgen Integration

Use macro-based binding generation (no build.rs):

```rust
wit_bindgen::generate!({
    world: "component",
    path: "wit/core",
});
```

---

## Validation Command

```bash
wasm-tools component wit wit/core/
```

---

## History

| Date | Version | Change |
|------|---------|--------|
| 2026-01-05 | 1.0 | Initial WIT interface design |

---

**This ADR defines all WIT interfaces for Phase 1 of the clean-slate rebuild.**
