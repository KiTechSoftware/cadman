use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use serde::Serialize;

use crate::engine::{
    ErrorCode, Result,
    config::{detect_project_config, load_project_config},
};

#[derive(Debug, Clone, Serialize)]
pub struct ProjectDiscoveryRequest {
    pub roots: Vec<PathBuf>,
    pub max_depth: usize,
    pub follow_symlinks: bool,
    pub include_hidden: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiscoveredProject {
    pub root: PathBuf,
    pub config_path: PathBuf,
    pub format: String,
    pub project_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectDiscoveryReport {
    pub scanned_roots: Vec<PathBuf>,
    pub projects: Vec<DiscoveredProject>,
    pub skipped: Vec<ProjectDiscoverySkip>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectDiscoverySkip {
    pub path: PathBuf,
    pub reason: String,
}

pub fn discover_project_configs(
    request: ProjectDiscoveryRequest,
) -> Result<ProjectDiscoveryReport> {
    let mut report = ProjectDiscoveryReport {
        scanned_roots: request.roots.clone(),
        projects: Vec::new(),
        skipped: Vec::new(),
    };
    let mut seen = BTreeSet::new();

    for root in &request.roots {
        if !root.exists() {
            return Err(ErrorCode::ConfigNotFound
                .error()
                .with_context("path", root.display().to_string()));
        }
        if !root.is_dir() {
            return Err(ErrorCode::ConfigInvalid
                .error()
                .with_context("path", root.display().to_string())
                .with_context("reason", "scan root is not a directory"));
        }

        walk_root(root, 0, &request, &mut report, &mut seen)?;
    }

    Ok(report)
}

fn walk_root(
    dir: &Path,
    depth: usize,
    request: &ProjectDiscoveryRequest,
    report: &mut ProjectDiscoveryReport,
    seen: &mut BTreeSet<PathBuf>,
) -> Result<()> {
    if !request.include_hidden && depth > 0 && is_hidden(dir) {
        return Ok(());
    }

    if let Some(config_path) = detect_project_config(dir)? {
        let project_dir_key = canonical_or_self(dir);
        if seen.insert(project_dir_key) {
            match load_project_config(&config_path) {
                Ok(config) => report.projects.push(DiscoveredProject {
                    root: dir.to_path_buf(),
                    format: config_format(&config_path).to_string(),
                    config_path,
                    project_name: config.project.name,
                }),
                Err(err) => report.skipped.push(ProjectDiscoverySkip {
                    path: config_path,
                    reason: err.to_string(),
                }),
            }
        }
        return Ok(());
    }

    if depth >= request.max_depth {
        return Ok(());
    }

    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) => {
            report.skipped.push(ProjectDiscoverySkip {
                path: dir.to_path_buf(),
                reason: format!("unable to read directory: {err}"),
            });
            return Ok(());
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                report.skipped.push(ProjectDiscoverySkip {
                    path: dir.to_path_buf(),
                    reason: format!("unable to read directory entry: {err}"),
                });
                continue;
            }
        };

        let path = entry.path();
        let metadata = if request.follow_symlinks {
            fs::metadata(&path)
        } else {
            fs::symlink_metadata(&path)
        };
        let Ok(metadata) = metadata else {
            report.skipped.push(ProjectDiscoverySkip {
                path,
                reason: "unable to read metadata".to_string(),
            });
            continue;
        };

        if metadata.file_type().is_symlink() && !request.follow_symlinks {
            continue;
        }

        if metadata.is_dir() {
            walk_root(&path, depth + 1, request, report, seen)?;
        }
    }

    Ok(())
}

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with('.'))
}

fn canonical_or_self(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn config_format(path: &Path) -> &'static str {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("yaml") | Some("yml") => "yaml",
        _ => "toml",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::config::ProjectConfigFormat;
    use crate::engine::config::write_default_project_config;
    use std::fs as std_fs;

    fn test_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("cadman_discovery_{}_{}", std::process::id(), name));
        let _ = std_fs::remove_dir_all(&dir);
        dir
    }

    fn request(root: PathBuf) -> ProjectDiscoveryRequest {
        ProjectDiscoveryRequest {
            roots: vec![root],
            max_depth: 6,
            follow_symlinks: false,
            include_hidden: false,
        }
    }

    #[test]
    fn finds_cadman_toml_recursively() {
        let root = test_dir("toml");
        let app = root.join("apps").join("web");
        write_default_project_config(&app, ProjectConfigFormat::Toml, false).unwrap();

        let report = discover_project_configs(request(root.clone())).unwrap();

        assert_eq!(report.projects.len(), 1);
        assert_eq!(report.projects[0].config_path, app.join("cadman.toml"));

        let _ = std_fs::remove_dir_all(&root);
    }

    #[test]
    fn finds_cadman_yaml_recursively() {
        let root = test_dir("yaml");
        let app = root.join("apps").join("web");
        write_default_project_config(&app, ProjectConfigFormat::Yaml, false).unwrap();

        let report = discover_project_configs(request(root.clone())).unwrap();

        assert_eq!(report.projects.len(), 1);
        assert_eq!(report.projects[0].format, "yaml");

        let _ = std_fs::remove_dir_all(&root);
    }

    #[test]
    fn prefers_cadman_toml_over_yaml() {
        let root = test_dir("prefer");
        let app = root.join("web");
        write_default_project_config(&app, ProjectConfigFormat::Yaml, false).unwrap();
        write_default_project_config(&app, ProjectConfigFormat::Toml, true).unwrap();

        let report = discover_project_configs(request(root.clone())).unwrap();

        assert_eq!(report.projects.len(), 1);
        assert!(report.projects[0].config_path.ends_with("cadman.toml"));

        let _ = std_fs::remove_dir_all(&root);
    }

    #[test]
    fn respects_max_depth() {
        let root = test_dir("depth");
        let app = root.join("one").join("two");
        write_default_project_config(&app, ProjectConfigFormat::Toml, false).unwrap();
        let mut req = request(root.clone());
        req.max_depth = 1;

        let report = discover_project_configs(req).unwrap();

        assert!(report.projects.is_empty());

        let _ = std_fs::remove_dir_all(&root);
    }

    #[test]
    fn skips_hidden_dirs_by_default() {
        let root = test_dir("hidden");
        let app = root.join(".hidden").join("web");
        write_default_project_config(&app, ProjectConfigFormat::Toml, false).unwrap();

        let report = discover_project_configs(request(root.clone())).unwrap();

        assert!(report.projects.is_empty());

        let _ = std_fs::remove_dir_all(&root);
    }

    #[test]
    fn includes_hidden_dirs_when_enabled() {
        let root = test_dir("hidden_enabled");
        let app = root.join(".hidden").join("web");
        write_default_project_config(&app, ProjectConfigFormat::Toml, false).unwrap();
        let mut req = request(root.clone());
        req.include_hidden = true;

        let report = discover_project_configs(req).unwrap();

        assert_eq!(report.projects.len(), 1);

        let _ = std_fs::remove_dir_all(&root);
    }
}
