use std::path::PathBuf;

use scriba::Output;
use serde::Serialize;

use crate::{
    core::{Context, CoreResult},
    engine::{
        ErrorCode,
        config::{ProjectConfigFormat, detect_project_config, write_default_project_config},
        system::fs,
    },
};

#[derive(Debug, Clone)]
pub struct InitRequest {
    pub yaml: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct InitReport {
    pub project_name: String,
    pub directory: PathBuf,
    pub config_path: PathBuf,
    pub format: String,
    pub overwritten: bool,
}

pub async fn run(ctx: &Context, request: InitRequest) -> CoreResult<()> {
    let report = init_project(ctx, request)?;
    render_report(ctx, &report)?;
    Ok(())
}

pub fn init_project(ctx: &Context, request: InitRequest) -> CoreResult<InitReport> {
    let target_dir = ctx.runtime().cwd().clone();
    let format = if request.yaml {
        ProjectConfigFormat::Yaml
    } else {
        ProjectConfigFormat::Toml
    };

    fs::create_dir_all(&target_dir)?;
    if !ctx.force() {
        if let Some(existing) = detect_project_config(&target_dir)? {
            return Err(ErrorCode::ConfigAlreadyExists
                .error()
                .with_context("path", existing.display().to_string())
                .with_context("hint", "use --force to overwrite"));
        }
    }

    let target_path = target_dir.join(format.file_name());
    let overwritten = target_path.exists();
    let config_path = write_default_project_config(&target_dir, format, ctx.force())?;

    Ok(InitReport {
        project_name: project_name(&target_dir),
        directory: target_dir,
        config_path,
        format: format.as_str().to_string(),
        overwritten,
    })
}

fn render_report(ctx: &Context, report: &InitReport) -> CoreResult<()> {
    let title = if report.overwritten {
        "Updated Cadman project config"
    } else {
        "Created Cadman project config"
    };

    let output = if ctx.runtime().options().output_format().is_structured()
        || ctx.runtime().options().output_envelope().is_json()
    {
        Output::from_serializable(report)
    } else {
        Output::new()
            .title(title)
            .key_value("Project", &report.project_name)
            .key_value("Directory", report.directory.display())
            .key_value("Config", report.config_path.display())
            .key_value("Format", &report.format)
            .key_value("Overwritten", report.overwritten)
    };

    ctx.ui().print(&output)
}

fn project_name(dir: &std::path::Path) -> String {
    dir.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("project")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs as std_fs;

    use crate::engine::{
        ErrorCode,
        constants::{PROJECT_CONFIG_FILE_NAME, PROJECT_CONFIG_FILE_NAME_YAML},
        models::runtime::Runtime,
    };

    fn test_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("cadman_init_{}_{}", std::process::id(), name));
        let _ = std_fs::remove_dir_all(&dir);
        dir
    }

    fn ctx_for(dir: PathBuf, force: bool) -> Context {
        let mut runtime = Runtime::new();
        runtime.set_cwd(dir).set_force(force);
        Context::new(runtime)
    }

    #[test]
    fn init_project_creates_toml_by_default() {
        let dir = test_dir("toml_default");
        let ctx = ctx_for(dir.clone(), false);

        let report = init_project(&ctx, InitRequest { yaml: false }).unwrap();

        assert_eq!(report.config_path, dir.join(PROJECT_CONFIG_FILE_NAME));
        assert_eq!(report.format, "toml");
        assert!(report.config_path.exists());

        let _ = std_fs::remove_dir_all(&dir);
    }

    #[test]
    fn init_project_creates_yaml() {
        let dir = test_dir("yaml");
        let ctx = ctx_for(dir.clone(), false);

        let report = init_project(&ctx, InitRequest { yaml: true }).unwrap();

        assert_eq!(report.config_path, dir.join(PROJECT_CONFIG_FILE_NAME_YAML));
        assert_eq!(report.format, "yaml");
        assert!(report.config_path.exists());

        let _ = std_fs::remove_dir_all(&dir);
    }

    #[test]
    fn init_project_returns_config_already_exists_without_force() {
        let dir = test_dir("exists");
        let ctx = ctx_for(dir.clone(), false);
        init_project(&ctx, InitRequest { yaml: false }).unwrap();

        let err = init_project(&ctx, InitRequest { yaml: false }).unwrap_err();
        assert_eq!(err.code, ErrorCode::ConfigAlreadyExists);

        let _ = std_fs::remove_dir_all(&dir);
    }

    #[test]
    fn init_project_overwrites_with_global_force() {
        let dir = test_dir("force");
        let ctx = ctx_for(dir.clone(), false);
        init_project(&ctx, InitRequest { yaml: false }).unwrap();
        std_fs::write(dir.join(PROJECT_CONFIG_FILE_NAME), "existing").unwrap();

        let force_ctx = ctx_for(dir.clone(), true);
        let report = init_project(&force_ctx, InitRequest { yaml: false }).unwrap();

        assert!(report.overwritten);
        assert_ne!(
            std_fs::read_to_string(dir.join(PROJECT_CONFIG_FILE_NAME)).unwrap(),
            "existing"
        );

        let _ = std_fs::remove_dir_all(&dir);
    }

    #[test]
    fn init_project_force_does_not_mark_new_file_overwritten() {
        let dir = test_dir("force_new");
        let ctx = ctx_for(dir.clone(), true);

        let report = init_project(&ctx, InitRequest { yaml: false }).unwrap();

        assert!(!report.overwritten);

        let _ = std_fs::remove_dir_all(&dir);
    }

    #[test]
    fn init_project_uses_runtime_cwd() {
        let dir = test_dir("cwd");
        let ctx = ctx_for(dir.clone(), false);

        let report = init_project(&ctx, InitRequest { yaml: false }).unwrap();

        assert_eq!(report.directory, dir);

        let _ = std_fs::remove_dir_all(&report.directory);
    }
}
