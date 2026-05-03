use std::process::ExitCode;

use crate::engine::Error;

pub type CoreResult<T> = Result<T, Error>;

pub fn report_error(err: &Error) {
    eprintln!("{}: {}", err.code.id(), err.message);
    for (key, value) in &err.context {
        eprintln!("  {key}: {value}");
    }
}

pub fn exit_code(err: &Error) -> ExitCode {
    ExitCode::from(err.exit_code() as u8)
}
