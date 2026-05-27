# syntax=docker/dockerfile:1
# ── Build stage ───────────────────────────────────────────────────────────────
FROM rust:1.88-slim AS builder

# Install wasm32 target for contract compilation
RUN apt-get update && apt-get install -y --no-install-recommends curl && rm -rf /var/lib/apt/lists/*
RUN rustup target add wasm32-unknown-unknown

WORKDIR /app

# Cache dependencies before copying source
COPY Cargo.toml ./
COPY contracts/ contracts/
COPY metrics/ metrics/
COPY integration-tests/ integration-tests/
COPY api-server/ api-server/

# Build all workspace members (native, for tests)
RUN cargo build --workspace 2>&1

# ── Test stage ────────────────────────────────────────────────────────────────
FROM builder AS test
CMD ["cargo", "test", "--workspace"]

# ── WASM build stage ──────────────────────────────────────────────────────────
FROM builder AS wasm
RUN cargo build --target wasm32-unknown-unknown --release \
    --package router-core \
    --package router-registry \
    --package router-access \
    --package router-middleware \
    --package router-timelock \
    --package router-multicall

# ── Metrics exporter runtime ──────────────────────────────────────────────────
FROM builder AS metrics-builder
RUN cargo build --release --package router-metrics-exporter

FROM debian:bookworm-slim AS metrics
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=metrics-builder /app/target/release/router-metrics-exporter /usr/local/bin/
EXPOSE 9090
ENTRYPOINT ["router-metrics-exporter"]
