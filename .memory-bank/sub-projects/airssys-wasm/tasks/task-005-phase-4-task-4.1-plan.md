# WASM-TASK-005 Phase 4 Task 4.1: ComponentActor Security Context Attachment - IMPLEMENTATION PLAN

**Task:** ComponentActor Security Context Attachment  
**Status:** 📋 PLANNED (Ready to Start)  
**Date Created:** 2025-12-19  
**Estimated Duration:** 1-2 days (8-12 hours)  
**Prerequisites:** ✅ Phase 3 complete (Tasks 3.1-3.3, 47 tests, 278 total tests, 0 warnings)

---

## Executive Summary

**What**: Attach `WasmSecurityContext` to the `ComponentActor` lifecycle, enabling component-level security isolation with automatic capability enforcement throughout the actor's lifetime.

**Why**: Components need isolated security contexts that persist across the actor lifecycle (spawn → run → restart → termination). This ensures:
- Each component has its own capability set (no sharing between actors)
- Security context is registered when actor spawns
- Security context is restored after supervisor restarts
- Security context is cleaned up on actor termination
- Capability checks work seamlessly in host functions

**How**: 
1. Add `security_context: WasmSecurityContext` field to `ComponentActor` struct
2. Initialize security context in `ComponentActor::new()` from manifest
3. Register with global capability checker (Phase 3 Task 3.1 API)
4. Integrate with spawn logic for validation
5. Implement restoration after supervisor restart
6. Implement cleanup on termination
7. Add comprehensive integration tests

**Architecture Position**: This task bridges Phase 1 (WasmSecurityContext) and Phase 3 (capability enforcement API) with the ComponentActor system (Block 3), creating end-to-end security isolation.

---

## Prerequisites Verified

### Phase 3 Dependencies (All Complete ✅)

**From Task 3.1 (Capability Check API)**:
```rust
// In src/security/enforcement.rs
pub fn register_component(
    component_id: &str,
    security_context: WasmSecurityContext,
) -> Result<(), CapabilityCheckError>;

pub fn unregister_component(
    component_id: &str
) -> Result<(), CapabilityCheckError>;

pub fn check_capability(
    component_id: &str,
    resource: &str,
    permission: &str,
) -> Result<(), CapabilityCheckError>;
```

**From Task 3.2 (Host Function Integration)**:
```rust
// Thread-local component context
pub fn set_current_component(component_id: &str);
pub fn get_current_component() -> Option<String>;
pub fn clear_current_component();

// RAII guard
pub struct ComponentContextGuard { /* ... */ }
```

**From Task 3.3 (Audit Logging)**:
```rust
// Audit logging integration
pub struct WasmAuditLogger { /* ... */ }
pub fn set_global_audit_logger(logger: Arc<WasmAuditLogger>) -> Result<(), AuditLogError>;
```

**From Phase 1 (WASM-OSL Bridge)**:
```rust
// In src/security/capability.rs
pub struct WasmSecurityContext {
    component_id: String,
    capabilities: HashSet<WasmCapability>,
    trust_level: TrustLevel,
    // ...
}

impl WasmSecurityContext {
    pub fn from_manifest(manifest: &ComponentManifest) -> Result<Self, SecurityError>;
    pub fn capabilities(&self) -> &HashSet<WasmCapability>;
    pub fn trust_level(&self) -> TrustLevel;
}
```

---

## Implementation Steps

### Step 1: Read Existing Codebase (30 minutes)

**Objective**: Understand current ComponentActor structure and integration points.

**Files to Read**:
1. `airssys-wasm/src/actor/component/component_actor.rs` - ComponentActor struct definition
2. `airssys-wasm/src/actor/component/component_spawner.rs` - Spawn logic
3. `airssys-wasm/src/actor/component/child_impl.rs` - Child trait lifecycle
4. `airssys-wasm/src/actor/lifecycle/hooks.rs` - Lifecycle hooks system
5. `airssys-wasm/src/security/capability.rs` - WasmSecurityContext
6. `airssys-wasm/src/security/enforcement.rs` - Registration API

**Key Information to Extract**:
- Current ComponentActor struct fields
- How ComponentActor::new() initializes the actor
- Where spawn logic creates actors
- How supervisor restarts actors
- How actors are terminated
- Available lifecycle hooks

**Expected Outcome**: Clear understanding of integration points.

---

### Step 2: Modify ComponentActor Struct (1 hour)

**File**: `airssys-wasm/src/actor/component/component_actor.rs`

**Current Structure** (approximate lines 400-500):
```rust
pub struct ComponentActor<S = ()>
where
    S: Send + Sync + 'static,
{
    // Existing fields...
    component_id: ComponentId,
    spec: ComponentSpec,
    capabilities: CapabilitySet,
    metadata: ComponentMetadata,
    state: Arc<RwLock<S>>,
    runtime: Option<WasmRuntime>,
    // ... other fields
}
```

**Modification** (add security_context field):
```rust
pub struct ComponentActor<S = ()>
where
    S: Send + Sync + 'static,
{
    // Existing fields...
    component_id: ComponentId,
    spec: ComponentSpec,
    capabilities: CapabilitySet,
    metadata: ComponentMetadata,
    state: Arc<RwLock<S>>,
    runtime: Option<WasmRuntime>,
    
    // NEW: Security context for capability enforcement
    /// Security context containing component capabilities and trust level.
    ///
    /// This context is initialized from the component manifest during actor creation
    /// and registered with the global capability checker. It persists for the actor's
    /// entire lifecycle and is restored after supervisor restarts.
    ///
    /// # Thread Safety
    ///
    /// The security context is immutable after creation (Arc-shared), ensuring
    /// thread-safe access from message handlers and host functions.
    security_context: Arc<WasmSecurityContext>,
}
```

**Import Addition** (top of file):
```rust
use crate::security::WasmSecurityContext;
```

**Expected Outcome**: ComponentActor struct has security_context field.

---

### Step 3: Update ComponentActor::new() (2 hours)

**File**: `airssys-wasm/src/actor/component/component_actor.rs`

**Current Constructor** (approximate lines 600-700):
```rust
impl<S> ComponentActor<S>
where
    S: Send + Sync + 'static,
{
    pub fn new(
        component_id: ComponentId,
        spec: ComponentSpec,
        capabilities: CapabilitySet,
    ) -> Self {
        // ... existing initialization
    }
}
```

**Modification**:
```rust
impl<S> ComponentActor<S>
where
    S: Send + Sync + 'static,
{
    /// Creates a new ComponentActor with security context.
    ///
    /// # Security
    ///
    /// This constructor builds a `WasmSecurityContext` from the component manifest
    /// and registers it with the global capability checker. Registration failures
    /// will cause actor creation to fail (fail-fast security).
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Security context creation fails (invalid capabilities)
    /// - Security context registration fails (duplicate component ID)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let actor = ComponentActor::new(component_id, spec, capabilities)?;
    /// // Security context is now registered and ready
    /// ```
    pub fn new(
        component_id: ComponentId,
        spec: ComponentSpec,
        capabilities: CapabilitySet,
    ) -> Result<Self, WasmError> {
        // NEW: Build security context from manifest/capabilities
        let security_context = WasmSecurityContext::new(
            component_id.to_string(),
            capabilities.clone(), // Convert CapabilitySet to HashSet<WasmCapability>
            spec.trust_level.clone(), // Get trust level from spec
        )?;
        
        // NEW: Register security context with global capability checker
        crate::security::enforcement::register_component(
            component_id.as_str(),
            security_context.clone(),
        ).map_err(|e| WasmError::Security(format!(
            "Failed to register security context for component {}: {}",
            component_id,
            e
        )))?;
        
        // Existing initialization...
        Ok(Self {
            component_id,
            spec,
            capabilities,
            metadata: ComponentMetadata::default(),
            state: Arc::new(RwLock::new(S::default())),
            runtime: None,
            security_context: Arc::new(security_context), // NEW field
            // ... other fields
        })
    }
}
```

**Add Getter Method**:
```rust
impl<S> ComponentActor<S>
where
    S: Send + Sync + 'static,
{
    /// Returns a reference to the component's security context.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let ctx = actor.security_context();
    /// println!("Trust level: {:?}", ctx.trust_level());
    /// ```
    pub fn security_context(&self) -> &Arc<WasmSecurityContext> {
        &self.security_context
    }
}
```

**Error Handling Addition**:
In `src/core/error.rs` or wherever `WasmError` is defined:
```rust
pub enum WasmError {
    // Existing variants...
    
    /// Security context error (registration, validation, etc.)
    #[error("Security error: {0}")]
    Security(String),
}
```

**Expected Outcome**: ComponentActor::new() builds and registers security context.

---

### Step 4: Add Drop Implementation for Cleanup (30 minutes)

**File**: `airssys-wasm/src/actor/component/component_actor.rs`

**Add Drop Trait**:
```rust
impl<S> Drop for ComponentActor<S>
where
    S: Send + Sync + 'static,
{
    /// Cleanup security context registration when actor is dropped.
    ///
    /// This ensures the global capability checker doesn't retain stale
    /// component registrations after actor termination.
    fn drop(&mut self) {
        // Unregister security context
        if let Err(e) = crate::security::enforcement::unregister_component(
            self.component_id.as_str()
        ) {
            // Log error but don't panic in Drop
            eprintln!(
                "Warning: Failed to unregister component {}: {}",
                self.component_id,
                e
            );
        }
        
        // Clear thread-local context if set
        crate::security::enforcement::clear_current_component();
    }
}
```

**Expected Outcome**: Security context is automatically cleaned up when actor drops.

---

### Step 5: Integrate with Spawn Logic (1 hour)

**File**: `airssys-wasm/src/actor/component/component_spawner.rs`

**Current Spawn Function** (approximate):
```rust
pub async fn spawn_component<S>(
    system: &ActorSystem,
    component_id: ComponentId,
    spec: ComponentSpec,
    capabilities: CapabilitySet,
) -> Result<ActorRef, WasmError>
where
    S: Send + Sync + 'static,
{
    // Create actor
    let actor = ComponentActor::new(component_id, spec, capabilities);
    
    // Spawn with system
    let actor_ref = system.spawn(actor).await?;
    
    Ok(actor_ref)
}
```

**Modification**:
```rust
/// Spawns a ComponentActor with security context validation.
///
/// # Security
///
/// This function creates a ComponentActor with security context registration.
/// If security context creation or registration fails, the spawn operation
/// fails immediately (fail-fast security).
///
/// # Errors
///
/// Returns error if:
/// - Security context creation fails
/// - Security context registration fails
/// - Actor system spawn fails
///
/// # Examples
///
/// ```rust,ignore
/// let actor_ref = spawn_component(&system, component_id, spec, capabilities).await?;
/// // Component is now running with registered security context
/// ```
pub async fn spawn_component<S>(
    system: &ActorSystem,
    component_id: ComponentId,
    spec: ComponentSpec,
    capabilities: CapabilitySet,
) -> Result<ActorRef, WasmError>
where
    S: Send + Sync + 'static,
{
    // Create actor (security context registered in ComponentActor::new)
    let actor = ComponentActor::new(component_id.clone(), spec, capabilities)?;
    
    // Spawn with system
    let actor_ref = system.spawn(actor).await.map_err(|e| {
        // If spawn fails, security context is cleaned up via Drop
        WasmError::ActorSystem(format!(
            "Failed to spawn component {}: {}",
            component_id,
            e
        ))
    })?;
    
    Ok(actor_ref)
}
```

**Expected Outcome**: Spawn validates security context and registers it.

---

### Step 6: Implement Security Context Restoration (2 hours)

**File**: `airssys-wasm/src/actor/lifecycle/hooks.rs` or create new file `airssys-wasm/src/actor/component/security_lifecycle.rs`

**Add Restoration Function**:
```rust
/// Restores security context after supervisor restart.
///
/// This function is called by the supervisor when restarting a component
/// after failure. It rebuilds the security context from the manifest and
/// re-registers it with the global capability checker.
///
/// # Arguments
///
/// * `component_id` - ID of the component being restarted
/// * `spec` - Component specification with trust level
/// * `capabilities` - Component capability set
///
/// # Errors
///
/// Returns error if:
/// - Security context creation fails
/// - Security context registration fails
///
/// # Examples
///
/// ```rust,ignore
/// // In supervisor restart logic
/// restore_security_context(&component_id, &spec, &capabilities).await?;
/// ```
pub async fn restore_security_context(
    component_id: &ComponentId,
    spec: &ComponentSpec,
    capabilities: &CapabilitySet,
) -> Result<(), WasmError> {
    // Rebuild security context
    let security_context = WasmSecurityContext::new(
        component_id.to_string(),
        capabilities.clone(),
        spec.trust_level.clone(),
    )?;
    
    // Re-register with global capability checker
    crate::security::enforcement::register_component(
        component_id.as_str(),
        security_context,
    ).map_err(|e| WasmError::Security(format!(
        "Failed to restore security context for component {}: {}",
        component_id,
        e
    )))?;
    
    Ok(())
}
```

**Integration with Supervisor** (if needed in supervisor restart logic):
```rust
// In supervisor restart handler
pub async fn handle_component_restart(
    system: &ActorSystem,
    component_id: ComponentId,
    spec: ComponentSpec,
    capabilities: CapabilitySet,
) -> Result<ActorRef, WasmError> {
    // Restore security context before recreating actor
    restore_security_context(&component_id, &spec, &capabilities).await?;
    
    // Recreate and spawn actor
    spawn_component(system, component_id, spec, capabilities).await
}
```

**Expected Outcome**: Security context is restored after supervisor restarts.

---

### Step 7: Add Error Handling (30 minutes)

**File**: `airssys-wasm/src/core/error.rs` (or wherever errors are defined)

**Add Security Error Variants**:
```rust
#[derive(Debug, Error)]
pub enum WasmError {
    // Existing variants...
    
    /// Security context creation failed
    #[error("Failed to create security context: {0}")]
    SecurityContextCreation(String),
    
    /// Security context registration failed
    #[error("Failed to register security context: {0}")]
    SecurityContextRegistration(String),
    
    /// Security context not found
    #[error("Security context not found for component: {0}")]
    SecurityContextNotFound(String),
    
    /// Capability validation failed
    #[error("Capability validation failed: {0}")]
    CapabilityValidation(String),
}
```

**Expected Outcome**: Clear error messages for security failures.

---

### Step 8: Create Integration Tests (3 hours)

**File**: `airssys-wasm/tests/actor_security_integration_tests.rs` (NEW FILE)

**Test Suite Structure**:
```rust
//! Integration tests for ComponentActor security context attachment.
//!
//! This test suite verifies that security context is correctly attached,
//! registered, isolated, restored, and cleaned up throughout the actor lifecycle.

use airssys_wasm::actor::{ComponentActor, ComponentSpawner};
use airssys_wasm::core::{ComponentId, ComponentSpec, CapabilitySet};
use airssys_wasm::security::{WasmCapability, check_capability};
use airssys_rt::actor::ActorSystem;

#[tokio::test]
async fn test_component_actor_has_security_context() {
    // Create actor
    let component_id = ComponentId::new("test-component");
    let spec = ComponentSpec::default();
    let capabilities = CapabilitySet::new()
        .grant(WasmCapability::Filesystem {
            paths: vec!["/app/data/*".to_string()],
            permissions: vec!["read".to_string()],
        });
    
    let actor = ComponentActor::new(component_id.clone(), spec, capabilities)
        .expect("Failed to create actor");
    
    // Verify security context exists
    let ctx = actor.security_context();
    assert_eq!(ctx.component_id(), component_id.as_str());
    assert!(!ctx.capabilities().is_empty());
}

#[tokio::test]
async fn test_security_context_registered_on_creation() {
    let component_id = ComponentId::new("test-registration");
    let spec = ComponentSpec::default();
    let capabilities = CapabilitySet::new()
        .grant(WasmCapability::Filesystem {
            paths: vec!["/app/data/*".to_string()],
            permissions: vec!["read".to_string()],
        });
    
    let actor = ComponentActor::new(component_id.clone(), spec, capabilities)
        .expect("Failed to create actor");
    
    // Verify capability check works (context is registered)
    let result = check_capability(
        component_id.as_str(),
        "/app/data/file.txt",
        "read"
    );
    
    assert!(result.is_ok(), "Capability check should succeed");
}

#[tokio::test]
async fn test_context_isolation_between_actors() {
    // Create two actors with different capabilities
    let actor1_id = ComponentId::new("actor1");
    let actor1_caps = CapabilitySet::new()
        .grant(WasmCapability::Filesystem {
            paths: vec!["/app/data/*".to_string()],
            permissions: vec!["read".to_string()],
        });
    
    let actor2_id = ComponentId::new("actor2");
    let actor2_caps = CapabilitySet::new()
        .grant(WasmCapability::Network {
            endpoints: vec!["api.example.com:443".to_string()],
            permissions: vec!["connect".to_string()],
        });
    
    let actor1 = ComponentActor::new(
        actor1_id.clone(),
        ComponentSpec::default(),
        actor1_caps
    ).expect("Failed to create actor1");
    
    let actor2 = ComponentActor::new(
        actor2_id.clone(),
        ComponentSpec::default(),
        actor2_caps
    ).expect("Failed to create actor2");
    
    // Verify actor1 can read filesystem but not access network
    assert!(check_capability(actor1_id.as_str(), "/app/data/file.txt", "read").is_ok());
    assert!(check_capability(actor1_id.as_str(), "api.example.com:443", "connect").is_err());
    
    // Verify actor2 can access network but not read filesystem
    assert!(check_capability(actor2_id.as_str(), "api.example.com:443", "connect").is_ok());
    assert!(check_capability(actor2_id.as_str(), "/app/data/file.txt", "read").is_err());
}

#[tokio::test]
async fn test_security_context_cleaned_up_on_drop() {
    let component_id = ComponentId::new("test-cleanup");
    let spec = ComponentSpec::default();
    let capabilities = CapabilitySet::new();
    
    {
        let actor = ComponentActor::new(component_id.clone(), spec.clone(), capabilities.clone())
            .expect("Failed to create actor");
        
        // Verify context is registered
        assert!(check_capability(component_id.as_str(), "/app/data/*", "read").is_err());
        // Error expected because no capabilities granted, but component is registered
        
        // Actor drops here
    }
    
    // Verify context is cleaned up
    let result = check_capability(component_id.as_str(), "/app/data/*", "read");
    // Should get "component not found" error
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[tokio::test]
async fn test_invalid_capabilities_prevent_spawn() {
    // Create actor with invalid capability pattern
    let component_id = ComponentId::new("invalid-component");
    let spec = ComponentSpec::default();
    let capabilities = CapabilitySet::new()
        .grant(WasmCapability::Filesystem {
            paths: vec!["[invalid-glob".to_string()], // Invalid glob pattern
            permissions: vec!["read".to_string()],
        });
    
    // Actor creation should fail
    let result = ComponentActor::new(component_id, spec, capabilities);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_spawn_validates_security_context() {
    let system = ActorSystem::new("test-system");
    let component_id = ComponentId::new("spawn-test");
    let spec = ComponentSpec::default();
    let capabilities = CapabilitySet::new();
    
    // Spawn should succeed
    let result = spawn_component(&system, component_id.clone(), spec, capabilities).await;
    assert!(result.is_ok());
    
    // Verify security context is registered
    let check_result = check_capability(component_id.as_str(), "/app/*", "read");
    // Component is registered (even if no capabilities)
    assert!(check_result.is_err()); // Denied because no capabilities
}

#[tokio::test]
async fn test_security_context_restored_after_restart() {
    let system = ActorSystem::new("restart-test-system");
    let component_id = ComponentId::new("restart-component");
    let spec = ComponentSpec::default();
    let capabilities = CapabilitySet::new()
        .grant(WasmCapability::Filesystem {
            paths: vec!["/app/data/*".to_string()],
            permissions: vec!["read".to_string()],
        });
    
    // Spawn component
    let actor_ref = spawn_component(
        &system,
        component_id.clone(),
        spec.clone(),
        capabilities.clone()
    ).await.expect("Failed to spawn");
    
    // Verify capability works
    assert!(check_capability(component_id.as_str(), "/app/data/file.txt", "read").is_ok());
    
    // Simulate restart (unregister then restore)
    crate::security::enforcement::unregister_component(component_id.as_str())
        .expect("Failed to unregister");
    
    // Restore security context
    restore_security_context(&component_id, &spec, &capabilities).await
        .expect("Failed to restore");
    
    // Verify capability still works
    assert!(check_capability(component_id.as_str(), "/app/data/file.txt", "read").is_ok());
}

// Additional test scenarios (20+ total):
// - test_duplicate_component_id_fails()
// - test_security_context_immutable_after_creation()
// - test_capability_check_during_message_handling()
// - test_trust_level_propagated_to_context()
// - test_security_context_with_multiple_capabilities()
// - test_security_context_with_no_capabilities()
// - test_actor_restart_preserves_trust_level()
// - test_security_error_messages_are_clear()
// - test_thread_local_context_integration()
// - test_security_audit_logging_on_spawn()
// - test_security_context_performance()
// - test_concurrent_actor_spawns()
// ... (add more as needed)
```

**Expected Outcome**: 20+ integration tests verifying all security lifecycle aspects.

---

### Step 9: Run Tests and Verify (1 hour)

**Commands**:
```bash
# Run security integration tests
cargo test --test actor_security_integration_tests

# Run all actor tests
cargo test --lib actor

# Run all security tests
cargo test --lib security

# Run full workspace tests
cargo test --workspace

# Check for warnings
cargo clippy --all-targets
cargo doc --no-deps
```

**Verification Checklist**:
- [ ] All new integration tests pass
- [ ] All existing actor tests still pass
- [ ] All existing security tests still pass
- [ ] Total test count increased by 20+
- [ ] Zero compiler warnings
- [ ] Zero clippy warnings
- [ ] Zero rustdoc warnings
- [ ] No test failures in other modules

**Expected Outcome**: All tests passing, 0 warnings.

---

### Step 10: Performance Validation (1 hour)

**Create Benchmark** (optional but recommended):
```rust
// In benches/security_context_overhead.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use airssys_wasm::actor::ComponentActor;
use airssys_wasm::core::{ComponentId, ComponentSpec, CapabilitySet};

fn benchmark_security_context_creation(c: &mut Criterion) {
    let component_id = ComponentId::new("bench-component");
    let spec = ComponentSpec::default();
    let capabilities = CapabilitySet::new();
    
    c.bench_function("security_context_creation", |b| {
        b.iter(|| {
            let actor = ComponentActor::new(
                black_box(component_id.clone()),
                black_box(spec.clone()),
                black_box(capabilities.clone())
            ).unwrap();
            black_box(actor);
        });
    });
}

criterion_group!(benches, benchmark_security_context_creation);
criterion_main!(benches);
```

**Manual Performance Test**:
```rust
#[tokio::test]
async fn test_security_context_setup_performance() {
    use std::time::Instant;
    
    let component_id = ComponentId::new("perf-test");
    let spec = ComponentSpec::default();
    let capabilities = CapabilitySet::new();
    
    let start = Instant::now();
    let actor = ComponentActor::new(component_id, spec, capabilities)
        .expect("Failed to create actor");
    let duration = start.elapsed();
    
    // Target: <1ms
    assert!(duration.as_millis() < 1, 
        "Security context setup took {}ms (target: <1ms)", 
        duration.as_millis()
    );
}
```

**Expected Outcome**: Security context setup <1ms (should be ~50-100μs in practice).

---

### Step 11: Documentation (1 hour)

**Update Module Documentation**:

**In `component_actor.rs`**:
```rust
//! # Security Integration
//!
//! Each ComponentActor has an isolated security context (`WasmSecurityContext`)
//! that defines its capabilities and trust level. The security context:
//! - Is initialized from the component manifest during actor creation
//! - Is registered with the global capability checker
//! - Persists throughout the actor's lifecycle
//! - Is restored after supervisor restarts
//! - Is cleaned up when the actor is dropped
//!
//! ## Security Context Lifecycle
//!
//! ```text
//! ComponentActor::new()
//!   → WasmSecurityContext::new()
//!   → register_component() [Phase 3]
//!   → Actor ready with security context
//!
//! Supervisor restart
//!   → restore_security_context()
//!   → ComponentActor::new()
//!   → Security context restored
//!
//! Actor drop
//!   → Drop impl
//!   → unregister_component()
//!   → Security context cleaned up
//! ```
//!
//! ## Examples
//!
//! ```rust,ignore
//! // Create actor with capabilities
//! let capabilities = CapabilitySet::new()
//!     .grant(WasmCapability::Filesystem {
//!         paths: vec!["/app/data/*".to_string()],
//!         permissions: vec!["read".to_string()],
//!     });
//!
//! let actor = ComponentActor::new(component_id, spec, capabilities)?;
//!
//! // Security context is registered and ready
//! check_capability(&component_id, "/app/data/file.txt", "read")?;
//! ```
```

**Add Examples Directory**:
Create `airssys-wasm/examples/component_actor_security.rs`:
```rust
//! Example: ComponentActor with security context.
//!
//! This example demonstrates how ComponentActor automatically manages
//! security context throughout its lifecycle.

use airssys_wasm::actor::ComponentActor;
use airssys_wasm::core::{ComponentId, ComponentSpec, CapabilitySet};
use airssys_wasm::security::{WasmCapability, check_capability};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create component with filesystem capabilities
    let component_id = ComponentId::new("example-component");
    let spec = ComponentSpec::default();
    let capabilities = CapabilitySet::new()
        .grant(WasmCapability::Filesystem {
            paths: vec!["/app/data/*".to_string()],
            permissions: vec!["read".to_string(), "write".to_string()],
        });
    
    // Create actor (security context registered automatically)
    let actor = ComponentActor::new(
        component_id.clone(),
        spec,
        capabilities
    )?;
    
    println!("✅ Component actor created with security context");
    
    // Verify capabilities work
    match check_capability(component_id.as_str(), "/app/data/config.json", "read") {
        Ok(_) => println!("✅ Read capability granted"),
        Err(e) => println!("❌ Read capability denied: {}", e),
    }
    
    match check_capability(component_id.as_str(), "/etc/passwd", "read") {
        Ok(_) => println!("❌ Unexpected access to /etc/passwd!"),
        Err(e) => println!("✅ Correctly denied access to /etc/passwd: {}", e),
    }
    
    // Actor drops here, security context cleaned up automatically
    drop(actor);
    println!("✅ Actor dropped, security context cleaned up");
    
    Ok(())
}
```

**Expected Outcome**: Comprehensive documentation with examples.

---

### Step 12: Create Completion Document (30 minutes)

**File**: `.memory-bank/sub-projects/airssys-wasm/tasks/task-005-phase-4-task-4.1-completion.md`

**Template**:
```markdown
# WASM-TASK-005 Phase 4 Task 4.1: ComponentActor Security Context Attachment - COMPLETION SUMMARY

**Task:** ComponentActor Security Context Attachment  
**Status:** ✅ COMPLETE  
**Date Completed:** YYYY-MM-DD  
**Implementation Duration:** X hours  
**Quality:** X/10

---

## Implementation Summary

### Files Modified/Created

1. **`airssys-wasm/src/actor/component/component_actor.rs`** (+XX lines)
   - Added security_context field to ComponentActor struct
   - Modified ComponentActor::new() to create and register security context
   - Added security_context() getter method
   - Implemented Drop trait for cleanup

2. **`airssys-wasm/src/actor/component/component_spawner.rs`** (+XX lines)
   - Updated spawn_component() with security validation
   - Added error handling for security failures

3. **`airssys-wasm/src/actor/component/security_lifecycle.rs`** (NEW, XXX lines)
   - Added restore_security_context() function
   - Added handle_component_restart() integration

4. **`airssys-wasm/src/core/error.rs`** (+XX lines)
   - Added Security error variants

5. **`airssys-wasm/tests/actor_security_integration_tests.rs`** (NEW, XXX lines)
   - 20+ integration tests

6. **`airssys-wasm/examples/component_actor_security.rs`** (NEW, XXX lines)
   - Working example

### Code Volume
- Production code: XXX lines
- Test code: XXX lines
- Documentation: XXX lines
- Total: XXX lines

---

## Test Results

- New tests: XX
- All tests passing: XXX/XXX
- Test pass rate: 100%
- Coverage: ~XX% (security lifecycle)

---

## Quality Metrics

- Compiler warnings: 0
- Clippy warnings: 0
- Rustdoc warnings: 0
- Code review score: X/10

---

## Performance Metrics

- Security context setup: XXμs (target: <1ms)
- Impact on message passing: 0μs (no regression)
- Context lookup: O(1) via DashMap

---

## Standards Compliance

- ✅ Microsoft Rust Guidelines
- ✅ PROJECTS_STANDARD.md
- ✅ Memory Bank documentation
- ✅ ADR-WASM-005 (Capability-Based Security)

---

## Next Steps

- Phase 4 Task 4.2: Message Passing Security (already complete per DEBT-WASM-004)
- Phase 4 Task 4.3: Resource Quota System
```

**Expected Outcome**: Complete task completion document.

---

## Test Scenarios (20+ Tests Required)

### Category 1: Security Context Attachment (5 tests)
1. `test_component_actor_has_security_context()` - Verify field exists
2. `test_security_context_registered_on_creation()` - Verify registration
3. `test_security_context_getter_works()` - Verify getter method
4. `test_security_context_immutable()` - Verify Arc immutability
5. `test_security_context_contains_correct_data()` - Verify data integrity

### Category 2: Context Isolation (4 tests)
6. `test_context_isolation_between_actors()` - Different capabilities
7. `test_duplicate_component_id_fails()` - No ID collisions
8. `test_concurrent_actor_spawns()` - Thread safety
9. `test_multiple_actors_different_trust_levels()` - Trust level isolation

### Category 3: Lifecycle Integration (5 tests)
10. `test_security_context_cleaned_up_on_drop()` - Drop cleanup
11. `test_security_context_restored_after_restart()` - Supervisor restart
12. `test_actor_restart_preserves_capabilities()` - Capability preservation
13. `test_actor_restart_preserves_trust_level()` - Trust level preservation
14. `test_termination_unregisters_context()` - Graceful termination

### Category 4: Spawn Integration (3 tests)
15. `test_spawn_validates_security_context()` - Spawn validation
16. `test_invalid_capabilities_prevent_spawn()` - Fail-fast validation
17. `test_spawn_failure_cleans_up_context()` - Cleanup on failure

### Category 5: Error Handling (3 tests)
18. `test_security_error_messages_are_clear()` - Error clarity
19. `test_registration_failure_returns_error()` - Registration errors
20. `test_invalid_trust_level_fails()` - Trust level validation

### Additional Tests (Optional)
21. `test_security_context_performance()` - Performance validation
22. `test_capability_check_during_message_handling()` - Integration check
23. `test_thread_local_context_integration()` - Thread-local usage
24. `test_security_audit_logging_on_spawn()` - Audit integration
25. `test_security_context_with_no_capabilities()` - Empty capability set

---

## Error Handling Patterns

### Registration Errors
```rust
if let Err(e) = register_component(component_id, security_context) {
    return Err(WasmError::SecurityContextRegistration(
        format!("Failed to register component {}: {}", component_id, e)
    ));
}
```

### Validation Errors
```rust
if !security_context.is_valid() {
    return Err(WasmError::CapabilityValidation(
        format!("Invalid capabilities for component {}", component_id)
    ));
}
```

### Cleanup Errors
```rust
// In Drop impl - log but don't panic
if let Err(e) = unregister_component(component_id) {
    eprintln!("Warning: Failed to unregister component {}: {}", component_id, e);
}
```

---

## Performance Targets

| Metric | Target | Expected Actual | Verification |
|--------|--------|-----------------|--------------|
| Security context setup | <1ms | ~50-100μs | Manual timing test |
| Message passing impact | 0μs | 0μs | Existing benchmarks |
| Context lookup | O(1) | O(1) | DashMap architecture |
| Registration overhead | <100μs | ~10-20μs | Benchmark |
| Unregistration overhead | <100μs | ~10-20μs | Benchmark |

---

## Standards Compliance Checklist

### Microsoft Rust Guidelines
- [ ] M-DESIGN-FOR-AI: Clear, documented APIs
- [ ] M-ESSENTIAL-FN-INHERENT: Getter methods on impl
- [ ] M-CANONICAL-DOCS: Module-level documentation
- [ ] M-SAFE-BY-DEFAULT: Security fail-fast
- [ ] M-ERROR-CONTEXT: Clear error messages

### PROJECTS_STANDARD.md
- [ ] §4.3: Module structure (mod.rs re-exports)
- [ ] §5.1: Dependencies (crate imports)
- [ ] §6.1-6.3: Testing (unit + integration + doc tests)

### Memory Bank Standards
- [ ] Kebab-case task files
- [ ] Plan → Implementation → Completion flow
- [ ] Completion document with metrics

---

## Success Criteria

- ✅ ComponentActor has security_context field
- ✅ Security context initialized in ComponentActor::new()
- ✅ Security context registered with global checker
- ✅ Security context restored after supervisor restart
- ✅ Security context cleaned up on actor drop
- ✅ 20+ integration tests passing
- ✅ Context isolation verified between actors
- ✅ Performance target met (<1ms setup)
- ✅ Zero warnings (compiler + clippy + rustdoc)
- ✅ Comprehensive documentation with examples
- ✅ All existing tests still passing

---

## Timeline Estimate

| Activity | Estimated Time | Notes |
|----------|----------------|-------|
| Step 1: Read codebase | 30 min | Understand integration points |
| Step 2: Modify struct | 1 hour | Add field, update docs |
| Step 3: Update constructor | 2 hours | Build + register context |
| Step 4: Add Drop impl | 30 min | Cleanup logic |
| Step 5: Spawn integration | 1 hour | Validate on spawn |
| Step 6: Restoration | 2 hours | Supervisor restart logic |
| Step 7: Error handling | 30 min | Add error variants |
| Step 8: Integration tests | 3 hours | 20+ tests |
| Step 9: Run tests | 1 hour | Verify + fix issues |
| Step 10: Performance | 1 hour | Benchmarks + validation |
| Step 11: Documentation | 1 hour | Rustdoc + examples |
| Step 12: Completion doc | 30 min | Summary report |
| **Total** | **12-14 hours** | **1-2 days** |

---

## Dependencies

### Required (Must be complete)
- ✅ Phase 3 Task 3.1: Capability Check API (register/check/unregister)
- ✅ Phase 3 Task 3.2: Host Function Integration (thread-local context)
- ✅ Phase 3 Task 3.3: Audit Logging Integration
- ✅ Phase 1 Task 1.3: WasmSecurityContext
- ✅ Block 3: ComponentActor system

### Optional (Nice to have)
- Phase 2 Task 2.3: Trust Configuration System (for trust level)

---

## Risk Mitigation

### Risk 1: ComponentActor struct already complex
**Mitigation**: Add single field (Arc), minimal impact on existing code.

### Risk 2: Spawn logic modification may break existing tests
**Mitigation**: Run existing tests frequently, fix regressions immediately.

### Risk 3: Drop impl errors may be silent
**Mitigation**: Log errors to stderr, consider metrics/monitoring integration.

### Risk 4: Performance regression
**Mitigation**: Benchmark before/after, security context creation is lightweight.

---

## References

- **Plan**: `.memory-bank/sub-projects/airssys-wasm/tasks/task-005-phase-4-task-4.1-plan.md`
- **Phase 3 Completion**: `.memory-bank/sub-projects/airssys-wasm/tasks/task-005-phase-3-task-3.3-completion.md`
- **ComponentActor**: `airssys-wasm/src/actor/component/component_actor.rs`
- **WasmSecurityContext**: `airssys-wasm/src/security/capability.rs`
- **Capability API**: `airssys-wasm/src/security/enforcement.rs`
- **ADR-WASM-005**: Capability-Based Security Model
- **ADR-WASM-006**: ComponentActor Pattern

---

**End of Plan**
