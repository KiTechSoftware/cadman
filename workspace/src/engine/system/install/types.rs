use std::path::PathBuf;

use crate::engine::models::runtime::{InstallScope, RunMode};

use super::package::PackageManager;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallTarget {
    User,
    System,
    ContainerUser,
    ContainerSystem,
}

pub fn install_target_from_scope(scope: InstallScope) -> InstallTarget {
    match scope {
        InstallScope::User => InstallTarget::User,
        InstallScope::System => InstallTarget::System,
        InstallScope::Container => InstallTarget::ContainerSystem,
    }
}

pub fn install_target_from_facts(facts: &InstallFacts) -> InstallTarget {
    match facts.install_scope {
        InstallScope::User => InstallTarget::User,
        InstallScope::System => InstallTarget::System,
        InstallScope::Container => {
            if facts.explicit_container_system
                || (facts.effective_uid_is_root && facts.container_system_paths_writable)
            {
                InstallTarget::ContainerSystem
            } else {
                InstallTarget::ContainerUser
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct InstallFacts {
    pub requested_run_mode: RunMode,
    pub effective_run_mode: RunMode,
    pub install_scope: InstallScope,
    pub dry_run: bool,
    pub current_exe: PathBuf,
    pub installed_binary_path: Option<PathBuf>,
    pub config_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub state_dir: PathBuf,
    pub log_dir: PathBuf,
    pub user_bin_dir: PathBuf,
    pub current_user: Option<String>,
    pub current_user_in_cadman_group: bool,
    pub current_user_in_admin_group: bool,
    pub effective_uid_is_root: bool,
    pub podman_installed: bool,
    pub caddy_installed: bool,
    pub cadman_group_exists: bool,
    pub cadman_user_exists: bool,
    pub systemd_available: bool,
    pub package_manager: Option<PackageManager>,
    pub container_system_paths_writable: bool,
    pub explicit_container_system: bool,
}

#[derive(Debug, Clone)]
pub struct InstallPlan {
    pub target: InstallTarget,
    pub dry_run: bool,
    pub steps: Vec<InstallStep>,
}

#[derive(Debug, Clone)]
pub struct InstallStep {
    pub label: String,
    pub action: InstallAction,
}

#[derive(Debug, Clone)]
pub enum InstallAction {
    Noop {
        note: String,
    },
    EnsureDir {
        path: PathBuf,
        mode: u32,
        run_mode: RunMode,
    },
    EnsureGroup {
        name: String,
    },
    EnsureUser {
        name: String,
        group: String,
        home: PathBuf,
        shell: String,
    },
    AddUserToGroup {
        user: String,
        group: String,
    },
    RemoveUserFromGroup {
        user: String,
        group: String,
    },
    RemoveUser {
        name: String,
    },
    RemoveGroup {
        name: String,
    },
    SetOwner {
        path: PathBuf,
        owner: String,
        group: String,
    },
    InstallPackage {
        manager: PackageManager,
        package: String,
        run_mode: RunMode,
    },
    InstallBinary {
        source: PathBuf,
        destination: PathBuf,
        mode: u32,
        run_mode: RunMode,
    },
    WriteFile {
        path: PathBuf,
        content: String,
        mode: u32,
        run_mode: RunMode,
    },
    RemoveFile {
        path: PathBuf,
        run_mode: RunMode,
    },
}

impl InstallStep {
    pub fn new(label: impl Into<String>, action: InstallAction) -> Self {
        Self {
            label: label.into(),
            action,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts(scope: InstallScope) -> InstallFacts {
        InstallFacts {
            requested_run_mode: RunMode::Current,
            effective_run_mode: RunMode::Current,
            install_scope: scope,
            dry_run: false,
            current_exe: PathBuf::from("/tmp/cadman"),
            installed_binary_path: None,
            config_dir: PathBuf::from("/tmp/config"),
            cache_dir: PathBuf::from("/tmp/cache"),
            state_dir: PathBuf::from("/tmp/state"),
            log_dir: PathBuf::from("/tmp/log"),
            user_bin_dir: PathBuf::from("/tmp/bin"),
            current_user: Some("alice".to_string()),
            current_user_in_cadman_group: false,
            current_user_in_admin_group: false,
            effective_uid_is_root: false,
            podman_installed: true,
            caddy_installed: true,
            cadman_group_exists: false,
            cadman_user_exists: false,
            systemd_available: false,
            package_manager: None,
            container_system_paths_writable: false,
            explicit_container_system: false,
        }
    }

    #[test]
    fn install_scope_maps_to_target() {
        assert_eq!(
            install_target_from_scope(InstallScope::User),
            InstallTarget::User
        );
        assert_eq!(
            install_target_from_scope(InstallScope::System),
            InstallTarget::System
        );
        assert_eq!(
            install_target_from_scope(InstallScope::Container),
            InstallTarget::ContainerSystem
        );
    }

    #[test]
    fn container_facts_choose_user_when_not_root_or_writable() {
        assert_eq!(
            install_target_from_facts(&facts(InstallScope::Container)),
            InstallTarget::ContainerUser
        );
    }

    #[test]
    fn container_facts_choose_system_when_explicit() {
        let mut facts = facts(InstallScope::Container);
        facts.explicit_container_system = true;
        assert_eq!(
            install_target_from_facts(&facts),
            InstallTarget::ContainerSystem
        );
    }
}
