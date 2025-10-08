//! Network executor module.

// Private submodules - implementation details
mod connect;
mod executor;
mod listen;
mod socket;

// Public re-exports
pub use executor::NetworkExecutor;
