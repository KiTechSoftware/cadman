use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::engine::{
    ErrorCode, Result,
    constants::{
        PROJECT_CONFIG_FILE_NAME, PROJECT_CONFIG_FILE_NAME_YAML, PROJECT_CONFIG_FILE_NAME_YML,
    },
    system::fs,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProjectConfig {
    pub project: ProjectSection,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub containers: BTreeMap<String, ProjectContainer>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub routes: BTreeMap<String, ProjectRoute>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProjectSection {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProjectContainer {
    pub image: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ports: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub volumes: Vec<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub environment: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub env_file: Vec<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restart: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProjectRoute {
    pub hosts: Vec<String>,
    pub container: String,
    pub upstream_port: u16,
    pub tls: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectConfigFormat {
    Toml,
    Yaml,
}

impl ProjectConfigFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Toml => "toml",
            Self::Yaml => "yaml",
        }
    }

    pub fn file_name(self) -> &'static str {
        match self {
            Self::Toml => PROJECT_CONFIG_FILE_NAME,
            Self::Yaml => PROJECT_CONFIG_FILE_NAME_YAML,
        }
    }
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            project: ProjectSection::default(),
            containers: BTreeMap::new(),
            routes: BTreeMap::new(),
            metadata: BTreeMap::new(),
        }
    }
}

impl Default for ProjectSection {
    fn default() -> Self {
        Self {
            name: String::new(),
            environment: None,
            description: None,
        }
    }
}

impl Default for ProjectContainer {
    fn default() -> Self {
        Self {
            image: String::new(),
            name: None,
            ports: Vec::new(),
            volumes: Vec::new(),
            environment: BTreeMap::new(),
            env_file: Vec::new(),
            restart: None,
            command: None,
        }
    }
}

impl Default for ProjectRoute {
    fn default() -> Self {
        Self {
            hosts: Vec::new(),
            container: String::new(),
            upstream_port: 80,
            tls: false,
            path: None,
        }
    }
}

/// Build a default `ProjectConfig` using the given project name.
pub fn default_project_config(project_name: &str) -> ProjectConfig {
    let mut containers = BTreeMap::new();
    containers.insert(
        "web".to_string(),
        ProjectContainer {
            image: "docker.io/library/nginx:alpine".to_string(),
            ports: vec!["127.0.0.1:8080:80".to_string()],
            volumes: vec!["./public:/usr/share/nginx/html:Z".to_string()],
            restart: Some("unless-stopped".to_string()),
            ..Default::default()
        },
    );
    containers.insert(
        "api".to_string(),
        ProjectContainer {
            image: "docker.io/library/httpd:alpine".to_string(),
            ports: vec!["127.0.0.1:8081:80".to_string()],
            volumes: vec!["./api-data:/usr/local/apache2/htdocs:Z".to_string()],
            restart: Some("unless-stopped".to_string()),
            ..Default::default()
        },
    );

    let mut routes = BTreeMap::new();
    routes.insert(
        "web".to_string(),
        ProjectRoute {
            hosts: vec![
                format!("{project_name}.local"),
                format!("www.{project_name}.local"),
            ],
            container: "web".to_string(),
            upstream_port: 80,
            tls: false,
            path: None,
        },
    );
    routes.insert(
        "api".to_string(),
        ProjectRoute {
            hosts: vec![format!("api.{project_name}.local")],
            container: "api".to_string(),
            upstream_port: 80,
            tls: false,
            path: None,
        },
    );

    ProjectConfig {
        project: ProjectSection {
            name: project_name.to_string(),
            environment: None,
            description: Some("Cadman managed project".to_string()),
        },
        containers,
        routes,
        ..Default::default()
    }
}

/// Detect the project config file in a directory.
/// Prefers `cadman.toml`, then `cadman.yaml`, then `cadman.yml`.
pub fn detect_project_config(dir: &Path) -> Result<Option<PathBuf>> {
    for name in &[
        PROJECT_CONFIG_FILE_NAME,
        PROJECT_CONFIG_FILE_NAME_YAML,
        PROJECT_CONFIG_FILE_NAME_YML,
    ] {
        let candidate = dir.join(name);
        if candidate.exists() {
            return Ok(Some(candidate));
        }
    }
    Ok(None)
}

/// Load a project config from a path. Format is inferred from extension.
pub fn load_project_config(path: &Path) -> Result<ProjectConfig> {
    let raw = fs::read_text(path).map_err(|_| {
        ErrorCode::ConfigNotFound
            .error()
            .with_context("path", path.display().to_string())
    })?;

    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    match ext {
        "toml" => toml::from_str(&raw).map_err(|err| {
            ErrorCode::ConfigInvalid
                .error()
                .with_context("path", path.display().to_string())
                .with_context("error", err.to_string())
        }),
        "yaml" | "yml" => serde_yaml::from_str(&raw).map_err(|err| {
            ErrorCode::ConfigInvalid
                .error()
                .with_context("path", path.display().to_string())
                .with_context("error", err.to_string())
        }),
        other => Err(ErrorCode::ConfigUnsupportedFormat
            .error()
            .with_context("extension", other)
            .with_context("path", path.display().to_string())),
    }
}

/// Write a default project config to `dir`.
/// Uses TOML unless `format` is `Yaml`.
/// Returns the path to the created file.
/// Returns `ConfigAlreadyExists` if a config exists and `force` is false.
pub fn write_default_project_config(
    dir: &Path,
    format: ProjectConfigFormat,
    force: bool,
) -> Result<PathBuf> {
    if !force {
        if let Some(existing) = detect_project_config(dir)? {
            return Err(ErrorCode::ConfigAlreadyExists
                .error()
                .with_context("path", existing.display().to_string())
                .with_context("hint", "use --force to overwrite"));
        }
    }

    let project_name = dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project")
        .to_string();

    let config = default_project_config(&project_name);

    match format {
        ProjectConfigFormat::Toml => {
            let p = dir.join(PROJECT_CONFIG_FILE_NAME);
            fs::save_toml(&p, &config)?;
            Ok(p)
        }
        ProjectConfigFormat::Yaml => {
            let p = dir.join(PROJECT_CONFIG_FILE_NAME_YAML);
            let raw = serde_yaml::to_string(&config).map_err(|err| {
                ErrorCode::SerializationFailure
                    .error()
                    .with_context("operation", "write_default_project_config")
                    .with_context("error", err.to_string())
            })?;
            fs::write_text(&p, &raw)?;
            Ok(p)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs as std_fs;

    fn test_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "cadman_project_config_{}_{}",
            std::process::id(),
            name
        ));
        let _ = std_fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn write_default_toml() {
        let dir = test_dir("toml");
        let path = write_default_project_config(&dir, ProjectConfigFormat::Toml, false).unwrap();

        assert_eq!(path, dir.join(PROJECT_CONFIG_FILE_NAME));
        assert!(path.exists());

        let raw = std_fs::read_to_string(&path).unwrap();
        assert!(raw.contains("[project]"));
        assert!(raw.contains("[containers.web]"));
        assert!(raw.contains("[routes.web]"));

        let _ = std_fs::remove_dir_all(&dir);
    }

    #[test]
    fn write_default_yaml() {
        let dir = test_dir("yaml");
        let path = write_default_project_config(&dir, ProjectConfigFormat::Yaml, false).unwrap();

        assert_eq!(path, dir.join(PROJECT_CONFIG_FILE_NAME_YAML));
        assert!(path.exists());

        let raw = std_fs::read_to_string(&path).unwrap();
        assert!(raw.contains("containers:"));
        assert!(raw.contains("routes:"));

        let _ = std_fs::remove_dir_all(&dir);
    }

    #[test]
    fn does_not_overwrite_existing_config_without_force() {
        let dir = test_dir("no_force");
        std_fs::create_dir_all(&dir).unwrap();
        let existing = dir.join(PROJECT_CONFIG_FILE_NAME);
        std_fs::write(&existing, "existing").unwrap();

        let err = write_default_project_config(&dir, ProjectConfigFormat::Toml, false).unwrap_err();
        assert_eq!(err.code, ErrorCode::ConfigAlreadyExists);
        assert_eq!(std_fs::read_to_string(&existing).unwrap(), "existing");

        let _ = std_fs::remove_dir_all(&dir);
    }

    #[test]
    fn overwrites_target_config_with_force() {
        let dir = test_dir("force");
        std_fs::create_dir_all(&dir).unwrap();
        let existing = dir.join(PROJECT_CONFIG_FILE_NAME);
        std_fs::write(&existing, "existing").unwrap();

        let path = write_default_project_config(&dir, ProjectConfigFormat::Toml, true).unwrap();
        assert_eq!(path, existing);
        assert_ne!(std_fs::read_to_string(&path).unwrap(), "existing");

        let _ = std_fs::remove_dir_all(&dir);
    }

    #[test]
    fn detect_prefers_toml_over_yaml() {
        let dir = test_dir("detect");
        std_fs::create_dir_all(&dir).unwrap();
        let toml = dir.join(PROJECT_CONFIG_FILE_NAME);
        let yaml = dir.join(PROJECT_CONFIG_FILE_NAME_YAML);
        std_fs::write(&yaml, "yaml").unwrap();
        std_fs::write(&toml, "toml").unwrap();

        assert_eq!(detect_project_config(&dir).unwrap(), Some(toml));

        let _ = std_fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_written_toml() {
        let dir = test_dir("load_toml");
        let path = write_default_project_config(&dir, ProjectConfigFormat::Toml, false).unwrap();
        let config = load_project_config(&path).unwrap();

        assert_eq!(
            config.project.name,
            dir.file_name().unwrap().to_string_lossy()
        );
        assert!(config.containers.contains_key("web"));
        assert_eq!(config.routes["web"].container, "web");

        let _ = std_fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_written_yaml() {
        let dir = test_dir("load_yaml");
        let path = write_default_project_config(&dir, ProjectConfigFormat::Yaml, false).unwrap();
        let config = load_project_config(&path).unwrap();

        assert_eq!(
            config.project.name,
            dir.file_name().unwrap().to_string_lossy()
        );
        assert!(config.containers.contains_key("api"));
        assert_eq!(config.routes["api"].container, "api");

        let _ = std_fs::remove_dir_all(&dir);
    }
}
