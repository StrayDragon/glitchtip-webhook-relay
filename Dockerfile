# =============================================================================
# Optimized Multi-stage Dockerfile for glitchtip-webhook-relay
# Target: linux/amd64 with glibc for better compatibility and faster builds
# =============================================================================

# ============================
# Build Stage - Debian for glibc compatibility
# ============================
FROM rust:slim-trixie AS builder

# Build argument for using Chinese mirror
ARG USE_CN_MIRROR

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set environment for optimized build
ENV CARGO_PROFILE_RELEASE_LTO=true
ENV CARGO_PROFILE_RELEASE_OPT_LEVEL="z"
ENV CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1
ENV CARGO_PROFILE_RELEASE_STRIP=true

# Create non-root user
RUN useradd -m -s /bin/bash -u 1000 appuser

# Set working directory
WORKDIR /app

# Copy dependency files
COPY Cargo.toml ./

# Copy optional cargo mirror configuration
COPY ./artifacts/docker/crate_cn_mirror.toml /tmp/crate_cn_mirror.toml

# Create minimal src structure and fetch dependencies (cached layer)
# Use Chinese mirror if USE_CN_MIRROR=1 is set
RUN mkdir src && echo 'fn main() {}' > src/main.rs && \
    if [ "$USE_CN_MIRROR" = "1" ]; then \
        mkdir -p ~/.cargo && \
        cp /tmp/crate_cn_mirror.toml ~/.cargo/config.toml && \
        echo "Using Chinese cargo mirror"; \
    else \
        echo "Using default cargo registry"; \
    fi && \
    cargo fetch

# Copy actual source files (overwrites dummy)
COPY src ./src
COPY templates ./templates
COPY config.example.yaml ./

# Build the application
RUN cargo build --release

# ============================
# Runtime Stage - Debian Slim
# ============================
FROM debian:trixie-slim

# Install runtime dependencies only
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -s /bin/bash -u 1000 appuser

# Set working directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/glitchtip-webhook-relay /usr/local/bin/glitchtip-webhook-relay

# Set default log level (can be overridden)
ENV RUST_LOG=info

# Switch to non-root user
USER appuser

# Default command
CMD ["glitchtip-webhook-relay"]

# Expose port
EXPOSE 7876

# Metadata labels
LABEL org.opencontainers.image.title="glitchtip-webhook-relay" \
      org.opencontainers.image.description="Rust-based webhook forwarding service for GlitchTip to Feishu" \
      org.opencontainers.image.vendor="StrayDragon" \
      org.opencontainers.image.version="0.1.0" \
      org.opencontainers.image.source="https://github.com/straydragon/glitchtip-webhook-relay"
