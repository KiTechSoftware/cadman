use std::path::PathBuf;

use scriba::Output;
use serde::Serialize;

use crate::{
    core::{Context, CoreResult},
    engine::{
        ErrorCode,
        config::{self, CadmanConfig},
    },
};

#[derive(Debug, Clone, Copy)]
pub enum ConfigCommand {
    Path,
    Show,
    Init,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigPathReport {
    pub path: PathBuf,
    pub install_scope: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigShowReport {
    pub path: PathBuf,
    pub exists: bool,
    pub config: CadmanConfig,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigInitReport {
    pub path: PathBuf,
    pub created: bool,
    pub overwritten: bool,
}

pub fn config_path(ctx: &Context) -> CoreResult<ConfigPathReport> {
    Ok(ConfigPathReport {
        path: config::path(ctx.runtime()),
        install_scope: ctx.runtime().install_scope().to_string(),
    })
}

pub fn config_show(ctx: &Context) -> CoreResult<ConfigShowReport> {
    let path = config::path(ctx.runtime());
    let exists = path.exists();
    let config = config::load(ctx.runtime())?;

    Ok(ConfigShowReport {
        path,
        exists,
        config,
    })
}

pub fn config_init(ctx: &Context) -> CoreResult<ConfigInitReport> {
    let path = config::path(ctx.runtime());
    let overwritten = path.exists();

    if overwritten && !ctx.force() {
        return Err(ErrorCode::ConfigAlreadyExists
            .error()
            .with_context("path", path.display().to_string())
            .with_context("hint", "use --force to overwrite"));
    }

    let config = config::default_for(ctx.runtime());
    config::save(ctx.runtime(), &config)?;

    Ok(ConfigInitReport {
        path,
        created: !overwritten,
        overwritten,
    })
}

pub async fn render_path(ctx: &Context) -> CoreResult<()> {
    let report = config_path(ctx)?;
    let output = if structured(ctx) {
        Output::from_serializable(report)
    } else {
        Output::new()
            .title("Cadman config path")
            .key_value("Path", report.path.display())
            .key_value("Install Scope", &report.install_scope)
    };

    ctx.ui().print(&output)
}

pub async fn render_show(ctx: &Context) -> CoreResult<()> {
    let report = config_show(ctx)?;
    let output = if structured(ctx) {
        Output::from_serializable(report)
    } else {
        Output::new()
            .title("Cadman config")
            .key_value("Path", report.path.display())
            .key_value("Exists", report.exists)
            .key_value("Caddy mode", format!("{:?}", report.config.caddy.mode))
            .key_value("Caddy config dir", report.config.caddy.config_dir.display())
            .key_value("Caddy sites dir", report.config.caddy.sites_dir.display())
            .key_value("Podman CLI fallback", report.config.podman.use_cli_fallback)
            .key_value("Poll interval", report.config.runtime.poll_interval_secs)
            .key_value("Log level", &report.config.logging.level)
            .key_value(
                "Discovery roots",
                format_paths(&report.config.discovery.roots),
            )
            .key_value("Discovery max depth", report.config.discovery.max_depth)
            .key_value(
                "Discovery follow symlinks",
                report.config.discovery.follow_symlinks,
            )
            .key_value(
                "Discovery include hidden",
                report.config.discovery.include_hidden,
            )
            .key_value(
                "Discovery auto register",
                report.config.discovery.auto_register,
            )
    };

    ctx.ui().print(&output)
}

pub async fn render_init(ctx: &Context) -> CoreResult<()> {
    let report = config_init(ctx)?;
    let title = if report.overwritten {
        "Updated Cadman config"
    } else {
        "Created Cadman config"
    };

    let output = if structured(ctx) {
        Output::from_serializable(report)
    } else {
        Output::new()
            .title(title)
            .key_value("Path", report.path.display())
            .key_value("Created", report.created)
            .key_value("Overwritten", report.overwritten)
    };

    ctx.ui().print(&output)
}

fn structured(ctx: &Context) -> bool {
    ctx.runtime().options().output_format().is_structured()
        || ctx.runtime().options().output_envelope().is_json()
}

fn format_paths(paths: &[PathBuf]) -> String {
    if paths.is_empty() {
        "-".to_string()
    } else {
        paths
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(",")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs as std_fs;

    use crate::engine::{
        constants::CONFIG_FILE_NAME,
        models::runtime::{InstallScope, Runtime},
    };

    fn test_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("cadman_config_{}_{}", std::process::id(), name));
        let _ = std_fs::remove_dir_all(&dir);
        dir
    }

    fn ctx_for(dir: PathBuf, force: bool) -> Context {
        let mut runtime = Runtime::new();
        runtime
            .set_install_scope(InstallScope::User)
            .set_config_path(Some(dir.join(CONFIG_FILE_NAME)))
            .set_force(force);
        Context::new(runtime)
    }

    #[test]
    fn config_path_uses_runtime_effective_path() {
        let dir = test_dir("path");
        let ctx = ctx_for(dir.clone(), false);

        let report = config_path(&ctx).unwrap();

        assert_eq!(report.path, dir.join(CONFIG_FILE_NAME));
        assert_eq!(report.install_scope, "user");

        let _ = std_fs::remove_dir_all(&dir);
    }

    #[test]
    fn config_show_returns_defaults_when_missing() {
        let dir = test_dir("show_missing");
        let ctx = ctx_for(dir.clone(), false);

        let report = config_show(&ctx).unwrap();

        assert!(!report.exists);
        assert_eq!(report.config.runtime.poll_interval_secs, 5);

        let _ = std_fs::remove_dir_all(&dir);
    }

    #[test]
    fn config_init_writes_default_config() {
        let dir = test_dir("init");
        let ctx = ctx_for(dir.clone(), false);

        let report = config_init(&ctx).unwrap();

        assert!(report.created);
        assert!(!report.overwritten);
        assert!(report.path.exists());

        let _ = std_fs::remove_dir_all(&dir);
    }

    #[test]
    fn config_init_refuses_to_overwrite_without_force() {
        let dir = test_dir("exists");
        let ctx = ctx_for(dir.clone(), false);
        config_init(&ctx).unwrap();

        let err = config_init(&ctx).unwrap_err();

        assert_eq!(err.code, ErrorCode::ConfigAlreadyExists);

        let _ = std_fs::remove_dir_all(&dir);
    }

    #[test]
    fn config_init_force_overwrites_existing_config() {
        let dir = test_dir("force");
        let path = dir.join(CONFIG_FILE_NAME);
        std_fs::create_dir_all(&dir).unwrap();
        std_fs::write(&path, "existing").unwrap();

        let ctx = ctx_for(dir.clone(), true);
        let report = config_init(&ctx).unwrap();

        assert!(!report.created);
        assert!(report.overwritten);
        assert_ne!(std_fs::read_to_string(&path).unwrap(), "existing");

        let _ = std_fs::remove_dir_all(&dir);
    }
}
