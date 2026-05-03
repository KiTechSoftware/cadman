use std::path::PathBuf;

use crate::engine::constants::{
    CACHE_DIR_NAME, CADMAN_USER_NAME, CONFIG_DIR_NAME, CONFIG_FILE_NAME,
    DEFAULT_POLL_INTERVAL_SECS, STATE_DIR_NAME,
};
use veltrix::os::unistd::{self, Gid, Uid};

pub mod mode;
pub mod options;

pub use mode::*;
pub use options::*;

#[derive(Debug, Clone)]
pub struct Runtime {
    run_mode: RunMode,
    interactive_mode: InteractiveMode,
    options: RuntimeOptions,
    paths: RuntimePaths,
    poll_interval_secs: u64,
    user_info: RuntimeUserInfo,
}

#[derive(Debug, Clone)]
pub struct RuntimePaths {
    cwd: PathBuf,
    config_path: Option<PathBuf>,
    config_dir: PathBuf,
    cache_dir: PathBuf,
    state_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct RuntimeUserInfo {
    pub cadman_uid: Option<Uid>,
    pub cadman_gid: Option<Gid>,
    pub current_uid: Uid,
    pub effective_uid: Uid,
    pub effective_user: String,
    pub original_user: Option<String>,
    pub ran_with_sudo: bool,
}

impl Default for RuntimeUserInfo {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeUserInfo {
    pub fn new() -> Self {
        let effective_uid = unistd::geteuid();
        let effective_user =
            unistd::username_by_uid(effective_uid).unwrap_or_else(|| effective_uid.to_string());

        let original_user = std::env::var("SUDO_USER").ok();
        let ran_with_sudo = original_user.is_some();

        let cadman_uid = unistd::uid_by_username(CADMAN_USER_NAME);
        let cadman_gid = unistd::gid_by_groupname(CADMAN_USER_NAME);
        let current_uid = unistd::getuid();

        Self {
            effective_uid,
            effective_user,
            original_user,
            ran_with_sudo,
            cadman_uid,
            cadman_gid,
            current_uid,
        }
    }

    pub fn is_cadman(&self) -> bool {
        self.cadman_uid
            .is_some_and(|cadman_uid| self.effective_uid == cadman_uid)
    }

    pub fn is_root(&self) -> bool {
        self.effective_uid.is_root()
    }

    pub fn in_cadman_group(&self) -> bool {
        self.cadman_gid.is_some_and(|cadman_gid| {
            unistd::primary_gid_by_uid(self.current_uid) == Some(cadman_gid)
        })
    }

    pub fn in_admin_group(&self) -> bool {
        unistd::user_in_admin_group(self.current_uid)
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            run_mode: RunMode::User,
            interactive_mode: InteractiveMode::Interactive,
            options: RuntimeOptions::new(),
            paths: RuntimePaths::new(),
            poll_interval_secs: DEFAULT_POLL_INTERVAL_SECS,
            user_info: RuntimeUserInfo::new(),
        }
    }
    /// Returns the effective user name (the user running the process)
    pub fn effective_user(&self) -> &str {
        &self.user_info.effective_user
    }

    /// Returns the original user if run with sudo, otherwise None
    pub fn original_user(&self) -> Option<&str> {
        self.user_info.original_user.as_deref()
    }

    /// Returns true if Cadman was run with sudo
    pub fn ran_with_sudo(&self) -> bool {
        self.user_info.ran_with_sudo
    }

    pub fn output_config(&self) -> scriba::Config {
        scriba::Config {
            interactive: self.is_interactive(),
            format: self.options.output_format(),
            color: self.options.output_color(),
            level: self.options.log_level(),
            auto_yes: self.options.auto_yes(),
        }
    }

    pub fn run_mode(&self) -> RunMode {
        self.run_mode
    }

    pub fn effective_run_mode(&self) -> RunMode {
        // check self run mod with users actual permissions to determine effective run mode
        match self.run_mode {
            RunMode::User => select_user_run_mode(&self.user_info),
            RunMode::Cadman => select_cadman_run_mode(&self.user_info),
            RunMode::Root => select_root_run_mode(&self.user_info),
        }
    }

    pub fn interactive_mode(&self) -> InteractiveMode {
        self.interactive_mode
    }

    pub fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    pub fn options(&self) -> &RuntimeOptions {
        &self.options
    }

    pub fn options_mut(&mut self) -> &mut RuntimeOptions {
        &mut self.options
    }

    pub fn cwd(&self) -> &PathBuf {
        &self.paths.cwd
    }

    pub fn config_path(&self) -> Option<&PathBuf> {
        self.paths.config_path.as_ref()
    }

    pub fn config_dir(&self) -> &PathBuf {
        &self.paths.config_dir
    }

    pub fn cache_dir(&self) -> &PathBuf {
        &self.paths.cache_dir
    }

    pub fn state_dir(&self) -> &PathBuf {
        &self.paths.state_dir
    }

    pub fn default_config_path(&self) -> PathBuf {
        self.paths.config_dir.join(CONFIG_FILE_NAME)
    }

    pub fn effective_config_path(&self) -> PathBuf {
        self.paths
            .config_path
            .clone()
            .unwrap_or_else(|| self.default_config_path())
    }

    pub fn poll_interval_secs(&self) -> u64 {
        self.poll_interval_secs
    }

    pub fn is_ci(&self) -> bool {
        matches!(self.interactive_mode, InteractiveMode::Ci)
    }

    pub fn is_non_interactive(&self) -> bool {
        matches!(self.interactive_mode, InteractiveMode::NonInteractive)
    }

    pub fn is_interactive(&self) -> bool {
        matches!(self.interactive_mode, InteractiveMode::Interactive)
    }

    pub fn set_run_mode(&mut self, mode: RunMode) -> &mut Self {
        self.run_mode = mode;
        self
    }

    pub fn set_interactive_mode(&mut self, mode: InteractiveMode) -> &mut Self {
        self.interactive_mode = mode;
        self
    }

    pub fn set_dry_run(&mut self, dry_run: bool) -> &mut Self {
        self.options.set_dry_run(dry_run);
        self
    }

    pub fn set_auto_yes(&mut self, auto_yes: bool) -> &mut Self {
        self.options.set_auto_yes(auto_yes);
        self
    }

    pub fn set_force(&mut self, force: bool) -> &mut Self {
        self.options.set_force(force);
        self
    }

    pub fn set_output_envelope(&mut self, envelope: scriba::EnvelopeMode) -> &mut Self {
        self.options.set_output_envelope(envelope);
        self
    }

    pub fn set_output_format(&mut self, format: scriba::Format) -> &mut Self {
        self.options.set_output_format(format);
        self
    }

    pub fn set_output_color(&mut self, color: scriba::ColorMode) -> &mut Self {
        self.options.set_output_color(color);
        self
    }

    pub fn set_log_level(&mut self, level: scriba::Level) -> &mut Self {
        self.options.set_log_level(level);
        self
    }

    pub fn set_cwd(&mut self, cwd: PathBuf) -> &mut Self {
        self.paths.cwd = std::fs::canonicalize(&cwd).unwrap_or(cwd);
        self
    }

    pub fn set_config_path(&mut self, path: Option<PathBuf>) -> &mut Self {
        self.paths.config_path = path;
        self
    }

    pub fn set_poll_interval_secs(&mut self, value: u64) -> &mut Self {
        self.poll_interval_secs = value;
        self
    }
}

impl Default for RuntimePaths {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimePaths {
    pub fn new() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        let config_dir = veltrix::os::paths::system_config_dir(CONFIG_DIR_NAME);

        let cache_dir = veltrix::os::paths::system_cache_dir(CACHE_DIR_NAME);

        let state_dir = veltrix::os::paths::system_state_dir(STATE_DIR_NAME);

        Self {
            cwd,
            config_path: None,
            config_dir,
            cache_dir,
            state_dir,
        }
    }
}

fn select_user_run_mode(user_info: &RuntimeUserInfo) -> RunMode {
    if user_info.is_cadman() {
        RunMode::Cadman
    } else {
        RunMode::User
    }
}

fn select_cadman_run_mode(user_info: &RuntimeUserInfo) -> RunMode {
    if user_info.is_cadman() || user_info.in_cadman_group() || user_info.in_admin_group() {
        RunMode::Cadman
    } else {
        RunMode::User
    }
}

fn select_root_run_mode(user_info: &RuntimeUserInfo) -> RunMode {
    if user_info.is_cadman() || user_info.is_root() || user_info.in_admin_group() {
        RunMode::Root
    } else {
        select_cadman_run_mode(user_info)
    }
}
