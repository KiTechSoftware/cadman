use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ErrorKind {
    User,
    Config,
    State,
    Registry,
    Podman,
    Caddy,
    Validation,
    Io,
    Process,
    Runtime,
    Permission,
    Authorization,
}

impl ErrorKind {
    pub const fn prefix(self) -> u16 {
        match self {
            Self::User => 0,
            Self::Config => 1,
            Self::State => 2,
            Self::Registry => 3,
            Self::Podman => 4,
            Self::Caddy => 5,
            Self::Validation => 6,
            Self::Io => 7,
            Self::Process => 8,
            Self::Runtime => 9,
            Self::Permission => 10,
            Self::Authorization => 11,
        }
    }
}
