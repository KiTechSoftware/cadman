use std::path::PathBuf;

use serde::Serialize;

use crate::{
    core::{Context, CoreResult},
    engine::{
        ErrorCode,
        config::{self, ProjectConfig},
        registry::{self, DesiredStatus, RegistryApp, RegistrySource},
    },
};

#[derive(Debug, Clone)]
pub struct AddRequest {
    pub name: Option<String>,
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AddReport {
    pub id: String,
    pub name: String,
    pub project_path: PathBuf,
    pub config_path: PathBuf,
    pub registry_path: PathBuf,
    pub desired_status: String,
    pub managed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct RemoveReport {
    pub id: String,
    pub name: String,
    pub registry_path: PathBuf,
}

pub async fn add(ctx: &Context, name: Option<String>, id: Option<String>) -> CoreResult<()> {
    let request = AddRequest { name, id };
    let report = add_app(ctx, request)?;
    render_add(ctx, &report)
}

pub async fn remove(ctx: &Context, app: &str) -> CoreResult<()> {
    let report = remove_app(ctx, app)?;
    render_remove(ctx, &report)
}

pub fn add_app(ctx: &Context, request: AddRequest) -> CoreResult<AddReport> {
    let project_path = ctx.runtime().cwd().clone();
    let config_path = config::detect_project_config(&project_path)?.ok_or_else(|| {
        ErrorCode::ConfigNotFound
            .error()
            .with_context("directory", project_path.display().to_string())
            .with_context("hint", "run cadman init first")
    })?;
    let project = config::load_project_config(&config_path)?;
    config::validate_project_config(&project)?;

    let app = registry_app_from_project(project_path, config_path, project, request);
    let registry_path = registry::path(ctx.runtime());
    let mut registry = registry::load(ctx.runtime())?;
    registry.add_app(app.clone())?;
    registry::save(ctx.runtime(), &registry)?;

    Ok(AddReport {
        id: app.id,
        name: app.name,
        project_path: app
            .project_path
            .expect("project-config registry app has project_path"),
        config_path: app
            .config_path
            .expect("project-config registry app has config_path"),
        registry_path,
        desired_status: desired_status_string(app.desired_status),
        managed: app.managed,
    })
}

pub fn remove_app(ctx: &Context, app: &str) -> CoreResult<RemoveReport> {
    let registry_path = registry::path(ctx.runtime());
    let mut registry = registry::load(ctx.runtime())?;
    let removed = registry.remove_app(app)?;
    registry::save(ctx.runtime(), &registry)?;

    Ok(RemoveReport {
        id: removed.id,
        name: removed.name,
        registry_path,
    })
}

fn registry_app_from_project(
    project_path: PathBuf,
    config_path: PathBuf,
    project: ProjectConfig,
    request: AddRequest,
) -> RegistryApp {
    let project_name = project.project.name;
    let name = request
        .name
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| project_name.clone());
    let id = request
        .id
        .filter(|value| !value.trim().is_empty())
        .map(|value| registry::sanitize_app_id(&value))
        .unwrap_or_else(|| registry::sanitize_app_id(&project_name));

    RegistryApp {
        id,
        name,
        source: RegistrySource::ProjectConfig,
        project_path: Some(project_path),
        config_path: Some(config_path),
        desired_status: DesiredStatus::Down,
        managed: true,
        container_scope: None,
        container_id: None,
        container_name: None,
    }
}

fn render_add(ctx: &Context, report: &AddReport) -> CoreResult<()> {
    let output = ctx
        .ui()
        .new_output_content()
        .json(report)
        .title("Added Cadman app")
        .key_value("ID", &report.id)
        .key_value("Name", &report.name)
        .key_value("Project", report.project_path.display())
        .key_value("Config", report.config_path.display())
        .key_value("Registry", report.registry_path.display())
        .key_value("Desired", &report.desired_status)
        .key_value("Managed", report.managed);

    ctx.ui().print(&output)
}

fn render_remove(ctx: &Context, report: &RemoveReport) -> CoreResult<()> {
    let output = ctx
        .ui()
        .new_output_content()
        .json(report)
        .title("Removed Cadman app")
        .key_value("ID", &report.id)
        .key_value("Name", &report.name)
        .key_value("Registry", report.registry_path.display());

    ctx.ui().print(&output)
}

fn desired_status_string(status: DesiredStatus) -> String {
    match status {
        DesiredStatus::Up => "up".to_string(),
        DesiredStatus::Down => "down".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs as std_fs;

    use crate::engine::{
        config::{ProjectConfigFormat, write_default_project_config},
        models::runtime::{InstallScope, Runtime},
        registry,
    };

    fn test_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("cadman_apps_{}_{}", std::process::id(), name));
        let _ = std_fs::remove_dir_all(&dir);
        dir
    }

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

    fn ctx_for(dir: PathBuf) -> Context {
        let mut runtime = Runtime::new();
        runtime.set_install_scope(InstallScope::User).set_cwd(dir);
        Context::new(runtime)
    }

    fn guarded_ctx_for(dir: PathBuf, state_home: &std::path::Path) -> (EnvGuard, Context) {
        let guard = EnvGuard::set_state_home(state_home);
        let ctx = ctx_for(dir);
        (guard, ctx)
    }

    #[test]
    fn add_app_registers_project_config() {
        let _guard = crate::TEST_ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let dir = test_dir("add");
        let registry_dir = test_dir("add_registry");
        write_default_project_config(&dir, ProjectConfigFormat::Toml, false).unwrap();
        let (_env_guard, ctx) = guarded_ctx_for(dir.clone(), &registry_dir);

        let report = add_app(
            &ctx,
            AddRequest {
                name: None,
                id: None,
            },
        )
        .unwrap();

        assert_eq!(report.name, dir.file_name().unwrap().to_string_lossy());
        assert_eq!(report.desired_status, "down");
        assert!(report.managed);
        assert!(
            registry::load(ctx.runtime())
                .unwrap()
                .get(&report.id)
                .is_some()
        );

        let _ = std_fs::remove_dir_all(&dir);
        let _ = std_fs::remove_dir_all(&registry_dir);
    }

    #[test]
    fn add_app_honors_name_and_id_overrides() {
        let _guard = crate::TEST_ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let dir = test_dir("overrides");
        let registry_dir = test_dir("overrides_registry");
        write_default_project_config(&dir, ProjectConfigFormat::Toml, false).unwrap();
        let (_env_guard, ctx) = guarded_ctx_for(dir.clone(), &registry_dir);

        let report = add_app(
            &ctx,
            AddRequest {
                name: Some("Display Name".to_string()),
                id: Some("Custom ID".to_string()),
            },
        )
        .unwrap();

        assert_eq!(report.name, "Display Name");
        assert_eq!(report.id, "custom-id");

        let _ = std_fs::remove_dir_all(&dir);
        let _ = std_fs::remove_dir_all(&registry_dir);
    }

    #[test]
    fn add_app_rejects_duplicate_id() {
        let _guard = crate::TEST_ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let dir = test_dir("duplicate");
        let registry_dir = test_dir("duplicate_registry");
        write_default_project_config(&dir, ProjectConfigFormat::Toml, false).unwrap();
        let (_env_guard, ctx) = guarded_ctx_for(dir.clone(), &registry_dir);
        add_app(
            &ctx,
            AddRequest {
                name: None,
                id: None,
            },
        )
        .unwrap();

        let err = add_app(
            &ctx,
            AddRequest {
                name: None,
                id: None,
            },
        )
        .unwrap_err();

        assert_eq!(err.code, ErrorCode::RegistryAppAlreadyExists);

        let _ = std_fs::remove_dir_all(&dir);
        let _ = std_fs::remove_dir_all(&registry_dir);
    }

    #[test]
    fn remove_app_deletes_registry_entry() {
        let _guard = crate::TEST_ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let dir = test_dir("remove");
        let registry_dir = test_dir("remove_registry");
        write_default_project_config(&dir, ProjectConfigFormat::Toml, false).unwrap();
        let (_env_guard, ctx) = guarded_ctx_for(dir.clone(), &registry_dir);
        let added = add_app(
            &ctx,
            AddRequest {
                name: None,
                id: None,
            },
        )
        .unwrap();

        let removed = remove_app(&ctx, &added.id).unwrap();

        assert_eq!(removed.id, added.id);
        assert!(
            registry::load(ctx.runtime())
                .unwrap()
                .get(&added.id)
                .is_none()
        );

        let _ = std_fs::remove_dir_all(&dir);
        let _ = std_fs::remove_dir_all(&registry_dir);
    }

    #[test]
    fn remove_missing_app_returns_registry_app_not_found() {
        let _guard = crate::TEST_ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let dir = test_dir("missing");
        let registry_dir = test_dir("missing_registry");
        let (_env_guard, ctx) = guarded_ctx_for(dir.clone(), &registry_dir);

        let err = remove_app(&ctx, "missing").unwrap_err();

        assert_eq!(err.code, ErrorCode::RegistryAppNotFound);

        let _ = std_fs::remove_dir_all(&dir);
        let _ = std_fs::remove_dir_all(&registry_dir);
    }
}
