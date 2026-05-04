use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use veltrix::os::unistd;

use crate::engine::{
    Result,
    capabilities::caddy::sites::MANAGED_HEADER,
    config,
    constants::{CADDY_SERVICE_NAME, CADMAN_GROUP_NAME, CADMAN_USER_NAME, PODMAN_SERVICE_NAME},
    models::runtime::Runtime,
    state,
    system::install::package::binary_exists,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsReport {
    pub podman_available: bool,
    pub caddy_available: bool,
    pub systemd_available: bool,
    pub cadman_user_exists: bool,
    pub cadman_group_exists: bool,
    pub current_user_in_cadman_group: bool,
    pub current_user_in_admin_group: bool,
    pub install_scope: String,
    pub effective_user: String,
    pub authorized_container_scopes: Vec<String>,
    pub config_path: PathBuf,
    pub registry_path: PathBuf,
    pub state_path: PathBuf,
    pub caddy_sites_dir: PathBuf,
    pub caddy_sites_dir_exists: bool,
    pub caddy_sites_dir_writable: bool,
    pub config_readable: bool,
    pub registry_readable: bool,
    pub registry_writable: bool,
    pub state_readable: bool,
    pub state_writable: bool,
    pub stale_generated_site_files: Vec<PathBuf>,
}

pub fn collect_diagnostics(runtime: &Runtime) -> Result<DiagnosticsReport> {
    let current_uid = unistd::getuid();
    let config_path = runtime.effective_config_path();
    let registry_path = runtime.registry_path();
    let state_path = runtime.state_path();
    let config = config::load(runtime).unwrap_or_else(|_| config::default_for(runtime));
    let caddy_sites_dir = config.caddy.sites_dir;
    let caddy_sites_dir_exists = caddy_sites_dir.is_dir();
    let caddy_sites_dir_writable = path_writable(&caddy_sites_dir);
    let state = state::load(runtime).ok();

    Ok(DiagnosticsReport {
        podman_available: binary_exists(PODMAN_SERVICE_NAME),
        caddy_available: binary_exists(CADDY_SERVICE_NAME),
        systemd_available: std::path::Path::new("/etc/systemd/system").exists(),
        cadman_user_exists: unistd::uid_by_username(CADMAN_USER_NAME).is_some(),
        cadman_group_exists: unistd::gid_by_groupname(CADMAN_GROUP_NAME).is_some(),
        current_user_in_cadman_group: unistd::user_in_group(current_uid, CADMAN_GROUP_NAME),
        current_user_in_admin_group: unistd::user_in_admin_group(current_uid),
        install_scope: runtime.install_scope().to_string(),
        effective_user: runtime.effective_user().to_string(),
        authorized_container_scopes: runtime
            .visible_container_scopes()
            .into_iter()
            .map(|scope| scope.to_string())
            .collect(),
        config_readable: file_readable_or_missing(&config_path),
        registry_readable: file_readable_or_missing(&registry_path),
        registry_writable: path_writable_for_file(&registry_path),
        state_readable: file_readable_or_missing(&state_path),
        state_writable: path_writable_for_file(&state_path),
        stale_generated_site_files: stale_generated_site_files(&caddy_sites_dir, state.as_ref()),
        caddy_sites_dir,
        caddy_sites_dir_exists,
        caddy_sites_dir_writable,
        config_path,
        registry_path,
        state_path,
    })
}

fn file_readable_or_missing(path: &std::path::Path) -> bool {
    !path.exists() || std::fs::File::open(path).is_ok()
}

fn path_writable_for_file(path: &std::path::Path) -> bool {
    path.parent().map(path_writable).unwrap_or(false)
}

fn path_writable(path: &std::path::Path) -> bool {
    if !path.exists() {
        return path.parent().map(path_writable).unwrap_or(false);
    }

    std::fs::metadata(path)
        .map(|metadata| !metadata.permissions().readonly())
        .unwrap_or(false)
}

fn stale_generated_site_files(
    sites_dir: &std::path::Path,
    state: Option<&state::CadmanState>,
) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(sites_dir) else {
        return Vec::new();
    };
    let referenced: std::collections::BTreeSet<PathBuf> = state
        .map(|state| {
            state
                .apps
                .values()
                .flat_map(|app| app.routes.iter().map(|route| route.site_path.clone()))
                .collect()
        })
        .unwrap_or_default();

    entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && !referenced.contains(path))
        .filter(|path| {
            std::fs::read_to_string(path)
                .map(|content| content.starts_with(MANAGED_HEADER))
                .unwrap_or(false)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::models::runtime::{InstallScope, Runtime};

    #[test]
    fn diagnostics_report_uses_runtime_paths_and_scope() {
        let mut runtime = Runtime::new();
        runtime.set_install_scope(InstallScope::User);

        let report = collect_diagnostics(&runtime).unwrap();

        assert_eq!(report.install_scope, "user");
        assert!(report.config_path.ends_with("config.toml"));
        assert!(report.registry_path.ends_with("registry.toml"));
        assert!(report.state_path.ends_with("state.json"));
        assert!(!report.authorized_container_scopes.is_empty());
    }
}
