use std::path::Path;

use crate::engine::{
    ErrorCode, Result, constants::CADDY_SERVICE_NAME, models::runtime::RunMode,
    system::process::Process,
};

use super::validate::{CaddyCommandStatus, run_caddy_command};

pub fn reload_config(config_path: &Path, mode: RunMode) -> Result<CaddyCommandStatus> {
    if !Process::new(CADDY_SERVICE_NAME).binary_exists() {
        return Err(ErrorCode::CaddyMissing
            .error()
            .with_context("binary", "caddy not found in PATH"));
    }

    Ok(run_caddy_command("reload", config_path, mode))
}
