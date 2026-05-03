use std::path::PathBuf;

use crate::engine::{Result, constants::{SYSTEM_UNIT_NAME, paths}};

pub fn system_unit_path() -> PathBuf {
    paths::systemd_unit_path()
}

pub fn user_unit_path() -> Result<PathBuf> {
    paths::user_systemd_unit_path()
}

pub fn systemctl_args(user_service: bool, verb: &str, unit: Option<&str>) -> Vec<String> {
    let mut args = Vec::new();

    if user_service {
        args.push("--user".to_string());
    }

    args.push(verb.to_string());

    if matches!(verb, "is-active" | "is-enabled") {
        args.push("--quiet".to_string());
    }

    if verb != "daemon-reload" {
        args.push(unit.unwrap_or(SYSTEM_UNIT_NAME).to_string());
    }

    args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn systemctl_user_args_include_user_flag() {
        assert_eq!(
            systemctl_args(true, "restart", Some(SYSTEM_UNIT_NAME)),
            vec!["--user", "restart", SYSTEM_UNIT_NAME]
        );
    }

    #[test]
    fn active_and_enabled_checks_are_quiet() {
        assert_eq!(
            systemctl_args(false, "is-active", None),
            vec!["is-active", "--quiet", SYSTEM_UNIT_NAME]
        );
        assert_eq!(
            systemctl_args(false, "is-enabled", None),
            vec!["is-enabled", "--quiet", SYSTEM_UNIT_NAME]
        );
    }

    #[test]
    fn daemon_reload_does_not_require_unit() {
        assert_eq!(
            systemctl_args(false, "daemon-reload", None),
            vec!["daemon-reload"]
        );
    }

    #[test]
    fn unit_paths_end_with_cadman_service() {
        assert!(system_unit_path().ends_with(SYSTEM_UNIT_NAME));
        assert!(user_unit_path().unwrap().ends_with(SYSTEM_UNIT_NAME));
    }
}
