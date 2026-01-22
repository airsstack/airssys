# WASM-TASK-038: Implementation Plans

## References
- ADR-WASM-031 (Component & Messaging Module Design)
- ADR-WASM-026 (Implementation Roadmap)
- ADR-WASM-025 (Clean Slate Architecture)
- ADR-WASM-023 (Module Boundary Enforcement)
- PROJECTS_STANDARD.md (§2.1, §2.2, §4.3)

---

## Actions

### Action 1: Implement ComponentRegistry

**Objective**: Create thread-safe registry for tracking ComponentId → ActorAddress mappings.

**Detailed Steps**:

#### Step 1.1: Create `src/component/registry.rs`

```rust
//! Component registry for tracking loaded components and their actor addresses.
//!
//! Provides thread-safe mapping of ComponentId → ActorAddress for message routing.

// Layer 1: Standard library imports
use std::collections::HashMap;
use std::sync::RwLock;

// Layer 2: Third-party crate imports
use airssys_rt::ActorAddress;

// Layer 3: Internal module imports
use crate::core::component::id::ComponentId;

/// Registry for tracking loaded components
///
/// Thread-safe registry using RwLock for concurrent read access
/// with minimal write contention.
pub struct ComponentRegistry {
    /// Maps ComponentId to ActorAddress
    components: RwLock<HashMap<ComponentId, ActorAddress>>,
}

impl ComponentRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            components: RwLock::new(HashMap::new()),
        }
    }

    /// Register a component with its actor address
    ///
    /// # Arguments
    /// * `id` - The component identifier
    /// * `address` - The actor address for this component
    pub fn register(&self, id: ComponentId, address: ActorAddress) {
        let mut components = self.components.write().unwrap();
        components.insert(id, address);
    }

    /// Unregister a component
    ///
    /// # Arguments
    /// * `id` - The component identifier to remove
    ///
    /// # Returns
    /// The actor address if the component was registered, None otherwise
    pub fn unregister(&self, id: &ComponentId) -> Option<ActorAddress> {
        let mut components = self.components.write().unwrap();
        components.remove(id)
    }

    /// Get actor address for a component
    ///
    /// # Arguments
    /// * `id` - The component identifier to lookup
    ///
    /// # Returns
    /// The actor address if found, None otherwise
    pub fn get(&self, id: &ComponentId) -> Option<ActorAddress> {
        let components = self.components.read().unwrap();
        components.get(id).cloned()
    }

    /// Check if component is registered
    ///
    /// # Arguments
    /// * `id` - The component identifier to check
    ///
    /// # Returns
    /// true if the component is registered, false otherwise
    pub fn contains(&self, id: &ComponentId) -> bool {
        let components = self.components.read().unwrap();
        components.contains_key(id)
    }

    /// List all registered component IDs
    ///
    /// # Returns
    /// Vector of all registered ComponentIds
    pub fn list(&self) -> Vec<ComponentId> {
        let components = self.components.read().unwrap();
        components.keys().cloned().collect()
    }

    /// Count of registered components
    ///
    /// # Returns
    /// Number of registered components
    pub fn count(&self) -> usize {
        let components = self.components.read().unwrap();
        components.len()
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_mock_address() -> ActorAddress {
        // Create a mock ActorAddress for testing
        // This will use airssys-rt's test utilities
        todo!("Mock ActorAddress")
    }

    #[test]
    fn test_registry_new() {
        let registry = ComponentRegistry::new();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_registry_default() {
        let registry = ComponentRegistry::default();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_register_and_get() {
        let registry = ComponentRegistry::new();
        let id = ComponentId::new("test-component");
        let address = create_mock_address();

        registry.register(id.clone(), address.clone());

        assert!(registry.contains(&id));
        assert_eq!(registry.get(&id), Some(address));
        assert_eq!(registry.count(), 1);
    }

    #[test]
    fn test_register_multiple() {
        let registry = ComponentRegistry::new();
        let id1 = ComponentId::new("component-1");
        let id2 = ComponentId::new("component-2");
        let addr1 = create_mock_address();
        let addr2 = create_mock_address();

        registry.register(id1.clone(), addr1.clone());
        registry.register(id2.clone(), addr2.clone());

        assert_eq!(registry.count(), 2);
        assert_eq!(registry.get(&id1), Some(addr1));
        assert_eq!(registry.get(&id2), Some(addr2));
    }

    #[test]
    fn test_unregister() {
        let registry = ComponentRegistry::new();
        let id = ComponentId::new("test-component");
        let address = create_mock_address();

        registry.register(id.clone(), address.clone());
        assert_eq!(registry.count(), 1);

        let removed = registry.unregister(&id);
        assert_eq!(removed, Some(address));
        assert_eq!(registry.count(), 0);
        assert!(!registry.contains(&id));
    }

    #[test]
    fn test_unregister_nonexistent() {
        let registry = ComponentRegistry::new();
        let id = ComponentId::new("nonexistent");

        let removed = registry.unregister(&id);
        assert_eq!(removed, None);
    }

    #[test]
    fn test_get_nonexistent() {
        let registry = ComponentRegistry::new();
        let id = ComponentId::new("nonexistent");

        assert_eq!(registry.get(&id), None);
        assert!(!registry.contains(&id));
    }

    #[test]
    fn test_list_empty() {
        let registry = ComponentRegistry::new();
        let list = registry.list();
        assert!(list.is_empty());
    }

    #[test]
    fn test_list_multiple() {
        let registry = ComponentRegistry::new();
        let id1 = ComponentId::new("component-1");
        let id2 = ComponentId::new("component-2");
        let id3 = ComponentId::new("component-3");

        registry.register(id1.clone(), create_mock_address());
        registry.register(id2.clone(), create_mock_address());
        registry.register(id3.clone(), create_mock_address());

        let list = registry.list();
        assert_eq!(list.len(), 3);
        assert!(list.contains(&id1));
        assert!(list.contains(&id2));
        assert!(list.contains(&id3));
    }

    #[test]
    fn test_count() {
        let registry = ComponentRegistry::new();
        assert_eq!(registry.count(), 0);

        registry.register(ComponentId::new("c1"), create_mock_address());
        assert_eq!(registry.count(), 1);

        registry.register(ComponentId::new("c2"), create_mock_address());
        assert_eq!(registry.count(), 2);

        registry.unregister(&ComponentId::new("c1"));
        assert_eq!(registry.count(), 1);
    }

    #[test]
    fn test_register_overwrites() {
        let registry = ComponentRegistry::new();
        let id = ComponentId::new("test-component");
        let addr1 = create_mock_address();
        let addr2 = create_mock_address();

        registry.register(id.clone(), addr1);
        assert_eq!(registry.count(), 1);

        // Register same ID with different address
        registry.register(id.clone(), addr2.clone());
        assert_eq!(registry.count(), 1); // Still 1 component
        assert_eq!(registry.get(&id), Some(addr2)); // Address updated
    }
}
```

#### Step 1.2: Update `src/component/mod.rs`

```rust
//! Component module - Actor system integration for WASM components
//!
//! This module integrates WASM components with the airssys-rt actor system.

pub mod registry;
pub mod wrapper;

pub use registry::ComponentRegistry;
pub use wrapper::{ComponentActorMessage, ComponentWrapper};
```

**Deliverables**:
- `src/component/registry.rs` with ComponentRegistry struct
- All CRUD methods (register, unregister, get, contains, list, count)
- Default trait implementation
- Comprehensive unit tests (12+ tests)
- Module export in `src/component/mod.rs`

**Constraints**:
- Must not import from `runtime/`, `security/`, or `system/`
- Must follow §2.1 3-Layer imports
- Must follow §2.2 No FQN in type annotations
- Must use RwLock for thread safety

---

## Verification Section

### Automated Tests
```bash
# Unit tests for registry module
cargo test -p airssys-wasm --lib -- component::registry

# All component module tests
cargo test -p airssys-wasm --lib -- component

# Build verification
cargo build -p airssys-wasm

# Clippy
cargo clippy -p airssys-wasm --lib -- -D warnings
```

### Architecture Compliance
```bash
# Verify no forbidden imports in registry.rs
grep -rn "use crate::runtime" src/component/registry.rs  # Should be empty
grep -rn "use crate::security" src/component/registry.rs  # Should be empty
grep -rn "use crate::system" src/component/registry.rs    # Should be empty

# Verify no FQN in type annotations
grep -rn "std::" src/component/registry.rs | grep -v "^.*use " | grep "::"  # Should be empty
```

---

## Success Criteria
- [ ] `src/component/registry.rs` exists and compiles
- [ ] ComponentRegistry struct with RwLock<HashMap>
- [ ] All 6 methods implemented (register, unregister, get, contains, list, count)
- [ ] Default trait implementation
- [ ] Unit tests pass (12+ tests)
- [ ] Thread-safety verified (RwLock usage)
- [ ] `cargo clippy -p airssys-wasm --lib -- -D warnings` passes
- [ ] No forbidden imports (architecture compliance)
- [ ] §2.1 3-Layer imports verified
- [ ] §2.2 No FQN verified
