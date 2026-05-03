use crate::{
    core::{Context, CoreResult},
    engine::capabilities::reconcile::{
        self as engine_reconcile, ReconcileReport, ReconcileRequest,
    },
};

pub async fn render(ctx: &Context) -> CoreResult<()> {
    let report = reconcile(ctx).await?;
    let mut output = ctx
        .ui()
        .new_output_content()
        .json(&report)
        .title("Cadman reconcile")
        .key_value("Dry Run", report.dry_run)
        .key_value("Registry", report.registry_path.display())
        .key_value("Desired Routes", report.desired_routes.len())
        .key_value("Registered Apps", report.registered_apps.len())
        .key_value("Updated Apps", report.updated_apps.len())
        .key_value("Auto Projects", report.auto_registered_projects.len())
        .key_value("Missing Label Apps", report.missing_label_apps.len());

    if !report.desired_routes.is_empty() {
        output = output.table(None, routes_table(&report))
    }

    ctx.ui().print(&output)
}

pub async fn reconcile(ctx: &Context) -> CoreResult<ReconcileReport> {
    engine_reconcile::reconcile(
        ctx.runtime(),
        ReconcileRequest {
            dry_run: ctx.runtime().options().dry_run(),
        },
    )
    .await
}

fn routes_table(report: &ReconcileReport) -> scriba::Table {
    let rows = report
        .desired_routes
        .iter()
        .map(|route| {
            vec![
                route.app_id.clone(),
                route.route_id.clone(),
                format!("{:?}", route.source),
                route.hosts.join(","),
                route.path.clone().unwrap_or_else(|| "-".to_string()),
                route.upstream.clone(),
            ]
        })
        .collect();

    scriba::Table::new(
        vec![
            "APP".to_string(),
            "ROUTE".to_string(),
            "SOURCE".to_string(),
            "HOSTS".to_string(),
            "PATH".to_string(),
            "UPSTREAM".to_string(),
        ],
        rows,
    )
}
