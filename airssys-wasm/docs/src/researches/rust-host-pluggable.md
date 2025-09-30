# Language Agnostic Backend System - Pluggable With WASM

## Host & Plugin WIT

### 1. The WIT Contract: Defining the World

First, we define the formal contract using a modular WIT package. This package will contain the interfaces for both the host's capabilities and the plugin's required exports.



#### `wit/host.wit` (Capabilities Provided by the Host)

This file defines the functions and resources your host system will provide to the plugins. For a network backend, this might include logging, database access, or a key-value store. Using `resource` is critical here for managing stateful connections securely.

Code snippet

```
package local:network-host@1.0.0;

interface logging {
  log: func(level: log-level, message: string);
}

enum log-level {
  info,
  warning,
  error,
}

/// A handle to a secure key-value store provided by the host.
resource key-value-store {
  /// Opens a bucket and returns a handle.
  constructor(bucket-name: string);
  /// Gets a value by key.
  get: func(self: borrow<key-value-store>, key: string) -> result<option<list<u8>>, string>;
  /// Sets a value for a key.
  set: func(self: borrow<key-value-store>, key: string, value: list<u8>) -> result<_, string>;
}
```



#### `wit/plugin.wit` (Contract for All Plugins)

This file defines the interface that every plugin engineer must implement. It directly maps to your requirements for `access_levels` and `entrypoint`.

Code snippet

```
package local:network-host@1.0.0;

/// Defines the permissions a plugin can request.
flags access-levels {
  /// Allows the plugin to read from the host's key-value store.
  key-value-read,
  /// Allows the plugin to write to the host's key-value store.
  key-value-write,
  /// Allows the plugin to make outbound HTTP requests.
  outbound-http,
}

/// Represents an incoming request from an external RPC call.
record rpc-request {
  /// A unique identifier for the request.
  id: string,
  /// The method or operation to be performed.
  method: string,
  /// The payload of the request, as raw bytes.
  payload: list<u8>,
}

/// Represents the response the plugin must return.
record rpc-response {
  /// The status of the operation.
  status: u16,
  /// The response payload, as raw bytes.
  body: list<u8>,
}

/// The interface every plugin must export.
interface handler {
  /// The host calls this first to determine what capabilities to grant.
  get-access-levels: func() -> access-levels;

  /// The main entrypoint for the plugin to handle a request.
  handle-request: func(req: rpc-request) -> result<rpc-response, string>;
}
```



#### `wit/world.wit` (The Top-Level World)

This file unites the host and plugin interfaces into a single, complete contract for a plugin component.

Code snippet

```
package local:network-host@1.0.0;

include "host.wit";
include "plugin.wit";

world network-plugin {
  // The plugin must provide this interface.
  export handler;

  // The host will provide these interfaces.
  import logging;
  import key-value-store;
  // We can also import standard WASI interfaces.
  import wasi:http/outgoing-handler@0.2.0;
}
```



### 2. The Host System Implementation (Rust + Wasmtime)

The host application is the orchestrator. It loads plugins, checks their permissions, grants capabilities, and forwards RPC calls to the correct plugin `entrypoint`.

Here’s the high-level logic for your `main.rs`:

1. **Plugin Registry**: Maintain a map of registered plugins, for example, `HashMap<String, PathBuf>` mapping a unique plugin name (e.g., `user-auth-service`) to the path of its `.wasm` component file.
2. **RPC Server**: The host application would expose an RPC endpoint. A great choice for this is **wRPC (WIT-RPC)**, a framework designed specifically for this purpose.1 It allows you to serve a component's functions over a network transport like TCP.3 An external client can then call the plugin as if it were a local function.
3. **Dynamic Invocation Flow**: When an RPC call arrives for a specific plugin:
   - **Load Plugin**: The host loads the plugin's `.wasm` file into a Wasmtime `Component`.
   - **Check Permissions**: Before fully linking, the host instantiates the plugin with minimal permissions and calls its `get-access-levels()` export. This is a critical security step.
   - **Build Sandboxed Environment**: Based on the permissions returned by the plugin and a host-defined security policy (e.g., "plugin 'user-auth-service' is allowed `key-value-read` but not `key-value-write`"), the host configures the `Linker`.
     - If `key-value-read` is requested and allowed, it adds the `key-value-store` implementation to the linker.
     - If `outbound-http` is requested and allowed, it adds the `wasi:http/outgoing-handler` to the linker.
     - If a requested permission is *not* allowed by the policy, the host rejects the request and does not proceed.
   - **Instantiate and Execute**: The host creates a `Store`, fully instantiates the component with the permission-scoped `Linker`, and calls the `handle-request()` function with the parameters from the RPC call.
   - **Return Response**: The result from the plugin is serialized and sent back to the external RPC client.

This "just-in-time" capability injection ensures that each plugin only gets the exact permissions it needs for a given operation, enforcing the principle of least privilege.



### 3. The Plugin Implementation (Rust)

An engineer building a plugin would only need to focus on the business logic, using the provided WIT package.

Here is what a simple plugin's `src/lib.rs` might look like:

Rust

```rust
// Generate bindings from the WIT package provided by the host system.
#[allow(warnings)]
mod bindings;

use bindings::Guest;
use bindings::local::network_host::{
    host, // The imported host interfaces
    plugin::{AccessLevels, RpcRequest, RpcResponse}, // The exported types
};

struct MyPlugin;

impl Guest for MyPlugin {
    /// Declare the permissions this plugin needs to function.
    fn get_access_levels() -> AccessLevels {
        // This plugin needs to read and write to the key-value store.
        AccessLevels::KEY_VALUE_READ | AccessLevels::KEY_VALUE_WRITE
    }

    /// Handle the incoming request.
    fn handle_request(req: RpcRequest) -> Result<RpcResponse, String> {
        host::log(host::LogLevel::Info, &format!("Handling request {}", req.id));

        // Use the imported `key-value-store` resource.
        let store = host::KeyValueStore::new("user-data")
           .map_err(|e| e.to_string())?;

        // Business logic: get a value, modify it, and set it back.
        let current_value = host::KeyValueStore::get(&store, "some-key")
           .map_err(|e| e.to_string())?
           .unwrap_or_default();

        let new_value = format!("{} processed by {}",
            String::from_utf8_lossy(&current_value),
            String::from_utf8_lossy(&req.payload)
        );

        host::KeyValueStore::set(&store, "some-key", new_value.as_bytes())
           .map_err(|e| e.to_string())?;

        // Return a successful response.
        Ok(RpcResponse {
            status: 200,
            body: "Successfully processed".as_bytes().to_vec(),
        })
    }
}

bindings::export!(MyPlugin with_types_in bindings);
```

This architecture is powerful because it creates a clean separation of concerns. The host is in full control of security and resources, while plugin developers can work in any supported language (Rust, C, Go, TypeScript, etc.) and only need to understand the high-level WIT contract to build powerful, sandboxed extensions for your backend system.

---

## Distributed WIT 

### Approach 1: The API Crate (Recommended for Rust-centric ecosystems)

This is the most idiomatic and powerful approach if your host and the majority of your plugin developers are using Rust. You treat the WIT package as a dedicated Rust crate whose primary purpose is to distribute the API contract.

**How it Works:**

1. **Host-Side (You): Create an API Crate**
   - You create a new, lightweight Rust library crate, for example, `network-plugin-api`.
   - This crate contains almost no Rust code. Its main job is to hold the `wit/` directory with your `host.wit`, `plugin.wit`, `types.wit`, and `world.wit` files.
   - You publish this crate to your company's private crate registry (or `crates.io` if it's a public project).
2. **Plugin-Side (The Engineer): Add a Dependency**
   - The plugin developer creates their new component project using `cargo component new my-plugin`.
   - Instead of copying files, they simply add a dependency on your API crate in their `Cargo.toml`. Crucially, this is done in two places:
     1. A standard dependency to make the WIT files available.
     2. A `cargo-component` specific dependency to tell the toolchain to look inside that crate for the WIT definitions.

**Example `Cargo.toml` for a plugin:**

Ini, TOML

```toml
[package]
name = "my-plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
# This is a regular dependency on the API crate
network-plugin-api = "0.1.0" 

[package.metadata.component]
# This tells cargo-component that our plugin's world
# depends on the WIT package provided by the `network-plugin-api` crate.
[package.metadata.component.dependencies]
"local:network-host" = { path = "wit" } # Assuming the plugin has its own wit folder for its world
```

*Note: The exact dependency declaration mechanism is evolving, but the principle is that `cargo-component` is designed to resolve WIT dependencies from your Cargo graph.*

**Why this is the best approach:**

- **Versioning:** Versioning is handled perfectly by Cargo. If you update the API, you publish a new version of the `network-plugin-api` crate (e.g., `0.2.0`). Plugin developers can update their `Cargo.toml` to get the new contract.
- **Developer Experience:** It's a familiar workflow for any Rust developer. They just `cargo add` a dependency.
- **Tooling Integration:** `cargo component` is built to understand this workflow.1 It automatically finds the WIT files from the dependency graph and generates the correct bindings without any manual path configuration.



### Approach 2: The Git Repository Approach

This is a simpler, more language-agnostic method that works well if you have a mix of languages or don't want to manage a crate registry.

**How it Works:**

1. **Host-Side (You): Create a WIT Repository**
   - You create a dedicated Git repository (e.g., `network-plugin-interfaces`).
   - This repository contains *only* the `wit/` directory with your API contract files.
   - You use Git tags to version your API (e.g., `v1.0.0`, `v1.1.0`).
2. **Plugin-Side (The Engineer): Use a Git Submodule**
   - The plugin developer adds your interface repository as a Git submodule to their own project.
   - They then point their build tools (`cargo-component` for Rust, `wit-bindgen` for other languages) to the local path of the submodule to find the WIT definitions.2

**Example for a Rust plugin:**

Bash

```bash
# Inside the plugin's project directory
git submodule add https://github.com/your-org/network-plugin-interfaces.git wit-api
```

Then, in their `src/lib.rs`, they would point the binding generator to that path:

Rust

```rust
// This tells wit-bindgen to look in the `wit-api/wit` directory
// instead of the default `wit` directory.
wit_bindgen::generate!({
    path: "wit-api/wit",
    world: "network-plugin",
});
```

**Why this approach is good:**

- **Language Agnostic:** Any developer, regardless of language, can clone the repository.
- **Simple Setup:** Doesn't require a package registry.

**Drawbacks:**

- **Manual Versioning:** Developers have to manually update the submodule to a new Git tag to get API updates. It's less automated than a package manager.
- **Less Integrated:** It feels less "native" to the build tooling of each language.



### Approach 3: The Component Registry (The Future)

This is the long-term vision for the entire WebAssembly ecosystem. WIT packages will be published to and consumed from dedicated component registries.

**How it will work:**

- **The Protocol:** A protocol called **warg** is being developed to define how these registries work and federate with each other.3
- **The Tooling:** Tools like `cargo component publish` and `wit publish` will be used to push versioned WIT packages to a registry.1
- **Consumption:** Plugin developers will use a command like `cargo component add wasi:http` to automatically fetch the correct WIT package from a registry and add it to their project's manifest.1

This is analogous to how `npm`, `crates.io`, and `PyPI` work today. While the tooling is still maturing, this is the ultimate goal and the most scalable solution.



### Summary and Recommendation

| Approach                  | Pros                                                         | Cons                                                         | Best For                                             |
| ------------------------- | ------------------------------------------------------------ | ------------------------------------------------------------ | ---------------------------------------------------- |
| **1. API Crate**          | Seamless Cargo integration, excellent versioning, great DX for Rust developers. | Primarily benefits Rust projects.                            | **Recommended for most Rust-based host systems.**    |
| **2. Git Repo**           | Language-agnostic, simple to set up.                         | Manual version management, less integrated with build tools. | Polyglot teams or projects without a crate registry. |
| **3. Component Registry** | The ultimate solution for cross-language dependency management, discovery, and versioning. | Tooling is still emerging and not yet universally adopted.   | The future of all component-based development.       |

For your use case, **start with Approach 1: The API Crate**. It provides the best developer experience for your engineers, leverages the powerful and familiar Cargo ecosystem for versioning and dependency management, and is fully supported by the current generation of component tooling.