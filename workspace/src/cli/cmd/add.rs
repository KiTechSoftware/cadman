use clap::Args as ClapArgs;

use crate::{
    cli::CliResult,
    core::{context::Context, usecases::apps},
};

#[derive(Debug, Clone, ClapArgs)]
#[command(about = "Register the current Cadman project")]
pub struct Args {
    /// Override the registered display name
    #[arg(long)]
    pub name: Option<String>,
    /// Override the registered app id
    #[arg(long)]
    pub id: Option<String>,
}

pub async fn run(ctx: &Context, args: Args) -> CliResult<()> {
    apps::add(ctx, args.name, args.id).await
}
