# ---------- build stage: Caddy ----------
FROM golang:latest AS caddybuilder

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates git build-essential pkg-config curl && \
    rm -rf /var/lib/apt/lists/*

RUN go install github.com/caddyserver/xcaddy/cmd/xcaddy@latest

WORKDIR /src
RUN xcaddy build \
    --with github.com/caddy-dns/cloudflare \
    --with github.com/caddyserver/transform-encoder \
    --output /out/caddy

# ---------- build stage: Rust (cadman) ----------
FROM rust:bookworm AS rustbuilder
# If your crate is at repo root, replace paths below accordingly.

# Leverage cargo caching
WORKDIR /app
# Copy manifest files first
COPY Cargo.toml Cargo.lock* ./cadman/
# Create a dummy src to prime deps cache
RUN mkdir -p cadman/src && echo "fn main(){}" > cadman/src/main.rs
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --manifest-path cadman/Cargo.toml

# Now copy real sources and rebuild
COPY . ./cadman/
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --locked --manifest-path cadman/Cargo.toml

# Optional: strip to shrink (binutils is present in rust:bookworm)
RUN strip /app/target/release/cadman || true
RUN install -m 0755 /app/target/release/cadman /out/cadman

# ---------- runtime stage ----------
FROM debian:bookworm-slim

# (5) Minimal runtime deps
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates tzdata libcap2 && \
    rm -rf /var/lib/apt/lists/*

# (6) Create non-root user and group "cadman"
RUN groupadd --system --gid 10001 cadman && \
    useradd  --system --no-create-home \
             --uid 10001 --gid 10001 \
             --shell /usr/sbin/nologin cadman

# (7) Copy artifacts
COPY --from=builder /out/caddy /usr/bin/caddy
COPY --from=builder /out/cadman /usr/bin/cadman

# (8) Allow binding to 80/443 as non-root
RUN setcap 'cap_net_bind_service=+ep' /usr/bin/caddy

# (9) Working dir & volumes
WORKDIR /var/lib/caddy
VOLUME ["/data", "/config"]

# (10) Drop privileges to cadman
USER cadman:cadman

# (11) Healthcheck (adjust as needed)
HEALTHCHECK --interval=30s --timeout=3s --retries=3 CMD \
  wget -qO- http://127.0.0.1:2019/ping || exit 1

# (12) Entrypoint
EXPOSE 80 443
ENTRYPOINT ["/usr/bin/caddy"]
CMD ["run", "--config", "/etc/cadman/caddy/Caddyfile", "--adapter", "caddyfile"]