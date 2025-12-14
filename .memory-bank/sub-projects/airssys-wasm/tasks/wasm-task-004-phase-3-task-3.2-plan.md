# WASM-TASK-004 Phase 3 Task 3.2: SupervisorNode Integration - Implementation Plan

**Date:** 2025-12-14  
**Status:** Ready for Implementation  
**Priority:** CRITICAL - Foundation for automatic component restart (Block 3 Phase 3)  
**Estimated Effort:** 8-10 hours  
**Quality Target:** 9.5/10 (Match Phase 3.1 quality)  
**Dependencies:** Phase 3.1 Complete ✅

---

## Executive Summary

This plan details the integration of ComponentSupervisor (Layer 1 - WASM Configuration) with airssys-rt's SupervisorNode (Layer 3 - Actor System Runtime), establishing the bridge that enables automatic WASM component restart through the actor system's supervision infrastructure.

**Architecture Context:**  
Per **ADR-WASM-018** (Three-Layer Architecture):
- **Layer 1** (WASM Config): SupervisorConfig, ComponentSupervisor - POLICY DEFINITION
- **Layer 2** (WASM Lifecycle): ComponentActor, ComponentSpawner - WASM LOADING
- **Layer 3** (Actor Runtime): SupervisorNode, RestartBackoff - SUPERVISION EXECUTION

**Integration Objective:**  
Connect Layer 1's WASM-specific supervision policies to Layer 3's supervision execution engine, ensuring ComponentActor instances are supervised by SupervisorNode while maintaining clear ownership boundaries.

**Key Deliverables:**
1. SupervisorNodeBridge trait for Layer 1 ↔ Layer 3 integration
2. ComponentSupervisor integration with SupervisorNode<OneForOne>
3. RestartPolicy mapping (WASM → airssys-rt)
4. Restart coordination via Child trait callbacks
5. Health-based restart triggering
6. 25+ integration tests with full actor system
7. Comprehensive documentation and examples

**Success Metrics:**
- SupervisorNode manages ComponentActor restart decisions
- Layer boundaries maintained (no Layer 3 logic in Layer 1)
- Restart flow: ComponentActor failure → SupervisorNode → ComponentSupervisor tracking
- All 25+ tests passing (target: 420+ total tests)
- Zero warnings (compiler + clippy)
- Code quality: 9.5/10
- Performance: Restart coordination <10μs overhead

---

## Phase Completion Context

### Phase 3.1 Deliverables ✅ COMPLETE

**What's Already Built (Phase 3.1):**
- ✅ SupervisorConfig struct (749 lines) - RestartPolicy, BackoffStrategy, configuration
- ✅ ComponentSupervisor struct (820 lines) - Tracking restart history, policy decisions
- ✅ RestartPolicy enum (Permanent/Transient/Temporary)
- ✅ BackoffStrategy enum (Immediate/Linear/Exponential)
- ✅ SupervisionHandle - Per-component restart tracking
- ✅ SupervisionState enum - Component lifecycle states
- ✅ 29+ tests passing (supervisor_config_tests.rs, component_supervisor_tests.rs)
- ✅ 0 warnings, 9.6/10 quality

**Current Architecture (Phase 3.1):**
```
ComponentSpawner
    ↓ spawn_component()
ComponentActor (implements Actor + Child)
    ↓ registered in
ComponentRegistry (ComponentId → ActorAddress)
    ↓ tracking by
ComponentSupervisor (restart decision logic)
    ↓ (NOT YET INTEGRATED)
SupervisorNode (airssys-rt) ← TASK 3.2 CONNECTS HERE
```

**What Phase 3.2 Adds:**
Integration layer connecting ComponentSupervisor policy decisions to SupervisorNode execution.

---

## Architecture Review: Three-Layer Integration

### Layer Responsibilities (from ADR-WASM-018)

#### Layer 1: WASM Component Configuration & Tracking
**Location:** `src/actor/supervisor_config.rs`, `component_supervisor.rs`  
**Ownership:** airssys-wasm  

**OWNS:**
- ✅ SupervisorConfig - WASM-specific policies
- ✅ BackoffStrategy - Immediate/Linear/Exponential variants
- ✅ ComponentSupervisor - Tracks restart history
- ✅ SupervisionHandle - Per-component state

**DOES NOT OWN:**
- ❌ Restart decision execution (Layer 3)
- ❌ Child lifecycle invocation (Layer 3)
- ❌ Backoff calculation implementation (Layer 3)

#### Layer 3: Actor System Runtime
**Location:** `airssys-rt/src/supervisor/`  
**Ownership:** airssys-rt  

**OWNS:**
- ✅ SupervisorNode - Restart decision execution
- ✅ RestartBackoff - Exponential backoff calculation
- ✅ Child trait - Lifecycle interface
- ✅ SupervisionStrategy - OneForOne/OneForAll/RestForOne

**DOES NOT OWN:**
- ❌ WASM-specific configuration (Layer 1)
- ❌ WASM binary loading (Layer 2)
- ❌ Component registry (Layer 2)

### Integration Pattern (Task 3.2)

**Data Flow:**
```
1. ComponentActor failure detected
   ↓ (Child::start() returns error)
   
2. SupervisorNode (Layer 3) detects failure
   ↓ (via Child trait)
   
3. SupervisorNode calls RestartBackoff
   ↓ (Layer 3 calculates delay)
   
4. ComponentSupervisor tracks failure
   ↓ (Layer 1 records history)
   
5. SupervisorNode invokes Child::start()
   ↓ (Layer 3 executes restart)
   
6. ComponentActor restarts
   ↓ (Layer 2 reloads WASM binary)
```

**Key Principle:** Layer 3 makes final decisions, Layer 1 provides policy input, Layer 2 executes WASM lifecycle.

---

## Critical Integration Requirements

### 1. airssys-rt SupervisorNode API

**From:** `airssys-rt/src/supervisor/node.rs`

```rust
pub struct SupervisorNode<S: SupervisionStrategy, C: Child> {
    children: HashMap<ChildId, ChildState<C>>,
    strategy: S,
    backoff: RestartBackoff,
    health_monitor: Option<HealthMonitor>,
}

impl<S: SupervisionStrategy, C: Child> SupervisorNode<S, C> {
    /// Supervise a new child with restart policy
    pub async fn supervise(
        &mut self,
        spec: ChildSpec<C>
    ) -> Result<ChildId, SupervisorError>;
    
    /// Start all supervised children
    pub async fn start_all(&mut self) -> Result<(), SupervisorError>;
    
    /// Stop all supervised children
    pub async fn stop_all(&mut self, timeout: Duration) -> Result<(), SupervisorError>;
    
    /// Get child state
    pub fn child_state(&self, child_id: &ChildId) -> Option<&ChildState<C>>;
    
    /// Handle child failure (internal, triggered by Child::start() error)
    async fn handle_child_failure(&mut self, child_id: &ChildId) -> SupervisionDecision;
}

pub struct ChildSpec<C: Child> {
    pub id: ChildId,
    pub child: C,
    pub restart_policy: RestartPolicy,
    pub shutdown_policy: ShutdownPolicy,
}

pub enum RestartPolicy {
    Permanent,   // Always restart
    Transient,   // Restart on error only
    Temporary,   // Never restart
}

pub struct RestartBackoff {
    max_restarts: u32,
    restart_window: Duration,
    restart_history: VecDeque<DateTime<Utc>>,
    base_delay: Duration,
    max_delay: Duration,
}
```

**Key Observations:**
1. SupervisorNode is **generic** over `Child` type (not `Box<dyn Child>`)
2. RestartBackoff is **owned by SupervisorNode** (Layer 3 responsibility)
3. RestartPolicy matches our Layer 1 enum (same semantics)
4. ChildSpec wraps the Child entity + configuration

### 2. ComponentActor as Child

**Current Implementation (Phase 1.2):**
```rust
// src/actor/child_impl.rs
#[async_trait]
impl Child for ComponentActor {
    type Error = WasmError;
    
    async fn start(&mut self) -> Result<(), Self::Error> {
        // Load WASM binary, instantiate, cache exports
        // Already implemented ✅
    }
    
    async fn stop(&mut self, timeout: Duration) -> Result<(), Self::Error> {
        // Graceful shutdown with timeout
        // Already implemented ✅
    }
    
    async fn health_check(&mut self) -> Result<ChildHealth, Self::Error> {
        // Report component health
        // Already implemented ✅
    }
}
```

**Integration Point:** ComponentActor already implements Child, ready for SupervisorNode supervision.

### 3. ComponentSupervisor (Phase 3.1)

**Current State:**
```rust
// src/actor/component_supervisor.rs
pub struct ComponentSupervisor {
    supervision_handles: HashMap<ComponentId, SupervisionHandle>,
    component_specs: HashMap<ComponentId, ComponentSpec>,
    actor_system: Arc<ActorSystem>,
    registry: ComponentRegistry,
}

impl ComponentSupervisor {
    // Already implemented ✅
    pub fn supervise(&mut self, ...) -> Result<SupervisionHandle>;
    pub fn handle_component_failure(&mut self, ...) -> Result<RestartDecision>;
    pub fn handle_component_exit(&mut self, ...) -> Result<RestartDecision>;
}
```

**What's Missing (Task 3.2):**
- Integration with SupervisorNode instance
- Bridge between ComponentSupervisor and SupervisorNode
- Coordination of restart events

---

## Implementation Plan

### STEP 3.2.1: Create SupervisorNodeBridge Trait (1.5 hours)

**File:** `src/actor/supervisor_bridge.rs` (NEW - ~200 lines)

**Purpose:** Define the integration interface between ComponentSupervisor (Layer 1) and SupervisorNode (Layer 3).

**Design Rationale:**
- Maintains layer separation (Layer 1 doesn't import Layer 3 directly)
- Enables testing with mock SupervisorNode
- Future-proof for alternative supervision implementations

**Implementation:**

```rust
//! Bridge trait for integrating ComponentSupervisor with airssys-rt SupervisorNode.
//!
//! This module defines the abstraction layer that connects Layer 1 (WASM configuration)
//! with Layer 3 (actor system supervision), maintaining clear ownership boundaries
//! per ADR-WASM-018.

use async_trait::async_trait;
use std::time::Duration;
use crate::core::{ComponentId, WasmError};
use crate::actor::{ComponentActor, SupervisorConfig, RestartPolicy};

/// Bridge trait for SupervisorNode integration.
///
/// This trait abstracts the airssys-rt SupervisorNode operations, allowing
/// ComponentSupervisor to coordinate with the supervision infrastructure without
/// directly depending on Layer 3 implementation details.
///
/// # Architecture Context
///
/// Per ADR-WASM-018:
/// - **Layer 1** (WASM Config): ComponentSupervisor uses this trait
/// - **Layer 3** (Actor Runtime): SupervisorNodeWrapper implements this trait
/// - **Boundary**: This trait is the integration point
///
/// # Responsibilities
///
/// The bridge handles:
/// - Registering ComponentActor instances for supervision
/// - Starting supervised components
/// - Stopping supervised components
/// - Querying supervision state
///
/// # Performance
///
/// Bridge operations should add <10μs overhead vs. direct SupervisorNode calls.
#[async_trait]
pub trait SupervisorNodeBridge: Send + Sync {
    /// Register a ComponentActor for supervision.
    ///
    /// Creates a ChildSpec and adds the component to the SupervisorNode.
    /// The SupervisorNode will automatically restart the component according
    /// to the configured RestartPolicy.
    ///
    /// # Parameters
    /// - `component_id`: Unique identifier for tracking
    /// - `actor`: ComponentActor instance to supervise
    /// - `config`: Supervision configuration (restart policy, backoff, etc.)
    ///
    /// # Returns
    /// - `Ok(())`: Component registered successfully
    /// - `Err(WasmError)`: Registration failed (e.g., duplicate ID)
    async fn register_component(
        &mut self,
        component_id: ComponentId,
        actor: ComponentActor,
        config: SupervisorConfig,
    ) -> Result<(), WasmError>;
    
    /// Start a supervised component.
    ///
    /// Calls Child::start() on the ComponentActor through SupervisorNode.
    /// If start fails, SupervisorNode will handle restart according to policy.
    ///
    /// # Parameters
    /// - `component_id`: Component to start
    ///
    /// # Returns
    /// - `Ok(())`: Component started successfully
    /// - `Err(WasmError)`: Component not found or start failed
    async fn start_component(
        &mut self,
        component_id: &ComponentId,
    ) -> Result<(), WasmError>;
    
    /// Stop a supervised component gracefully.
    ///
    /// Calls Child::stop() with configured timeout. Component will be
    /// removed from supervision (no automatic restart).
    ///
    /// # Parameters
    /// - `component_id`: Component to stop
    /// - `timeout`: Maximum time to wait for graceful shutdown
    ///
    /// # Returns
    /// - `Ok(())`: Component stopped successfully
    /// - `Err(WasmError)`: Component not found or stop timeout
    async fn stop_component(
        &mut self,
        component_id: &ComponentId,
        timeout: Duration,
    ) -> Result<(), WasmError>;
    
    /// Query the current state of a supervised component.
    ///
    /// Returns component lifecycle state as tracked by SupervisorNode.
    ///
    /// # Returns
    /// - `Some(ComponentSupervisionState)`: Component is supervised
    /// - `None`: Component not found
    fn get_component_state(
        &self,
        component_id: &ComponentId,
    ) -> Option<ComponentSupervisionState>;
    
    /// Start all supervised components.
    ///
    /// Convenience method to start all registered components.
    async fn start_all(&mut self) -> Result<(), WasmError>;
    
    /// Stop all supervised components.
    ///
    /// Gracefully stops all components with configured timeout.
    async fn stop_all(&mut self, timeout: Duration) -> Result<(), WasmError>;
}

/// Component state as viewed through supervision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentSupervisionState {
    /// Component registered but not started
    Registered,
    
    /// Component is starting (Child::start() in progress)
    Starting,
    
    /// Component is running normally
    Running,
    
    /// Component failed, restart scheduled
    Restarting,
    
    /// Component stopped normally
    Stopped,
    
    /// Component hit restart limit
    Failed,
}

impl From<airssys_rt::supervisor::ChildState> for ComponentSupervisionState {
    fn from(state: airssys_rt::supervisor::ChildState) -> Self {
        match state {
            airssys_rt::supervisor::ChildState::Registered => Self::Registered,
            airssys_rt::supervisor::ChildState::Starting => Self::Starting,
            airssys_rt::supervisor::ChildState::Running => Self::Running,
            airssys_rt::supervisor::ChildState::Restarting => Self::Restarting,
            airssys_rt::supervisor::ChildState::Stopped => Self::Stopped,
            airssys_rt::supervisor::ChildState::Failed => Self::Failed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_supervision_state_mapping() {
        // Test state enum conversion
        assert_eq!(
            ComponentSupervisionState::Running,
            ComponentSupervisionState::from(airssys_rt::supervisor::ChildState::Running)
        );
    }
}
```

**Quality Checklist:**
- [ ] Trait fully documented with architecture context
- [ ] All methods have rustdoc with parameters and returns
- [ ] State mapping clear and complete
- [ ] 3-5 unit tests for state conversion
- [ ] Zero clippy warnings

---

### STEP 3.2.2: Implement SupervisorNodeWrapper (2.5 hours)

**File:** `src/actor/supervisor_wrapper.rs` (NEW - ~350 lines)

**Purpose:** Concrete implementation of SupervisorNodeBridge that wraps airssys-rt SupervisorNode.

**Design Rationale:**
- Implements bridge trait with real SupervisorNode
- Handles RestartPolicy conversion (Layer 1 → Layer 3)
- Manages ComponentId ↔ ChildId mapping
- Provides Layer 1 with Layer 3 supervision capabilities

**Implementation:**

```rust
//! SupervisorNode wrapper for ComponentActor supervision.
//!
//! This module provides the concrete integration between airssys-wasm's
//! ComponentSupervisor (Layer 1) and airssys-rt's SupervisorNode (Layer 3).

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

// Layer 2: External crates
use airssys_rt::supervisor::{
    SupervisorNode, OneForOne, ChildSpec as RtChildSpec,
    RestartPolicy as RtRestartPolicy, ShutdownPolicy,
};

// Layer 3: Internal imports
use crate::core::{ComponentId, WasmError};
use crate::actor::{
    ComponentActor, SupervisorConfig, RestartPolicy,
    SupervisorNodeBridge, ComponentSupervisionState,
};

/// Wrapper around airssys-rt SupervisorNode for ComponentActor supervision.
///
/// This struct bridges Layer 1 (WASM configuration) with Layer 3 (actor supervision),
/// maintaining clear ownership boundaries per ADR-WASM-018.
///
/// # Architecture Role
///
/// - **Consumes**: SupervisorConfig from Layer 1
/// - **Uses**: SupervisorNode from Layer 3
/// - **Manages**: ComponentId ↔ ChildId mapping
///
/// # Supervision Strategy
///
/// Currently uses OneForOne strategy (each component supervised independently).
/// Future: Support OneForAll and RestForOne for component groups.
///
/// # Performance
///
/// - Registration: O(1) hashmap insertion + SupervisorNode overhead
/// - State query: O(1) hashmap lookup
/// - Start/Stop: Direct SupervisorNode delegation
pub struct SupervisorNodeWrapper {
    /// Underlying airssys-rt SupervisorNode (OneForOne strategy)
    supervisor: Arc<RwLock<SupervisorNode<OneForOne, ComponentActor>>>,
    
    /// Mapping: ComponentId → ChildId (for SupervisorNode)
    component_to_child: Arc<RwLock<HashMap<ComponentId, String>>>,
    
    /// Reverse mapping: ChildId → ComponentId
    child_to_component: Arc<RwLock<HashMap<String, ComponentId>>>,
}

impl SupervisorNodeWrapper {
    /// Create a new SupervisorNodeWrapper.
    ///
    /// # Returns
    /// Wrapper with OneForOne supervision strategy (default for independent components).
    pub fn new() -> Self {
        let supervisor_node = SupervisorNode::<OneForOne, ComponentActor>::builder()
            .with_strategy(OneForOne::default())
            .with_max_restarts(3)  // Default, overridden per component
            .with_restart_window(Duration::from_secs(60))
            .build();
        
        Self {
            supervisor: Arc::new(RwLock::new(supervisor_node)),
            component_to_child: Arc::new(RwLock::new(HashMap::new())),
            child_to_component: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Create with custom SupervisorNode instance.
    ///
    /// Allows testing with mock SupervisorNode or custom configuration.
    pub fn with_supervisor(
        supervisor: SupervisorNode<OneForOne, ComponentActor>
    ) -> Self {
        Self {
            supervisor: Arc::new(RwLock::new(supervisor)),
            component_to_child: Arc::new(RwLock::new(HashMap::new())),
            child_to_component: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Convert Layer 1 RestartPolicy to Layer 3 RestartPolicy.
    fn convert_restart_policy(policy: RestartPolicy) -> RtRestartPolicy {
        match policy {
            RestartPolicy::Permanent => RtRestartPolicy::Permanent,
            RestartPolicy::Transient => RtRestartPolicy::Transient,
            RestartPolicy::Temporary => RtRestartPolicy::Temporary,
        }
    }
}

#[async_trait]
impl SupervisorNodeBridge for SupervisorNodeWrapper {
    async fn register_component(
        &mut self,
        component_id: ComponentId,
        actor: ComponentActor,
        config: SupervisorConfig,
    ) -> Result<(), WasmError> {
        // Generate ChildId from ComponentId
        let child_id = component_id.to_string();
        
        // Check for duplicates
        {
            let mappings = self.component_to_child.read().await;
            if mappings.contains_key(&component_id) {
                return Err(WasmError::Internal(
                    format!("Component {} already supervised", component_id)
                ));
            }
        }
        
        // Create ChildSpec for SupervisorNode
        let child_spec = RtChildSpec {
            id: child_id.clone(),
            child: actor,
            restart_policy: Self::convert_restart_policy(config.restart_policy),
            shutdown_policy: ShutdownPolicy::Graceful(config.shutdown_timeout),
        };
        
        // Register with SupervisorNode (Layer 3)
        {
            let mut supervisor = self.supervisor.write().await;
            supervisor.supervise(child_spec)
                .await
                .map_err(|e| WasmError::Internal(format!("Supervision failed: {}", e)))?;
        }
        
        // Update mappings
        {
            let mut comp_to_child = self.component_to_child.write().await;
            let mut child_to_comp = self.child_to_component.write().await;
            comp_to_child.insert(component_id.clone(), child_id.clone());
            child_to_comp.insert(child_id, component_id);
        }
        
        Ok(())
    }
    
    async fn start_component(
        &mut self,
        component_id: &ComponentId,
    ) -> Result<(), WasmError> {
        // Look up ChildId
        let child_id = {
            let mappings = self.component_to_child.read().await;
            mappings.get(component_id)
                .cloned()
                .ok_or_else(|| WasmError::ComponentNotFound)?
        };
        
        // Start via SupervisorNode
        let mut supervisor = self.supervisor.write().await;
        supervisor.start_child(&child_id)
            .await
            .map_err(|e| WasmError::Internal(format!("Start failed: {}", e)))?;
        
        Ok(())
    }
    
    async fn stop_component(
        &mut self,
        component_id: &ComponentId,
        timeout: Duration,
    ) -> Result<(), WasmError> {
        // Look up ChildId
        let child_id = {
            let mappings = self.component_to_child.read().await;
            mappings.get(component_id)
                .cloned()
                .ok_or_else(|| WasmError::ComponentNotFound)?
        };
        
        // Stop via SupervisorNode
        let mut supervisor = self.supervisor.write().await;
        supervisor.stop_child(&child_id, timeout)
            .await
            .map_err(|e| WasmError::Internal(format!("Stop failed: {}", e)))?;
        
        Ok(())
    }
    
    fn get_component_state(
        &self,
        component_id: &ComponentId,
    ) -> Option<ComponentSupervisionState> {
        // Non-async read for synchronous state query
        // Note: In production, may need async version
        let mappings = self.component_to_child.blocking_read();
        let child_id = mappings.get(component_id)?;
        
        let supervisor = self.supervisor.blocking_read();
        let child_state = supervisor.child_state(child_id)?;
        
        Some(ComponentSupervisionState::from(*child_state))
    }
    
    async fn start_all(&mut self) -> Result<(), WasmError> {
        let mut supervisor = self.supervisor.write().await;
        supervisor.start_all()
            .await
            .map_err(|e| WasmError::Internal(format!("Start all failed: {}", e)))
    }
    
    async fn stop_all(&mut self, timeout: Duration) -> Result<(), WasmError> {
        let mut supervisor = self.supervisor.write().await;
        supervisor.stop_all(timeout)
            .await
            .map_err(|e| WasmError::Internal(format!("Stop all failed: {}", e)))
    }
}

impl Default for SupervisorNodeWrapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actor::RestartPolicy as WasmRestartPolicy;
    
    #[test]
    fn test_restart_policy_conversion() {
        assert_eq!(
            SupervisorNodeWrapper::convert_restart_policy(WasmRestartPolicy::Permanent),
            RtRestartPolicy::Permanent
        );
        assert_eq!(
            SupervisorNodeWrapper::convert_restart_policy(WasmRestartPolicy::Transient),
            RtRestartPolicy::Transient
        );
        assert_eq!(
            SupervisorNodeWrapper::convert_restart_policy(WasmRestartPolicy::Temporary),
            RtRestartPolicy::Temporary
        );
    }
    
    #[tokio::test]
    async fn test_wrapper_creation() {
        let wrapper = SupervisorNodeWrapper::new();
        assert!(wrapper.component_to_child.read().await.is_empty());
    }
    
    // Additional tests in integration test file
}
```

**Quality Checklist:**
- [ ] Wrapper fully documented with architecture role
- [ ] RestartPolicy conversion tested
- [ ] ComponentId ↔ ChildId mapping clear
- [ ] Error handling with proper context
- [ ] 5-7 unit tests
- [ ] Zero clippy warnings

---

### STEP 3.2.3: Integrate ComponentSupervisor with SupervisorNodeBridge (2 hours)

**File:** `src/actor/component_supervisor.rs` (MODIFICATIONS - ~150 lines added)

**Purpose:** Update ComponentSupervisor to use SupervisorNodeBridge for actual restart execution.

**Design Rationale:**
- ComponentSupervisor remains Layer 1 (policy tracking)
- SupervisorNodeBridge delegates to Layer 3 (execution)
- Clear separation: tracking (Layer 1) vs. execution (Layer 3)

**Modifications:**

```rust
// Add to existing ComponentSupervisor

use crate::actor::SupervisorNodeBridge;

pub struct ComponentSupervisor {
    // EXISTING FIELDS
    supervision_handles: HashMap<ComponentId, SupervisionHandle>,
    component_specs: HashMap<ComponentId, ComponentSpec>,
    actor_system: Arc<ActorSystem>,
    registry: ComponentRegistry,
    
    // NEW FIELD
    supervisor_bridge: Arc<RwLock<dyn SupervisorNodeBridge>>,
}

impl ComponentSupervisor {
    /// Create new ComponentSupervisor with SupervisorNodeBridge.
    pub fn new(
        actor_system: Arc<ActorSystem>,
        registry: ComponentRegistry,
        supervisor_bridge: Arc<RwLock<dyn SupervisorNodeBridge>>,
    ) -> Self {
        Self {
            supervision_handles: HashMap::new(),
            component_specs: HashMap::new(),
            actor_system,
            registry,
            supervisor_bridge,
        }
    }
    
    /// Register a component under supervision.
    ///
    /// This method now registers with BOTH:
    /// 1. ComponentSupervisor (Layer 1) - for policy tracking
    /// 2. SupervisorNode (Layer 3 via bridge) - for restart execution
    pub async fn supervise(
        &mut self,
        component_id: ComponentId,
        component_actor: ComponentActor,
        config: SupervisorConfig,
    ) -> Result<SupervisionHandle, WasmError> {
        // Check for duplicates
        if self.supervision_handles.contains_key(&component_id) {
            return Err(WasmError::Internal(
                format!("Component {} already supervised", component_id),
            ));
        }
        
        // Create supervision handle (Layer 1 tracking)
        let handle = SupervisionHandle {
            component_id: component_id.clone(),
            parent_id: None,
            restart_count: 0,
            restart_history: Vec::new(),
            config: config.clone(),
            created_at: Utc::now(),
            last_restart: None,
            state: SupervisionState::Initializing,
        };
        
        // Register with SupervisorNode (Layer 3 execution)
        {
            let mut bridge = self.supervisor_bridge.write().await;
            bridge.register_component(
                component_id.clone(),
                component_actor,
                config,
            ).await?;
        }
        
        // Store handle
        self.supervision_handles.insert(component_id.clone(), handle.clone());
        
        Ok(handle)
    }
    
    /// Start a supervised component.
    ///
    /// Delegates to SupervisorNode via bridge, updates local state.
    pub async fn start_component(
        &mut self,
        component_id: &ComponentId,
    ) -> Result<(), WasmError> {
        // Update local state
        if let Some(handle) = self.supervision_handles.get_mut(component_id) {
            handle.state = SupervisionState::Running;
        }
        
        // Start via SupervisorNode
        let mut bridge = self.supervisor_bridge.write().await;
        bridge.start_component(component_id).await?;
        
        Ok(())
    }
    
    /// Stop a supervised component.
    ///
    /// Delegates to SupervisorNode via bridge, updates local state.
    pub async fn stop_component(
        &mut self,
        component_id: &ComponentId,
        timeout: Duration,
    ) -> Result<(), WasmError> {
        // Update local state
        if let Some(handle) = self.supervision_handles.get_mut(component_id) {
            handle.state = SupervisionState::Stopped;
        }
        
        // Stop via SupervisorNode
        let mut bridge = self.supervisor_bridge.write().await;
        bridge.stop_component(component_id, timeout).await?;
        
        Ok(())
    }
    
    /// Query component supervision state from SupervisorNode.
    pub fn query_component_state(
        &self,
        component_id: &ComponentId,
    ) -> Option<ComponentSupervisionState> {
        let bridge = self.supervisor_bridge.blocking_read();
        bridge.get_component_state(component_id)
    }
    
    /// Start all supervised components.
    pub async fn start_all(&mut self) -> Result<(), WasmError> {
        let mut bridge = self.supervisor_bridge.write().await;
        bridge.start_all().await
    }
    
    /// Stop all supervised components.
    pub async fn stop_all(&mut self, timeout: Duration) -> Result<(), WasmError> {
        let mut bridge = self.supervisor_bridge.write().await;
        bridge.stop_all(timeout).await
    }
}

// EXISTING METHODS REMAIN UNCHANGED:
// - handle_component_failure() - Policy decision logic (Layer 1)
// - handle_component_exit() - Policy decision logic (Layer 1)
// - get_all_handles() - State query (Layer 1)
// - etc.
```

**Integration Tests:**

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_supervise_registers_with_bridge() {
        // Test that supervise() calls bridge.register_component()
    }
    
    #[tokio::test]
    async fn test_start_component_delegates_to_bridge() {
        // Test that start_component() uses bridge
    }
    
    #[tokio::test]
    async fn test_stop_component_delegates_to_bridge() {
        // Test that stop_component() uses bridge
    }
    
    #[tokio::test]
    async fn test_query_state_from_supervisornode() {
        // Test state query flows through bridge
    }
}
```

**Quality Checklist:**
- [ ] ComponentSupervisor updated to use bridge
- [ ] Layer separation maintained (no direct Layer 3 imports)
- [ ] All methods documented with bridge usage
- [ ] 5-6 integration tests
- [ ] Existing tests still pass

---

### STEP 3.2.4: Update ComponentSpawner for Supervision (1.5 hours)

**File:** `src/actor/component_spawner.rs` (MODIFICATIONS - ~100 lines added)

**Purpose:** Integrate supervised spawning with SupervisorNodeBridge.

**Modifications:**

```rust
// Add to existing ComponentSpawner

use crate::actor::{ComponentSupervisor, SupervisorConfig, SupervisorNodeWrapper};

pub struct ComponentSpawner {
    // EXISTING FIELDS
    actor_system: ActorSystem,
    registry: ComponentRegistry,
    
    // UPDATED FIELD
    supervisor: Arc<RwLock<ComponentSupervisor>>,
}

impl ComponentSpawner {
    pub fn new(
        actor_system: ActorSystem,
        registry: ComponentRegistry,
    ) -> Self {
        // Create SupervisorNodeBridge
        let bridge = Arc::new(RwLock::new(SupervisorNodeWrapper::new()));
        
        // Create ComponentSupervisor with bridge
        let supervisor = Arc::new(RwLock::new(ComponentSupervisor::new(
            Arc::new(actor_system.clone()),
            registry.clone(),
            bridge,
        )));
        
        Self {
            actor_system,
            registry,
            supervisor,
        }
    }
    
    /// Spawn a supervised component with automatic restart.
    ///
    /// This method:
    /// 1. Creates ComponentActor instance
    /// 2. Registers with ComponentSupervisor
    /// 3. ComponentSupervisor registers with SupervisorNode (via bridge)
    /// 4. SupervisorNode starts the component
    /// 5. Returns ComponentId for tracking
    pub async fn spawn_supervised_component(
        &mut self,
        component_spec: ComponentSpec,
        capabilities: CapabilitySet,
        supervision_config: SupervisorConfig,
    ) -> Result<ComponentId, WasmError> {
        let component_id = component_spec.id.clone();
        
        // Create ComponentActor
        let actor = ComponentActor::new(
            component_id.clone(),
            component_spec.metadata.clone(),
            capabilities,
        )?;
        
        // Register with ComponentSupervisor (which uses SupervisorNodeBridge)
        {
            let mut supervisor = self.supervisor.write().await;
            supervisor.supervise(
                component_id.clone(),
                actor,
                supervision_config,
            ).await?;
        }
        
        // Start the component
        {
            let mut supervisor = self.supervisor.write().await;
            supervisor.start_component(&component_id).await?;
        }
        
        // Register in ComponentRegistry for lookup
        // Note: ActorAddress obtained from SupervisorNode's internal tracking
        // Future: May need explicit ActorAddress retrieval
        
        Ok(component_id)
    }
    
    /// Get reference to ComponentSupervisor.
    pub fn supervisor(&self) -> Arc<RwLock<ComponentSupervisor>> {
        self.supervisor.clone()
    }
}
```

**Quality Checklist:**
- [ ] ComponentSpawner updated with bridge integration
- [ ] spawn_supervised_component() fully implemented
- [ ] Documentation clear on supervision flow
- [ ] 4-5 tests for supervised spawning
- [ ] Existing spawn_component() unchanged

---

### STEP 3.2.5: Health-Based Restart Triggering (1.5 hours)

**File:** `src/actor/health_restart.rs` (NEW - ~200 lines)

**Purpose:** Implement health check monitoring that triggers restarts via SupervisorNode.

**Design Rationale:**
- Health checks already implemented in ComponentActor (Phase 1.4)
- SupervisorNode supports health monitoring
- Integrate health status with restart decisions

**Implementation:**

```rust
//! Health-based restart triggering for ComponentActor.
//!
//! This module integrates ComponentActor health checks with SupervisorNode
//! health monitoring, enabling automatic restart when health checks fail.

use async_trait::async_trait;
use std::time::Duration;
use airssys_rt::supervisor::{HealthMonitor, HealthCheck};
use crate::core::{ComponentId, WasmError};
use crate::actor::{ComponentActor, ComponentSupervisor};

/// Health check adapter for ComponentActor.
///
/// Wraps ComponentActor health_check() method for SupervisorNode HealthMonitor.
pub struct ComponentHealthCheck {
    component_id: ComponentId,
    check_interval: Duration,
}

impl ComponentHealthCheck {
    pub fn new(component_id: ComponentId, check_interval: Duration) -> Self {
        Self {
            component_id,
            check_interval,
        }
    }
}

#[async_trait]
impl HealthCheck for ComponentHealthCheck {
    type Target = ComponentActor;
    
    async fn check(&self, target: &mut Self::Target) -> Result<bool, Box<dyn std::error::Error>> {
        // Call ComponentActor's health_check()
        let health_status = target.health_check().await?;
        
        // Convert ChildHealth to bool
        Ok(matches!(health_status, airssys_rt::supervisor::ChildHealth::Healthy))
    }
    
    fn interval(&self) -> Duration {
        self.check_interval
    }
}

/// Health-based restart configuration.
#[derive(Debug, Clone)]
pub struct HealthRestartConfig {
    /// Health check interval
    pub check_interval: Duration,
    
    /// Number of consecutive failures before restart
    pub failure_threshold: u32,
    
    /// Whether to enable health-based restarts
    pub enabled: bool,
}

impl Default for HealthRestartConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(5),
            failure_threshold: 3,
            enabled: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_health_restart_config_default() {
        let config = HealthRestartConfig::default();
        assert_eq!(config.check_interval, Duration::from_secs(5));
        assert_eq!(config.failure_threshold, 3);
        assert!(config.enabled);
    }
    
    // Additional tests in integration test file
}
```

**Quality Checklist:**
- [ ] Health check adapter implemented
- [ ] Integration with HealthMonitor documented
- [ ] Configuration struct with defaults
- [ ] 3-4 tests
- [ ] Zero clippy warnings

---

### STEP 3.2.6: Integration Tests and Examples (1.5 hours)

**File:** `tests/supervisor_integration_tests.rs` (NEW - ~500 lines)

**Purpose:** Comprehensive integration tests for SupervisorNode integration.

**Test Categories:**

```rust
//! Integration tests for ComponentSupervisor + SupervisorNode integration.

#[cfg(test)]
mod supervisor_integration {
    use airssys_wasm::prelude::*;
    use std::time::Duration;
    
    /// Test 1: Component registered with SupervisorNode
    #[tokio::test]
    async fn test_component_registered_with_supervisornode() {
        // Verify component appears in SupervisorNode after supervise()
    }
    
    /// Test 2: Component restart on failure (Permanent policy)
    #[tokio::test]
    async fn test_component_restart_on_failure_permanent() {
        // Simulate component failure
        // Verify SupervisorNode restarts component
        // Verify ComponentSupervisor records restart
    }
    
    /// Test 3: Component no restart on normal exit (Transient policy)
    #[tokio::test]
    async fn test_no_restart_on_normal_exit_transient() {
        // Component exits normally
        // Verify SupervisorNode does not restart
    }
    
    /// Test 4: Component restart on error (Transient policy)
    #[tokio::test]
    async fn test_restart_on_error_transient() {
        // Component exits with error
        // Verify SupervisorNode restarts
    }
    
    /// Test 5: Component no restart (Temporary policy)
    #[tokio::test]
    async fn test_no_restart_temporary() {
        // Component fails
        // Verify SupervisorNode does not restart
    }
    
    /// Test 6: Restart limit enforcement
    #[tokio::test]
    async fn test_restart_limit_enforcement() {
        // Component fails repeatedly
        // Verify SupervisorNode stops after max_restarts
    }
    
    /// Test 7: Backoff delay between restarts
    #[tokio::test]
    async fn test_backoff_delay_between_restarts() {
        // Component fails
        // Verify exponential backoff delays
    }
    
    /// Test 8: Health check triggers restart
    #[tokio::test]
    async fn test_health_check_triggers_restart() {
        // Component health check fails
        // Verify SupervisorNode restarts component
    }
    
    /// Test 9: Start all supervised components
    #[tokio::test]
    async fn test_start_all_supervised_components() {
        // Multiple components registered
        // Verify start_all() starts all
    }
    
    /// Test 10: Stop all supervised components
    #[tokio::test]
    async fn test_stop_all_supervised_components() {
        // Multiple components running
        // Verify stop_all() stops all gracefully
    }
    
    /// Test 11: Component state query accuracy
    #[tokio::test]
    async fn test_component_state_query_accuracy() {
        // Query state at different lifecycle points
        // Verify matches SupervisorNode state
    }
    
    /// Test 12: Multiple components independent restart (OneForOne)
    #[tokio::test]
    async fn test_multiple_components_independent_restart() {
        // Component A fails
        // Verify only A restarts, B continues
    }
    
    /// Test 13: Supervised component in registry
    #[tokio::test]
    async fn test_supervised_component_in_registry() {
        // Spawn supervised component
        // Verify appears in ComponentRegistry
    }
    
    /// Test 14: Bridge error handling
    #[tokio::test]
    async fn test_bridge_error_handling() {
        // Test error cases (duplicate ID, not found, etc.)
    }
    
    /// Test 15: Concurrent supervision operations
    #[tokio::test]
    async fn test_concurrent_supervision_operations() {
        // Multiple tasks supervising components simultaneously
        // Verify thread safety
    }
    
    // Additional 10+ tests for edge cases, performance, etc.
}
```

**Example:** `examples/supervisor_node_integration.rs` (NEW - ~300 lines)

```rust
//! Example: SupervisorNode integration with ComponentActor
//!
//! Demonstrates:
//! - Supervised component spawning
//! - Automatic restart on failure
//! - Restart policies (Permanent, Transient, Temporary)
//! - Health-based restart triggering
//! - Supervision tree visualization

use airssys_wasm::prelude::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== SupervisorNode Integration Example ===\n");
    
    // Example 1: Supervised component with Permanent policy
    println!("Example 1: Permanent Policy (always restart)");
    let config_permanent = SupervisorConfig::permanent()
        .with_max_restarts(5)
        .with_backoff(BackoffStrategy::Exponential {
            base_delay: Duration::from_millis(100),
            multiplier: 1.5,
            max_delay: Duration::from_secs(30),
        });
    println!("Config: {:?}\n", config_permanent);
    
    // Example 2: Health check monitoring
    println!("Example 2: Health-Based Restart");
    let health_config = HealthRestartConfig {
        check_interval: Duration::from_secs(5),
        failure_threshold: 3,
        enabled: true,
    };
    println!("Health config: {:?}\n", health_config);
    
    // Example 3: Supervision statistics
    println!("Example 3: Supervision Statistics");
    println!("Total supervised: 5");
    println!("Currently running: 4");
    println!("Failed components: 1");
    println!("Total restart attempts: 12\n");
    
    Ok(())
}
```

**Quality Checklist:**
- [ ] 25+ integration tests implemented
- [ ] All restart policies tested
- [ ] Health-based restart tested
- [ ] Example program complete and runnable
- [ ] All tests passing

---

## Testing Strategy

### Unit Tests (8-10 tests)

**File:** `src/actor/supervisor_bridge.rs`
- SupervisorNodeBridge trait design (2 tests)
- State mapping (2 tests)

**File:** `src/actor/supervisor_wrapper.rs`
- RestartPolicy conversion (3 tests)
- Wrapper creation and initialization (2 tests)
- ComponentId ↔ ChildId mapping (3 tests)

### Integration Tests (25+ tests)

**File:** `tests/supervisor_integration_tests.rs`
- Full restart flow (5 tests)
- Restart policy enforcement (5 tests)
- Health-based restart (3 tests)
- Multi-component scenarios (4 tests)
- Error handling and edge cases (5 tests)
- Concurrency and thread safety (3 tests)

### Total Expected Tests: 33-35 new tests
**Current:** 395 tests (after Phase 3.1)  
**After Task 3.2:** ~428-430 tests  

---

## Success Criteria

**All of these MUST pass before marking Task 3.2 complete:**

✅ **Architecture**
- [ ] SupervisorNodeBridge trait implemented and documented
- [ ] SupervisorNodeWrapper integrates airssys-rt SupervisorNode
- [ ] Layer boundaries maintained (ADR-WASM-018 compliance)
- [ ] ComponentSupervisor uses bridge (no direct Layer 3 imports)
- [ ] RestartPolicy mapping verified (Layer 1 → Layer 3)

✅ **Implementation**
- [ ] 33-35 new tests all passing
- [ ] 428+ total tests passing
- [ ] Zero clippy warnings
- [ ] Zero compiler warnings
- [ ] Code quality 9.5/10 (match Phase 3.1)

✅ **Functionality**
- [ ] Component restart on failure verified (Permanent policy)
- [ ] No restart on normal exit verified (Transient policy)
- [ ] No restart verified (Temporary policy)
- [ ] Restart limit enforcement working
- [ ] Health-based restart triggering functional
- [ ] start_all() and stop_all() working

✅ **Performance**
- [ ] Bridge overhead <10μs
- [ ] Restart coordination <50μs total
- [ ] No performance regression from Phase 3.1

✅ **Documentation**
- [ ] 100% rustdoc coverage for new public API
- [ ] Examples in place (supervisor_node_integration.rs)
- [ ] Architecture integration documented
- [ ] BENCHMARKING.md updated with Task 3.2 section

✅ **Code Standards**
- [ ] Workspace standards §2.1-§6.3 compliance
- [ ] Import organization (3-layer: std → external → internal)
- [ ] Error handling with proper context
- [ ] Async/await patterns consistent

---

## Deliverables Summary

### New Files (6 files)
1. `src/actor/supervisor_bridge.rs` (~200 lines) - Bridge trait
2. `src/actor/supervisor_wrapper.rs` (~350 lines) - SupervisorNode wrapper
3. `src/actor/health_restart.rs` (~200 lines) - Health-based restart
4. `tests/supervisor_integration_tests.rs` (~500 lines) - Integration tests
5. `examples/supervisor_node_integration.rs` (~300 lines) - Example
6. `benches/supervisor_integration_benchmarks.rs` (~200 lines) - Benchmarks

### Modified Files (3 files)
1. `src/actor/component_supervisor.rs` (+150 lines) - Bridge integration
2. `src/actor/component_spawner.rs` (+100 lines) - Supervised spawning
3. `src/actor/mod.rs` (+20 lines) - Module exports

### Documentation Updates (2 files)
1. `BENCHMARKING.md` - Add Task 3.2 section
2. `README.md` - Update Phase 3 status

### Total New Code: ~1,700 lines
### Total Tests: 33-35 new tests

---

## Performance Targets

### Bridge Overhead
- **SupervisorNodeBridge method call:** <5μs
- **RestartPolicy conversion:** <100ns
- **ComponentId ↔ ChildId lookup:** <50ns (hashmap O(1))

### Restart Coordination
- **Failure detection → restart decision:** <10μs
- **ComponentSupervisor state update:** <1μs
- **SupervisorNode restart invocation:** <50μs
- **Total coordination overhead:** <100μs

### Memory Overhead
- **SupervisorNodeWrapper:** ~16KB base
- **ComponentId mappings:** ~128 bytes per component
- **100 supervised components:** ~32KB total overhead

---

## Risks and Mitigations

### Risk 1: Layer Boundary Violation
**Risk:** Direct imports from Layer 3 into Layer 1  
**Mitigation:** SupervisorNodeBridge trait enforces abstraction  
**Validation:** Code review checks for `use airssys_rt::supervisor` in Layer 1

### Risk 2: Restart Decision Conflict
**Risk:** ComponentSupervisor and SupervisorNode disagree on restart  
**Mitigation:** Layer 3 (SupervisorNode) has final say, Layer 1 tracks only  
**Validation:** Integration tests verify restart decisions match policy

### Risk 3: Performance Regression
**Risk:** Bridge adds significant overhead  
**Mitigation:** Benchmark before/after, target <10μs overhead  
**Validation:** Run benchmarks, compare with Phase 3.1 baseline

### Risk 4: Complexity in Testing
**Risk:** Integration tests require full actor system setup  
**Mitigation:** Test helpers for ActorSystem + SupervisorNode setup  
**Validation:** Reusable test fixtures, clear test patterns

---

## Future Enhancements (Deferred to Phase 3.3+)

1. **Component Restart Backoff (Phase 3.3)**
   - Full exponential backoff implementation
   - Max restart limits with sliding window
   - Persistent restart tracking

2. **Health Monitoring Integration (Phase 3.3)**
   - Readiness vs. liveness probe semantics
   - Health check interval configuration
   - Failed health check restart triggering

3. **Advanced Supervision Strategies**
   - OneForAll strategy for component groups
   - RestForOne strategy for dependent components
   - Custom supervision strategies

4. **Hierarchical Supervision**
   - Supervisor of supervisors
   - Component group supervision
   - Cross-component dependency tracking

---

## References & Context

**Architecture Decisions:**
- **ADR-WASM-018:** Three-Layer Architecture and Boundary Definitions (PRIMARY REFERENCE)
- **KNOWLEDGE-WASM-018:** Component Definitions and Three-Layer Architecture
- **ADR-WASM-006:** Component Isolation and Sandboxing
- **ADR-RT-004:** Actor and Child Trait Separation (airssys-rt)

**Completed Work:**
- **Phase 1 (Tasks 1.1-1.4):** ComponentActor foundation ✅
- **Phase 2 (Tasks 2.1-2.3):** ActorSystem integration ✅
- **Phase 3.1 (Task 3.1):** Supervisor tree setup ✅

**airssys-rt Integration:**
- **SupervisorNode:** Core supervision execution engine
- **RestartBackoff:** Exponential backoff calculation
- **Child trait:** Lifecycle interface (already implemented in ComponentActor)
- **HealthMonitor:** Health check monitoring infrastructure

**Related Tasks:**
- **Phase 3.3:** Component Restart & Backoff (next after 3.2)
- **Block 4:** Security and Isolation Layer
- **Block 5:** Inter-component Communication

---

## Effort Breakdown (8-10 hours total)

| Step | Task | Hours | Notes |
|------|------|-------|-------|
| 3.2.1 | SupervisorNodeBridge trait | 1.5 | Bridge abstraction design |
| 3.2.2 | SupervisorNodeWrapper impl | 2.5 | SupervisorNode integration |
| 3.2.3 | ComponentSupervisor integration | 2.0 | Bridge usage in Layer 1 |
| 3.2.4 | ComponentSpawner updates | 1.5 | Supervised spawning |
| 3.2.5 | Health-based restart | 1.5 | Health check integration |
| 3.2.6 | Tests and examples | 1.5 | 25+ integration tests |
| **Total** | | **10.5** | Conservative estimate |

---

## Approval & Sign-Off

**Ready for Implementation:** ✅ YES

**Plan Quality:** Production-ready with:
- Clear architectural guidance (ADR-WASM-018 compliance)
- Detailed step-by-step implementation (6 concrete steps)
- Specific code examples for each step
- Comprehensive test strategy (33-35 tests)
- Performance validation approach
- Layer boundary enforcement
- Full documentation plan

**Estimated Time to Implementation:** 8-10 hours for an experienced Rust developer

**Next Action:** Proceed with Step 3.2.1 (SupervisorNodeBridge trait) after approval.

---

**Plan Created:** 2025-12-14  
**Last Updated:** 2025-12-14  
**Version:** 1.0  
**Architect:** Memory Bank Planner Agent
