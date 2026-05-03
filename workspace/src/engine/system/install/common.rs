use std::path::{Path, PathBuf};

use crate::engine::{Result, constants::paths};

use super::types::InstallTarget;

pub fn expected_binary_path(target: InstallTarget) -> Result<PathBuf> {
    match target {
        InstallTarget::User | InstallTarget::ContainerUser => {
            Ok(paths::user_bin_dir()?.join("cadman"))
        }
        InstallTarget::System | InstallTarget::ContainerSystem => {
            Ok(PathBuf::from("/usr/local/bin/cadman"))
        }
    }
}

pub fn sudoers_content() -> String {
    "cadman ALL=(root) NOPASSWD: /usr/local/bin/cadman\n%cadman ALL=(root) NOPASSWD: /usr/local/bin/cadman\n".to_string()
}

pub fn systemd_service_content(binary: &Path, target: InstallTarget) -> String {
    let user = if target == InstallTarget::System {
        "User=cadman\nGroup=cadman\n"
    } else {
        ""
    };

    format!(
        "[Unit]\nDescription=Cadman\nAfter=network-online.target\n\n[Service]\nType=simple\n{user}ExecStart={} serve\nRestart=on-failure\n\n[Install]\nWantedBy=multi-user.target\n",
        binary.display()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sudoers_content_only_references_cadman_binary() {
        let content = sudoers_content();
        assert!(content.contains("/usr/local/bin/cadman"));
        assert!(!content.contains("podman"));
        assert!(!content.contains("systemctl"));
        assert!(!content.contains("loginctl"));
    }
}
