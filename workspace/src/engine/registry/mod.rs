use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::engine::{
    ErrorCode, Result,
    models::{containers::ContainerScope, runtime::Runtime},
    system::fs,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Registry {
    pub version: u32,
    pub apps: BTreeMap<String, RegistryApp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryApp {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub source: RegistrySource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_path: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config_path: Option<PathBuf>,
    pub desired_status: DesiredStatus,
    pub managed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub container_scope: Option<ContainerScope>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub container_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub container_name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RegistrySource {
    ProjectConfig,
    PodmanLabels,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DesiredStatus {
    Up,
    Down,
}

impl Default for RegistrySource {
    fn default() -> Self {
        Self::ProjectConfig
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self {
            version: 1,
            apps: BTreeMap::new(),
        }
    }
}

impl Registry {
    pub fn empty() -> Self {
        Self::default()
    }

    /// Get an app by exact id or by name.
    pub fn get(&self, id_or_name: &str) -> Option<&RegistryApp> {
        self.apps
            .get(id_or_name)
            .or_else(|| self.apps.values().find(|a| a.name == id_or_name))
    }

    /// Add an app. Returns error if the id or name already exists.
    pub fn add_app(&mut self, app: RegistryApp) -> Result<()> {
        app.validate()?;
        if self.apps.contains_key(&app.id) {
            return Err(ErrorCode::RegistryAppAlreadyExists
                .error()
                .with_context("id", &app.id));
        }
        if self.apps.values().any(|a| a.name == app.name) {
            return Err(ErrorCode::RegistryAppAlreadyExists
                .error()
                .with_context("name", &app.name));
        }
        self.apps.insert(app.id.clone(), app);
        Ok(())
    }

    /// Remove an app by id or name. Returns the removed app.
    pub fn remove_app(&mut self, id_or_name: &str) -> Result<RegistryApp> {
        let key = self
            .apps
            .get(id_or_name)
            .map(|a| a.id.clone())
            .or_else(|| {
                self.apps
                    .values()
                    .find(|a| a.name == id_or_name)
                    .map(|a| a.id.clone())
            })
            .ok_or_else(|| {
                ErrorCode::RegistryAppNotFound
                    .error()
                    .with_context("id_or_name", id_or_name)
            })?;
        Ok(self.apps.remove(&key).expect("key was just found"))
    }

    /// List all apps.
    pub fn list(&self) -> Vec<&RegistryApp> {
        self.apps.values().collect()
    }
}

impl RegistryApp {
    pub fn validate(&self) -> Result<()> {
        if self.id.trim().is_empty() {
            return Err(ErrorCode::RegistryInvalid
                .error()
                .with_context("field", "id")
                .with_context("reason", "registry app id must not be empty"));
        }
        if self.name.trim().is_empty() {
            return Err(ErrorCode::RegistryInvalid
                .error()
                .with_context("field", "name")
                .with_context("reason", "registry app name must not be empty"));
        }

        match self.source {
            RegistrySource::ProjectConfig => {
                if self.project_path.is_none() || self.config_path.is_none() {
                    return Err(ErrorCode::RegistryInvalid
                        .error()
                        .with_context("app", &self.id)
                        .with_context(
                            "reason",
                            "project-config registry apps require project_path and config_path",
                        ));
                }
            }
            RegistrySource::PodmanLabels => {
                if self.container_name.is_none() || self.container_scope.is_none() {
                    return Err(ErrorCode::RegistryInvalid
                        .error()
                        .with_context("app", &self.id)
                        .with_context(
                            "reason",
                            "podman-label registry apps require container_name and container_scope",
                        ));
                }
            }
        }

        Ok(())
    }
}

/// Return the path to the registry file.
pub fn path(runtime: &Runtime) -> PathBuf {
    runtime.registry_path()
}

/// Load registry. Returns an empty registry if the file does not exist.
pub fn load(runtime: &Runtime) -> Result<Registry> {
    let p = path(runtime);
    if !p.exists() {
        return Ok(Registry::empty());
    }
    fs::load_toml(&p).map_err(|_| {
        ErrorCode::RegistryInvalid
            .error()
            .with_context("path", p.display().to_string())
            .with_context("reason", "registry file could not be read or parsed")
    })
}

/// Save the registry to disk, creating parent directories as needed.
pub fn save(runtime: &Runtime, registry: &Registry) -> Result<()> {
    let p = path(runtime);
    fs::save_toml(&p, registry).map_err(|_| {
        ErrorCode::RegistryWriteFailed
            .error()
            .with_context("path", p.display().to_string())
    })
}

pub fn sanitize_app_id(value: &str) -> String {
    let mut sanitized = String::new();
    let mut previous_dash = false;

    for ch in value.trim().to_ascii_lowercase().chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            sanitized.push(ch);
            previous_dash = false;
        } else if ch == '-' || ch.is_ascii_whitespace() || ch == '.' || ch == '/' {
            if !previous_dash && !sanitized.is_empty() {
                sanitized.push('-');
                previous_dash = true;
            }
        }
    }

    let sanitized = sanitized.trim_matches('-').to_string();
    if sanitized.is_empty() {
        "project".to_string()
    } else {
        sanitized
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use toml::toml;

    fn project_app(id: &str, name: &str) -> RegistryApp {
        RegistryApp {
            id: id.to_string(),
            name: name.to_string(),
            source: RegistrySource::ProjectConfig,
            project_path: Some(PathBuf::from(format!("/projects/{id}"))),
            config_path: Some(PathBuf::from(format!("/projects/{id}/cadman.toml"))),
            desired_status: DesiredStatus::Down,
            managed: true,
            container_scope: None,
            container_id: None,
            container_name: None,
        }
    }

    #[test]
    fn sanitize_app_id_normalizes_names() {
        assert_eq!(sanitize_app_id("My App.local"), "my-app-local");
        assert_eq!(sanitize_app_id(""), "project");
        assert_eq!(sanitize_app_id("___"), "___");
    }

    #[test]
    fn old_registry_entries_deserialize_as_project_config_source() {
        let value = toml! {
            id = "web"
            name = "Web"
            project_path = "/projects/web"
            config_path = "/projects/web/cadman.toml"
            desired_status = "down"
            managed = true
        };

        let app: RegistryApp = value.try_into().unwrap();

        assert_eq!(app.source, RegistrySource::ProjectConfig);
        assert_eq!(app.project_path, Some(PathBuf::from("/projects/web")));
        assert_eq!(
            app.config_path,
            Some(PathBuf::from("/projects/web/cadman.toml"))
        );
    }

    #[test]
    fn project_config_app_validates_with_paths() {
        let app = project_app("web", "Web");

        assert!(app.validate().is_ok());
    }

    #[test]
    fn podman_label_app_validates_with_container_metadata() {
        let app = RegistryApp {
            id: "web".to_string(),
            name: "Web".to_string(),
            source: RegistrySource::PodmanLabels,
            project_path: None,
            config_path: None,
            desired_status: DesiredStatus::Down,
            managed: true,
            container_scope: Some(ContainerScope::Current),
            container_id: Some("abc".to_string()),
            container_name: Some("web".to_string()),
        };

        assert!(app.validate().is_ok());
    }

    #[test]
    fn add_duplicate_id_fails() {
        let mut registry = Registry::empty();
        registry.add_app(project_app("web", "Web")).unwrap();

        let err = registry
            .add_app(project_app("web", "Other Web"))
            .unwrap_err();

        assert_eq!(err.code, ErrorCode::RegistryAppAlreadyExists);
    }

    #[test]
    fn add_duplicate_name_fails() {
        let mut registry = Registry::empty();
        registry.add_app(project_app("web", "Web")).unwrap();

        let err = registry.add_app(project_app("api", "Web")).unwrap_err();

        assert_eq!(err.code, ErrorCode::RegistryAppAlreadyExists);
    }
}
