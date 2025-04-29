# ---- Builder Stage ----
# Use the specific Rust version you need, matching your log
FROM rust:1.86.0-slim-bookworm as builder

# Install build dependencies including C/C++ compiler and make
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev \
    pkg-config libssl-dev curl \
    build-essential \
    python3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

# Copy manifests first for layer caching
COPY Cargo.toml Cargo.lock ./

# Dummy build to cache dependencies
# Ensure the binary name here matches your project/binary name
RUN mkdir src && \
    echo "fn main() {println!(\"Dummy build\");}" > src/main.rs && \
    # Build dependencies only. Specify the binary name if it's not the default crate name.
    # Use the actual binary name 'rs-kafka-mongo' consistently.
    cargo build --release --bin rs-kafka-mongo && \
    # Ensure the rm pattern matches the actual dependency artifact names
    rm -rf src target/release/deps/rs_kafka_mongo-* # Note: Rust usually uses underscores for deps

# Copy the actual source code
COPY src ./src

# Build the application
# Clear incremental artifacts if any, then build
# Specify the binary name consistently.
RUN rm -rf target/release/incremental && \
    cargo build --release --bin rs-kafka-mongo && \
    cargo build --release --bin event_consumer

# ---- Runtime Stage ----
FROM debian:bookworm-slim as runtime

# Install runtime dependencies (only libssl required by Rust binary usually)
RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl3 ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/local/bin

# Copy the built binary from the builder stage
# Ensure the source path and binary name match the build step output
COPY --from=builder /usr/src/app/target/release/rs-kafka-mongo .
COPY --from=builder /usr/src/app/target/release/event_consumer .

EXPOSE 8000

# Set environment variable to find CA certificates
ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt

# Copy the start script
COPY start.sh .

# Make it executable
RUN chmod +x start.sh

# Replace CMD to run the script
CMD ["./start.sh"]