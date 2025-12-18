//! Lifecycle hooks for ComponentActor extensibility.
//!
//! This module provides the `LifecycleHooks` trait which enables components to hook
//! into key lifecycle events (pre/post-start/stop, message handling, errors, restarts)
//! without modifying framework code.
//!
//! # Design Principles
//!
//! - **Opt-in Customization**: Default no-op implementations (zero overhead if unused)
//! - **Timeout Protection**: All hooks have configurable timeout (default 1000ms)
//! - **Panic Safety**: Hooks are panic-safe via catch_unwind
//! - **Non-blocking**: Hook errors don't prevent lifecycle progression
//!
//! # Performance
//!
//! - Hook overhead: <50μs per hook (including timeout check)
//! - No-op hooks: ~1-2μs (trait object indirection only)
//!
//! # Examples
//!
//! ```rust,ignore
//! use airssys_wasm::actor::lifecycle::{LifecycleHooks, LifecycleContext, HookResult};
//!
//! struct LoggingHooks;
//!
//! impl LifecycleHooks for LoggingHooks {
//!     fn pre_start(&mut self, ctx: &LifecycleContext) -> HookResult {
//!         println!("Component {} starting...", ctx.component_id.as_str());
//!         HookResult::Ok
//!     }
//!     
//!     fn post_start(&mut self, ctx: &LifecycleContext) -> HookResult {
//!         println!("Component {} started successfully", ctx.component_id.as_str());
//!         HookResult::Ok
//!     }
//! }
//! ```

// Layer 1: Standard library imports
// (none)

// Layer 2: Third-party crate imports
use airssys_rt::ActorAddress;
use chrono::{DateTime, Utc};

// Layer 3: Internal module imports
use crate::actor::ComponentMessage;
use crate::core::{ComponentId, WasmError};

/// Context passed to lifecycle hooks.
///
/// LifecycleContext provides contextual information about the component and
/// the current lifecycle event to hook implementations.
///
/// # Fields
///
/// - `component_id`: Unique identifier for the component
/// - `actor_address`: Actor system address (for sending messages)
/// - `timestamp`: When the lifecycle event occurred
///
/// # Examples
///
/// ```rust,ignore
/// use airssys_wasm::actor::lifecycle::LifecycleContext;
/// use airssys_wasm::core::ComponentId;
/// use airssys_rt::actor::ActorAddress;
///
/// let ctx = LifecycleContext::new(
///     ComponentId::new("my-component"),
///     ActorAddress::new(),
/// );
/// ```
#[derive(Clone)]
pub struct LifecycleContext {
    /// Component unique identifier
    pub component_id: ComponentId,

    /// Actor system address
    pub actor_address: ActorAddress,

    /// Event timestamp
    pub timestamp: DateTime<Utc>,
}

impl LifecycleContext {
    /// Create a new LifecycleContext.
    ///
    /// # Arguments
    ///
    /// * `component_id` - Component unique identifier
    /// * `actor_address` - Actor system address
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let ctx = LifecycleContext::new(component_id, actor_address);
    /// ```
    pub fn new(component_id: ComponentId, actor_address: ActorAddress) -> Self {
        Self {
            component_id,
            actor_address,
            timestamp: Utc::now(),
        }
    }
}

/// Hook invocation result.
///
/// HookResult allows hooks to report success, errors, or timeouts without
/// panicking. Framework code logs errors but continues lifecycle progression.
///
/// # Variants
///
/// - `Ok`: Hook executed successfully
/// - `Error(String)`: Hook reported an error (non-fatal)
/// - `Timeout`: Hook execution exceeded timeout limit
///
/// # Examples
///
/// ```rust
/// use airssys_wasm::actor::lifecycle::HookResult;
///
/// let result = HookResult::Ok;
/// assert!(matches!(result, HookResult::Ok));
///
/// let error = HookResult::Error("validation failed".to_string());
/// match error {
///     HookResult::Error(msg) => assert_eq!(msg, "validation failed"),
///     _ => panic!("Expected Error variant"),
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HookResult {
    /// Hook executed successfully
    Ok,

    /// Hook reported an error (non-fatal, logged)
    Error(String),

    /// Hook execution exceeded timeout
    Timeout,
}

/// Reason for component restart.
///
/// RestartReason provides context to on_restart hooks about why the supervisor
/// triggered a component restart.
///
/// # Variants
///
/// - `Crashed`: Component crashed (panic or fatal error)
/// - `HealthCheck`: Failed health check
/// - `Manual`: Manual restart requested
/// - `Timeout`: Component exceeded timeout limit
///
/// # Examples
///
/// ```rust
/// use airssys_wasm::actor::lifecycle::RestartReason;
///
/// let reason = RestartReason::Crashed("panic in message handler".to_string());
/// match reason {
///     RestartReason::Crashed(msg) => assert!(msg.contains("panic")),
///     _ => panic!("Expected Crashed variant"),
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RestartReason {
    /// Component crashed (panic or fatal error)
    Crashed(String),

    /// Failed health check
    HealthCheck(String),

    /// Manual restart requested
    Manual,

    /// Component exceeded timeout limit
    Timeout,
}

/// Lifecycle hooks for ComponentActor.
///
/// Hooks are called at key lifecycle events. Implement this trait to customize
/// component behavior without modifying framework code.
///
/// # Default Implementations
///
/// All methods have default no-op implementations (return `HookResult::Ok`),
/// allowing opt-in customization. Only override hooks you need.
///
/// # Timeout Protection
///
/// All hooks have 1000ms timeout protection (configurable). Hooks that exceed
/// timeout return `HookResult::Timeout` (non-fatal, logged).
///
/// # Panic Safety
///
/// Hooks are wrapped in `catch_unwind` to prevent panics from crashing the
/// component. Panics are converted to `HookResult::Error` (logged).
///
/// # Examples
///
/// ```rust,ignore
/// use airssys_wasm::actor::lifecycle::{LifecycleHooks, LifecycleContext, HookResult};
/// use airssys_wasm::actor::ComponentMessage;
/// use airssys_wasm::core::WasmError;
///
/// struct MyHooks {
///     start_count: u32,
/// }
///
/// impl LifecycleHooks for MyHooks {
///     fn pre_start(&mut self, ctx: &LifecycleContext) -> HookResult {
///         self.start_count += 1;
///         println!("Starting component (attempt {})", self.start_count);
///         HookResult::Ok
///     }
///     
///     fn on_error(&mut self, ctx: &LifecycleContext, error: &WasmError) -> HookResult {
///         eprintln!("Component error: {:?}", error);
///         HookResult::Ok
///     }
/// }
/// ```
pub trait LifecycleHooks: Send + Sync {
    /// Called before component starts.
    ///
    /// Use for: initial setup, configuration validation, resource allocation.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Lifecycle context with component_id, actor_address, timestamp
    ///
    /// # Returns
    ///
    /// - `HookResult::Ok`: Setup successful, proceed with start
    /// - `HookResult::Error(msg)`: Setup failed (logged, start continues)
    /// - `HookResult::Timeout`: Hook exceeded timeout (logged, start continues)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// fn pre_start(&mut self, ctx: &LifecycleContext) -> HookResult {
    ///     // Validate configuration
    ///     if !self.config.is_valid() {
    ///         return HookResult::Error("Invalid configuration".to_string());
    ///     }
    ///     HookResult::Ok
    /// }
    /// ```
    fn pre_start(&mut self, _ctx: &LifecycleContext) -> HookResult {
        HookResult::Ok
    }

    /// Called after component successfully starts.
    ///
    /// Use for: startup completion logging, dependency injection, initialization callbacks.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Lifecycle context
    ///
    /// # Returns
    ///
    /// Hook result (errors logged but don't affect started state)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// fn post_start(&mut self, ctx: &LifecycleContext) -> HookResult {
    ///     println!("Component {} started", ctx.component_id.as_str());
    ///     HookResult::Ok
    /// }
    /// ```
    fn post_start(&mut self, _ctx: &LifecycleContext) -> HookResult {
        HookResult::Ok
    }

    /// Called before component stops.
    ///
    /// Use for: cleanup logic, state saving, connection closing.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Lifecycle context
    ///
    /// # Returns
    ///
    /// Hook result (errors logged but don't prevent stop)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// fn pre_stop(&mut self, ctx: &LifecycleContext) -> HookResult {
    ///     // Save state before shutdown
    ///     self.save_state();
    ///     HookResult::Ok
    /// }
    /// ```
    fn pre_stop(&mut self, _ctx: &LifecycleContext) -> HookResult {
        HookResult::Ok
    }

    /// Called after component stops.
    ///
    /// Use for: final cleanup, resource release, stop confirmation.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Lifecycle context
    ///
    /// # Returns
    ///
    /// Hook result (errors logged)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// fn post_stop(&mut self, ctx: &LifecycleContext) -> HookResult {
    ///     println!("Component {} stopped", ctx.component_id.as_str());
    ///     HookResult::Ok
    /// }
    /// ```
    fn post_stop(&mut self, _ctx: &LifecycleContext) -> HookResult {
        HookResult::Ok
    }

    /// Called when message received (before routing to WASM).
    ///
    /// Use for: request logging, authentication, rate limiting pre-checks.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Lifecycle context
    /// * `msg` - Incoming ComponentMessage
    ///
    /// # Returns
    ///
    /// Hook result (errors logged but don't block message delivery)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// fn on_message_received(&mut self, ctx: &LifecycleContext, msg: &ComponentMessage) -> HookResult {
    ///     println!("Message received: {:?}", msg);
    ///     HookResult::Ok
    /// }
    /// ```
    fn on_message_received(
        &mut self,
        _ctx: &LifecycleContext,
        _msg: &ComponentMessage,
    ) -> HookResult {
        HookResult::Ok
    }

    /// Called when error occurs anywhere in component.
    ///
    /// Use for: error logging, metrics collection, error recovery.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Lifecycle context
    /// * `error` - WasmError that occurred
    ///
    /// # Returns
    ///
    /// Hook result (errors logged)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// fn on_error(&mut self, ctx: &LifecycleContext, error: &WasmError) -> HookResult {
    ///     eprintln!("Component error: {:?}", error);
    ///     self.error_count += 1;
    ///     HookResult::Ok
    /// }
    /// ```
    fn on_error(&mut self, _ctx: &LifecycleContext, _error: &WasmError) -> HookResult {
        HookResult::Ok
    }

    /// Called when supervisor triggers component restart.
    ///
    /// Use for: restart logging, state reset, notification hooks.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Lifecycle context
    /// * `reason` - Why the restart was triggered
    ///
    /// # Returns
    ///
    /// Hook result (errors logged)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// fn on_restart(&mut self, ctx: &LifecycleContext, reason: RestartReason) -> HookResult {
    ///     println!("Component restarting: {:?}", reason);
    ///     self.restart_count += 1;
    ///     HookResult::Ok
    /// }
    /// ```
    fn on_restart(&mut self, _ctx: &LifecycleContext, _reason: RestartReason) -> HookResult {
        HookResult::Ok
    }
}

/// Default no-op implementation (opt-in customization).
///
/// NoOpHooks provides a zero-overhead default implementation of LifecycleHooks
/// where all hooks immediately return `HookResult::Ok`. Components that don't
/// need lifecycle hooks can use this implementation (or rely on trait defaults).
///
/// # Examples
///
/// ```rust
/// use airssys_wasm::actor::lifecycle::{LifecycleHooks, NoOpHooks};
///
/// let hooks: Box<dyn LifecycleHooks> = Box::new(NoOpHooks);
/// // All hooks are no-ops (minimal overhead)
/// ```
#[derive(Debug)]
pub struct NoOpHooks;

impl LifecycleHooks for NoOpHooks {}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_lifecycle_context_creation() {
        let component_id = ComponentId::new("test-component");
        let actor_address = ActorAddress::anonymous();

        let ctx = LifecycleContext::new(component_id.clone(), actor_address.clone());

        assert_eq!(ctx.component_id, component_id);
        assert_eq!(ctx.actor_address, actor_address);
    }

    #[test]
    fn test_lifecycle_context_clone() {
        let component_id = ComponentId::new("test-component");
        let actor_address = ActorAddress::anonymous();
        let ctx = LifecycleContext::new(component_id.clone(), actor_address.clone());

        let cloned = ctx.clone();
        assert_eq!(cloned.component_id, component_id);
        assert_eq!(cloned.actor_address, actor_address);
    }

    #[test]
    fn test_hook_result_ok() {
        let result = HookResult::Ok;
        assert!(matches!(result, HookResult::Ok));
    }

    #[test]
    fn test_hook_result_error() {
        let result = HookResult::Error("test error".to_string());
        match result {
            HookResult::Error(msg) => assert_eq!(msg, "test error"),
            _ => panic!("Expected Error variant"),
        }
    }

    #[test]
    fn test_hook_result_timeout() {
        let result = HookResult::Timeout;
        assert!(matches!(result, HookResult::Timeout));
    }

    #[test]
    fn test_hook_result_equality() {
        assert_eq!(HookResult::Ok, HookResult::Ok);
        assert_eq!(
            HookResult::Error("test".to_string()),
            HookResult::Error("test".to_string())
        );
        assert_eq!(HookResult::Timeout, HookResult::Timeout);

        assert_ne!(HookResult::Ok, HookResult::Timeout);
        assert_ne!(
            HookResult::Error("a".to_string()),
            HookResult::Error("b".to_string())
        );
    }

    #[test]
    fn test_restart_reason_crashed() {
        let reason = RestartReason::Crashed("panic occurred".to_string());
        match reason {
            RestartReason::Crashed(msg) => assert_eq!(msg, "panic occurred"),
            _ => panic!("Expected Crashed variant"),
        }
    }

    #[test]
    fn test_restart_reason_health_check() {
        let reason = RestartReason::HealthCheck("unhealthy".to_string());
        match reason {
            RestartReason::HealthCheck(msg) => assert_eq!(msg, "unhealthy"),
            _ => panic!("Expected HealthCheck variant"),
        }
    }

    #[test]
    fn test_restart_reason_manual() {
        let reason = RestartReason::Manual;
        assert!(matches!(reason, RestartReason::Manual));
    }

    #[test]
    fn test_restart_reason_timeout() {
        let reason = RestartReason::Timeout;
        assert!(matches!(reason, RestartReason::Timeout));
    }

    #[test]
    fn test_noop_hooks_trait_implementation() {
        let mut hooks = NoOpHooks;
        let component_id = ComponentId::new("test");
        let actor_address = ActorAddress::anonymous();
        let ctx = LifecycleContext::new(component_id, actor_address);

        // All hooks should return Ok
        assert_eq!(hooks.pre_start(&ctx), HookResult::Ok);
        assert_eq!(hooks.post_start(&ctx), HookResult::Ok);
        assert_eq!(hooks.pre_stop(&ctx), HookResult::Ok);
        assert_eq!(hooks.post_stop(&ctx), HookResult::Ok);

        let msg = ComponentMessage::HealthCheck;
        assert_eq!(hooks.on_message_received(&ctx, &msg), HookResult::Ok);

        let error = WasmError::internal("test error");
        assert_eq!(hooks.on_error(&ctx, &error), HookResult::Ok);

        let reason = RestartReason::Manual;
        assert_eq!(hooks.on_restart(&ctx, reason), HookResult::Ok);
    }

    #[test]
    fn test_noop_hooks_boxed() {
        let hooks: Box<dyn LifecycleHooks> = Box::new(NoOpHooks);
        // Should compile and work as trait object
        drop(hooks);
    }
}
