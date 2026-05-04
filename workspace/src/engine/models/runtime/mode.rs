use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum InteractiveMode {
    #[default]
    Interactive,
    NonInteractive,
    Ci,
}

impl InteractiveMode {
    pub fn from_flags(ci: bool, non_interactive: bool) -> Self {
        if ci {
            Self::Ci
        } else if non_interactive {
            Self::NonInteractive
        } else {
            Self::Interactive
        }
    }

    pub fn allows_prompts(self) -> bool {
        matches!(self, Self::Interactive)
    }

    pub fn is_strict(self) -> bool {
        matches!(self, Self::Ci | Self::NonInteractive)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum RunMode {
    #[default]
    Current,
    Cadman,
    Root,
}

impl RunMode {
    pub fn parse(value: &str) -> Self {
        match value {
            "current" | "user" => Self::Current,
            "cadman" => Self::Cadman,
            "root" => Self::Root,
            _ => Self::Current,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::Cadman => "cadman",
            Self::Root => "root",
        }
    }

    pub fn is_poc_supported(self) -> bool {
        matches!(self, Self::Current | Self::Cadman)
    }
}

impl std::fmt::Display for RunMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
