# syntax=docker/dockerfile:1

FROM rust:1.88-bookworm AS builder

WORKDIR /app

# Build release binary
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build --release --locked && \
    cp target/release/codebox-rmcp /app/codebox-rmcp


FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Install runtime dependencies: Python + curl/CA for uv installer
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        python3 \
    && rm -rf /var/lib/apt/lists/*

# Install uv and ensure it's in PATH
RUN curl -LsSf https://astral.sh/uv/install.sh | sh
ENV PATH="/root/.local/bin:${PATH}"

# Default network settings (can be overridden by env_file/runtime env)
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=8080

# Copy compiled binary
COPY --from=builder /app/codebox-rmcp /usr/local/bin/codebox-rmcp

EXPOSE 8080

CMD ["codebox-rmcp"]
