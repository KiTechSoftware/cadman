use crate::core::{Context, CoreResult};
use crate::engine::system::infra::podman;

pub async fn run(ctx: &Context, args: Vec<String>) -> CoreResult<()> {
    ctx.ui()
        .logger()
        .info(&format!("Running Podman with args: {:?}", args));
    ctx.ui().logger().info(&format!(
        "Effective run mode: {:?}",
        ctx.runtime().effective_run_mode()
    ));
    podman::run(ctx.runtime().effective_run_mode(), args).await
}

pub async fn compose(ctx: &Context, args: Vec<String>) -> CoreResult<()> {
    ctx.ui()
        .logger()
        .info(&format!("Running Podman Compose with args: {:?}", args));
    ctx.ui().logger().info(&format!(
        "Effective run mode: {:?}",
        ctx.runtime().effective_run_mode()
    ));
    podman::compose(ctx.runtime().effective_run_mode(), args).await
}
