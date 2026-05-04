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
#[derive(Default)]
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
#[derive(Default)]
pub struct ProjectSection {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
#[derive(Default)]
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
    let config = match ext {
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
    }?;

    validate_project_config(&config)?;
    Ok(config)
}

pub fn validate_project_config(config: &ProjectConfig) -> Result<()> {
    if config.project.name.trim().is_empty() {
        return Err(ErrorCode::ValidationProjectInvalid
            .error()
            .with_context("field", "project.name")
            .with_context("reason", "project name must not be empty"));
    }

    if config.containers.is_empty() {
        return Err(ErrorCode::ValidationProjectInvalid
            .error()
            .with_context("field", "containers")
            .with_context("reason", "at least one container must exist"));
    }

    for (container_name, container) in &config.containers {
        if container_name.trim().is_empty() {
            return Err(ErrorCode::ValidationProjectInvalid
                .error()
                .with_context("field", "containers")
                .with_context("reason", "container key must not be empty"));
        }

        if container.image.trim().is_empty() {
            return Err(ErrorCode::ValidationProjectInvalid
                .error()
                .with_context("container", container_name)
                .with_context("field", "image")
                .with_context("reason", "container image must not be empty"));
        }

        for port in &container.ports {
            if port.trim().is_empty() {
                return Err(ErrorCode::ValidationPortInvalid
                    .error()
                    .with_context("container", container_name)
                    .with_context("field", "ports")
                    .with_context("reason", "container port mapping must not be empty"));
            }
        }

        for volume in &container.volumes {
            if volume.trim().is_empty() {
                return Err(ErrorCode::ValidationProjectInvalid
                    .error()
                    .with_context("container", container_name)
                    .with_context("field", "volumes")
                    .with_context("reason", "volume mapping must not be empty"));
            }
        }

        for env_file in &container.env_file {
            if env_file.as_os_str().is_empty() {
                return Err(ErrorCode::ValidationProjectInvalid
                    .error()
                    .with_context("container", container_name)
                    .with_context("field", "env_file")
                    .with_context("reason", "env file path must not be empty"));
            }
        }
    }

    for (route_name, route) in &config.routes {
        if route_name.trim().is_empty() {
            return Err(ErrorCode::ValidationRouteInvalid
                .error()
                .with_context("field", "routes")
                .with_context("reason", "route key must not be empty"));
        }

        if route.hosts.is_empty() {
            return Err(ErrorCode::ValidationRouteInvalid
                .error()
                .with_context("route", route_name)
                .with_context("field", "hosts")
                .with_context("reason", "route must have at least one host"));
        }

        for host in &route.hosts {
            if host.trim().is_empty() {
                return Err(ErrorCode::ValidationRouteInvalid
                    .error()
                    .with_context("route", route_name)
                    .with_context("field", "hosts")
                    .with_context("reason", "route host must not be empty"));
            }
        }

        if !config.containers.contains_key(&route.container) {
            return Err(ErrorCode::ValidationRouteInvalid
                .error()
                .with_context("route", route_name)
                .with_context("field", "container")
                .with_context("reason", "route references an unknown container"));
        }

        if route.upstream_port == 0 {
            return Err(ErrorCode::ValidationRouteInvalid
                .error()
                .with_context("route", route_name)
                .with_context("field", "upstream_port")
                .with_context("reason", "route upstream port must be greater than 0"));
        }
    }

    Ok(())
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
    if !force && let Some(existing) = detect_project_config(dir)? {
        return Err(ErrorCode::ConfigAlreadyExists
            .error()
            .with_context("path", existing.display().to_string())
            .with_context("hint", "use --force to overwrite"));
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
    fn default_project_config_is_valid() {
        validate_project_config(&default_project_config("my-app")).unwrap();
    }

    #[test]
    fn empty_project_name_fails() {
        let mut config = default_project_config("my-app");
        config.project.name.clear();

        let err = validate_project_config(&config).unwrap_err();
        assert_eq!(err.code, ErrorCode::ValidationProjectInvalid);
    }

    #[test]
    fn no_containers_fails() {
        let mut config = default_project_config("my-app");
        config.containers.clear();

        let err = validate_project_config(&config).unwrap_err();
        assert_eq!(err.code, ErrorCode::ValidationProjectInvalid);
    }

    #[test]
    fn container_with_empty_image_fails() {
        let mut config = default_project_config("my-app");
        config.containers.get_mut("web").unwrap().image.clear();

        let err = validate_project_config(&config).unwrap_err();
        assert_eq!(err.code, ErrorCode::ValidationProjectInvalid);
    }

    #[test]
    fn route_with_no_hosts_fails() {
        let mut config = default_project_config("my-app");
        config.routes.get_mut("web").unwrap().hosts.clear();

        let err = validate_project_config(&config).unwrap_err();
        assert_eq!(err.code, ErrorCode::ValidationRouteInvalid);
    }

    #[test]
    fn route_with_empty_host_fails() {
        let mut config = default_project_config("my-app");
        config
            .routes
            .get_mut("web")
            .unwrap()
            .hosts
            .push(" ".to_string());

        let err = validate_project_config(&config).unwrap_err();
        assert_eq!(err.code, ErrorCode::ValidationRouteInvalid);
    }

    #[test]
    fn route_referencing_missing_container_fails() {
        let mut config = default_project_config("my-app");
        config.routes.get_mut("web").unwrap().container = "missing".to_string();

        let err = validate_project_config(&config).unwrap_err();
        assert_eq!(err.code, ErrorCode::ValidationRouteInvalid);
    }

    #[test]
    fn route_with_zero_upstream_port_fails() {
        let mut config = default_project_config("my-app");
        config.routes.get_mut("web").unwrap().upstream_port = 0;

        let err = validate_project_config(&config).unwrap_err();
        assert_eq!(err.code, ErrorCode::ValidationRouteInvalid);
    }

    #[test]
    fn empty_port_mapping_fails() {
        let mut config = default_project_config("my-app");
        config
            .containers
            .get_mut("web")
            .unwrap()
            .ports
            .push(" ".to_string());

        let err = validate_project_config(&config).unwrap_err();
        assert_eq!(err.code, ErrorCode::ValidationPortInvalid);
    }

    #[test]
    fn empty_volume_mapping_fails() {
        let mut config = default_project_config("my-app");
        config
            .containers
            .get_mut("web")
            .unwrap()
            .volumes
            .push(" ".to_string());

        let err = validate_project_config(&config).unwrap_err();
        assert_eq!(err.code, ErrorCode::ValidationProjectInvalid);
    }

    #[test]
    fn empty_env_file_path_fails() {
        let mut config = default_project_config("my-app");
        config
            .containers
            .get_mut("web")
            .unwrap()
            .env_file
            .push(PathBuf::new());

        let err = validate_project_config(&config).unwrap_err();
        assert_eq!(err.code, ErrorCode::ValidationProjectInvalid);
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

    #[test]
    fn load_project_config_validates_toml() {
        let dir = test_dir("load_invalid_toml");
        std_fs::create_dir_all(&dir).unwrap();
        let path = dir.join(PROJECT_CONFIG_FILE_NAME);
        std_fs::write(
            &path,
            r#"
[project]
name = ""

[containers.web]
image = "docker.io/library/nginx:alpine"

[routes.web]
hosts = ["my-app.local"]
container = "web"
upstream_port = 80
tls = false
"#,
        )
        .unwrap();

        let err = load_project_config(&path).unwrap_err();
        assert_eq!(err.code, ErrorCode::ValidationProjectInvalid);

        let _ = std_fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_project_config_validates_yaml() {
        let dir = test_dir("load_invalid_yaml");
        std_fs::create_dir_all(&dir).unwrap();
        let path = dir.join(PROJECT_CONFIG_FILE_NAME_YAML);
        std_fs::write(
            &path,
            r#"
project:
  name: my-app
containers:
  web:
    image: docker.io/library/nginx:alpine
routes:
  web:
    hosts: []
    container: web
    upstream_port: 80
    tls: false
"#,
        )
        .unwrap();

        let err = load_project_config(&path).unwrap_err();
        assert_eq!(err.code, ErrorCode::ValidationRouteInvalid);

        let _ = std_fs::remove_dir_all(&dir);
    }
}
