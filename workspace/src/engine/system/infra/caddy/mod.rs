use crate::engine::{ErrorCode, Result, models::runtime::RunMode, system::infra::process::Process};

pub async fn run(mode: RunMode, args: Vec<String>) -> Result<()> {
    // need to use veltrix here
    let proc = Process::new("caddy").set_args(args).set_mode(mode);

    if !proc.binary_exists() {
        return Err(ErrorCode::CaddyMissing
            .error()
            .with_context("binary", "caddy not found in PATH"));
    }
    proc.run_async().await?.emit()
}
