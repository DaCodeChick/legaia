//! Game-specific asset format utilities
//!
//! This module re-exports PlayStation format parsers from `psxutils`
//! and may add Legaia-specific format wrappers in the future.

// Re-export PSX format parsers for convenience
pub use psxutils::{Tim, Vab, Vag};

// Note: Users can also import directly from psxutils if they prefer:
// use psxutils::{Tim, Vab, Vag};
