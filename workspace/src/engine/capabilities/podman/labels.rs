use std::collections::BTreeMap;

use serde_json::Value;

use crate::engine::constants::{
    LABEL_CADMAN_ENABLE, LABEL_CADMAN_HOST, LABEL_CADMAN_PORT, LABEL_CADMAN_SERVICE,
};

pub fn labels_from_container_value(value: &Value) -> BTreeMap<String, String> {
    let mut labels = BTreeMap::new();
    let Some(raw) = value.get("Labels").or_else(|| value.get("labels")) else {
        return labels;
    };

    match raw {
        Value::Object(map) => {
            for (key, value) in map {
                let value = value.as_str().map(str::to_string).or_else(|| match value {
                    Value::Null => None,
                    other => Some(other.to_string()),
                });

                if let Some(value) = value {
                    labels.insert(key.clone(), value);
                }
            }
        }
        Value::String(text) => {
            for item in text.split(',') {
                if let Some((key, value)) = item.split_once('=') {
                    labels.insert(key.trim().to_string(), value.trim().to_string());
                }
            }
        }
        _ => {}
    }

    labels
}

pub fn enabled(labels: &BTreeMap<String, String>) -> bool {
    labels
        .get(LABEL_CADMAN_ENABLE)
        .or_else(|| labels.get("cadman.enabled"))
        .map(|value| matches!(value.as_str(), "true" | "1" | "yes" | "on"))
        .unwrap_or(false)
}

pub fn split_hosts(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|host| host.trim().trim_matches('"').trim_matches('`').to_string())
        .filter(|host| !host.is_empty())
        .collect()
}

pub fn parse_host_rule(rule: &str) -> Vec<String> {
    let Some(start) = rule.find("Host(") else {
        return Vec::new();
    };
    let rest = &rule[start + "Host(".len()..];
    let Some(end) = rest.find(')') else {
        return Vec::new();
    };

    split_hosts(&rest[..end])
}

pub fn router_service(labels: &BTreeMap<String, String>) -> Option<String> {
    labels
        .iter()
        .find(|(key, _)| key.starts_with("cadman.http.routers.") && key.ends_with(".service"))
        .map(|(_, value)| value.clone())
}

pub fn service_port(labels: &BTreeMap<String, String>, service: &str) -> Option<u16> {
    labels
        .get(&format!(
            "cadman.http.services.{service}.loadbalancer.server.port"
        ))
        .or_else(|| labels.get(LABEL_CADMAN_PORT))
        .and_then(|value| value.parse().ok())
        .or_else(|| {
            labels
                .iter()
                .find(|(key, _)| {
                    key.starts_with("cadman.http.services.")
                        && key.ends_with(".loadbalancer.server.port")
                })
                .and_then(|(_, value)| value.parse().ok())
        })
}

pub fn sanitize_file_stem(value: &str) -> String {
    let stem: String = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect();

    stem.trim_matches('-').to_string()
}

pub fn hosts(labels: &BTreeMap<String, String>) -> Vec<String> {
    if let Some(host) = labels.get(LABEL_CADMAN_HOST) {
        return split_hosts(host);
    }

    labels
        .iter()
        .find(|(key, _)| key.starts_with("cadman.http.routers.") && key.ends_with(".rule"))
        .map(|(_, value)| parse_host_rule(value))
        .unwrap_or_default()
}

pub fn service(labels: &BTreeMap<String, String>) -> Option<String> {
    labels
        .get(LABEL_CADMAN_SERVICE)
        .cloned()
        .or_else(|| router_service(labels))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn cadman_enable_true_is_enabled() {
        let labels = BTreeMap::from([(LABEL_CADMAN_ENABLE.to_string(), "true".to_string())]);

        assert!(enabled(&labels));
    }

    #[test]
    fn cadman_enable_false_is_not_enabled() {
        let labels = BTreeMap::from([(LABEL_CADMAN_ENABLE.to_string(), "false".to_string())]);

        assert!(!enabled(&labels));
    }

    #[test]
    fn cadman_host_supports_comma_separated_hosts() {
        assert_eq!(
            split_hosts("one.local, two.local"),
            vec!["one.local", "two.local"]
        );
    }

    #[test]
    fn host_rule_parses_hosts() {
        assert_eq!(
            parse_host_rule("Host(`one.local`,`two.local`)"),
            vec!["one.local", "two.local"]
        );
    }

    #[test]
    fn router_service_parses_from_router_label() {
        let labels = BTreeMap::from([(
            "cadman.http.routers.web.service".to_string(),
            "app".to_string(),
        )]);

        assert_eq!(router_service(&labels), Some("app".to_string()));
    }

    #[test]
    fn service_port_parses_from_service_label() {
        let labels = BTreeMap::from([(
            "cadman.http.services.app.loadbalancer.server.port".to_string(),
            "8080".to_string(),
        )]);

        assert_eq!(service_port(&labels, "app"), Some(8080));
    }

    #[test]
    fn sanitize_file_stem_removes_unsafe_chars() {
        assert_eq!(sanitize_file_stem("../my app:8080"), "my-app-8080");
    }

    #[test]
    fn labels_from_container_value_handles_object_labels() {
        let value = json!({
            "Labels": {
                "cadman.enable": "true",
                "cadman.port": 8080
            }
        });
        let labels = labels_from_container_value(&value);

        assert_eq!(labels.get("cadman.enable"), Some(&"true".to_string()));
        assert_eq!(labels.get("cadman.port"), Some(&"8080".to_string()));
    }

    #[test]
    fn labels_from_container_value_handles_comma_string_labels() {
        let value = json!({
            "labels": "cadman.enable=true,cadman.host=app.local"
        });
        let labels = labels_from_container_value(&value);

        assert_eq!(labels.get("cadman.enable"), Some(&"true".to_string()));
        assert_eq!(labels.get("cadman.host"), Some(&"app.local".to_string()));
    }
}
