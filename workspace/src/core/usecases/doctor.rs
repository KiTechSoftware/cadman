use scriba::Output;
use serde::Serialize;

use crate::{
    core::{Context, CoreResult},
    engine::capabilities::diagnostics::{DiagnosticsReport, collect_diagnostics},
};

#[derive(Debug, Clone, Serialize)]
pub struct DoctorReport {
    pub ok: bool,
    pub diagnostics: DiagnosticsReport,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

pub async fn render(ctx: &Context) -> CoreResult<()> {
    let report = doctor(ctx)?;
    let output = if structured(ctx) {
        Output::from_serializable(&report)
    } else {
        let output = Output::new()
            .title("Cadman doctor")
            .key_value("OK", report.ok)
            .key_value("Podman", report.diagnostics.podman_available)
            .key_value("Caddy", report.diagnostics.caddy_available)
            .key_value("systemd", report.diagnostics.systemd_available)
            .key_value("Cadman user", report.diagnostics.cadman_user_exists)
            .key_value("Cadman group", report.diagnostics.cadman_group_exists)
            .key_value(
                "User in cadman group",
                report.diagnostics.current_user_in_cadman_group,
            )
            .key_value(
                "User in admin group",
                report.diagnostics.current_user_in_admin_group,
            )
            .key_value("Install Scope", &report.diagnostics.install_scope)
            .key_value("Effective User", &report.diagnostics.effective_user)
            .key_value("Config", report.diagnostics.config_path.display())
            .key_value("Registry", report.diagnostics.registry_path.display())
            .key_value("State", report.diagnostics.state_path.display());

        if report.warnings.is_empty() && report.errors.is_empty() {
            output
        } else {
            output
                .key_value("Warnings", format_messages(&report.warnings))
                .key_value("Errors", format_messages(&report.errors))
        }
    };

    ctx.ui().print(&output)
}

pub fn doctor(ctx: &Context) -> CoreResult<DoctorReport> {
    let diagnostics = collect_diagnostics(ctx.runtime())?;
    Ok(report_from_diagnostics(diagnostics))
}

fn report_from_diagnostics(diagnostics: DiagnosticsReport) -> DoctorReport {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    if !diagnostics.podman_available {
        warnings.push("Podman is not available in PATH".to_string());
    }
    if !diagnostics.caddy_available {
        warnings.push("Caddy is not available in PATH".to_string());
    }
    if !diagnostics.systemd_available {
        warnings.push("systemd unit directory is not available".to_string());
    }

    if diagnostics.install_scope == "system" {
        if !diagnostics.cadman_user_exists {
            errors.push("cadman service user does not exist".to_string());
        }
        if !diagnostics.cadman_group_exists {
            errors.push("cadman group does not exist".to_string());
        }
        if !diagnostics.current_user_in_cadman_group && !diagnostics.current_user_in_admin_group {
            warnings.push("current user is not in cadman or admin group".to_string());
        }
    }

    DoctorReport {
        ok: errors.is_empty(),
        diagnostics,
        warnings,
        errors,
    }
}

fn structured(ctx: &Context) -> bool {
    ctx.runtime().options().output_format().is_structured()
        || ctx.runtime().options().output_envelope().is_json()
}

fn format_messages(messages: &[String]) -> String {
    if messages.is_empty() {
        "-".to_string()
    } else {
        messages.join("; ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::models::runtime::{InstallScope, Runtime};
    use std::path::PathBuf;

    fn diagnostics() -> DiagnosticsReport {
        DiagnosticsReport {
            podman_available: true,
            caddy_available: true,
            systemd_available: true,
            cadman_user_exists: true,
            cadman_group_exists: true,
            current_user_in_cadman_group: true,
            current_user_in_admin_group: false,
            install_scope: "system".to_string(),
            effective_user: "alice".to_string(),
            config_path: PathBuf::from("/etc/cadman/config.toml"),
            registry_path: PathBuf::from("/var/lib/cadman/registry.toml"),
            state_path: PathBuf::from("/var/lib/cadman/state.json"),
        }
    }

    #[test]
    fn doctor_report_is_ok_when_required_system_facts_exist() {
        let report = report_from_diagnostics(diagnostics());

        assert!(report.ok);
        assert!(report.errors.is_empty());
    }

    #[test]
    fn doctor_report_warns_for_missing_runtime_dependencies() {
        let mut diagnostics = diagnostics();
        diagnostics.podman_available = false;
        diagnostics.caddy_available = false;

        let report = report_from_diagnostics(diagnostics);

        assert!(report.ok);
        assert_eq!(report.warnings.len(), 2);
    }

    #[test]
    fn doctor_report_errors_for_missing_system_identity() {
        let mut diagnostics = diagnostics();
        diagnostics.cadman_user_exists = false;
        diagnostics.cadman_group_exists = false;

        let report = report_from_diagnostics(diagnostics);

        assert!(!report.ok);
        assert_eq!(report.errors.len(), 2);
    }

    #[test]
    fn doctor_collects_diagnostics_without_mutation() {
        let mut runtime = Runtime::new();
        runtime.set_install_scope(InstallScope::User);
        let ctx = Context::new(runtime);

        let report = doctor(&ctx).unwrap();

        assert_eq!(report.diagnostics.install_scope, "user");
        assert!(report.diagnostics.config_path.ends_with("config.toml"));
    }
}
