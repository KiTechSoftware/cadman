use crate::engine::{
    ErrorCode, Result, constants::CADDY_SERVICE_NAME, models::runtime::RunMode,
    system::process::Process,
};

pub mod apply;
pub mod reload;
pub mod sites;
pub mod validate;

// ── Caddy passthrough wrappers ─────────────────────────────────────────────
pub async fn run(mode: RunMode, args: Vec<String>) -> Result<()> {
    // need to use veltrix here
    let proc = Process::new(CADDY_SERVICE_NAME)
        .set_args(args)
        .set_mode(mode);

    if !proc.binary_exists() {
        return Err(ErrorCode::CaddyMissing
            .error()
            .with_context("binary", "caddy not found in PATH"));
    }
    proc.run_async().await?.emit()
}
