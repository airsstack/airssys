# ADR-WASM-029: Security Module Design

**ADR ID:** ADR-WASM-029  
**Created:** 2026-01-05  
**Status:** Accepted  
**Deciders:** Architecture Team  
**Category:** Module Design / Security  
**Parent:** [ADR-WASM-026](adr-wasm-026-implementation-roadmap-clean-slate-rebuild.md) (Phase 4)

---

## Title

Security Module Design with airssys-osl Integration

---

## Context

The `security/` module is **Layer 2A** of the architecture. It:
- Implements `core/security/traits.rs` (SecurityValidator, SecurityAuditLogger)
- Integrates with airssys-osl for SecurityContext
- Provides capability-based security enforcement

**Import Rules:**
- ✅ Can import: `core/`
- ✅ Can import: `airssys-osl` (external)
- ❌ Cannot import: `runtime/`, `component/`, `messaging/`, `system/`

### References

- [ADR-WASM-005](adr-wasm-005-capability-based-security-model.md): Capability-Based Security Model
- [ADR-WASM-025](adr-wasm-025-clean-slate-rebuild-architecture.md): Clean-Slate Architecture
- [KNOWLEDGE-WASM-020](../knowledges/knowledge-wasm-020-airssys-osl-security-integration.md): OSL Integration

---

## Decision

### Security Module Structure

```
security/
├── mod.rs
├── capability/
│   ├── mod.rs
│   ├── types.rs        # Capability enum, permission types
│   ├── set.rs          # CapabilitySet for component permissions
│   ├── validator.rs    # CapabilityValidator implementation
│   └── grant.rs        # CapabilityGrant (permission grants)
├── policy/
│   ├── mod.rs
│   ├── engine.rs       # PolicyEngine for rule evaluation
│   └── rules.rs        # SecurityPolicy, PolicyRule types
└── audit.rs            # SecurityAuditLogger implementation
```

---

## Detailed Specifications

### security/capability/types.rs

```rust
use crate::core::security::capability::{
    Capability, MessagingCapability, StorageCapability,
    FilesystemCapability, NetworkCapability,
};

// Re-export from core
pub use crate::core::security::capability::*;

/// Pattern matcher for capability validation
pub struct PatternMatcher;

impl PatternMatcher {
    /// Match a target against a pattern (glob-like)
    pub fn matches(pattern: &str, target: &str) -> bool {
        if pattern == "*" {
            return true;
        }
        if pattern.ends_with("/*") {
            let prefix = &pattern[..pattern.len() - 2];
            return target.starts_with(prefix);
        }
        pattern == target
    }
}
```

---

### security/capability/set.rs

```rust
use crate::core::security::capability::Capability;
use std::collections::HashSet;

/// Set of capabilities granted to a component
#[derive(Debug, Clone, Default)]
pub struct CapabilitySet {
    messaging: Vec<MessagingPermission>,
    storage: Vec<StoragePermission>,
    filesystem: Vec<FilesystemPermission>,
    network: Vec<NetworkPermission>,
}

#[derive(Debug, Clone)]
pub struct MessagingPermission {
    pub action: String,
    pub target_pattern: String,
}

#[derive(Debug, Clone)]
pub struct StoragePermission {
    pub action: String,
    pub namespace_pattern: String,
}

#[derive(Debug, Clone)]
pub struct FilesystemPermission {
    pub action: String,
    pub path_pattern: String,
}

#[derive(Debug, Clone)]
pub struct NetworkPermission {
    pub action: String,
    pub host_pattern: String,
    pub port: Option<u16>,
}

impl CapabilitySet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_messaging(&mut self, action: &str, target_pattern: &str) {
        self.messaging.push(MessagingPermission {
            action: action.to_string(),
            target_pattern: target_pattern.to_string(),
        });
    }

    pub fn add_storage(&mut self, action: &str, namespace_pattern: &str) {
        self.storage.push(StoragePermission {
            action: action.to_string(),
            namespace_pattern: namespace_pattern.to_string(),
        });
    }

    pub fn has_messaging_permission(&self, action: &str, target: &str) -> bool {
        self.messaging.iter().any(|p| {
            p.action == action && PatternMatcher::matches(&p.target_pattern, target)
        })
    }

    pub fn has_storage_permission(&self, action: &str, namespace: &str) -> bool {
        self.storage.iter().any(|p| {
            p.action == action && PatternMatcher::matches(&p.namespace_pattern, namespace)
        })
    }
}
```

---

### security/capability/validator.rs

```rust
use std::collections::HashMap;
use std::sync::RwLock;

use crate::core::component::id::ComponentId;
use crate::core::errors::security::SecurityError;
use crate::core::security::capability::Capability;
use crate::core::security::traits::SecurityValidator;

use super::set::CapabilitySet;

/// Implementation of SecurityValidator trait
pub struct CapabilityValidator {
    /// Registered capabilities per component
    capabilities: RwLock<HashMap<ComponentId, CapabilitySet>>,
}

impl CapabilityValidator {
    pub fn new() -> Self {
        Self {
            capabilities: RwLock::new(HashMap::new()),
        }
    }

    /// Register capabilities for a component
    pub fn register_component(&self, id: ComponentId, capabilities: CapabilitySet) {
        let mut caps = self.capabilities.write().unwrap();
        caps.insert(id, capabilities);
    }

    /// Unregister a component
    pub fn unregister_component(&self, id: &ComponentId) {
        let mut caps = self.capabilities.write().unwrap();
        caps.remove(id);
    }
}

impl SecurityValidator for CapabilityValidator {
    fn validate_capability(
        &self,
        component: &ComponentId,
        capability: &Capability,
    ) -> Result<(), SecurityError> {
        let caps = self.capabilities.read().unwrap();
        
        let component_caps = caps.get(component).ok_or_else(|| {
            SecurityError::CapabilityDenied(format!(
                "Component {} not registered",
                component
            ))
        })?;

        match capability {
            Capability::Messaging(msg_cap) => {
                if !component_caps.has_messaging_permission(
                    &format!("{:?}", msg_cap.action),
                    &msg_cap.target_pattern,
                ) {
                    return Err(SecurityError::CapabilityDenied(format!(
                        "Messaging capability denied for {}",
                        component
                    )));
                }
            }
            Capability::Storage(storage_cap) => {
                if !component_caps.has_storage_permission(
                    &format!("{:?}", storage_cap.action),
                    &storage_cap.namespace_pattern,
                ) {
                    return Err(SecurityError::CapabilityDenied(format!(
                        "Storage capability denied for {}",
                        component
                    )));
                }
            }
            // Handle other capability types...
            _ => {}
        }

        Ok(())
    }

    fn can_send_to(
        &self,
        sender: &ComponentId,
        target: &ComponentId,
    ) -> Result<(), SecurityError> {
        let caps = self.capabilities.read().unwrap();
        
        let sender_caps = caps.get(sender).ok_or_else(|| {
            SecurityError::CapabilityDenied(format!(
                "Sender {} not registered",
                sender
            ))
        })?;

        let target_str = target.to_string_id();
        if !sender_caps.has_messaging_permission("Send", &target_str) {
            return Err(SecurityError::PermissionDenied(format!(
                "{} cannot send to {}",
                sender, target
            )));
        }

        Ok(())
    }
}
```

---

### security/policy/engine.rs

```rust
use crate::core::component::id::ComponentId;
use crate::core::errors::security::SecurityError;

use super::rules::{PolicyRule, SecurityPolicy};

/// Policy evaluation engine
pub struct PolicyEngine {
    policies: Vec<SecurityPolicy>,
}

impl PolicyEngine {
    pub fn new() -> Self {
        Self {
            policies: Vec::new(),
        }
    }

    pub fn add_policy(&mut self, policy: SecurityPolicy) {
        self.policies.push(policy);
    }

    /// Evaluate all policies for a component action
    pub fn evaluate(
        &self,
        component: &ComponentId,
        action: &str,
        resource: &str,
    ) -> Result<(), SecurityError> {
        for policy in &self.policies {
            if policy.applies_to(component) {
                policy.evaluate(action, resource)?;
            }
        }
        Ok(())
    }
}
```

---

### security/policy/rules.rs

```rust
use crate::core::component::id::ComponentId;
use crate::core::errors::security::SecurityError;

/// Security policy containing rules
#[derive(Debug, Clone)]
pub struct SecurityPolicy {
    pub name: String,
    pub component_pattern: String,
    pub rules: Vec<PolicyRule>,
}

/// Individual policy rule
#[derive(Debug, Clone)]
pub struct PolicyRule {
    pub action: String,
    pub resource_pattern: String,
    pub effect: PolicyEffect,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PolicyEffect {
    Allow,
    Deny,
}

impl SecurityPolicy {
    pub fn new(name: &str, component_pattern: &str) -> Self {
        Self {
            name: name.to_string(),
            component_pattern: component_pattern.to_string(),
            rules: Vec::new(),
        }
    }

    pub fn add_rule(&mut self, rule: PolicyRule) {
        self.rules.push(rule);
    }

    pub fn applies_to(&self, component: &ComponentId) -> bool {
        let component_str = component.to_string_id();
        self.component_pattern == "*" || component_str.starts_with(&self.component_pattern)
    }

    pub fn evaluate(&self, action: &str, resource: &str) -> Result<(), SecurityError> {
        for rule in &self.rules {
            if rule.action == action || rule.action == "*" {
                if rule.resource_pattern == "*" || resource.starts_with(&rule.resource_pattern) {
                    if rule.effect == PolicyEffect::Deny {
                        return Err(SecurityError::PolicyViolation(format!(
                            "Policy {} denies {} on {}",
                            self.name, action, resource
                        )));
                    }
                }
            }
        }
        Ok(())
    }
}
```

---

### security/audit.rs

```rust
use std::sync::mpsc::{self, Sender};
use std::thread;

use crate::core::component::id::ComponentId;
use crate::core::security::traits::{SecurityAuditLogger, SecurityEvent};

/// Console-based security audit logger
pub struct ConsoleSecurityAuditLogger {
    sender: Sender<SecurityEvent>,
}

impl ConsoleSecurityAuditLogger {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel::<SecurityEvent>();

        // Background thread for async logging
        thread::spawn(move || {
            while let Ok(event) = receiver.recv() {
                let status = if event.granted { "GRANTED" } else { "DENIED" };
                println!(
                    "[SECURITY] {} | {} | action={} resource={} | {}",
                    event.timestamp_ms,
                    event.component,
                    event.action,
                    event.resource,
                    status
                );
            }
        });

        Self { sender }
    }
}

impl SecurityAuditLogger for ConsoleSecurityAuditLogger {
    fn log_event(&self, event: SecurityEvent) {
        let _ = self.sender.send(event);
    }
}

/// Creates a security event for logging
pub fn create_security_event(
    component: ComponentId,
    action: &str,
    resource: &str,
    granted: bool,
) -> SecurityEvent {
    SecurityEvent {
        component,
        action: action.to_string(),
        resource: resource.to_string(),
        granted,
        timestamp_ms: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
    }
}
```

---

## airssys-osl Integration

```rust
// security/mod.rs - Bridge to airssys-osl

use airssys_osl::SecurityContext;

/// Bridge to airssys-osl SecurityContext
pub struct OslSecurityBridge {
    security_context: SecurityContext,
}

impl OslSecurityBridge {
    pub fn new(security_context: SecurityContext) -> Self {
        Self { security_context }
    }

    /// Check OSL-level permissions before capability check
    pub fn check_osl_permission(
        &self,
        principal: &str,
        resource: &str,
        action: &str,
    ) -> Result<(), SecurityError> {
        // Integrate with airssys-osl RBAC/ACL
        if self.security_context.is_permitted(principal, resource, action) {
            Ok(())
        } else {
            Err(SecurityError::PermissionDenied(format!(
                "OSL denied: {} cannot {} on {}",
                principal, action, resource
            )))
        }
    }
}
```

---

## History

| Date | Version | Change |
|------|---------|--------|
| 2026-01-05 | 1.0 | Initial security module design |

---

**This ADR defines the security module structure for Phase 4 of the rebuild.**
