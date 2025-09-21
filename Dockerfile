# Multi-stage build for rdfless
# Build stage
FROM debian:bookworm-slim AS builder

# Install build dependencies including rustup
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install Rust via rustup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Verify Rust installation
RUN rustc --version && cargo --version

# Create app directory
WORKDIR /usr/src/rdfless

# Copy dependency files first for better layer caching
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src/ ./src/
COPY tests/ ./tests/

# Build the application in release mode
RUN cargo build --release --bin rdfless

# Runtime stage - minimal Debian image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd --create-home --shell /bin/bash rdfless

# Copy the binary from builder stage
COPY --from=builder /usr/src/rdfless/target/release/rdfless /usr/local/bin/rdfless

# Make sure the binary is executable
RUN chmod +x /usr/local/bin/rdfless

# Copy sample files for testing (optional)
COPY samples/ /opt/rdfless/samples/

# Switch to non-root user
USER rdfless
WORKDIR /home/rdfless

# Set the entrypoint
ENTRYPOINT ["rdfless"]

# Default command shows help
CMD ["--help"]

# Metadata
LABEL org.opencontainers.image.title="rdfless" \
      org.opencontainers.image.description="A colorful pretty printer for RDF data with ANSI colors" \
      org.opencontainers.image.version="0.3.12" \
      org.opencontainers.image.authors="Lars Wilhelmsen <lars@lars-backwards.org>" \
      org.opencontainers.image.url="https://github.com/larsw/rdfless" \
      org.opencontainers.image.source="https://github.com/larsw/rdfless" \
      org.opencontainers.image.licenses="BSD-3-Clause"
