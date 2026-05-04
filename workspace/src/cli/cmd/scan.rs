use std::path::PathBuf;

use clap::Args as ClapArgs;

use crate::{
    cli::CliResult,
    core::{context::Context, usecases::scan},
};

#[derive(Debug, Clone, ClapArgs)]
#[command(about = "Scan for Cadman project configs")]
pub struct Args {
    /// Add discovered projects to registry.toml
    #[arg(long)]
    pub add: bool,
    /// Override discovery max depth
    #[arg(long)]
    pub max_depth: Option<usize>,
    /// Optional scan roots
    pub paths: Vec<PathBuf>,
}

pub async fn run(ctx: &Context, args: Args) -> CliResult<()> {
    scan::render(
        ctx,
        scan::ScanRequest {
            paths: args.paths,
            add: args.add,
            max_depth: args.max_depth,
        },
    )
    .await
}
