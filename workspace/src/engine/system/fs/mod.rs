use std::fs;
use std::path::Path;

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::engine::{ErrorCode, Result};

pub fn read_text(path: &Path) -> Result<String> {
    fs::read_to_string(path).map_err(|err| {
        ErrorCode::IoFailure
            .error()
            .with_context("operation", "load_file")
            .with_context("path", path.display().to_string())
            .with_context("error", err.to_string())
    })
}

pub fn write_text(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }

    fs::write(path, content).map_err(|err| {
        ErrorCode::IoFailure
            .error()
            .with_context("operation", "save_file")
            .with_context("path", path.display().to_string())
            .with_context("error", err.to_string())
    })
}

pub fn create_dir_all(path: &Path) -> Result<()> {
    fs::create_dir_all(path).map_err(|err| {
        ErrorCode::IoFailure
            .error()
            .with_context("operation", "create_dir_all")
            .with_context("path", path.display().to_string())
            .with_context("error", err.to_string())
    })
}

pub fn current_dir() -> Result<std::path::PathBuf> {
    std::env::current_dir().map_err(|err| {
        ErrorCode::IoFailure
            .error()
            .with_context("operation", "current_dir")
            .with_context("error", err.to_string())
    })
}
pub fn load_json<T>(path: &Path) -> Result<T>
where
    T: DeserializeOwned,
{
    let raw = read_text(path)?;

    serde_json::from_str(&raw).map_err(|err| {
        ErrorCode::SerializationFailure
            .error()
            .with_context("operation", "load_json")
            .with_context("path", path.display().to_string())
            .with_context("error", err.to_string())
    })
}

pub fn save_json<T>(path: &Path, value: &T) -> Result<()>
where
    T: Serialize,
{
    let raw = serde_json::to_string_pretty(value).map_err(|err| {
        ErrorCode::SerializationFailure
            .error()
            .with_context("operation", "save_json")
            .with_context("path", path.display().to_string())
            .with_context("error", err.to_string())
    })?;

    write_text(path, &raw)
}

pub fn load_toml<T>(path: &Path) -> Result<T>
where
    T: DeserializeOwned,
{
    let raw = read_text(path)?;

    toml::from_str(&raw).map_err(|err| {
        ErrorCode::SerializationFailure
            .error()
            .with_context("operation", "load_toml")
            .with_context("path", path.display().to_string())
            .with_context("error", err.to_string())
    })
}

pub fn save_toml<T>(path: &Path, value: &T) -> Result<()>
where
    T: Serialize,
{
    let raw = toml::to_string_pretty(value).map_err(|err| {
        ErrorCode::SerializationFailure
            .error()
            .with_context("operation", "save_toml")
            .with_context("path", path.display().to_string())
            .with_context("error", err.to_string())
    })?;

    write_text(path, &raw)
}
