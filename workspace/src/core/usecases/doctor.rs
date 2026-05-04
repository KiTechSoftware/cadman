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
    let mut output = ctx
        .ui()
        .new_output_content()
        .json(&report)
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
        .key_value(
            "Container Scopes",
            report.diagnostics.authorized_container_scopes.join(","),
        )
        .key_value("Config", report.diagnostics.config_path.display())
        .key_value("Registry", report.diagnostics.registry_path.display())
        .key_value("State", report.diagnostics.state_path.display())
        .key_value("Caddy Sites", report.diagnostics.caddy_sites_dir.display())
        .key_value("Config Readable", report.diagnostics.config_readable)
        .key_value("Registry Readable", report.diagnostics.registry_readable)
        .key_value("Registry Writable", report.diagnostics.registry_writable)
        .key_value("State Readable", report.diagnostics.state_readable)
        .key_value("State Writable", report.diagnostics.state_writable)
        .key_value(
            "Stale Sites",
            report.diagnostics.stale_generated_site_files.len(),
        );

    if !report.warnings.is_empty() {
        output = output.key_value("Warnings", format_messages(&report.warnings))
    }
    if !report.errors.is_empty() {
        output = output.key_value("Errors", format_messages(&report.errors))
    }
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
    if !diagnostics.caddy_sites_dir_exists {
        warnings.push("Caddy sites directory does not exist".to_string());
    } else if !diagnostics.caddy_sites_dir_writable {
        warnings.push("Caddy sites directory is not writable".to_string());
    }
    if !diagnostics.config_readable {
        errors.push("config file is not readable".to_string());
    }
    if !diagnostics.registry_readable {
        errors.push("registry file is not readable".to_string());
    }
    if !diagnostics.registry_writable {
        warnings.push("registry path is not writable".to_string());
    }
    if !diagnostics.state_readable {
        errors.push("state file is not readable".to_string());
    }
    if !diagnostics.state_writable {
        warnings.push("state path is not writable".to_string());
    }
    if !diagnostics.stale_generated_site_files.is_empty() {
        warnings.push(format!(
            "{} stale generated Caddy site file(s) found",
            diagnostics.stale_generated_site_files.len()
        ));
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
            authorized_container_scopes: vec!["current".to_string()],
            config_path: PathBuf::from("/etc/cadman/config.toml"),
            registry_path: PathBuf::from("/var/lib/cadman/registry.toml"),
            state_path: PathBuf::from("/var/lib/cadman/state.json"),
            caddy_sites_dir: PathBuf::from("/var/lib/cadman/caddy/sites"),
            caddy_sites_dir_exists: true,
            caddy_sites_dir_writable: true,
            config_readable: true,
            registry_readable: true,
            registry_writable: true,
            state_readable: true,
            state_writable: true,
            stale_generated_site_files: Vec::new(),
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

    #[test]
    fn doctor_report_serializes() {
        let report = report_from_diagnostics(diagnostics());

        assert!(serde_json::to_string(&report).is_ok());
    }
}
