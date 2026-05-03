pub mod args;
pub mod cmd;
mod errors;

use crate::{
    core::{bootstrap::build_app_context, exit_code, report_error},
    engine::ErrorCode,
};
use clap::Parser;
use std::process::ExitCode;

pub use errors::CliResult;

pub fn run() -> ExitCode {
    #[cfg(not(target_os = "linux"))]
    {
        let err = ErrorCode::RuntimeUnsupportedOS.error()
        .with_context_str("notice", "Windows and MacOS is not supported, use linux instead or a containerized version of cadman");

        report_error(&err);
        exit_code(&err)
    }
    #[cfg(target_os = "linux")]
    {
        match tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
        {
            Ok(runtime) => runtime.block_on(async_run()),
            Err(err) => {
                let err = ErrorCode::ProcessFailure
                    .error()
                    .with_context_str("runtime", err);

                report_error(&err);
                exit_code(&err)
            }
        }
    }
}

async fn async_run() -> ExitCode {
    let cli = args::Cli::parse();

    let ctx = match build_app_context(cli.global.into()) {
        Ok(ctx) => ctx,
        Err(err) => {
            let err = ErrorCode::ProcessFailure
                .error()
                .with_context_str("app_context", err);
            report_error(&err);
            return exit_code(&err);
        }
    };

    let result = match cli.command {
        args::Command::Caddy(args) => cmd::wrappers::caddy::run(&ctx, args).await,
        args::Command::Config(args) => cmd::config::run(&ctx, args).await,
        args::Command::Containers(args) => cmd::containers::run(&ctx, args).await,
        args::Command::Compose(args) => cmd::wrappers::compose::run(&ctx, args).await,
        args::Command::Init(args) => cmd::init::run(&ctx, args).await,
        args::Command::Podman(args) => cmd::wrappers::podman::run(&ctx, args).await,
        args::Command::Registry(args) => cmd::registry::run(&ctx, args).await,
    };

    if let Err(err) = result {
        report_error(&err);
        return exit_code(&err);
    }

    ExitCode::SUCCESS
}
