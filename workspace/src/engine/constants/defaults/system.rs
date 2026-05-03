pub const SYSTEM_SERVICE: &str = r#"[Unit]
Description=Cadman daemon
After=network.target podman.socket caddy.service
Wants=podman.socket

[Service]
Type=exec
ExecStart=/usr/local/bin/cadman serve
Restart=on-failure
RestartSec=5
User=cadman
Group=cadman

[Install]
WantedBy=multi-user.target
"#;

/// Generate a systemd service file.
/// If user_or_group is None, omits User/Group for user-level services.
pub fn generate_service_file(
    binary_location: &str,
    user: Option<&str>,
    group: Option<&str>,
) -> String {
    let mut service = SYSTEM_SERVICE.replace("/usr/local/bin/cadman", binary_location);
    if let (Some(user), Some(group)) = (user, group) {
        service = service
            .replace("User=cadman", &format!("User={}", user))
            .replace("Group=cadman", &format!("Group={}", group));
    } else {
        // Remove User/Group lines for user-level service
        service = service
            .lines()
            .filter(|line| {
                !line.trim_start().starts_with("User=") && !line.trim_start().starts_with("Group=")
            })
            .collect::<Vec<_>>()
            .join("\n");
        service.push('\n');
    }
    service
}
