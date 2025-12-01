# ADR-WASM-006: Component Isolation and Sandboxing

**Status:** Accepted (Revised)  
**Date:** 2025-10-19 (Original), 2025-10-19 (Revised)  
**Decision Makers:** Architecture Team  
**Related:** ADR-WASM-002 (Runtime Engine), ADR-WASM-003 (Lifecycle), ADR-WASM-005 (Security), ADR-WASM-007 (Storage), ADR-WASM-009 (Communication), ADR-RT-004 (Actor/Child Separation), KNOWLEDGE-RT-013 (Actor System Performance)

---

## Context

The airssys-wasm framework must ensure strong isolation between WASM components and between components and the host system. Components may originate from untrusted sources, contain bugs, or exhibit malicious behavior. Without proper isolation, a misbehaving component could:

- Access memory or data from other components
- Consume excessive system resources (CPU, memory, disk, network)
- Interfere with host system operations
- Bypass security policies and access unauthorized resources
- Cause system-wide failures or denial of service

### The Challenge

WASM provides memory safety and sandboxing at the WebAssembly level, but several isolation concerns remain:

**Memory Isolation:**
- WASM linear memory is isolated per instance
- BUT: Host functions can share data between components if not careful
- RISK: Malicious components accessing other components' data

**Process Isolation:**
- Should each component run in a separate lightweight process (Erlang-style actor)?
- OR: Should we use OS-level process isolation?
- TRADEOFF: Security vs Performance vs Complexity

**Resource Limits:**
- WASM runtime can limit memory/CPU within the sandbox
- BUT: Storage, network, file handles managed by host
- QUESTION: Where to enforce resource quotas?

**Inter-Component Communication:**
- Components may need to call other components
- Must prevent unauthorized cross-component access
- QUESTION: How to route and validate component-to-component calls?

### Requirements

**Security Requirements:**
- Strong isolation between components (no cross-component data access)
- Resource exhaustion prevention (quotas and limits)
- Failure isolation (component crash doesn't affect others)
- Security policy enforcement (capability-based access control)

**Performance Requirements:**
- Low overhead for isolation mechanisms (<5% performance impact)
- Fast component spawning (<100ms startup time target, <20ms achieved)
- Efficient inter-component communication (<1ms latency)
- Support for 10,000+ concurrent components on modest hardware

**Operational Requirements:**
- Platform compatibility (Linux, macOS, Windows)
- Integration with existing airssys-rt actor system and supervision
- Monitoring and auditing of resource usage
- Graceful degradation under resource pressure

### Architectural Context: Integration with airssys-rt

The airssys-wasm framework builds on top of airssys-rt, a lightweight Erlang-Actor model runtime. Key characteristics from airssys-rt:

- **Lightweight processes**: Tokio tasks (green threads), NOT OS processes
- **Actor spawn**: ~625ns per actor (KNOWLEDGE-RT-013 §6.1)
- **Message passing**: ~211ns broker routing (RT-TASK-008 baseline)
- **Supervision trees**: BEAM-inspired fault tolerance with automatic restart
- **Concurrent actors**: 10,000+ proven in benchmarks (BENCHMARKING.md)
- **Memory per actor**: <1KB overhead
- **Scaling**: Linear with 6% overhead (1→50 actors)

**Critical Insight:** We should leverage airssys-rt's proven lightweight process model, NOT introduce heavy OS process isolation.

---

## Decision

### Core Decision: Erlang-Style Lightweight Process Isolation with 4-Layer Defense

**We will implement Erlang-style lightweight process isolation using airssys-rt actors, combined with WASM linear memory sandboxing and hybrid resource enforcement.**

### Revision Note

**Original Proposal (Oct 19, 2025 - Morning):** Initially proposed OS process isolation with systemd-run, cgroups, namespaces.

**Revised Decision (Oct 19, 2025 - Afternoon):** Corrected to use airssys-rt lightweight actor isolation after architectural review. The original proposal confused OS processes with Erlang lightweight processes, which would have resulted in:
- **100,000x slower** spawn times (50-100ms vs 625ns)
- **5,000-10,000x more memory** (5-10MB vs <1KB per actor)
- **Platform-specific complexity** (cgroups/namespaces Linux-only)
- **Poor scalability** (100-500 vs 10,000+ concurrent components)

The revised architecture leverages existing airssys-rt infrastructure for superior performance and cross-platform compatibility.

### Key Design Principles

1. **Leverage Existing Infrastructure**: Use airssys-rt actor system and supervision trees
2. **Defense in Depth**: Multiple isolation layers (Capability → WASM → Actor → Supervision)
3. **Fail-Safe Defaults**: Components isolated by default, communication requires explicit permission
4. **Resource Accountability**: Per-component resource tracking and enforcement
5. **Performance First**: Lightweight isolation with minimal overhead
6. **Platform Portability**: Cross-platform via Tokio + WASM, no OS-specific mechanisms

---

## Detailed Decisions

### Decision 1: Isolation Architecture - 4-Layer Defense in Depth

**Decision:** Each component isolated through four complementary layers.

**Layer 1: Capability-Based Security (ADR-WASM-005)**
- Permission checks at host function boundaries
- Fine-grained resource access control (filesystem paths, network domains, storage namespaces)
- Declarative capabilities in Component.toml manifest
- Runtime enforcement via CapabilityManager

**Layer 2: WASM Linear Memory Sandbox (Wasmtime)**
- Isolated heap per component (512KB-4MB configurable)
- Bounds checking prevents buffer overflows
- No shared memory between components
- Memory-safe execution guaranteed by WASM spec

**Layer 3: Actor Isolation (airssys-rt)**
- Each component = ComponentActor (implements Actor trait)
- Private mailbox per actor (tokio unbounded channel)
- Message passing ONLY (no shared mutable state)
- Actor spawn via ActorSystem::spawn() (~625ns overhead)
- Broker-based message routing (~211ns latency)

**Layer 4: Supervision Trees (airssys-rt)**
- Components managed by SupervisorNode
- Automatic restart on failure (OneForOne strategy)
- Health monitoring via Child::health_check()
- Graceful shutdown with configurable timeout
- "Let it crash" philosophy - failures isolated and recovered

**Performance Characteristics:**

| Metric | Value | Source |
|--------|-------|--------|
| Component spawn | ~2-11 ms | 625ns (actor) + 1-10ms (WASM) |
| Actor overhead | ~625 ns | KNOWLEDGE-RT-013 §6.1 |
| Message routing | ~211 ns | RT-TASK-008 baseline |
| Memory per component | ~1-2 MB | <1KB (actor) + 512KB-2MB (WASM) |
| Concurrent components | 10,000+ | Limited by WASM memory, not actors |
| Restart latency | ~10-50 µs | Supervision overhead only |

**Rationale:**

✅ **Layered security**: Defense in depth with four independent isolation mechanisms  
✅ **Proven performance**: airssys-rt achieves 625ns actor spawn, 10,000+ concurrent actors  
✅ **Platform portability**: Tokio + WASM work on Linux, macOS, Windows (no OS-specific code)  
✅ **Operational simplicity**: Supervision trees provide automatic fault recovery  
✅ **Scalability**: Linear scaling proven in airssys-rt benchmarks (6% overhead 1→50 actors)  

---

### Decision 2: ComponentActor Implementation - Dual Trait Design

**Decision:** ComponentActor implements both Actor (message handling) and Child (supervision) traits, with WASM lifecycle controlled by supervisor.

**Key Insight:** WASM instantiation happens in `Child::start()`, NOT `Actor::pre_start()`.

**Lifecycle Flow:**

```text
1. CREATION PHASE
   ComponentRuntime::install_component()
   └─> ComponentActor::new()
       └─> state = Created
       └─> wasm_runtime = None  ← WASM not loaded yet

2. SUPERVISION START PHASE (WASM loads here!)
   Supervisor::start_child(spec)
   └─> Child::start()
       ├─> Load WASM bytes from storage
       ├─> Create Wasmtime engine
       ├─> Compile module
       ├─> Instantiate with linker (register host functions)
       ├─> Cache exported functions
       ├─> Call _start export (if exists)
       └─> wasm_runtime = Some(runtime)
       └─> state = Running

3. ACTOR START PHASE
   ActorSystem::spawn(component_actor)
   └─> Actor::pre_start()
       ├─> Verify WASM is loaded (sanity check)
       ├─> Register with component registry
       └─> Initialize metrics

4. RUNNING PHASE
   Actor::handle_message()
   └─> ComponentMessage::Invoke { function, args }
       ├─> Layer 1: Check capability permission
       ├─> Layer 2: Execute in WASM sandbox
       └─> Layer 3: Send reply via broker

5. SHUTDOWN PHASE
   ComponentMessage::Shutdown
   └─> ctx.stop() signals actor system
       └─> Supervisor detects child stopping
           └─> Supervisor::stop_child()
               └─> Child::stop(timeout)
                   ├─> Call _cleanup export (if exists)
                   ├─> Drop WasmRuntime (frees linear memory)
                   └─> wasm_runtime = None

6. ACTOR CLEANUP PHASE
   ActorSystem finalizes actor
   └─> Actor::post_stop()
       ├─> Verify WASM cleaned up (assertion)
       └─> Deregister from registry
```

**Benefits:**

✅ **Clear responsibility separation**: Supervisor controls WASM lifecycle, Actor handles messages  
✅ **Supervisor-controlled restart**: Component restarts reload WASM (fresh state, clean slate)  
✅ **Health monitoring integration**: Child::health_check() can call component's _health export  
✅ **Graceful shutdown guarantee**: Child::stop() ensures WASM cleanup with timeout  
✅ **Performance**: WASM loaded once at startup (Child::start), not on every message  

---

### Decision 3: Resource Limits - Hybrid Enforcement (WASM + Application Level)

**Decision:** Enforce resource limits at two levels for defense in depth.

**WASM-Level Limits (Layer 2 - Enforced by Wasmtime):**

- **Memory**: 512KB-4MB linear memory limit
- **CPU**: Fuel metering (~1% overhead per instruction)
- **Execution**: Per-function timeout (configurable)

**Application-Level Limits (Layer 1 - Enforced by Host Functions):**

- **Storage**: 100MB default quota per component
- **Network**: 1GB/day transfer limit, 1000 requests/min rate limit
- **API calls**: 1000/min rate limit for inter-component calls

**Implementation Approach:**

```rust
// WASM-level (Wasmtime Store resource limiter)
pub struct ComponentResourceLimiter {
    max_memory_bytes: usize,        // 4MB default
    max_table_elements: u32,        // 1000 default
}

impl ResourceLimiter for ComponentResourceLimiter {
    fn memory_growing(&mut self, current: usize, desired: usize, _max: Option<usize>) 
        -> Result<bool, Error> {
        Ok(desired <= self.max_memory_bytes)
    }
}

// Application-level (Host function quota tracking)
pub struct ComponentQuotas {
    storage_bytes_used: AtomicU64,
    storage_bytes_limit: u64,  // 100MB default
    network_bytes_sent_today: AtomicU64,
    network_bytes_limit_daily: u64,  // 1GB/day default
}
```

**Performance Overhead:**

| Check Type | Latency | Implementation |
|------------|---------|----------------|
| Memory growth check | ~10 ns | Wasmtime inline check |
| Fuel metering | ~1% overhead | Wasmtime JIT instrumentation |
| Storage quota | ~50 ns | Atomic counter check |
| Network quota | ~100 ns | Atomic counter + timestamp |
| **Total overhead** | **<5%** | Combined across all layers |

**Rationale:**

✅ **Defense in depth**: WASM limits prevent sandbox escape, app limits prevent abuse  
✅ **Fail-safe**: Wasmtime traps on limit violation (doesn't log-and-continue)  
✅ **Performance**: Atomic counters for fast quota checks (<100ns)  
✅ **Granularity**: Different limits for different resource types  

---

### Decision 4: Inter-Component Communication Integration

**Decision:** Inter-component communication uses airssys-rt MessageBroker with host-mediated security enforcement.

**Brief Architecture:**
```text
Component A → host::send_message() → [Capability Check] → MessageBroker → ActorSystem → Component B
```

**Security Enforcement (Host Function Layer):**
- **Capability validation**: Permission checks before message publication (ADR-WASM-005)
- **Quota enforcement**: Per-component API call limits (1000/min default)
- **Audit logging**: All inter-component messages logged for security compliance
- **No direct access**: Components cannot bypass host functions to access broker

**Performance Characteristics:**
- **MessageBroker routing**: ~211ns (proven RT-TASK-008 baseline)
- **Capability validation**: ~50ns (permission lookup + quota check)
- **Total overhead**: ~260ns per message (faster than initial proxy proposal)
- **Throughput**: 4.7M messages/sec (MessageBroker capacity)

**Messaging Patterns Supported:**
- Fire-and-forget (one-way notifications)
- Request-response (async RPC with callbacks)
- Pub-sub broadcasting (event distribution)

**Rationale:**

✅ **Reuses proven infrastructure**: MessageBroker already tested, benchmarked, production-ready  
✅ **Simpler architecture**: No custom ComponentProxyActor needed, clear separation of concerns  
✅ **Better performance**: ~260ns total vs ~330ns with dedicated proxy actor layer  
✅ **Aligned with decisions**: Matches KNOWLEDGE-WASM-005 and airssys-rt MessageBroker design  

**For comprehensive messaging architecture, patterns, serialization, and implementation details, see ADR-WASM-009: Component Communication Model.**

---

### Decision 5: Component Installation and Spawning

**Decision:** Components installed via ComponentRuntime with supervisor-managed lifecycle.

**Installation Flow:**

```text
1. User calls ComponentRuntime::install_component(spec)
   └─> spec contains: id, version, WASM path, capabilities

2. Create ComponentActor (WASM not loaded yet)
   └─> ComponentActor::new(id, spec, capabilities)
   └─> wasm_runtime = None

3. Add to supervisor tree
   └─> Supervisor::start_child(child_spec)
       └─> Calls Child::start()  ← WASM LOADS HERE
           ├─> Load WASM bytes
           ├─> Create Wasmtime engine
           ├─> Compile module
           ├─> Instantiate with linker
           └─> wasm_runtime = Some(runtime)

4. Spawn actor
   └─> ActorSystem::spawn(component_actor)
       └─> Calls Actor::pre_start()  ← Verify WASM loaded

5. Component ready to receive messages via MessageBroker!
```

**Performance:**

| Phase | Time | Details |
|-------|------|---------|
| Create ComponentActor | ~100 ns | Struct creation |
| Supervisor start_child | ~2-11 ms | WASM load + compile + instantiate |
| Actor spawn | ~625 ns | airssys-rt actor spawn |
| **Total installation** | **~2-11 ms** | Dominated by WASM compilation |

**Rationale:**

✅ **Automatic fault recovery**: Supervisor restarts failed components  
✅ **Health monitoring**: Supervisor can poll Child::health_check()  
✅ **Graceful shutdown**: Supervisor ensures WASM cleanup via Child::stop()  
✅ **Fast installation**: ~2-12ms total (10x better than 100ms target)  

---

## Consequences

### Positive Consequences

✅ **Excellent performance**: ~2-11ms component spawn (10x better than 100ms target)  
✅ **High scalability**: 10,000+ concurrent components (100x better than 100 target)  
✅ **Platform portability**: Tokio + WASM work on Linux, macOS, Windows  
✅ **Proven infrastructure**: Leverages airssys-rt's 625ns actor spawn, 211ns message routing  
✅ **Defense in depth**: 4 isolation layers (Capability → WASM → Actor → Supervision)  
✅ **Automatic recovery**: Supervision trees handle component crashes  
✅ **Low memory overhead**: ~1-2MB per component (10x better than OS process approach)  
✅ **Fast inter-component calls**: ~260ns overhead (host validation + MessageBroker routing)  

### Negative Consequences

❌ **Shared address space**: Components in same process (mitigated by WASM sandbox)  
❌ **No kernel-level isolation**: Relies on WASM + runtime enforcement (acceptable tradeoff)  
❌ **Resource limit bypass potential**: Malicious WASM could exploit Wasmtime bugs (mitigated by defense in depth)  

### Neutral Consequences

➖ **Dependency on airssys-rt**: Tightly coupled to actor system (intentional design choice)  
➖ **Supervision overhead**: ~10-50µs restart latency (acceptable for fault tolerance benefits)  
➖ **Two-trait implementation**: ComponentActor implements both Actor and Child (clean separation of concerns)  

---

## Implementation Notes

### Phase 1: Core Infrastructure (Week 1-2)

1. **ComponentActor implementation**
   - Implement Actor trait (message handling)
   - Implement Child trait (WASM lifecycle)
   - WASM loading in Child::start()
   - WASM cleanup in Child::stop()

2. **ComponentRuntime implementation**
   - Actor system integration
   - Supervisor tree setup
   - Component registry
   - Installation/uninstallation flow

3. **Basic resource limits**
   - Wasmtime memory limits
   - Fuel metering for CPU
   - Application-level quota structure

### Phase 2: Resource Enforcement (Week 3-4)

1. **Hybrid resource limits**
   - Storage quota tracking
   - Network quota tracking
   - API call rate limiting
   - Host function integration

2. **MessageBroker integration**
   - Host function security layer
   - Capability validation before publish
   - Quota enforcement
   - Audit logging

### Phase 3: Testing & Validation (Week 5-6)

1. **Unit tests**
   - ComponentActor lifecycle
   - Resource limit enforcement
   - Permission validation

2. **Integration tests**
   - Component installation/uninstallation
   - Fault tolerance (crash recovery)
   - Resource exhaustion scenarios
   - Supervision tree behavior

3. **Performance benchmarks**
   - Component spawn latency
   - Message routing overhead
   - Resource check overhead
   - Concurrent component scalability

---

## References

### ADRs
- ADR-WASM-002: WASM Runtime Engine Selection (Wasmtime, resource limits)
- ADR-WASM-003: Component Lifecycle Management (installation, immutability)
- ADR-WASM-005: Capability-Based Security Model (permission system)
- ADR-WASM-007: Storage Backend Selection (storage quotas)
- ADR-WASM-009: Component Communication Model (inter-component messaging)
- ADR-RT-004: Actor/Child Trait Separation (supervision integration)

### Knowledge Documentation
- KNOWLEDGE-RT-013: Actor System Performance Characteristics (625ns spawn, 211ns routing)
- KNOWLEDGE-WASM-001: Component Framework Architecture

### External References
- [Wasmtime Resource Limits](https://docs.wasmtime.dev/api/wasmtime/struct.ResourceLimiter.html)
- [Erlang/OTP Supervision Principles](https://www.erlang.org/doc/design_principles/sup_princ.html)
- [airssys-rt BENCHMARKING.md](../../airssys-rt/BENCHMARKING.md) - Performance baselines
- [WebAssembly Component Model](https://github.com/WebAssembly/component-model)

---

## Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2025-10-19 | 1.0 (Original) | Initial decision (OS process isolation) | Architecture Team |
| 2025-10-19 | 2.0 (Revised) | Corrected to Erlang-style lightweight isolation | Architecture Team |

**Revision Rationale:** Original decision proposed OS process isolation based on misunderstanding of "process-level isolation" terminology. Architectural review revealed that:
1. airssys-rt uses lightweight Tokio tasks (green threads), not OS processes
2. OS processes would cause 100,000x slower spawn times
3. Existing infrastructure already provides isolation via actors + supervision
4. Platform portability and scalability would be severely compromised

Revised decision aligns with proven airssys-rt architecture and delivers superior performance while maintaining strong isolation guarantees.
