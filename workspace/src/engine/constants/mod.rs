pub mod defaults;
pub mod emoji;
pub mod env;
pub mod labels;
pub mod paths;

pub use defaults::*;
pub use emoji::*;
pub use env::*;
pub use labels::*;
pub use paths::*;

pub const APP_NAME: &str = "cadman";
pub const APP_ABOUT: &str = "👷‍♂️ Cadman";
pub const BIN_NAME: &str = APP_NAME;

// Project-level constants
pub const PROJECT_CONFIG_FILE_NAME: &str = "cadman.toml";
pub const PROJECT_CONFIG_FILE_NAME_YAML: &str = "cadman.yaml";
pub const PROJECT_CONFIG_FILE_NAME_YML: &str = "cadman.yml";

// System Constants
pub const CONFIG_DIR_NAME: &str = APP_NAME;
pub const CACHE_DIR_NAME: &str = APP_NAME;
pub const STATE_DIR_NAME: &str = APP_NAME;
pub const LOG_DIR_NAME: &str = APP_NAME;

pub const CONFIG_FILE_NAME: &str = "config.toml";
pub const STATE_FILE_NAME: &str = "state.json";
pub const APPS_FILE_NAME: &str = "registry.toml";
pub const LOG_FILE_NAME: &str = "cadman.log";

pub const CADMAN_USER_NAME: &str = APP_NAME;
pub const CADMAN_GROUP_NAME: &str = APP_NAME;
