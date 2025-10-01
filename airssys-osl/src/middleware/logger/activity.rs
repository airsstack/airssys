//! Activity logging types and core trait definition.
//!
//! This module defines the core types for structured activity logging,
//! including the ActivityLog structure and ActivityLogger trait.

// Layer 1: Standard library imports
// (imports will be added in Phase 2)

// Layer 2: Third-party crate imports
// (imports will be added in Phase 2)

// Layer 3: Internal module imports
// (imports will be added in Phase 2)

/// Structured log entry representing a single OS operation activity.
///
/// This structure contains comprehensive metadata about an operation
/// execution, suitable for audit trails and debugging.
///
/// Implementation will be added in Phase 2.
#[derive(Debug)]
pub struct ActivityLog {
    // TODO: Implement in Phase 2
}

/// Core trait for pluggable activity logging destinations.
///
/// Implementations can target different output destinations (console, file,
/// tracing, external systems) while maintaining a consistent interface.
///
/// Implementation will be added in Phase 2.
pub trait ActivityLogger: std::fmt::Debug + Send + Sync + 'static {
    // TODO: Implement in Phase 2
}
