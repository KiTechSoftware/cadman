use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ContainerScope {
    Current,
    Cadman,
    Root,
}

impl ContainerScope {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::Cadman => "cadman",
            Self::Root => "root",
        }
    }
}

impl std::fmt::Display for ContainerScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerListReport {
    pub runtime: ContainerRuntimeContext,
    pub containers: Vec<ContainerSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerRuntimeContext {
    pub install_scope: String,
    pub requested_mode: String,
    pub effective_mode: String,
    pub effective_user: String,
    pub authorized_scopes: Vec<String>,
    pub podman_source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerSummary {
    pub scope: ContainerScope,
    pub id: String,
    pub name: String,
    pub image: String,
    pub state: String,
    pub health: Option<String>,
    pub ports: Vec<ContainerPort>,
    pub labels: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerPort {
    pub host_ip: Option<String>,
    pub host_port: Option<u16>,
    pub container_port: u16,
    pub protocol: String,
}
