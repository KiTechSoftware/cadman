use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::engine::{ErrorCode, Result, models::runtime::Runtime, system::fs};

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
    pub project_path: PathBuf,
    pub config_path: PathBuf,
    pub desired_status: DesiredStatus,
    pub managed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DesiredStatus {
    Up,
    Down,
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

    #[test]
    fn sanitize_app_id_normalizes_names() {
        assert_eq!(sanitize_app_id("My App.local"), "my-app-local");
        assert_eq!(sanitize_app_id(""), "project");
        assert_eq!(sanitize_app_id("___"), "___");
    }
}
