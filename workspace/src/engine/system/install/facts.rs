use std::path::{Path, PathBuf};

use veltrix::os::unistd::{self, Uid};

use crate::engine::{
    Result,
    constants::{CADMAN_GROUP_NAME, CADMAN_USER_NAME, paths},
    models::runtime::Runtime,
};

use super::{
    package::{binary_exists, detect_package_manager},
    types::InstallFacts,
};

pub fn collect_install_facts(runtime: &Runtime) -> Result<InstallFacts> {
    let current_exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("cadman"));
    let user_bin_dir = paths::user_bin_dir()?;
    let user_binary_path = user_bin_dir.join("cadman");
    let installed_binary_path = installed_binary_path(&user_binary_path);
    let current_uid = unistd::getuid();
    let effective_uid = unistd::geteuid();

    Ok(InstallFacts {
        requested_run_mode: runtime.run_mode(),
        effective_run_mode: runtime.effective_run_mode(),
        install_scope: runtime.install_scope(),
        dry_run: runtime.options().dry_run(),
        current_exe,
        installed_binary_path,
        config_dir: runtime.config_dir().clone(),
        cache_dir: runtime.cache_dir().clone(),
        state_dir: runtime.state_dir().clone(),
        log_dir: runtime.log_dir().clone(),
        user_bin_dir,
        current_user: unistd::username_by_uid(current_uid),
        current_user_in_cadman_group: unistd::user_in_group(current_uid, CADMAN_GROUP_NAME),
        current_user_in_admin_group: unistd::user_in_admin_group(current_uid),
        effective_uid_is_root: effective_uid == Uid::from_raw(0),
        podman_installed: binary_exists("podman"),
        caddy_installed: binary_exists("caddy"),
        cadman_group_exists: unistd::gid_by_groupname(CADMAN_GROUP_NAME).is_some(),
        cadman_user_exists: unistd::uid_by_username(CADMAN_USER_NAME).is_some(),
        systemd_available: Path::new("/etc/systemd/system").exists(),
        package_manager: detect_package_manager(),
        container_system_paths_writable: container_system_paths_writable(effective_uid),
        explicit_container_system: false,
    })
}

fn installed_binary_path(user_binary_path: &Path) -> Option<PathBuf> {
    [
        paths::system_bin_path(),
        paths::system_local_bin_path(),
        user_binary_path.to_path_buf(),
    ]
    .into_iter()
    .find(|path| path.is_file())
}

fn container_system_paths_writable(effective_uid: Uid) -> bool {
    if effective_uid == Uid::from_raw(0) {
        return true;
    }

    [
        paths::system_config_dir(),
        paths::system_state_dir(),
        paths::system_cache_dir(),
        paths::system_log_dir(),
    ]
    .iter()
    .all(|path| path_group_writable_or_creatable(path))
}

fn path_group_writable_or_creatable(path: &Path) -> bool {
    let Some(existing) = nearest_existing(path) else {
        return false;
    };

    std::fs::metadata(existing)
        .map(|metadata| !metadata.permissions().readonly())
        .unwrap_or(false)
}

fn nearest_existing(path: &Path) -> Option<&Path> {
    let mut candidate = Some(path);
    while let Some(path) = candidate {
        if path.exists() {
            return Some(path);
        }
        candidate = path.parent();
    }
    None
}
