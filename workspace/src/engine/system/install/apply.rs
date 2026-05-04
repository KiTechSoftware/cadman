use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

use veltrix::os::unistd::{self, Uid};

use crate::engine::{
    ErrorCode, Result,
    models::runtime::RunMode,
    system::process::{Process, ProcessOutput},
};

use super::types::{InstallAction, InstallPlan, InstallStep};

#[derive(Debug, Clone)]
pub struct AppliedInstallStep {
    pub label: String,
    pub status: AppliedInstallStepStatus,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppliedInstallStepStatus {
    Applied,
    Skipped,
}

pub async fn apply_plan(plan: &InstallPlan) -> Result<Vec<AppliedInstallStep>> {
    let mut applied = Vec::new();

    for step in &plan.steps {
        match &step.action {
            InstallAction::Noop { note } => {
                applied.push(skipped(step, Some(note.clone())));
            }
            action if plan.dry_run => {
                applied.push(skipped(
                    step,
                    Some(format!("dry run: would {}", action_description(action))),
                ));
            }
            InstallAction::EnsureDir {
                path,
                mode,
                run_mode,
            } => {
                ensure_run_mode(*run_mode)?;
                ensure_dir(path, *mode)?;
                applied.push(done(step));
            }
            InstallAction::InstallBinary {
                source,
                destination,
                mode,
                run_mode,
            } => {
                ensure_run_mode(*run_mode)?;
                install_binary(source, destination, *mode)?;
                applied.push(done(step));
            }
            InstallAction::WriteFile {
                path,
                content,
                mode,
                run_mode,
            } => {
                ensure_run_mode(*run_mode)?;
                write_file(path, content, *mode)?;
                applied.push(done(step));
            }
            InstallAction::RemoveFile { path, run_mode } => {
                ensure_run_mode(*run_mode)?;
                if remove_file(path)? {
                    applied.push(done(step));
                } else {
                    applied.push(skipped(step, Some("file does not exist".to_string())));
                }
            }
            InstallAction::EnsureGroup { name } => {
                ensure_run_mode(RunMode::Root)?;
                run("groupadd", vec![name.clone()]).await?;
                applied.push(done(step));
            }
            InstallAction::EnsureUser {
                name,
                group,
                home,
                shell,
            } => {
                ensure_run_mode(RunMode::Root)?;
                run(
                    "useradd",
                    vec![
                        "--system".to_string(),
                        "--gid".to_string(),
                        group.clone(),
                        "--home-dir".to_string(),
                        home.display().to_string(),
                        "--shell".to_string(),
                        shell.clone(),
                        "--no-create-home".to_string(),
                        name.clone(),
                    ],
                )
                .await?;
                applied.push(done(step));
            }
            InstallAction::AddUserToGroup { user, group } => {
                ensure_run_mode(RunMode::Root)?;
                run(
                    "usermod",
                    vec!["-aG".to_string(), group.clone(), user.clone()],
                )
                .await?;
                applied.push(done(step));
            }
            InstallAction::RemoveUserFromGroup { user, group } => {
                ensure_run_mode(RunMode::Root)?;
                run(
                    "gpasswd",
                    vec!["-d".to_string(), user.clone(), group.clone()],
                )
                .await?;
                applied.push(done(step));
            }
            InstallAction::RemoveUser { name } => {
                ensure_run_mode(RunMode::Root)?;
                run("userdel", vec![name.clone()]).await?;
                applied.push(done(step));
            }
            InstallAction::RemoveGroup { name } => {
                ensure_run_mode(RunMode::Root)?;
                run("groupdel", vec![name.clone()]).await?;
                applied.push(done(step));
            }
            InstallAction::SetOwner { path, owner, group } => {
                ensure_run_mode(RunMode::Root)?;
                run(
                    "chown",
                    vec![format!("{owner}:{group}"), path.display().to_string()],
                )
                .await?;
                applied.push(done(step));
            }
            InstallAction::InstallPackage {
                manager,
                package,
                run_mode,
            } => {
                ensure_run_mode(*run_mode)?;
                run(manager.binary(), manager.install_args(package)).await?;
                applied.push(done(step));
            }
        }
    }

    Ok(applied)
}

fn ensure_run_mode(run_mode: RunMode) -> Result<()> {
    if run_mode == RunMode::Root && unistd::geteuid() != Uid::from_raw(0) {
        return Err(ErrorCode::PermissionRootRequired
            .error()
            .with_context("mode", run_mode.as_str())
            .with_context(
                "reason",
                "install apply does not perform implicit sudo escalation",
            ));
    }

    Ok(())
}

fn ensure_dir(path: &Path, mode: u32) -> Result<()> {
    fs::create_dir_all(path)?;
    fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
    Ok(())
}

fn install_binary(source: &Path, destination: &Path, mode: u32) -> Result<()> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }

    let temp = temp_path(destination);
    fs::copy(source, &temp)?;
    fs::set_permissions(&temp, fs::Permissions::from_mode(mode))?;
    fs::rename(&temp, destination)?;
    Ok(())
}

fn write_file(path: &Path, content: &str, mode: u32) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let temp = temp_path(path);
    fs::write(&temp, content)?;
    fs::set_permissions(&temp, fs::Permissions::from_mode(mode))?;
    fs::rename(&temp, path)?;
    Ok(())
}

fn remove_file(path: &Path) -> Result<bool> {
    if !path.exists() {
        return Ok(false);
    }

    fs::remove_file(path)?;
    Ok(true)
}

async fn run(binary: &str, args: Vec<String>) -> Result<ProcessOutput> {
    Process::new(binary)
        .set_args(args)
        .set_mode(RunMode::Root)
        .run_async()
        .await?
        .into_result(binary)
}

fn temp_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "cadman-install".to_string());
    path.with_file_name(format!(".{file_name}.tmp.{}", std::process::id()))
}

fn done(step: &InstallStep) -> AppliedInstallStep {
    AppliedInstallStep {
        label: step.label.clone(),
        status: AppliedInstallStepStatus::Applied,
        note: None,
    }
}

fn skipped(step: &InstallStep, note: Option<String>) -> AppliedInstallStep {
    AppliedInstallStep {
        label: step.label.clone(),
        status: AppliedInstallStepStatus::Skipped,
        note,
    }
}

fn action_description(action: &InstallAction) -> &'static str {
    match action {
        InstallAction::Noop { .. } => "skip",
        InstallAction::EnsureDir { .. } => "create directory",
        InstallAction::EnsureGroup { .. } => "create group",
        InstallAction::EnsureUser { .. } => "create user",
        InstallAction::AddUserToGroup { .. } => "add user to group",
        InstallAction::RemoveUserFromGroup { .. } => "remove user from group",
        InstallAction::RemoveUser { .. } => "remove user",
        InstallAction::RemoveGroup { .. } => "remove group",
        InstallAction::SetOwner { .. } => "set owner",
        InstallAction::InstallPackage { .. } => "install package",
        InstallAction::InstallBinary { .. } => "install binary",
        InstallAction::WriteFile { .. } => "write file",
        InstallAction::RemoveFile { .. } => "remove file",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::system::install::types::{InstallPlan, InstallTarget};

    #[tokio::test]
    async fn dry_run_apply_skips_mutating_steps() {
        let plan = InstallPlan {
            target: InstallTarget::User,
            dry_run: true,
            steps: vec![InstallStep::new(
                "create test dir",
                InstallAction::EnsureDir {
                    path: PathBuf::from("/tmp/cadman-dry-run-test"),
                    mode: 0o755,
                    run_mode: RunMode::Current,
                },
            )],
        };

        let applied = apply_plan(&plan).await.unwrap();

        assert_eq!(applied[0].status, AppliedInstallStepStatus::Skipped);
        assert!(
            applied[0]
                .note
                .as_deref()
                .is_some_and(|note| note.contains("dry run"))
        );
    }

    #[tokio::test]
    async fn root_apply_without_effective_root_returns_permission_root_required() {
        if unistd::geteuid() == Uid::from_raw(0) {
            return;
        }

        let plan = InstallPlan {
            target: InstallTarget::System,
            dry_run: false,
            steps: vec![InstallStep::new(
                "create root dir",
                InstallAction::EnsureDir {
                    path: PathBuf::from("/usr/local/share/cadman-test"),
                    mode: 0o755,
                    run_mode: RunMode::Root,
                },
            )],
        };

        let err = apply_plan(&plan).await.unwrap_err();

        assert_eq!(err.code, ErrorCode::PermissionRootRequired);
    }
}
