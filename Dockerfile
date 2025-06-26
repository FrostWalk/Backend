# Build Stage
FROM rust:slim AS builder

## Install build dependencies
RUN DEBIAN_FRONTEND=noninteractive apt-get update && apt-get install -y \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy source code
COPY ./ ./

# Build the application
RUN cargo build --release

# Runtime Stage
FROM debian:stable-slim

# Install runtime dependencies
RUN DEBIAN_FRONTEND=noninteractive apt-get update && apt-get install -y \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false appuser

# Create app directory
WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/ferris-store /app/

# Change ownership to non-root user
RUN chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Expose port (adjust as needed)
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the application
CMD ["./ferris-store"]

# Volumes for storage
VOLUME ["/data"]

# Define env variables
ENV ADDRESS="127.0.0.1" \
    PORT="8080" \
    WORKERS="4" \
    DB_URL="" \
    JWT_SECRET="" \
    JWT_VALIDITY_DAYS="7" \
    LOGS_MONGO_URI="" \
    LOGS_DB_NAME="backend" \
    DEFAULT_ADMIN_PASSWORD="" \
    DEFAULT_ADMIN_EMAIL="" \
    DATA_PATH="/data"
