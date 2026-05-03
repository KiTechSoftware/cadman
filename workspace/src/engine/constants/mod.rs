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
pub const BIN_NAME: &str = APP_NAME;

pub const CONFIG_FILE_NAME: &str = "config.toml";
pub const PROJECT_CONFIG_FILE_NAME: &str = "cadman.yaml";
pub const PROJECT_CONFIG_FILE_NAME_HIDDEN: &str = ".cadman.yaml";

pub const STATE_FILE_NAME: &str = "state.toml";
pub const APPS_FILE_NAME: &str = "apps.toml";

pub const CONFIG_DIR_NAME: &str = APP_NAME;
pub const CACHE_DIR_NAME: &str = APP_NAME;
pub const STATE_DIR_NAME: &str = APP_NAME;
pub const LOG_DIR_NAME: &str = APP_NAME;

pub const CADMAN_USER_NAME: &str = APP_NAME;
pub const CADMAN_GROUP_NAME: &str = APP_NAME;
