//! # Security Module
//!
//! Security-related types and logic: capabilities, policies, validation.
//!
//! Security must be independent so it can be applied uniformly
//! across all other layers.

pub mod audit;
pub mod capability;
pub mod policy;
