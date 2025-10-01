//! Concrete implementations of the ActivityLogger trait.
//!
//! This module provides ready-to-use logger implementations for different
//! output destinations and formats.

// Layer 1: Standard library imports
// (none for this module)

// Layer 2: Third-party crate imports
// (none for this module)

// Layer 3: Internal module imports
// (none for this module)

// Public logger implementations
pub use console::ConsoleActivityLogger;
pub use file::FileActivityLogger;
pub use tracing::TracingActivityLogger;

// Internal modules
mod console;
mod file;
mod tracing;