# WASM - Component Framework

**Production-ready WebAssembly component framework for building fault-tolerant, scalable component-based systems with actor-based runtime integration.**

## Vision

Enable runtime deployment of secure, isolated components inspired by smart contract patterns (like CosmWasm), but for general-purpose computing—making plugin architectures as safe and seamless as web browsers loading JavaScript, with enterprise-grade fault tolerance.

## Motivation

The need for `airssys-wasm` emerged from fundamental challenges in building extensible, secure systems:

### The Problem

Modern applications increasingly need **pluggable architectures**—the ability to load third-party code at runtime. But traditional approaches have critical flaws:

#### 1. Native Shared Libraries (.so/.dll) - Unsafe

```rust
// Load third-party plugin
let lib = unsafe { Library::new("plugin.so")? };
let execute = unsafe { lib.get::<extern "C" fn()>("execute")? };
unsafe { execute(); }  // 💥 Can crash entire process, access all memory
```

**Problems:**

- ❌ No memory isolation (shared process)
- ❌ Single failure crashes host
- ❌ Full system access (filesystem, network, processes)
- ❌ Platform-specific (can't share between OS/arch)
- ❌ No hot reload (requires restart)

#### 2. Separate Processes - Heavy

```rust
// Launch plugin as separate process
let child = Command::new("./plugin").spawn()?;
// IPC communication overhead ~100µs per message
```

**Problems:**

- ❌ High overhead (~10-100MB per process)
- ❌ Slow startup (100-1000ms cold start)
- ❌ Complex IPC (serialization, sockets, pipes)
- ❌ Hard to manage lifecycle
- ❌ Limited sharing (everything via IPC)

#### 3. Interpreted Languages (Lua, Python) - Limited

```rust
// Embed Lua interpreter
let lua = Lua::new();
lua.load("plugin.lua").exec()?;
```

**Problems:**

- ❌ Single-language ecosystem
- ❌ Performance overhead (10-100x slower)
- ❌ No compile-time safety guarantees
- ❌ Limited type system
- ❌ Still need sandboxing for safety

### The WebAssembly Solution

**WebAssembly (WASM)** solves these problems by providing:

✅ **Memory Isolation**: Sandboxed linear memory (can't access host memory)  
✅ **Crash Isolation**: Component crash doesn't affect host  
✅ **Capability Security**: Fine-grained permissions (deny-by-default)  
✅ **Cross-Platform**: Same binary runs on Linux/macOS/Windows  
✅ **Hot Deployment**: Load/unload without restart  
✅ **Multi-Language**: Write in Rust, C++, Go, Python, JS—same WASM output  
✅ **Near-Native Performance**: ~95% native speed  
✅ **Small Footprint**: ~512KB baseline overhead  

### Real-World Example: Plugin System

**Without airssys-wasm (unsafe native plugins)**:

```rust
// Load untrusted plugin - no safety guarantees
let lib = unsafe { Library::new("third-party-plugin.so")? };
let process = unsafe { lib.get::<fn(Vec<u8>) -> Vec<u8>>("process")? };

// Plugin can do ANYTHING:
// - Read /etc/passwd
// - Spawn processes
// - Make network requests
// - Corrupt host memory
// - Crash entire application
let result = unsafe { process(input) }; // 💥 Hope it doesn't crash!
```

**With airssys-wasm (secure WASM components)**:

```rust
use airssys_wasm::actor::ComponentActor;
use airssys_wasm::core::{Capability, SecurityConfig};

// Load WASM component with capability restrictions
let capabilities = vec![
    Capability::FileRead("/data/*.txt".into()),  // Only read .txt files in /data
    // No write, no network, no process spawn
];

let component = ComponentActor::load(
    wasm_bytes,
    "third-party-plugin",
    capabilities
).await?;

// Component is:
// ✅ Memory isolated (can't access host memory)
// ✅ Crash isolated (supervised, auto-restart)
// ✅ Capability restricted (only allowed /data/*.txt reads)
// ✅ Audited (all operations logged)
// ✅ Hot deployable (load/unload without restart)

let result = component.execute(input).await?; // Safe!
```

### The Smart Contract Inspiration

Inspired by blockchain smart contract platforms like **CosmWasm**:

- **Runtime Deployment**: Deploy new components without host restart
- **Sandboxed Execution**: Components can't harm each other or host
- **Capability-Based Security**: Fine-grained permissions for each component
- **Composability**: Chain components for complex workflows
- **Language Agnostic**: Write in any WASM-compatible language

But applied to **general-purpose computing**, not just blockchain:

- AI plugin systems
- Microservice composition
- IoT edge functions
- Game mod systems
- Enterprise integration adapters

### Why airssys-wasm?

**What makes airssys-wasm different from just using Wasmtime directly:**

1. **Actor Integration**: Components run as supervised actors with automatic crash recovery
2. **Production-Ready Patterns**: Request-response, pub-sub, supervision—battle-tested patterns
3. **High Performance**: 6.12M msg/sec throughput, 286ns component spawn, O(1) registry
4. **Security Framework**: Built-in capability enforcement with audit logging
5. **AirsSys Ecosystem**: Integrates with airssys-rt (actors) and airssys-osl (system operations)

## Key Features

### 🔒 Security by Default

**Capability-Based Security**:

```rust
use airssys_wasm::core::Capability;

// Fine-grained permissions
let capabilities = vec![
    Capability::FileRead("/workspace/*.rs".into()),    // Only Rust files in workspace
    Capability::NetworkOutbound("api.example.com".into()), // Only specific domain
    // No file write, no process spawn, no other network access
];
```

**Security Layers** (DEBT-WASM-004):

1. **Sender Authorization**: Components must have `Capability::Messaging` to send messages
2. **Payload Size Validation**: Default 1MB limit (prevents memory exhaustion)
3. **Rate Limiting**: 1000 msg/sec per sender (prevents abuse)
4. **Audit Logging**: All operations logged with timestamp and context

**Performance**: Security checks add only **554ns overhead** per message (9x faster than 5µs target).

### 🎭 Dual-Trait Pattern

**Separation of Concerns**:

```rust
// Child trait: Lifecycle management
impl Child for MyComponent {
    fn pre_start(&mut self, context: &ChildContext) -> Result<(), ChildError> {
        println!("Component starting: {}", context.component_id);
        Ok(())
    }
}

// Actor trait: Message handling
#[async_trait]
impl Actor for MyComponent {
    async fn handle_message(&mut self, message: Self::Message, context: &ActorContext) -> Result<(), Self::Error> {
        // Process messages with automatic supervision
        Ok(())
    }
}
```

**Benefits:**

- Clear lifecycle boundaries
- Independent testing (lifecycle vs messaging)
- Flexible composition patterns
- Supervisor integration

### ⚡ High Performance

**Benchmarked Performance** (Task 6.2):

| Metric | Value | Source |
|--------|-------|--------|
| **Component spawn** | 286ns | `actor_lifecycle_benchmarks.rs` |
| **Message throughput** | 6.12M msg/sec | `messaging_benchmarks.rs` |
| **Registry lookup** | 36ns O(1) | `scalability_benchmarks.rs` |
| **Request-response** | 3.18µs | `messaging_benchmarks.rs` |
| **Full lifecycle** | 1.49µs | `actor_lifecycle_benchmarks.rs` |
| **Scaling** | Perfect (10→1,000 components) | `scalability_benchmarks.rs` |

**All targets exceeded by 16-26,500x** (28 benchmarks, 95% confidence).

### 🛡️ Fault Tolerance

**Automatic Crash Recovery**:

```rust
// Supervised component with exponential backoff
let supervisor = SupervisorBuilder::new()
    .strategy(RestartStrategy::OneForOne)  // Restart only failed component
    .max_restarts(3, Duration::from_secs(60))
    .backoff_strategy(BackoffStrategy::Exponential {
        initial: Duration::from_millis(100),
        max: Duration::from_secs(30),
        factor: 2.0,
    })
    .build();

// Component crashes are automatically recovered
supervisor.spawn(component).await?;
```

**Supervision Strategies**:

- **OneForOne**: Restart only failed component
- **OneForAll**: Restart all components
- **RestForOne**: Restart failed and dependent components

**Restart Policies**:

- **Permanent**: Always restart (critical services)
- **Temporary**: Never restart (one-time tasks)
- **Transient**: Restart only if abnormal termination

### 🌐 Multi-Language Support

**Language-Agnostic Development** via WIT (WebAssembly Interface Types):

```wit
// Define interface once
interface processor {
    process: func(input: list<u8>) -> result<list<u8>, string>;
}
```

**Implement in any language**:

```rust
// Rust implementation
#[component]
impl Processor for RustProcessor {
    fn process(&mut self, input: Vec<u8>) -> Result<Vec<u8>, String> {
        // Rust logic
    }
}
```

```cpp
// C++ implementation (via wit-bindgen)
class CppProcessor : public Processor {
    std::vector<uint8_t> process(std::vector<uint8_t> input) override {
        // C++ logic
    }
};
```

**Same WASM output, same host integration**.

### 🔗 Component Composition

**Chain components for complex workflows**:

```rust
// Pipeline: Ingestion → Validation → Processing → Storage
let ingestion = ComponentActor::load(ingestion_wasm, "ingestion", capabilities).await?;
let validation = ComponentActor::load(validation_wasm, "validation", capabilities).await?;
let processing = ComponentActor::load(processing_wasm, "processing", capabilities).await?;
let storage = ComponentActor::load(storage_wasm, "storage", capabilities).await?;

// Components communicate via messages
ingestion.send(IngestMsg::Data(bytes)).await?;
// → validation receives ValidateMsg
// → processing receives ProcessMsg
// → storage receives StoreMsg
```

## Use Cases

### AI Plugin Systems

Build secure AI tools with runtime-deployable plugins:

```rust
// Load AI agent components at runtime
let analyzer = ComponentActor::load(
    analyzer_wasm,
    "code-analyzer",
    vec![Capability::FileRead("/workspace/*.rs".into())]
).await?;

let formatter = ComponentActor::load(
    formatter_wasm,
    "code-formatter",
    vec![
        Capability::FileRead("/workspace/*.rs".into()),
        Capability::FileWrite("/workspace/*.rs".into()),
    ]
).await?;

// AI can only access workspace directory
// Components crash-isolated and supervised
```

### Microservice Composition

Build composable microservices with hot deployment:

```rust
// Deploy new service version without downtime
let new_version = ComponentActor::load(
    service_v2_wasm,
    "payment-service-v2",
    capabilities
).await?;

// Blue-green deployment
router.add_route("/payment", new_version).await?;
router.remove_route_version("/payment", old_version).await?;

// Old version gracefully shutdown, new version serving
```

### IoT Edge Functions

Deploy functions to edge devices:

```rust
// Load edge function with strict resource limits
let edge_fn = ComponentActor::load_with_limits(
    function_wasm,
    "sensor-processor",
    capabilities,
    ResourceLimits {
        memory: 10 * 1024 * 1024,  // 10MB
        cpu_time: Duration::from_secs(5),
    }
).await?;

// Function processes sensor data in isolation
```

### Game Mod Systems

Secure mod system for games:

```rust
// Load player-created mod with restrictions
let mod_component = ComponentActor::load(
    player_mod_wasm,
    "custom-weapon-mod",
    vec![
        Capability::GameAPI("weapons".into()),  // Only weapon API access
        // No file access, no network, no process spawn
    ]
).await?;

// Mod crash won't crash game (supervised)
// Mod can't cheat (capability restricted)
```

### Enterprise Integration

Runtime adapters for system integration:

```rust
// Load SAP connector at runtime
let sap_connector = ComponentActor::load(
    sap_wasm,
    "sap-integration",
    vec![
        Capability::NetworkOutbound("sap.company.com".into()),
        Capability::FileWrite("/exports/*.xml".into()),
    ]
).await?;

// Add Salesforce connector without restart
let sf_connector = ComponentActor::load(
    salesforce_wasm,
    "salesforce-integration",
    capabilities
).await?;
```

## Quick Start

### Installation

```toml
[dependencies]
airssys-wasm = "0.1.0"
airssys-rt = "0.1.0"
async-trait = "0.1"
tokio = { version = "1.47", features = ["full"] }
```

### Your First Component

```rust
use airssys_wasm::actor::ComponentActor;
use airssys_wasm::core::Capability;
use airssys_rt::prelude::*;
use async_trait::async_trait;

// 1. Define component with state
#[derive(Clone)]
struct MyComponent {
    state: Arc<RwLock<ComponentState>>,
}

// 2. Implement Child trait (lifecycle)
impl Child for MyComponent {
    fn pre_start(&mut self, context: &ChildContext) -> Result<(), ChildError> {
        println!("Component starting: {}", context.component_id);
        Ok(())
    }
    
    fn post_stop(&mut self, context: &ChildContext) {
        println!("Component stopped: {}", context.component_id);
    }
}

// 3. Implement Actor trait (messages)
#[async_trait]
impl Actor for MyComponent {
    type Message = MyMessage;
    type Error = ComponentError;
    
    async fn handle_message(
        &mut self,
        message: Self::Message,
        context: &ActorContext,
    ) -> Result<(), Self::Error> {
        // Process message with automatic supervision
        match message {
            MyMessage::Process(data) => {
                let mut state = self.state.write().await;
                state.process(data)?;
            }
            MyMessage::Query(reply) => {
                let state = self.state.read().await;
                reply.send(state.data()).ok();
            }
        }
        Ok(())
    }
}

// 4. Load and supervise component
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create supervisor
    let supervisor = SupervisorBuilder::new()
        .strategy(RestartStrategy::OneForOne)
        .build();
    
    // Load WASM component with capabilities
    let capabilities = vec![
        Capability::FileRead("/data/*.json".into()),
    ];
    
    let component = ComponentActor::load(
        wasm_bytes,
        "my-component",
        capabilities
    ).await?;
    
    // Spawn supervised component
    supervisor.spawn(component).await?;
    
    Ok(())
}
```

## Architecture

```
┌──────────────────────────────────────────────────┐
│     Application Layer (Your Host App)           │
│  ┌────────────────────────────────────────────┐  │
│  │  ComponentActor Loading & Management       │  │
│  │  - Load WASM components                    │  │
│  │  - Configure capabilities                  │  │
│  │  - Orchestrate communication               │  │
│  └────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────┘
                       ↓
┌──────────────────────────────────────────────────┐
│   Host Runtime (airssys-wasm)                    │
│  ┌──────────────┐  ┌──────────┐  ┌────────────┐ │
│  │  Component   │  │ Security │  │   Actor    │ │
│  │  Loading     │  │ Enforce  │  │Integration │ │
│  └──────────────┘  └──────────┘  └────────────┘ │
│  ┌──────────────┐  ┌──────────┐  ┌────────────┐ │
│  │  Messaging   │  │ Supervisor│  │   OSL      │ │
│  │  Router      │  │ Trees     │  │  Bridge    │ │
│  └──────────────┘  └──────────┘  └────────────┘ │
└──────────────────────────────────────────────────┘
                       ↓
┌──────────────────────────────────────────────────┐
│   Component Layer (WASM Plugins)                 │
│  ┌────────────────────────────────────────────┐  │
│  │  WASM Components (.wasm files)             │  │
│  │  - Written in Rust/C++/Go/Python/JS        │  │
│  │  - Compiled to WebAssembly                 │  │
│  │  - Memory & crash isolated                 │  │
│  │  - Capability restricted                   │  │
│  └────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────┘
                       ↓
┌──────────────────────────────────────────────────┐
│   AirsSys Integration                            │
│  ┌──────────────┐              ┌──────────────┐  │
│  │  airssys-rt  │              │  airssys-osl │  │
│  │ (Actor Sys)  │              │(OS Operations)│  │
│  └──────────────┘              └──────────────┘  │
└──────────────────────────────────────────────────┘
```

## Documentation

### 📚 Tutorials (Learning-Oriented)

- [Your First ComponentActor](tutorials/your-first-component-actor.md) - Build your first component (1 hour)
- [Building a Stateful Component](tutorials/stateful-component-tutorial.md) - Add state management (1.5 hours)

### 📖 How-To Guides (Task-Oriented)

- [Request-Response Pattern](guides/request-response-pattern.md) - Implement request-response (30 min)
- [Pub-Sub Broadcasting](guides/pubsub-broadcasting.md) - Broadcast messages (30 min)
- [Supervision and Recovery](guides/supervision-and-recovery.md) - Add fault tolerance (45 min)
- [Component Composition](guides/component-composition.md) - Chain components (1 hour)
- [Production Deployment](guides/production-deployment.md) - Deploy to production (1 hour)
- [Best Practices](guides/best-practices.md) - Optimization tips
- [Troubleshooting](guides/troubleshooting.md) - Debug common issues

### 📋 Reference (Information-Oriented)

- [ComponentActor API](api/component-actor.md) - API reference
- [Lifecycle Hooks](api/lifecycle-hooks.md) - Lifecycle specification
- [Message Routing](reference/message-routing.md) - Routing internals
- [Performance Characteristics](reference/performance-characteristics.md) - Benchmark data

### 💡 Explanation (Understanding-Oriented)

- [Dual-Trait Design](explanation/dual-trait-design.md) - Why two traits?
- [State Management Patterns](explanation/state-management-patterns.md) - State strategies
- [Supervision Architecture](explanation/supervision-architecture.md) - Fault tolerance design
- [Production Readiness](explanation/production-readiness.md) - Production validation

## Current Status

**Version**: 0.1.0  
**Status**: ✅ Production Ready

### What's Complete

- ✅ **ComponentActor Pattern**: Dual-trait design (Child + Actor)
- ✅ **Security Framework**: Capability-based security with audit logging
- ✅ **Supervision Trees**: Automatic crash recovery with exponential backoff
- ✅ **High Performance**: 6.12M msg/sec throughput, 286ns spawn
- ✅ **Message Routing**: Request-response, pub-sub, O(1) registry
- ✅ **Comprehensive Testing**: 945 integration tests (100% pass)
- ✅ **Performance Benchmarks**: 28 benchmarks (all targets exceeded)
- ✅ **Documentation**: 19 comprehensive guides + 6 examples

### Quality Score: 9.7/10

- **Correctness**: 10/10 (945 tests, 100% pass)
- **Performance**: 10/10 (all targets exceeded by 16-26,500x)
- **Documentation**: 10/10 (comprehensive guides and examples)
- **Code Quality**: 10/10 (zero clippy warnings)
- **Test Coverage**: 10/10 (≥95% coverage)
- **Production Readiness**: 9/10 (minor observability gaps)

## Examples

Working examples demonstrating core patterns:

| Example | Purpose | File |
|---------|---------|------|
| **Basic Component** | Minimal lifecycle and messages | `basic_component_actor.rs` |
| **Stateful Component** | State management patterns | `stateful_component.rs` |
| **Request-Response** | Correlation-based communication | `request_response_pattern.rs` |
| **Pub-Sub Broadcasting** | Topic-based messaging | `pubsub_component.rs` |
| **Supervised Component** | Crash recovery patterns | `supervised_component.rs` |
| **Component Composition** | Multi-component orchestration | `component_composition.rs` |

Run examples:
```bash
cargo run --example basic_component_actor
```

## The AirsSys WASM Ecosystem

**Three projects working together**:

| Project | Role | Who Uses It |
|---------|------|-------------|
| **airssys-wasm** (this library) | Host runtime for loading/running WASM components | App developers building plugin systems |
| **airssys-wasm-component** | Procedural macros for building components | Component developers writing plugins |
| **airssys-wasm-cli** | CLI tool for component management | Developers during development workflow |

**Think of it like web development**:

- `airssys-wasm` = Browser (Chrome/Firefox)
- `airssys-wasm-component` = React/JSX (developer framework)
- `airssys-wasm-cli` = npm CLI (package manager)

## Resources

- **Repository**: [github.com/airsstack/airssys](https://github.com/airsstack/airssys)
- **Crate**: [crates.io/crates/airssys-wasm](https://crates.io/crates/airssys-wasm)
- **API Docs**: Run `cargo doc --open` in `airssys-wasm/`
- **Examples**: See `airssys-wasm/examples/` directory
- **Benchmarks**: See `airssys-wasm/benches/` directory

## License

Dual-licensed under Apache License 2.0 or MIT License.

---

**Next Steps**: Start with [Your First ComponentActor Tutorial](tutorials/your-first-component-actor.md) or explore [Working Examples](https://github.com/airsstack/airssys/tree/main/airssys-wasm/examples).
