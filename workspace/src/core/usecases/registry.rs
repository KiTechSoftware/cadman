use std::path::PathBuf;

use scriba::Output;
use serde::Serialize;

use crate::{
    core::{Context, CoreResult},
    engine::{
        ErrorCode,
        registry::{self, RegistryApp},
    },
};

#[derive(Debug, Clone, Serialize)]
pub struct RegistryPathReport {
    pub path: PathBuf,
    pub install_scope: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegistryListReport {
    pub path: PathBuf,
    pub apps: Vec<RegistryApp>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegistryShowReport {
    pub path: PathBuf,
    pub app: RegistryApp,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegistryRemoveReport {
    pub path: PathBuf,
    pub removed: RegistryApp,
}

pub fn registry_path(ctx: &Context) -> CoreResult<RegistryPathReport> {
    Ok(RegistryPathReport {
        path: registry::path(ctx.runtime()),
        install_scope: ctx.runtime().install_scope().to_string(),
    })
}

pub fn registry_list(ctx: &Context) -> CoreResult<RegistryListReport> {
    let path = registry::path(ctx.runtime());
    let registry = registry::load(ctx.runtime())?;
    let apps = registry.list().into_iter().cloned().collect();

    Ok(RegistryListReport { path, apps })
}

pub fn registry_show(ctx: &Context, app: &str) -> CoreResult<RegistryShowReport> {
    let path = registry::path(ctx.runtime());
    let registry = registry::load(ctx.runtime())?;
    let app = registry.get(app).cloned().ok_or_else(|| {
        ErrorCode::RegistryAppNotFound
            .error()
            .with_context("app", app)
    })?;

    Ok(RegistryShowReport { path, app })
}

pub fn registry_remove(ctx: &Context, app: &str) -> CoreResult<RegistryRemoveReport> {
    let path = registry::path(ctx.runtime());
    let mut registry = registry::load(ctx.runtime())?;
    let removed = registry.remove_app(app)?;
    registry::save(ctx.runtime(), &registry)?;

    Ok(RegistryRemoveReport { path, removed })
}

pub async fn render_path(ctx: &Context) -> CoreResult<()> {
    let report = registry_path(ctx)?;
    let output = if structured(ctx) {
        Output::from_serializable(report)
    } else {
        Output::new()
            .title("Cadman registry path")
            .key_value("Path", report.path.display())
            .key_value("Install Scope", &report.install_scope)
    };

    ctx.ui().print(&output)
}

pub async fn render_list(ctx: &Context) -> CoreResult<()> {
    let report = registry_list(ctx)?;
    let output = if structured(ctx) {
        Output::from_serializable(&report)
    } else {
        let output = Output::new()
            .title("Cadman registry")
            .key_value("Path", report.path.display())
            .key_value("Apps", report.apps.len());

        if report.apps.is_empty() {
            output
        } else {
            output.table(None, apps_table(&report.apps))
        }
    };

    ctx.ui().print(&output)
}

pub async fn render_show(ctx: &Context, app: &str) -> CoreResult<()> {
    let report = registry_show(ctx, app)?;
    let output = if structured(ctx) {
        Output::from_serializable(&report)
    } else {
        app_output("Cadman registry app", report.path, &report.app)
    };

    ctx.ui().print(&output)
}

pub async fn render_remove(ctx: &Context, app: &str) -> CoreResult<()> {
    let report = registry_remove(ctx, app)?;
    let output = if structured(ctx) {
        Output::from_serializable(&report)
    } else {
        app_output("Removed registry app", report.path, &report.removed)
    };

    ctx.ui().print(&output)
}

fn app_output(title: &str, path: PathBuf, app: &RegistryApp) -> Output {
    Output::new()
        .title(title)
        .key_value("Registry", path.display())
        .key_value("ID", &app.id)
        .key_value("Name", &app.name)
        .key_value("Project", app.project_path.display())
        .key_value("Config", app.config_path.display())
        .key_value("Desired", format!("{:?}", app.desired_status))
        .key_value("Managed", app.managed)
}

fn apps_table(apps: &[RegistryApp]) -> scriba::Table {
    let rows = apps
        .iter()
        .map(|app| {
            vec![
                app.id.clone(),
                app.name.clone(),
                format!("{:?}", app.desired_status),
                app.managed.to_string(),
                app.project_path.display().to_string(),
            ]
        })
        .collect();

    scriba::Table::new(
        vec![
            "ID".to_string(),
            "NAME".to_string(),
            "DESIRED".to_string(),
            "MANAGED".to_string(),
            "PROJECT".to_string(),
        ],
        rows,
    )
}

fn structured(ctx: &Context) -> bool {
    ctx.runtime().options().output_format().is_structured()
        || ctx.runtime().options().output_envelope().is_json()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{
        models::runtime::Runtime,
        registry::{DesiredStatus, Registry},
    };

    fn app(id: &str, name: &str) -> RegistryApp {
        RegistryApp {
            id: id.to_string(),
            name: name.to_string(),
            project_path: PathBuf::from(format!("/projects/{id}")),
            config_path: PathBuf::from(format!("/projects/{id}/cadman.toml")),
            desired_status: DesiredStatus::Down,
            managed: true,
        }
    }

    #[test]
    fn registry_path_uses_runtime_registry_path() {
        let ctx = Context::new(Runtime::new());

        let report = registry_path(&ctx).unwrap();

        assert!(report.path.ends_with("registry.toml"));
    }

    #[test]
    fn registry_get_by_id_or_name() {
        let mut registry = Registry::empty();
        registry.add_app(app("web", "Website")).unwrap();

        assert_eq!(registry.get("web").unwrap().name, "Website");
        assert_eq!(registry.get("Website").unwrap().id, "web");
    }

    #[test]
    fn registry_remove_missing_app_returns_registry_app_not_found() {
        let mut registry = Registry::empty();

        let err = registry.remove_app("missing").unwrap_err();

        assert_eq!(err.code, ErrorCode::RegistryAppNotFound);
    }

    #[test]
    fn registry_list_on_empty_registry_is_valid() {
        let registry = Registry::empty();

        assert!(registry.list().is_empty());
    }
}
