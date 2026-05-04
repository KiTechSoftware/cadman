use std::fmt;

use serde::{Deserialize, Serialize};

use crate::engine::errors::ErrorKind;

pub mod exit_status {
    // User input errors
    pub const USER: i32 = 2;

    // Configuration/state errors
    pub const CONFIG: i32 = 10;
    pub const STATE: i32 = 11;

    // Permission/auth errors
    pub const PERMISSION: i32 = 13;
    pub const AUTHORIZATION: i32 = 14;

    // External integration errors
    pub const REGISTRY: i32 = 20;
    pub const PODMAN: i32 = 21;
    pub const CADDY: i32 = 22;
    pub const IO: i32 = 25;
    pub const PROCESS: i32 = 26;

    // Validation/workflow errors
    pub const VALIDATION: i32 = 30;

    // Runtime errors
    pub const RUNTIME: i32 = 50;
}

macro_rules! error_codes {
    (
        $(
            $variant:ident => ($kind:ident, $num:expr, $msg:expr)
        ),* $(,)?
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        pub enum ErrorCode {
            $($variant),*
        }

        impl ErrorCode {
            pub const fn kind(self) -> ErrorKind {
                match self {
                    $(Self::$variant => ErrorKind::$kind),*
                }
            }

            pub const fn message(self) -> &'static str {
                match self {
                    $(Self::$variant => $msg),*
                }
            }

            pub const fn index(self) -> u16 {
                match self {
                    $(Self::$variant => $num),*
                }
            }

            pub const fn numeric(self) -> u16 {
                self.kind().prefix() * 1000 + self.index()
            }

            pub fn id(self) -> String {
                format!("E{:04}", self.numeric())
            }

            pub const fn exit_code(self) -> i32 {
                match self.kind() {
                    ErrorKind::User => exit_status::USER,
                    ErrorKind::Config => exit_status::CONFIG,
                    ErrorKind::State => exit_status::STATE,
                    ErrorKind::Registry => exit_status::REGISTRY,
                    ErrorKind::Podman => exit_status::PODMAN,
                    ErrorKind::Caddy => exit_status::CADDY,
                    ErrorKind::Validation => exit_status::VALIDATION,
                    ErrorKind::Io => exit_status::IO,
                    ErrorKind::Process => exit_status::PROCESS,
                    ErrorKind::Runtime => exit_status::RUNTIME,
                    ErrorKind::Permission => exit_status::PERMISSION,
                    ErrorKind::Authorization => exit_status::AUTHORIZATION,
                }
            }
        }

        impl fmt::Display for ErrorCode {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{} ({})", self.message(), self.id())
            }
        }
    };
}

error_codes! {
    UserCancelled => (User, 1, "Operation cancelled"),
    UserInteractionRequired => (User, 2, "Interactive input required"),
    UserInvalidArgument => (User, 3, "Invalid argument"),
    FormatUnsupported => (User, 4, "Format is not supported for this command"),
    InvalidInput => (User, 5, "Input is invalid"),

    ConfigInvalid => (Config, 1, "Configuration is invalid"),
    ConfigUnreadable => (Config, 2, "Configuration is unreadable"),
    ConfigReferenceInvalid => (Config, 3, "Configuration reference is invalid"),
    ConfigNotFound => (Config, 4, "Configuration file not found"),
    ConfigUnsupportedFormat => (Config, 5, "Configuration format is not supported"),
    ConfigAlreadyExists => (Config, 6, "Configuration already exists"),

    StateInvalid => (State, 1, "State is invalid"),
    StateUnreadable => (State, 2, "State is unreadable"),
    StateWriteFailed => (State, 3, "State write failed"),
    StateLockFailed => (State, 4, "State lock failed"),
    StateConflict => (State, 5, "State conflict detected"),

    RegistryUnknown => (Registry, 1, "Registry is unknown"),
    RegistryInvalid => (Registry, 2, "Registry is invalid"),
    RegistryUnreachable => (Registry, 3, "Registry is unreachable"),
    RegistrySyncFailed => (Registry, 4, "Registry sync failed"),
    RegistryRefNotFound => (Registry, 5, "Registry ref not found"),
    RegistrySectionMissing => (Registry, 6, "Registry section is missing"),
    RegistryOffline => (Registry, 7, "Registry unavailable in offline mode"),
    RegistryWriteFailed => (Registry, 8, "Registry write failed"),
    RegistryLockFailed => (Registry, 9, "Registry lock failed"),
    RegistryAppNotFound => (Registry, 10, "Managed app not found"),
    RegistryAppAlreadyExists => (Registry, 11, "Managed app already exists"),

    PodmanMissing => (Podman, 1, "Podman is missing"),
    PodmanComposeMissing => (Podman, 2, "Podman Compose plugin is missing"),
    PodmanCommandFailed => (Podman, 3, "Podman command failed"),
    PodmanComposeCommandFailed => (Podman, 4, "Podman Compose command failed"),
    PodmanRefNotFound => (Podman, 5, "Podman image ref not found"),
    PodmanSocketUnavailable => (Podman, 6, "Podman socket is unavailable"),
    PodmanSocketConnectionFailed => (Podman, 7, "Podman socket connection failed"),
    PodmanContainerNotFound => (Podman, 8, "Podman container not found"),
    PodmanContainerRemoved => (Podman, 9, "Podman container has been removed"),

    CaddyMissing => (Caddy, 1, "Caddy is missing"),
    CaddyCommandFailed => (Caddy, 2, "Caddy command failed"),
    CaddyCaddyfileInvalid => (Caddy, 3, "Caddyfile is invalid"),
    CaddyCaddyfileNotFound => (Caddy, 4, "Caddyfile not found"),
    CaddyValidateFailed => (Caddy, 5, "Caddy validation failed"),
    CaddyReloadFailed => (Caddy, 6, "Caddy reload failed"),
    CaddySiteNotFound => (Caddy, 7, "Caddy site not found"),
    CaddySiteWriteFailed => (Caddy, 8, "Caddy site write failed"),
    CaddyApiUnavailable => (Caddy, 9, "Caddy API is unavailable"),
    CaddyApiReadFailed => (Caddy, 10, "Caddy API read failed"),

    ValidationFailed => (Validation, 1, "Validation failed"),
    ValidationProjectInvalid => (Validation, 2, "Project configuration is invalid"),
    ValidationRouteInvalid => (Validation, 3, "Route configuration is invalid"),
    ValidationPortInvalid => (Validation, 4, "Port configuration is invalid"),
    ValidationRunModeInvalid => (Validation, 5, "Run mode is invalid"),

    IoFailure => (Io, 1, "I/O failure"),
    SerializationFailure => (Io, 2, "Serialization failure"),
    OutputRenderFailure => (Io, 3, "Output render failure"),

    ProcessFailure => (Process, 1, "Process failure"),
    ProcessTimedOut => (Process, 2, "Process timed out"),
    ProcessInterrupted => (Process, 3, "Process interrupted"),

    RuntimeFailure => (Runtime, 1, "Runtime failure"),
    RuntimeUnsupportedOS => (Runtime, 2, "Operating system is not supported"),
    RuntimeUnsupportedContainerMode => (Runtime, 3, "Runtime mode is not supported in containerized Cadman"),
    RuntimeCapabilityUnavailable => (Runtime, 4, "Requested capability is unavailable"),

    PermissionDenied => (Permission, 1, "Permission denied"),
    PermissionModeDenied => (Permission, 2, "Requested run mode is not permitted"),
    PermissionRootRequired => (Permission, 3, "Root privileges are required"),
    PermissionCadmanGroupRequired => (Permission, 4, "Cadman group membership is required"),
    PermissionCadmanRootGroupRequired => (Permission, 5, "Cadman root group membership is required"),

    AuthorizationFailed => (Authorization, 1, "Authorization failed"),
    AuthorizationPolicyDenied => (Authorization, 2, "Operation denied by policy"),
    AuthorizationScopeDenied => (Authorization, 3, "Operation is outside the authorized scope"),
}
