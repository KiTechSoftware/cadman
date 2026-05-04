use crate::engine::{
    ErrorCode, Result,
    capabilities::podman::labels as podman_labels,
    constants::LABEL_CADMAN_PATH,
    models::containers::ContainerSummary,
    registry::{DesiredStatus, Registry, RegistryApp, RegistrySource, sanitize_app_id},
};

use super::desired::{DesiredRoute, RouteSource};

#[derive(Debug, Clone)]
pub struct LabelRouteResult {
    pub routes: Vec<DesiredRoute>,
    pub registered_apps: Vec<RegistryApp>,
    pub updated_apps: Vec<RegistryApp>,
    pub registry_changed: bool,
}

pub fn collect_label_routes(
    containers: &[ContainerSummary],
    registry: &mut Registry,
    dry_run: bool,
) -> Result<LabelRouteResult> {
    let mut routes = Vec::new();
    let mut registered_apps = Vec::new();
    let mut updated_apps = Vec::new();
    let mut registry_changed = false;

    for container in containers
        .iter()
        .filter(|container| podman_labels::enabled(&container.labels))
    {
        let app = app_from_container(container);
        let app_id = app.id.clone();
        let existing = registry
            .get(&app.id)
            .or_else(|| registry.get(&app.name))
            .cloned();

        if let Some(mut existing) = existing {
            let changed = existing.container_id.as_ref() != Some(&container.id)
                || existing.container_name.as_ref() != Some(&container.name)
                || existing.container_scope != Some(container.scope);

            if changed && existing.source == RegistrySource::PodmanLabels {
                existing.container_id = Some(container.id.clone());
                existing.container_name = Some(container.name.clone());
                existing.container_scope = Some(container.scope);
                updated_apps.push(existing.clone());
                registry_changed = true;

                if !dry_run {
                    registry.remove_app(&existing.id)?;
                    registry.add_app(existing)?;
                }
            }
        } else {
            registered_apps.push(app.clone());
            registry_changed = true;
            if !dry_run {
                registry.add_app(app)?;
            }
        }

        routes.push(route_from_container(&app_id, container)?);
    }

    Ok(LabelRouteResult {
        routes,
        registered_apps,
        updated_apps,
        registry_changed,
    })
}

fn app_from_container(container: &ContainerSummary) -> RegistryApp {
    let id = container
        .labels
        .get("cadman.id")
        .map(|value| sanitize_app_id(value))
        .unwrap_or_else(|| sanitize_app_id(&container.name));
    let name = container
        .labels
        .get("cadman.name")
        .cloned()
        .unwrap_or_else(|| container.name.clone());

    RegistryApp {
        id,
        name,
        source: RegistrySource::PodmanLabels,
        project_path: None,
        config_path: None,
        desired_status: DesiredStatus::Down,
        managed: true,
        container_scope: Some(container.scope),
        container_id: Some(container.id.clone()),
        container_name: Some(container.name.clone()),
    }
}

fn route_from_container(app_id: &str, container: &ContainerSummary) -> Result<DesiredRoute> {
    let service =
        podman_labels::service(&container.labels).unwrap_or_else(|| container.name.clone());
    let port = podman_labels::service_port(&container.labels, &service).ok_or_else(|| {
        ErrorCode::ValidationRouteInvalid
            .error()
            .with_context("container", &container.name)
            .with_context("field", "cadman.port")
            .with_context("reason", "labeled container route requires a service port")
    })?;
    let hosts = podman_labels::hosts(&container.labels);
    if hosts.is_empty() {
        return Err(ErrorCode::ValidationRouteInvalid
            .error()
            .with_context("container", &container.name)
            .with_context("field", "cadman.host")
            .with_context(
                "reason",
                "labeled container route requires at least one host",
            ));
    }

    let host_port = container
        .ports
        .iter()
        .find(|mapping| mapping.container_port == port)
        .and_then(|mapping| mapping.host_port)
        .ok_or_else(|| {
            ErrorCode::ValidationPortInvalid
                .error()
                .with_context("container", &container.name)
                .with_context("container_port", port.to_string())
                .with_context(
                    "reason",
                    "container port must be published for loopback upstream",
                )
        })?;

    Ok(DesiredRoute {
        app_id: app_id.to_string(),
        route_id: service,
        hosts,
        path: container.labels.get(LABEL_CADMAN_PATH).cloned(),
        upstream: format!("127.0.0.1:{host_port}"),
        source: RouteSource::PodmanLabels,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    use crate::engine::models::containers::{ContainerPort, ContainerScope};

    fn container(labels: BTreeMap<String, String>) -> ContainerSummary {
        ContainerSummary {
            scope: ContainerScope::Current,
            id: "abc".to_string(),
            name: "cadman-demo".to_string(),
            image: "nginx".to_string(),
            state: "running".to_string(),
            health: None,
            ports: vec![ContainerPort {
                host_ip: Some("127.0.0.1".to_string()),
                host_port: Some(8080),
                container_port: 80,
                protocol: "tcp".to_string(),
            }],
            labels,
        }
    }

    #[test]
    fn simple_labels_create_desired_route() {
        let mut registry = Registry::empty();
        let result = collect_label_routes(
            &[container(BTreeMap::from([
                ("cadman.enable".to_string(), "true".to_string()),
                ("cadman.host".to_string(), "demo.local".to_string()),
                ("cadman.port".to_string(), "80".to_string()),
            ]))],
            &mut registry,
            false,
        )
        .unwrap();

        assert_eq!(result.routes.len(), 1);
        assert_eq!(result.routes[0].hosts, vec!["demo.local"]);
        assert_eq!(result.routes[0].upstream, "127.0.0.1:8080");
        assert!(registry.get("cadman-demo").is_some());
    }

    #[test]
    fn multi_host_labels_create_desired_route() {
        let mut registry = Registry::empty();
        let result = collect_label_routes(
            &[container(BTreeMap::from([
                ("cadman.enable".to_string(), "true".to_string()),
                (
                    "cadman.host".to_string(),
                    "demo.local,www.demo.local".to_string(),
                ),
                ("cadman.port".to_string(), "80".to_string()),
            ]))],
            &mut registry,
            false,
        )
        .unwrap();

        assert_eq!(result.routes[0].hosts, vec!["demo.local", "www.demo.local"]);
    }

    #[test]
    fn router_style_labels_create_desired_route() {
        let mut registry = Registry::empty();
        let result = collect_label_routes(
            &[container(BTreeMap::from([
                ("cadman.enable".to_string(), "true".to_string()),
                (
                    "cadman.http.routers.web.rule".to_string(),
                    "Host(`demo.local`,`www.demo.local`)".to_string(),
                ),
                (
                    "cadman.http.routers.web.service".to_string(),
                    "web".to_string(),
                ),
                (
                    "cadman.http.services.web.loadbalancer.server.port".to_string(),
                    "80".to_string(),
                ),
            ]))],
            &mut registry,
            false,
        )
        .unwrap();

        assert_eq!(result.routes[0].route_id, "web");
        assert_eq!(result.routes[0].hosts, vec!["demo.local", "www.demo.local"]);
    }

    #[test]
    fn labeled_container_is_added_to_registry() {
        let mut registry = Registry::empty();
        collect_label_routes(
            &[container(BTreeMap::from([
                ("cadman.enable".to_string(), "true".to_string()),
                ("cadman.id".to_string(), "demo-app".to_string()),
                ("cadman.name".to_string(), "Demo App".to_string()),
                ("cadman.host".to_string(), "demo.local".to_string()),
                ("cadman.port".to_string(), "80".to_string()),
            ]))],
            &mut registry,
            false,
        )
        .unwrap();

        let app = registry.get("demo-app").unwrap();
        assert_eq!(app.name, "Demo App");
        assert_eq!(app.source, RegistrySource::PodmanLabels);
    }
}
