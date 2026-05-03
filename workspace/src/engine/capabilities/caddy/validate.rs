use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::engine::{
    ErrorCode, Result,
    constants::CADDY_SERVICE_NAME,
    models::runtime::RunMode,
    system::process::{Process, ProcessOutput},
};

#[derive(Debug, Clone, Serialize)]
pub struct CaddyCommandStatus {
    pub command: String,
    pub skipped: bool,
    pub success: bool,
    pub status: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

impl CaddyCommandStatus {
    pub fn skipped(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            skipped: true,
            success: false,
            status: None,
            stdout: String::new(),
            stderr: String::new(),
        }
    }

    pub fn from_output(command: impl Into<String>, output: ProcessOutput) -> Self {
        Self {
            command: command.into(),
            skipped: false,
            success: output.success(),
            status: Some(output.status),
            stdout: output.stdout,
            stderr: output.stderr,
        }
    }

    pub fn failure(command: impl Into<String>, error: impl ToString) -> Self {
        Self {
            command: command.into(),
            skipped: false,
            success: false,
            status: None,
            stdout: String::new(),
            stderr: error.to_string(),
        }
    }
}

pub trait CaddyCommandRunner {
    fn validate(&self, config_path: &Path, mode: RunMode) -> CaddyCommandStatus;
    fn reload(&self, config_path: &Path, mode: RunMode) -> CaddyCommandStatus;
}

#[derive(Debug, Clone, Copy)]
pub struct ProcessCaddyCommandRunner;

impl CaddyCommandRunner for ProcessCaddyCommandRunner {
    fn validate(&self, config_path: &Path, mode: RunMode) -> CaddyCommandStatus {
        run_caddy_command("validate", config_path, mode)
    }

    fn reload(&self, config_path: &Path, mode: RunMode) -> CaddyCommandStatus {
        run_caddy_command("reload", config_path, mode)
    }
}

pub fn default_config_path(config_dir: &Path) -> PathBuf {
    config_dir.join("Caddyfile")
}

pub fn validate_config(config_path: &Path, mode: RunMode) -> Result<CaddyCommandStatus> {
    if !Process::new(CADDY_SERVICE_NAME).binary_exists() {
        return Err(ErrorCode::CaddyMissing
            .error()
            .with_context("binary", "caddy not found in PATH"));
    }

    Ok(run_caddy_command("validate", config_path, mode))
}

pub(crate) fn run_caddy_command(
    command: &str,
    config_path: &Path,
    mode: RunMode,
) -> CaddyCommandStatus {
    let args = vec![
        command.to_string(),
        "--config".to_string(),
        config_path.display().to_string(),
    ];
    let command_text = format!("caddy {}", args.join(" "));
    let output = Process::new(CADDY_SERVICE_NAME)
        .set_args(args)
        .set_mode(mode)
        .run();

    match output {
        Ok(output) => CaddyCommandStatus::from_output(command_text, output),
        Err(err) => CaddyCommandStatus::failure(command_text, err),
    }
}
