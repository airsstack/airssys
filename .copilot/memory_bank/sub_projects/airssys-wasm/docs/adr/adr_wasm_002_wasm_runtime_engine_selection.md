# ADR-WASM-002: WASM Runtime Engine Selection

**Status:** Accepted  
**Date:** 2025-10-19  
**Decision Makers:** Architecture Team  
**Related:** KNOWLEDGE-WASM-001 (Component Framework Architecture), KNOWLEDGE-WASM-003 (Core Architecture Design)

---

## Context

The airssys-wasm framework requires a WebAssembly runtime engine to execute components written in multiple programming languages (Rust, JavaScript, Go, Python, C++, etc.). This is the most foundational technical decision for the entire framework, as it affects:

- Component execution performance and reliability
- Developer experience and language support
- Security model and isolation guarantees
- Integration with airssys-rt actor system
- Async/await patterns for non-blocking I/O
- Capability-based security enforcement
- Production deployment characteristics

### Requirements

**Functional Requirements:**
- Support WebAssembly Component Model for interface composition
- Implement WASI Preview 2 for standardized system interface
- Enable cross-language component development
- Provide fine-grained capability-based security
- Support async/await for non-blocking operations
- Allow component instantiation and execution
- Enable resource limiting (CPU, memory, I/O)

**Non-Functional Requirements:**
- Production-ready stability and maturity
- Active development and security maintenance
- Comprehensive documentation and tooling
- License compatibility (Apache 2.0 or MIT)
- Pure Rust implementation (preferred for ecosystem fit)
- Integration with Tokio async runtime
- Support for supervisor patterns (airssys-rt integration)

**Performance Requirements (Baseline, Not Optimization Targets):**
- Component instantiation: "Fast enough" (~10ms ballpark acceptable)
- Function call overhead: Negligible (prefer <100ns but not critical)
- Execution speed: Near-native performance (95%+ of native code)
- Memory overhead: Reasonable per-component baseline (<1MB ideal)

**Note on Performance:** Following YAGNI principles, we intentionally defer performance optimization. The selected runtime should provide good baseline performance, but we will measure actual bottlenecks in real-world usage before adding complexity for theoretical performance gains.

---

## Decision

### Primary Decision: Wasmtime Runtime Engine

**We will use Wasmtime as the WebAssembly runtime engine for airssys-wasm.**

**Configuration:**
```rust
use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::{Config, Engine, Store};

/// Create configured Wasmtime engine for airssys-wasm
pub fn create_wasm_engine() -> Result<Engine> {
    let mut config = Config::new();
    
    // Enable WebAssembly Component Model (critical)
    config.wasm_component_model(true);
    
    // Enable async support (mandatory for airssys-rt integration)
    config.async_support(true);
    
    // JIT compilation with Cranelift optimizer
    config.strategy(wasmtime::Strategy::Cranelift);
    config.cranelift_opt_level(wasmtime::OptLevel::Speed);
    
    // Security: Enable fuel metering for CPU limits
    config.consume_fuel(true);
    
    // Memory: Reasonable stack limit (configurable per-component)
    config.max_wasm_stack(2 * 1024 * 1024); // 2MB default stack
    
    Ok(Engine::new(&config)?)
}
```

### Supporting Decisions

#### 1. Compilation Strategy: JIT (Just-In-Time)

**Decision:** Use JIT compilation with Cranelift, not AOT (Ahead-of-Time).

**Rationale:**
- Simpler deployment workflow (no pre-compilation step)
- Faster development iteration (instant component loading)
- Dynamic optimization capabilities
- Meets instantiation performance needs (~8ms typical)
- YAGNI: AOT can be added later if proven necessary

**Implementation:**
```rust
// JIT: Component compiled at runtime
let component = Component::from_file(&engine, "plugin.wasm")?;
let instance = linker.instantiate_async(&mut store, &component).await?;
```

**Future Enhancement (Phase 2+):**
AOT support can be added as optional optimization:
```rust
// Future: Pre-compile for faster instantiation
// $ wasmtime compile plugin.wasm -o plugin.cwasm
let component = unsafe {
    Component::deserialize_file(&engine, "plugin.cwasm")?
};
```

#### 2. Async Support: Async-First Architecture

**Decision:** Enable async support by default, make it mandatory for all components.

**Rationale:**
- Critical for airssys-rt actor model integration
- Enables non-blocking I/O (network, filesystem, messaging)
- Perfect Tokio integration (seamless async/await)
- Supports concurrent component execution
- Allows component cancellation via Tokio primitives

**Implementation:**
```rust
// Enable async in engine config
config.async_support(true);

// All component operations use async
let instance = linker.instantiate_async(&mut store, &component).await?;
let func = instance.get_typed_func::<Args, Output>(&mut store, "execute")?;
let result = func.call_async(&mut store, args).await?;

// Host functions can be async and use Tokio
linker.func_wrap_async(
    "host",
    "fetch-url",
    |mut caller: Caller<'_, State>, url: String| {
        Box::new(async move {
            // Use Tokio's HTTP client
            let response = caller.data()
                .http_client
                .get(&url)
                .send()
                .await?;
            Ok(response.bytes().await?)
        })
    },
)?;
```

#### 3. Resource Limits: Mandatory Memory + Hybrid CPU Limiting

**Decision 3a: Memory Limits - Mandatory Engineer-Defined**

Memory limits MUST be explicitly declared in Component.toml. No defaults provided.

**Rationale:**
- Forces engineers to think about resource usage
- No silent defaults that hide memory costs
- Clear resource awareness during development
- Prevents "works on my machine" production surprises
- Engineering decision, not host-imposed

**Component.toml (REQUIRED):**
```toml
[component]
name = "my-component"
version = "1.0.0"

[resources]
# MANDATORY: Must be explicitly declared
max_memory = "128MB"              # Maximum memory allocation
max_stack = "2MB"                 # Stack size limit
initial_memory = "16MB"           # Initial allocation

# OPTIONAL: CPU limits have defaults
max_fuel_per_execution = 1000000  # Fuel units per call
max_execution_time_ms = 100       # Wall-clock timeout
```

**Validation:**
```rust
pub fn validate_component_manifest(manifest: &ComponentManifest) -> Result<()> {
    // Memory limits are MANDATORY
    if manifest.resources.max_memory.is_none() {
        return Err(ComponentError::MissingResourceLimits {
            component: manifest.component.name.clone(),
            missing: vec!["max_memory"],
            message: 
                "Memory limits must be explicitly declared in Component.toml.\n\
                 This is an engineering decision to ensure resource awareness.\n\
                 Example:\n\
                 [resources]\n\
                 max_memory = \"128MB\"\n\
                 max_stack = \"2MB\"\n\
                 initial_memory = \"16MB\"".to_string(),
        });
    }
    
    if manifest.resources.max_stack.is_none() {
        return Err(ComponentError::MissingResourceLimits {
            component: manifest.component.name.clone(),
            missing: vec!["max_stack"],
            message: "Stack limits must be explicitly declared in Component.toml.".to_string(),
        });
    }
    
    // Fuel and timeout have defaults if not specified
    let fuel = manifest.resources.max_fuel_per_execution
        .unwrap_or(DEFAULT_FUEL_PER_EXECUTION);
    let timeout = manifest.resources.max_execution_time_ms
        .unwrap_or(DEFAULT_EXECUTION_TIMEOUT_MS);
    
    Ok(())
}
```

**Decision 3b: CPU Limits - Hybrid Fuel + Timeout**

Use both fuel metering (deterministic) and wall-clock timeout (guaranteed).

**Rationale:**
- **Fuel metering**: Deterministic CPU limiting, can't be bypassed by slow I/O
- **Wall-clock timeout**: Protects against slow operations (network calls, etc.)
- **Dual protection**: Best of both worlds - determinism + guarantees
- **Complementary**: Fuel limits CPU instructions, timeout limits real time

**Implementation:**
```rust
use tokio::time::{timeout, Duration};
use wasmtime::Store;

pub struct ComponentExecutor {
    engine: Engine,
    linker: Linker<HostState>,
}

impl ComponentExecutor {
    pub async fn execute_with_limits(
        &self,
        component: &Component,
        limits: ResourceLimits,
        args: ComponentArgs,
    ) -> Result<ExecutionResult> {
        let mut store = Store::new(&self.engine, HostState::new());
        
        // 1. Fuel metering (deterministic CPU limit)
        store.add_fuel(limits.max_fuel_per_execution)?;
        
        // 2. Memory resource limiter
        store.limiter(|state| &mut state.resource_limiter);
        
        // Instantiate component with limits
        let instance = self.linker.instantiate_async(&mut store, component).await?;
        let func = instance.get_typed_func::<ComponentArgs, ComponentOutput>(
            &mut store,
            "execute"
        )?;
        
        // 3. Wall-clock timeout (time-based limit)
        let result = timeout(
            Duration::from_millis(limits.max_execution_time_ms),
            func.call_async(&mut store, args)
        ).await;
        
        match result {
            Ok(Ok(output)) => {
                // Success - record metrics
                let fuel_consumed = store.fuel_consumed()?;
                Ok(ExecutionResult::Success {
                    output,
                    fuel_consumed,
                })
            }
            Ok(Err(trap)) => {
                // Component trapped (out of fuel, memory violation, invalid operation)
                let fuel_consumed = store.fuel_consumed().ok();
                Err(ComponentError::Trapped {
                    reason: trap.to_string(),
                    fuel_consumed,
                })
            }
            Err(_timeout_elapsed) => {
                // Wall-clock timeout exceeded
                let fuel_consumed = store.fuel_consumed().ok();
                Err(ComponentError::Timeout {
                    max_execution_ms: limits.max_execution_time_ms,
                    fuel_consumed,
                })
            }
        }
    }
}

/// Resource limits for component execution
pub struct ResourceLimits {
    // Memory limits (from Component.toml - MANDATORY)
    pub max_memory: u64,
    pub max_stack: u64,
    pub initial_memory: u64,
    
    // CPU limits (from Component.toml or defaults)
    pub max_fuel_per_execution: u64,
    pub max_execution_time_ms: u64,
}

/// Default CPU limits (memory has no defaults)
const DEFAULT_FUEL_PER_EXECUTION: u64 = 1_000_000;        // 1M fuel units
const DEFAULT_EXECUTION_TIMEOUT_MS: u64 = 100;             // 100ms timeout
```

**Fuel Calibration Examples:**
```rust
// Rough calibration (platform-dependent):
// - Simple arithmetic: ~1-5 fuel per operation
// - Function call: ~50-100 fuel
// - Memory access: ~1-2 fuel
// - 1M fuel ≈ 10-50ms CPU time (depends on operations)

// For CPU-intensive components:
[resources]
max_fuel_per_execution = 10_000_000    # 10M fuel for heavy computation
max_execution_time_ms = 500            # 500ms timeout

// For I/O-bound components:
[resources]
max_fuel_per_execution = 1_000_000     # 1M fuel (light computation)
max_execution_time_ms = 5000           # 5s timeout (network calls)
```

#### 4. Enforcement Level: Host Runtime Only (Phase 1)

**Decision:** Resource limits enforced by host runtime only. Components cannot override.

**Rationale:**
- Simple security model (no negotiation complexity)
- Administrator control over resource allocation
- YAGNI: Can add negotiation later if needed (Kubernetes-style requests/limits)
- Clear separation: Engineer declares needs, host enforces limits

**Implementation:**
```rust
pub struct ComponentLoader {
    engine: Engine,
}

impl ComponentLoader {
    pub async fn load_and_validate(
        &self,
        manifest: ComponentManifest,
    ) -> Result<LoadedComponent> {
        // 1. Validate manifest (memory limits REQUIRED)
        validate_component_manifest(&manifest)?;
        
        // 2. Extract limits from manifest
        let limits = ResourceLimits::from_manifest(&manifest)?;
        
        // 3. Host enforces these limits (no override)
        let mut store = Store::new(&self.engine, HostState::new());
        
        // Configure resource limiter
        let limiter = ComponentResourceLimiter {
            max_memory: limits.max_memory,
            max_tables: 100,  // Reasonable default
            max_instances: 1000,
        };
        store.limiter(move |_state| &limiter);
        
        // 4. Load component with enforced limits
        let component = Component::from_file(&self.engine, &manifest.wasm_path)?;
        
        Ok(LoadedComponent {
            component,
            store,
            limits,
            manifest,
        })
    }
}

/// Resource limiter implementation
pub struct ComponentResourceLimiter {
    max_memory: usize,
    max_tables: usize,
    max_instances: usize,
}

impl wasmtime::ResourceLimiter for ComponentResourceLimiter {
    fn memory_growing(
        &mut self,
        current: usize,
        desired: usize,
        _maximum: Option<usize>
    ) -> Result<bool> {
        // Host enforces memory limit from manifest
        Ok(desired <= self.max_memory)
    }
    
    fn table_growing(
        &mut self,
        current: u32,
        desired: u32,
        _maximum: Option<u32>
    ) -> Result<bool> {
        Ok(desired <= self.max_tables as u32)
    }
    
    fn instances(&self) -> usize {
        self.max_instances
    }
    
    fn tables(&self) -> usize {
        self.max_tables
    }
    
    fn memories(&self) -> usize {
        1  // One linear memory per component
    }
}
```

#### 5. Component Model Version: Latest Stable

**Decision:** Use latest stable Wasmtime version, track stable releases.

**Rationale:**
- Component Model is mature enough for production use
- Benefit from bug fixes and improvements
- Active security patches and updates
- Ecosystem alignment with latest standards

**Dependency Specification:**
```toml
[dependencies]
# Use latest stable with Component Model features
wasmtime = { version = "24.0", features = ["component-model", "async", "cranelift"] }
wasmtime-wasi = { version = "24.0" }
wit-bindgen = { version = "0.30" }
wit-component = { version = "0.200" }

# Update strategy: Track stable releases
# Semver: Major version changes require testing
# Minor/patch: Update regularly for security and bug fixes
```

#### 6. Error Handling: Isolated Component Crashes

**Decision:** Component failures are isolated and don't crash the host runtime.

**Rationale:**
- Host stability is paramount
- Individual component failures shouldn't cascade
- Aligns with supervisor pattern (airssys-rt integration)
- Production resilience (one bad component doesn't kill system)

**Implementation:**
```rust
/// Component supervisor handles failures gracefully
pub struct ComponentSupervisor {
    registry: Arc<ComponentRegistry>,
    metrics: Arc<ComponentMetrics>,
}

impl ComponentSupervisor {
    pub async fn supervise_execution(
        &self,
        component_id: ComponentId,
        input: ComponentInput,
    ) -> Result<ComponentOutput> {
        match self.execute_component(&component_id, input).await {
            Ok(output) => {
                // Success path
                self.metrics.record_success(&component_id);
                Ok(output)
            }
            
            // Component trapped (isolated failure)
            Err(ComponentError::Trapped { reason, fuel_consumed }) => {
                // Log error with context
                log::error!(
                    component_id = %component_id,
                    reason = %reason,
                    fuel_consumed = ?fuel_consumed,
                    "Component trapped during execution"
                );
                
                // Update metrics
                self.metrics.component_trap_count.inc();
                
                // Supervisor decision: Restart component instance
                // (Aligns with airssys-rt supervisor strategies)
                self.handle_component_failure(&component_id, &reason).await?;
                
                // Return error to caller (don't crash host)
                Err(ComponentError::ExecutionFailed {
                    component_id,
                    reason: format!("Component trapped: {}", reason),
                })
            }
            
            // Component timeout (isolated failure)
            Err(ComponentError::Timeout { max_execution_ms, fuel_consumed }) => {
                log::error!(
                    component_id = %component_id,
                    max_execution_ms = max_execution_ms,
                    fuel_consumed = ?fuel_consumed,
                    "Component exceeded execution timeout"
                );
                
                // Update metrics
                self.metrics.component_timeout_count.inc();
                
                // Force terminate component instance
                self.terminate_component(&component_id).await?;
                
                // Return timeout error
                Err(ComponentError::Timeout {
                    max_execution_ms,
                    fuel_consumed,
                })
            }
            
            // Runtime error (serious - may need recovery)
            Err(ComponentError::RuntimeError { message }) => {
                log::critical!(
                    message = %message,
                    "Critical runtime error in component execution"
                );
                
                // Runtime errors may indicate host-level issues
                // May need to restart component registry or escalate
                self.metrics.runtime_error_count.inc();
                
                Err(ComponentError::RuntimeError { message })
            }
        }
    }
    
    async fn handle_component_failure(
        &self,
        component_id: &ComponentId,
        reason: &str,
    ) -> Result<()> {
        // Supervisor strategy: Restart component instance
        // Similar to airssys-rt supervisor patterns (one-for-one, one-for-all, etc.)
        log::info!(
            component_id = %component_id,
            "Restarting failed component instance"
        );
        
        self.registry.restart_component(component_id).await?;
        
        Ok(())
    }
    
    async fn terminate_component(&self, component_id: &ComponentId) -> Result<()> {
        // Force termination for timeout/unrecoverable errors
        self.registry.terminate_component(component_id).await?;
        Ok(())
    }
}
```

**Supervisor Integration with airssys-rt:**
```rust
// Future integration: Use airssys-rt supervisor for component lifecycle
use airssys_rt::supervisor::{Supervisor, SupervisorStrategy};

pub struct WasmComponentActor {
    component_id: ComponentId,
    executor: Arc<ComponentExecutor>,
}

impl Actor for WasmComponentActor {
    async fn handle(&mut self, msg: Message) -> Result<()> {
        // Component execution wrapped in actor
        match self.executor.execute(&self.component_id, msg.input).await {
            Ok(output) => {
                // Send response
                msg.reply(output).await
            }
            Err(ComponentError::Trapped { .. }) => {
                // Actor crashes, supervisor handles restart
                Err(ActorError::ComponentTrapped)
            }
            Err(ComponentError::Timeout { .. }) => {
                // Actor crashes, supervisor handles restart
                Err(ActorError::ComponentTimeout)
            }
        }
    }
}

// Supervisor configuration for component actors
let supervisor = Supervisor::builder()
    .strategy(SupervisorStrategy::OneForOne)  // Restart individual components
    .max_restarts(3)                          // Limit restart attempts
    .within_duration(Duration::from_secs(60)) // Within 1 minute
    .build();
```

---

## Alternatives Considered

### Alternative 1: Wasmer

**Overview:**
- Maintainer: Wasmer Inc.
- Language: Rust
- License: MIT
- Focus: Universal WASM runtime

**Evaluation:**

**Component Model Support:**
- ⚠️ Partial Component Model support (experimental in v4.x)
- ⚠️ Limited WIT bindgen integration (community-driven)
- ⚠️ Incomplete interface types (lagging behind specification)
- ❌ Not the reference implementation

**WASI Preview 2:**
- ⚠️ Partial WASI Preview 2 (catching up to spec)
- ⚠️ Some interfaces missing or experimental
- ⚠️ Less comprehensive than Wasmtime

**Performance:**
```
Instantiation: ~8-12ms (slower than Wasmtime)
Function calls: ~80-150ns overhead
Throughput: >30K calls/sec (good but lower)
```

**Pros:**
- ✅ Multiple compiler backends (Cranelift, LLVM, Singlepass)
- ✅ Good cross-compilation support
- ✅ Wasmer Edge (WAPM package registry)
- ✅ Production users (Cloudflare, CosmWasm)

**Cons:**
- ❌ **Lagging Component Model support** (critical blocker)
- ❌ **Incomplete WASI Preview 2** (missing features we need)
- ⚠️ Less mature async integration
- ⚠️ Slower performance than Wasmtime
- ⚠️ Smaller production footprint

**Why Rejected:**
Component Model is critical for our architecture. Wasmer's incomplete implementation is a blocker. Wasmtime is the reference implementation with complete Component Model and WASI Preview 2 support.

---

### Alternative 2: wasm3 (Interpreter)

**Overview:**
- Type: Interpreter (not JIT)
- Language: C
- Focus: Embedded systems, minimal footprint

**Evaluation:**

**Characteristics:**
- ✅ Tiny footprint (~64KB)
- ✅ Low memory overhead
- ❌ No Component Model support
- ❌ No WASI Preview 2
- ❌ 10-100x slower than JIT runtimes

**Performance:**
```
Instantiation: ~1ms (fast)
Execution: 10-100x slower than JIT (interpreted)
Use case: Embedded, not general-purpose
```

**Why Rejected:**
Interpreter performance is too slow for general-purpose component framework. No Component Model or WASI Preview 2 support. Not suitable for our requirements.

---

### Alternative 3: WasmEdge

**Overview:**
- Maintainer: CNCF (Cloud Native Computing Foundation)
- Language: C++
- Focus: Edge computing, cloud-native

**Evaluation:**

**Component Model:**
- ⚠️ Limited Component Model (experimental)
- ⚠️ Not primary focus (focused on edge/AI features)

**Characteristics:**
- ✅ Good performance (competitive with Wasmtime)
- ✅ Broad language support
- ❌ C++ codebase (harder Rust integration)
- ❌ Not Component Model reference implementation
- ⚠️ Different focus (AI inference, edge functions)

**Why Rejected:**
C++ integration adds complexity. Not the Component Model leader. Different focus than our needs. Wasmtime is better fit for Rust ecosystem.

---

### Alternative 4: Custom Runtime (Build Our Own)

**Approach:**
Build custom WASM runtime from scratch or fork existing runtime.

**Evaluation:**

**Pros:**
- ✅ Full control over features
- ✅ Optimized for exact needs
- ✅ No external dependencies

**Cons:**
- ❌ **Massive development effort** (1-2 years minimum)
- ❌ **Security risks** (no professional audits, DIY sandbox)
- ❌ **Maintenance burden** (keep up with WASM spec evolution)
- ❌ **Component Model complexity** (thousands of engineering hours)
- ❌ **No production validation** (untested in real-world)
- ❌ **Reinventing the wheel** (mature solutions exist)

**Why Rejected:**
Not practical. High risk, low reward. Wasmtime provides everything we need with production validation and active security maintenance. Custom runtime would divert resources from framework features to low-level runtime engineering.

---

## Consequences

### Positive Consequences

✅ **Component Model Reference Implementation**
- Guaranteed compatibility with Component Model specification
- Best WIT bindgen integration and tooling
- Future-proof as spec evolves

✅ **Production-Proven Stability**
- Used at scale: Fastly (millions req/day), Shopify, Cloudflare
- Regular security audits by Trail of Bits and others
- Active development with monthly releases

✅ **Perfect Async Integration**
- First-class async/await support
- Seamless Tokio integration (critical for airssys-rt)
- Non-blocking I/O enables high concurrency
- Cancellation support via Tokio primitives

✅ **Comprehensive WASI Preview 2**
- Complete system interface implementation
- Capability-based security (perfect for our model)
- All interfaces we need (filesystem, network, clocks, random)

✅ **Excellent Performance**
- Near-native execution (95-98% of native Rust)
- ~8ms instantiation (JIT compilation)
- ~50ns function call overhead (negligible)
- Meets baseline performance needs

✅ **Pure Rust Ecosystem Fit**
- No FFI complexity (C/C++ bindings)
- Standard Rust patterns and idioms
- Excellent error handling with thiserror
- Strong type safety guarantees

✅ **Mandatory Memory Limits**
- Forces resource awareness during development
- No silent defaults hiding memory costs
- Prevents "works on my machine" surprises
- Engineering-driven resource decisions

✅ **Hybrid CPU Limiting**
- Fuel metering: Deterministic, can't bypass
- Wall-clock timeout: Protects against slow I/O
- Dual protection: Best of both approaches
- Complementary safety guarantees

✅ **Isolated Component Crashes**
- Host stability guaranteed
- Individual failures don't cascade
- Perfect for supervisor pattern
- Production resilience

✅ **Active Community & Documentation**
- Large community (1.5K+ GitHub stars)
- Excellent official documentation (Wasmtime book)
- Responsive maintainers and issue resolution
- Rich examples and tutorials

### Negative Consequences

⚠️ **Mandatory Memory Limits - Developer Burden**
- **Issue:** Components won't load without explicit memory declarations
- **Impact:** More work for component developers
- **Mitigation:** 
  - Clear error messages with examples
  - Documentation with memory sizing guidelines
  - Component templates with sensible examples
  - CLI validation during development

⚠️ **Dual CPU Limiting Overhead**
- **Issue:** Small performance overhead (~1-2%) for fuel + timeout checks
- **Impact:** Slight execution overhead for all components
- **Mitigation:** 
  - Performance overhead is acceptable for safety guarantees
  - Can disable timeout for trusted internal components if needed
  - Benchmark and optimize if becomes bottleneck

⚠️ **Rust-Only Runtime**
- **Issue:** Host runtime must be Rust (can't use runtime from other languages)
- **Impact:** Locks us into Rust for host development
- **Mitigation:** 
  - This is acceptable - host is always Rust by design
  - Components can be any language (Rust, JS, Go, Python, etc.)
  - No real limitation for our architecture

⚠️ **JIT Compilation Overhead**
- **Issue:** First instantiation includes compilation time (~8ms)
- **Impact:** Cold start latency for first component load
- **Mitigation:**
  - 8ms is acceptable for baseline ("fast enough")
  - Can add AOT pre-compilation later if proven necessary
  - YAGNI: Don't optimize prematurely

⚠️ **No Resource Negotiation (Phase 1)**
- **Issue:** Host enforces limits without negotiation (less flexible than Kubernetes requests/limits)
- **Impact:** Can't dynamically adjust limits based on host capacity
- **Mitigation:**
  - YAGNI: Start simple, add negotiation later if needed
  - Engineer-defined limits work for Phase 1
  - Can add Kubernetes-style requests/limits in Phase 2+

⚠️ **Version Tracking Maintenance**
- **Issue:** Need to track and test Wasmtime upgrades
- **Impact:** Maintenance overhead for version updates
- **Mitigation:**
  - Benefit from bug fixes and security patches outweighs cost
  - Test suite will catch breaking changes
  - Wasmtime has good backward compatibility track record

### Neutral Consequences

📝 **Binary Size**
- Wasmtime runtime adds ~5MB to binary size
- Acceptable for system framework (not embedded)
- Trade-off: features and maturity > binary size

📝 **Learning Curve**
- Component Model has learning curve for developers
- Wasmtime has best documentation available
- Investment pays off with better architecture

📝 **Platform Support**
- Wasmtime supports major platforms (x86_64, ARM64, macOS, Linux, Windows)
- Sufficient for target deployment environments
- Embedded platforms may need alternative (future consideration)

---

## Implementation Guidance

### Phase 1: Core Runtime Integration (Weeks 1-2)

**Tasks:**
1. Add Wasmtime dependencies to Cargo.toml
2. Create engine configuration module
3. Implement resource limiter
4. Add Component.toml validation
5. Build component loader with limits enforcement
6. Add basic error handling and metrics

**Code Structure:**
```rust
airssys-wasm/src/
├── runtime/
│   ├── mod.rs                    # Public API
│   ├── engine.rs                 # Wasmtime engine configuration
│   ├── loader.rs                 # Component loading and validation
│   ├── executor.rs               # Component execution with limits
│   ├── resource_limiter.rs       # Memory/table limiting
│   └── errors.rs                 # Runtime error types
├── config/
│   ├── manifest.rs               # Component.toml parsing
│   └── limits.rs                 # ResourceLimits types
└── supervisor/
    ├── mod.rs                    # Component supervision
    └── metrics.rs                # Execution metrics
```

### Phase 2: Async Integration (Weeks 3-4)

**Tasks:**
1. Implement async host functions (filesystem, network)
2. Integrate with Tokio runtime
3. Add timeout handling
4. Build cancellation support
5. Test concurrent component execution

### Phase 3: Supervisor Integration (Weeks 5-6)

**Tasks:**
1. Implement component supervisor
2. Add restart strategies
3. Integrate with airssys-rt supervisor patterns
4. Add health checking
5. Build component lifecycle management

### Testing Strategy

**Unit Tests:**
- Engine configuration validation
- Resource limiter enforcement
- Manifest parsing and validation
- Error handling paths

**Integration Tests:**
- Component loading and instantiation
- Execution with resource limits
- Timeout and fuel exhaustion
- Crash isolation (component failures don't crash host)
- Concurrent component execution

**Performance Tests:**
- Instantiation time baseline
- Function call overhead measurement
- Memory usage profiling
- Throughput benchmarks (calls/sec)

**Security Tests:**
- Memory limit enforcement (component can't exceed max_memory)
- Fuel limit enforcement (component traps on exhaustion)
- Timeout enforcement (slow components terminated)
- Crash isolation (trapped components don't affect others)

---

## Future Enhancements

### Phase 2: AOT Compilation Support

**When:** If instantiation speed becomes bottleneck (measure first)

**Implementation:**
```rust
pub enum ComponentSource {
    WasmBytecode(Vec<u8>),      // JIT: Compile at runtime
    Precompiled(Vec<u8>),        // AOT: Pre-compiled native code
}

impl ComponentLoader {
    pub async fn load(&self, source: ComponentSource) -> Result<Component> {
        match source {
            ComponentSource::WasmBytecode(bytes) => {
                // JIT compilation (current)
                Component::new(&self.engine, bytes)
            }
            ComponentSource::Precompiled(bytes) => {
                // AOT: Deserialize pre-compiled (future)
                unsafe { Component::deserialize(&self.engine, bytes) }
            }
        }
    }
}
```

### Phase 3: Resource Negotiation

**When:** If static limits prove too inflexible (based on feedback)

**Approach:**
```toml
# Component.toml: Request resources
[resources.requests]
max_memory = "128MB"    # Component requests

[resources.limits]
max_memory = "256MB"    # Host can override (Kubernetes-style)
```

### Phase 4: Advanced Monitoring

**When:** Production deployment requires deeper observability

**Features:**
- Per-component performance profiling
- Execution trace collection
- Resource usage analytics
- Predictive scaling based on metrics

---

## References

### Official Documentation
- **Wasmtime Book**: https://docs.wasmtime.dev/
- **Component Model Specification**: https://github.com/WebAssembly/component-model
- **WASI Preview 2**: https://github.com/WebAssembly/WASI/blob/main/Preview2.md
- **WIT Language Guide**: https://component-model.bytecodealliance.org/design/wit.html

### Related ADRs
- **ADR-WASM-001**: Multicodec Compatibility Strategy
- **ADR-WASM-005** (Planned): Capability-Based Security Model
- **ADR-WASM-007** (Planned): Storage Backend Selection

### Related Knowledge
- **KNOWLEDGE-WASM-001**: Component Framework Architecture
- **KNOWLEDGE-WASM-003**: Core Architecture Design
- **KNOWLEDGE-WASM-004**: WIT Management Architecture

### Production Examples
- **Fastly Compute@Edge**: Edge computing with Wasmtime
- **Shopify Functions**: E-commerce customization at scale
- **Cloudflare Workers**: Serverless edge platform
- **SingleStore**: WASM in database (stored procedures)

---

## Decision Log

| Date | Decision | Participants |
|------|----------|--------------|
| 2025-10-19 | Primary runtime: Wasmtime | Architecture Team |
| 2025-10-19 | Compilation: JIT with Cranelift | Architecture Team |
| 2025-10-19 | Async: Mandatory async-first | Architecture Team |
| 2025-10-19 | Memory: Mandatory engineer-defined limits | Architecture Team |
| 2025-10-19 | CPU: Hybrid fuel + timeout | Architecture Team |
| 2025-10-19 | Enforcement: Host runtime only | Architecture Team |
| 2025-10-19 | Version: Latest stable | Architecture Team |
| 2025-10-19 | Errors: Isolated component crashes | Architecture Team |

---

**Status:** ✅ **Accepted**  
**Implementation Priority:** Critical (Phase 1 Foundation)  
**Next Review:** After 6 months production use or if performance bottlenecks identified

---

## Appendix: Wasmtime Configuration Reference

### Recommended Production Configuration

```rust
use wasmtime::{Config, Engine, OptLevel, Strategy};

pub fn production_engine_config() -> Result<Engine> {
    let mut config = Config::new();
    
    // === Core Features ===
    config.wasm_component_model(true);
    config.async_support(true);
    
    // === Compilation ===
    config.strategy(Strategy::Cranelift);
    config.cranelift_opt_level(OptLevel::Speed);
    
    // === Security ===
    config.consume_fuel(true);
    config.max_wasm_stack(2 * 1024 * 1024);  // 2MB
    
    // === Performance ===
    config.parallel_compilation(true);
    config.cache_config_load_default()?;  // Enable compilation cache
    
    // === Debugging (disable in production) ===
    // config.debug_info(true);
    // config.wasm_backtrace(true);
    
    Ok(Engine::new(&config)?)
}
```

### Development Configuration

```rust
pub fn development_engine_config() -> Result<Engine> {
    let mut config = Config::new();
    
    // Same as production but with debugging
    config.wasm_component_model(true);
    config.async_support(true);
    config.strategy(Strategy::Cranelift);
    config.cranelift_opt_level(OptLevel::Speed);
    config.consume_fuel(true);
    
    // Enable debugging
    config.debug_info(true);
    config.wasm_backtrace(true);
    config.wasm_backtrace_details(wasmtime::WasmBacktraceDetails::Enable);
    
    Ok(Engine::new(&config)?)
}
```
