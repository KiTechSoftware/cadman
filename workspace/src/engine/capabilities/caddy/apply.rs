use std::{
    collections::BTreeSet,
    fs as std_fs,
    path::{Path, PathBuf},
};

use serde::Serialize;

use crate::engine::{
    ErrorCode, Result, capabilities::reconcile::DesiredRoute, config::CaddyConfig,
    models::runtime::Runtime, system::fs,
};

use super::{
    sites::{CaddySiteRoute, MANAGED_HEADER, site_content, site_file_name},
    validate::{
        CaddyCommandRunner, CaddyCommandStatus, ProcessCaddyCommandRunner, default_config_path,
    },
};

#[derive(Debug, Clone, Serialize)]
pub struct CaddyApplyReport {
    pub sites_dir: PathBuf,
    pub changed: bool,
    pub actions: Vec<CaddySiteAction>,
    pub validation: CaddyCommandStatus,
    pub reload: CaddyCommandStatus,
}

#[derive(Debug, Clone, Serialize)]
pub struct CaddySiteAction {
    pub path: PathBuf,
    pub action: CaddySiteActionKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CaddySiteActionKind {
    Create,
    Update,
    Unchanged,
    Remove,
}

pub fn apply_desired_routes(
    runtime: &Runtime,
    config: &CaddyConfig,
    routes: &[DesiredRoute],
) -> Result<CaddyApplyReport> {
    apply_desired_routes_with_runner(
        runtime,
        config,
        routes,
        runtime.options().dry_run(),
        &ProcessCaddyCommandRunner,
    )
}

pub fn apply_desired_routes_with_runner(
    runtime: &Runtime,
    config: &CaddyConfig,
    routes: &[DesiredRoute],
    dry_run: bool,
    runner: &dyn CaddyCommandRunner,
) -> Result<CaddyApplyReport> {
    let desired = desired_sites(&config.sites_dir, routes);
    let mut desired_paths = BTreeSet::new();
    let mut actions = Vec::new();
    let mut changed = false;

    if !dry_run {
        fs::create_dir_all(&config.sites_dir)?;
    }

    for (path, content) in desired {
        desired_paths.insert(path.clone());
        let action = match fs::read_text(&path) {
            Ok(existing) if existing == content => CaddySiteActionKind::Unchanged,
            Ok(_) => CaddySiteActionKind::Update,
            Err(_) => CaddySiteActionKind::Create,
        };

        if matches!(
            action,
            CaddySiteActionKind::Create | CaddySiteActionKind::Update
        ) {
            changed = true;
            if !dry_run {
                fs::write_text(&path, &content).map_err(|err| {
                    ErrorCode::CaddySiteWriteFailed
                        .error()
                        .with_context("path", path.display().to_string())
                        .with_context("error", err.to_string())
                })?;
            }
        }

        actions.push(CaddySiteAction { path, action });
    }

    for path in stale_managed_sites(&config.sites_dir, &desired_paths)? {
        changed = true;
        if !dry_run {
            std_fs::remove_file(&path).map_err(|err| {
                ErrorCode::CaddySiteWriteFailed
                    .error()
                    .with_context("path", path.display().to_string())
                    .with_context("error", err.to_string())
            })?;
        }
        actions.push(CaddySiteAction {
            path,
            action: CaddySiteActionKind::Remove,
        });
    }

    let config_path = default_config_path(&config.config_dir);
    let validation = if dry_run || !changed {
        CaddyCommandStatus::skipped("caddy validate")
    } else {
        runner.validate(&config_path, runtime.effective_run_mode())
    };
    let reload = if dry_run || !changed || !validation.success {
        CaddyCommandStatus::skipped("caddy reload")
    } else {
        runner.reload(&config_path, runtime.effective_run_mode())
    };

    Ok(CaddyApplyReport {
        sites_dir: config.sites_dir.clone(),
        changed,
        actions,
        validation,
        reload,
    })
}

fn desired_sites(sites_dir: &Path, routes: &[DesiredRoute]) -> Vec<(PathBuf, String)> {
    routes
        .iter()
        .map(|route| {
            let id = format!("{}-{}", route.app_id, route.route_id);
            let site_route = CaddySiteRoute {
                id: id.clone(),
                hosts: route.hosts.clone(),
                path: route.path.clone(),
                upstream: route.upstream.clone(),
            };
            (
                sites_dir.join(site_file_name(&id)),
                site_content(&site_route),
            )
        })
        .collect()
}

fn stale_managed_sites(
    sites_dir: &Path,
    desired_paths: &BTreeSet<PathBuf>,
) -> Result<Vec<PathBuf>> {
    if !sites_dir.exists() {
        return Ok(Vec::new());
    }

    let entries = std_fs::read_dir(sites_dir).map_err(|err| {
        ErrorCode::CaddySiteNotFound
            .error()
            .with_context("path", sites_dir.display().to_string())
            .with_context("error", err.to_string())
    })?;

    let mut stale = Vec::new();
    for entry in entries {
        let path = entry?.path();
        if desired_paths.contains(&path) || !path.is_file() {
            continue;
        }
        let Ok(content) = fs::read_text(&path) else {
            continue;
        };
        if content.starts_with(MANAGED_HEADER) {
            stale.push(path);
        }
    }

    Ok(stale)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::engine::models::runtime::{InstallScope, RunMode, Runtime};

    #[derive(Debug)]
    struct FakeRunner {
        validate_success: bool,
    }

    impl CaddyCommandRunner for FakeRunner {
        fn validate(&self, _config_path: &Path, _mode: RunMode) -> CaddyCommandStatus {
            CaddyCommandStatus {
                command: "caddy validate".to_string(),
                skipped: false,
                success: self.validate_success,
                status: Some(if self.validate_success { 0 } else { 1 }),
                stdout: String::new(),
                stderr: String::new(),
            }
        }

        fn reload(&self, _config_path: &Path, _mode: RunMode) -> CaddyCommandStatus {
            CaddyCommandStatus {
                command: "caddy reload".to_string(),
                skipped: false,
                success: true,
                status: Some(0),
                stdout: String::new(),
                stderr: String::new(),
            }
        }
    }

    fn test_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("cadman_caddy_{}_{}", std::process::id(), name));
        let _ = std_fs::remove_dir_all(&dir);
        dir
    }

    fn runtime() -> Runtime {
        let mut runtime = Runtime::new();
        runtime.set_install_scope(InstallScope::User);
        runtime
    }

    fn config(dir: &Path) -> CaddyConfig {
        CaddyConfig {
            config_dir: dir.join("config"),
            sites_dir: dir.join("sites"),
            ..Default::default()
        }
    }

    fn route() -> DesiredRoute {
        DesiredRoute {
            app_id: "app".to_string(),
            route_id: "web".to_string(),
            hosts: vec!["app.local".to_string()],
            path: None,
            upstream: "127.0.0.1:8080".to_string(),
            source: crate::engine::capabilities::reconcile::RouteSource::ProjectConfig,
        }
    }

    #[test]
    fn unchanged_content_produces_no_write_action() {
        let dir = test_dir("unchanged");
        let config = config(&dir);
        std_fs::create_dir_all(&config.sites_dir).unwrap();
        let path = config.sites_dir.join("app-web.site");
        std_fs::write(
            &path,
            site_content(&CaddySiteRoute {
                id: "app-web".to_string(),
                hosts: vec!["app.local".to_string()],
                path: None,
                upstream: "127.0.0.1:8080".to_string(),
            }),
        )
        .unwrap();

        let report = apply_desired_routes_with_runner(
            &runtime(),
            &config,
            &[route()],
            false,
            &FakeRunner {
                validate_success: true,
            },
        )
        .unwrap();

        assert!(!report.changed);
        assert_eq!(report.actions[0].action, CaddySiteActionKind::Unchanged);
        assert!(report.reload.skipped);
        let _ = std_fs::remove_dir_all(&dir);
    }

    #[test]
    fn changed_content_writes_file() {
        let dir = test_dir("changed");
        let config = config(&dir);

        let report = apply_desired_routes_with_runner(
            &runtime(),
            &config,
            &[route()],
            false,
            &FakeRunner {
                validate_success: true,
            },
        )
        .unwrap();

        assert!(report.changed);
        assert_eq!(report.actions[0].action, CaddySiteActionKind::Create);
        assert!(config.sites_dir.join("app-web.site").exists());
        assert!(report.validation.success);
        assert!(report.reload.success);
        let _ = std_fs::remove_dir_all(&dir);
    }

    #[test]
    fn stale_managed_file_is_removed() {
        let dir = test_dir("stale");
        let config = config(&dir);
        std_fs::create_dir_all(&config.sites_dir).unwrap();
        let path = config.sites_dir.join("old.site");
        std_fs::write(&path, format!("{MANAGED_HEADER}\nold.local {{}}\n")).unwrap();

        let report = apply_desired_routes_with_runner(
            &runtime(),
            &config,
            &[],
            false,
            &FakeRunner {
                validate_success: true,
            },
        )
        .unwrap();

        assert!(report.changed);
        assert_eq!(report.actions[0].action, CaddySiteActionKind::Remove);
        assert!(!path.exists());
        let _ = std_fs::remove_dir_all(&dir);
    }

    #[test]
    fn non_managed_file_is_ignored() {
        let dir = test_dir("non_managed");
        let config = config(&dir);
        std_fs::create_dir_all(&config.sites_dir).unwrap();
        let path = config.sites_dir.join("manual.site");
        std_fs::write(&path, "manual.local {}\n").unwrap();

        let report = apply_desired_routes_with_runner(
            &runtime(),
            &config,
            &[],
            false,
            &FakeRunner {
                validate_success: true,
            },
        )
        .unwrap();

        assert!(!report.changed);
        assert!(path.exists());
        let _ = std_fs::remove_dir_all(&dir);
    }

    #[test]
    fn validate_failure_prevents_reload() {
        let dir = test_dir("validate_failure");
        let config = config(&dir);

        let report = apply_desired_routes_with_runner(
            &runtime(),
            &config,
            &[route()],
            false,
            &FakeRunner {
                validate_success: false,
            },
        )
        .unwrap();

        assert!(!report.validation.success);
        assert!(report.reload.skipped);
        let _ = std_fs::remove_dir_all(&dir);
    }

    #[test]
    fn no_changes_skips_reload() {
        let dir = test_dir("no_changes");
        let config = config(&dir);

        let report = apply_desired_routes_with_runner(
            &runtime(),
            &config,
            &[],
            false,
            &FakeRunner {
                validate_success: true,
            },
        )
        .unwrap();

        assert!(!report.changed);
        assert!(report.validation.skipped);
        assert!(report.reload.skipped);
        let _ = std_fs::remove_dir_all(&dir);
    }
}
