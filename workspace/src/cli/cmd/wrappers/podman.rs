use clap::Args as ClapArgs;

use crate::{
    cli::CliResult,
    core::{context::Context, wrappers::podman},
};

#[derive(Debug, Clone, ClapArgs)]
#[command(about = "Run Podman through Cadman")]
pub struct Args {
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<String>,
}

pub async fn run(ctx: &Context, args: Args) -> CliResult<()> {
    podman::run(ctx, args.args).await
}
