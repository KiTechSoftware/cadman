use clap::Args as ClapArgs;

use crate::{
    cli::CliResult,
    core::{context::Context, usecases::reconcile},
};

#[derive(Debug, Clone, ClapArgs)]
#[command(about = "Discover Podman services and reconcile Caddy routes")]
pub struct Args {}

pub async fn run(ctx: &Context, _args: Args) -> CliResult<()> {
    reconcile::render(ctx).await
}
