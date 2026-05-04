use clap::Args as ClapArgs;

use crate::{
    cli::CliResult,
    core::{Context, containers},
};

#[derive(Debug, Clone, ClapArgs)]
#[command(about = "List Podman containers visible to Cadman")]
pub struct Args {
    /// List running and stopped containers
    #[arg(long)]
    pub all: bool,
    /// List only running containers
    #[arg(long)]
    pub running: bool,
    /// List only stopped or exited containers
    #[arg(long)]
    pub stopped: bool,
    /// Include labels in text output
    #[arg(long)]
    pub labels: bool,
}

pub async fn run(ctx: &Context, args: Args) -> CliResult<()> {
    containers::list(ctx, args.all, args.running, args.stopped, args.labels).await
}
