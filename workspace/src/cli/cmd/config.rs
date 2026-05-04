use clap::{Args as ClapArgs, Subcommand};

use crate::{
    cli::CliResult,
    core::{context::Context, usecases::config},
};

#[derive(Debug, Clone, ClapArgs)]
#[command(about = "Inspect and initialize Cadman global configuration")]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    /// Show the resolved config.toml path
    Path,
    /// Show the current config or runtime defaults
    Show,
    /// Write the default config.toml
    Init,
}

pub async fn run(ctx: &Context, args: Args) -> CliResult<()> {
    match args.command {
        Command::Path => config::render_path(ctx).await,
        Command::Show => config::render_show(ctx).await,
        Command::Init => config::render_init(ctx).await,
    }
}
