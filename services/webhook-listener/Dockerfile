# Stage 1: Build Stage
FROM rust:1.75-slim-bullseye as builder

WORKDIR /app
COPY . .

# Install OpenSSL and dependencies for compilation
RUN apt-get update && \
    apt-get install -y libssl-dev pkg-config ca-certificates && \
    cargo build --release

# Stage 2: Runtime Stage
FROM debian:bullseye-slim

WORKDIR /app

# Install OpenSSL (Fixes `libssl.so.3` missing error)
RUN apt-get update && \
    apt-get install -y libssl1.1 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from build stage
COPY --from=builder /app/target/release/webhook-listener /app/webhook-listener
COPY .env /app/.env

# Expose port
EXPOSE 3000

# Set environment variables (to be overridden in Docker Compose)
ENV AI_SERVICE_URL="http://ai-review:5000/review"
ENV GITHUB_TOKEN=""

# Start the Webhook Listener
CMD ["./webhook-listener"]
