// ── Podman passthrough wrappers ─────────────────────────────────────────────

use crate::engine::{ErrorCode, Result, models::runtime::RunMode, system::infra::process::Process};

pub async fn run(mode: RunMode, args: Vec<String>) -> Result<()> {
    let proc = Process::new("podman").set_args(args).set_mode(mode);

    if !proc.binary_exists() {
        return Err(ErrorCode::PodmanMissing
            .error()
            .with_context("binary", "podman not found in PATH"));
    }
    proc.run_async().await?.emit()
}

pub async fn compose(mode: RunMode, args: Vec<String>) -> Result<()> {
    let proc = Process::new("podman-compose").set_args(args).set_mode(mode);
    if !proc.binary_exists() {
        return Err(ErrorCode::PodmanComposeMissing
            .error()
            .with_context("binary", "podman-compose not found in PATH"));
    }
    proc.run_async().await?.emit()
}
