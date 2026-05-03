use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use veltrix::os::unistd;

use crate::engine::{
    Result,
    constants::{CADMAN_GROUP_NAME, CADMAN_USER_NAME, PODMAN_SERVICE_NAME, CADDY_SERVICE_NAME},
    models::runtime::Runtime,
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
    pub config_path: PathBuf,
    pub registry_path: PathBuf,
    pub state_path: PathBuf,
}

pub fn collect_diagnostics(runtime: &Runtime) -> Result<DiagnosticsReport> {
    let current_uid = unistd::getuid();

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
        config_path: runtime.effective_config_path(),
        registry_path: runtime.registry_path(),
        state_path: runtime.state_path(),
    })
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
    }
}
