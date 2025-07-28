# Multi-stage Docker build for ABB-Klipper-Middleware

# Stage 1: Build Rust bridge
FROM rust:latest AS rust-builder

WORKDIR /usr/src/app

# Install system dependencies for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy Rust project files
COPY Cargo.toml ./
COPY src/bridge ./src/bridge

# Remove lock file and let cargo regenerate it
# This ensures compatibility with the Rust version in the container
RUN cargo generate-lockfile

# Build the Rust bridge binary
RUN cargo build --release --bin bridge

# Stage 2: Build Python converter
FROM python:3.10-slim AS python-builder

WORKDIR /usr/src/app

# Copy Python project files
COPY src/converter ./src/converter

# Install Python package
RUN pip install --no-cache-dir -e ./src/converter

# Stage 3: Runtime image
FROM python:3.10-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy built artifacts
COPY --from=rust-builder /usr/src/app/target/release/bridge ./bridge
COPY --from=python-builder /usr/local/lib/python3.10/site-packages /usr/local/lib/python3.10/site-packages
COPY --from=python-builder /usr/local/bin /usr/local/bin

# Copy configuration and source files
COPY config.toml ./config.toml
COPY src/converter ./src/converter

# Create non-root user
RUN useradd -m -s /bin/bash abbuser && \
    chown -R abbuser:abbuser /app
USER abbuser

# Create directories for logs and data
RUN mkdir -p /app/logs /app/input /app/output

# Expose bridge port
EXPOSE 1234

# Add labels for better container management
LABEL maintainer="David Seyser"
LABEL description="ABB-Klipper-Middleware Bridge Service"
LABEL version="0.1.0"

# Default command runs the bridge
CMD ["./bridge"]
