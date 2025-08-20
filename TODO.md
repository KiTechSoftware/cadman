create a simple rust cli that interacts with the caddy and podman API, or podman sockets


docker-compose.yml
```yaml
services:
  cadman:
    image: cadman:v0.1.0
    container_name: cadman
    restart: unless-stopped
    security_opt:
      - no-new-privileges:true

    networks:
     # Connect to the 'cadman_proxy' overlay network for inter-container communication across nodes
      - proxy

    ports:
      - "80:80"
      - "443:443"
      - "8080:8080"

    volumes:
      - /var/run/podman.sock:/var/run/podman.sock:ro

    command:
      # EntryPoints
      - "--entrypoints.web.address=:80"
      - "--entrypoints.web.http.redirections.entrypoint.to=websecure"
      - "--entrypoints.web.http.redirections.entrypoint.scheme=https"
      - "--entrypoints.web.http.redirections.entrypoint.permanent=true"
      - "--entrypoints.websecure.address=:443"
      - "--entrypoints.websecure.http.tls=true"

      # Providers 
      - "--providers.podman=true"
      - "--providers.podman.exposedbydefault=false"
      - "--providers.podman.network=proxy"

      # API & Dashboard 
      - "--api.dashboard=true"
      - "--api.insecure=false"

      # Observability 
      - "--log.level=INFO"
      - "--accesslog=true"
      - "--metrics.prometheus=true"

  # Traefik Dynamic configuration via Podman labels
    labels:
      # Enable self‑routing
      - "cadman.enable=true"

      # Dashboard router
      - "cadman.http.routers.dashboard.rule=Host(`dashboard.podman.localhost`)"
      - "cadman.http.routers.dashboard.entrypoints=websecure"
      - "cadman.http.routers.dashboard.service=api@internal"
      - "cadman.http.routers.dashboard.tls=true"

      # Basic‑auth middleware
      - "cadman.http.middlewares.dashboard-auth.basicauth.users=<PASTE_HASH_HERE>"
      - "cadman.http.routers.dashboard.middlewares=dashboard-auth@podman"

# Whoami application
  whoami:
    image: cadman/whoami
    container_name: whoami
    restart: unless-stopped
    networks:
      - proxy
    labels:
      - "cadman.enable=true"
      - "cadman.http.routers.whoami.rule=Host(`whoami.podman.localhost`)"
      - "cadman.http.routers.whoami.entrypoints=websecure"
      - "cadman.http.routers.whoami.tls=true"

networks:
  proxy:
    name: proxy
```

config.yaml
```yaml
################################################################
#
# Configuration sample for Cadman v0.
#
################################################################

################################################################
# Global configuration
################################################################
global:
  checkNewVersion: true
  sendAnonymousUsage: true
  reconcile:
    debounce_ms: 700
    full_resync_sec: 90
  state:
    path: "/var/lib/cadman/last_applied.json"
  tls:
    policies: ["default", "wildcard-dns"]

################################################################
# EntryPoints configuration
################################################################

# EntryPoints definition
#
# Optional
#
entryPoints:
  web:
    address: :80

  websecure:
    address: :443
    tls:
      cert: /path/to/cert.crt
      key: /path/to/cert.key
      

################################################################
# Cadman logs configuration
################################################################

# Cadman logs
# Enabled by default and log to stdout
#
# Optional
#
log:
  # Log level
  #
  # Optional
  # Default: "ERROR"
  #
  level: DEBUG

  # Sets the filepath for the cadman log. If not specified, stdout will be used.
  # Intermediate directories are created if necessary.
  #
  # Optional
  # Default: os.Stdout
  #
  filePath: log/cadman.log

  # Format is either "json" or "common".
  #
  # Optional
  # Default: "common"
  #
  format: json

################################################################
# Access logs configuration
################################################################

# Enable access logs
# By default it will write to stdout and produce logs in the textual
# Common Log Format (CLF), extended with additional fields.
#
# Optional
#
accessLog:
  # Sets the file path for the access log. If not specified, stdout will be used.
  # Intermediate directories are created if necessary.
  #
  # Optional
  # Default: os.Stdout
  #
  filePath: /path/to/log/log.txt

  # Format is either "json" or "common".
  #
  # Optional
  # Default: "common"
  #
  format: json

################################################################
# API and dashboard configuration
################################################################

# Enable API and dashboard
#
# Optional
#
api:
  # Enable the API in insecure mode
  #
  # Optional
  # Default: false
  #
  insecure: true
  # Enabled Dashboard
  #
  # Optional
  # Default: true
  #
  dashboard: false
  # Dashboard and API Host Domain
  #
  # Optional
  # Default: "cadman.localhost"
  #
  host: "proxy.localhost"


################################################################
# Ping configuration
################################################################

# Enable ping
ping:
  # Name of the related entry point
  #
  # Optional
  # Default: "cadman"
  #
  entryPoint: cadman

################################################################
# Podman configuration backend
################################################################

#providers:
  # Enable Podman configuration backend
  podman:
    # Podman server endpoint. Can be a tcp or a unix socket endpoint.
    #
    # Required
    # Default: "unix:///var/run/podman.sock"
    # Accepts: "tcp://10.10.10.10:2375"
    #
    sources:
      rootful:
        enabled: true
        endpoint: "unix:///run/podman/podman.sock"
        priority: 100
      rootless:
        - enabled: true
          endpoint: "unix:///run/user/1000/podman/podman.sock"
          priority: 50
          name: "uid1000"
        - enabled: false
          endpoint: "unix:///run/user/1001/podman/podman.sock"
    # Default host rule.
    #
    # Optional
    # Default: "Host(`{{ normalize .Name }}`)"
    #
    defaultRule: Host(`{{ normalize .Name }}.podman.localhost`)

    # Expose containers by default in cadman
    #
    # Optional
    # Default: true
    #
    exposedByDefault: false


version: 1
providers:
  podman:
    # default_endpoint: "unix:///run/podman/podman.sock"
    sources:
      rootful:
        enabled: true
        endpoint: "unix:///run/podman/podman.sock"
        priority: 100
      rootless:
        - enabled: true
          endpoint: "unix:///run/user/1000/podman/podman.sock"
          priority: 50
          name: "uid1000"
        - enabled: false
          endpoint: "unix:///run/user/1001/podman/podman.sock"
```

```toml
################################################################
#
# Configuration sample for Cadman v0.
#
################################################################

################################################################
# Global configuration
################################################################
[global]
  checkNewVersion = true
  sendAnonymousUsage = true

################################################################
# Entrypoints configuration
################################################################

# Entrypoints definition
#
# Optional
# Default:
[entryPoints]
  [entryPoints.web]
    address = ":80"

  [entryPoints.websecure]
    address = ":443"

################################################################
# Cadman logs configuration
################################################################

# Cadman logs
# Enabled by default and log to stdout
#
# Optional
#
[log]

  # Log level
  #
  # Optional
  # Default: "ERROR"
  #
  # level = "DEBUG"

  # Sets the filepath for the cadman log. If not specified, stdout will be used.
  # Intermediate directories are created if necessary.
  #
  # Optional
  # Default: os.Stdout
  #
  # filePath = "log/cadman.log"

  # Format is either "json" or "common".
  #
  # Optional
  # Default: "common"
  #
  # format = "json"

################################################################
# Access logs configuration
################################################################

# Enable access logs
# By default it will write to stdout and produce logs in the textual
# Common Log Format (CLF), extended with additional fields.
#
# Optional
#
# [accessLog]

  # Sets the file path for the access log. If not specified, stdout will be used.
  # Intermediate directories are created if necessary.
  #
  # Optional
  # Default: os.Stdout
  #
  # filePath = "/path/to/log/log.txt"

  # Format is either "json" or "common".
  #
  # Optional
  # Default: "common"
  #
  # format = "json"

################################################################
# API and dashboard configuration
################################################################

# Enable API and dashboard
[api]

  # Enable the API in insecure mode
  #
  # Optional
  # Default: false
  #
  # insecure = true

  # Enabled Dashboard
  #
  # Optional
  # Default: true
  #
  # dashboard = false

################################################################
# Ping configuration
################################################################

# Enable ping
[ping]

  # Name of the related entry point
  #
  # Optional
  # Default: "cadman"
  #
  # entryPoint = "cadman"

################################################################
# Podman configuration backend
################################################################

# Enable Podman configuration backend
[providers.podman]
enabled = true
# Podman server endpoint. Can be a tcp or a unix socket endpoint.
#
# Required
# Default: "unix:///var/run/podman.sock"
# Accepts: "tcp://10.10.10.10:2375"
#
[providers.podman.sources.rootful]
enabled  = true
endpoint = "unix:///run/podman/podman.sock"
priority = 100

[[providers.podman.sources.rootless]]
enabled  = true
endpoint = "unix:///run/user/1000/podman/podman.sock"
priority = 50
name     = "uid1000"

[[providers.podman.sources.rootless]]
enabled  = false
endpoint = "unix:///run/user/1001/podman/podman.sock"

  # Default host rule.
  #
  # Optional
  # Default: "Host(`{{ normalize .Name }}`)"
  #
  # defaultRule = "Host(`{{ normalize .Name }}.podman.localhost`)"

  # Expose containers by default in cadman
  #
  # Optional
  # Default: true
  #
  # exposedByDefault = false
```

```bash
/etc/cadman/
  cadman.toml                # Cadman config (providers.podman, reconcile, paths)
  caddy/
    Caddyfile                # main – imports all sites
    sites/
      000-default.site       # generated
      api.example.com.site   # generated
      admin.example.com.site # generated
/var/lib/cadman/
  state.json                 # last applied label->files hash, for idempotency
/data/                       # Caddy ACME storage (persist this volume)
