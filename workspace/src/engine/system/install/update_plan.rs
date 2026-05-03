use std::path::Path;

use crate::engine::{
    Result,
    constants::{CADMAN_GROUP_NAME, CADMAN_USER_NAME, paths},
    models::runtime::RunMode,
};

use super::{
    common::{expected_binary_path, sudoers_content},
    types::{
        InstallAction, InstallFacts, InstallPlan, InstallStep, InstallTarget,
        install_target_from_facts,
    },
};

pub fn build_update_plan(facts: InstallFacts) -> Result<InstallPlan> {
    let target = install_target_from_facts(&facts);
    let mut steps = Vec::new();
    let expected = expected_binary_path(target)?;

    let Some(installed) = facts.installed_binary_path.as_ref() else {
        steps.push(noop(
            "not installed",
            "cadman is not installed; run install first",
        ));
        return Ok(plan(target, facts.dry_run, steps));
    };

    if installed == Path::new("/usr/bin/cadman") {
        steps.push(noop(
            "package managed install",
            "cadman is installed at /usr/bin/cadman; update through the package manager",
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

    if facts.current_exe == *installed {
        steps.push(noop(
            "already current",
            "the running cadman binary is already installed",
        ));
    } else {
        steps.push(InstallStep::new(
            format!("update {}", installed.display()),
            InstallAction::InstallBinary {
                source: facts.current_exe.clone(),
                destination: installed.clone(),
                mode: 0o755,
                run_mode: write_run_mode(target),
            },
        ));
    }

    if target == InstallTarget::System {
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

        steps.push(InstallStep::new(
            "write cadman sudoers",
            InstallAction::WriteFile {
                path: paths::sudoers_file_path(),
                content: sudoers_content(),
                mode: 0o440,
                run_mode: RunMode::Root,
            },
        ));
    }

    Ok(plan(target, facts.dry_run, steps))
}

fn write_run_mode(target: InstallTarget) -> RunMode {
    match target {
        InstallTarget::User | InstallTarget::ContainerUser => RunMode::Current,
        InstallTarget::System | InstallTarget::ContainerSystem => RunMode::Root,
    }
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
    use std::path::PathBuf;

    fn facts() -> InstallFacts {
        InstallFacts {
            requested_run_mode: RunMode::Current,
            effective_run_mode: RunMode::Current,
            install_scope: InstallScope::User,
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
    fn update_plan_handles_not_installed() {
        let plan = build_update_plan(facts()).unwrap();

        assert!(
            plan.steps
                .iter()
                .any(|step| matches!(step.action, InstallAction::Noop { ref note } if note.contains("run install first")))
        );
    }
}
