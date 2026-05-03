use std::{
    env,
    path::{Path, PathBuf},
    process::Output,
};
use veltrix::os::{
    process::cmd::{async_cmd, spec::CmdSpec, std_cmd},
    unistd::{self, Uid},
};

use crate::engine::{
    Error, ErrorCode, Result, constants::CADMAN_USER_NAME, models::runtime::RunMode,
};
pub struct Process {
    mode: RunMode,
    cwd: Option<PathBuf>,
    spec: CmdSpec,
}

impl Default for Process {
    fn default() -> Self {
        Self::new("")
    }
}
impl Process {
    pub fn new(binary: &str) -> Self {
        Self {
            mode: RunMode::User,
            cwd: None,
            spec: CmdSpec::new(binary),
        }
    }

    pub fn set_mode(mut self, mode: RunMode) -> Self {
        self.mode = mode;
        self.determine_user();
        self
    }

    pub fn set_cwd<P: Into<PathBuf>>(mut self, cwd: P) -> Self {
        self.cwd = Some(cwd.into());
        self
    }
    // need a function to determine the user via the run mode to

    pub fn set_program(mut self, binary: &str) -> Self {
        self.spec.program = binary.to_string();
        self
    }

    pub fn set_sudo(mut self, sudo: bool) -> Self {
        self.spec.sudo = sudo;
        self
    }

    pub fn set_uid(mut self, uid: u32) -> Self {
        self.spec.uid = Some(uid);
        self
    }

    pub fn set_gid(mut self, gid: u32) -> Self {
        self.spec.gid = Some(gid);
        self
    }

    pub fn set_user(self, uid: u32, gid: u32) -> Self {
        self.set_uid(uid).set_gid(gid)
    }

    pub fn set_args(mut self, args: Vec<String>) -> Self {
        self.spec.args = args;
        self
    }

    pub fn run(&self) -> Result<ProcessOutput> {
        let output = std_cmd::run(self.spec.clone());
        if output.is_err() {
            return Err(ErrorCode::IoFailure.error().with_context(
                "error",
                format!("failed to execute process: {}", output.err().unwrap()),
            ));
        }
        self.convert(output.unwrap())
    }

    pub async fn run_async(&self) -> Result<ProcessOutput> {
        let output = async_cmd::run(self.spec.clone()).await;
        if output.is_err() {
            return Err(ErrorCode::IoFailure.error().with_context(
                "error",
                format!("failed to execute process: {}", output.err().unwrap()),
            ));
        }
        self.convert(output.unwrap())
    }

    pub fn binary_exists(&self) -> bool {
        self.find_binary(&self.spec.program).is_some()
    }

    pub fn find_binary(&self, name: &str) -> Option<PathBuf> {
        let candidate = Path::new(name);

        if candidate.components().count() > 1 {
            return candidate.is_file().then(|| candidate.to_path_buf());
        }

        let path = env::var_os("PATH")?;

        env::split_paths(&path)
            .map(|dir| dir.join(name))
            .find(|path| path.is_file())
    }

    fn convert(&self, output: Output) -> Result<ProcessOutput> {
        Ok(ProcessOutput {
            status: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }

    fn determine_user(&mut self) {
        let user = check_user(self.mode);
        if user.use_sudo {
            self.spec.sudo = true;
        }
        if let Some(uid) = user.uid {
            self.spec.uid = Some(uid);
        }
        if let Some(gid) = user.gid {
            self.spec.gid = Some(gid);
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProcessOutput {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
}

impl ProcessOutput {
    pub fn success(&self) -> bool {
        self.status == 0
    }

    pub fn into_result(self, bin: &str) -> Result<Self> {
        if self.success() {
            Ok(self)
        } else {
            Err(self.process_failure(bin))
        }
    }

    pub fn process_failure(&self, bin: &str) -> Error {
        let mut err = ErrorCode::ProcessFailure
            .error()
            .with_context("command", bin.to_string())
            .with_context_str("status", self.status);

        let stderr = self.stderr.trim();

        if !stderr.is_empty() {
            err = err.with_context("stderr", stderr.to_string());
        }

        err
    }

    pub fn emit(&self) -> Result<()> {
        print!("{}", self.stdout);
        if !self.stderr.is_empty() {
            eprint!("{}", self.stderr);
        }
        Ok(())
    }
}

fn check_user(mode: RunMode) -> User {
    let mut use_sudo = false;
    let mut uid = None;
    let gid = None;

    let current_uid = unistd::geteuid();
    let cadman_uid = unistd::uid_by_username(CADMAN_USER_NAME);

    match mode {
        RunMode::User => {
            // If running as cadman, treat as Cadman.
            if cadman_uid.is_some_and(|uid| current_uid == uid) {
                uid = cadman_uid.map(|u| u.as_raw());
            }
        }

        RunMode::Cadman => {
            // If already cadman, don't sudo.
            if cadman_uid.is_some_and(|uid| current_uid == uid) {
                uid = cadman_uid.map(|u| u.as_raw());
            } else {
                use_sudo = true;
                uid = cadman_uid.map(|u| u.as_raw());
            }
        }

        RunMode::Root => {
            // If already root, don't sudo.
            if current_uid != Uid::from_raw(0) {
                use_sudo = true;
            }
        }
    }
    User { use_sudo, uid, gid }
}

pub struct User {
    pub use_sudo: bool,
    pub uid: Option<u32>,
    pub gid: Option<u32>,
}
