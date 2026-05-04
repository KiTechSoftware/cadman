use crate::engine::{
    ErrorCode, Result, config,
    registry::{Registry, RegistrySource},
};

use super::desired::{DesiredRoute, RouteSource};

pub fn collect_project_routes(registry: &Registry) -> Result<Vec<DesiredRoute>> {
    let mut routes = Vec::new();

    for app in registry
        .list()
        .into_iter()
        .filter(|app| app.source == RegistrySource::ProjectConfig)
    {
        let config_path = app.config_path.as_ref().ok_or_else(|| {
            ErrorCode::RegistryInvalid
                .error()
                .with_context("app", &app.id)
                .with_context("reason", "project-config app is missing config_path")
        })?;
        let project = config::load_project_config(config_path)?;

        for (route_id, route) in &project.routes {
            let container = project.containers.get(&route.container).ok_or_else(|| {
                ErrorCode::ValidationRouteInvalid
                    .error()
                    .with_context("app", &app.id)
                    .with_context("route", route_id)
                    .with_context("field", "container")
                    .with_context("reason", "route references an unknown container")
            })?;

            let host_port = container
                .ports
                .iter()
                .find_map(|mapping| host_port_for_mapping(mapping, route.upstream_port))
                .ok_or_else(|| {
                    ErrorCode::ValidationPortInvalid
                        .error()
                        .with_context("app", &app.id)
                        .with_context("route", route_id)
                        .with_context("container", &route.container)
                        .with_context("container_port", route.upstream_port.to_string())
                        .with_context(
                            "reason",
                            "route container port must be published for loopback upstream",
                        )
                })?;

            routes.push(DesiredRoute {
                app_id: app.id.clone(),
                route_id: route_id.clone(),
                hosts: route.hosts.clone(),
                path: route.path.clone(),
                upstream: format!("127.0.0.1:{host_port}"),
                source: RouteSource::ProjectConfig,
            });
        }
    }

    Ok(routes)
}

fn host_port_for_mapping(mapping: &str, upstream_port: u16) -> Option<u16> {
    let mapping = mapping.split('/').next().unwrap_or(mapping);
    let parts: Vec<&str> = mapping.split(':').collect();

    match parts.as_slice() {
        [host, container] if container.parse::<u16>().ok() == Some(upstream_port) => {
            host.parse().ok()
        }
        [_ip, host, container] if container.parse::<u16>().ok() == Some(upstream_port) => {
            host.parse().ok()
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs as std_fs;

    use crate::engine::{
        config::{ProjectConfigFormat, write_default_project_config},
        registry::{DesiredStatus, RegistryApp},
    };

    fn test_dir(name: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("cadman_reconcile_{}_{}", std::process::id(), name));
        let _ = std_fs::remove_dir_all(&dir);
        dir
    }

    fn app(id: &str, config_path: std::path::PathBuf) -> RegistryApp {
        RegistryApp {
            id: id.to_string(),
            name: id.to_string(),
            source: RegistrySource::ProjectConfig,
            project_path: config_path.parent().map(std::path::Path::to_path_buf),
            config_path: Some(config_path),
            desired_status: DesiredStatus::Down,
            managed: true,
            container_scope: None,
            container_id: None,
            container_name: None,
        }
    }

    #[test]
    fn project_config_routes_create_desired_routes() {
        let dir = test_dir("project_routes");
        let config_path =
            write_default_project_config(&dir, ProjectConfigFormat::Toml, false).unwrap();
        let mut registry = Registry::empty();
        registry.add_app(app("demo", config_path)).unwrap();

        let routes = collect_project_routes(&registry).unwrap();

        assert_eq!(routes.len(), 2);
        assert!(
            routes
                .iter()
                .any(|route| route.upstream == "127.0.0.1:8080")
        );

        let _ = std_fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_project_config_returns_structured_error() {
        let mut registry = Registry::empty();
        registry
            .add_app(app(
                "demo",
                std::path::PathBuf::from("/missing/cadman.toml"),
            ))
            .unwrap();

        let err = collect_project_routes(&registry).unwrap_err();

        assert_eq!(err.code, ErrorCode::ConfigNotFound);
    }

    #[test]
    fn missing_route_container_returns_validation_error() {
        let dir = test_dir("missing_container");
        std_fs::create_dir_all(&dir).unwrap();
        let config_path = dir.join("cadman.toml");
        std_fs::write(
            &config_path,
            r#"
[project]
name = "demo"

[containers.web]
image = "nginx"
ports = ["127.0.0.1:8080:80"]

[routes.web]
hosts = ["demo.local"]
container = "missing"
upstream_port = 80
"#,
        )
        .unwrap();
        let mut registry = Registry::empty();
        registry.add_app(app("demo", config_path)).unwrap();

        let err = collect_project_routes(&registry).unwrap_err();

        assert_eq!(err.code, ErrorCode::ValidationRouteInvalid);
        let _ = std_fs::remove_dir_all(&dir);
    }
}
