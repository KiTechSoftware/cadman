pub mod code;
pub mod kind;

pub use code::ErrorCode;
pub use kind::ErrorKind;

use serde::Serialize;
use std::fmt;
use thiserror::Error;

#[derive(Debug, Clone, Serialize)]
pub struct ErrorContextEntry {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorDetails {
    pub code: String,
    pub kind: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context: Vec<ErrorContextEntry>,
}

impl ErrorDetails {
    pub fn context_from_vec_string(&mut self, entries: Vec<(String, String)>) {
        self.context = entries
            .into_iter()
            .map(|(key, value)| ErrorContextEntry { key, value })
            .collect();
    }

    pub fn context_as_map(&self) -> std::collections::BTreeMap<String, String> {
        self.context
            .iter()
            .map(|entry| (entry.key.clone(), entry.value.clone()))
            .collect()
    }
}

#[derive(Debug, Error)]
pub struct Error {
    pub code: ErrorCode,
    pub kind: ErrorKind,
    pub message: String,
    pub context: Vec<(String, String)>,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)?;
        for (key, value) in &self.context {
            write!(f, "\n  {key}: {value}")?;
        }
        Ok(())
    }
}

impl Error {
    pub fn from_code(code: ErrorCode) -> Self {
        Self {
            kind: code.kind(),
            message: code.message().to_string(),
            code,
            context: Vec::new(),
        }
    }

    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            kind: code.kind(),
            code,
            message: message.into(),
            context: Vec::new(),
        }
    }

    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.push((key.into(), value.into()));
        self
    }

    pub fn with_context_str(
        mut self,
        key: impl Into<String>,
        value: impl std::fmt::Display,
    ) -> Self {
        self.context.push((key.into(), value.to_string()));
        self
    }

    pub fn exit_code(&self) -> i32 {
        self.code.exit_code()
    }

    pub fn details(&self) -> ErrorDetails {
        ErrorDetails {
            code: self.code.id(),
            kind: format!("{:?}", self.kind).to_lowercase(),
            message: self.message.clone(),
            context: self
                .context
                .iter()
                .map(|(key, value)| ErrorContextEntry {
                    key: key.clone(),
                    value: value.clone(),
                })
                .collect(),
        }
    }
}

impl ErrorCode {
    pub fn error(self) -> Error {
        Error::from_code(self)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        ErrorCode::IoFailure
            .error()
            .with_context("error", err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        ErrorCode::SerializationFailure
            .error()
            .with_context("error", err.to_string())
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        ErrorCode::ConfigInvalid
            .error()
            .with_context("error", err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
