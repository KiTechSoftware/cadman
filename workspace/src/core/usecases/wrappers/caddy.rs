use crate::core::{Context, CoreResult};
use crate::engine::system::infra::caddy;

pub async fn run(ctx: &Context, args: Vec<String>) -> CoreResult<()> {
    ctx.ui()
        .logger()
        .info(&format!("Running Caddy with args: {:?}", args));
    ctx.ui().logger().info(&format!(
        "Effective run mode: {:?}",
        ctx.runtime().effective_run_mode()
    ));
    caddy::run(ctx.runtime().effective_run_mode(), args).await
}
