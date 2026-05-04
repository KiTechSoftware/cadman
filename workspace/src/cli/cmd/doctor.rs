use clap::Args as ClapArgs;

use crate::{
    cli::CliResult,
    core::{context::Context, usecases::doctor},
};

#[derive(Debug, Clone, ClapArgs)]
#[command(about = "Run read-only Cadman diagnostics")]
pub struct Args {}

pub async fn run(ctx: &Context, _args: Args) -> CliResult<()> {
    doctor::render(ctx).await
}
