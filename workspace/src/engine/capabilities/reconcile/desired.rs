use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesiredRoute {
    pub app_id: String,
    pub route_id: String,
    pub hosts: Vec<String>,
    pub path: Option<String>,
    pub upstream: String,
    pub source: RouteSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RouteSource {
    ProjectConfig,
    PodmanLabels,
}
