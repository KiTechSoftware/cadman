use std::path::PathBuf;

use serde::Serialize;

use crate::engine::{capabilities::caddy::apply::CaddyApplyReport, registry::RegistryApp};

use super::desired::DesiredRoute;

#[derive(Debug, Clone, Serialize)]
pub struct ReconcileReport {
    pub dry_run: bool,
    pub registry_path: PathBuf,
    pub desired_routes: Vec<DesiredRoute>,
    pub registered_apps: Vec<RegistryApp>,
    pub updated_apps: Vec<RegistryApp>,
    pub auto_registered_projects: Vec<RegistryApp>,
    pub missing_label_apps: Vec<String>,
    pub caddy: CaddyApplyReport,
}
