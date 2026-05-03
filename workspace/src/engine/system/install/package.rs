use std::env;
use std::path::{Path, PathBuf};

use crate::engine::models::runtime::RunMode;

use super::types::InstallTarget;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageManager {
    Apt,
    Dnf,
    Yum,
    Pacman,
    Zypper,
    Apk,
}

impl PackageManager {
    pub fn binary(self) -> &'static str {
        match self {
            Self::Apt => "apt-get",
            Self::Dnf => "dnf",
            Self::Yum => "yum",
            Self::Pacman => "pacman",
            Self::Zypper => "zypper",
            Self::Apk => "apk",
        }
    }

    pub fn install_args(self, package: &str) -> Vec<String> {
        match self {
            Self::Apt => vec!["install", "-y", package],
            Self::Dnf | Self::Yum => vec!["install", "-y", package],
            Self::Pacman => vec!["-S", "--noconfirm", package],
            Self::Zypper => vec!["install", "-y", package],
            Self::Apk => vec!["add", package],
        }
        .into_iter()
        .map(str::to_string)
        .collect()
    }

    pub fn run_mode(self, _target: InstallTarget) -> RunMode {
        RunMode::Root
    }
}

pub fn detect_package_manager() -> Option<PackageManager> {
    [
        PackageManager::Apt,
        PackageManager::Dnf,
        PackageManager::Yum,
        PackageManager::Pacman,
        PackageManager::Zypper,
        PackageManager::Apk,
    ]
    .into_iter()
    .find(|manager| binary_exists(manager.binary()))
}

pub fn binary_exists(binary: &str) -> bool {
    find_binary(binary).is_some()
}

pub fn find_binary(binary: &str) -> Option<PathBuf> {
    let candidate = Path::new(binary);
    if candidate.components().count() > 1 {
        return candidate.is_file().then(|| candidate.to_path_buf());
    }

    let path = env::var_os("PATH")?;
    env::split_paths(&path)
        .map(|dir| dir.join(binary))
        .find(|path| path.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_manager_install_args() {
        assert_eq!(
            PackageManager::Apt.install_args("podman"),
            vec!["install", "-y", "podman"]
        );
        assert_eq!(
            PackageManager::Pacman.install_args("caddy"),
            vec!["-S", "--noconfirm", "caddy"]
        );
    }
}
