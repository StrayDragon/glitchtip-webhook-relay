# =============================================================================
# Optimized Multi-stage Dockerfile for glitchtip-webhook-relay
# Target: linux/amd64 with glibc for better compatibility and faster builds
# =============================================================================

# ============================
# Build Stage - Alpine for musl compatibility
# ============================
FROM rust:1.91.1-alpine AS builder

# Install build dependencies
RUN apk add --no-cache \
    pkgconfig \
    openssl-dev \
    musl-dev

# Set environment for optimized build
ENV CARGO_PROFILE_RELEASE_LTO=true
ENV CARGO_PROFILE_RELEASE_OPT_LEVEL="z"
ENV CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1
ENV CARGO_PROFILE_RELEASE_STRIP=true

# Create non-root user
RUN adduser -D -s /bin/sh -u 1000 appuser

# Set working directory
WORKDIR /app

# Copy dependency files for better caching
COPY Cargo.toml ./

# Create dummy main.rs to cache dependencies
RUN mkdir src && echo 'fn main() {}' > src/main.rs

# Build dependencies (cached layer)
RUN cargo build --release

# Remove dummy and copy actual source
RUN rm -rf src
COPY src ./src
COPY templates ./templates

# Build the application
RUN cargo build --release

# ============================
# Runtime Stage - Alpine Linux
# ============================
FROM alpine:3.19

# Install runtime dependencies only
RUN apk add --no-cache \
    ca-certificates \
    openssl3

# Create non-root user
RUN adduser -D -s /bin/sh -u 1000 appuser

# Set working directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/glitchtip-webhook-relay /usr/local/bin/glitchtip-webhook-relay

# Switch to non-root user
USER appuser

# Entry point
ENTRYPOINT ["glitchtip-webhook-relay"]

# Expose port
EXPOSE 7876

# Metadata labels
LABEL org.opencontainers.image.title="glitchtip-webhook-relay" \
      org.opencontainers.image.description="Rust-based webhook forwarding service for GlitchTip to Feishu" \
      org.opencontainers.image.vendor="StrayDragon" \
      org.opencontainers.image.version="0.1.0" \
      org.opencontainers.image.source="https://github.com/straydragon/glitchtip-webhook-relay"
