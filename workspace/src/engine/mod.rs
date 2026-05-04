pub mod capabilities;
pub mod config;
pub mod constants;
pub mod errors;
pub mod models;
pub mod registry;
pub mod state;
pub mod system;

pub use errors::*;

// Engine-level public API (stubs)
pub fn version() -> &'static str {
    "0.1.0"
}
