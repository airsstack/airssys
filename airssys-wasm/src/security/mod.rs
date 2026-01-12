//! Security module for capability-based access control.
//!
//! This module is **Layer 2A** of the architecture and provides
//! integration with airssys-osl's security infrastructure.

pub mod audit;
pub mod capability;
pub mod osl;
pub mod policy;
