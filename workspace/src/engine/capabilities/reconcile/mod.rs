pub mod desired;
pub mod labels;
pub mod plan;
pub mod projects;
pub mod report;

use crate::engine::{
    ErrorCode, Result, config,
    models::runtime::Runtime,
    registry::{self, Registry, RegistryApp, RegistrySource},
};

pub use desired::{DesiredRoute, RouteSource};
pub use plan::ReconcileRequest;
pub use report::ReconcileReport;

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

    Ok(ReconcileReport {
        dry_run: request.dry_run,
        registry_path,
        desired_routes,
        registered_apps: label_result.registered_apps,
        updated_apps: label_result.updated_apps,
        auto_registered_projects,
        missing_label_apps,
        caddy,
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

fn missing_label_apps(
    registry: &Registry,
    containers: &[crate::engine::models::containers::ContainerSummary],
) -> Vec<String> {
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
