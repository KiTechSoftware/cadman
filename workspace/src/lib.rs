//! Minimal `cadman` crate stubs for CLI/core/engine layers.

pub mod cli;
pub mod core;
pub mod engine;

// Re-export errors for CLI mapping if needed
pub use engine::errors;

/// Single global env-var lock used by all test modules that mutate XDG_STATE_HOME or
/// similar environment variables.  One shared lock prevents cross-module races when
/// cargo runs tests in parallel.
#[cfg(test)]
pub static TEST_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
