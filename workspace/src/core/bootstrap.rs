use std::path::PathBuf;

use crate::{
    core::context::Context,
    engine::{Result, models::runtime::{InteractiveMode, RunMode, Runtime}},
};

/// All inputs needed to build an application context from CLI arguments.
pub struct AppContextArgs {
    pub verbose: u8,
    pub quiet: u8,
    pub json: bool,
    pub format: String,
    pub dry_run: bool,
    pub output_color: String,
    pub cwd: PathBuf,
    pub ci: bool,
    pub non_interactive: bool,
    pub auto_yes: bool,
    pub force: bool,
    pub config_path: Option<PathBuf>,
    pub run_mode: String,
}

pub fn build_app_context(args: AppContextArgs) -> Result<Context> {
    let imode = InteractiveMode::from_flags(args.ci, args.non_interactive);

    let output_color = if args.output_color == "auto" {
        if imode == InteractiveMode::Interactive {
            scriba::ColorMode::Auto
        } else {
            scriba::ColorMode::Never
        }
    } else {
        scriba::ColorMode::from_str(&args.output_color)
    };

    let output_envelope = if args.json {
        scriba::EnvelopeMode::Json
    } else {
        scriba::EnvelopeMode::None
    };

    let mut runtime = Runtime::new();

    runtime
        .set_run_mode(RunMode::parse(&args.run_mode))
        .set_interactive_mode(imode)
        .set_cwd(args.cwd)
        .set_dry_run(args.dry_run)
        .options_mut()
        .set_auto_yes(args.auto_yes)
        .set_force(args.force)
        .set_output_envelope(output_envelope)
        .set_output_format(scriba::Format::from_str(&args.format))
        .set_output_color(output_color)
        .set_log_level(scriba::Level::from_flags(args.verbose, args.quiet));

    let ctx = Context::new(runtime);

    Ok(ctx)
}
