use crate::{
    core::{Context, CoreResult},
    engine::{
        ErrorCode,
        capabilities::podman::{self, ContainerListFilter},
        system::terminal::TerminalSize,
    },
};

pub use crate::engine::models::containers::{ContainerListReport, ContainerPort, ContainerSummary};

#[derive(Debug, Clone, Copy)]
pub struct ContainerListOptions {
    pub all: bool,
    pub running: bool,
    pub stopped: bool,
}

impl ContainerListOptions {
    fn filter(self) -> CoreResult<ContainerListFilter> {
        let selected = [self.all, self.running, self.stopped]
            .into_iter()
            .filter(|value| *value)
            .count();

        if selected > 1 {
            return Err(ErrorCode::ValidationFailed.error().with_context(
                "flags",
                "--all, --running, and --stopped are mutually exclusive",
            ));
        }

        Ok(if self.all {
            ContainerListFilter::All
        } else if self.stopped {
            ContainerListFilter::Stopped
        } else {
            ContainerListFilter::Running
        })
    }
}

pub async fn list(
    ctx: &Context,
    all: bool,
    running: bool,
    stopped: bool,
    labels: bool,
) -> CoreResult<()> {
    let options = ContainerListOptions {
        all,
        running,
        stopped,
    };
    let filter = options.filter()?;

    let report = podman::list_containers(ctx.runtime(), filter).await?;
    let ui = ctx.ui();
    let mut output = ui
        .new_output_content()
        .key_value("Requested RunMode", &report.runtime.requested_mode)
        .key_value("Effective RunMode", &report.runtime.effective_mode)
        .key_value("Effective user", &report.runtime.effective_user)
        .key_value("Podman source", &report.runtime.podman_source);

    let terminal_size = TerminalSize::current();
    output = output.table(None, container_table(&report, labels, terminal_size));
    ctx.ui().print(&output)
}

fn container_table(
    report: &ContainerListReport,
    include_labels: bool,
    terminal_size: TerminalSize,
) -> scriba::Table {
    let layout = terminal_size.table_layout();

    let mut headers = vec![
        "NAME".to_string(),
        "ID".to_string(),
        "IMAGE".to_string(),
        "STATE".to_string(),
        "HEALTH".to_string(),
        "PORTS".to_string(),
    ];

    if include_labels {
        headers.push("LABELS".to_string());
    }

    let rows = report
        .containers
        .iter()
        .map(|container| {
            let mut row = vec![
                text_or_dash(&container.name),
                short_id(&container.id),
                text_or_dash(&container.image),
                text_or_dash(&container.state),
                container.health.clone().unwrap_or_else(|| "-".to_string()),
                format_ports(&container.ports),
            ];

            if include_labels {
                row.push(format_labels(container));
            }

            row
        })
        .collect();

    scriba::Table::new(headers, rows).with_layout(layout)
}

fn text_or_dash(value: &str) -> String {
    if value.is_empty() {
        "-".to_string()
    } else {
        value.to_string()
    }
}

fn short_id(id: &str) -> String {
    if id.is_empty() {
        "-".to_string()
    } else {
        id.chars().take(12).collect()
    }
}

fn format_ports(ports: &[ContainerPort]) -> String {
    if ports.is_empty() {
        return "-".to_string();
    }

    ports
        .iter()
        .map(|port| {
            let container = format!("{}/{}", port.container_port, port.protocol);
            match (&port.host_ip, port.host_port) {
                (Some(host_ip), Some(host_port)) => format!("{host_ip}:{host_port}->{container}"),
                (None, Some(host_port)) => format!("{host_port}->{container}"),
                _ => container,
            }
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn format_labels(container: &ContainerSummary) -> String {
    if container.labels.is_empty() {
        return "-".to_string();
    }

    container
        .labels
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join(",")
}
