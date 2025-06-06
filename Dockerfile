# Build stage
FROM rust:1.86 AS builder

WORKDIR /usr/src/app
COPY . .

# Build the application with cargo
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install required dependencies for running Rust binaries and PostgreSQL client
RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates \
    postgresql-client \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app


# Copy the binary and configuration files
COPY --from=builder /usr/src/app/target/release/elmo-api /app/
COPY --from=builder /usr/src/app/.env /app/

# Set environment variables
ENV RUST_LOG=tower_http=trace,axum=trace,elmo_api=trace

# Command to run the application
CMD ["/app/elmo-api"]

EXPOSE 3000
