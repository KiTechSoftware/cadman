use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::engine::{ErrorCode, Result, models::runtime::Runtime, system::fs};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CadmanConfig {
    pub caddy: CaddyConfig,
    pub podman: PodmanConfig,
    pub runtime: RuntimeConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CaddyConfig {
    pub mode: CaddyMode,
    pub config_dir: PathBuf,
    pub sites_dir: PathBuf,
    pub admin_url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CaddyMode {
    Managed,
    External,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PodmanConfig {
    pub socket_path: Option<PathBuf>,
    pub use_socket: bool,
    pub use_cli_fallback: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RuntimeConfig {
    pub poll_interval_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    pub level: String,
    pub output: String,
}

impl Default for CadmanConfig {
    fn default() -> Self {
        Self {
            caddy: CaddyConfig::default(),
            podman: PodmanConfig::default(),
            runtime: RuntimeConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

impl Default for CaddyConfig {
    fn default() -> Self {
        Self {
            mode: CaddyMode::External,
            config_dir: PathBuf::from("/etc/caddy"),
            sites_dir: PathBuf::from("/var/lib/cadman/caddy/sites"),
            admin_url: Some("http://127.0.0.1:2019".to_string()),
        }
    }
}

impl Default for PodmanConfig {
    fn default() -> Self {
        Self {
            socket_path: None,
            use_socket: false,
            use_cli_fallback: true,
        }
    }
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            poll_interval_secs: 5,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            output: "stderr".to_string(),
        }
    }
}

/// Return the path to the global config file for this runtime.
pub fn path(runtime: &Runtime) -> PathBuf {
    runtime
        .config_path()
        .cloned()
        .unwrap_or_else(|| runtime.default_config_path())
}

/// Build a `CadmanConfig` with defaults resolved relative to the runtime state dir.
pub fn default_for(runtime: &Runtime) -> CadmanConfig {
    let mut config = CadmanConfig::default();
    config.caddy.sites_dir = runtime.state_dir().join("caddy").join("sites");
    config
}

/// Load global config. Returns defaults if the file does not exist.
pub fn load(runtime: &Runtime) -> Result<CadmanConfig> {
    let p = path(runtime);
    if !p.exists() {
        return Ok(default_for(runtime));
    }
    fs::load_toml(&p).map_err(|_| {
        ErrorCode::ConfigUnreadable
            .error()
            .with_context("path", p.display().to_string())
    })
}

/// Save global config to the runtime config path.
pub fn save(runtime: &Runtime, config: &CadmanConfig) -> Result<()> {
    let p = path(runtime);
    fs::save_toml(&p, config).map_err(|_| {
        ErrorCode::ConfigInvalid
            .error()
            .with_context("path", p.display().to_string())
    })
}
