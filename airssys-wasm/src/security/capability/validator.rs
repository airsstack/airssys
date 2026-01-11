//! Capability validator implementation.
//!
//! Provides thread-safe capability validation for components using a
//! read-write lock protected HashMap of ComponentId -> CapabilitySet.

// Layer 1: Standard library imports (per PROJECTS_STANDARD.md §2.1)
use std::collections::HashMap;
use std::sync::RwLock;

// Layer 3: Internal module imports
use crate::core::component::id::ComponentId;
use crate::core::security::capability::{Capability, MessagingAction, StorageAction};
use crate::core::security::errors::SecurityError;
use crate::core::security::traits::SecurityValidator;

use super::set::CapabilitySet;

/// Implementation of SecurityValidator trait.
///
/// Stores component capabilities in a thread-safe HashMap and validates
/// capability requests against registered permissions.
///
/// # Examples
///
/// ```
/// use airssys_wasm::core::component::id::ComponentId;
/// use airssys_wasm::security::capability::{
///     CapabilitySet,
///     CapabilityValidator,
///     MessagingPermission,
/// };
///
/// let validator = CapabilityValidator::new();
///
/// // Register a component with messaging permissions
/// let component_id = ComponentId::new("org", "service", "inst-1");
/// let capabilities = CapabilitySet::builder()
///     .messaging(MessagingPermission {
///         can_send_to: vec!["org.target/*".to_string()],
///         can_receive_from: vec![],
///     })
///     .build();
///
/// validator.register_component(component_id.clone(), capabilities);
///
/// // Check if component can send to a target
/// let target = ComponentId::new("org", "target", "inst-2");
/// assert!(validator.can_send_to(&component_id, &target).is_ok());
/// ```
#[derive(Debug, Default)]
pub struct CapabilityValidator {
    /// Registered capabilities per component.
    capabilities: RwLock<HashMap<ComponentId, CapabilitySet>>,
}

impl CapabilityValidator {
    /// Create a new empty CapabilityValidator.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::security::capability::CapabilityValidator;
    ///
    /// let validator = CapabilityValidator::new();
    /// ```
    pub fn new() -> Self {
        Self {
            capabilities: RwLock::new(HashMap::new()),
        }
    }

    /// Register capabilities for a component.
    ///
    /// # Arguments
    ///
    /// * `id` - The component ID
    /// * `capabilities` - The capability set to register
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::component::id::ComponentId;
    /// use airssys_wasm::security::capability::{
    ///     CapabilitySet,
    ///     CapabilityValidator,
    ///     MessagingPermission,
    /// };
    ///
    /// let validator = CapabilityValidator::new();
    /// let component_id = ComponentId::new("org", "service", "inst-1");
    /// let capabilities = CapabilitySet::new();
    ///
    /// validator.register_component(component_id.clone(), capabilities);
    /// ```
    pub fn register_component(&self, id: ComponentId, capabilities: CapabilitySet) {
        let mut caps = self.capabilities.write().unwrap();
        caps.insert(id, capabilities);
    }

    /// Unregister a component.
    ///
    /// # Arguments
    ///
    /// * `id` - The component ID to unregister
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::component::id::ComponentId;
    /// use airssys_wasm::security::capability::{
    ///     CapabilitySet,
    ///     CapabilityValidator,
    /// };
    ///
    /// let validator = CapabilityValidator::new();
    /// let component_id = ComponentId::new("org", "service", "inst-1");
    ///
    /// validator.register_component(component_id.clone(), CapabilitySet::new());
    /// validator.unregister_component(&component_id);
    /// ```
    pub fn unregister_component(&self, id: &ComponentId) {
        let mut caps = self.capabilities.write().unwrap();
        caps.remove(id);
    }
}

impl SecurityValidator for CapabilityValidator {
    /// Validate if component has required capability.
    ///
    /// # Arguments
    ///
    /// * `component` - The component requesting the capability
    /// * `capability` - The capability being requested
    ///
    /// # Returns
    ///
    /// * `Ok(())` if capability is granted
    /// * `Err(SecurityError)` if capability is denied or component not registered
    fn validate_capability(
        &self,
        component: &ComponentId,
        capability: &Capability,
    ) -> Result<(), SecurityError> {
        let caps = self.capabilities.read().unwrap();

        let component_caps = caps.get(component).ok_or_else(|| {
            SecurityError::CapabilityDenied(format!("Component {} not registered", component))
        })?;

        match capability {
            // For messaging capabilities, we verify that the target pattern
            // matches a pattern in the component's send permissions
            Capability::Messaging(msg_cap) => {
                // Convert action to string for error messages
                let action_str = match msg_cap.action {
                    MessagingAction::Send => "send",
                    MessagingAction::Request => "request",
                    MessagingAction::Broadcast => "broadcast",
                };

                // Check if component has permission to send to this target pattern
                if !component_caps.can_send_to(&msg_cap.target_pattern) {
                    return Err(SecurityError::CapabilityDenied(format!(
                        "Messaging capability denied for {}: cannot {} to {}",
                        component, action_str, msg_cap.target_pattern
                    )));
                }
            }

            // For storage capabilities, we verify that the namespace pattern
            // matches a pattern in the component's read/write permissions
            Capability::Storage(storage_cap) => {
                // Convert action to string for error messages
                let action_str = match storage_cap.action {
                    StorageAction::Read => "read",
                    StorageAction::Write => "write",
                    StorageAction::Delete => "delete",
                };

                // Check if component has permission for this namespace pattern
                let has_permission = match storage_cap.action {
                    StorageAction::Read => {
                        component_caps.can_read_key(&storage_cap.namespace_pattern)
                    }
                    StorageAction::Write => {
                        component_caps.can_write_key(&storage_cap.namespace_pattern)
                    }
                    StorageAction::Delete => {
                        component_caps.can_write_key(&storage_cap.namespace_pattern)
                    } // Delete requires write
                };

                if !has_permission {
                    return Err(SecurityError::CapabilityDenied(format!(
                        "Storage capability denied for {}: cannot {} namespace {}",
                        component, action_str, storage_cap.namespace_pattern
                    )));
                }
            }

            // Other capability types (Filesystem, Network) are not yet supported
            Capability::Filesystem(_) => {
                return Err(SecurityError::CapabilityDenied(format!(
                    "Filesystem capability not supported for {}",
                    component
                )));
            }

            Capability::Network(_) => {
                return Err(SecurityError::CapabilityDenied(format!(
                    "Network capability not supported for {}",
                    component
                )));
            }
        }

        Ok(())
    }

    /// Check if component can send message to target.
    ///
    /// # Arguments
    ///
    /// * `sender` - The sending component
    /// * `target` - The target component
    ///
    /// # Returns
    ///
    /// * `Ok(())` if sending is allowed
    /// * `Err(SecurityError)` if sending is denied or sender not registered
    fn can_send_to(&self, sender: &ComponentId, target: &ComponentId) -> Result<(), SecurityError> {
        let caps = self.capabilities.read().unwrap();

        let sender_caps = caps.get(sender).ok_or_else(|| {
            SecurityError::CapabilityDenied(format!("Sender {} not registered", sender))
        })?;

        let target_str = target.to_string_id();
        if !sender_caps.can_send_to(&target_str) {
            return Err(SecurityError::PermissionDenied(format!(
                "{} cannot send to {}",
                sender, target
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::security::capability::{MessagingCapability, StorageCapability};
    use crate::security::capability::set::{MessagingPermission, StoragePermission};

    #[test]
    fn test_new_validator_has_empty_capabilities() {
        let validator = CapabilityValidator::new();

        // Create a component ID to test with
        let component_id = ComponentId::new("org", "service", "inst-1");

        // Verify that an unregistered component cannot validate capabilities
        let msg_cap = MessagingCapability {
            action: MessagingAction::Send,
            target_pattern: "any/target".to_string(),
        };
        let result = validator.validate_capability(&component_id, &Capability::Messaging(msg_cap));

        // Should fail with CapabilityDenied indicating component is not registered
        assert!(result.is_err());
        assert!(matches!(result, Err(SecurityError::CapabilityDenied(_))));
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("not registered"));
    }

    #[test]
    fn test_register_component_with_capabilities() {
        let validator = CapabilityValidator::new();
        let component_id = ComponentId::new("org", "service", "inst-1");

        let capabilities = CapabilitySet::builder()
            .messaging(MessagingPermission {
                can_send_to: vec!["org/target/*".to_string()],
                can_receive_from: vec![],
            })
            .build();

        // Register should succeed without panicking
        validator.register_component(component_id.clone(), capabilities);

        // Verify component is registered by checking can_send_to
        let target = ComponentId::new("org", "target", "inst-2");
        assert!(validator.can_send_to(&component_id, &target).is_ok());
    }

    #[test]
    fn test_unregister_component() {
        let validator = CapabilityValidator::new();
        let component_id = ComponentId::new("org", "service", "inst-1");

        let capabilities = CapabilitySet::builder()
            .messaging(MessagingPermission {
                can_send_to: vec!["org/target/*".to_string()],
                can_receive_from: vec![],
            })
            .build();

        validator.register_component(component_id.clone(), capabilities);

        // Verify registration worked
        let target = ComponentId::new("org", "target", "inst-2");
        assert!(validator.can_send_to(&component_id, &target).is_ok());

        // Unregister component
        validator.unregister_component(&component_id);

        // Verify unregistration worked - should return error
        let result = validator.can_send_to(&component_id, &target);
        assert!(result.is_err());
        assert!(matches!(result, Err(SecurityError::CapabilityDenied(_))));
    }

    #[test]
    fn test_validate_messaging_capability_granted() {
        let validator = CapabilityValidator::new();
        let component_id = ComponentId::new("org", "service", "inst-1");

        let capabilities = CapabilitySet::builder()
            .messaging(MessagingPermission {
                can_send_to: vec!["org.target/*".to_string()],
                can_receive_from: vec![],
            })
            .build();

        validator.register_component(component_id.clone(), capabilities);

        let msg_cap = MessagingCapability {
            action: MessagingAction::Send,
            target_pattern: "org.target/inst-2".to_string(),
        };

        let result = validator.validate_capability(&component_id, &Capability::Messaging(msg_cap));
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_messaging_capability_denied() {
        let validator = CapabilityValidator::new();
        let component_id = ComponentId::new("org", "service", "inst-1");

        let capabilities = CapabilitySet::builder()
            .messaging(MessagingPermission {
                can_send_to: vec!["org.allowed/*".to_string()],
                can_receive_from: vec![],
            })
            .build();

        validator.register_component(component_id.clone(), capabilities);

        let msg_cap = MessagingCapability {
            action: MessagingAction::Send,
            target_pattern: "org.denied/inst-2".to_string(),
        };

        let result = validator.validate_capability(&component_id, &Capability::Messaging(msg_cap));
        assert!(result.is_err());
        assert!(matches!(result, Err(SecurityError::CapabilityDenied(_))));
    }

    #[test]
    fn test_validate_storage_capability_granted() {
        let validator = CapabilityValidator::new();
        let component_id = ComponentId::new("org", "service", "inst-1");

        let capabilities = CapabilitySet::builder()
            .storage(StoragePermission {
                can_write_keys: vec!["user/*".to_string()],
                can_read_keys: vec!["*".to_string()],
            })
            .build();

        validator.register_component(component_id.clone(), capabilities);

        let storage_cap = StorageCapability {
            action: StorageAction::Read,
            namespace_pattern: "user/data".to_string(),
        };

        let result =
            validator.validate_capability(&component_id, &Capability::Storage(storage_cap));
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_storage_capability_denied() {
        let validator = CapabilityValidator::new();
        let component_id = ComponentId::new("org", "service", "inst-1");

        let capabilities = CapabilitySet::builder()
            .storage(StoragePermission {
                can_write_keys: vec!["user/*".to_string()],
                can_read_keys: vec!["user/*".to_string()],
            })
            .build();

        validator.register_component(component_id.clone(), capabilities);

        let storage_cap = StorageCapability {
            action: StorageAction::Read,
            namespace_pattern: "system/data".to_string(),
        };

        let result =
            validator.validate_capability(&component_id, &Capability::Storage(storage_cap));
        assert!(result.is_err());
        assert!(matches!(result, Err(SecurityError::CapabilityDenied(_))));
    }

    #[test]
    fn test_can_send_to_granted_sender_has_permission() {
        let validator = CapabilityValidator::new();
        let sender_id = ComponentId::new("org", "sender", "inst-1");
        let target_id = ComponentId::new("org", "target", "inst-2");

        let capabilities = CapabilitySet::builder()
            .messaging(MessagingPermission {
                can_send_to: vec!["org/target/*".to_string()],
                can_receive_from: vec![],
            })
            .build();

        validator.register_component(sender_id.clone(), capabilities);

        let result = validator.can_send_to(&sender_id, &target_id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_can_send_to_denied_sender_lacks_permission() {
        let validator = CapabilityValidator::new();
        let sender_id = ComponentId::new("org", "sender", "inst-1");
        let target_id = ComponentId::new("org", "forbidden", "inst-2");

        let capabilities = CapabilitySet::builder()
            .messaging(MessagingPermission {
                can_send_to: vec!["org.allowed/*".to_string()],
                can_receive_from: vec![],
            })
            .build();

        validator.register_component(sender_id.clone(), capabilities);

        let result = validator.can_send_to(&sender_id, &target_id);
        assert!(result.is_err());
        assert!(matches!(result, Err(SecurityError::PermissionDenied(_))));
    }

    #[test]
    fn test_thread_safety_concurrent_read_write_operations() {
        use std::sync::Arc;
        use std::thread;

        let validator = Arc::new(CapabilityValidator::new());

        // Spawn threads that concurrently read and write
        let mut handles = vec![];

        // Create a writer thread
        for i in 0..10 {
            let validator_clone = Arc::clone(&validator);
            let handle = thread::spawn(move || {
                let component_id = ComponentId::new("org", "service", format!("inst-{}", i));
                let capabilities = CapabilitySet::builder()
                    .messaging(MessagingPermission {
                        can_send_to: vec!["*".to_string()],
                        can_receive_from: vec![],
                    })
                    .build();
                validator_clone.register_component(component_id, capabilities);
            });
            handles.push(handle);
        }

        // Create reader threads
        for i in 0..20 {
            let validator_clone = Arc::clone(&validator);
            let handle = thread::spawn(move || {
                let sender_id = ComponentId::new("org", "service", "inst-1");
                let target_id = ComponentId::new("org", "target", format!("inst-{}", i));
                let _ = validator_clone.can_send_to(&sender_id, &target_id);
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().expect("Thread panicked");
        }

        // If we got here without panicking, thread safety is verified
    }
}
