use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InstallScope {
    User,
    System,
    Container,
}

impl Default for InstallScope {
    fn default() -> Self {
        Self::User
    }
}

#[derive(Debug, Clone)]
pub struct InstallScopeFacts {
    pub current_exe: PathBuf,
    pub explicit_config_path: Option<PathBuf>,
    pub systemd_unit_exists: bool,
    pub container_env: bool,
}

pub fn detect_install_scope(
    current_exe: &Path,
    explicit_config_path: Option<&Path>,
) -> InstallScope {
    detect_install_scope_from_facts(&InstallScopeFacts {
        current_exe: current_exe.to_path_buf(),
        explicit_config_path: explicit_config_path.map(Path::to_path_buf),
        systemd_unit_exists: Path::new("/etc/systemd/system/cadman.service").exists(),
        container_env: container_environment_detected(),
    })
}

pub fn detect_install_scope_from_facts(facts: &InstallScopeFacts) -> InstallScope {
    if facts.current_exe == Path::new("/usr/bin/cadman")
        || facts.current_exe == Path::new("/usr/local/bin/cadman")
        || facts.systemd_unit_exists
        || facts
            .explicit_config_path
            .as_deref()
            .is_some_and(is_system_config_path)
    {
        return InstallScope::System;
    }

    if facts.container_env {
        return InstallScope::Container;
    }

    InstallScope::User
}

fn is_system_config_path(path: &Path) -> bool {
    path.starts_with("/etc/cadman")
}

fn container_environment_detected() -> bool {
    Path::new("/.dockerenv").exists()
        || Path::new("/run/.containerenv").exists()
        || std::env::var("CADMAN_CONTAINER").is_ok_and(|value| value == "true")
        || std::env::var("container").is_ok_and(|value| value == "podman" || value == "docker")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts(path: &str) -> InstallScopeFacts {
        InstallScopeFacts {
            current_exe: PathBuf::from(path),
            explicit_config_path: None,
            systemd_unit_exists: false,
            container_env: false,
        }
    }

    #[test]
    fn target_debug_cadman_is_user_scope() {
        assert_eq!(
            detect_install_scope_from_facts(&facts("target/debug/cadman")),
            InstallScope::User
        );
    }

    #[test]
    fn relative_cadman_is_user_scope() {
        assert_eq!(
            detect_install_scope_from_facts(&facts("./cadman")),
            InstallScope::User
        );
    }

    #[test]
    fn usr_local_bin_cadman_is_system_scope() {
        assert_eq!(
            detect_install_scope_from_facts(&facts("/usr/local/bin/cadman")),
            InstallScope::System
        );
    }

    #[test]
    fn usr_bin_cadman_is_system_scope() {
        assert_eq!(
            detect_install_scope_from_facts(&facts("/usr/bin/cadman")),
            InstallScope::System
        );
    }

    #[test]
    fn explicit_etc_cadman_config_is_system_scope() {
        let mut facts = facts("target/debug/cadman");
        facts.explicit_config_path = Some(PathBuf::from("/etc/cadman/config.toml"));

        assert_eq!(
            detect_install_scope_from_facts(&facts),
            InstallScope::System
        );
    }

    #[test]
    fn root_like_executable_alone_is_still_user_scope() {
        assert_eq!(
            detect_install_scope_from_facts(&facts("/root/cadman")),
            InstallScope::User
        );
    }

    #[test]
    fn container_env_is_container_scope() {
        let mut facts = facts("target/debug/cadman");
        facts.container_env = true;

        assert_eq!(
            detect_install_scope_from_facts(&facts),
            InstallScope::Container
        );
    }

    #[test]
    fn system_install_wins_over_container_env() {
        let mut facts = facts("/usr/bin/cadman");
        facts.container_env = true;

        assert_eq!(
            detect_install_scope_from_facts(&facts),
            InstallScope::System
        );
    }
}
