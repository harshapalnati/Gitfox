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
COPY --from=builder /app/target/release/ai-review /app/ai-review
COPY .env /app/.env

# Expose port
EXPOSE 5000

# Set environment variables (to be overridden in Docker Compose)
ENV GITHUB_TOKEN=""
ENV OPENAI_API_KEY=""

# Start the AI Review Service
CMD ["./ai-review"]
