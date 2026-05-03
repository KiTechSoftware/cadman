use clap::Args as ClapArgs;

use crate::{
    cli::CliResult,
    core::{context::Context, usecases::status},
};

#[derive(Debug, Clone, ClapArgs)]
#[command(about = "Show Cadman runtime, registry, state, and app status")]
pub struct Args {
    /// Optional app id or name to show
    pub app: Option<String>,
}

pub async fn run(ctx: &Context, args: Args) -> CliResult<()> {
    status::render(ctx, args.app.as_deref()).await
}
