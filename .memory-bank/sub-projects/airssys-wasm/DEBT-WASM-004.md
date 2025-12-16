# DEBT-WASM-004: Example Implementation Quality

**Status**: 🔴 OPEN  
**Priority**: Medium  
**Category**: Code Quality / Examples  
**Created**: 2025-12-17  
**Estimated Effort**: 2-3 hours  
**Blocks**: None (documentation complete and approved)  

---

## Summary

Two examples (`supervised_component.rs` and `component_composition.rs`) use "simulation approach" (printing what would happen) instead of implementing real Actor trait integrations with actual component spawning and message passing.

---

## Problem Description

### What Was Delivered

**Files**:
- `airssys-wasm/examples/supervised_component.rs` (200 lines)
- `airssys-wasm/examples/component_composition.rs` (380 lines)

**Current Approach**:
- Examples demonstrate SupervisorConfig and ComponentRegistry APIs
- They print explanatory text showing what "would happen" in a real system
- They do NOT implement Actor trait or spawn real components
- They do NOT demonstrate actual message passing or supervision behavior

**Example of Current Code**:
```rust
// Current: Simulation approach
println!("1. Component crashes (panic or error)");
println!("2. Supervisor detects failure");
println!("3. Applies restart strategy (exponential backoff)");
// ^ Just prints text, doesn't actually do it
```

### What Should Exist

**Desired Approach**:
- Examples should follow the pattern of existing REAL examples:
  - `actor_routing_example.rs` - Actually routes messages
  - `actor_supervision_example.rs` - Actually demonstrates supervision
  - `supervisor_node_integration.rs` - Actually integrates supervisor

**Example of Desired Code**:
```rust
// Desired: Real implementation
let system = ActorSystem::new("example");
let component = MyComponent::new();
let actor_ref = system.spawn_actor("component", component).await?;

// Actually crash and observe restart
actor_ref.send(CrashMessage).await?;
// Supervisor automatically restarts component
```

---

## Impact Assessment

### Current Impact: **LOW**

**Why Low**:
1. ✅ Documentation quality is excellent (9.7/10)
2. ✅ Three existing examples ARE real implementations
3. ✅ Examples compile and run without errors
4. ✅ Examples teach the concepts effectively
5. ✅ Phase 6 Task 6.3 is complete and approved

**Affected Areas**:
- Developer learning experience (slightly reduced)
- Example quality consistency (inconsistent with existing examples)

### Future Impact: **MEDIUM**

**If Not Fixed**:
- Developers may copy "simulation pattern" instead of real Actor implementations
- Examples don't demonstrate full ComponentActor capabilities
- Inconsistency with existing high-quality examples
- Missed opportunity to showcase actual system behavior

---

## Root Cause Analysis

### Why This Happened

1. **Implementer Hit Technical Issue**:
   - Tried to implement Actor trait
   - Encountered signature mismatch with expected API
   - Made decision to use "simulation" instead of resolving issue

2. **Justification Was Incorrect**:
   - Claimed "this matches other examples" 
   - But existing examples (actor_routing_example.rs, etc.) ARE real implementations
   - Audit identified this as technical debt

3. **Time Pressure**:
   - Task 6.3 had 4 checkpoints to complete
   - Simulation approach was faster than resolving Actor trait issue

---

## Recommended Solution

### Approach: Rewrite Examples as Real Implementations

**Step 1**: Study Existing Real Examples (30 min)
- Review `actor_routing_example.rs`
- Review `actor_supervision_example.rs`
- Review `supervisor_node_integration.rs`
- Understand Actor trait signatures from airssys-rt

**Step 2**: Rewrite `supervised_component.rs` (1-1.5 hours)

**New Structure**:
```rust
//! # Supervised Component Example
//! Demonstrates actual component supervision with crash and restart

use airssys_rt::actor::{Actor, Child, Context};
use airssys_rt::system::ActorSystem;
use airssys_wasm::actor::{ComponentSupervisor, SupervisorConfig};
use async_trait::async_trait;

// Define a component that can crash
#[derive(Clone)]
struct CrashableComponent {
    id: ComponentId,
    crash_after: Arc<AtomicU32>,
}

impl Child for CrashableComponent {
    fn pre_start(&mut self) {
        println!("Component starting: {}", self.id.as_str());
    }
    
    fn post_stop(&mut self) {
        println!("Component stopped: {}", self.id.as_str());
    }
}

#[async_trait]
impl Actor for CrashableComponent {
    type Message = ComponentMessage;
    
    async fn handle_message(&mut self, msg: Self::Message, _ctx: &Context) -> Result<(), ActorError> {
        match msg {
            ComponentMessage::Process(data) => {
                let count = self.crash_after.fetch_sub(1, Ordering::SeqCst);
                if count == 0 {
                    panic!("Simulated crash!"); // Supervisor will restart us
                }
                println!("Processed: {:?}", data);
                Ok(())
            }
            _ => Ok(())
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Supervised Component Example ===\n");
    
    // Create ActorSystem with supervision
    let system = ActorSystem::new("supervised-example");
    
    // Configure permanent supervision (always restart)
    let config = SupervisorConfig::permanent()
        .with_max_restarts(3)
        .with_time_window(Duration::from_secs(10));
    
    // Create component that crashes after 3 messages
    let component = CrashableComponent {
        id: ComponentId::new("crashable"),
        crash_after: Arc::new(AtomicU32::new(3)),
    };
    
    // Spawn with supervision
    let actor_ref = system.spawn_supervised("crashable", component, config).await?;
    
    // Send messages (will crash on 3rd message and restart)
    for i in 1..=5 {
        println!("\nSending message {}...", i);
        actor_ref.send(ComponentMessage::Process(vec![i])).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    
    println!("\n✅ Component crashed and was automatically restarted by supervisor");
    
    Ok(())
}
```

**Step 3**: Rewrite `component_composition.rs` (1-1.5 hours)

**New Structure**:
```rust
//! # Component Composition Example
//! Demonstrates actual pipeline of components with real message passing

// Create 3 components: Input → Processor → Output
// Actually spawn them and route messages through the pipeline
// Show real coordination and message flow
```

**Step 4**: Test and Verify (30 min)
- Compile both examples: `cargo build --examples`
- Run both examples: `cargo run --example supervised_component`
- Verify zero warnings: `cargo clippy --examples -- -D warnings`
- Confirm output shows ACTUAL behavior, not simulation

---

## Acceptance Criteria

**This debt is resolved when**:

1. ✅ `supervised_component.rs` actually spawns a component with ActorSystem
2. ✅ Component actually crashes and supervisor actually restarts it
3. ✅ `component_composition.rs` actually creates pipeline and routes messages
4. ✅ Both examples compile and run with zero warnings
5. ✅ Examples follow pattern of existing real examples
6. ✅ Examples demonstrate full Actor trait integration
7. ✅ Output shows ACTUAL system behavior (not "what would happen")

---

## Dependencies

### Required Knowledge
- Actor trait signature from airssys-rt
- ActorSystem spawning patterns
- SupervisorConfig integration
- MessageRouter usage

### Reference Examples
- `airssys-wasm/examples/actor_routing_example.rs` (REAL example)
- `airssys-wasm/examples/actor_supervision_example.rs` (REAL example)
- `airssys-wasm/examples/supervisor_node_integration.rs` (REAL example)

---

## Priority Justification

**Why Medium (not High)**:
1. Documentation task (6.3) is complete and approved
2. Three existing examples ARE real implementations
3. Examples do compile and run successfully
4. Developer onboarding is not blocked

**Why Medium (not Low)**:
1. Inconsistency with existing example quality
2. Examples don't demonstrate full system capabilities
3. Risk of developers copying "simulation pattern"
4. Relatively quick fix (2-3 hours)

---

## Blocking Status

**Does NOT Block**:
- ✅ Phase 6 completion (already done)
- ✅ Phase 7 start (can proceed)
- ✅ Block 4 planning (independent)
- ✅ Documentation quality (approved at 9.7/10)

**Should Be Fixed Before**:
- ⏳ Production release
- ⏳ Public documentation release
- ⏳ Developer onboarding materials finalized

---

## Related Items

- **Task**: WASM-TASK-004 Phase 6 Task 6.3 ✅ COMPLETE
- **Audit**: AUDIT-REPORT-task-004-phase-6-task-6.3-checkpoints-3-4.md (approved 9.7/10)
- **Status**: WASM-TASK-004-STATUS.md (Phase 6 complete)
- **Examples to Study**: actor_routing_example.rs, actor_supervision_example.rs

---

## Resolution Plan

**When to Address**: Next refactoring cycle or before public release

**Assignee**: TBD

**Steps**:
1. Study existing real examples (30 min)
2. Rewrite supervised_component.rs (1-1.5 hours)
3. Rewrite component_composition.rs (1-1.5 hours)
4. Test and verify (30 min)

**Total Estimated Time**: 2-3 hours

---

## Notes

- This was identified during audit of Task 6.3 Checkpoints 3 & 4
- User explicitly requested this be marked as technical debt
- User mandate: "MUST BE FIX in the further development"
- Examples were created under time pressure during documentation sprint
- Implementer made pragmatic choice to deliver documentation on schedule
- Quality of documentation (9.7/10) validates that decision
- But examples should be upgraded when time permits

---

**Created**: 2025-12-17  
**Last Updated**: 2025-12-17  
**User Requirement**: MUST BE FIXED in further development  
**Status**: 🔴 OPEN
