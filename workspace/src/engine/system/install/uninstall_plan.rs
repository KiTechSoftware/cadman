use std::path::{Path, PathBuf};

use crate::engine::{
    Result,
    constants::{CADMAN_GROUP_NAME, CADMAN_USER_NAME, paths},
    models::runtime::RunMode,
};

use super::{
    common::expected_binary_path,
    types::{
        InstallAction, InstallFacts, InstallPlan, InstallStep, InstallTarget,
        install_target_from_facts,
    },
};

pub fn build_uninstall_plan(facts: InstallFacts) -> Result<InstallPlan> {
    let target = install_target_from_facts(&facts);
    let mut steps = Vec::new();
    let expected = expected_binary_path(target)?;

    let Some(installed) = facts.installed_binary_path.as_ref() else {
        steps.push(noop("not installed", "nothing to uninstall"));
        return Ok(plan(target, facts.dry_run, steps));
    };

    if installed == Path::new("/usr/bin/cadman") {
        steps.push(noop(
            "package managed install",
            "cadman is installed at /usr/bin/cadman; uninstall through the package manager",
        ));
        return Ok(plan(target, facts.dry_run, steps));
    }

    if installed != &expected {
        steps.push(noop(
            "unexpected install location",
            format!(
                "cadman is installed at {}; expected {}",
                installed.display(),
                expected.display()
            ),
        ));
        return Ok(plan(target, facts.dry_run, steps));
    }

    match target {
        InstallTarget::User | InstallTarget::ContainerUser => {
            steps.push(remove_file(installed.clone(), RunMode::Current));
        }
        InstallTarget::System => {
            steps.push(remove_file(paths::systemd_unit_path(), RunMode::Root));
            steps.push(remove_file(paths::sudoers_file_path(), RunMode::Root));

            if facts.current_user_in_cadman_group
                && let Some(user) = &facts.current_user
            {
                steps.push(InstallStep::new(
                    "remove current user from cadman group",
                    InstallAction::RemoveUserFromGroup {
                        user: user.clone(),
                        group: CADMAN_GROUP_NAME.to_string(),
                    },
                ));
            }

            steps.push(remove_file(installed.clone(), RunMode::Root));

            if facts.cadman_user_exists {
                steps.push(InstallStep::new(
                    "remove cadman user",
                    InstallAction::RemoveUser {
                        name: CADMAN_USER_NAME.to_string(),
                    },
                ));
            }

            if facts.cadman_group_exists {
                steps.push(InstallStep::new(
                    "remove cadman group",
                    InstallAction::RemoveGroup {
                        name: CADMAN_GROUP_NAME.to_string(),
                    },
                ));
            }
        }
        InstallTarget::ContainerSystem => {
            steps.push(remove_file(installed.clone(), RunMode::Root));

            if facts.effective_uid_is_root {
                if facts.cadman_user_exists {
                    steps.push(InstallStep::new(
                        "remove cadman user",
                        InstallAction::RemoveUser {
                            name: CADMAN_USER_NAME.to_string(),
                        },
                    ));
                }

                if facts.cadman_group_exists {
                    steps.push(InstallStep::new(
                        "remove cadman group",
                        InstallAction::RemoveGroup {
                            name: CADMAN_GROUP_NAME.to_string(),
                        },
                    ));
                }
            }
        }
    }

    Ok(plan(target, facts.dry_run, steps))
}

fn remove_file(path: PathBuf, run_mode: RunMode) -> InstallStep {
    InstallStep::new(
        format!("remove {}", path.display()),
        InstallAction::RemoveFile { path, run_mode },
    )
}

fn noop(label: impl Into<String>, note: impl Into<String>) -> InstallStep {
    InstallStep::new(label, InstallAction::Noop { note: note.into() })
}

fn plan(target: InstallTarget, dry_run: bool, steps: Vec<InstallStep>) -> InstallPlan {
    InstallPlan {
        target,
        dry_run,
        steps,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::models::runtime::InstallScope;

    fn facts() -> InstallFacts {
        InstallFacts {
            requested_run_mode: RunMode::Current,
            effective_run_mode: RunMode::Current,
            install_scope: InstallScope::System,
            dry_run: false,
            current_exe: PathBuf::from("/tmp/cadman"),
            installed_binary_path: Some(PathBuf::from("/usr/local/bin/cadman")),
            config_dir: PathBuf::from("/etc/cadman"),
            cache_dir: PathBuf::from("/var/cache/cadman"),
            state_dir: PathBuf::from("/var/lib/cadman"),
            log_dir: PathBuf::from("/var/log/cadman"),
            user_bin_dir: PathBuf::from("/tmp/bin"),
            current_user: Some("alice".to_string()),
            current_user_in_cadman_group: true,
            current_user_in_admin_group: false,
            effective_uid_is_root: false,
            podman_installed: true,
            caddy_installed: true,
            cadman_group_exists: true,
            cadman_user_exists: true,
            systemd_available: true,
            package_manager: None,
            container_system_paths_writable: false,
            explicit_container_system: false,
        }
    }

    #[test]
    fn uninstall_plan_does_not_remove_data_dirs() {
        let plan = build_uninstall_plan(facts()).unwrap();

        assert!(!plan.steps.iter().any(|step| {
            matches!(step.action, InstallAction::RemoveFile { ref path, .. } if path == &PathBuf::from("/etc/cadman")
                || path == &PathBuf::from("/var/cache/cadman")
                || path == &PathBuf::from("/var/lib/cadman")
                || path == &PathBuf::from("/var/log/cadman"))
        }));
    }
}
