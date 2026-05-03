use std::path::PathBuf;

use serde::Serialize;

use crate::{
    core::{Context, CoreResult},
    engine::{
        config::{
            self, DiscoveredProject, ProjectDiscoveryReport, ProjectDiscoveryRequest,
            discover_project_configs,
        },
        registry::{self, DesiredStatus, RegistryApp, RegistrySource},
    },
};

#[derive(Debug, Clone)]
pub struct ScanRequest {
    pub paths: Vec<PathBuf>,
    pub add: bool,
    pub max_depth: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanReport {
    pub discovery: ProjectDiscoveryReport,
    pub registry_path: Option<PathBuf>,
    pub added: Vec<ScanAddedApp>,
    pub skipped_apps: Vec<ScanAppSkip>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanAddedApp {
    pub id: String,
    pub name: String,
    pub project_path: PathBuf,
    pub config_path: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanAppSkip {
    pub project_path: PathBuf,
    pub config_path: PathBuf,
    pub reason: String,
}

pub async fn render(ctx: &Context, request: ScanRequest) -> CoreResult<()> {
    let report = scan(ctx, request)?;
    let mut output = ctx
        .ui()
        .new_output_content()
        .json(&report)
        .title("Cadman scan")
        .key_value("Roots", format_paths(&report.discovery.scanned_roots))
        .key_value("Projects", report.discovery.projects.len())
        .key_value("Skipped", report.discovery.skipped.len())
        .key_value("Added", report.added.len())
        .key_value("App skips", report.skipped_apps.len());

    if !report.discovery.projects.is_empty() {
        output = output.table(None, projects_table(&report.discovery.projects));
    }

    ctx.ui().print(&output)
}

pub fn scan(ctx: &Context, request: ScanRequest) -> CoreResult<ScanReport> {
    let config = config::load(ctx.runtime())?;
    let roots = scan_roots(ctx, &config.discovery.roots, &request.paths)?;
    let discovery_request = ProjectDiscoveryRequest {
        roots,
        max_depth: request.max_depth.unwrap_or(config.discovery.max_depth),
        follow_symlinks: config.discovery.follow_symlinks,
        include_hidden: config.discovery.include_hidden,
    };
    let discovery = discover_project_configs(discovery_request)?;

    if request.add {
        add_discovered_projects(ctx, discovery)
    } else {
        Ok(ScanReport {
            discovery,
            registry_path: None,
            added: Vec::new(),
            skipped_apps: Vec::new(),
        })
    }
}

fn scan_roots(
    ctx: &Context,
    configured_roots: &[PathBuf],
    explicit_paths: &[PathBuf],
) -> CoreResult<Vec<PathBuf>> {
    let raw_roots = if !explicit_paths.is_empty() {
        explicit_paths.to_vec()
    } else if !configured_roots.is_empty() {
        configured_roots.to_vec()
    } else {
        vec![ctx.runtime().cwd().clone()]
    };

    raw_roots
        .iter()
        .map(|root| config::expand_discovery_path(root))
        .collect()
}

fn add_discovered_projects(
    ctx: &Context,
    discovery: ProjectDiscoveryReport,
) -> CoreResult<ScanReport> {
    let registry_path = registry::path(ctx.runtime());
    let mut registry = registry::load(ctx.runtime())?;
    let mut added = Vec::new();
    let mut skipped_apps = Vec::new();

    for project in &discovery.projects {
        let app = registry_app(project);
        let duplicate = registry.apps.contains_key(&app.id)
            || registry
                .apps
                .values()
                .any(|existing| existing.name == app.name);

        if duplicate && !ctx.force() {
            skipped_apps.push(ScanAppSkip {
                project_path: project.root.clone(),
                config_path: project.config_path.clone(),
                reason: "duplicate app id or name".to_string(),
            });
            continue;
        }

        if duplicate {
            if let Some(existing_id) = registry.get(&app.id).map(|existing| existing.id.clone()) {
                let _ = registry.remove_app(&existing_id)?;
            }
            if let Some(existing_id) = registry.get(&app.name).map(|existing| existing.id.clone()) {
                let _ = registry.remove_app(&existing_id)?;
            }
        }

        registry.add_app(app.clone())?;
        added.push(ScanAddedApp {
            id: app.id,
            name: app.name,
            project_path: app
                .project_path
                .expect("project-config registry app has project_path"),
            config_path: app
                .config_path
                .expect("project-config registry app has config_path"),
        });
    }

    if !added.is_empty() {
        registry::save(ctx.runtime(), &registry)?;
    }

    Ok(ScanReport {
        discovery,
        registry_path: Some(registry_path),
        added,
        skipped_apps,
    })
}

fn registry_app(project: &DiscoveredProject) -> RegistryApp {
    RegistryApp {
        id: registry::sanitize_app_id(&project.project_name),
        name: project.project_name.clone(),
        source: RegistrySource::ProjectConfig,
        project_path: Some(project.root.clone()),
        config_path: Some(project.config_path.clone()),
        desired_status: DesiredStatus::Down,
        managed: true,
        container_scope: None,
        container_id: None,
        container_name: None,
    }
}

fn projects_table(projects: &[DiscoveredProject]) -> scriba::Table {
    let rows = projects
        .iter()
        .map(|project| {
            vec![
                project.project_name.clone(),
                project.format.clone(),
                project.root.display().to_string(),
                project.config_path.display().to_string(),
            ]
        })
        .collect();

    scriba::Table::new(
        vec![
            "NAME".to_string(),
            "FORMAT".to_string(),
            "ROOT".to_string(),
            "CONFIG".to_string(),
        ],
        rows,
    )
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
    use std::sync::Mutex;

    use crate::engine::{
        config::{ProjectConfigFormat, write_default_project_config},
        models::runtime::{InstallScope, Runtime},
        registry,
    };

    static ENV_LOCK: Mutex<()> = Mutex::new(());

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
        let dir = std::env::temp_dir().join(format!("cadman_scan_{}_{}", std::process::id(), name));
        let _ = std_fs::remove_dir_all(&dir);
        dir
    }

    fn ctx_for(cwd: PathBuf, state_home: &std::path::Path, force: bool) -> (EnvGuard, Context) {
        let guard = EnvGuard::set_state_home(state_home);
        let mut runtime = Runtime::new();
        runtime
            .set_install_scope(InstallScope::User)
            .set_cwd(cwd)
            .set_force(force);
        (guard, Context::new(runtime))
    }

    #[test]
    fn scan_add_registers_discovered_projects() {
        let _lock = ENV_LOCK.lock().unwrap();
        let root = test_dir("add");
        let state_home = test_dir("add_state");
        let app = root.join("web");
        write_default_project_config(&app, ProjectConfigFormat::Toml, false).unwrap();
        let (_guard, ctx) = ctx_for(root.clone(), &state_home, false);

        let report = scan(
            &ctx,
            ScanRequest {
                paths: Vec::new(),
                add: true,
                max_depth: None,
            },
        )
        .unwrap();

        assert_eq!(report.added.len(), 1);
        assert!(
            registry::load(ctx.runtime())
                .unwrap()
                .get(&report.added[0].id)
                .is_some()
        );

        let _ = std_fs::remove_dir_all(&root);
        let _ = std_fs::remove_dir_all(&state_home);
    }

    #[test]
    fn scan_add_does_not_duplicate_apps() {
        let _lock = ENV_LOCK.lock().unwrap();
        let root = test_dir("dupe");
        let state_home = test_dir("dupe_state");
        let app = root.join("web");
        write_default_project_config(&app, ProjectConfigFormat::Toml, false).unwrap();
        let (_guard, ctx) = ctx_for(root.clone(), &state_home, false);
        let request = ScanRequest {
            paths: Vec::new(),
            add: true,
            max_depth: None,
        };
        scan(&ctx, request.clone()).unwrap();

        let report = scan(&ctx, request).unwrap();

        assert!(report.added.is_empty());
        assert_eq!(report.skipped_apps.len(), 1);

        let _ = std_fs::remove_dir_all(&root);
        let _ = std_fs::remove_dir_all(&state_home);
    }
}
