use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use veltrix::os::unistd::{self, Uid};

use crate::engine::{
    Result,
    constants::{
        BIN_NAME, CADDY_SERVICE_NAME, CADMAN_GROUP_NAME, CADMAN_USER_NAME, PODMAN_SERVICE_NAME,
        paths,
    },
    models::runtime::Runtime,
};

use super::{
    package::{binary_exists, detect_package_manager},
    types::InstallFacts,
};

pub fn collect_install_facts(runtime: &Runtime) -> Result<InstallFacts> {
    let current_exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from(BIN_NAME));
    let user_bin_dir = paths::user_bin_dir()?;
    let user_binary_path = user_bin_dir.join(BIN_NAME);
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
        podman_installed: binary_exists(PODMAN_SERVICE_NAME),
        caddy_installed: binary_exists(CADDY_SERVICE_NAME),
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
    .all(|path| path_writable_or_creatable(path))
}

fn path_writable_or_creatable(path: &Path) -> bool {
    if path.exists() {
        return path_is_writable(path);
    }

    let Some(existing_parent) = nearest_existing_parent(path) else {
        return false;
    };

    can_create_in_dir(existing_parent)
}

fn path_is_writable(path: &Path) -> bool {
    if path.is_dir() {
        can_create_in_dir(path)
    } else {
        OpenOptions::new().append(true).open(path).is_ok()
    }
}

fn can_create_in_dir(dir: &Path) -> bool {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();

    let probe_path = dir.join(format!(
        ".cadman-write-probe-{}-{unique}",
        std::process::id()
    ));

    match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&probe_path)
    {
        Ok(_) => {
            let _ = fs::remove_file(&probe_path);
            true
        }
        Err(_) => false,
    }
}

fn nearest_existing_parent(path: &Path) -> Option<&Path> {
    let mut candidate = path.parent();

    while let Some(path) = candidate {
        if path.exists() {
            return Some(path);
        }

        candidate = path.parent();
    }

    None
}
