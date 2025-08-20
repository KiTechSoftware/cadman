# Acronyms & Abbreviations

> This page lists all abbreviations and shorthand used in Cadman’s documentation, CLI help, and requirements/specifications.

---

## General

| Acronym | Meaning                           | Description                                                                                      |
| ------- | --------------------------------- | ------------------------------------------------------------------------------------------------ |
| **API** | Application Programming Interface | A set of rules and functions that allow software to talk to each other.                          |
| **CLI** | Command-Line Interface            | Text-based interface where you run Cadman commands.                                              |
| **UI**  | User Interface                    | Any interface intended for direct user interaction — in Cadman’s case, typically the CLI output. |
| **UX**  | User Experience                   | The overall feel of using Cadman, including ease of use and clarity.                             |
| **FS**  | File System                       | The structure of files and directories on disk.                                                  |
| **PID** | Process Identifier                | A number that uniquely identifies a running process on the system.                               |
| **TTY** | Teletype Terminal                 | Terminal/console environment.                                                                    |
| **UID** | User Identifier                   | Numeric ID assigned to a system user.                                                            |
| **GID** | Group Identifier                  | Numeric ID assigned to a system group.                                                           |

---

## Development & Build

| Acronym  | Meaning                          | Description                                                                  |
| -------- | -------------------------------- | ---------------------------------------------------------------------------- |
| **CI**   | Continuous Integration           | Automated process for building and testing code whenever changes are pushed. |
| **CD**   | Continuous Delivery/Deployment   | Automatically delivering built software to a deployment target.              |
| **RTM**  | Requirements Traceability Matrix | Table mapping requirements to implementation and tests.                      |
| **FR**   | Functional Requirement           | A specific behaviour or feature Cadman must implement.                       |
| **NFR**  | Non-Functional Requirement       | A constraint or quality requirement (e.g. performance, security).            |
| **MSRV** | Minimum Supported Rust Version   | Oldest Rust compiler version Cadman supports.                                |
| **SBOM** | Software Bill of Materials       | List of all components and dependencies in a build.                          |

---

## Platforms & Services

| Acronym     | Meaning            | Description                                                            |
| ----------- | ------------------ | ---------------------------------------------------------------------- |
| **OS**      | Operating System   | Software that manages computer hardware and software (Linux, macOS).   |
| **Podman**  | Pod Manager        | A daemonless container engine used by Cadman for running applications. |
| **Compose** | Podman Compose     | A tool to run multi-container applications defined in a YAML file.     |
| **Caddy**   | Caddy Web Server   | A web server with automatic HTTPS used as Cadman’s reverse proxy.      |
| **DNS**     | Domain Name System | Translates domain names (e.g., `example.com`) into IP addresses.       |

---

## Files & Config

| Acronym  | Meaning                                                  | Description                                                     |
| -------- | -------------------------------------------------------- | --------------------------------------------------------------- |
| **YAML** | YAML Ain’t Markup Language                               | Human-readable configuration file format.                       |
| **TOML** | Tom’s Obvious, Minimal Language                          | Configuration file format used for Cadman’s registry and state. |
| **JSON** | JavaScript Object Notation                               | Text format for data interchange, often used in APIs and logs.  |
| **ULID** | Universally Unique Lexicographically Sortable Identifier | A unique ID format Cadman uses for applications.                |

---

## Testing & Codes

| Acronym       | Meaning                    | Description                                                          |
| ------------- | -------------------------- | -------------------------------------------------------------------- |
| **U**         | Unit Test                  | Tests a single function or module in isolation.                      |
| **I**         | Integration Test           | Tests how different parts of Cadman work together.                   |
| **E**         | End-to-End Test            | Simulates real user flows from start to finish.                      |
| **S**         | Static Analysis            | Automated code review for security, style, or correctness.           |
| **P**         | Performance Test           | Measures how fast Cadman performs certain operations.                |
| **Exit Code** | Program Termination Status | Numeric code indicating success (`0`) or type of error (`non-zero`). |

---

## Operations & Deployment

| Acronym    | Meaning                     | Description                                                        |
| ---------- | --------------------------- | ------------------------------------------------------------------ |
| **STDIN**  | Standard Input              | Data stream input to a command.                                    |
| **STDOUT** | Standard Output             | Data stream output from a command.                                 |
| **STDERR** | Standard Error              | Data stream for error messages.                                    |
| **FQDN**   | Fully Qualified Domain Name | Complete domain name that specifies its exact location in the DNS. |
| **TLS**    | Transport Layer Security    | Encryption protocol for secure communication over the internet.    |
| **OIDC**   | OpenID Connect              | Authentication protocol used for identity in CI/CD pipelines.      |
