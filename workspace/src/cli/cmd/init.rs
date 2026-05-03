use clap::Args as ClapArgs;

use crate::{
    cli::CliResult,
    core::{context::Context, usecases::init},
};

#[derive(Debug, Clone, ClapArgs)]
#[command(about = "Create a Cadman project configuration")]
pub struct Args {
    /// Create cadman.yaml instead of cadman.toml
    #[arg(long)]
    pub yaml: bool,
}

pub async fn run(ctx: &Context, args: Args) -> CliResult<()> {
    init::run(ctx, init::InitRequest { yaml: args.yaml }).await
}
