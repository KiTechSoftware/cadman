pub const LABEL_PREFIX: &str = "cadman";
pub const LABEL_SEPARATOR: &str = ".";

// reserved labels for cadman
pub const LABEL_CADMAN_ENABLE: &str = "cadman.enable";
pub const LABEL_CADMAN_HOST: &str = "cadman.host";
pub const LABEL_CADMAN_PORT: &str = "cadman.port";
pub const LABEL_CADMAN_SERVICE: &str = "cadman.service";
pub const LABEL_CADMAN_PATH: &str = "cadman.path";

/// Create a label with the standard prefix and separator.
/// If section already starts with the prefix, don't add it again.
pub fn make_label(section: &str, key: Option<&str>) -> String {
    let base = if section.starts_with(LABEL_PREFIX) {
        section.to_string()
    } else {
        make_label_key(section)
    };
    match key {
        Some(k) => make_label_with_prefix(&base, k),
        None => base,
    }
}

/// Create a label with the standard prefix and a key (no extra section)
pub fn make_label_key(key: &str) -> String {
    make_label_with_prefix(LABEL_PREFIX, key)
}

fn make_label_with_prefix(prefix: &str, key: &str) -> String {
    format!("{}{}{}", prefix, LABEL_SEPARATOR, key)
}

/// Check if a label starts with the cadman prefix
pub fn check_for_prefix(label: &str) -> bool {
    label.starts_with(LABEL_PREFIX)
}

/// Parse a label into (prefix, rest) if it starts with the prefix
pub fn parse_label(label: &str) -> Option<(&str, &str)> {
    if label.starts_with(LABEL_PREFIX) {
        let (prefix, rest) = label.split_once(LABEL_SEPARATOR)?;
        Some((prefix, rest))
    } else {
        None
    }
}
