use clap::{Args as ClapArgs, Subcommand};

use crate::{
    cli::CliResult,
    core::{context::Context, usecases::registry},
};

#[derive(Debug, Clone, ClapArgs)]
#[command(about = "Inspect and mutate the Cadman registry")]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    /// Show the resolved registry.toml path
    Path,
    /// List registered apps
    List,
    /// Show a registered app by id or name
    Show { app: String },
    /// Remove a registered app by id or name
    Remove { app: String },
}

pub async fn run(ctx: &Context, args: Args) -> CliResult<()> {
    match args.command {
        Command::Path => registry::render_path(ctx).await,
        Command::List => registry::render_list(ctx).await,
        Command::Show { app } => registry::render_show(ctx, &app).await,
        Command::Remove { app } => registry::render_remove(ctx, &app).await,
    }
}
