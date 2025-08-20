# Deployment

## Platforms

- **Linux**: Debian/Ubuntu, Fedora (systemd).
- **macOS**: launchd.
- **WSL**: partial planned.

## Packaging

- **Homebrew** first; **apt/dnf** later.

## Daemon Integration

- `cadman daemon start|stop|reload|status` delegates to systemd/launchd.
- Caddy reloads are **zero‑downtime** after validation.

## Logs

- Default: file logs at `~/.cadman/logs` (user scope) or `/var/log/cadman` (system scope).


Yep—that’s a solid pattern: **build a container image in GitHub Actions** and **have your self-hosted box pull/run it**. For a static Astro site this gives you reproducible builds, easy rollbacks, proper headers via Nginx, and clean deploys.

Here’s a tight, working setup.

# 1) Dockerfile (static site served by Nginx)

```dockerfile
# Build stage
FROM node:20-alpine AS build
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build  # -> dist/

# Runtime stage
FROM nginx:alpine
# Drop in a minimal nginx config with good caching/headers
COPY infra/nginx.conf /etc/nginx/nginx.conf
COPY --from=build /app/dist /usr/share/nginx/html
EXPOSE 80
HEALTHCHECK --interval=30s --timeout=3s \
  CMD wget -qO- http://127.0.0.1/healthz || exit 1
```

`infra/nginx.conf` (tweak as you like):

```nginx
worker_processes auto;
events { worker_connections 1024; }
http {
  include /etc/nginx/mime.types;
  sendfile on;

  server {
    listen 80;
    server_name _;

    root /usr/share/nginx/html;

    # Simple health endpoint
    location = /healthz { return 200 "ok\n"; add_header Content-Type text/plain; }

    # Cache static assets (adjust as needed)
    location ~* \.(js|css|png|jpg|jpeg|gif|svg|ico|webp|ttf|woff2?)$ {
      add_header Cache-Control "public, max-age=31536000, immutable";
      try_files $uri =404;
    }

    # Default: short cache for HTML
    location / {
      add_header Cache-Control "no-cache";
      try_files $uri $uri/ /index.html;
    }
  }
}
```

# 2) Build & push image in GitHub Actions (to GHCR)

Create `.github/workflows/container.yml`:

```yaml
name: Build & Push Site Image

on:
  push:
    branches: [ main ]
  workflow_dispatch:

permissions:
  contents: read
  packages: write    # needed for GHCR
  id-token: write    # if you later add provenance

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}  # org/repo
  NODE_VERSION: 20

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Extract metadata for tags
        id: meta
        uses: docker/metadata-action@v5
        with:
          images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}
          tags: |
            type=raw,value=main
            type=sha,format=long

      - name: Set up QEMU
        uses: docker/setup-qemu-action@v3

      - name: Set up Buildx
        uses: docker/setup-buildx-action@v3

      - name: Login to GHCR
        uses: docker/login-action@v3
        with:
          registry: ${{ env.REGISTRY }}
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Build and push
        uses: docker/build-push-action@v6
        with:
          context: .
          push: true
          platforms: linux/amd64
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          # optional: provenance/SBOM
          # provenance: true
          # sbom: true
```

This publishes:

* `ghcr.io/<owner>/<repo>:main`
* `ghcr.io/<owner>/<repo>:sha-<commit>`

# 3) Run it on your server (self-hosted runner or just Docker host)

## Option A: Self-hosted runner runs the update

Install a self-hosted runner on the server (non-root user). Then a second workflow updates the running container.

`.github/workflows/deploy-selfhosted.yml`:

```yaml
name: Deploy (self-hosted)

on:
  workflow_run:
    workflows: ["Build & Push Site Image"]
    types: [completed]
  workflow_dispatch:

jobs:
  deploy:
    if: ${{ github.event.workflow_run.conclusion == 'success' || github.event_name == 'workflow_dispatch' }}
    runs-on: [self-hosted, prod]   # labels on your runner
    environment: production
    permissions:
      contents: read
      packages: read
    steps:
      - name: Login to GHCR
        run: echo "${GITHUB_TOKEN}" | docker login ghcr.io -u "${GITHUB_ACTOR}" --password-stdin
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          GITHUB_ACTOR: ${{ github.actor }}

      - name: Pull latest image
        run: |
          IMAGE="ghcr.io/${GITHUB_REPOSITORY}:main"
          docker pull "$IMAGE"
          # Resolve digest for immutability
          DIGEST=$(docker inspect --format='{{index .RepoDigests 0}}' "$IMAGE")
          echo "DIGEST=$DIGEST" >> $GITHUB_ENV

      - name: Run blue/green container
        run: |
          # Parameters
          APP_NAME=astro-site
          PORT=8080

          # Start new container
          docker run -d --rm \
            --name ${APP_NAME}-next \
            -p ${PORT}:80 \
            $DIGEST

          # Health check loop
          for i in {1..20}; do
            sleep 2
            curl -fsS http://127.0.0.1:${PORT}/healthz && ok=1 && break || true
          done
          [ "${ok:-}" = "1" ] || (echo "New container unhealthy"; exit 1)

          # Swap: stop old, rename new
          if docker ps --format '{{.Names}}' | grep -q "^${APP_NAME}$"; then
            docker stop ${APP_NAME}
          fi
          docker rename ${APP_NAME}-next ${APP_NAME}
```

Put Nginx/Caddy in front (80/443) and proxy to the container’s port, or bind the container to :80 directly if it owns the host port.

## Option B: Docker Compose on the server

`/opt/astro-site/docker-compose.yml`:

```yaml
services:
  web:
    image: ghcr.io/<owner>/<repo>:main
    restart: unless-stopped
    ports:
      - "80:80"
```

Deploy step becomes:

```bash
docker login ghcr.io -u "$GITHUB_ACTOR" -p "$GITHUB_TOKEN"
docker compose pull web
docker compose up -d web
```

(You can wire that into the self-hosted deploy job instead of the blue/green script.)

# 4) Why this is nice

* **Reproducible**: “it runs on my machine” becomes “it runs in this image.”
* **Headers & caching**: handled in Nginx config (better SEO/Perf than raw Pages).
* **Rollback**: `docker run` previous digest or `compose` pin to older tag.
* **No SSH keys**: self-hosted runner pulls over HTTPS from GHCR.

# 5) Practical tips

* **Pin by digest** in prod (we resolved `:main` to a digest above).
* Protect `production` environment; don’t let PRs from forks auto-deploy.
* Add **CDN** (Cloudflare in front) for TLS, edge caching, and WAF.
* If you need Brotli, enable it in Nginx or terminate at CDN.
* Keep an eye on **logs**: `docker logs astro-site`.
* Want zero downtime with Compose? Run two services (`web_a`, `web_b`) and switch the proxy upstream on health success.

If you want, tell me your repo name/org and whether you prefer Compose or single-container, and I’ll tailor the exact workflow + compose + nginx for your box.
