use std::path::PathBuf;

use crate::engine::{
    ErrorCode, Result,
    constants::{
        CADDY_SERVICE_NAME, CADMAN_GROUP_NAME, CADMAN_USER_NAME, PODMAN_SERVICE_NAME,
        paths,
    },
    models::runtime::RunMode,
};

use super::{
    common::{expected_binary_path, sudoers_content, systemd_service_content},
    types::{
        InstallAction, InstallFacts, InstallPlan, InstallStep, InstallTarget,
        install_target_from_facts,
    },
};

pub fn build_install_plan(facts: InstallFacts) -> Result<InstallPlan> {
    let target = install_target_from_facts(&facts);
    let mut steps = Vec::new();

    add_dependency_steps(&mut steps, target, &facts)?;

    match target {
        InstallTarget::User | InstallTarget::ContainerUser => {
            add_user_install_steps(&mut steps, &facts, target)?;
        }
        InstallTarget::System => add_system_install_steps(&mut steps, &facts)?,
        InstallTarget::ContainerSystem => add_container_system_install_steps(&mut steps, &facts)?,
    }

    Ok(InstallPlan {
        target,
        dry_run: facts.dry_run,
        steps,
    })
}

fn add_dependency_steps(
    steps: &mut Vec<InstallStep>,
    target: InstallTarget,
    facts: &InstallFacts,
) -> Result<()> {
    for package in missing_dependencies(facts) {
        match target {
            InstallTarget::System => {
                if let Some(manager) = facts.package_manager {
                    steps.push(InstallStep::new(
                        format!("install {package}"),
                        InstallAction::InstallPackage {
                            manager,
                            package: package.to_string(),
                            run_mode: manager.run_mode(target),
                        },
                    ));
                } else if facts.dry_run {
                    steps.push(InstallStep::new(
                        format!("missing {package}"),
                        InstallAction::Noop {
                            note: format!("{package} is missing and no package manager was found"),
                        },
                    ));
                } else {
                    return Err(ErrorCode::ProcessFailure
                        .error()
                        .with_context("dependency", package)
                        .with_context("reason", "package manager not found"));
                }
            }
            InstallTarget::User | InstallTarget::ContainerUser | InstallTarget::ContainerSystem => {
                steps.push(InstallStep::new(
                    format!("missing {package}"),
                    InstallAction::Noop {
                        note: format!("{package} is missing; install it manually for this target"),
                    },
                ));
            }
        }
    }

    Ok(())
}

fn missing_dependencies(facts: &InstallFacts) -> Vec<&'static str> {
    let mut packages = Vec::new();
    if !facts.podman_installed {
        packages.push(PODMAN_SERVICE_NAME);
    }
    if !facts.caddy_installed {
        packages.push(CADDY_SERVICE_NAME);
    }
    packages
}

fn add_user_install_steps(
    steps: &mut Vec<InstallStep>,
    facts: &InstallFacts,
    target: InstallTarget,
) -> Result<()> {
    for path in [
        &facts.config_dir,
        &facts.cache_dir,
        &facts.state_dir,
        &facts.log_dir,
        &facts.user_bin_dir,
    ] {
        steps.push(ensure_dir(path.clone(), 0o755, RunMode::Current));
    }

    steps.push(install_binary(
        facts.current_exe.clone(),
        expected_binary_path(target)?,
        0o755,
        RunMode::Current,
    ));

    Ok(())
}

fn add_system_install_steps(steps: &mut Vec<InstallStep>, facts: &InstallFacts) -> Result<()> {
    let dirs = system_dirs();
    for path in &dirs {
        steps.push(ensure_dir(path.clone(), 0o755, RunMode::Root));
    }

    if !facts.cadman_group_exists {
        steps.push(InstallStep::new(
            "create cadman group",
            InstallAction::EnsureGroup {
                name: CADMAN_GROUP_NAME.to_string(),
            },
        ));
    }

    if !facts.cadman_user_exists {
        steps.push(InstallStep::new(
            "create cadman user",
            InstallAction::EnsureUser {
                name: CADMAN_USER_NAME.to_string(),
                group: CADMAN_GROUP_NAME.to_string(),
                home: paths::system_state_dir(),
                shell: "/usr/sbin/nologin".to_string(),
            },
        ));
    }

    for path in &dirs {
        steps.push(set_owner(path.clone(), CADMAN_USER_NAME, CADMAN_GROUP_NAME));
    }

    let binary = expected_binary_path(InstallTarget::System)?;
    steps.push(install_binary(
        facts.current_exe.clone(),
        binary.clone(),
        0o755,
        RunMode::Root,
    ));

    if !facts.current_user_in_cadman_group {
        if let Some(user) = &facts.current_user {
            steps.push(InstallStep::new(
                "add current user to cadman group",
                InstallAction::AddUserToGroup {
                    user: user.clone(),
                    group: CADMAN_GROUP_NAME.to_string(),
                },
            ));
        }
    }

    steps.push(InstallStep::new(
        "write cadman sudoers",
        InstallAction::WriteFile {
            path: paths::sudoers_file_path(),
            content: sudoers_content(),
            mode: 0o440,
            run_mode: RunMode::Root,
        },
    ));

    if facts.systemd_available {
        steps.push(InstallStep::new(
            "write cadman systemd unit",
            InstallAction::WriteFile {
                path: paths::systemd_unit_path(),
                content: systemd_service_content(&binary, InstallTarget::System),
                mode: 0o644,
                run_mode: RunMode::Root,
            },
        ));
    }

    Ok(())
}

fn add_container_system_install_steps(
    steps: &mut Vec<InstallStep>,
    facts: &InstallFacts,
) -> Result<()> {
    let dirs = system_dirs();
    for path in &dirs {
        steps.push(ensure_dir(path.clone(), 0o755, RunMode::Root));
    }

    if facts.effective_uid_is_root {
        if !facts.cadman_group_exists {
            steps.push(InstallStep::new(
                "create cadman group",
                InstallAction::EnsureGroup {
                    name: CADMAN_GROUP_NAME.to_string(),
                },
            ));
        }

        if !facts.cadman_user_exists {
            steps.push(InstallStep::new(
                "create cadman user",
                InstallAction::EnsureUser {
                    name: CADMAN_USER_NAME.to_string(),
                    group: CADMAN_GROUP_NAME.to_string(),
                    home: paths::system_state_dir(),
                    shell: "/usr/sbin/nologin".to_string(),
                },
            ));
        }

        if facts.cadman_group_exists && facts.cadman_user_exists {
            for path in &dirs {
                steps.push(set_owner(path.clone(), CADMAN_USER_NAME, CADMAN_GROUP_NAME));
            }
        }

        steps.push(install_binary(
            facts.current_exe.clone(),
            expected_binary_path(InstallTarget::ContainerSystem)?,
            0o755,
            RunMode::Root,
        ));
    } else {
        steps.push(InstallStep::new(
            "container system install requires root inside the container",
            InstallAction::Noop {
                note: "run as root inside the container to install to system-style paths"
                    .to_string(),
            },
        ));
    }

    Ok(())
}

fn system_dirs() -> Vec<PathBuf> {
    vec![
        paths::system_config_dir(),
        paths::system_state_dir(),
        paths::system_cache_dir(),
        paths::system_log_dir(),
    ]
}

fn ensure_dir(path: PathBuf, mode: u32, run_mode: RunMode) -> InstallStep {
    InstallStep::new(
        format!("create {}", path.display()),
        InstallAction::EnsureDir {
            path,
            mode,
            run_mode,
        },
    )
}

fn install_binary(
    source: PathBuf,
    destination: PathBuf,
    mode: u32,
    run_mode: RunMode,
) -> InstallStep {
    InstallStep::new(
        format!("install {}", destination.display()),
        InstallAction::InstallBinary {
            source,
            destination,
            mode,
            run_mode,
        },
    )
}

fn set_owner(path: PathBuf, owner: &str, group: &str) -> InstallStep {
    InstallStep::new(
        format!("set owner {}", path.display()),
        InstallAction::SetOwner {
            path,
            owner: owner.to_string(),
            group: group.to_string(),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::constants::BIN_NAME;
    use crate::engine::models::runtime::InstallScope;
    use crate::engine::system::install::package::PackageManager;

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
            package_manager: Some(PackageManager::Apt),
            container_system_paths_writable: false,
            explicit_container_system: false,
        }
    }

    #[test]
    fn user_install_plan_creates_user_dirs_and_binary_step() {
        let plan = build_install_plan(facts(InstallScope::User)).unwrap();

        assert_eq!(plan.target, InstallTarget::User);
        assert!(
            plan.steps
                .iter()
                .any(|step| matches!(step.action, InstallAction::EnsureDir { ref path, .. } if path == &PathBuf::from("/tmp/config")))
        );
        assert!(
            plan.steps
                .iter()
                .any(|step| matches!(step.action, InstallAction::InstallBinary { ref destination, .. } if destination.file_name().is_some_and(|name| name == BIN_NAME)))
        );
    }

    #[test]
    fn system_install_plan_creates_system_guards() {
        let mut facts = facts(InstallScope::System);
        facts.systemd_available = true;
        let plan = build_install_plan(facts).unwrap();

        assert!(
            plan.steps
                .iter()
                .any(|step| matches!(step.action, InstallAction::EnsureGroup { .. }))
        );
        assert!(
            plan.steps
                .iter()
                .any(|step| matches!(step.action, InstallAction::EnsureUser { .. }))
        );
        assert!(
            plan.steps
                .iter()
                .any(|step| matches!(step.action, InstallAction::InstallBinary { ref destination, .. } if destination == &PathBuf::from("/usr/local/bin/cadman")))
        );
        assert!(
            plan.steps
                .iter()
                .any(|step| matches!(step.action, InstallAction::WriteFile { ref path, .. } if path == &paths::sudoers_file_path()))
        );
        assert!(
            plan.steps
                .iter()
                .any(|step| matches!(step.action, InstallAction::WriteFile { ref path, .. } if path == &paths::systemd_unit_path()))
        );
    }

    #[test]
    fn container_user_install_plan_omits_host_guards() {
        let plan = build_install_plan(facts(InstallScope::Container)).unwrap();

        assert_eq!(plan.target, InstallTarget::ContainerUser);
        assert!(!plan.steps.iter().any(|step| matches!(
            step.action,
            InstallAction::WriteFile { .. }
                | InstallAction::EnsureUser { .. }
                | InstallAction::EnsureGroup { .. }
        )));
    }

    #[test]
    fn container_system_plan_omits_sudoers_systemd_and_loginctl() {
        let mut facts = facts(InstallScope::Container);
        facts.effective_uid_is_root = true;
        facts.container_system_paths_writable = true;
        let plan = build_install_plan(facts).unwrap();

        assert_eq!(plan.target, InstallTarget::ContainerSystem);
        assert!(
            !plan
                .steps
                .iter()
                .any(|step| matches!(step.action, InstallAction::WriteFile { .. }))
        );
        assert!(
            !plan
                .steps
                .iter()
                .any(|step| step.label.contains("loginctl"))
        );
    }
}
