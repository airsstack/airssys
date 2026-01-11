//! Capability grant management.

use super::set::CapabilitySet;
use crate::core::component::id::ComponentId;

/// Represents a capability grant to a component.
///
/// Associates a component with a set of capabilities and optional expiration.
#[derive(Debug, Clone)]
pub struct CapabilityGrant {
    /// The component receiving the grant.
    pub component: ComponentId,
    /// The set of capabilities granted.
    pub capabilities: CapabilitySet,
    /// Optional expiration timestamp (ms since epoch).
    pub expires_at: Option<u64>,
}

impl CapabilityGrant {
    /// Create a new capability grant.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::component::id::ComponentId;
    /// use airssys_wasm::security::capability::{set::CapabilitySet, grant::CapabilityGrant};
    ///
    /// let component = ComponentId::new("test", "component", "1");
    /// let capabilities = CapabilitySet::new();
    /// let grant = CapabilityGrant::new(component, capabilities);
    ///
    /// assert!(!grant.is_expired(1000));
    /// ```
    pub fn new(component: ComponentId, capabilities: CapabilitySet) -> Self {
        Self {
            component,
            capabilities,
            expires_at: None,
        }
    }

    /// Create a new capability grant with expiration.
    ///
    /// # Arguments
    ///
    /// * `component` - The component receiving the grant
    /// * `capabilities` - The set of capabilities granted
    /// * `expires_at` - Expiration timestamp (ms since epoch)
    pub fn with_expiration(
        component: ComponentId,
        capabilities: CapabilitySet,
        expires_at: u64,
    ) -> Self {
        Self {
            component,
            capabilities,
            expires_at: Some(expires_at),
        }
    }

    /// Check if the grant has expired.
    ///
    /// # Arguments
    ///
    /// * `current_time_ms` - Current time in milliseconds since epoch
    ///
    /// # Returns
    ///
    /// `true` if the grant has expired, `false` otherwise
    pub fn is_expired(&self, current_time_ms: u64) -> bool {
        self.expires_at.is_some_and(|exp| current_time_ms > exp)
    }

    /// Get the component ID.
    pub fn component(&self) -> &ComponentId {
        &self.component
    }

    /// Get the capability set.
    pub fn capabilities(&self) -> &CapabilitySet {
        &self.capabilities
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::capability::set::MessagingPermission;

    #[test]
    fn test_create_grant() {
        let component = ComponentId::new("test", "component", "1");
        let capabilities = CapabilitySet::new();
        let grant = CapabilityGrant::new(component.clone(), capabilities);

        assert_eq!(grant.component(), &component);
        assert!(!grant.is_expired(1000));
    }

    #[test]
    fn test_create_grant_with_expiration() {
        let component = ComponentId::new("test", "component", "1");
        let capabilities = CapabilitySet::new();
        let grant = CapabilityGrant::with_expiration(component, capabilities, 1000);

        assert!(!grant.is_expired(999));
        assert!(grant.is_expired(1001));
    }

    #[test]
    fn test_grant_no_expiration() {
        let component = ComponentId::new("test", "component", "1");
        let capabilities = CapabilitySet::new();
        let grant = CapabilityGrant::new(component, capabilities);

        assert!(!grant.is_expired(0));
        assert!(!grant.is_expired(u64::MAX));
    }

    #[test]
    fn test_grant_with_capabilities() {
        let component = ComponentId::new("test", "component", "1");
        let mut capabilities = CapabilitySet::new();
        capabilities.add_messaging(MessagingPermission {
            can_send_to: vec!["*".to_string()],
            can_receive_from: vec![],
        });

        let grant = CapabilityGrant::new(component.clone(), capabilities);
        assert_eq!(grant.component(), &component);
        assert!(grant.capabilities().can_send_to("any"));
    }
}
