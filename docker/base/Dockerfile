# ---------- build stage: Caddy ----------
FROM golang:bookworm AS caddybuilder

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

WORKDIR /app

# copy manifests from workspace crate
COPY workspace/Cargo.toml workspace/Cargo.lock* ./workspace/

# prime dependency cache
RUN mkdir -p workspace/src && echo "fn main(){}" > workspace/src/main.rs

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --manifest-path workspace/Cargo.toml

# copy real source
COPY workspace ./workspace

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --locked --manifest-path workspace/Cargo.toml

RUN strip /app/target/release/cadman || true
RUN install -m 0755 /app/target/release/cadman /out/cadman

# ---------- runtime stage ----------
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates tzdata libcap2 wget && \
    rm -rf /var/lib/apt/lists/*

RUN groupadd --system --gid 10001 cadman && \
    useradd --system --no-create-home \
      --uid 10001 --gid 10001 \
      --shell /usr/sbin/nologin cadman

COPY --from=caddybuilder /out/caddy /usr/bin/caddy
COPY --from=rustbuilder /out/cadman /usr/bin/cadman

RUN setcap 'cap_net_bind_service=+ep' /usr/bin/caddy

WORKDIR /var/lib/cadman

VOLUME ["/data", "/config"]

USER cadman:cadman

EXPOSE 80 443 8080

HEALTHCHECK --interval=30s --timeout=3s --retries=3 CMD \
  wget -qO- http://127.0.0.1:2019/ping || exit 1

ENTRYPOINT ["/usr/bin/cadman"]
CMD ["serve"]