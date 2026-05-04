use serde::Serialize;

use crate::{
    core::{Context, CoreResult},
    engine::{
        ErrorCode,
        capabilities::diagnostics::{DiagnosticsReport, collect_diagnostics},
        system::install::{
            AppliedInstallStep, AppliedInstallStepStatus, InstallAction, InstallFacts, InstallPlan,
            InstallStep, apply_plan, build_install_plan, build_uninstall_plan, build_update_plan,
            collect_install_facts,
        },
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelfAction {
    Install,
    Update,
    Uninstall,
}

#[derive(Debug, Clone, Serialize)]
pub struct SelfPlanReport {
    pub action: String,
    pub target: String,
    pub dry_run: bool,
    pub confirmed: bool,
    pub steps: Vec<SelfStepReport>,
    pub applied: Vec<SelfAppliedStepReport>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SelfStepReport {
    pub label: String,
    pub action: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SelfAppliedStepReport {
    pub label: String,
    pub status: String,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SelfHealthcheckReport {
    pub facts: SelfFactsReport,
    pub diagnostics: DiagnosticsReport,
}

#[derive(Debug, Clone, Serialize)]
pub struct SelfFactsReport {
    pub requested_run_mode: String,
    pub effective_run_mode: String,
    pub install_scope: String,
    pub current_exe: String,
    pub installed_binary_path: Option<String>,
    pub config_dir: String,
    pub cache_dir: String,
    pub state_dir: String,
    pub log_dir: String,
    pub user_bin_dir: String,
    pub current_user: Option<String>,
    pub current_user_in_cadman_group: bool,
    pub current_user_in_admin_group: bool,
    pub effective_uid_is_root: bool,
    pub podman_installed: bool,
    pub caddy_installed: bool,
    pub cadman_group_exists: bool,
    pub cadman_user_exists: bool,
    pub systemd_available: bool,
    pub package_manager: Option<String>,
    pub container_system_paths_writable: bool,
}

pub async fn render_install(ctx: &Context) -> CoreResult<()> {
    render_plan(ctx, SelfAction::Install).await
}

pub async fn render_update(ctx: &Context) -> CoreResult<()> {
    render_plan(ctx, SelfAction::Update).await
}

pub async fn render_uninstall(ctx: &Context) -> CoreResult<()> {
    render_plan(ctx, SelfAction::Uninstall).await
}

pub async fn render_healthcheck(ctx: &Context) -> CoreResult<()> {
    let report = healthcheck(ctx)?;
    let output = ctx
        .ui()
        .new_output_content()
        .json(&report)
        .title("Cadman self healthcheck")
        .key_value("Install Scope", &report.facts.install_scope)
        .key_value("Effective RunMode", &report.facts.effective_run_mode)
        .key_value("Current Binary", &report.facts.current_exe)
        .key_value(
            "Installed Binary",
            report.facts.installed_binary_path.as_deref().unwrap_or("-"),
        )
        .key_value("Podman", report.facts.podman_installed)
        .key_value("Caddy", report.facts.caddy_installed)
        .key_value("Cadman User", report.facts.cadman_user_exists)
        .key_value("Cadman Group", report.facts.cadman_group_exists)
        .key_value(
            "Current User In Cadman Group",
            report.facts.current_user_in_cadman_group,
        )
        .key_value("Systemd", report.facts.systemd_available);

    ctx.ui().print(&output)
}

pub async fn self_install(ctx: &Context) -> CoreResult<SelfPlanReport> {
    plan_and_maybe_apply(ctx, SelfAction::Install).await
}

pub async fn self_update(ctx: &Context) -> CoreResult<SelfPlanReport> {
    plan_and_maybe_apply(ctx, SelfAction::Update).await
}

pub async fn self_uninstall(ctx: &Context) -> CoreResult<SelfPlanReport> {
    plan_and_maybe_apply(ctx, SelfAction::Uninstall).await
}

pub fn healthcheck(ctx: &Context) -> CoreResult<SelfHealthcheckReport> {
    Ok(SelfHealthcheckReport {
        facts: facts_report(&collect_install_facts(ctx.runtime())?),
        diagnostics: collect_diagnostics(ctx.runtime())?,
    })
}

async fn render_plan(ctx: &Context, action: SelfAction) -> CoreResult<()> {
    let report = plan_and_maybe_apply(ctx, action).await?;
    let mut output = ctx
        .ui()
        .new_output_content()
        .json(&report)
        .title(format!("Cadman self {}", action.as_str()))
        .key_value("Target", &report.target)
        .key_value("Dry Run", report.dry_run)
        .key_value("Confirmed", report.confirmed)
        .key_value("Steps", report.steps.len())
        .key_value("Applied", report.applied.len());

    if !report.steps.is_empty() {
        output = output.table(None, steps_table(&report.steps));
    }

    ctx.ui().print(&output)
}

async fn plan_and_maybe_apply(ctx: &Context, action: SelfAction) -> CoreResult<SelfPlanReport> {
    let facts = collect_install_facts(ctx.runtime())?;
    let plan = match action {
        SelfAction::Install => build_install_plan(facts)?,
        SelfAction::Update => build_update_plan(facts)?,
        SelfAction::Uninstall => build_uninstall_plan(facts)?,
    };

    let confirmed = confirm_if_needed(ctx, action, &plan)?;
    let applied = if confirmed {
        apply_plan(&plan).await?
    } else {
        Vec::new()
    };

    Ok(SelfPlanReport {
        action: action.as_str().to_string(),
        target: format!("{:?}", plan.target),
        dry_run: plan.dry_run,
        confirmed,
        steps: plan.steps.iter().map(step_report).collect(),
        applied: applied.iter().map(applied_report).collect(),
    })
}

fn confirm_if_needed(ctx: &Context, action: SelfAction, plan: &InstallPlan) -> CoreResult<bool> {
    if plan.dry_run || plan.steps.is_empty() {
        return Ok(true);
    }

    if !ctx.is_interactive() || ctx.auto_yes() {
        return Ok(true);
    }

    let confirmed = ctx.ui().confirm(
        &format!("Apply cadman self {} plan?", action.as_str()),
        false,
    )?;
    if confirmed {
        Ok(true)
    } else {
        Err(ErrorCode::UserCancelled.error())
    }
}

fn facts_report(facts: &InstallFacts) -> SelfFactsReport {
    SelfFactsReport {
        requested_run_mode: facts.requested_run_mode.to_string(),
        effective_run_mode: facts.effective_run_mode.to_string(),
        install_scope: facts.install_scope.to_string(),
        current_exe: facts.current_exe.display().to_string(),
        installed_binary_path: facts
            .installed_binary_path
            .as_ref()
            .map(|path| path.display().to_string()),
        config_dir: facts.config_dir.display().to_string(),
        cache_dir: facts.cache_dir.display().to_string(),
        state_dir: facts.state_dir.display().to_string(),
        log_dir: facts.log_dir.display().to_string(),
        user_bin_dir: facts.user_bin_dir.display().to_string(),
        current_user: facts.current_user.clone(),
        current_user_in_cadman_group: facts.current_user_in_cadman_group,
        current_user_in_admin_group: facts.current_user_in_admin_group,
        effective_uid_is_root: facts.effective_uid_is_root,
        podman_installed: facts.podman_installed,
        caddy_installed: facts.caddy_installed,
        cadman_group_exists: facts.cadman_group_exists,
        cadman_user_exists: facts.cadman_user_exists,
        systemd_available: facts.systemd_available,
        package_manager: facts.package_manager.map(|manager| format!("{manager:?}")),
        container_system_paths_writable: facts.container_system_paths_writable,
    }
}

fn step_report(step: &InstallStep) -> SelfStepReport {
    SelfStepReport {
        label: step.label.clone(),
        action: action_label(&step.action),
    }
}

fn applied_report(step: &AppliedInstallStep) -> SelfAppliedStepReport {
    SelfAppliedStepReport {
        label: step.label.clone(),
        status: match step.status {
            AppliedInstallStepStatus::Applied => "applied".to_string(),
            AppliedInstallStepStatus::Skipped => "skipped".to_string(),
        },
        note: step.note.clone(),
    }
}

fn action_label(action: &InstallAction) -> String {
    match action {
        InstallAction::Noop { note } => format!("noop: {note}"),
        InstallAction::EnsureDir { path, .. } => format!("ensure-dir {}", path.display()),
        InstallAction::EnsureGroup { name } => format!("ensure-group {name}"),
        InstallAction::EnsureUser { name, .. } => format!("ensure-user {name}"),
        InstallAction::AddUserToGroup { user, group } => format!("add {user} to {group}"),
        InstallAction::RemoveUserFromGroup { user, group } => {
            format!("remove {user} from {group}")
        }
        InstallAction::RemoveUser { name } => format!("remove-user {name}"),
        InstallAction::RemoveGroup { name } => format!("remove-group {name}"),
        InstallAction::SetOwner { path, owner, group } => {
            format!("set-owner {} {owner}:{group}", path.display())
        }
        InstallAction::InstallPackage {
            manager, package, ..
        } => {
            format!("install-package {manager:?} {package}")
        }
        InstallAction::InstallBinary {
            source,
            destination,
            ..
        } => {
            format!(
                "install-binary {} -> {}",
                source.display(),
                destination.display()
            )
        }
        InstallAction::WriteFile { path, .. } => format!("write-file {}", path.display()),
        InstallAction::RemoveFile { path, .. } => format!("remove-file {}", path.display()),
    }
}

fn steps_table(steps: &[SelfStepReport]) -> scriba::Table {
    let rows = steps
        .iter()
        .map(|step| vec![step.label.clone(), step.action.clone()])
        .collect();

    scriba::Table::new(vec!["STEP".to_string(), "ACTION".to_string()], rows)
}

impl SelfAction {
    fn as_str(self) -> &'static str {
        match self {
            Self::Install => "install",
            Self::Update => "update",
            Self::Uninstall => "uninstall",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::engine::models::runtime::{InstallScope, Runtime};

    fn dry_run_ctx() -> Context {
        let mut runtime = Runtime::new();
        runtime.set_install_scope(InstallScope::User);
        runtime.set_dry_run(true);
        Context::new(runtime)
    }

    #[tokio::test]
    async fn self_install_dry_run_renders_plan() {
        let report = self_install(&dry_run_ctx()).await.unwrap();

        assert!(report.dry_run);
        assert!(!report.steps.is_empty());
        assert_eq!(report.applied.len(), report.steps.len());
    }

    #[tokio::test]
    async fn self_update_dry_run_renders_plan() {
        let report = self_update(&dry_run_ctx()).await.unwrap();

        assert!(report.dry_run);
        assert!(!report.steps.is_empty());
    }

    #[tokio::test]
    async fn self_uninstall_dry_run_renders_plan() {
        let report = self_uninstall(&dry_run_ctx()).await.unwrap();

        assert!(report.dry_run);
        assert!(!report.steps.is_empty());
    }

    #[test]
    fn healthcheck_reports_install_facts() {
        let report = healthcheck(&dry_run_ctx()).unwrap();

        assert!(!report.facts.current_exe.is_empty());
        assert!(!report.facts.config_dir.is_empty());
    }
}
