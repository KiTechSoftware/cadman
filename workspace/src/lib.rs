//! Minimal `cadman` crate stubs for CLI/core/engine layers.

pub mod cli;
pub mod core;
pub mod engine;

// Re-export errors for CLI mapping if needed
pub use engine::errors;
