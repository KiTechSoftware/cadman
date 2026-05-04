use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use veltrix::os::paths as os_paths;

use crate::engine::{
    ErrorCode, Result, constants::CADDY_SERVICE_NAME, models::runtime::Runtime, system::fs,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct CadmanConfig {
    pub caddy: CaddyConfig,
    pub podman: PodmanConfig,
    pub runtime: RuntimeConfig,
    pub logging: LoggingConfig,
    pub discovery: DiscoveryConfig,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DiscoveryConfig {
    pub roots: Vec<PathBuf>,
    pub max_depth: usize,
    pub follow_symlinks: bool,
    pub include_hidden: bool,
    pub auto_register: bool,
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

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            roots: Vec::new(),
            max_depth: 6,
            follow_symlinks: false,
            include_hidden: false,
            auto_register: false,
        }
    }
}

/// Return the path to the global config file for this runtime.
pub fn path(runtime: &Runtime) -> PathBuf {
    runtime.effective_config_path()
}

/// Build a `CadmanConfig` with defaults resolved relative to the runtime state dir.
pub fn default_for(runtime: &Runtime) -> CadmanConfig {
    let mut config = CadmanConfig::default();
    config.caddy.sites_dir = runtime.state_dir().join(CADDY_SERVICE_NAME).join("sites");
    config
}

/// Load global config. Returns defaults if the file does not exist.
pub fn load(runtime: &Runtime) -> Result<CadmanConfig> {
    let p = path(runtime);
    if !p.exists() {
        return Ok(default_for(runtime));
    }

    let raw = fs::read_text(&p).map_err(|err| {
        ErrorCode::ConfigUnreadable
            .error()
            .with_context("path", p.display().to_string())
            .with_context("error", err.to_string())
    })?;

    let mut config: CadmanConfig = toml::from_str(&raw).map_err(|err| {
        ErrorCode::ConfigInvalid
            .error()
            .with_context("path", p.display().to_string())
            .with_context("error", err.to_string())
    })?;
    expand_discovery_roots(&mut config)?;
    Ok(config)
}

/// Save global config to the runtime config path.
pub fn save(runtime: &Runtime, config: &CadmanConfig) -> Result<()> {
    let p = path(runtime);
    fs::save_toml(&p, config)
}

fn expand_discovery_roots(config: &mut CadmanConfig) -> Result<()> {
    config.discovery.roots = config
        .discovery
        .roots
        .iter()
        .map(|root| expand_discovery_path(root))
        .collect::<Result<Vec<_>>>()?;
    Ok(())
}

pub fn expand_discovery_path(root: &std::path::Path) -> Result<PathBuf> {
    let root = root.to_string_lossy();
    if root == "~" {
        return os_paths::home_dir().map_err(|err| {
            ErrorCode::ConfigInvalid
                .error()
                .with_context("field", "discovery.roots")
                .with_context("root", root.to_string())
                .with_context("error", err.to_string())
        });
    }

    if let Some(rest) = root.strip_prefix("~/") {
        return os_paths::home_dir()
            .map(|home| home.join(rest))
            .map_err(|err| {
                ErrorCode::ConfigInvalid
                    .error()
                    .with_context("field", "discovery.roots")
                    .with_context("root", root.to_string())
                    .with_context("error", err.to_string())
            });
    }

    os_paths::expand_user_path(&root).map_err(|err| {
        ErrorCode::ConfigInvalid
            .error()
            .with_context("field", "discovery.roots")
            .with_context("root", root.to_string())
            .with_context("error", err.to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs as std_fs;

    use crate::engine::{
        constants::CONFIG_FILE_NAME,
        models::runtime::{InstallScope, Runtime},
    };



    struct EnvGuard {
        home: Option<std::ffi::OsString>,
    }

    impl EnvGuard {
        fn set_home(path: &std::path::Path) -> Self {
            let home = std::env::var_os("HOME");
            unsafe {
                std::env::set_var("HOME", path);
            }
            Self { home }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            unsafe {
                if let Some(value) = &self.home {
                    std::env::set_var("HOME", value);
                } else {
                    std::env::remove_var("HOME");
                }
            }
        }
    }

    fn test_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "cadman_global_config_{}_{}",
            std::process::id(),
            name
        ));
        let _ = std_fs::remove_dir_all(&dir);
        dir
    }

    fn runtime_for(path: PathBuf) -> Runtime {
        let mut runtime = Runtime::new();
        runtime
            .set_install_scope(InstallScope::User)
            .set_config_path(Some(path));
        runtime
    }

    #[test]
    fn discovery_config_defaults_are_stable() {
        let discovery = DiscoveryConfig::default();

        assert!(discovery.roots.is_empty());
        assert_eq!(discovery.max_depth, 6);
        assert!(!discovery.follow_symlinks);
        assert!(!discovery.include_hidden);
        assert!(!discovery.auto_register);
    }

    #[test]
    fn missing_discovery_section_uses_defaults() {
        let dir = test_dir("missing_discovery");
        let config_path = dir.join(CONFIG_FILE_NAME);
        std_fs::create_dir_all(&dir).unwrap();
        std_fs::write(
            &config_path,
            r#"
[runtime]
poll_interval_secs = 10
"#,
        )
        .unwrap();
        let runtime = runtime_for(config_path);

        let config = load(&runtime).unwrap();

        assert!(config.discovery.roots.is_empty());
        assert_eq!(config.discovery.max_depth, 6);
        assert_eq!(config.runtime.poll_interval_secs, 10);

        let _ = std_fs::remove_dir_all(&dir);
    }

    #[test]
    fn config_loads_discovery_roots() {
        let dir = test_dir("roots");
        let config_path = dir.join(CONFIG_FILE_NAME);
        std_fs::create_dir_all(&dir).unwrap();
        std_fs::write(
            &config_path,
            r#"
[discovery]
roots = ["/var/apps", "/srv/apps"]
max_depth = 8
follow_symlinks = true
include_hidden = true
auto_register = true
"#,
        )
        .unwrap();
        let runtime = runtime_for(config_path);

        let config = load(&runtime).unwrap();

        assert_eq!(
            config.discovery.roots,
            vec![PathBuf::from("/var/apps"), PathBuf::from("/srv/apps")]
        );
        assert_eq!(config.discovery.max_depth, 8);
        assert!(config.discovery.follow_symlinks);
        assert!(config.discovery.include_hidden);
        assert!(config.discovery.auto_register);

        let _ = std_fs::remove_dir_all(&dir);
    }

    #[test]
    fn tilde_discovery_roots_expand_to_home() {
        let _guard = crate::TEST_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = test_dir("tilde");
        let home = dir.join("home");
        let _home_guard = EnvGuard::set_home(&home);
        let config_path = dir.join(CONFIG_FILE_NAME);
        std_fs::create_dir_all(&dir).unwrap();
        std_fs::write(
            &config_path,
            r#"
[discovery]
roots = ["~/apps"]
"#,
        )
        .unwrap();
        let runtime = runtime_for(config_path);

        let config = load(&runtime).unwrap();

        assert_eq!(config.discovery.roots, vec![home.join("apps")]);

        let _ = std_fs::remove_dir_all(&dir);
    }
}
