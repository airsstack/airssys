// Layer 1: Standard library imports (per PROJECTS_STANDARD.md §2.1)
// None needed for this module

// Layer 2: Third-party crate imports (per PROJECTS_STANDARD.md §2.1)
// None needed for this module

// Layer 3: Internal module imports (per PROJECTS_STANDARD.md §2.1)
use super::id::ComponentId;

/// Opaque handle to a loaded component instance.
///
/// A ComponentHandle wraps a ComponentId with an internal handle identifier
/// used by the runtime engine to reference the loaded WASM component instance.
///
/// This is an opaque type - external code cannot modify its internal state.
/// Access is provided only through getter methods that return references or
/// copies of the internal values.
///
/// # Architecture Note
///
/// ComponentHandle lives in `core/component/` (Layer 1) as a pure data structure.
/// The actual WASM execution happens in the `runtime/` module (Layer 2B),
/// which uses ComponentHandle as a reference to loaded components.
///
/// # Examples
///
/// ```rust
/// use airssys_wasm::core::component::id::ComponentId;
/// use airssys_wasm::core::component::handle::ComponentHandle;
///
/// let id = ComponentId::new("system", "database", "prod");
/// let handle = ComponentHandle::new(id, 12345);
///
/// // Access component ID
/// assert_eq!(handle.id().to_string_id(), "system/database/prod");
///
/// // Access internal handle ID
/// assert_eq!(handle.handle_id(), 12345);
/// ```
#[derive(Debug, Clone)]
pub struct ComponentHandle {
    /// Component identifier
    id: ComponentId,
    /// Internal runtime handle identifier
    handle_id: u64,
}

impl ComponentHandle {
    /// Creates a new ComponentHandle from a ComponentId and handle identifier.
    ///
    /// # Arguments
    ///
    /// * `id` - Component identifier
    /// * `handle_id` - Internal runtime handle identifier (typically from Wasmtime)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::component::id::ComponentId;
    /// use airssys_wasm::core::component::handle::ComponentHandle;
    ///
    /// let id = ComponentId::new("system", "database", "prod");
    /// let handle = ComponentHandle::new(id, 42);
    /// ```
    pub fn new(id: ComponentId, handle_id: u64) -> Self {
        Self { id, handle_id }
    }

    /// Returns a reference to the ComponentId.
    ///
    /// This provides read-only access to the component's identifier.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::component::{ComponentId, ComponentHandle};
    ///
    /// let id = ComponentId::new("system", "database", "prod");
    /// let handle = ComponentHandle::new(id.clone(), 42);
    ///
    /// assert_eq!(handle.id(), &id);
    /// ```
    pub fn id(&self) -> &ComponentId {
        &self.id
    }

    /// Returns the internal handle identifier.
    ///
    /// This identifier is used by the runtime engine to reference the
    /// specific loaded WASM component instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::component::{ComponentId, ComponentHandle};
    ///
    /// let id = ComponentId::new("system", "database", "prod");
    /// let handle = ComponentHandle::new(id, 12345);
    ///
    /// assert_eq!(handle.handle_id(), 12345);
    /// ```
    pub fn handle_id(&self) -> u64 {
        self.handle_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_handle_creation() {
        let id = ComponentId::new("system", "database", "prod");
        let handle = ComponentHandle::new(id.clone(), 42);

        assert_eq!(handle.id(), &id);
        assert_eq!(handle.handle_id(), 42);
    }

    #[test]
    fn test_id_getter_returns_reference() {
        let id = ComponentId::new("test", "comp", "1");
        let handle = ComponentHandle::new(id.clone(), 1);

        let returned_id = handle.id();
        assert_eq!(returned_id, &id);
    }

    #[test]
    fn test_handle_id_getter_returns_original_value() {
        let id = ComponentId::new("test", "comp", "1");
        let handle = ComponentHandle::new(id, 12345);

        assert_eq!(handle.handle_id(), 12345);
    }

    #[test]
    fn test_clone_creates_independent_copy() {
        let id = ComponentId::new("test", "comp", "1");
        let handle1 = ComponentHandle::new(id, 100);
        let handle2 = handle1.clone();

        // Both handles have same values
        assert_eq!(handle1.id(), handle2.id());
        assert_eq!(handle1.handle_id(), handle2.handle_id());

        // But are independent instances
        // (ComponentId is Clone, so both have their own copy)
        assert!(!std::ptr::eq(handle1.id(), handle2.id()));
    }

    #[test]
    fn test_multiple_handles_to_different_components() {
        let id1 = ComponentId::new("system", "database", "prod");
        let id2 = ComponentId::new("system", "cache", "dev");

        let handle1 = ComponentHandle::new(id1, 100);
        let handle2 = ComponentHandle::new(id2, 200);

        assert_eq!(handle1.id().to_string_id(), "system/database/prod");
        assert_eq!(handle1.handle_id(), 100);

        assert_eq!(handle2.id().to_string_id(), "system/cache/dev");
        assert_eq!(handle2.handle_id(), 200);
    }

    #[test]
    fn test_debug_formatting() {
        let id = ComponentId::new("test", "comp", "1");
        let handle = ComponentHandle::new(id, 42);

        let debug_str = format!("{:?}", handle);
        assert!(debug_str.contains("ComponentHandle"));
    }
}
