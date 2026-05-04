use crate::engine::{
    constants::{
        APP_NAME, BIN_NAME, CACHE_DIR_NAME, CONFIG_DIR_NAME, CONFIG_FILE_NAME, REGISTRY_FILE_NAME,
        STATE_DIR_NAME, STATE_FILE_NAME,
    },
    errors::{ErrorCode, Result},
};
use std::path::{Path, PathBuf};
use veltrix::os::paths;

//
pub const SUDOERS_DIR: &str = "/etc/sudoers.d";

pub fn sudoers_file_path() -> PathBuf {
    PathBuf::from(SUDOERS_DIR).join(APP_NAME)
}

// System-wide paths

pub fn system_bin_path() -> PathBuf {
    paths::system_bin_path(BIN_NAME)
}

pub fn system_local_bin_path() -> PathBuf {
    paths::system_local_bin_path(BIN_NAME)
}

pub fn system_config_dir() -> PathBuf {
    paths::system_config_dir(CONFIG_DIR_NAME)
}

pub fn system_config_path() -> PathBuf {
    system_config_dir().join(CONFIG_FILE_NAME)
}

pub fn system_state_dir() -> PathBuf {
    paths::system_state_dir(STATE_DIR_NAME)
}

pub fn system_registry_path() -> PathBuf {
    system_state_dir().join(REGISTRY_FILE_NAME)
}

pub fn system_state_path() -> PathBuf {
    system_state_dir().join(STATE_FILE_NAME)
}

pub fn system_cache_dir() -> PathBuf {
    paths::system_cache_dir(CACHE_DIR_NAME)
}

pub fn system_log_dir() -> PathBuf {
    paths::system_log_dir(APP_NAME)
}

pub fn systemd_unit_path() -> PathBuf {
    paths::systemd_unit_path(APP_NAME)
}

// User-level resolved paths

pub fn user_bin_path() -> Option<PathBuf> {
    Some(user_bin_dir().ok()?.join(APP_NAME))
}

pub fn user_config_dir() -> Result<PathBuf> {
    paths::user_config_dir(CONFIG_DIR_NAME).map_err(|_| {
        ErrorCode::ConfigInvalid
            .error()
            .with_context("reason", "Unable to determine user config directory")
    })
}

pub fn user_config_path() -> Result<PathBuf> {
    Ok(user_config_dir()?.join(CONFIG_FILE_NAME))
}

pub fn user_systemd_unit_path() -> Result<PathBuf> {
    paths::user_systemd_unit_path(APP_NAME).map_err(|_| {
        ErrorCode::ConfigInvalid
            .error()
            .with_context("reason", "Unable to determine user systemd unit path")
    })
}

pub fn user_state_dir() -> Result<PathBuf> {
    paths::user_state_dir(STATE_DIR_NAME).map_err(|_| {
        ErrorCode::ConfigInvalid
            .error()
            .with_context("reason", "Unable to determine user state directory")
    })
}

pub fn user_registry_path() -> Result<PathBuf> {
    Ok(user_state_dir()?.join(REGISTRY_FILE_NAME))
}

pub fn user_state_path() -> Result<PathBuf> {
    Ok(user_state_dir()?.join(STATE_FILE_NAME))
}

pub fn user_cache_dir() -> Result<PathBuf> {
    paths::user_cache_dir(CACHE_DIR_NAME).map_err(|_| {
        ErrorCode::ConfigInvalid
            .error()
            .with_context("reason", "Unable to determine user cache directory")
    })
}

pub fn user_log_dir() -> Result<PathBuf> {
    paths::user_log_dir(APP_NAME).or_else(|_| Ok(user_state_dir()?.join("logs")))
}

pub fn user_bin_dir() -> Result<PathBuf> {
    paths::user_bin_dir().map_err(|_| {
        ErrorCode::ConfigInvalid
            .error()
            .with_context("reason", "Unable to determine user bin directory")
    })
}

// Config resolution

pub fn resolve_config_path(explicit_config_path: Option<&Path>) -> Result<PathBuf> {
    if let Some(path) = explicit_config_path {
        return Ok(path.to_path_buf());
    }

    let user_path = user_config_path()?;
    if user_path.exists() {
        return Ok(user_path);
    }

    let system_path = system_config_path();
    if system_path.exists() {
        return Ok(system_path);
    }

    Ok(user_path)
}

pub fn resolve_new_config_path(system: bool) -> Result<PathBuf> {
    if system {
        Ok(system_config_path())
    } else {
        user_config_path()
    }
}
