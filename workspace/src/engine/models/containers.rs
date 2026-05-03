use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerListReport {
    pub runtime: ContainerRuntimeContext,
    pub containers: Vec<ContainerSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerRuntimeContext {
    pub requested_mode: String,
    pub effective_mode: String,
    pub effective_user: String,
    pub podman_source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerSummary {
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
