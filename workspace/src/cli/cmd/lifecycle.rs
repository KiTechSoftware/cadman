use clap::{Args as ClapArgs, Subcommand};

use crate::{
    cli::CliResult,
    core::{context::Context, usecases::self_cmd},
};

#[derive(Debug, Clone, ClapArgs)]
#[command(about = "Manage the Cadman installation")]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    /// Install Cadman
    Install,
    /// Update the installed Cadman binary
    Update,
    /// Uninstall Cadman
    Uninstall,
    /// Check Cadman installation health
    Healthcheck,
}

pub async fn run(ctx: &Context, args: Args) -> CliResult<()> {
    match args.command {
        Command::Install => self_cmd::render_install(ctx).await,
        Command::Update => self_cmd::render_update(ctx).await,
        Command::Uninstall => self_cmd::render_uninstall(ctx).await,
        Command::Healthcheck => self_cmd::render_healthcheck(ctx).await,
    }
}
