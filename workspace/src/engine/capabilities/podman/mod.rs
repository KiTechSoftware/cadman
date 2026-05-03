use std::collections::BTreeMap;

use serde_json::Value;
use veltrix::{
    os::unistd::{self, Uid},
    services::podman::{PodmanCliClient, PodmanCliSpec, PodmanContainerSummary},
};

use crate::engine::{
    ErrorCode, Result,
    constants::CADMAN_USER_NAME,
    models::{
        containers::{
            ContainerListReport, ContainerPort, ContainerRuntimeContext, ContainerSummary,
        },
        runtime::{RunMode, Runtime},
    },
    system::process::Process,
};

#[derive(Debug, Clone, Copy)]
pub enum ContainerListFilter {
    Running,
    Stopped,
    All,
}

impl ContainerListFilter {
    fn includes(self, container: &ContainerSummary) -> bool {
        let running = container.state.to_ascii_lowercase().contains("running");

        match self {
            Self::Running => running,
            Self::Stopped => !running,
            Self::All => true,
        }
    }
}

pub async fn list_containers(
    runtime: &Runtime,
    filter: ContainerListFilter,
) -> Result<ContainerListReport> {
    let requested_mode = runtime.run_mode();
    let effective_mode = runtime.effective_run_mode();
    if !Process::new("podman").binary_exists() {
        return Err(ErrorCode::PodmanMissing
            .error()
            .with_context("binary", "podman not found in PATH"));
    }

    let client = PodmanCliClient::new(cli_spec_for_mode(effective_mode));

    let response = client.containers_async().await.map_err(|err| {
        ErrorCode::PodmanCommandFailed
            .error()
            .with_context("command", "podman ps --all --format json")
            .with_context("error", err.to_string())
    })?;

    let mut containers = Vec::new();
    for item in response.data {
        let value = value_from_podman_summary(item);
        let container = container_from_value(&value);

        if filter.includes(&container) {
            containers.push(container);
        }
    }

    Ok(ContainerListReport {
        runtime: ContainerRuntimeContext {
            requested_mode: requested_mode.to_string(),
            effective_mode: effective_mode.to_string(),
            effective_user: runtime.effective_user().to_string(),
            podman_source: "cli".to_string(),
        },
        containers,
    })
}

// ── Podman passthrough wrappers ─────────────────────────────────────────────
pub async fn run(mode: RunMode, args: Vec<String>) -> Result<()> {
    let proc = Process::new("podman").set_args(args).set_mode(mode);

    if !proc.binary_exists() {
        return Err(ErrorCode::PodmanMissing
            .error()
            .with_context("binary", "podman not found in PATH"));
    }
    proc.run_async().await?.emit()
}

fn cli_spec_for_mode(mode: RunMode) -> PodmanCliSpec {
    let mut spec = PodmanCliSpec::new();
    let current_uid = unistd::geteuid();
    let cadman_uid = unistd::uid_by_username(CADMAN_USER_NAME);

    match mode {
        RunMode::Current => {
            if cadman_uid.is_some_and(|uid| current_uid == uid) {
                if let Some(uid) = cadman_uid {
                    spec = spec.uid(uid.as_raw());
                }
            }
        }
        RunMode::Cadman => {
            if cadman_uid.is_some_and(|uid| current_uid == uid) {
                if let Some(uid) = cadman_uid {
                    spec = spec.uid(uid.as_raw());
                }
            } else {
                spec = spec.sudo();
                if let Some(uid) = cadman_uid {
                    spec = spec.uid(uid.as_raw());
                }
            }
        }
        RunMode::Root => {
            if current_uid != Uid::from_raw(0) {
                spec = spec.sudo();
            }
        }
    }

    spec
}

fn value_from_podman_summary(item: PodmanContainerSummary) -> Value {
    let mut value = serde_json::Map::new();

    if let Some(id) = item.id {
        value.insert("id".to_string(), Value::String(id));
    }
    if let Some(names) = item.names {
        value.insert(
            "names".to_string(),
            Value::Array(names.into_iter().map(Value::String).collect()),
        );
    }
    if let Some(image) = item.image {
        value.insert("image".to_string(), Value::String(image));
    }
    if let Some(state) = item.state {
        value.insert("state".to_string(), Value::String(state));
    }
    if let Some(status) = item.status {
        value.insert("status".to_string(), Value::String(status));
    }

    value.extend(item.extra);

    Value::Object(value)
}

fn container_from_value(value: &Value) -> ContainerSummary {
    let state = first_string(value, &["state", "State", "status", "Status"])
        .unwrap_or_else(|| "-".to_string());

    ContainerSummary {
        id: first_string(value, &["id", "Id", "ID", "container_id"]).unwrap_or_default(),
        name: first_name(value).unwrap_or_default(),
        image: first_string(value, &["image", "Image"]).unwrap_or_default(),
        health: first_string(value, &["health", "Health"]),
        ports: parse_ports(first_field(value, &["ports", "Ports"])),
        labels: parse_labels(first_field(value, &["labels", "Labels"])),
        state,
    }
}

fn first_name(value: &Value) -> Option<String> {
    first_string(value, &["name", "Name"]).or_else(|| {
        first_field(value, &["names", "Names"]).and_then(|names| match names {
            Value::Array(items) => items.first().and_then(Value::as_str).map(str::to_string),
            Value::String(name) => Some(name.clone()),
            _ => None,
        })
    })
}

fn first_string(value: &Value, keys: &[&str]) -> Option<String> {
    first_field(value, keys).and_then(|field| match field {
        Value::String(value) => Some(value.clone()),
        Value::Number(value) => Some(value.to_string()),
        Value::Bool(value) => Some(value.to_string()),
        _ => None,
    })
}

fn first_field<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a Value> {
    keys.iter().find_map(|key| value.get(*key))
}

fn parse_labels(value: Option<&Value>) -> BTreeMap<String, String> {
    let Some(Value::Object(labels)) = value else {
        return BTreeMap::new();
    };

    labels
        .iter()
        .filter_map(|(key, value)| {
            value
                .as_str()
                .map(str::to_string)
                .or_else(|| match value {
                    Value::Null => None,
                    other => Some(other.to_string()),
                })
                .map(|value| (key.clone(), value))
        })
        .collect()
}

fn parse_ports(value: Option<&Value>) -> Vec<ContainerPort> {
    match value {
        Some(Value::Array(items)) => items.iter().flat_map(parse_port_value).collect(),
        Some(Value::String(value)) => parse_port_string(value),
        _ => Vec::new(),
    }
}

fn parse_port_value(value: &Value) -> Vec<ContainerPort> {
    match value {
        Value::Object(_) => vec![ContainerPort {
            host_ip: first_string(value, &["host_ip", "HostIp", "HostIP", "hostIP"])
                .filter(|s| !s.is_empty()),
            host_port: first_u16(value, &["host_port", "HostPort", "hostPort", "HostPort"]),
            container_port: first_u16(
                value,
                &[
                    "container_port",
                    "ContainerPort",
                    "containerPort",
                    "PrivatePort",
                    "privatePort",
                ],
            )
            .unwrap_or(0),
            protocol: first_string(value, &["protocol", "Protocol", "type", "Type"])
                .unwrap_or_else(|| "tcp".to_string()),
        }],
        Value::String(value) => parse_port_string(value),
        _ => Vec::new(),
    }
}

fn first_u16(value: &Value, keys: &[&str]) -> Option<u16> {
    first_field(value, keys).and_then(|field| match field {
        Value::Number(value) => value.as_u64().and_then(|value| u16::try_from(value).ok()),
        Value::String(value) => value.parse().ok(),
        _ => None,
    })
}

fn parse_port_string(value: &str) -> Vec<ContainerPort> {
    value
        .split(',')
        .filter_map(|entry| parse_single_port(entry.trim()))
        .collect()
}

fn parse_single_port(value: &str) -> Option<ContainerPort> {
    if value.is_empty() {
        return None;
    }

    let (mapping, protocol) = value.rsplit_once('/').unwrap_or((value, "tcp"));
    let right = mapping
        .rsplit_once("->")
        .map_or(mapping, |(_, right)| right);
    let container_port = right.split('/').next()?.parse().ok()?;

    let left = mapping.rsplit_once("->").map(|(left, _)| left);
    let (host_ip, host_port) = left
        .and_then(|left| {
            left.rsplit_once(':').map(|(ip, port)| {
                (
                    (!ip.is_empty()).then(|| ip.to_string()),
                    port.parse::<u16>().ok(),
                )
            })
        })
        .unwrap_or((None, None));

    Some(ContainerPort {
        host_ip,
        host_port,
        container_port,
        protocol: protocol.to_string(),
    })
}

pub async fn compose(mode: RunMode, args: Vec<String>) -> Result<()> {
    let proc = Process::new("podman-compose").set_args(args).set_mode(mode);
    if !proc.binary_exists() {
        return Err(ErrorCode::PodmanComposeMissing
            .error()
            .with_context("binary", "podman-compose not found in PATH"));
    }
    proc.run_async().await?.emit()
}
