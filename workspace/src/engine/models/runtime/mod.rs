use std::path::{Path, PathBuf};

use crate::engine::{Result, constants::paths};
use crate::engine::{
    constants::{
        CADMAN_USER_NAME, CONFIG_FILE_NAME, DEFAULT_POLL_INTERVAL_SECS, REGISTRY_FILE_NAME,
        STATE_FILE_NAME,
    },
    models::containers::ContainerScope,
};
use veltrix::os::unistd::{self, Gid, Uid};

pub mod mode;
pub mod options;
pub mod scope;

pub use mode::*;
pub use options::*;
pub use scope::*;

#[derive(Debug, Clone)]
pub struct Runtime {
    run_mode: RunMode,
    interactive_mode: InteractiveMode,
    install_scope: InstallScope,
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
    log_dir: PathBuf,
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
        unistd::user_in_group(self.current_uid, CADMAN_USER_NAME)
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
        let current_exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
        let install_scope = detect_install_scope(&current_exe, None);

        Self {
            run_mode: default_run_mode_for_install_scope(install_scope),
            interactive_mode: InteractiveMode::Interactive,
            install_scope,
            options: RuntimeOptions::new(),
            paths: RuntimePaths::new_for_scope(install_scope)
                .unwrap_or_else(|_| RuntimePaths::user_fallback()),
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
            RunMode::Current => select_user_run_mode(&self.user_info),
            RunMode::Cadman => select_cadman_run_mode(&self.user_info),
            RunMode::Root => select_root_run_mode(&self.user_info),
        }
    }

    pub fn visible_container_scopes(&self) -> Vec<ContainerScope> {
        if self.install_scope == InstallScope::System {
            authorized_container_scopes(&self.user_info)
        } else {
            vec![ContainerScope::Current]
        }
    }

    pub fn can_access_container_scope(&self, scope: ContainerScope) -> bool {
        self.visible_container_scopes().contains(&scope)
    }

    pub fn interactive_mode(&self) -> InteractiveMode {
        self.interactive_mode
    }

    pub fn install_scope(&self) -> InstallScope {
        self.install_scope
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

    pub fn log_dir(&self) -> &PathBuf {
        &self.paths.log_dir
    }

    pub fn default_config_path(&self) -> PathBuf {
        self.paths.config_dir.join(CONFIG_FILE_NAME)
    }

    pub fn registry_path(&self) -> PathBuf {
        self.paths.state_dir.join(REGISTRY_FILE_NAME)
    }

    pub fn state_path(&self) -> PathBuf {
        self.paths.state_dir.join(STATE_FILE_NAME)
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

    pub fn set_install_scope(&mut self, scope: InstallScope) -> &mut Self {
        let config_path = self.paths.config_path.clone();
        self.install_scope = scope;
        self.paths =
            RuntimePaths::new_for_scope(scope).unwrap_or_else(|_| RuntimePaths::user_fallback());
        self.paths.config_path = config_path;
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
        let explicit_path = path.clone();
        let scope = if path
            .as_deref()
            .is_some_and(|path| path.starts_with("/etc/cadman"))
        {
            InstallScope::System
        } else {
            self.install_scope
        };

        if scope != self.install_scope {
            self.set_install_scope(scope);
        }
        self.paths.config_path = explicit_path;
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
        Self::new_for_scope(InstallScope::User).unwrap_or_else(|_| Self::user_fallback())
    }

    pub fn new_for_scope(scope: InstallScope) -> Result<Self> {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        let (config_dir, cache_dir, state_dir, log_dir) = match scope {
            InstallScope::System => (
                paths::system_config_dir(),
                paths::system_cache_dir(),
                paths::system_state_dir(),
                paths::system_log_dir(),
            ),
            InstallScope::User | InstallScope::Container => (
                paths::user_config_dir()?,
                paths::user_cache_dir()?,
                paths::user_state_dir()?,
                paths::user_log_dir()?,
            ),
        };

        Ok(Self {
            cwd,
            config_path: None,
            config_dir,
            cache_dir,
            state_dir,
            log_dir,
        })
    }

    pub fn user_fallback() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        let local_share = home.join(".local").join("share").join("cadman");
        let local_state = home.join(".local").join("state").join("cadman");

        Self {
            cwd,
            config_path: None,
            config_dir: home.join(".config").join("cadman"),
            cache_dir: home.join(".cache").join("cadman"),
            state_dir: local_share,
            log_dir: local_state.join("logs"),
        }
    }

    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    pub fn state_dir(&self) -> &Path {
        &self.state_dir
    }

    pub fn log_dir(&self) -> &Path {
        &self.log_dir
    }
}

fn select_user_run_mode(user_info: &RuntimeUserInfo) -> RunMode {
    if user_info.is_cadman() {
        RunMode::Cadman
    } else {
        RunMode::Current
    }
}

fn select_cadman_run_mode(user_info: &RuntimeUserInfo) -> RunMode {
    if user_info.is_cadman() || user_info.in_cadman_group() || user_info.in_admin_group() {
        RunMode::Cadman
    } else {
        RunMode::Current
    }
}

fn select_root_run_mode(user_info: &RuntimeUserInfo) -> RunMode {
    if user_info.is_cadman() || user_info.is_root() || user_info.in_admin_group() {
        RunMode::Root
    } else {
        select_cadman_run_mode(user_info)
    }
}

fn default_run_mode_for_install_scope(scope: InstallScope) -> RunMode {
    match scope {
        InstallScope::System => RunMode::Cadman,
        InstallScope::User | InstallScope::Container => RunMode::Current,
    }
}

fn authorized_container_scopes(user_info: &RuntimeUserInfo) -> Vec<ContainerScope> {
    authorized_container_scopes_from_membership(
        user_info.is_cadman(),
        user_info.in_cadman_group(),
        user_info.in_admin_group(),
    )
}

#[cfg(test)]
fn visible_container_scopes_from_membership(
    install_scope: InstallScope,
    is_cadman_user: bool,
    in_cadman_group: bool,
    in_admin_group: bool,
) -> Vec<ContainerScope> {
    if install_scope == InstallScope::System {
        authorized_container_scopes_from_membership(is_cadman_user, in_cadman_group, in_admin_group)
    } else {
        vec![ContainerScope::Current]
    }
}

fn authorized_container_scopes_from_membership(
    is_cadman_user: bool,
    in_cadman_group: bool,
    in_admin_group: bool,
) -> Vec<ContainerScope> {
    let mut scopes = vec![ContainerScope::Current];

    if in_cadman_group || is_cadman_user {
        scopes.push(ContainerScope::Cadman);
    }

    if in_cadman_group && in_admin_group {
        scopes.push(ContainerScope::Root);
    }

    scopes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::constants::{CONFIG_FILE_NAME, REGISTRY_FILE_NAME, STATE_FILE_NAME};

    #[test]
    fn user_scope_uses_user_dirs() {
        let paths = RuntimePaths::new_for_scope(InstallScope::User).unwrap();

        assert!(!paths.config_dir().starts_with("/etc/cadman"));
        assert!(!paths.state_dir().starts_with("/var/lib/cadman"));
        assert!(!paths.cache_dir().starts_with("/var/cache/cadman"));
        assert!(!paths.log_dir().starts_with("/var/log/cadman"));
    }

    #[test]
    fn system_scope_uses_system_dirs() {
        let paths = RuntimePaths::new_for_scope(InstallScope::System).unwrap();

        assert_eq!(paths.config_dir(), Path::new("/etc/cadman"));
        assert_eq!(paths.state_dir(), Path::new("/var/lib/cadman"));
        assert_eq!(paths.cache_dir(), Path::new("/var/cache/cadman"));
        assert_eq!(paths.log_dir(), Path::new("/var/log/cadman"));
    }

    #[test]
    fn container_scope_does_not_default_to_system_dirs() {
        let paths = RuntimePaths::new_for_scope(InstallScope::Container).unwrap();

        assert!(!paths.config_dir().starts_with("/etc/cadman"));
        assert!(!paths.state_dir().starts_with("/var/lib/cadman"));
    }

    #[test]
    fn runtime_file_paths_use_v1_names() {
        let mut runtime = Runtime::new();
        runtime.set_install_scope(InstallScope::User);

        assert!(runtime.default_config_path().ends_with(CONFIG_FILE_NAME));
        assert!(runtime.registry_path().ends_with(REGISTRY_FILE_NAME));
        assert!(runtime.state_path().ends_with(STATE_FILE_NAME));
    }

    #[test]
    fn setting_root_run_mode_does_not_change_install_scope() {
        let mut runtime = Runtime::new();
        runtime.set_install_scope(InstallScope::User);
        runtime.set_run_mode(RunMode::Root);

        assert_eq!(runtime.install_scope(), InstallScope::User);
        assert!(!runtime.default_config_path().starts_with("/etc/cadman"));
    }

    #[test]
    fn non_admin_non_cadman_group_sees_current_containers_only() {
        assert_eq!(
            authorized_container_scopes_from_membership(false, false, false),
            vec![ContainerScope::Current]
        );
    }

    #[test]
    fn cadman_group_sees_current_and_cadman_containers() {
        assert_eq!(
            authorized_container_scopes_from_membership(false, true, false),
            vec![ContainerScope::Current, ContainerScope::Cadman]
        );
    }

    #[test]
    fn admin_without_cadman_group_sees_current_containers_only() {
        assert_eq!(
            authorized_container_scopes_from_membership(false, false, true),
            vec![ContainerScope::Current]
        );
    }

    #[test]
    fn admin_and_cadman_group_sees_current_cadman_and_root_containers() {
        assert_eq!(
            authorized_container_scopes_from_membership(false, true, true),
            vec![
                ContainerScope::Current,
                ContainerScope::Cadman,
                ContainerScope::Root
            ]
        );
    }

    #[test]
    fn user_install_limits_visibility_to_current_even_with_groups() {
        assert_eq!(
            visible_container_scopes_from_membership(InstallScope::User, false, true, true),
            vec![ContainerScope::Current]
        );
    }

    #[test]
    fn system_install_uses_authorized_visibility_scopes() {
        assert_eq!(
            visible_container_scopes_from_membership(InstallScope::System, false, true, true),
            vec![
                ContainerScope::Current,
                ContainerScope::Cadman,
                ContainerScope::Root
            ]
        );
    }

    #[test]
    fn system_install_defaults_to_cadman_run_mode() {
        assert_eq!(
            default_run_mode_for_install_scope(InstallScope::System),
            RunMode::Cadman
        );
    }

    #[test]
    fn user_install_defaults_to_current_run_mode() {
        assert_eq!(
            default_run_mode_for_install_scope(InstallScope::User),
            RunMode::Current
        );
    }

    #[test]
    fn explicit_non_system_config_path_changes_config_only() {
        let mut runtime = Runtime::new();
        runtime.set_install_scope(InstallScope::User);
        runtime.set_config_path(Some(PathBuf::from("/tmp/cadman/config.toml")));

        assert_eq!(
            runtime.effective_config_path(),
            PathBuf::from("/tmp/cadman/config.toml")
        );
        assert_eq!(runtime.install_scope(), InstallScope::User);
        assert!(runtime.registry_path().ends_with(REGISTRY_FILE_NAME));
    }

    #[test]
    fn explicit_etc_config_path_selects_system_scope() {
        let mut runtime = Runtime::new();
        runtime.set_install_scope(InstallScope::User);
        runtime.set_config_path(Some(PathBuf::from("/etc/cadman/config.toml")));

        assert_eq!(runtime.install_scope(), InstallScope::System);
        assert_eq!(
            runtime.effective_config_path(),
            PathBuf::from("/etc/cadman/config.toml")
        );
        assert!(runtime.registry_path().starts_with("/var/lib/cadman"));
    }
}
