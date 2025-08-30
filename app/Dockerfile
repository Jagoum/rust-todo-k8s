# Use the official Rust image as base
FROM rustlang/rust:nightly

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src/ src/
COPY migrations/ migrations/

# Build the application
RUN cargo build --release

# Expose port
EXPOSE 8080

# Run the application
CMD ["./target/release/app"]
