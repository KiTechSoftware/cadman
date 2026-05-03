use std::path::PathBuf;

use serde::Serialize;

use crate::engine::{
    capabilities::caddy::apply::{CaddyApplyReport, CaddySiteActionKind},
    registry::RegistryApp,
};

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
    pub counts: ReconcileCounts,
    pub state_updated: bool,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ReconcileCounts {
    pub routes_planned: usize,
    pub sites_created: usize,
    pub sites_updated: usize,
    pub sites_unchanged: usize,
    pub sites_removed: usize,
}

impl ReconcileCounts {
    pub fn from_report(routes_planned: usize, caddy: &CaddyApplyReport) -> Self {
        Self {
            routes_planned,
            sites_created: count_actions(caddy, CaddySiteActionKind::Create),
            sites_updated: count_actions(caddy, CaddySiteActionKind::Update),
            sites_unchanged: count_actions(caddy, CaddySiteActionKind::Unchanged),
            sites_removed: count_actions(caddy, CaddySiteActionKind::Remove),
        }
    }
}

fn count_actions(caddy: &CaddyApplyReport, kind: CaddySiteActionKind) -> usize {
    caddy
        .actions
        .iter()
        .filter(|action| action.action == kind)
        .count()
}
