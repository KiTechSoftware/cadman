use std::error::Error;

use serde::Serialize;
use serde_json::Value;
use scriba::{Output, Table, Ui};
use veltrix::services::podman::{PodmanCliClient, PodmanCliSpec};

fn main() -> Result<(), Box<dyn Error>> {
    let containers = running_podman_containers()?;
    let rows = containers
        .into_iter()
        .map(|container| vec![container.name, container.labels, container.status])
        .collect();

    let table = Table::new(
        vec!["Container".into(), "Labels".into(), "Status".into()],
        rows,
    );
    let output = Output::new().table(Some("Running Podman Containers".into()), table);

    Ui::new().print(&output)?;

    Ok(())
}

#[derive(Debug)]
struct ContainerRow {
    name: String,
    labels: String,
    status: String,
}

fn running_podman_containers() -> Result<Vec<ContainerRow>, Box<dyn Error>> {
    let podman = PodmanCliClient::new(PodmanCliSpec::default());
    let containers = podman
        .containers()?
        .data
        .into_iter()
        .filter_map(|container| container_row_from_summary(&container).transpose())
        .collect::<Result<Vec<_>, _>>()?;

    Ok(containers)
}

fn container_row_from_summary<T>(summary: &T) -> Result<Option<ContainerRow>, serde_json::Error>
where
    T: Serialize,
{
    let value = serde_json::to_value(summary)?;
    let status = string_field(&value, &["status", "Status", "state", "State"]);

    if !status.to_ascii_lowercase().contains("running") {
        return Ok(None);
    }

    Ok(Some(ContainerRow {
        name: string_field(&value, &["names", "Names", "name", "Name"]),
        labels: labels_field(&value),
        status,
    }))
}

fn string_field(value: &Value, keys: &[&str]) -> String {
    keys.iter()
        .find_map(|key| value.get(*key))
        .and_then(value_to_string)
        .unwrap_or_else(|| "-".into())
}

fn labels_field(value: &Value) -> String {
    let labels = ["labels", "Labels"]
        .iter()
        .find_map(|key| value.get(*key))
        .and_then(value_to_labels)
        .unwrap_or_default();

    if labels.is_empty() || labels == "<none>" {
        "-".into()
    } else {
        labels
    }
}

fn value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.trim_start_matches('/').to_owned()),
        Value::Array(items) => Some(
            items
                .iter()
                .filter_map(value_to_string)
                .collect::<Vec<_>>()
                .join(", "),
        ),
        _ => None,
    }
}

fn value_to_labels(value: &Value) -> Option<String> {
    match value {
        Value::Object(labels) => Some(
            labels
                .iter()
                .map(|(key, value)| {
                    let value = value_to_string(value).unwrap_or_else(|| value.to_string());
                    format!("{key}={value}")
                })
                .collect::<Vec<_>>()
                .join(", "),
        ),
        Value::String(text) => Some(text.to_owned()),
        _ => None,
    }
}

#[allow(dead_code)]
fn parse_container_row(line: &str) -> ContainerRow {
    let mut columns = line.splitn(3, '\t');

    ContainerRow {
        name: clean_cell(columns.next()),
        labels: clean_labels(columns.next()),
        status: clean_cell(columns.next()),
    }
}

fn clean_labels(labels: Option<&str>) -> String {
    let labels = clean_cell(labels);

    if labels.is_empty() || labels == "<none>" {
        "-".into()
    } else {
        labels
    }
}

fn clean_cell(value: Option<&str>) -> String {
    value.unwrap_or_default().trim().to_owned()
}
