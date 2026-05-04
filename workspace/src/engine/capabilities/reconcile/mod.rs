pub mod desired;
pub mod labels;
pub mod plan;
pub mod projects;
pub mod report;

use std::{
    collections::{BTreeMap, hash_map::DefaultHasher},
    hash::{Hash, Hasher},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::engine::{
    ErrorCode, Result,
    capabilities::caddy::sites::{CaddySiteRoute, site_content, site_file_name},
    config,
    models::{containers::ContainerSummary, runtime::Runtime},
    registry::{self, Registry, RegistryApp, RegistrySource},
    state::{self, AppState, RouteState},
    system::fs,
};

pub use desired::{DesiredRoute, RouteSource};
pub use plan::ReconcileRequest;
pub use report::{ReconcileCounts, ReconcileReport};

pub async fn reconcile(runtime: &Runtime, request: ReconcileRequest) -> Result<ReconcileReport> {
    let registry_path = registry::path(runtime);
    let mut registry = registry::load(runtime)?;
    let config = config::load(runtime)?;

    let auto_registered_projects = if config.discovery.auto_register {
        auto_register_projects(runtime, &mut registry)?
    } else {
        Vec::new()
    };

    let containers =
        match super::podman::list_containers(runtime, super::podman::ContainerListFilter::Running)
            .await
        {
            Ok(report) => report.containers,
            Err(err)
                if request.dry_run
                    && matches!(
                        err.code,
                        ErrorCode::PodmanMissing | ErrorCode::PodmanCommandFailed
                    ) =>
            {
                Vec::new()
            }
            Err(err) => return Err(err),
        };

    let label_result = labels::collect_label_routes(&containers, &mut registry, request.dry_run)?;
    let project_routes = projects::collect_project_routes(&registry)?;

    if !request.dry_run && (label_result.registry_changed || !auto_registered_projects.is_empty()) {
        registry::save(runtime, &registry)?;
    }

    let mut desired_routes = label_result.routes;
    desired_routes.extend(project_routes);

    let missing_label_apps = missing_label_apps(&registry, &containers);
    let caddy = super::caddy::apply::apply_desired_routes(runtime, &config.caddy, &desired_routes)?;
    let counts = ReconcileCounts::from_report(desired_routes.len(), &caddy);
    let warnings = reconcile_warnings(&missing_label_apps, &caddy);
    let state_updated = if request.dry_run {
        false
    } else {
        update_state(
            runtime,
            &registry,
            &containers,
            &desired_routes,
            &config.caddy,
            &missing_label_apps,
        )?;
        true
    };

    Ok(ReconcileReport {
        dry_run: request.dry_run,
        registry_path,
        desired_routes,
        registered_apps: label_result.registered_apps,
        updated_apps: label_result.updated_apps,
        auto_registered_projects,
        missing_label_apps,
        caddy,
        counts,
        state_updated,
        warnings,
    })
}

fn auto_register_projects(runtime: &Runtime, registry: &mut Registry) -> Result<Vec<RegistryApp>> {
    let config = config::load(runtime)?;
    if config.discovery.roots.is_empty() {
        return Ok(Vec::new());
    }

    let report = config::discover_project_configs(config::ProjectDiscoveryRequest {
        roots: config.discovery.roots.clone(),
        max_depth: config.discovery.max_depth,
        follow_symlinks: config.discovery.follow_symlinks,
        include_hidden: config.discovery.include_hidden,
    })?;

    let mut registered = Vec::new();
    for project in report.projects {
        let app = RegistryApp {
            id: registry::sanitize_app_id(&project.project_name),
            name: project.project_name,
            source: RegistrySource::ProjectConfig,
            project_path: Some(project.root),
            config_path: Some(project.config_path),
            desired_status: registry::DesiredStatus::Down,
            managed: true,
            container_scope: None,
            container_id: None,
            container_name: None,
        };

        if registry.get(&app.id).is_some() || registry.get(&app.name).is_some() {
            continue;
        }

        registry.add_app(app.clone())?;
        registered.push(app);
    }

    Ok(registered)
}

fn missing_label_apps(registry: &Registry, containers: &[ContainerSummary]) -> Vec<String> {
    registry
        .list()
        .into_iter()
        .filter(|app| app.source == RegistrySource::PodmanLabels)
        .filter(|app| {
            !containers.iter().any(|container| {
                app.container_name.as_ref() == Some(&container.name)
                    || app
                        .container_id
                        .as_ref()
                        .is_some_and(|id| id == &container.id)
            })
        })
        .map(|app| app.id.clone())
        .collect()
}

fn update_state(
    runtime: &Runtime,
    registry: &Registry,
    containers: &[ContainerSummary],
    desired_routes: &[DesiredRoute],
    caddy_config: &config::CaddyConfig,
    missing_label_apps: &[String],
) -> Result<()> {
    let now = timestamp();
    let mut state = state::load(runtime)?;
    let mut routes_by_app: BTreeMap<String, Vec<RouteState>> = BTreeMap::new();

    for route in desired_routes {
        let site_path = site_path(caddy_config, route);
        let site_hash = hash_string(&site_content(&CaddySiteRoute {
            id: site_id(route),
            hosts: route.hosts.clone(),
            path: route.path.clone(),
            upstream: route.upstream.clone(),
        }));
        routes_by_app
            .entry(route.app_id.clone())
            .or_default()
            .push(RouteState {
                route_id: route.route_id.clone(),
                hosts: route.hosts.clone(),
                site_path,
                site_hash,
            });
    }

    for app in registry.list() {
        let live = containers
            .iter()
            .find(|container| container_matches(app, container));
        let routes = routes_by_app.remove(&app.id).unwrap_or_default();
        let first_route = routes.first();

        let mut app_state = AppState {
            app_id: app.id.clone(),
            container_id: live
                .map(|container| container.id.clone())
                .or_else(|| app.container_id.clone()),
            container_name: live
                .map(|container| container.name.clone())
                .or_else(|| app.container_name.clone()),
            container_state: live.map(|container| container.state.clone()),
            container_health: live.and_then(|container| container.health.clone()),
            caddy_site_path: first_route.map(|route| route.site_path.clone()),
            config_hash: app
                .config_path
                .as_ref()
                .and_then(|path| fs::read_text(path).ok())
                .map(|content| hash_string(&content)),
            labels_hash: live.map(|container| hash_debug(&container.labels)),
            ports_hash: live.map(|container| hash_debug(&container.ports)),
            site_hash: first_route.map(|route| route.site_hash.clone()),
            last_seen_at: live.map(|_| now.clone()),
            last_reconcile_at: Some(now.clone()),
            routes,
        };

        if missing_label_apps.iter().any(|id| id == &app.id) {
            app_state.container_state = Some("missing".to_string());
        }

        state.upsert_app(app_state);
    }

    state.last_reconcile_at = Some(now);
    state::save(runtime, &state)
}

fn reconcile_warnings(
    missing_label_apps: &[String],
    caddy: &super::caddy::apply::CaddyApplyReport,
) -> Vec<String> {
    let mut warnings = Vec::new();
    for app in missing_label_apps {
        warnings.push(format!(
            "label-sourced app '{app}' was not observed at runtime"
        ));
    }
    if !caddy.validation.skipped && !caddy.validation.success {
        warnings.push("caddy validation failed; reload was skipped".to_string());
    }
    warnings
}

fn container_matches(app: &RegistryApp, container: &ContainerSummary) -> bool {
    app.container_id
        .as_ref()
        .is_some_and(|id| id == &container.id)
        || app.container_name.as_ref() == Some(&container.name)
        || container.labels.get("cadman.id") == Some(&app.id)
        || container.labels.get("cadman.name") == Some(&app.name)
        || container.name == app.name
        || container.name == app.id
}

fn site_path(caddy_config: &config::CaddyConfig, route: &DesiredRoute) -> PathBuf {
    caddy_config.sites_dir.join(site_file_name(&site_id(route)))
}

fn site_id(route: &DesiredRoute) -> String {
    format!("{}-{}", route.app_id, route.route_id)
}

fn hash_string(value: &str) -> String {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn hash_debug<T: std::fmt::Debug>(value: &T) -> String {
    hash_string(&format!("{value:?}"))
}

fn timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::BTreeMap, fs as std_fs};

    use crate::engine::{
        capabilities::{
            caddy::apply::{CaddyApplyReport, CaddySiteAction, CaddySiteActionKind},
            reconcile::RouteSource,
        },
        config::{CaddyConfig, ProjectConfigFormat, write_default_project_config},
        models::{
            containers::{ContainerPort, ContainerScope},
            runtime::{InstallScope, Runtime},
        },
        registry::DesiredStatus,
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
        let dir = std::env::temp_dir().join(format!(
            "cadman_reconcile_state_{}_{}",
            std::process::id(),
            name
        ));
        let _ = std_fs::remove_dir_all(&dir);
        dir
    }

    fn runtime(state_home: &std::path::Path) -> (EnvGuard, Runtime) {
        let guard = EnvGuard::set_state_home(state_home);
        let mut runtime = Runtime::new();
        runtime.set_install_scope(InstallScope::User);
        (guard, runtime)
    }

    fn caddy_config(dir: &std::path::Path) -> CaddyConfig {
        CaddyConfig {
            config_dir: dir.join("config"),
            sites_dir: dir.join("sites"),
            ..Default::default()
        }
    }

    fn route(app_id: &str) -> DesiredRoute {
        DesiredRoute {
            app_id: app_id.to_string(),
            route_id: "web".to_string(),
            hosts: vec!["demo.local".to_string()],
            path: None,
            upstream: "127.0.0.1:8080".to_string(),
            source: RouteSource::ProjectConfig,
        }
    }

    fn label_container() -> ContainerSummary {
        ContainerSummary {
            scope: ContainerScope::Current,
            id: "abc".to_string(),
            name: "demo".to_string(),
            image: "nginx".to_string(),
            state: "running".to_string(),
            health: Some("healthy".to_string()),
            ports: vec![ContainerPort {
                host_ip: Some("127.0.0.1".to_string()),
                host_port: Some(8080),
                container_port: 80,
                protocol: "tcp".to_string(),
            }],
            labels: BTreeMap::from([
                ("cadman.enable".to_string(), "true".to_string()),
                ("cadman.id".to_string(), "demo".to_string()),
            ]),
        }
    }

    #[test]
    fn state_records_project_config_app() {
        let _lock = crate::TEST_ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let root = test_dir("project");
        let state_home = test_dir("project_state");
        let (_guard, runtime) = runtime(&state_home);
        let config_path =
            write_default_project_config(&root, ProjectConfigFormat::Toml, false).unwrap();
        let caddy_config = caddy_config(&root);
        let mut registry = Registry::empty();
        registry
            .add_app(RegistryApp {
                id: "demo".to_string(),
                name: "demo".to_string(),
                source: RegistrySource::ProjectConfig,
                project_path: Some(root.clone()),
                config_path: Some(config_path),
                desired_status: DesiredStatus::Down,
                managed: true,
                container_scope: None,
                container_id: None,
                container_name: None,
            })
            .unwrap();

        update_state(
            &runtime,
            &registry,
            &[],
            &[route("demo")],
            &caddy_config,
            &[],
        )
        .unwrap();

        let state = state::load(&runtime).unwrap();
        let app = state.get("demo").unwrap();
        assert!(app.config_hash.is_some());
        assert_eq!(app.routes.len(), 1);
        assert!(state.last_reconcile_at.is_some());

        let _ = std_fs::remove_dir_all(&root);
        let _ = std_fs::remove_dir_all(&state_home);
    }

    #[test]
    fn state_records_label_sourced_app() {
        let _lock = crate::TEST_ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let root = test_dir("label");
        let state_home = test_dir("label_state");
        let (_guard, runtime) = runtime(&state_home);
        let caddy_config = caddy_config(&root);
        let mut registry = Registry::empty();
        registry
            .add_app(RegistryApp {
                id: "demo".to_string(),
                name: "demo".to_string(),
                source: RegistrySource::PodmanLabels,
                project_path: None,
                config_path: None,
                desired_status: DesiredStatus::Down,
                managed: true,
                container_scope: Some(ContainerScope::Current),
                container_id: Some("abc".to_string()),
                container_name: Some("demo".to_string()),
            })
            .unwrap();
        let container = label_container();

        update_state(
            &runtime,
            &registry,
            &[container],
            &[route("demo")],
            &caddy_config,
            &[],
        )
        .unwrap();

        let state = state::load(&runtime).unwrap();
        let app = state.get("demo").unwrap();
        assert_eq!(app.container_state.as_deref(), Some("running"));
        assert!(app.labels_hash.is_some());
        assert!(app.ports_hash.is_some());

        let _ = std_fs::remove_dir_all(&root);
        let _ = std_fs::remove_dir_all(&state_home);
    }

    #[test]
    fn report_counts_include_site_actions_and_serializes() {
        let caddy = CaddyApplyReport {
            sites_dir: PathBuf::from("/sites"),
            changed: true,
            actions: vec![
                CaddySiteAction {
                    path: PathBuf::from("/sites/a.site"),
                    action: CaddySiteActionKind::Create,
                },
                CaddySiteAction {
                    path: PathBuf::from("/sites/b.site"),
                    action: CaddySiteActionKind::Unchanged,
                },
            ],
            validation: super::super::caddy::validate::CaddyCommandStatus::skipped(
                "caddy validate",
            ),
            reload: super::super::caddy::validate::CaddyCommandStatus::skipped("caddy reload"),
        };
        let counts = ReconcileCounts::from_report(2, &caddy);
        let report = ReconcileReport {
            dry_run: true,
            registry_path: PathBuf::from("/registry.toml"),
            desired_routes: vec![route("demo")],
            registered_apps: Vec::new(),
            updated_apps: Vec::new(),
            auto_registered_projects: Vec::new(),
            missing_label_apps: Vec::new(),
            caddy,
            counts,
            state_updated: false,
            warnings: Vec::new(),
        };

        assert_eq!(report.counts.routes_planned, 2);
        assert_eq!(report.counts.sites_created, 1);
        assert_eq!(report.counts.sites_unchanged, 1);
        assert!(serde_json::to_string(&report).is_ok());
    }
}
