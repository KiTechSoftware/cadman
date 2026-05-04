use clap::Args as ClapArgs;

use crate::{
    cli::CliResult,
    core::{context::Context, usecases::apps},
};

#[derive(Debug, Clone, ClapArgs)]
#[command(about = "Remove an app from the Cadman registry")]
pub struct Args {
    /// App id or name to remove
    pub app: String,
}

pub async fn run(ctx: &Context, args: Args) -> CliResult<()> {
    apps::remove(ctx, &args.app).await
}
