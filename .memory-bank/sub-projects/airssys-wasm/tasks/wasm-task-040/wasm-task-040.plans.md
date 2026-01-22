# WASM-TASK-040: Implementation Plans

## References
- ADR-WASM-031 (Component & Messaging Module Design)
- ADR-WASM-026 (Implementation Roadmap)
- ADR-WASM-025 (Clean Slate Architecture)
- ADR-WASM-023 (Module Boundary Enforcement)
- PROJECTS_STANDARD.md (§2.1, §2.2, §4.3)

---

## Actions

### Action 1: Implement SupervisorConfig

**Objective**: Create SupervisorConfig for component actor fault tolerance configuration.

**Detailed Steps**:

#### Step 1.1: Create `src/component/supervisor.rs`

```rust
//! Supervisor configuration for component actor fault tolerance.
//!
//! Provides restart policies and backoff strategies that map to airssys-rt SupervisorNode.

// Layer 1: Standard library imports
use std::time::Duration;

// Layer 2: Third-party crate imports
use airssys_rt::SupervisorNode;

// Layer 3: Internal module imports
// (none needed for this module)

/// Supervisor configuration for component actors
///
/// Defines how component actors should be restarted on failure.
#[derive(Debug, Clone)]
pub struct SupervisorConfig {
    /// Maximum number of restarts within the restart window
    pub max_restarts: u32,
    /// Time window for restart counting
    pub restart_window: Duration,
    /// Backoff strategy between restarts
    pub backoff_strategy: BackoffStrategy,
}

/// Backoff strategy for restart delays
#[derive(Debug, Clone)]
pub enum BackoffStrategy {
    /// Fixed delay between restarts
    Fixed(Duration),
    /// Exponential backoff with base and max delay
    Exponential {
        /// Base delay for first restart
        base: Duration,
        /// Maximum delay cap
        max: Duration,
    },
}

impl Default for SupervisorConfig {
    fn default() -> Self {
        Self {
            max_restarts: 3,
            restart_window: Duration::from_secs(60),
            backoff_strategy: BackoffStrategy::Exponential {
                base: Duration::from_millis(100),
                max: Duration::from_secs(30),
            },
        }
    }
}

impl SupervisorConfig {
    /// Create a new supervisor configuration
    ///
    /// # Arguments
    /// * `max_restarts` - Maximum restarts within the window
    /// * `restart_window` - Time window for restart counting
    ///
    /// # Example
    /// ```ignore
    /// use std::time::Duration;
    /// use airssys_wasm::component::SupervisorConfig;
    ///
    /// let config = SupervisorConfig::new(5, Duration::from_secs(120));
    /// ```
    pub fn new(max_restarts: u32, restart_window: Duration) -> Self {
        Self {
            max_restarts,
            restart_window,
            backoff_strategy: BackoffStrategy::default(),
        }
    }

    /// Set the backoff strategy
    ///
    /// # Arguments
    /// * `strategy` - The backoff strategy to use
    ///
    /// # Example
    /// ```ignore
    /// use std::time::Duration;
    /// use airssys_wasm::component::{SupervisorConfig, BackoffStrategy};
    ///
    /// let config = SupervisorConfig::new(3, Duration::from_secs(60))
    ///     .with_backoff(BackoffStrategy::Fixed(Duration::from_secs(5)));
    /// ```
    pub fn with_backoff(mut self, strategy: BackoffStrategy) -> Self {
        self.backoff_strategy = strategy;
        self
    }

    /// Convert to airssys-rt SupervisorNode
    ///
    /// Maps our configuration to the airssys-rt supervisor format.
    ///
    /// # Returns
    /// SupervisorNode configured with these settings
    pub fn to_supervisor_node(&self) -> SupervisorNode {
        // Map to airssys-rt supervisor configuration
        // Note: Actual implementation depends on airssys-rt API
        todo!("Map to airssys-rt SupervisorNode")
    }
}

impl Default for BackoffStrategy {
    fn default() -> Self {
        Self::Exponential {
            base: Duration::from_millis(100),
            max: Duration::from_secs(30),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SupervisorConfig::default();

        assert_eq!(config.max_restarts, 3);
        assert_eq!(config.restart_window, Duration::from_secs(60));
        
        match config.backoff_strategy {
            BackoffStrategy::Exponential { base, max } => {
                assert_eq!(base, Duration::from_millis(100));
                assert_eq!(max, Duration::from_secs(30));
            }
            _ => panic!("Expected exponential backoff"),
        }
    }

    #[test]
    fn test_new_config() {
        let config = SupervisorConfig::new(5, Duration::from_secs(120));

        assert_eq!(config.max_restarts, 5);
        assert_eq!(config.restart_window, Duration::from_secs(120));
        
        // Default backoff strategy
        match config.backoff_strategy {
            BackoffStrategy::Exponential { .. } => {}
            _ => panic!("Expected exponential backoff"),
        }
    }

    #[test]
    fn test_with_backoff_fixed() {
        let config = SupervisorConfig::new(3, Duration::from_secs(60))
            .with_backoff(BackoffStrategy::Fixed(Duration::from_secs(5)));

        match config.backoff_strategy {
            BackoffStrategy::Fixed(duration) => {
                assert_eq!(duration, Duration::from_secs(5));
            }
            _ => panic!("Expected fixed backoff"),
        }
    }

    #[test]
    fn test_with_backoff_exponential() {
        let config = SupervisorConfig::new(3, Duration::from_secs(60))
            .with_backoff(BackoffStrategy::Exponential {
                base: Duration::from_millis(200),
                max: Duration::from_secs(60),
            });

        match config.backoff_strategy {
            BackoffStrategy::Exponential { base, max } => {
                assert_eq!(base, Duration::from_millis(200));
                assert_eq!(max, Duration::from_secs(60));
            }
            _ => panic!("Expected exponential backoff"),
        }
    }

    #[test]
    fn test_backoff_strategy_default() {
        let strategy = BackoffStrategy::default();

        match strategy {
            BackoffStrategy::Exponential { base, max } => {
                assert_eq!(base, Duration::from_millis(100));
                assert_eq!(max, Duration::from_secs(30));
            }
            _ => panic!("Expected exponential backoff"),
        }
    }

    #[test]
    fn test_config_clone() {
        let config = SupervisorConfig::new(5, Duration::from_secs(120));
        let cloned = config.clone();

        assert_eq!(cloned.max_restarts, config.max_restarts);
        assert_eq!(cloned.restart_window, config.restart_window);
    }

    #[test]
    fn test_config_debug() {
        let config = SupervisorConfig::default();
        let debug_str = format!("{:?}", config);

        assert!(debug_str.contains("SupervisorConfig"));
        assert!(debug_str.contains("max_restarts"));
        assert!(debug_str.contains("restart_window"));
    }

    #[test]
    fn test_backoff_fixed_clone() {
        let strategy = BackoffStrategy::Fixed(Duration::from_secs(10));
        let cloned = strategy.clone();

        match cloned {
            BackoffStrategy::Fixed(duration) => {
                assert_eq!(duration, Duration::from_secs(10));
            }
            _ => panic!("Expected fixed backoff"),
        }
    }

    #[test]
    fn test_backoff_exponential_clone() {
        let strategy = BackoffStrategy::Exponential {
            base: Duration::from_millis(50),
            max: Duration::from_secs(120),
        };
        let cloned = strategy.clone();

        match cloned {
            BackoffStrategy::Exponential { base, max } => {
                assert_eq!(base, Duration::from_millis(50));
                assert_eq!(max, Duration::from_secs(120));
            }
            _ => panic!("Expected exponential backoff"),
        }
    }

    #[test]
    fn test_production_ready_defaults() {
        let config = SupervisorConfig::default();

        // Verify defaults are production-ready
        assert!(config.max_restarts >= 3, "Should allow multiple restarts");
        assert!(config.restart_window >= Duration::from_secs(30), "Window should be reasonable");
        
        match config.backoff_strategy {
            BackoffStrategy::Exponential { base, max } => {
                assert!(base >= Duration::from_millis(50), "Base delay should be reasonable");
                assert!(max <= Duration::from_secs(60), "Max delay should cap reasonably");
            }
            BackoffStrategy::Fixed(duration) => {
                assert!(duration >= Duration::from_millis(100), "Fixed delay should be reasonable");
            }
        }
    }

    #[test]
    fn test_builder_pattern_chaining() {
        let config = SupervisorConfig::new(10, Duration::from_secs(300))
            .with_backoff(BackoffStrategy::Fixed(Duration::from_secs(2)));

        assert_eq!(config.max_restarts, 10);
        assert_eq!(config.restart_window, Duration::from_secs(300));
        
        match config.backoff_strategy {
            BackoffStrategy::Fixed(d) => assert_eq!(d, Duration::from_secs(2)),
            _ => panic!("Expected fixed backoff"),
        }
    }
}
```

#### Step 1.2: Update `src/component/mod.rs`

```rust
//! Component module - Actor system integration for WASM components
//!
//! This module integrates WASM components with the airssys-rt actor system.

pub mod registry;
pub mod supervisor;
pub mod wrapper;

pub use registry::ComponentRegistry;
pub use supervisor::{BackoffStrategy, SupervisorConfig};
pub use wrapper::{ComponentActorMessage, ComponentWrapper};
```

**Deliverables**:
- `src/component/supervisor.rs` with SupervisorConfig struct
- BackoffStrategy enum (Fixed, Exponential)
- Default trait implementations
- Builder pattern (new, with_backoff)
- to_supervisor_node() method
- Comprehensive unit tests (12+ tests)
- Module export in `src/component/mod.rs`

**Constraints**:
- Must not import from `runtime/`, `security/`, or `system/`
- Must follow §2.1 3-Layer imports
- Must follow §2.2 No FQN in type annotations
- Default configuration must be production-ready

---

## Verification Section

### Automated Tests
```bash
# Unit tests for supervisor module
cargo test -p airssys-wasm --lib -- component::supervisor

# All component module tests
cargo test -p airssys-wasm --lib -- component

# Build verification
cargo build -p airssys-wasm

# Clippy
cargo clippy -p airssys-wasm --lib -- -D warnings
```

### Architecture Compliance
```bash
# Verify no forbidden imports in supervisor.rs
grep -rn "use crate::runtime" src/component/supervisor.rs  # Should be empty
grep -rn "use crate::security" src/component/supervisor.rs  # Should be empty
grep -rn "use crate::system" src/component/supervisor.rs    # Should be empty

# Verify no FQN in type annotations
grep -rn "std::" src/component/supervisor.rs | grep -v "^.*use " | grep "::"  # Should be empty
```

---

## Success Criteria
- [ ] `src/component/supervisor.rs` exists and compiles
- [ ] SupervisorConfig struct with all fields
- [ ] BackoffStrategy enum (Fixed, Exponential)
- [ ] Default trait implementations (production-ready)
- [ ] Builder pattern (new, with_backoff)
- [ ] to_supervisor_node() method signature
- [ ] Unit tests pass (12+ tests)
- [ ] Debug and Clone traits derived
- [ ] `cargo clippy -p airssys-wasm --lib -- -D warnings` passes
- [ ] No forbidden imports (architecture compliance)
- [ ] §2.1 3-Layer imports verified
- [ ] §2.2 No FQN verified
