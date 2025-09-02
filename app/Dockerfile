# Multi-stage build for Rust backend
FROM rust:latest AS builder

# Install system dependencies
RUN apt-get update --allow-releaseinfo-change && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy Cargo.toml first
COPY Cargo.toml ./
COPY Cargo.lock ./



# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update --allow-releaseinfo-change && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false appuser

WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/todo-app /app/todo-app

# Change ownership
RUN chown appuser:appuser /app/todo-app

# Switch to app user
USER appuser

# Expose port
EXPOSE 8080

# Set environment variables
ENV RUST_LOG=info
ENV APP_HOST=0.0.0.0
ENV APP_PORT=8080

# Run the application
CMD ["./todo-app"]