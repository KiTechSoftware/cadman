use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::engine::{
    ErrorCode, Result, constants::STATE_FILE_NAME, models::runtime::Runtime, system::fs,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CadmanState {
    pub version: u32,
    pub apps: BTreeMap<String, AppState>,
    pub last_reconcile_at: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AppState {
    pub app_id: String,
    pub container_id: Option<String>,
    pub container_name: Option<String>,
    pub container_state: Option<String>,
    pub container_health: Option<String>,
    pub caddy_site_path: Option<PathBuf>,
    pub config_hash: Option<String>,
    pub labels_hash: Option<String>,
    pub ports_hash: Option<String>,
    pub site_hash: Option<String>,
    pub last_seen_at: Option<String>,
}

impl Default for CadmanState {
    fn default() -> Self {
        Self {
            version: 1,
            apps: BTreeMap::new(),
            last_reconcile_at: None,
        }
    }
}

impl CadmanState {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn get(&self, app_id: &str) -> Option<&AppState> {
        self.apps.get(app_id)
    }

    pub fn upsert_app(&mut self, state: AppState) {
        self.apps.insert(state.app_id.clone(), state);
    }

    pub fn remove_app(&mut self, app_id: &str) -> Option<AppState> {
        self.apps.remove(app_id)
    }
}

/// Return the path to the state file.
pub fn path(runtime: &Runtime) -> PathBuf {
    runtime.state_dir().join(STATE_FILE_NAME)
}

/// Load state. Returns empty state if the file does not exist.
pub fn load(runtime: &Runtime) -> Result<CadmanState> {
    let p = path(runtime);
    if !p.exists() {
        return Ok(CadmanState::empty());
    }
    fs::load_json(&p).map_err(|_| {
        ErrorCode::StateUnreadable
            .error()
            .with_context("path", p.display().to_string())
    })
}

/// Save state to disk, creating parent directories as needed.
pub fn save(runtime: &Runtime, state: &CadmanState) -> Result<()> {
    let p = path(runtime);
    fs::save_json(&p, state).map_err(|_| {
        ErrorCode::StateWriteFailed
            .error()
            .with_context("path", p.display().to_string())
    })
}
