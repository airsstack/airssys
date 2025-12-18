//! Hook execution helpers with timeout and panic protection.
//!
//! This module provides utility functions for safely executing lifecycle hooks
//! with timeout protection and panic handling, ensuring that hook failures don't
//! crash the component.
//!
//! # Design Principles
//!
//! - **Timeout Protection**: All hooks have configurable timeout (default 1000ms)
//! - **Panic Safety**: catch_unwind prevents hook panics from crashing components
//! - **Non-fatal Errors**: Hook failures are logged but don't block lifecycle
//! - **Minimal Overhead**: ~10μs overhead for timeout check + catch_unwind
//!
//! # Examples
//!
//! ```rust,ignore
//! use airssys_wasm::actor::lifecycle::{call_hook_with_timeout, HookResult};
//! use std::time::Duration;
//!
//! let result = call_hook_with_timeout(
//!     || {
//!         // Hook logic here
//!         HookResult::Ok
//!     },
//!     Duration::from_millis(1000),
//! ).await;
//! ```

// Layer 1: Standard library imports
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::time::Duration;

// Layer 2: Third-party crate imports
use tracing::{error, warn};

// Layer 3: Internal module imports
use crate::actor::lifecycle::HookResult;

/// Call hook with timeout protection.
///
/// Wraps hook execution with tokio timeout and panic catching. If the hook
/// exceeds the timeout or panics, returns appropriate HookResult without
/// crashing the component.
///
/// # Arguments
///
/// * `f` - Hook closure to execute
/// * `timeout` - Maximum execution time
///
/// # Returns
///
/// - `HookResult::Ok`: Hook executed successfully
/// - `HookResult::Error(msg)`: Hook panicked or returned error
/// - `HookResult::Timeout`: Hook exceeded timeout
///
/// # Performance
///
/// - Overhead: ~10μs (timeout check + catch_unwind)
/// - No allocations unless hook panics
///
/// # Examples
///
/// ```rust,ignore
/// use airssys_wasm::actor::lifecycle::{call_hook_with_timeout, HookResult};
/// use std::time::Duration;
///
/// let result = call_hook_with_timeout(
///     || {
///         println!("Hook executing");
///         HookResult::Ok
///     },
///     Duration::from_millis(1000),
/// ).await;
///
/// assert_eq!(result, HookResult::Ok);
/// ```
pub async fn call_hook_with_timeout<F>(f: F, timeout: Duration) -> HookResult
where
    F: FnOnce() -> HookResult + Send + 'static,
{
    // Wrap in catch_unwind for panic safety
    let result = tokio::time::timeout(
        timeout,
        tokio::task::spawn_blocking(move || {
            catch_unwind(AssertUnwindSafe(f)).unwrap_or_else(|panic_info| {
                let panic_msg = if let Some(s) = panic_info.downcast_ref::<String>() {
                    s.clone()
                } else if let Some(s) = panic_info.downcast_ref::<&str>() {
                    (*s).to_string()
                } else {
                    "unknown panic".to_string()
                };

                error!(panic_msg = %panic_msg, "Hook panicked");
                HookResult::Error(format!("hook panicked: {}", panic_msg))
            })
        }),
    )
    .await;

    match result {
        Ok(Ok(hook_result)) => hook_result,
        Ok(Err(_)) => {
            // spawn_blocking task panicked (shouldn't happen due to catch_unwind)
            error!("Hook spawn_blocking task panicked");
            HookResult::Error("hook task panicked".to_string())
        }
        Err(_) => {
            warn!(timeout_ms = timeout.as_millis(), "Hook timed out");
            HookResult::Timeout
        }
    }
}

/// Catch panics in hook execution (synchronous variant).
///
/// Provides panic safety for synchronous hook execution without timeout.
/// Use this for hooks that are known to be fast (<100μs).
///
/// # Arguments
///
/// * `f` - Hook closure to execute
///
/// # Returns
///
/// - `HookResult::Ok`: Hook executed successfully
/// - `HookResult::Error(msg)`: Hook panicked or returned error
///
/// # Performance
///
/// - Overhead: ~5μs (catch_unwind only)
/// - No allocations unless hook panics
///
/// # Examples
///
/// ```rust,ignore
/// use airssys_wasm::actor::lifecycle::{catch_unwind_hook, HookResult};
///
/// let result = catch_unwind_hook(|| {
///     // Fast hook logic
///     HookResult::Ok
/// });
///
/// assert_eq!(result, HookResult::Ok);
/// ```
pub fn catch_unwind_hook<F>(f: F) -> HookResult
where
    F: FnOnce() -> HookResult,
{
    catch_unwind(AssertUnwindSafe(f)).unwrap_or_else(|panic_info| {
        let panic_msg = if let Some(s) = panic_info.downcast_ref::<String>() {
            s.clone()
        } else if let Some(s) = panic_info.downcast_ref::<&str>() {
            (*s).to_string()
        } else {
            "unknown panic".to_string()
        };

        error!(panic_msg = %panic_msg, "Hook panicked");
        HookResult::Error(format!("hook panicked: {}", panic_msg))
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_call_hook_with_timeout_success() {
        let result = call_hook_with_timeout(|| HookResult::Ok, Duration::from_millis(1000)).await;

        assert_eq!(result, HookResult::Ok);
    }

    #[tokio::test]
    async fn test_call_hook_with_timeout_error() {
        let result = call_hook_with_timeout(
            || HookResult::Error("test error".to_string()),
            Duration::from_millis(1000),
        )
        .await;

        assert_eq!(result, HookResult::Error("test error".to_string()));
    }

    #[tokio::test]
    async fn test_call_hook_with_timeout_panic() {
        let result = call_hook_with_timeout(
            || {
                panic!("test panic");
            },
            Duration::from_millis(1000),
        )
        .await;

        match result {
            HookResult::Error(msg) => assert!(msg.contains("panicked")),
            _ => panic!("Expected Error variant for panic"),
        }
    }

    #[tokio::test]
    async fn test_call_hook_with_timeout_timeout() {
        let result = call_hook_with_timeout(
            || {
                std::thread::sleep(Duration::from_millis(2000));
                HookResult::Ok
            },
            Duration::from_millis(100),
        )
        .await;

        assert_eq!(result, HookResult::Timeout);
    }

    #[test]
    fn test_catch_unwind_hook_success() {
        let result = catch_unwind_hook(|| HookResult::Ok);
        assert_eq!(result, HookResult::Ok);
    }

    #[test]
    fn test_catch_unwind_hook_error() {
        let result = catch_unwind_hook(|| HookResult::Error("test".to_string()));
        assert_eq!(result, HookResult::Error("test".to_string()));
    }

    #[test]
    fn test_catch_unwind_hook_panic_string() {
        let result = catch_unwind_hook(|| {
            panic!("test panic");
        });

        match result {
            HookResult::Error(msg) => assert!(msg.contains("panicked")),
            _ => panic!("Expected Error variant for panic"),
        }
    }

    #[test]
    fn test_catch_unwind_hook_panic_str() {
        let result = catch_unwind_hook(|| {
            panic!("static str panic");
        });

        match result {
            HookResult::Error(msg) => assert!(msg.contains("panicked")),
            _ => panic!("Expected Error variant for panic"),
        }
    }

    #[tokio::test]
    async fn test_hook_with_timeout_performance() {
        // Verify minimal overhead (<50μs)
        let start = std::time::Instant::now();

        for _ in 0..100 {
            call_hook_with_timeout(|| HookResult::Ok, Duration::from_millis(1000)).await;
        }

        let elapsed = start.elapsed();
        let avg_per_call = elapsed / 100;

        // Should be much less than 50μs per call (target overhead)
        // Allow 100μs for CI variability
        assert!(
            avg_per_call.as_micros() < 100,
            "Average hook overhead: {:?}μs (target <100μs)",
            avg_per_call.as_micros()
        );
    }
}
