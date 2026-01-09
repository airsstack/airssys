# WASM-TASK-023: Implementation Plans

## Plan References
- **ADR-WASM-028:** Core Module Structure (primary specification)
- **ADR-WASM-025:** Clean-Slate Rebuild Architecture
- **ADR-WASM-026:** Implementation Roadmap (Phase 3 - lines 109-123)
- **KNOWLEDGE-WASM-037:** Rebuild Architecture Clean-Slate
- **PROJECTS_STANDARD.md §4.3:** Module Architecture Patterns

## Target Structure Reference

```
core/config/
├── mod.rs           # Module declarations only
└── component.rs     # ComponentConfig + ConfigValidationError
```

## Key Design Decisions

### Private Fields + Getters + Builder Pattern
- **Private fields** - encapsulation, immutable after construction
- **Concrete getters** - access to internal state
- **Concrete builder methods** - fluent construction API

### Simple Validation
Validate that values are sensible (not zero/invalid), not arbitrary minimums:
- `max_memory_bytes`: must be > 0
- `max_execution_time_ms`: must be > 0
- `max_fuel`: if set, must be > 0
- `storage_namespace`: if set, must follow format rules

---

## Implementation Actions

### Action 1: Create core/config/component.rs

**Implementation:**
```rust
//! Component configuration types.

use thiserror::Error;
use crate::core::component::id::ComponentId;

// =============================================================================
// Constants
// =============================================================================

/// Default maximum memory in bytes (64MB).
pub const DEFAULT_MAX_MEMORY_BYTES: u64 = 64 * 1024 * 1024;

/// Default maximum execution time in milliseconds (30 seconds).
pub const DEFAULT_MAX_EXECUTION_TIME_MS: u64 = 30_000;

// =============================================================================
// Validation Error
// =============================================================================

/// Configuration validation error.
#[derive(Debug, Clone, Error)]
pub enum ConfigValidationError {
    /// Memory limit cannot be zero.
    #[error("max_memory_bytes cannot be zero")]
    MemoryIsZero,

    /// Execution time cannot be zero.
    #[error("max_execution_time_ms cannot be zero")]
    ExecutionTimeIsZero,

    /// Fuel limit cannot be zero (if set).
    #[error("max_fuel cannot be zero when set")]
    FuelIsZero,

    /// Storage namespace is invalid.
    #[error("Storage namespace is invalid: {reason}")]
    InvalidStorageNamespace {
        /// The invalid namespace value.
        value: String,
        /// Reason why it's invalid.
        reason: String,
    },
}

// =============================================================================
// ComponentConfig
// =============================================================================

/// Configuration for component instantiation.
///
/// Fields are private for encapsulation. Use getters to read values
/// and builder methods to construct with custom values.
///
/// # Validation
///
/// Use [`validate()`](Self::validate) to check configuration before use.
#[derive(Debug, Clone)]
pub struct ComponentConfig {
    id: ComponentId,
    max_memory_bytes: u64,
    max_execution_time_ms: u64,
    max_fuel: Option<u64>,
    storage_namespace: Option<String>,
    debug_mode: bool,
}

impl Default for ComponentConfig {
    fn default() -> Self {
        Self {
            id: ComponentId::new("default", "component", "000"),
            max_memory_bytes: DEFAULT_MAX_MEMORY_BYTES,
            max_execution_time_ms: DEFAULT_MAX_EXECUTION_TIME_MS,
            max_fuel: None,
            storage_namespace: None,
            debug_mode: false,
        }
    }
}

impl ComponentConfig {
    // =========================================================================
    // Constructor
    // =========================================================================

    /// Create a new ComponentConfig with the given ComponentId.
    pub fn new(id: ComponentId) -> Self {
        Self { id, ..Default::default() }
    }

    // =========================================================================
    // Builder Methods
    // =========================================================================

    /// Set maximum memory limit in bytes.
    pub fn with_max_memory(mut self, bytes: u64) -> Self {
        self.max_memory_bytes = bytes;
        self
    }

    /// Set maximum execution time in milliseconds.
    pub fn with_max_execution_time(mut self, ms: u64) -> Self {
        self.max_execution_time_ms = ms;
        self
    }

    /// Set fuel limit for deterministic execution.
    pub fn with_fuel_limit(mut self, fuel: u64) -> Self {
        self.max_fuel = Some(fuel);
        self
    }

    /// Set storage namespace for component isolation.
    pub fn with_storage_namespace(mut self, namespace: impl Into<String>) -> Self {
        self.storage_namespace = Some(namespace.into());
        self
    }

    /// Enable or disable debug mode.
    pub fn with_debug_mode(mut self, enabled: bool) -> Self {
        self.debug_mode = enabled;
        self
    }

    // =========================================================================
    // Validation
    // =========================================================================

    /// Validates the configuration.
    ///
    /// # Validation Rules
    ///
    /// - `max_memory_bytes` must be > 0
    /// - `max_execution_time_ms` must be > 0
    /// - `max_fuel` (if set) must be > 0
    /// - `storage_namespace` (if set) must not be empty or contain `/` or `\`
    pub fn validate(&self) -> Result<(), ConfigValidationError> {
        if self.max_memory_bytes == 0 {
            return Err(ConfigValidationError::MemoryIsZero);
        }

        if self.max_execution_time_ms == 0 {
            return Err(ConfigValidationError::ExecutionTimeIsZero);
        }

        if let Some(fuel) = self.max_fuel {
            if fuel == 0 {
                return Err(ConfigValidationError::FuelIsZero);
            }
        }

        if let Some(ns) = &self.storage_namespace {
            if ns.is_empty() {
                return Err(ConfigValidationError::InvalidStorageNamespace {
                    value: ns.clone(),
                    reason: "namespace cannot be empty".to_string(),
                });
            }
            if ns.contains('/') || ns.contains('\\') {
                return Err(ConfigValidationError::InvalidStorageNamespace {
                    value: ns.clone(),
                    reason: "namespace cannot contain '/' or '\\'".to_string(),
                });
            }
        }

        Ok(())
    }

    // =========================================================================
    // Getters
    // =========================================================================

    pub fn id(&self) -> &ComponentId { &self.id }
    pub fn max_memory_bytes(&self) -> u64 { self.max_memory_bytes }
    pub fn max_execution_time_ms(&self) -> u64 { self.max_execution_time_ms }
    pub fn max_fuel(&self) -> Option<u64> { self.max_fuel }
    pub fn storage_namespace(&self) -> Option<&str> { self.storage_namespace.as_deref() }
    pub fn debug_mode(&self) -> bool { self.debug_mode }
}
```

**Unit Tests:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_default_passes() {
        let config = ComponentConfig::new(ComponentId::new("a", "b", "c"));
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_memory_zero() {
        let config = ComponentConfig::new(ComponentId::new("a", "b", "c"))
            .with_max_memory(0);
        assert!(matches!(config.validate(), Err(ConfigValidationError::MemoryIsZero)));
    }

    #[test]
    fn test_validate_execution_time_zero() {
        let config = ComponentConfig::new(ComponentId::new("a", "b", "c"))
            .with_max_execution_time(0);
        assert!(matches!(config.validate(), Err(ConfigValidationError::ExecutionTimeIsZero)));
    }

    #[test]
    fn test_validate_fuel_zero() {
        let config = ComponentConfig::new(ComponentId::new("a", "b", "c"))
            .with_fuel_limit(0);
        assert!(matches!(config.validate(), Err(ConfigValidationError::FuelIsZero)));
    }

    #[test]
    fn test_validate_empty_namespace() {
        let config = ComponentConfig::new(ComponentId::new("a", "b", "c"))
            .with_storage_namespace("");
        assert!(matches!(config.validate(), Err(ConfigValidationError::InvalidStorageNamespace { .. })));
    }

    #[test]
    fn test_validate_namespace_with_slash() {
        let config = ComponentConfig::new(ComponentId::new("a", "b", "c"))
            .with_storage_namespace("invalid/ns");
        assert!(matches!(config.validate(), Err(ConfigValidationError::InvalidStorageNamespace { .. })));
    }

    #[test]
    fn test_validate_valid_namespace() {
        let config = ComponentConfig::new(ComponentId::new("a", "b", "c"))
            .with_storage_namespace("valid-namespace_123");
        assert!(config.validate().is_ok());
    }
}
```

---

### Action 2: Create core/config/mod.rs

```rust
//! Configuration types for airssys-wasm.

pub mod component;
```

---

### Action 3: Update core/mod.rs

```rust
// Add to core/mod.rs
pub mod config;
```

---

## Validation Summary

| Field | Validation Rule |
|:---|:---|
| `max_memory_bytes` | Must be > 0 |
| `max_execution_time_ms` | Must be > 0 |
| `max_fuel` | If set, must be > 0 |
| `storage_namespace` | If set, not empty, no `/` or `\` |

```
ConfigValidationError
├── MemoryIsZero
├── ExecutionTimeIsZero
├── FuelIsZero
└── InvalidStorageNamespace { value, reason }
```
