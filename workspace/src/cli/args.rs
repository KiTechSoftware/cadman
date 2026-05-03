use std::path::PathBuf;

use crate::{
    cli::cmd,
    core::{APP_ABOUT, APP_NAME, bootstrap::AppContextArgs},
};
use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum FormatArg {
    Auto,
    Text,
    Markdown,
    Json,
    Jsonl,
    Plain,
}

impl FormatArg {
    pub fn to_output_format_string(&self, use_json: bool) -> String {
        match (use_json, self) {
            (_, FormatArg::Text) => "text".to_string(),
            (_, FormatArg::Markdown) => "markdown".to_string(),
            (_, FormatArg::Json) => "json".to_string(),
            (_, FormatArg::Jsonl) => "jsonl".to_string(),
            (_, FormatArg::Plain) => "plain".to_string(),
            (true, FormatArg::Auto) => "json".to_string(),
            (false, FormatArg::Auto) => "text".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

impl std::fmt::Display for ColorMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorMode::Auto => write!(f, "auto"),
            ColorMode::Always => write!(f, "always"),
            ColorMode::Never => write!(f, "never"),
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum UserMode {
    Current,
    Cadman,
    Root,
}

impl std::fmt::Display for UserMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserMode::Current => write!(f, "current"),
            UserMode::Cadman => write!(f, "cadman"),
            UserMode::Root => write!(f, "root"),
        }
    }
}

/// Cadman
#[derive(Parser, Debug)]
#[command(
    name = APP_NAME,
    version,
    author,
    about = APP_ABOUT,
    propagate_version = true,
    arg_required_else_help = true
)]
pub struct Cli {
    /// Global flags (apply to all subcommands)
    #[command(flatten)]
    pub global: GlobalArgs,
    /// Subcommands
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    /// List Podman containers visible to Cadman
    Containers(cmd::containers::Args),
    /// Create a Cadman project configuration
    Init(cmd::init::Args),
    /// Run Podman Compose through Cadman
    Compose(cmd::wrappers::compose::Args),
    /// Run Podman through Cadman
    Podman(cmd::wrappers::podman::Args),
    /// Run Caddy through Cadman
    Caddy(cmd::wrappers::caddy::Args),
}

#[derive(Debug, Clone, Args)]
pub struct GlobalArgs {
    /// Increase verbosity (-v, -vv, -vvv); combine with -q to reduce
    #[arg(short = 'v', long = "verbose", action = ArgAction::Count, global = true)]
    pub verbose: u8,
    /// Decrease verbosity (-q, -qq)
    #[arg(short = 'q', long = "quiet", action = ArgAction::Count, global = true)]
    pub quiet: u8,
    /// Output Envelop as JSON instead of human-readable text
    #[arg(long, global = true)]
    pub json: bool,
    /// Output Payload format (json, jsonl, markdown, text)
    #[arg(long, global = true, default_value_t = FormatArg::Auto, value_enum, conflicts_with = "plain")]
    pub format: FormatArg,
    /// Output plain text (equivalent to --format plain)
    #[arg(long, short = 'p', global = true, conflicts_with = "format")]
    pub plain: bool,
    /// Simulate actions without changes
    #[arg(long, global = true)]
    pub dry_run: bool,
    /// Color policy for output
    #[arg(long, value_enum, default_value_t = ColorMode::Auto, global = true)]
    pub color: ColorMode,
    /// Run against another directory (default: current working directory)
    #[arg(short = 'C', long = "cwd", global = true, default_value = ".")]
    pub cwd: PathBuf,
    /// Strict, non-interactive mode (assume yes, no prompts, CI-friendly)
    #[arg(long, global = true, conflicts_with = "non_interactive")]
    pub ci: bool,
    /// Strict, non-interactive mode (assume no, no prompts, CI-friendly)
    #[arg(long, global = true, conflicts_with = "ci")]
    pub non_interactive: bool,
    /// Accept defaults automatically (assume yes, no prompts)
    #[arg(long, short = 'y', global = true)]
    pub yes: bool,
    /// Force actions that would normally be prevented (e.g. pushing to protected branches)
    #[arg(long, global = true)]
    pub force: bool,
    /// Use a specific Cadman config not in the default location
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,
    /// Run mode (current, cadman, root)
    #[arg(long, value_enum, global = true)]
    pub mode: Option<UserMode>,
}

impl From<GlobalArgs> for AppContextArgs {
    fn from(args: GlobalArgs) -> Self {
        Self {
            verbose: args.verbose,
            quiet: args.quiet,
            json: args.json,
            format: if args.plain {
                "plain".to_string()
            } else {
                args.format.to_output_format_string(args.json)
            },
            dry_run: args.dry_run,
            output_color: args.color.to_string(),
            cwd: args.cwd,
            ci: args.ci,
            non_interactive: args.non_interactive,
            auto_yes: args.yes,
            force: args.force,
            config_path: args.config,
            run_mode: args.mode.map(|mode| mode.to_string()),
        }
    }
}
