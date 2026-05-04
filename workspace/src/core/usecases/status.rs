use std::path::PathBuf;

use serde::Serialize;

use crate::{
    core::{Context, CoreResult},
    engine::{
        ErrorCode,
        capabilities::{
            diagnostics::{DiagnosticsReport, collect_diagnostics},
            podman::{self, ContainerListFilter},
        },
        models::containers::ContainerSummary,
        registry::{self, DesiredStatus, RegistryApp, RegistrySource},
        state,
    },
};

#[derive(Debug, Clone, Serialize)]
pub struct StatusReport {
    pub runtime: StatusRuntime,
    pub diagnostics: DiagnosticsReport,
    pub config_path: PathBuf,
    pub registry_path: PathBuf,
    pub state_path: PathBuf,
    pub authorized_container_scopes: Vec<String>,
    pub observed_containers: usize,
    pub last_reconcile_at: Option<String>,
    pub apps: Vec<AppStatus>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatusRuntime {
    pub install_scope: String,
    pub requested_mode: String,
    pub effective_mode: String,
    pub effective_user: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AppStatus {
    pub id: String,
    pub name: String,
    pub source: String,
    pub desired_status: String,
    pub managed: bool,
    pub project_path: Option<PathBuf>,
    pub config_path: Option<PathBuf>,
    pub container_name: Option<String>,
    pub container_state: Option<String>,
    pub container_health: Option<String>,
    pub caddy_site_path: Option<PathBuf>,
    pub config_hash: Option<String>,
    pub labels_hash: Option<String>,
    pub ports_hash: Option<String>,
    pub site_hash: Option<String>,
    pub last_seen_at: Option<String>,
    pub last_reconcile_at: Option<String>,
    pub routes: Vec<RouteStatus>,
    pub route_hosts: Vec<String>,
    pub site_state: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RouteStatus {
    pub route_id: String,
    pub hosts: Vec<String>,
    pub site_path: PathBuf,
    pub site_hash: String,
}

pub async fn render(ctx: &Context, app: Option<&str>) -> CoreResult<()> {
    let report = status(ctx, app).await?;
    let mut output = ctx
        .ui()
        .new_output_content()
        .json(&report)
        .title("Cadman status")
        .key_value("Install Scope", &report.runtime.install_scope)
        .key_value("Requested RunMode", &report.runtime.requested_mode)
        .key_value("Effective RunMode", &report.runtime.effective_mode)
        .key_value("Effective User", &report.runtime.effective_user)
        .key_value(
            "Container Scopes",
            report.authorized_container_scopes.join(","),
        )
        .key_value("Config", report.config_path.display())
        .key_value("Registry", report.registry_path.display())
        .key_value("State", report.state_path.display())
        .key_value(
            "Last Reconcile",
            report.last_reconcile_at.as_deref().unwrap_or("-"),
        )
        .key_value("Observed Containers", report.observed_containers)
        .key_value("Podman", report.diagnostics.podman_available)
        .key_value("Caddy", report.diagnostics.caddy_available)
        .key_value("Apps", report.apps.len());

    if !report.apps.is_empty() {
        output = output.table(None, apps_table(&report.apps));
    }

    ctx.ui().print(&output)
}

pub async fn status(ctx: &Context, app: Option<&str>) -> CoreResult<StatusReport> {
    let diagnostics = collect_diagnostics(ctx.runtime())?;
    let config_path = diagnostics.config_path.clone();
    let registry_path = registry::path(ctx.runtime());
    let state_path = state::path(ctx.runtime());
    let registry = registry::load(ctx.runtime())?;
    let state = state::load(ctx.runtime())?;
    let containers = visible_containers(ctx, diagnostics.podman_available).await;
    let observed_containers = containers.len();

    let apps: Vec<RegistryApp> = if let Some(app) = app {
        vec![registry.get(app).cloned().ok_or_else(|| {
            ErrorCode::RegistryAppNotFound
                .error()
                .with_context("app", app)
        })?]
    } else {
        registry.list().into_iter().cloned().collect()
    };

    let apps = apps
        .iter()
        .map(|app| {
            let saved = state.get(&app.id);
            let live = containers
                .iter()
                .find(|container| container_matches(app, container));

            let routes: Vec<RouteStatus> = saved
                .map(|state| {
                    state
                        .routes
                        .iter()
                        .map(|route| RouteStatus {
                            route_id: route.route_id.clone(),
                            hosts: route.hosts.clone(),
                            site_path: route.site_path.clone(),
                            site_hash: route.site_hash.clone(),
                        })
                        .collect()
                })
                .unwrap_or_default();
            let route_hosts = routes
                .iter()
                .flat_map(|route| route.hosts.clone())
                .collect::<Vec<_>>();
            let caddy_site_path = saved.and_then(|state| state.caddy_site_path.clone());
            let site_state = site_state(caddy_site_path.as_ref());
            let mut warnings = Vec::new();
            if app.source == RegistrySource::PodmanLabels && live.is_none() {
                warnings.push("label-sourced container is not currently visible".to_string());
            }
            if site_state == "missing" {
                warnings.push("Caddy site file is missing".to_string());
            }

            AppStatus {
                id: app.id.clone(),
                name: app.name.clone(),
                source: source_string(app.source),
                desired_status: desired_status_string(app.desired_status),
                managed: app.managed,
                project_path: app.project_path.clone(),
                config_path: app.config_path.clone(),
                container_name: app.container_name.clone(),
                container_state: live
                    .map(|container| container.state.clone())
                    .or_else(|| saved.and_then(|state| state.container_state.clone())),
                container_health: live
                    .and_then(|container| container.health.clone())
                    .or_else(|| saved.and_then(|state| state.container_health.clone())),
                caddy_site_path,
                config_hash: saved.and_then(|state| state.config_hash.clone()),
                labels_hash: saved.and_then(|state| state.labels_hash.clone()),
                ports_hash: saved.and_then(|state| state.ports_hash.clone()),
                site_hash: saved.and_then(|state| state.site_hash.clone()),
                last_seen_at: saved.and_then(|state| state.last_seen_at.clone()),
                last_reconcile_at: saved.and_then(|state| state.last_reconcile_at.clone()),
                routes,
                route_hosts,
                site_state,
                warnings,
            }
        })
        .collect();

    Ok(StatusReport {
        runtime: StatusRuntime {
            install_scope: ctx.runtime().install_scope().to_string(),
            requested_mode: ctx.runtime().run_mode().to_string(),
            effective_mode: ctx.runtime().effective_run_mode().to_string(),
            effective_user: ctx.runtime().effective_user().to_string(),
        },
        diagnostics,
        config_path,
        registry_path,
        state_path,
        authorized_container_scopes: ctx
            .runtime()
            .visible_container_scopes()
            .into_iter()
            .map(|scope| scope.to_string())
            .collect(),
        observed_containers,
        last_reconcile_at: state.last_reconcile_at.clone(),
        apps,
    })
}

async fn visible_containers(ctx: &Context, podman_available: bool) -> Vec<ContainerSummary> {
    if !podman_available {
        return Vec::new();
    }

    podman::list_containers(ctx.runtime(), ContainerListFilter::All)
        .await
        .map(|report| report.containers)
        .unwrap_or_default()
}

fn container_matches(app: &RegistryApp, container: &ContainerSummary) -> bool {
    container.name == app.name
        || container.name == app.id
        || container.labels.get("cadman.app.id") == Some(&app.id)
        || container.labels.get("cadman.app.name") == Some(&app.name)
}

fn apps_table(apps: &[AppStatus]) -> scriba::Table {
    let rows = apps
        .iter()
        .map(|app| {
            vec![
                app.id.clone(),
                app.name.clone(),
                app.desired_status.clone(),
                app.source.clone(),
                app.container_state
                    .clone()
                    .unwrap_or_else(|| "-".to_string()),
                app.container_health
                    .clone()
                    .unwrap_or_else(|| "-".to_string()),
                app.routes.len().to_string(),
                app.site_state.clone(),
            ]
        })
        .collect();

    scriba::Table::new(
        vec![
            "ID".to_string(),
            "NAME".to_string(),
            "DESIRED".to_string(),
            "SOURCE".to_string(),
            "STATE".to_string(),
            "HEALTH".to_string(),
            "ROUTES".to_string(),
            "SITE".to_string(),
        ],
        rows,
    )
}

fn site_state(path: Option<&PathBuf>) -> String {
    match path {
        Some(path) if path.is_file() => "present".to_string(),
        Some(_) => "missing".to_string(),
        None => "-".to_string(),
    }
}

fn desired_status_string(status: DesiredStatus) -> String {
    match status {
        DesiredStatus::Up => "up".to_string(),
        DesiredStatus::Down => "down".to_string(),
    }
}

fn source_string(source: RegistrySource) -> String {
    match source {
        RegistrySource::ProjectConfig => "project-config".to_string(),
        RegistrySource::PodmanLabels => "podman-labels".to_string(),
    }
}

#[cfg(test)]
#[allow(clippy::await_holding_lock)]
mod tests {
    use super::*;
    use std::fs as std_fs;

    use crate::engine::{
        models::runtime::{InstallScope, Runtime},
        registry::{Registry, RegistryApp},
    };



    struct EnvGuard {
        xdg_state_home: Option<std::ffi::OsString>,
    }

    impl EnvGuard {
        fn set_state_home(path: &std::path::Path) -> Self {
            let xdg_state_home = std::env::var_os("XDG_STATE_HOME");
            unsafe {
                std::env::set_var("XDG_STATE_HOME", path);
            }
            Self { xdg_state_home }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            unsafe {
                if let Some(value) = &self.xdg_state_home {
                    std::env::set_var("XDG_STATE_HOME", value);
                } else {
                    std::env::remove_var("XDG_STATE_HOME");
                }
            }
        }
    }

    fn test_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("cadman_status_{}_{}", std::process::id(), name));
        let _ = std_fs::remove_dir_all(&dir);
        dir
    }

    fn ctx_for(state_home: &std::path::Path) -> (EnvGuard, Context) {
        let guard = EnvGuard::set_state_home(state_home);
        let mut runtime = Runtime::new();
        runtime.set_install_scope(InstallScope::User);
        (guard, Context::new(runtime))
    }

    fn app(id: &str, name: &str) -> RegistryApp {
        RegistryApp {
            id: id.to_string(),
            name: name.to_string(),
            source: RegistrySource::ProjectConfig,
            project_path: Some(PathBuf::from(format!("/projects/{id}"))),
            config_path: Some(PathBuf::from(format!("/projects/{id}/cadman.toml"))),
            desired_status: DesiredStatus::Down,
            managed: true,
            container_scope: None,
            container_id: None,
            container_name: None,
        }
    }

    #[tokio::test]
    async fn status_works_with_empty_registry_and_state() {
        let _lock = crate::TEST_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let state_home = test_dir("empty");
        let (_guard, ctx) = ctx_for(&state_home);

        let report = status(&ctx, None).await.unwrap();

        assert!(report.apps.is_empty());
        assert!(report.registry_path.ends_with("registry.toml"));
        assert!(report.state_path.ends_with("state.json"));

        let _ = std_fs::remove_dir_all(&state_home);
    }

    #[tokio::test]
    async fn status_filters_app_by_id_or_name() {
        let _lock = crate::TEST_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let state_home = test_dir("filter");
        let (_guard, ctx) = ctx_for(&state_home);
        let mut registry = Registry::empty();
        registry.add_app(app("web", "Website")).unwrap();
        registry::save(ctx.runtime(), &registry).unwrap();

        let report = status(&ctx, Some("Website")).await.unwrap();

        assert_eq!(report.apps.len(), 1);
        assert_eq!(report.apps[0].id, "web");

        let _ = std_fs::remove_dir_all(&state_home);
    }

    #[tokio::test]
    async fn status_missing_app_returns_registry_app_not_found() {
        let _lock = crate::TEST_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let state_home = test_dir("missing");
        let (_guard, ctx) = ctx_for(&state_home);

        let err = status(&ctx, Some("missing")).await.unwrap_err();

        assert_eq!(err.code, ErrorCode::RegistryAppNotFound);

        let _ = std_fs::remove_dir_all(&state_home);
    }
}
