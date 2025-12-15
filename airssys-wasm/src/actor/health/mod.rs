//! Health monitoring and restart triggering system.
//!
//! This module provides health check monitoring, decision-making for
//! component restarts based on health status, and health-triggered
//! restart configuration.
//!
//! # Architecture
//!
//! The health monitoring system:
//! - Tracks consecutive health check failures
//! - Makes restart decisions based on degradation patterns
//! - Integrates with supervision infrastructure
//!
//! # Module Organization
//!
//! - `health_monitor` - Health check evaluation and decision logic
//! - `health_restart` - Health-triggered restart configuration

// Module declarations
pub mod health_monitor;
pub mod health_restart;

// Public re-exports
#[doc(inline)]
pub use health_monitor::{HealthDecision, HealthMonitor, HealthStatus as MonitorHealthStatus};
#[doc(inline)]
pub use health_restart::HealthRestartConfig;
