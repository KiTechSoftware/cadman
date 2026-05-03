use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::engine::{
    ErrorCode, Result,
    constants::{PROJECT_CONFIG_FILE_NAME, PROJECT_CONFIG_FILE_NAME_YAML, PROJECT_CONFIG_FILE_NAME_YML},
    system::fs,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProjectConfig {
    pub project: ProjectSection,
    pub containers: BTreeMap<String, ProjectContainer>,
    pub proxy: Option<ProxySection>,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProjectSection {
    pub name: String,
    pub environment: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProjectContainer {
    pub image: String,
    pub name: Option<String>,
    pub ports: Vec<String>,
    pub volumes: Vec<String>,
    pub environment: BTreeMap<String, String>,
    pub env_file: Vec<PathBuf>,
    pub restart: Option<String>,
    pub command: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProxySection {
    pub domain: String,
    pub upstream_port: u16,
    pub tls: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectConfigFormat {
    Toml,
    Yaml,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            project: ProjectSection::default(),
            containers: BTreeMap::new(),
            proxy: None,
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

impl Default for ProxySection {
    fn default() -> Self {
        Self {
            domain: String::new(),
            upstream_port: 80,
            tls: false,
        }
    }
}

/// Build a default `ProjectConfig` using the given project name.
pub fn default_project_config(project_name: &str) -> ProjectConfig {
    ProjectConfig {
        project: ProjectSection {
            name: project_name.to_string(),
            environment: None,
            description: None,
        },
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
        for name in &[
            PROJECT_CONFIG_FILE_NAME,
            PROJECT_CONFIG_FILE_NAME_YAML,
            PROJECT_CONFIG_FILE_NAME_YML,
        ] {
            let candidate = dir.join(name);
            if candidate.exists() {
                return Err(ErrorCode::ConfigAlreadyExists
                    .error()
                    .with_context("path", candidate.display().to_string()));
            }
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
