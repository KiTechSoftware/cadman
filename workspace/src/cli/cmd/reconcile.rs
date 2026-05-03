use clap::Args as ClapArgs;

use crate::{
    cli::CliResult,
    core::{app::reconcile, context::Context},
};

#[derive(Debug, Clone, ClapArgs)]
#[command(about = "Discover Podman services and reconcile Caddy routes")]
pub struct Args {}

pub async fn run(ctx: &Context, _args: Args) -> CliResult<()> {
    reconcile::run_and_print(ctx).await
}
